//! Footage (IDL `video`, docs/design.md §2.9): `og.load.video` uploads a
//! cooked video's frames and makes the Texture that shows one of them
//! (og-render's `FrameTexture`), and loads its sound; the `og.video` system
//! (after the game's systems, before og.audio) plays every og.VideoPlayer by
//! the tick and keeps the entity's og.AudioSource in step; extraction shows
//! each video's frame (the first player's, in query order) in its texture.
//!
//! Playback is world state only: a player's time, frame and finished flag
//! are its component's fields, and a seek is read from them (a time outside
//! the frame last written), so a restored snapshot plays on exactly as the
//! world that took it did.

use super::assets;
use super::codec::Layout;
use super::engine::{Engine, engine};
use super::extract::RenderState;
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
    texture: u64,
}

type Clips = Arc<Mutex<BTreeMap<u64, Clip>>>;

thread_local! {
    /// Each engine's loaded videos, by Video handle (kept across a reload).
    static CLIPS: RefCell<BTreeMap<u32, Clips>> = const { RefCell::new(BTreeMap::new()) };
}

fn clips(engine: u32) -> Clips {
    CLIPS.with(|c| c.borrow_mut().entry(engine).or_default().clone())
}

/// A loaded video in the render state (by Video handle index).
pub(crate) struct LoadedVideo {
    pub id: String,
    pub frames: og_render::FrameTexture,
    pub info: Value,
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
            },
            move |ctx| run(ctx, &shared, &player, &audio),
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
    shared: &Mutex<BTreeMap<u64, Clip>>,
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
    for e in entities {
        let bytes = ctx.get_bytes(e, player.0)?;
        let mut v = player.1.decode(&bytes);
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
        v["texture"] = json!(c.texture as f64);
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

/// Extraction: each video's texture shows its first player's frame.
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
    let mut want: BTreeMap<usize, u32> = BTreeMap::new();
    if let Ok(all) = world.query(&q) {
        for a in all {
            for e in a.entities() {
                let Ok(b) = world.get_bytes(*e, id) else {
                    continue;
                };
                let v = layout.decode(&b);
                if let Some(i) = v["video"].as_f64().and_then(assets::index_of) {
                    want.entry(i)
                        .or_insert(v["frame"].as_u64().unwrap_or(0) as u32);
                }
            }
        }
    }
    let RenderState { videos, scene, .. } = rs;
    for (i, frame) in want {
        if let Some(v) = videos.get_mut(i) {
            v.frames.show(scene, frame as usize);
        }
    }
}

fn load_video(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
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
    let (data, pixel) = assets::texture_array_data(eng, &format!("{id}#frames"))?;
    let sound = if meta.audio == 1 {
        let i = assets::load_sound(eng, &format!("{id}#audio"))?;
        eng.assets.borrow().sound_handles[i]
    } else {
        0
    };
    let player = eng.state.borrow().layouts.get(PLAYER).cloned();
    let mut rs = eng.render.borrow_mut();
    let frames = og_render::FrameTexture::new(&mut rs.scene, data, pixel)
        .map_err(|e| format!("{id}: {e}"))?;
    rs.textures.push(frames.texture());
    let texture = assets::handle(rs.textures.len() - 1);
    let video = assets::handle(rs.videos.len());
    let gpu = frames.info();
    let info = json!({
        "video": video as f64, "texture": texture as f64, "sound": sound as f64,
        "width": meta.width, "height": meta.height, "frames": meta.frames, "fps": meta.fps,
        "seconds": meta.seconds, "format": gpu.format.name(), "bytes": gpu.bytes as f64,
    });
    rs.videos.push(LoadedVideo {
        id,
        frames,
        info: info.clone(),
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
                texture,
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
