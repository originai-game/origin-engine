//! Footage (IDL `video`, docs/design.md §2.9): `og.load.video` uploads a
//! cooked video's frames and makes the Texture that shows one of them
//! (og-render's `FrameTexture`), and loads its sound; the `og.video` system
//! (after the game's systems, before og.audio) plays every og.VideoPlayer by
//! the tick, gives each player a texture of its own (the video's first, then
//! a view of the same frames per further player: [`assign`]) and keeps the
//! entity's og.AudioSource in step; extraction shows each player's frame in
//! its texture.
//!
//! A streamed video (`stream true`, G3-2) has a stream texture per player
//! (og-render's `add_stream_texture`) instead of a clip's frames.
//! Extraction reads its units once they are fetched (`<id>#stream`, not
//! part of the startup bundle) and asks each player's decoder for the frame
//! the player is at: natively a rav1d thread per player that decodes ahead,
//! waited on so every capture shows its tick's frame; on the web the host's
//! decoder (WebCodecs or rav1d's own wasm module), never waited on. What a
//! decoder has ready is never read by the simulation.
//!
//! Playback is world state only: a player's time, frame and finished flag
//! are its component's fields, and a seek is read from them (a time outside
//! the frame last written), so a restored snapshot plays on exactly as the
//! world that took it did.

use super::assets;
use super::codec::Layout;
use super::engine::{Engine, engine};
use super::extract::RenderState;
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// The system's name.
pub(crate) const VIDEO_SYSTEM: &str = "og.video";
const PLAYER: &str = "og.VideoPlayer";
const AUDIO: &str = "og.AudioSource";

/// What the system needs of a loaded video.
#[derive(Clone, Copy, Debug)]
struct Clip {
    fps: u32,
    frames: u32,
    seconds: f32,
    sound: u64,
}

type Clips = Arc<Mutex<IdMap<u64, Clip>>>;

thread_local! {
    /// Each engine's loaded videos, by Video handle (kept across a reload).
    static CLIPS: RefCell<IdMap<u32, Clips>> = const { RefCell::new(IdMap::new()) };
}

fn clips(engine: u32) -> Clips {
    CLIPS.with(|c| c.borrow_mut().entry(engine).or_default().clone())
}

/// A loaded video in the render state (by Video handle index).
pub(crate) struct LoadedVideo {
    pub id: String,
    /// Its textures, each with its Texture handle: the first is
    /// VideoInfo.texture, the others made as further players claimed one.
    /// A texture is never dropped (a snapshot may name it).
    pub views: Vec<(u64, Frames)>,
    pub info: Value,
    /// A stream's units and where they are read from; `None` for a clip.
    pub stream: Option<StreamSource>,
}

/// What shows one player's frame.
pub(crate) enum Frames {
    /// A view of a clip's cooked frames.
    Clip(og_render::FrameTexture),
    /// A stream texture written as its decoder answers.
    Stream(StreamView),
}

/// One player's stream texture and decoder.
pub(crate) struct StreamView {
    texture: og_render::TextureHandle,
    /// Natively its rav1d thread, started once the units are read.
    #[cfg(not(target_arch = "wasm32"))]
    player: Option<og_video::stream::Player>,
    /// The frame the texture shows.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    shown: Option<u32>,
}

impl StreamView {
    fn new(scene: &mut og_render::RenderScene, s: &StreamSource) -> StreamView {
        StreamView {
            texture: scene.add_stream_texture(s.width, s.height, s.nearest),
            #[cfg(not(target_arch = "wasm32"))]
            player: None,
            shown: None,
        }
    }
}

/// A stream's units, read from its cooked file when it lands.
pub(crate) struct StreamSource {
    /// The cooked file (`<root>/cooked/...`) and its stamp.
    path: String,
    stamp: u64,
    id: String,
    width: u32,
    height: u32,
    nearest: bool,
    units: Option<Arc<og_video::stream::Units>>,
    /// Why it cannot play (a failed read, a decoder error), once.
    pub error: Option<String>,
}

impl StreamSource {
    /// Reads the units when the host has them (`false` while fetched).
    fn poll(&mut self) -> bool {
        if self.units.is_some() {
            return true;
        }
        if self.error.is_some() {
            return false;
        }
        let read = match super::engine::hooks() {
            Some(h) => h.read_asset(&self.path),
            None => super::assets::Fetch::Failed("no host is installed to read assets".into()),
        };
        match read {
            super::assets::Fetch::Pending => false,
            super::assets::Fetch::Failed(e) => {
                self.error = Some(format!("{}: {e}", self.path));
                false
            }
            super::assets::Fetch::Ready(bytes) => {
                match og_bundle::runtime::video_stream_of(&bytes, &self.path, self.stamp, &self.id)
                {
                    Ok(c) => {
                        let units = c
                            .units
                            .into_iter()
                            .enumerate()
                            .map(|(i, (key, data))| og_video::ivf::Unit {
                                pts: i as u64,
                                key,
                                data,
                            })
                            .collect();
                        self.units = Some(Arc::new(og_video::stream::Units::new(units)));
                        true
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        false
                    }
                }
            }
        }
    }
}

fn record(
    world: &World,
    layouts: &BTreeMap<String, Layout>,
    name: &str,
) -> Result<(SchemaId, Layout), String> {
    let id = world.id_by_name(name).map_err(|e| e.reason().to_string())?;
    let layout = layouts
        .get(name)
        .cloned()
        .ok_or_else(|| format!("{name} has no layout"))?;
    Ok((id, layout))
}

/// Schedules `og.video` here (after the game's systems, before og.audio).
pub(crate) fn install(
    world: &mut World,
    engine_id: u32,
    layouts: &BTreeMap<String, Layout>,
) -> Result<(), String> {
    let player = record(world, layouts, PLAYER)?;
    let audio = record(world, layouts, AUDIO)?;
    let shared = clips(engine_id);
    world
        .add_system(
            SystemDesc {
                name: VIDEO_SYSTEM.into(),
                reads: vec![PLAYER.into()],
                writes: vec![PLAYER.into(), AUDIO.into()],
                queries: vec![QueryDesc {
                    write: vec![PLAYER.into()],
                    ..QueryDesc::default()
                }],
                ..Default::default()
            },
            move |ctx| run(ctx, engine_id, &shared, &player, &audio),
        )
        .map_err(|e| e.reason().to_string())
}

fn num(v: &Value, k: &str, or: f32) -> f32 {
    v[k].as_f64().map_or(or, |x| x as f32)
}

/// The frame shown at `time` (a hair of slack: 90 ticks of an f32 dt at
/// 30 Hz add up to 2.9999998 s, which is frame 90, not 89).
fn frame_at(time: f32, c: &Clip) -> u32 {
    ((time * c.fps as f32 + 1e-3).floor().max(0.0) as u32).min(c.frames - 1)
}

fn run(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    shared: &Mutex<IdMap<u64, Clip>>,
    player: &(SchemaId, Layout),
    audio: &(SchemaId, Layout),
) -> og_core::Result<()> {
    let dt = ctx.time().dt;
    let mut entities: Vec<Entity> = Vec::new();
    for a in ctx.query(0)? {
        entities.extend_from_slice(a.entities());
    }
    let clips = shared.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let decoded: Vec<(Entity, Vec<u8>, Value)> = entities
        .into_iter()
        .map(|e| {
            let bytes = ctx.get_bytes(e, player.0)?;
            let v = player.1.decode(&bytes);
            Ok((e, bytes, v))
        })
        .collect::<og_core::Result<_>>()?;
    let textures = assign(engine_id, &decoded).map_err(bad)?;
    for ((e, bytes, mut v), texture) in decoded.into_iter().zip(textures) {
        let handle = v["video"].as_f64().unwrap_or(0.0) as u64;
        let Some(c) = clips.get(&handle).copied() else {
            continue;
        };
        let fps = c.fps as f32;
        let speed = num(&v, "speed", 1.0).clamp(0.0, 4.0);
        let playing = v["playing"].as_bool().unwrap_or(true);
        let looped = v["loop"].as_bool().unwrap_or(false);
        let start = num(&v, "time", 0.0).max(0.0);
        // A player this system has not written yet starts where it says;
        // afterwards a time outside the frame last written is a seek.
        let fresh = v["texture"].as_f64().unwrap_or(0.0) == 0.0;
        let last = v["frame"].as_f64().unwrap_or(0.0) as f32;
        let seek = !fresh && (start < last / fps - 1e-4 || start >= (last + 1.0) / fps + 1e-4);
        let mut time = start;
        if playing {
            time += dt * speed;
        }
        let mut finished = false;
        if time >= c.seconds {
            if looped && c.seconds > 0.0 {
                time %= c.seconds;
            } else {
                time = c.seconds;
                finished = true;
            }
        }
        v["time"] = json!(time);
        v["frame"] = json!(frame_at(time, &c));
        v["finished"] = json!(finished);
        v["texture"] = json!(texture as f64);
        let out = player.1.encode(Some(&bytes), Some(&v)).map_err(bad)?;
        ctx.set_bytes(e, player.0, &out)?;
        // The sound: started at the video's time, stopped for a tick on a
        // seek so it starts again there on the next.
        if c.sound == 0 {
            continue;
        }
        let Ok(ab) = ctx.get_bytes(e, audio.0) else {
            continue;
        };
        let mut s = audio.1.decode(&ab);
        s["sound"] = json!(c.sound as f64);
        s["playing"] = json!(playing && !finished && !seek && speed > 0.0);
        s["pitch"] = json!(speed.max(0.0625));
        s["loop"] = json!(looped);
        s["startOffset"] = json!(f64::from(start));
        let out = audio.1.encode(Some(&ab), Some(&s)).map_err(bad)?;
        ctx.set_bytes(e, audio.0, &out)?;
    }
    Ok(())
}

/// Each player's texture (in `players`' order; 0 for one without a loaded
/// video): the one its `texture` field names when that is a view of its
/// video no earlier player kept, else the first view nobody keeps, else a
/// new view of the video's frames. Kept textures go first, so a player that
/// has one never loses it to a newcomer earlier in the query.
fn assign(engine_id: u32, players: &[(Entity, Vec<u8>, Value)]) -> Result<Vec<u64>, String> {
    let eng = super::engine::engine(engine_id)?;
    let mut rs = eng
        .render
        .try_borrow_mut()
        .map_err(|_| "og.video: the render state is busy".to_string())?;
    let video = |v: &Value| v["video"].as_f64().and_then(assets::index_of);
    let mut out = vec![0u64; players.len()];
    let mut taken: std::collections::BTreeSet<u64> = Default::default();
    // Kept: a view of its own video, the first player to name it.
    for (k, (_, _, v)) in players.iter().enumerate() {
        let (Some(i), Some(t)) = (video(v), v["texture"].as_f64().map(|t| t as u64)) else {
            continue;
        };
        let is_view = rs
            .videos
            .get(i)
            .is_some_and(|lv| lv.views.iter().any(|(h, _)| *h == t));
        if is_view && taken.insert(t) {
            out[k] = t;
        }
    }
    // The others: a free view, or a new one.
    for (k, (_, _, v)) in players.iter().enumerate() {
        if out[k] != 0 {
            continue;
        }
        let Some(i) = video(v) else {
            continue;
        };
        let Some(lv) = rs.videos.get(i) else {
            continue;
        };
        if let Some(&(h, _)) = lv.views.iter().find(|(h, _)| !taken.contains(h)) {
            taken.insert(h);
            out[k] = h;
            continue;
        }
        let RenderState {
            videos,
            scene,
            textures,
            ..
        } = &mut *rs;
        let v = &mut videos[i];
        let view = match (&v.views[0].1, &v.stream) {
            (Frames::Clip(f), _) => Frames::Clip(f.view(scene)),
            (Frames::Stream(_), Some(s)) => Frames::Stream(StreamView::new(scene, s)),
            (Frames::Stream(_), None) => unreachable!("a stream's views have its source"),
        };
        textures.push(match &view {
            Frames::Clip(f) => f.texture(),
            Frames::Stream(sv) => sv.texture,
        });
        let h = assets::handle(textures.len() - 1);
        v.views.push((h, view));
        taken.insert(h);
        out[k] = h;
    }
    Ok(out)
}

/// Extraction: each player's texture shows its frame.
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    if rs.videos.is_empty() {
        return;
    }
    let (Some(layout), Ok(id)) = (rs.video_player.as_ref(), world.id_by_name(PLAYER)) else {
        return;
    };
    let q = QueryDesc {
        read: vec![PLAYER.into()],
        ..QueryDesc::default()
    };
    // (video index, texture handle) -> the frame its player shows, and
    // whether it loops (a stream's decoder decodes on into frame 0).
    let mut want: BTreeMap<(usize, u64), (u32, bool)> = BTreeMap::new();
    if let Ok(all) = world.query(&q) {
        for a in all {
            for e in a.entities() {
                let Ok(b) = world.get_bytes(*e, id) else {
                    continue;
                };
                let v = layout.decode(&b);
                if let (Some(i), Some(t)) = (
                    v["video"].as_f64().and_then(assets::index_of),
                    v["texture"].as_f64(),
                ) {
                    want.entry((i, t as u64)).or_insert((
                        v["frame"].as_u64().unwrap_or(0) as u32,
                        v["loop"].as_bool().unwrap_or(false),
                    ));
                }
            }
        }
    }
    let RenderState { videos, scene, .. } = rs;
    for ((i, t), (frame, looped)) in want {
        let Some(video) = videos.get_mut(i) else {
            continue;
        };
        let ready = video.stream.as_mut().is_some_and(StreamSource::poll);
        let Some((_, view)) = video.views.iter_mut().find(|(h, _)| *h == t) else {
            continue;
        };
        match view {
            Frames::Clip(f) => {
                f.show(scene, frame as usize);
            }
            Frames::Stream(sv) if ready => {
                if let Some(source) = video.stream.as_mut() {
                    show_stream(scene, source, sv, frame, looped);
                }
            }
            Frames::Stream(_) => {}
        }
    }
}

/// A stream player's frame into its texture once decoded: natively its
/// rav1d thread, started on first use, waited on (an executor's capture
/// shows its tick's frame).
#[cfg(not(target_arch = "wasm32"))]
fn show_stream(
    scene: &mut og_render::RenderScene,
    source: &mut StreamSource,
    sv: &mut StreamView,
    frame: u32,
    looped: bool,
) {
    let Some(units) = source.units.clone() else {
        return;
    };
    if sv.player.is_none() {
        match og_video::stream::Player::new(units, DECODE_THREADS) {
            Ok(p) => sv.player = Some(p),
            Err(e) => {
                source.error = Some(e);
                return;
            }
        }
    }
    let Some(player) = &sv.player else { return };
    player.want(frame, looped);
    if sv.shown == Some(frame) {
        return;
    }
    match player.take(frame, true) {
        Ok(Some(p)) => {
            if scene.write_stream_texture(sv.texture, p.rgba.clone()) {
                sv.shown = Some(frame);
            }
        }
        Ok(None) => {}
        Err(e) => source.error = Some(format!("{}: {e}", source.id)),
    }
}

/// On the web the host decodes (batch 2 of G3-2 wires WebCodecs here).
#[cfg(target_arch = "wasm32")]
fn show_stream(
    _scene: &mut og_render::RenderScene,
    _source: &mut StreamSource,
    _sv: &mut StreamView,
    _frame: u32,
    _looped: bool,
) {
}

/// rav1d's threads per player natively: a 1080p stream decodes in real
/// time on four.
#[cfg(not(target_arch = "wasm32"))]
const DECODE_THREADS: u32 = 4;

fn load_video(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(v) = eng.render.borrow().videos.iter().find(|v| v.id == id) {
        return Ok(v.info.clone());
    }
    let meta = {
        let a = eng.assets.borrow();
        let (bytes, bundle) = a.blob(&id)?;
        bundle
            .video(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let sound = if meta.audio == 1 {
        let i = assets::load_sound(eng, &format!("{id}#audio"))?;
        eng.assets.borrow().sound_handles[i]
    } else {
        0
    };
    let player = eng.state.borrow().layouts.get(PLAYER).cloned();
    let nearest = meta.kind == og_bundle::format::kind::PIXEL;
    let (view, stream, format, bytes, poster) = if meta.streamed == 1 {
        let stream_id = format!("{id}#stream");
        let (path, stamp) = {
            let a = eng.assets.borrow();
            let (bundle, dir) = a.bundle_for(&stream_id)?;
            let o = bundle.output(&stream_id).map_err(|e| e.to_string())?;
            (format!("{dir}/{}", o.file), o.stamp)
        };
        let poster = assets::handle(assets::load_texture(eng, &format!("{id}#poster"))?);
        let mut source = StreamSource {
            path,
            stamp,
            id: id.clone(),
            width: meta.width,
            height: meta.height,
            nearest,
            units: None,
            error: None,
        };
        // Natively the units are read now; on the web their fetch starts.
        source.poll();
        if let Some(e) = &source.error {
            return Err(e.clone());
        }
        let mut rs = eng.render.borrow_mut();
        let view = StreamView::new(&mut rs.scene, &source);
        let bytes = f64::from(meta.width) * f64::from(meta.height) * 4.0;
        (
            Frames::Stream(view),
            Some(source),
            "rgba8",
            bytes,
            Some(poster),
        )
    } else {
        let (data, pixel) = assets::texture_array_data(eng, &format!("{id}#frames"))?;
        let mut rs = eng.render.borrow_mut();
        let frames = og_render::FrameTexture::new(&mut rs.scene, data, pixel)
            .map_err(|e| format!("{id}: {e}"))?;
        let gpu = frames.info();
        (
            Frames::Clip(frames),
            None,
            gpu.format.name(),
            gpu.bytes as f64,
            None,
        )
    };
    let mut rs = eng.render.borrow_mut();
    rs.textures.push(match &view {
        Frames::Clip(f) => f.texture(),
        Frames::Stream(sv) => sv.texture,
    });
    let texture = assets::handle(rs.textures.len() - 1);
    let video = assets::handle(rs.videos.len());
    let info = json!({
        "video": video as f64, "texture": texture as f64, "sound": sound as f64,
        "width": meta.width, "height": meta.height, "frames": meta.frames, "fps": meta.fps,
        "seconds": meta.seconds, "format": format, "bytes": bytes,
        "streamed": stream.is_some(),
        "codec": if stream.is_some() { "av1" } else { "" },
        "poster": poster.unwrap_or(texture) as f64,
    });
    rs.videos.push(LoadedVideo {
        id,
        views: vec![(texture, view)],
        info: info.clone(),
        stream,
    });
    rs.video_player = player;
    clips(eng.id)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(
            video,
            Clip {
                fps: meta.fps,
                frames: meta.frames,
                seconds: meta.frames as f32 / meta.fps as f32,
                sound,
            },
        );
    Ok(info)
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => json!({ "ok": true, "value": v }).to_string(),
        Err(e) => json!({ "ok": false, "reason": e }).to_string(),
    }
}

/// `og.load.video`.
pub fn og_load_video(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_video(&eng, &path)))
}

/// `og.load.videoInfo`.
pub fn og_load_video_info(world: u32, video: f64) -> String {
    wire(engine(world).and_then(|eng| {
        let rs = eng.render.borrow();
        assets::index_of(video)
            .and_then(|i| rs.videos.get(i))
            .map(|v| v.info.clone())
            .ok_or_else(|| format!("video {video} was not loaded by og.load.video"))
    }))
}

/// `og.video.capabilities`.
pub fn og_video_capabilities(world: u32) -> String {
    let rendered = engine(world).is_ok_and(|eng| eng.render.borrow().backend.is_some());
    let d = super::engine::hooks()
        .map_or_else(super::engine::VideoDecoder::default, |h| h.video_decoder());
    let streamed = rendered && d.name != "none";
    json!({
        "streamed": streamed,
        "decoder": if streamed { d.name.as_str() } else { "none" },
        "hardware": streamed && d.hardware,
        "maxWidth": if streamed { d.max_width } else { 0 },
        "maxHeight": if streamed { d.max_height } else { 0 },
    })
    .to_string()
}
