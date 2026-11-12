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
use super::wire::{FromJson, VideoCapabilities, VideoInfo, record};
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// The system's name.
pub(crate) const VIDEO_SYSTEM: &str = "og.video";
pub(crate) const PLAYER: &str = "og.VideoPlayer";
/// og.VideoPlayer's schema version: 2 since G3-2 made `time` an f64
/// (super::save_migrations upgrades a version 1 save).
pub(crate) const PLAYER_VERSION: u32 = 2;
const AUDIO: &str = "og.AudioSource";
/// A document's `<video>` or `<audio>` asking its player (IDL og.MediaIntent).
pub(crate) const MEDIA_INTENT: &str = "og.MediaIntent";

/// A player's state as the last tick left it, by engine and entity, for a
/// document's `<video>` and `<audio>` (presentation: never read by the
/// simulation) with its texture's handle.
type Media = IdMap<u64, (og_ui::bind::MediaState, f64)>;

/// What the system needs of a loaded video.
#[derive(Clone, Copy, Debug)]
struct Clip {
    fps: u32,
    frames: u32,
    seconds: f64,
    sound: u64,
}

type Clips = Arc<Mutex<IdMap<u64, Clip>>>;

/// og.video on one engine: its loaded videos, by Video handle (kept across
/// a reload), and the state of the players and sounds its documents name,
/// as the last tick left them (presentation: documents read it).
#[derive(Default)]
pub(crate) struct ModuleState {
    clips: Clips,
    media: Arc<Mutex<Media>>,
}

/// The state of `player` for a document ([`og_ui::bind::Env::media`]).
pub(crate) fn media_state(engine: u32, player: u64) -> Option<(og_ui::bind::MediaState, f64)> {
    let eng = super::engine::engine(engine).ok()?;
    let media = eng
        .modules
        .video
        .media
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    media.get(&player).copied()
}

/// Sends a frame's `<video>` and `<audio>` requests as og.MediaIntent.
pub(crate) fn send_media(
    eng: &crate::engine::Engine,
    ctx: &SystemContext<'_>,
    ops: &[og_ui::bind::MediaOp],
) -> Result<(), String> {
    let Ok((layout, _)) =
        crate::api::running_record(eng, MEDIA_INTENT, crate::codec::RecordKind::Event)
    else {
        return Ok(()); // no document with one loaded
    };
    let tri = |b: Option<bool>| b.map_or(-1, i32::from);
    for op in ops {
        let v = json!({
            "player": op.player,
            "playing": tri(op.playing),
            "seek": op.seek.unwrap_or(-1.0),
            "gain": op.gain.unwrap_or(-1.0),
            "repeat": tri(op.repeat),
        });
        let bytes = layout.encode(None, Some(&v))?;
        crate::api::send_intent(eng, Some(ctx), MEDIA_INTENT, &bytes)?;
    }
    Ok(())
}

/// Applies last tick's og.MediaIntent to the players they name: a
/// video's playing, time and loop, and its sound's (or a sound's own)
/// volume, playing and loop.
fn apply_media(
    ctx: &mut SystemContext<'_>,
    layouts: &MediaLayouts,
    audio: &(SchemaId, Layout),
) -> og_core::Result<Vec<Entity>> {
    let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let mut named = Vec::new();
    let Some(intent) = &layouts.intent else {
        return Ok(named);
    };
    let Ok(id) = ctx.id_by_name(MEDIA_INTENT) else {
        return Ok(named);
    };
    let player = ctx.id_by_name(PLAYER).ok();
    for bytes in ctx.events_bytes(id)? {
        let i: record::MediaIntent = intent.read(&bytes).map_err(bad)?;
        let e = Entity::from_bits(i.player);
        let (playing, repeat, seek) = (i.playing, i.repeat, i.seek);
        let gain = f64::from(i.gain);
        named.push(e);
        let is_video = match player {
            Some(p) => match ctx.get_bytes(e, p) {
                Ok(b) => {
                    let mut v = layouts.player.decode(&b);
                    if playing >= 0 {
                        v["playing"] = json!(playing == 1);
                    }
                    if repeat >= 0 {
                        v["loop"] = json!(repeat == 1);
                    }
                    if seek >= 0.0 {
                        v["time"] = json!(seek);
                    }
                    let out = layouts.player.encode(Some(&b), Some(&v)).map_err(bad)?;
                    ctx.set_bytes(e, p, &out)?;
                    true
                }
                Err(_) => false,
            },
            None => false,
        };
        let Ok(ab) = ctx.get_bytes(e, audio.0) else {
            continue;
        };
        let mut s = audio.1.decode(&ab);
        if gain >= 0.0 {
            s["volume"] = json!(gain);
        }
        // A video's sound follows the picture (the loop below); a sound
        // of its own plays as asked.
        if !is_video {
            if playing >= 0 {
                s["playing"] = json!(playing == 1);
            }
            if repeat >= 0 {
                s["loop"] = json!(repeat == 1);
            }
        }
        let out = audio.1.encode(Some(&ab), Some(&s)).map_err(bad)?;
        ctx.set_bytes(e, audio.0, &out)?;
    }
    Ok(named)
}

/// The layouts og.video decodes.
struct MediaLayouts {
    player: Layout,
    intent: Option<Layout>,
}

fn clips(eng: &super::engine::Engine) -> Clips {
    eng.modules.video.clips.clone()
}

/// A loaded video in the render state (by Video handle index).
pub(crate) struct LoadedVideo {
    pub id: String,
    /// Its textures, each with its Texture handle: the first is
    /// VideoInfo.texture, the others made as further players claimed one.
    /// A texture is never dropped (a snapshot may name it).
    pub views: Vec<(u64, Frames)>,
    pub info: VideoInfo,
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
    /// Its Video handle: the host's key for it on the web.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    key: u64,
    /// On the web: whether the units went to the host's decoder (then they
    /// are dropped here).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    handed: bool,
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
                // Read once: a host that kept the fetched file lets it go.
                if let Some(h) = super::engine::hooks() {
                    h.release_stream(&self.path, &[]);
                }
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
/// og.VideoPlayer is lazy (super::lazy): registered, and granted to this
/// system, when a game first names it, so a world without footage keeps
/// its type table and hash; until then the system has nothing to play.
pub(crate) fn install(
    world: &mut World,
    mount: super::engine::Mount<'_>,
    layouts: &BTreeMap<String, Layout>,
) -> Result<(), String> {
    let engine_id = mount.id;
    let player = layouts
        .get(PLAYER)
        .cloned()
        .ok_or_else(|| format!("{PLAYER} has no layout"))?;
    let audio = record(world, layouts, AUDIO)?;
    let shared = mount.modules.video.clips.clone();
    let media = mount.modules.video.media.clone();
    let player = MediaLayouts {
        player,
        intent: layouts.get(MEDIA_INTENT).cloned(),
    };
    world
        .add_system(
            SystemDesc {
                name: VIDEO_SYSTEM.into(),
                writes: vec![AUDIO.into()],
                ..Default::default()
            },
            move |ctx| run(ctx, engine_id, &shared, &media, &player, &audio),
        )
        .map_err(|e| e.reason().to_string())
}

/// The frame shown at `time` (a hair of slack, so a sum of ticks that lands
/// a rounding short of a frame's start shows that frame).
fn frame_at(time: f64, c: &Clip) -> u32 {
    ((time * f64::from(c.fps) + 1e-6).floor().max(0.0) as u32).min(c.frames - 1)
}

fn run(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    shared: &Mutex<IdMap<u64, Clip>>,
    media_out: &Mutex<Media>,
    layouts: &MediaLayouts,
    audio: &(SchemaId, Layout),
) -> og_core::Result<()> {
    let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let named = apply_media(ctx, layouts, audio)?;
    let mut media = Media::new();
    // A sound a document names: whether it plays.
    for e in named {
        if let Ok(ab) = ctx.get_bytes(e, audio.0) {
            let s: record::AudioSource = audio.1.read(&ab).map_err(bad)?;
            let state = og_ui::bind::MediaState {
                playing: s.playing,
                ..Default::default()
            };
            media.insert(e.to_bits(), (state, 0.0));
        }
    }
    let publish = |media: Media| {
        let mut all = media_out.lock().unwrap_or_else(|e| e.into_inner());
        // A sound keeps its last state between the ticks it is named.
        all.retain(|_, (s, _)| s.duration == 0.0);
        all.extend(media);
    };
    let Ok(id) = ctx.id_by_name(PLAYER) else {
        publish(media);
        return Ok(());
    };
    let player = &(id, layouts.player.clone());
    // The tick's length as og.audio counts it (a whole number of ticks a
    // second), so a player's time and its sound's playhead move together.
    let dt = 1.0 / (1.0 / f64::from(ctx.time().dt)).round().max(1.0);
    let mut entities: Vec<Entity> = Vec::new();
    let players = QueryDesc {
        write: vec![PLAYER.into()],
        ..QueryDesc::default()
    };
    for a in ctx.query_with(&players)? {
        entities.extend_from_slice(a.entities());
    }
    let clips = shared.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let decoded: Vec<(Entity, Vec<u8>, Value, record::VideoPlayer)> = entities
        .into_iter()
        .map(|e| {
            let bytes = ctx.get_bytes(e, player.0)?;
            let v = player.1.decode(&bytes);
            let row = record::VideoPlayer::from_json(&v).map_err(bad)?;
            Ok((e, bytes, v, row))
        })
        .collect::<og_core::Result<_>>()?;
    let textures = assign(engine_id, decoded.iter().map(|d| &d.3)).map_err(bad)?;
    for ((e, bytes, mut v, row), texture) in decoded.into_iter().zip(textures) {
        let Some(c) = clips.get(&row.video).copied() else {
            continue;
        };
        let fps = f64::from(c.fps);
        let speed = f64::from(row.speed).clamp(0.0, 4.0);
        let playing = row.playing;
        let looped = row.r#loop;
        let start = row.time.max(0.0);
        // A player this system has not written yet starts where it says;
        // afterwards a time outside the frame last written is a seek.
        let fresh = row.texture == 0;
        let last = f64::from(row.frame);
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
        media.insert(
            e.to_bits(),
            (
                og_ui::bind::MediaState {
                    playing: playing && !finished,
                    time,
                    duration: c.seconds,
                    ended: finished,
                    texture: None,
                },
                texture as f64,
            ),
        );
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
        s["playbackRate"] = json!(speed.max(0.0625));
        s["loop"] = json!(looped);
        // The loop wraps where the picture does, never at the sound's own
        // end (an encoder's last frame may run past it).
        s["loopStart"] = json!(0.0);
        s["loopEnd"] = json!(c.seconds);
        s["startOffset"] = json!(start);
        let out = audio.1.encode(Some(&ab), Some(&s)).map_err(bad)?;
        ctx.set_bytes(e, audio.0, &out)?;
    }
    publish(media);
    Ok(())
}

/// Each player's texture (in `players`' order; 0 for one without a loaded
/// video): the one its `texture` field names when that is a view of its
/// video no earlier player kept, else the first view nobody keeps, else a
/// new view of the video's frames. Kept textures go first, so a player that
/// has one never loses it to a newcomer earlier in the query.
fn assign<'a>(
    engine_id: u32,
    players: impl Iterator<Item = &'a record::VideoPlayer> + Clone,
) -> Result<Vec<u64>, String> {
    let eng = super::engine::engine(engine_id)?;
    let mut rs = eng
        .render
        .try_borrow_mut()
        .map_err(|_| "og.video: the render state is busy".to_string())?;
    let video = |p: &record::VideoPlayer| assets::index_of(p.video as f64);
    let mut out = vec![0u64; players.clone().count()];
    let mut taken: std::collections::BTreeSet<u64> = Default::default();
    // Kept: a view of its own video, the first player to name it.
    for (k, p) in players.clone().enumerate() {
        let (Some(i), t) = (video(p), p.texture) else {
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
    for (k, p) in players.enumerate() {
        if out[k] != 0 {
            continue;
        }
        let Some(i) = video(p) else {
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
                let Ok(p) = layout.read::<record::VideoPlayer>(&b) else {
                    continue;
                };
                if let Some(i) = assets::index_of(p.video as f64) {
                    want.entry((i, p.texture)).or_insert((p.frame, p.r#loop));
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

/// On the web the host decodes (WebCodecs, or rav1d's own wasm module in
/// a Worker of its own): the units are handed over once, then each frame
/// asks for the player's frame, never waited on; a frame not decoded yet
/// keeps the one shown.
#[cfg(target_arch = "wasm32")]
fn show_stream(
    scene: &mut og_render::RenderScene,
    source: &mut StreamSource,
    sv: &mut StreamView,
    frame: u32,
    looped: bool,
) {
    let Some(hooks) = super::engine::hooks() else {
        return;
    };
    if !source.handed {
        let Some(units) = source.units.take() else {
            return;
        };
        let mut table = Vec::with_capacity(units.units.len() * 3);
        let mut data = Vec::new();
        for u in &units.units {
            table.extend_from_slice(&[data.len() as u32, u.data.len() as u32, u32::from(u.key)]);
            data.extend_from_slice(&u.data);
        }
        source.handed = true;
        // The host keeps the units; this copy is dropped.
        source.units = Some(Arc::new(og_video::stream::Units::new(Vec::new())));
        if !hooks.video_stream(source.key, (source.width, source.height), &table, &data) {
            return;
        }
    }
    let player = scene_key(sv);
    if let Some(f) = hooks.video_frame(source.key, player, frame, looped) {
        scene.write_stream_frame(sv.texture, f);
    }
}

/// A player's key for the host: its stream texture's slot.
#[cfg(target_arch = "wasm32")]
fn scene_key(sv: &StreamView) -> u64 {
    u64::from(sv.texture.slot())
}

/// rav1d's threads per player natively: a 1080p stream decodes in real
/// time on four.
#[cfg(not(target_arch = "wasm32"))]
const DECODE_THREADS: u32 = 4;

fn load_video(eng: &Engine, path: &str) -> Result<VideoInfo, String> {
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
        // The web's software fallback plays the 720p rendition the cook adds
        // beside a larger stream (og_video::stream::fallback_size).
        let software =
            super::engine::hooks().is_some_and(|h| h.video_decoder().name == "rav1d-wasm");
        let rendition = og_video::stream::fallback_size((meta.width, meta.height))
            .filter(|_| software)
            .map(|size| (format!("{id}#stream720"), size));
        let (stream_id, (width, height)) =
            rendition.unwrap_or_else(|| (format!("{id}#stream"), (meta.width, meta.height)));
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
            width,
            height,
            nearest,
            units: None,
            key: assets::handle(eng.render.borrow().videos.len()),
            handed: false,
            error: None,
        };
        // Natively the units are read now; on the web their fetch starts.
        source.poll();
        if let Some(e) = &source.error {
            return Err(e.clone());
        }
        let mut rs = eng.render.borrow_mut();
        let view = StreamView::new(&mut rs.scene, &source);
        let bytes = f64::from(width) * f64::from(height) * 4.0;
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
    let info = VideoInfo {
        video,
        texture,
        sound,
        width: meta.width,
        height: meta.height,
        frames: meta.frames,
        fps: meta.fps,
        seconds: meta.seconds,
        format: format.to_string(),
        bytes,
        streamed: stream.is_some(),
        codec: if stream.is_some() { "av1" } else { "" }.to_string(),
        poster: poster.unwrap_or(texture),
    };
    rs.videos.push(LoadedVideo {
        id,
        views: vec![(texture, view)],
        info: info.clone(),
        stream,
    });
    rs.video_player = player;
    clips(eng).lock().unwrap_or_else(|e| e.into_inner()).insert(
        video,
        Clip {
            fps: meta.fps,
            frames: meta.frames,
            seconds: f64::from(meta.frames) / f64::from(meta.fps),
            sound,
        },
    );
    Ok(info)
}

/// `og.load.video`.
pub fn og_load_video(world: u32, path: String) -> Result<crate::wire::VideoInfo, String> {
    engine(world).and_then(|eng| load_video(&eng, &path))
}

/// `og.load.videoInfo`.
pub fn og_load_video_info(world: u32, video: f64) -> Result<crate::wire::VideoInfo, String> {
    engine(world).and_then(|eng| {
        let rs = eng.render.borrow();
        assets::index_of(video)
            .and_then(|i| rs.videos.get(i))
            .map(|v| v.info.clone())
            .ok_or_else(|| format!("video {video} was not loaded by og.load.video"))
    })
}

/// `og.video.capabilities`.
pub fn og_video_capabilities(world: u32) -> crate::wire::VideoCapabilities {
    let rendered = engine(world).is_ok_and(|eng| eng.render.borrow().backend.is_some());
    let d = super::engine::hooks()
        .map_or_else(super::engine::VideoDecoder::default, |h| h.video_decoder());
    let streamed = rendered && d.name != "none";
    VideoCapabilities {
        streamed,
        decoder: if streamed { d.name.as_str() } else { "none" }.to_string(),
        hardware: streamed && d.hardware,
        max_width: if streamed { d.max_width } else { 0 },
        max_height: if streamed { d.max_height } else { 0 },
    }
}
