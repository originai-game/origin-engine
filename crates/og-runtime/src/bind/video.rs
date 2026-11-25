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
//! `og.load.video` fetches none of it: extraction reads a stream's index
//! (`<id>#stream`) once a player wants its frames, and its chunks
//! (`<id>#stream.<n>`, each from a key frame on) land through the streaming
//! queue (crate::world::stream's `fetch`), each ahead of the players that
//! need it ([`wanted_chunks`], og-video's `chunks_wanted`) and let go once
//! none does. Extraction asks each player's decoder for the frame the
//! player is at: natively a rav1d thread per player that decodes ahead,
//! its chunk read at once when it has not landed and waited on, so every
//! capture shows its tick's frame; on the web the host's decoder (WebCodecs
//! or rav1d's own wasm module), handed each chunk as it lands and never
//! waited on. What a decoder has ready is never read by the simulation: a
//! player with `wait` that is behind holds the engine's clock instead
//! (crate::present::extract's `video_stalled`), so the world ticks later.
//!
//! Playback is world state only: a player's time, frame and finished flag
//! are its component's fields, and a seek is read from them (a time outside
//! the frame last written), so a restored snapshot plays on exactly as the
//! world that took it did.

use crate::engine::{Engine, engine};
use crate::host::codec::Layout;
use crate::host::wire::{
    FromJson, VideoBuffering, VideoCapabilities, VideoInfo, VideoQuality, record,
};
use crate::present::extract::RenderState;
use crate::world::assets;
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// The system's name.
pub(crate) const VIDEO_SYSTEM: &str = "og.video";
pub(crate) const PLAYER: &str = "og.VideoPlayer";
const AUDIO: &str = "og.AudioSource";
/// A request to change a player at the tick boundary (IDL og.MediaIntent).
pub(crate) const MEDIA_INTENT: &str = "og.MediaIntent";
/// Consecutive chunk fetch failures before a stream marks itself failed and
/// unblocks `video_stalled`.
const MAX_CHUNK_FAILURES: u32 = 3;
/// Idle frames before a stream view parks its GPU storage and closes its decoder.
const IDLE_PARK_FRAMES: u32 = 2;

/// What the system needs of a loaded video.
#[derive(Clone, Debug)]
struct Clip {
    fps: u32,
    frames: u32,
    seconds: f64,
    sound: u64,
    /// Localized audio tracks (`(BCP-47 tag, sound handle)`).
    locale_sounds: Vec<(String, u64)>,
}

type Clips = Arc<Mutex<IdMap<u64, Clip>>>;

/// og.video on one engine: its loaded videos, by Video handle (kept across
/// a reload), plus presentation-only ABR quality and buffering progress.
pub(crate) struct ModuleState {
    clips: Clips,
    quality: Mutex<VideoQuality>,
    buffer_progress: Mutex<f32>,
}

impl Default for ModuleState {
    fn default() -> Self {
        Self {
            clips: Clips::default(),
            quality: Mutex::new(VideoQuality::Auto),
            buffer_progress: Mutex::new(1.0),
        }
    }
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

fn clips(eng: &crate::engine::Engine) -> Clips {
    eng.modules.video.clips.clone()
}

/// A loaded video in the render state (by Video handle index).
pub(crate) struct LoadedVideo {
    pub id: String,
    /// Cooked manifest output hash of `id` (for hot reload).
    pub hash: u64,
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
    /// Whether full-resolution GPU storage is currently allocated (false:
    /// parked as a 1x1 placeholder so idle streams take 4 bytes of VRAM).
    active: bool,
    /// Consecutive frames no player wanted this view.
    idle_frames: u32,
    /// Natively its rav1d thread, started once the units are read.
    #[cfg(not(target_arch = "wasm32"))]
    player: Option<og_video::stream::Player>,
    /// The frame the texture shows.
    shown: Option<u32>,
}

impl StreamView {
    fn new(scene: &mut og_render::RenderScene, s: &StreamSource) -> StreamView {
        StreamView {
            texture: scene.add_stream_texture_parked(s.nearest, s.alpha),
            active: false,
            idle_frames: 0,
            #[cfg(not(target_arch = "wasm32"))]
            player: None,
            shown: None,
        }
    }

    /// Releases GPU storage (back to 1x1 placeholder) and closes the decoder
    /// while preserving `self.texture` handle validity for snapshots.
    fn park(&mut self, scene: &mut og_render::RenderScene, _s: &StreamSource) {
        if self.active {
            scene.park_stream_texture(self.texture);
            self.active = false;
        }
        self.shown = None;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.player = None;
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(h) = crate::engine::hooks() {
            h.video_close(scene_key(self));
        }
    }
}

/// A stream: its index, read from its cooked file once a player wants its
/// frames, and its chunks, which land and go as players need them.
pub(crate) struct StreamSource {
    /// Base cooked video asset id (e.g. `videos/intro.ogt`).
    base_id: String,
    /// Full source resolution (width, height) for quality selection.
    base_size: (u32, u32),
    /// The index's cooked file (`<root>/cooked/...`) and its stamp.
    path: String,
    stamp: u64,
    id: String,
    width: u32,
    height: u32,
    nearest: bool,
    /// `alpha true`: each unit is packed alpha, twice `height` tall (the
    /// colour above, the matte below), folded into the frame's alpha.
    alpha: bool,
    fps: u32,
    /// Active quality tier currently selected for this stream.
    active_quality: VideoQuality,
    /// Each chunk's cooked file and stamp (`<stream id>.<n>`).
    chunks: Vec<(String, u64)>,
    /// Consecutive fetch/decode failures per chunk index.
    chunk_failures: BTreeMap<u32, u32>,
    /// The index while its rows are needed: natively to split each chunk
    /// into its units; on the web until it is handed to the host.
    index: Option<og_bundle::runtime::CookedVideoStream>,
    /// The stream's shape and its chunks here: natively their units, read
    /// by the players' decode threads; on the web a chunk handed to the
    /// host is marked with no units (the host holds them).
    units: Option<Arc<og_video::stream::Units>>,
    /// An H.264 or HEVC rendition's codec (`hevc`, `h264`) read in place
    /// of the AV1 where the web's WebCodecs decodes no AV1; `None`: AV1.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    rendition: Option<og_video::mp4::Codec>,
    /// The units' codec configuration (`config`), read with the index.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    config: Vec<u8>,
    /// Its Video handle: the host's key for it on the web.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    key: u64,
    /// On the web: whether the index went to the host's decoder.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    handed: bool,
    /// On the web: chunks that landed before the index was handed over.
    #[cfg(target_arch = "wasm32")]
    inbox: Vec<(u32, Vec<u8>)>,
    /// Why it cannot play (a failed read, a decoder error), once.
    pub error: Option<String>,
}

impl StreamSource {
    /// Reads the index when the host has it (`false` while fetched).
    fn poll(&mut self) -> bool {
        if self.units.is_some() {
            return true;
        }
        if self.error.is_some() {
            return false;
        }
        let read = match crate::engine::hooks() {
            Some(h) => h.read_asset(&self.path),
            None => {
                crate::world::assets::Fetch::Failed("no host is installed to read assets".into())
            }
        };
        match read {
            crate::world::assets::Fetch::Pending => false,
            crate::world::assets::Fetch::Failed(e) => {
                self.error = Some(format!("{}: {e}", self.path));
                false
            }
            crate::world::assets::Fetch::Ready(bytes) => {
                // Read once: a host that kept the fetched file lets it go.
                if let Some(h) = crate::engine::hooks() {
                    h.release_stream(&self.path, &[]);
                }
                match og_bundle::runtime::video_stream_of(&bytes, &self.path, self.stamp, &self.id)
                {
                    Ok(c) if c.meta.chunks as usize == self.chunks.len() => {
                        let keys = c.units.iter().map(|u| u.key != 0).collect();
                        self.units = Some(Arc::new(og_video::stream::Units::new(
                            keys,
                            c.meta.chunk_frames,
                        )));
                        self.config = c.config.clone();
                        self.index = Some(c);
                        true
                    }
                    Ok(c) => {
                        self.error = Some(format!(
                            "{}: its index names {} chunks, the bundle {} (cook again)",
                            self.path,
                            c.meta.chunks,
                            self.chunks.len()
                        ));
                        false
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        false
                    }
                }
            }
        }
    }

    /// Chunk `n` landed (its file's bytes): natively its units go to the
    /// players' decode threads; on the web to the host's decoder, once it
    /// has the index.
    fn land(&mut self, n: u32, bytes: &[u8]) -> Result<(), String> {
        let (file, stamp) = self
            .chunks
            .get(n as usize)
            .cloned()
            .ok_or_else(|| format!("{}: no chunk {n}", self.id))?;
        let chunk =
            og_bundle::runtime::video_chunk_of(bytes, &file, stamp).map_err(|e| e.to_string())?;
        if chunk.chunk != n {
            return Err(format!("{file} holds chunk {}, not {n}", chunk.chunk));
        }
        let Some(units) = self.units.clone() else {
            return Err(format!("{}: chunk {n} landed before its index", self.id));
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let index = self.index.as_ref().ok_or("the index was let go")?;
            let split = index.split(&chunk)?;
            units.put(n, split.into_iter().map(|(_, d)| d.to_vec()).collect());
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.handed {
                if let Some(h) = crate::engine::hooks() {
                    h.video_chunk(self.key, n, Some(&chunk.data));
                }
                units.put(n, Vec::new());
            } else {
                self.inbox.retain(|(k, _)| *k != n);
                self.inbox.push((n, chunk.data));
            }
        }
        Ok(())
    }

    /// Chunk `n` is let go: no player needs it now.
    fn release(&mut self, n: u32) {
        let Some(units) = &self.units else { return };
        #[cfg(target_arch = "wasm32")]
        {
            self.inbox.retain(|(k, _)| *k != n);
            if self.handed
                && units.resident(n)
                && let Some(h) = crate::engine::hooks()
            {
                h.video_chunk(self.key, n, None);
            }
        }
        units.drop_chunk(n);
    }

    /// Whether chunk `n` is here (or, on the web, waits to be handed over).
    fn has(&self, n: u32) -> bool {
        #[cfg(target_arch = "wasm32")]
        if self.inbox.iter().any(|(k, _)| *k == n) {
            return true;
        }
        self.units.as_ref().is_some_and(|u| u.resident(n))
    }
}

/// What a player of a loaded video asks of its stream: the frame it is at,
/// whether it plays (and how fast), loops and holds the clock.
#[derive(Clone, Copy, Debug)]
struct Want {
    frame: u32,
    playing: bool,
    speed: f32,
    looped: bool,
    wait: bool,
}

/// Every player's wants of the loaded videos, by video index and texture,
/// from the world (presentation: read after the ticks, never written).
fn wants(world: &World, layout: &Layout) -> BTreeMap<(usize, u64), Want> {
    let mut out = BTreeMap::new();
    let Ok(id) = world.id_by_name(PLAYER) else {
        return out;
    };
    let q = QueryDesc {
        read: vec![PLAYER.into()],
        ..QueryDesc::default()
    };
    let Ok(all) = world.query(&q) else {
        return out;
    };
    for a in all {
        for e in a.entities() {
            let Ok(b) = world.get_bytes(*e, id) else {
                continue;
            };
            let Ok(p) = layout.read::<record::VideoPlayer>(&b) else {
                continue;
            };
            if let Some(i) = assets::index_of(p.video as f64) {
                out.entry((i, p.texture)).or_insert(Want {
                    frame: p.frame,
                    playing: p.playing && !p.finished,
                    speed: p.speed,
                    looped: p.r#loop,
                    wait: p.wait,
                });
            }
        }
    }
    out
}

/// The streams' chunks the players want and have not got, the soonest
/// needed first (the streaming queue reads them), after letting go of every
/// chunk no player wants. Nothing without a renderer: nothing decodes there.
pub(crate) fn wanted_chunks(eng: &Engine) -> Vec<crate::world::stream::VideoChunkWant> {
    let Ok(world) = eng.world() else {
        return Vec::new();
    };
    let mut rs = eng.render.borrow_mut();
    if rs.videos.iter().all(|v| v.stream.is_none()) || rs.backend.is_none() {
        return Vec::new();
    }
    let Some(layout) = rs.video_player.clone() else {
        return Vec::new();
    };
    let wants = wants(world, &layout);
    // og.flow's cut targets: each one's first chunk, needed at the cut.
    let mut ahead: BTreeMap<usize, Vec<(f32, f64)>> = BTreeMap::new();
    for a in crate::bind::flow::prefetch(eng) {
        if let Some(i) = assets::index_of(a.video as f64) {
            ahead.entry(i).or_default().push((a.from, a.seconds));
        }
    }
    let mut out = Vec::new();
    let mut active_wanted: u32 = 0;
    let mut active_ready: u32 = 0;
    for (i, video) in rs.videos.iter_mut().enumerate() {
        let Some(source) = video.stream.as_mut() else {
            continue;
        };
        // A cut target no deck plays yet: its index first (a player reads
        // its own in `extract`).
        if ahead.contains_key(&i) {
            source.poll();
        }
        let Some(units) = source.units.clone() else {
            if wants.range((i, 0)..=(i, u64::MAX)).next().is_some() {
                active_wanted += 1;
            }
            continue;
        };
        let mut need: BTreeMap<u32, f64> = BTreeMap::new();
        let mut active_need: BTreeSet<u32> = BTreeSet::new();
        for w in wants.range((i, 0)..=(i, u64::MAX)).map(|(_, w)| w) {
            for (n, seconds) in og_video::stream::chunks_wanted(
                &units, source.fps, w.frame, w.playing, w.speed, w.looped,
            ) {
                active_need.insert(n);
                let s = need.entry(n).or_insert(seconds);
                *s = s.min(seconds);
            }
        }
        for n in &active_need {
            active_wanted += 1;
            if source.has(*n) {
                active_ready += 1;
            }
        }
        for &(from, seconds) in ahead.get(&i).into_iter().flatten() {
            let frame = (f64::from(from.max(0.0)) * f64::from(source.fps)).floor() as u32;
            let n = units.chunk_of(frame.min(units.len().saturating_sub(1)));
            let s = need.entry(n).or_insert(seconds);
            *s = s.min(seconds);
        }
        for n in 0..units.chunks() {
            if !need.contains_key(&n) && source.has(n) {
                source.release(n);
            }
        }
        for (n, seconds) in need {
            if source.has(n) || source.error.is_some() {
                continue;
            }
            out.push(crate::world::stream::VideoChunkWant {
                path: source.chunks[n as usize].0.clone(),
                video: i,
                stream: og_bundle::runtime::video_chunk_id(&source.id, n),
                n,
                seconds,
            });
        }
    }
    let progress = if active_wanted == 0 {
        1.0
    } else {
        (active_ready as f32 / active_wanted as f32).clamp(0.0, 1.0)
    };
    *eng.modules
        .video
        .buffer_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = progress;
    out
}

/// `dump_stream`'s `videos`: each streamed video, the stream file this host
/// plays (the AV1 or one rendition), its codec, whether its index is read,
/// its chunks and those here now, and why it cannot play.
pub(crate) fn stream_json(eng: &Engine) -> Value {
    let Ok(rs) = eng.render.try_borrow() else {
        return json!([]);
    };
    json!(
        rs.videos
            .iter()
            .filter_map(|v| {
                let s = v.stream.as_ref()?;
                let resident: Vec<u32> = (0..s.chunks.len() as u32).filter(|n| s.has(*n)).collect();
                Some(json!({
                    "video": v.id,
                    "file": s.path,
                    "codec": v.info.codec,
                    "quality": s.active_quality.as_str(),
                    "index": s.units.is_some(),
                    "chunks": s.chunks.len(),
                    "resident": resident,
                    "error": s.error,
                }))
            })
            .collect::<Vec<_>>()
    )
}

/// A chunk the streaming queue read for video `video` (by index).
pub(crate) fn deliver_chunk(eng: &Engine, video: usize, n: u32, bytes: Result<Vec<u8>, String>) {
    let mut rs = eng.render.borrow_mut();
    let Some(source) = rs.videos.get_mut(video).and_then(|v| v.stream.as_mut()) else {
        return;
    };
    // A failed read retries up to MAX_CHUNK_FAILURES times, then marks the
    // stream broken so players with `wait` unblock instead of hanging forever.
    match bytes.and_then(|b| source.land(n, &b)) {
        Ok(()) => {
            source.chunk_failures.remove(&n);
            #[cfg(not(target_arch = "wasm32"))]
            wake(&rs.videos[video]);
        }
        Err(e) => {
            let count = source.chunk_failures.entry(n).or_default();
            *count += 1;
            if *count >= MAX_CHUNK_FAILURES && source.error.is_none() {
                source.error = Some(e.clone());
            }
            drop(rs);
            let mut st = eng.state.borrow_mut();
            if st.ui_warned.insert(e.clone()) {
                st.log
                    .push(json!({ "tick": 0, "warning": true, "text": e }));
            }
        }
    }
}

/// Natively: a chunk landed, so each player's decode thread waiting for it
/// goes on.
#[cfg(not(target_arch = "wasm32"))]
fn wake(video: &LoadedVideo) {
    for (_, view) in &video.views {
        if let Frames::Stream(StreamView {
            player: Some(p), ..
        }) = view
        {
            p.wake();
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

/// Schedules `og.video` here (after og.timeline, before og.audio).
/// og.VideoPlayer is lazy (crate::engine::lazy): registered, and granted to this
/// system, when a game first names it, so a world without footage keeps
/// its type table and hash; until then the system has nothing to play.
pub(crate) fn install(
    world: &mut World,
    mount: crate::engine::Mount<'_>,
    layouts: &BTreeMap<String, Layout>,
) -> Result<(), String> {
    let engine_id = mount.id;
    let player = layouts
        .get(PLAYER)
        .cloned()
        .ok_or_else(|| format!("{PLAYER} has no layout"))?;
    let audio = record(world, layouts, AUDIO)?;
    let shared = mount.modules.video.clips.clone();
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
            move |ctx| run(ctx, engine_id, &shared, &player, &audio),
        )
        .map_err(|e| e.reason().to_string())
}

/// The frame shown at `time` (a hair of slack, so a sum of ticks that lands
/// a rounding short of a frame's start shows that frame).
fn frame_at(time: f64, c: &Clip) -> u32 {
    ((time * f64::from(c.fps) + 1e-6).floor().max(0.0) as u32).min(c.frames.saturating_sub(1))
}

/// Picks the localized audio track matching `locale` (exact BCP-47 tag first,
/// then primary language subtag), falling back to the base track.
fn sound_for_locale(c: &Clip, locale: &str) -> u64 {
    if c.locale_sounds.is_empty() || locale.is_empty() {
        return c.sound;
    }
    if let Some((_, h)) = c
        .locale_sounds
        .iter()
        .find(|(tag, _)| tag.eq_ignore_ascii_case(locale))
    {
        return *h;
    }
    if let Some(lang) = locale.split(['-', '_']).next()
        && let Some((_, h)) = c.locale_sounds.iter().find(|(tag, _)| {
            tag.split(['-', '_'])
                .next()
                .is_some_and(|l| l.eq_ignore_ascii_case(lang))
        })
    {
        return *h;
    }
    c.sound
}

fn stop_audio(
    ctx: &mut SystemContext<'_>,
    e: Entity,
    audio: &(SchemaId, Layout),
    bad: &impl Fn(String) -> og_core::Error,
) -> og_core::Result<()> {
    let Ok(ab) = ctx.get_bytes(e, audio.0) else {
        return Ok(());
    };
    let mut s = audio.1.decode(&ab);
    if s["playing"].as_bool() == Some(true) {
        s["playing"] = json!(false);
        let out = audio.1.encode(Some(&ab), Some(&s)).map_err(bad)?;
        ctx.set_bytes(e, audio.0, &out)?;
    }
    Ok(())
}

fn run(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    shared: &Mutex<IdMap<u64, Clip>>,
    layouts: &MediaLayouts,
    audio: &(SchemaId, Layout),
) -> og_core::Result<()> {
    let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    apply_media(ctx, layouts, audio)?;
    let Ok(id) = ctx.id_by_name(PLAYER) else {
        return Ok(());
    };
    let player = &(id, layouts.player.clone());
    // The tick's step as og.audio counts it (a whole number of ticks a
    // second): a player's time advances by speed / rate, the one step the
    // tick rate gives, so its playhead moves with its sound's.
    let rate = ctx.time().rate;
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
    let locale = {
        let l = crate::bind::i18n::og_i18n_locale(engine_id);
        if l.requested.is_empty() {
            l.active
        } else {
            l.requested
        }
    };
    for ((e, bytes, mut v, row), texture) in decoded.into_iter().zip(textures) {
        let Some(c) = clips.get(&row.video) else {
            if row.texture != 0 {
                v["texture"] = json!(0.0);
                let out = player.1.encode(Some(&bytes), Some(&v)).map_err(bad)?;
                ctx.set_bytes(e, player.0, &out)?;
            }
            stop_audio(ctx, e, audio, &bad)?;
            continue;
        };
        let fps = f64::from(c.fps);
        let speed = f64::from(row.speed).clamp(0.0, 4.0);
        let playing = row.playing;
        let looped = row.r#loop;
        let has_sub_loop = looped && row.loop_end > row.loop_start && row.loop_start >= 0.0;
        let loop_start = if has_sub_loop {
            row.loop_start.min(c.seconds)
        } else {
            0.0
        };
        let loop_end = if has_sub_loop {
            row.loop_end.min(c.seconds)
        } else {
            c.seconds
        };
        let start = row.time.max(0.0);
        // A player this system has not written yet starts where it says;
        // afterwards a time outside the frame last written is a seek.
        let fresh = row.texture == 0;
        let last = f64::from(row.frame);
        let seek = !fresh && (start < last / fps - 1e-4 || start >= (last + 1.0) / fps + 1e-4);
        let mut time = start;
        if playing {
            time += speed / f64::from(rate);
        }
        let mut finished = false;
        if has_sub_loop && loop_end > loop_start {
            if time >= loop_end {
                let span = loop_end - loop_start;
                time = loop_start + (time - loop_start).rem_euclid(span);
            }
        } else if time >= c.seconds {
            if looped && c.seconds > 0.0 {
                time %= c.seconds;
            } else {
                time = c.seconds;
                finished = true;
            }
        }
        v["time"] = json!(time);
        v["frame"] = json!(frame_at(time, c));
        v["finished"] = json!(finished);
        v["texture"] = json!(texture as f64);
        let out = player.1.encode(Some(&bytes), Some(&v)).map_err(bad)?;
        ctx.set_bytes(e, player.0, &out)?;
        // The sound: started at the video's time, stopped for a tick on a
        // seek so it starts again there on the next. A silent clip stops any
        // sound left over from an earlier video on the same entity.
        let sound = sound_for_locale(c, &locale);
        if sound == 0 {
            stop_audio(ctx, e, audio, &bad)?;
            continue;
        }
        let Ok(ab) = ctx.get_bytes(e, audio.0) else {
            continue;
        };
        let mut s = audio.1.decode(&ab);
        s["sound"] = json!(sound as f64);
        s["playing"] = json!(playing && !finished && !seek && speed > 0.0);
        s["playbackRate"] = json!(speed.max(0.0625));
        s["loop"] = json!(looped);
        // The loop wraps where the picture does, never at the sound's own
        // end (an encoder's last frame may run past it).
        s["loopStart"] = json!(loop_start);
        s["loopEnd"] = json!(loop_end);
        s["startOffset"] = json!(start);
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
fn assign<'a>(
    engine_id: u32,
    players: impl Iterator<Item = &'a record::VideoPlayer> + Clone,
) -> Result<Vec<u64>, String> {
    let eng = crate::engine::engine(engine_id)?;
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

/// Extraction: each player's texture shows its frame. A player with
/// `wait` whose frame is not shown yet holds the clock (`video_stalled`).
/// Idle stream views park their GPU texture and close their decoder after
/// `IDLE_PARK_FRAMES` frames so long-form interactive films keep bounded VRAM.
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    rs.video_stalled = false;
    if rs.videos.is_empty() {
        return;
    }
    let Some(layout) = rs.video_player.as_ref() else {
        return;
    };
    let want = wants(world, layout);
    let wanted_keys: BTreeSet<(usize, u64)> = want.keys().copied().collect();
    let mut stalled = false;
    let RenderState { videos, scene, .. } = rs;
    for ((i, t), w) in want {
        let Some(video) = videos.get_mut(i) else {
            continue;
        };
        let ready = video.stream.as_mut().is_some_and(StreamSource::poll);
        let Some((_, view)) = video.views.iter_mut().find(|(h, _)| *h == t) else {
            continue;
        };
        let shown = match view {
            Frames::Clip(f) => {
                f.show(scene, w.frame as usize);
                true
            }
            Frames::Stream(sv) => {
                sv.idle_frames = 0;
                if !sv.active
                    && let Some(source) = video.stream.as_ref()
                {
                    scene.activate_stream_texture(
                        sv.texture,
                        source.width,
                        source.height,
                        source.nearest,
                        source.alpha,
                    );
                    sv.active = true;
                }
                if ready {
                    match video.stream.as_mut() {
                        Some(source) => show_stream(scene, source, sv, w.frame, w.looped),
                        None => true,
                    }
                } else {
                    false
                }
            }
        };
        // A stream that cannot play (a failed read, no decoder) never holds it.
        let broken = video.stream.as_ref().is_some_and(|s| s.error.is_some());
        stalled |= w.wait && w.playing && w.speed > 0.0 && !shown && !broken;
    }
    for (i, video) in videos.iter_mut().enumerate() {
        let Some(source) = video.stream.as_ref() else {
            continue;
        };
        for (h, view) in &mut video.views {
            if wanted_keys.contains(&(i, *h)) {
                continue;
            }
            if let Frames::Stream(sv) = view {
                #[cfg(not(target_arch = "wasm32"))]
                let open = sv.active || sv.player.is_some();
                #[cfg(target_arch = "wasm32")]
                let open = sv.active;
                if open {
                    sv.idle_frames = sv.idle_frames.saturating_add(1);
                    if sv.idle_frames > IDLE_PARK_FRAMES {
                        sv.park(scene, source);
                    }
                }
            }
        }
    }
    rs.video_stalled = stalled;
}

/// A stream player's frame into its texture once decoded: natively its
/// rav1d thread, started on first use, its chunk read at once when it has
/// not landed, and waited on (an executor's capture shows its tick's
/// frame). Whether the texture shows `frame` now.
#[cfg(not(target_arch = "wasm32"))]
fn show_stream(
    scene: &mut og_render::RenderScene,
    source: &mut StreamSource,
    sv: &mut StreamView,
    frame: u32,
    looped: bool,
) -> bool {
    let Some(units) = source.units.clone() else {
        return false;
    };
    let frame = frame.min(units.len().saturating_sub(1));
    let n = units.chunk_of(frame);
    if !units.resident(n) {
        let (file, _) = source.chunks[n as usize].clone();
        let read = crate::engine::hooks().map(|h| h.read_asset(&file));
        let landed = match read {
            Some(crate::world::assets::Fetch::Ready(bytes)) => source.land(n, &bytes),
            Some(crate::world::assets::Fetch::Failed(e)) => Err(format!("{file}: {e}")),
            Some(crate::world::assets::Fetch::Pending) | None => {
                Err(format!("{file}: this host cannot read it now"))
            }
        };
        if let Err(e) = landed {
            source.error = Some(e);
            return false;
        }
        if let Some(p) = &sv.player {
            p.wake();
        }
    }
    if sv.player.is_none() {
        match og_video::stream::Player::new(units, DECODE_THREADS, source.alpha) {
            Ok(p) => sv.player = Some(p),
            Err(e) => {
                source.error = Some(e);
                return false;
            }
        }
    }
    let Some(player) = &sv.player else {
        return false;
    };
    player.want(frame, looped);
    if sv.shown == Some(frame) {
        return true;
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
    sv.shown == Some(frame)
}

/// On the web the host decodes (WebCodecs, or rav1d's own wasm module in
/// a Worker of its own): the index is handed over once, then each chunk as
/// it lands, then each frame asks for the player's frame, never waited
/// on; a frame not decoded yet keeps the one shown. Whether the texture
/// shows `frame` now.
#[cfg(target_arch = "wasm32")]
fn show_stream(
    scene: &mut og_render::RenderScene,
    source: &mut StreamSource,
    sv: &mut StreamView,
    frame: u32,
    looped: bool,
) -> bool {
    let Some(hooks) = crate::engine::hooks() else {
        return false;
    };
    if !source.handed {
        let Some(index) = source.index.take() else {
            return false;
        };
        let mut table = Vec::with_capacity(index.units.len() * 3);
        for u in &index.units {
            table.extend_from_slice(&[u.offset, u.size, u.key]);
        }
        source.handed = true;
        let codec = match source.rendition {
            Some(c) => match og_video::mp4::codec_string(c, &source.config) {
                Ok(name) => name,
                Err(e) => {
                    source.error = Some(format!("{}: {e}", source.id));
                    return false;
                }
            },
            None => String::new(),
        };
        let config: &[u8] = if source.rendition.is_some() {
            &source.config
        } else {
            &[]
        };
        if !hooks.video_stream(
            source.key,
            (
                source.width,
                source.height * if source.alpha { 2 } else { 1 },
            ),
            &codec,
            config,
            &table,
            index.meta.chunk_frames,
        ) {
            source.error = Some(format!(
                "{}: this browser decodes none of its codecs",
                source.id
            ));
            return false;
        }
        // The chunks that landed meanwhile follow the index.
        for (n, data) in std::mem::take(&mut source.inbox) {
            hooks.video_chunk(source.key, n, Some(&data));
            if let Some(u) = &source.units {
                u.put(n, Vec::new());
            }
        }
    }
    let player = scene_key(sv);
    if let Some(f) = hooks.video_frame(source.key, player, frame, looped) {
        sv.shown = Some(f.timestamp() as u32);
        scene.write_stream_frame(sv.texture, f);
    } else if let Some(err) = hooks.video_error(source.key, player) {
        source.error = Some(format!("{}: {err}", source.id));
        return false;
    }
    sv.shown == Some(frame)
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

fn has_output(eng: &Engine, id: &str) -> bool {
    let a = eng.assets.borrow();
    a.bundle_for(id).is_ok_and(|(b, _)| b.output(id).is_ok())
}

/// Picks the cooked stream rendition for `base_id` given the host's decoder
/// capabilities and the player's quality preference.
fn select_rendition(
    eng: &Engine,
    base_id: &str,
    base_size: (u32, u32),
    pref: VideoQuality,
) -> (
    String,
    (u32, u32),
    Option<og_video::mp4::Codec>,
    &'static str,
    VideoQuality,
) {
    let decoder = crate::engine::hooks()
        .map_or_else(crate::engine::VideoDecoder::default, |h| h.video_decoder());
    let stream_high = format!("{base_id}#stream");
    let stream_med = format!("{base_id}#stream720");
    let stream_low = format!("{base_id}#stream480");
    let has_med = has_output(eng, &stream_med);
    let has_low = has_output(eng, &stream_low);
    let med_size = og_video::stream::fallback_size(base_size).unwrap_or(base_size);
    let low_size = og_video::stream::low_size(base_size).unwrap_or(med_size);

    let pick_high = || -> (String, (u32, u32), Option<og_video::mp4::Codec>, &'static str, VideoQuality) {
        if !matches!(decoder.name.as_str(), "rav1d" | "webcodecs") {
            for c in &decoder.codecs {
                let kind = match c.as_str() {
                    "hevc" => og_video::mp4::Codec::Hevc,
                    "h264" => og_video::mp4::Codec::H264,
                    _ => continue,
                };
                let s = format!("{base_id}#stream{c}");
                if has_output(eng, &s) {
                    let word = if c == "hevc" { "hevc" } else { "h264" };
                    return (s, base_size, Some(kind), word, VideoQuality::High);
                }
            }
        }
        (stream_high.clone(), base_size, None, "av1", VideoQuality::High)
    };

    match pref {
        VideoQuality::Low => {
            if has_low {
                (stream_low, low_size, None, "av1", VideoQuality::Low)
            } else if has_med {
                (stream_med, med_size, None, "av1", VideoQuality::Medium)
            } else {
                pick_high()
            }
        }
        VideoQuality::Medium => {
            if has_med {
                (stream_med, med_size, None, "av1", VideoQuality::Medium)
            } else {
                pick_high()
            }
        }
        VideoQuality::High => pick_high(),
        VideoQuality::Auto => {
            let high = pick_high();
            if high.2.is_some() {
                return high;
            }
            if decoder.name == "rav1d-wasm"
                && has_med
                && og_video::stream::fallback_size(base_size).is_some()
            {
                (stream_med, med_size, None, "av1", VideoQuality::Medium)
            } else {
                high
            }
        }
    }
}

type StreamManifest = (String, u64, Vec<(String, u64)>);

fn read_stream_manifest(eng: &Engine, stream_id: &str) -> Result<StreamManifest, String> {
    let a = eng.assets.borrow();
    let (bundle, dir) = a.bundle_for(stream_id)?;
    let o = bundle.output(stream_id).map_err(|e| e.to_string())?;
    let mut chunks = Vec::new();
    while let Ok(c) = bundle.output(&og_bundle::runtime::video_chunk_id(
        stream_id,
        chunks.len() as u32,
    )) {
        chunks.push((format!("{dir}/{}", c.file), c.stamp));
    }
    if chunks.is_empty() {
        return Err(format!("{stream_id} has no chunks: cook it again"));
    }
    Ok((format!("{dir}/{}", o.file), o.stamp, chunks))
}

fn available_qualities(eng: &Engine, base_id: &str) -> Vec<String> {
    let mut out = Vec::new();
    if has_output(eng, &format!("{base_id}#stream"))
        || has_output(eng, &format!("{base_id}#streamhevc"))
        || has_output(eng, &format!("{base_id}#streamh264"))
    {
        out.push("high".to_string());
    }
    if has_output(eng, &format!("{base_id}#stream720")) {
        out.push("medium".to_string());
    }
    if has_output(eng, &format!("{base_id}#stream480")) {
        out.push("low".to_string());
    }
    out
}

fn discover_audio_locales(eng: &Engine, id: &str) -> Result<Vec<(String, u64)>, String> {
    let prefix = format!("{id}#audio.");
    let mut tags: Vec<String> = {
        let a = eng.assets.borrow();
        let Ok((bundle, _)) = a.bundle_for(id) else {
            return Ok(Vec::new());
        };
        bundle
            .manifest()
            .outputs()
            .filter(|o| o.ty == "og.Sound")
            .filter_map(|o| o.id.strip_prefix(&prefix))
            .filter(|tag| !tag.contains('#') && !tag.is_empty())
            .map(str::to_string)
            .collect()
    };
    tags.sort();
    tags.dedup();
    let mut out = Vec::with_capacity(tags.len());
    for tag in tags {
        let i = assets::load_sound(eng, &format!("{prefix}{tag}"))?;
        let handle = eng.assets.borrow().sound_handles[i];
        out.push((tag, handle));
    }
    Ok(out)
}

fn output_hash(eng: &Engine, id: &str) -> u64 {
    let a = eng.assets.borrow();
    a.bundle_for(id)
        .ok()
        .and_then(|(b, _)| b.output(id).ok().map(|o| o.hash))
        .unwrap_or(0)
}

fn load_video(eng: &Engine, path: &str) -> Result<VideoInfo, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(v) = eng.render.borrow().videos.iter().find(|v| v.id == id) {
        return Ok(v.info.clone());
    }
    let hash = output_hash(eng, &id);
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
    let locale_sounds = discover_audio_locales(eng, &id)?;
    let audio_locales: Vec<String> = locale_sounds.iter().map(|(t, _)| t.clone()).collect();
    let player = eng.state.borrow().layouts.get(PLAYER).cloned();
    let nearest = meta.kind == og_bundle::format::kind::PIXEL;
    let pref = *eng
        .modules
        .video
        .quality
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut codec = "av1";
    let qualities = if meta.streamed == 1 {
        available_qualities(eng, &id)
    } else {
        Vec::new()
    };
    let (view, stream, format, bytes, poster) = if meta.streamed == 1 {
        let (stream_id, (width, height), rendition, chosen_codec, active_quality) =
            select_rendition(eng, &id, (meta.width, meta.height), pref);
        codec = chosen_codec;
        let (path, stamp, chunks) = read_stream_manifest(eng, &stream_id)?;
        let poster = assets::handle(assets::load_texture(eng, &format!("{id}#poster"))?);
        // Nothing is fetched until a player wants its frames, and GPU storage
        // stays parked (4 bytes) until a player activates it in `extract`.
        let source = StreamSource {
            base_id: id.clone(),
            base_size: (meta.width, meta.height),
            path,
            stamp,
            id: stream_id,
            width,
            height,
            nearest,
            alpha: meta.alpha == 1,
            fps: meta.fps,
            active_quality,
            chunks,
            chunk_failures: BTreeMap::new(),
            index: None,
            units: None,
            rendition,
            config: Vec::new(),
            key: assets::handle(eng.render.borrow().videos.len()),
            handed: false,
            #[cfg(target_arch = "wasm32")]
            inbox: Vec::new(),
            error: None,
        };
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
        ai_generated: meta.ai_generated == 1,
        alpha: meta.alpha == 1,
        codec: if stream.is_some() { codec } else { "" }.to_string(),
        poster: poster.unwrap_or(texture),
        qualities,
        audio_locales,
    };
    rs.videos.push(LoadedVideo {
        id,
        hash,
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
            locale_sounds,
        },
    );
    Ok(info)
}

/// Switches a loaded stream to a new rendition (for ABR `setQuality` or hot reload).
fn switch_stream(
    eng: &Engine,
    scene: &mut og_render::RenderScene,
    video: &mut LoadedVideo,
    pref: VideoQuality,
    force: bool,
) -> Result<(), String> {
    let Some(source) = video.stream.as_mut() else {
        return Ok(());
    };
    let (stream_id, (width, height), rendition, codec, active_quality) =
        select_rendition(eng, &source.base_id, source.base_size, pref);
    if !force && source.id == stream_id && source.error.is_none() {
        return Ok(());
    }
    let (path, stamp, chunks) = read_stream_manifest(eng, &stream_id)?;
    for n in 0..source.chunks.len() as u32 {
        source.release(n);
    }
    for (_, view) in &mut video.views {
        if let Frames::Stream(sv) = view {
            sv.park(scene, source);
        }
    }
    source.path = path;
    source.stamp = stamp;
    source.id = stream_id;
    source.width = width;
    source.height = height;
    source.rendition = rendition;
    source.active_quality = active_quality;
    source.chunks = chunks;
    source.chunk_failures.clear();
    source.index = None;
    source.units = None;
    source.config.clear();
    source.handed = false;
    #[cfg(target_arch = "wasm32")]
    source.inbox.clear();
    source.error = None;
    video.info.codec = codec.to_string();
    video.info.bytes = f64::from(width) * f64::from(height) * 4.0;
    Ok(())
}

/// After a hot reload: refreshes every loaded streamed video whose cooked
/// output hash changed.
pub(crate) fn refresh(eng: &Engine) -> Vec<String> {
    let pref = *eng
        .modules
        .video
        .quality
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut failed = Vec::new();
    let count = eng.render.borrow().videos.len();
    for i in 0..count {
        let (id, old_hash, streamed) = {
            let rs = eng.render.borrow();
            let v = &rs.videos[i];
            (v.id.clone(), v.hash, v.stream.is_some())
        };
        let now_hash = output_hash(eng, &id);
        if now_hash == 0 || now_hash == old_hash {
            continue;
        }
        if !streamed {
            continue;
        }
        let meta = {
            let a = eng.assets.borrow();
            match a.blob(&id).and_then(|(bytes, bundle)| {
                bundle.video(&id, bytes.bytes()).map_err(|e| e.to_string())
            }) {
                Ok(m) => m,
                Err(e) => {
                    failed.push(format!("video {id}: {e}"));
                    continue;
                }
            }
        };
        let qualities = available_qualities(eng, &id);
        let mut rs = eng.render.borrow_mut();
        let RenderState { videos, scene, .. } = &mut *rs;
        let v = &mut videos[i];
        v.hash = now_hash;
        v.info.width = meta.width;
        v.info.height = meta.height;
        v.info.frames = meta.frames;
        v.info.fps = meta.fps;
        v.info.seconds = meta.seconds;
        v.info.qualities = qualities;
        if let Some(s) = v.stream.as_mut() {
            s.base_size = (meta.width, meta.height);
            s.fps = meta.fps;
        }
        if let Err(e) = switch_stream(eng, scene, v, pref, true) {
            failed.push(format!("video {id}: {e}"));
        }
        let video_handle = v.info.video;
        drop(rs);
        if let Some(c) = clips(eng)
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&video_handle)
        {
            c.fps = meta.fps;
            c.frames = meta.frames;
            c.seconds = f64::from(meta.frames) / f64::from(meta.fps);
        }
    }
    failed
}

/// Info of a loaded video handle (for og.flow view dimensions & chart posters).
pub(crate) fn info_of(eng: &Engine, video: u64) -> Option<VideoInfo> {
    let rs = eng.render.try_borrow().ok()?;
    assets::index_of(video as f64)
        .and_then(|i| rs.videos.get(i))
        .map(|v| v.info.clone())
}

/// Current presentation buffering status `(buffering, progress)` for `og.flow`.
pub(crate) fn buffering_state(eng: &Engine) -> (bool, f32) {
    let buffering = eng.render.try_borrow().is_ok_and(|rs| rs.video_stalled);
    let progress = *eng
        .modules
        .video
        .buffer_progress
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    (buffering, if buffering { progress } else { 1.0 })
}

/// A video's og.VideoPlayer.video handle, loaded as og.load.video does (a
/// timeline's video track).
pub(crate) fn load_handle(eng: &Engine, path: &str) -> Result<u64, String> {
    load_video(eng, path).map(|i| i.video)
}

/// `og.load.video`.
pub fn og_load_video(world: u32, path: String) -> Result<crate::host::wire::VideoInfo, String> {
    engine(world).and_then(|eng| load_video(&eng, &path))
}

/// `og.load.videoInfo`.
pub fn og_load_video_info(world: u32, video: f64) -> Result<crate::host::wire::VideoInfo, String> {
    engine(world).and_then(|eng| {
        let rs = eng.render.borrow();
        assets::index_of(video)
            .and_then(|i| rs.videos.get(i))
            .map(|v| v.info.clone())
            .ok_or_else(|| format!("video {video} was not loaded by og.load.video"))
    })
}

/// `og.video.capabilities`.
pub fn og_video_capabilities(world: u32) -> crate::host::wire::VideoCapabilities {
    let rendered = engine(world).is_ok_and(|eng| eng.render.borrow().backend.is_some());
    let d = crate::engine::hooks()
        .map_or_else(crate::engine::VideoDecoder::default, |h| h.video_decoder());
    let streamed = rendered && d.name != "none";
    // AV1 first where it has a decoder of its own, last where only software.
    let mut codecs = Vec::new();
    if streamed {
        let others = d.codecs.iter().cloned();
        if d.name == "rav1d-wasm" {
            codecs.extend(others);
            codecs.push("av1".to_string());
        } else {
            codecs.push("av1".to_string());
            codecs.extend(others);
        }
    }
    VideoCapabilities {
        streamed,
        decoder: if streamed { d.name.as_str() } else { "none" }.to_string(),
        hardware: streamed && d.hardware,
        max_width: if streamed { d.max_width } else { 0 },
        max_height: if streamed { d.max_height } else { 0 },
        codecs,
    }
}

/// `og.video.setQuality`: presentation-only quality switch for streamed videos.
pub fn og_video_set_quality(world: u32, quality: String) -> Result<(), String> {
    let q = VideoQuality::parse(&quality)?;
    let eng = engine(world)?;
    *eng.modules
        .video
        .quality
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = q;
    let mut rs = eng.render.borrow_mut();
    let RenderState { videos, scene, .. } = &mut *rs;
    for v in videos.iter_mut() {
        switch_stream(&eng, scene, v, q, false)?;
    }
    Ok(())
}

/// `og.video.buffering`: presentation-only buffering state.
pub fn og_video_buffering(world: u32) -> VideoBuffering {
    let Ok(eng) = engine(world) else {
        return VideoBuffering {
            buffering: false,
            progress: 1.0,
            quality: VideoQuality::Auto,
        };
    };
    let (buffering, progress) = buffering_state(&eng);
    let quality = *eng
        .modules
        .video
        .quality
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    VideoBuffering {
        buffering,
        progress,
        quality,
    }
}
