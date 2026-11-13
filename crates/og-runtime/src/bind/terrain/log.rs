//! The edit log, its streaming, and its place in saves and checkpoints.

use super::*;

/// The edits made since the world began, in order: (terrain asset id, the
/// brush as JSON). `replay`: a restore changed them, and the terrains must
/// be rebuilt from their cook.
#[derive(Default)]
pub(crate) struct EditLog {
    pub edits: Vec<(String, String)>,
    pub replay: bool,
}

pub(crate) type SharedLog = Arc<std::sync::Mutex<EditLog>>;

/// og.terrain on one engine: its edit log (its worlds' og.terrain blocks
/// share it).
#[derive(Default)]
pub(crate) struct ModuleState(SharedLog);

/// An engine's edit log.
pub(crate) fn log_of(eng: &Engine) -> SharedLog {
    eng.modules.terrain.0.clone()
}

/// The probe's `terrain_stream:<stream|resident>[:<MiB>]`: `resident`
/// decodes and shows every tile the view wants each frame, `stream` (the
/// default) streams them; the MiB are each terrain's tile budget on the GPU
/// (0 or none: its lod's).
pub(crate) fn set_streaming(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write terrain_stream:<stream|resident>[:<MiB>], such as terrain_stream:resident or terrain_stream:stream:4";
    let resident = match args.first().map(String::as_str) {
        Some("resident") => true,
        Some("stream") => false,
        _ => return Err(USAGE.into()),
    };
    let budget = match args.get(1).filter(|a| !a.is_empty()) {
        None => 0,
        Some(m) => match m.parse::<f64>() {
            Ok(mib) if mib.is_finite() && mib >= 0.0 => (mib * 1048576.0) as u64,
            _ => {
                return Err(format!(
                    "terrain_stream: '{m}' is not a number of MiB; {USAGE}"
                ));
            }
        },
    };
    let eng = engine(engine_id)?;
    let mut rs = eng
        .render
        .try_borrow_mut()
        .map_err(|_| "terrain_stream: the renderer is busy".to_string())?;
    rs.terrain.resident = resident;
    rs.terrain.budget = budget;
    Ok(())
}

/// The terrain streamer for `dump_stream`: its mode, the tile budget asked
/// for (0: each terrain's own), and its last frame's tiles.
pub(crate) fn stream_json(engine_id: u32) -> Value {
    let Ok(eng) = engine(engine_id) else {
        return Value::Null;
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return Value::Null;
    };
    let t = &rs.terrain;
    let s = t.streamer.stats;
    json!({
        "mode": if t.resident { "resident" } else { "stream" },
        "budget": t.budget as f64,
        "terrains": t.loaded.len(),
        "resident": s.resident,
        "waiting": s.waiting,
        "inFlight": s.in_flight,
        "cached": s.cached,
        "cachedBytes": s.cached_bytes as f64,
        "decoded": s.decoded as f64,
        "handed": s.handed,
        "worstFrameMs": (s.worst_frame_ms * 1000.0).round() / 1000.0,
    })
}

/// The edit log as world state.
struct TerrainBlock(SharedLog);

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn take_str(bytes: &[u8], at: &mut usize) -> Option<String> {
    let n = u32::from_le_bytes(bytes.get(*at..*at + 4)?.try_into().ok()?) as usize;
    let s = std::str::from_utf8(bytes.get(*at + 4..*at + 4 + n)?).ok()?;
    *at += 4 + n;
    Some(s.to_string())
}

impl og_ecs::StateBlock for TerrainBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let log = self.0.lock().expect("the edit log");
        out.extend_from_slice(&(log.edits.len() as u32).to_le_bytes());
        for (id, brush) in &log.edits {
            put_str(out, id);
            put_str(out, brush);
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn std::any::Any + Send>> {
        let bad = || og_core::Error::new(og_core::ErrorKind::Parse, "a damaged terrain edit log");
        let n = u32::from_le_bytes(
            bytes
                .get(..4)
                .ok_or_else(bad)?
                .try_into()
                .map_err(|_| bad())?,
        );
        let mut at = 4;
        let mut edits = Vec::with_capacity(n.min(1 << 16) as usize);
        for _ in 0..n {
            let id = take_str(bytes, &mut at).ok_or_else(bad)?;
            let brush = take_str(bytes, &mut at).ok_or_else(bad)?;
            edits.push((id, brush));
        }
        if at != bytes.len() {
            return Err(bad());
        }
        Ok(Box::new(edits))
    }

    fn apply(&mut self, decoded: Option<Box<dyn std::any::Any + Send>>) {
        let edits = decoded
            .and_then(|d| d.downcast::<Vec<(String, String)>>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        let mut log = self.0.lock().expect("the edit log");
        log.edits = edits;
        log.replay = true;
    }

    /// Each logged brush carries its terrain's pose with its centre, as
    /// they were when it was applied: a replay applies it where it was in
    /// the terrain, whatever the world's zero is now. Left as logged, so a
    /// replay after a shift touches the same samples, bit for bit.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}

const TERRAIN_SYSTEM: &str = "og.terrain";

/// The og.terrain system (front: before physics steps) and the edit log's
/// state block.
pub(crate) fn mount(world: &mut World, mount: crate::engine::Mount<'_>) -> Result<(), String> {
    let (engine_id, log) = (mount.id, mount.modules.terrain.0.clone());
    world
        .add_state_block(TERRAIN_SYSTEM, Box::new(TerrainBlock(log)))
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(og_ecs::SystemDesc::new(TERRAIN_SYSTEM), move |_ctx| {
            if let Ok(eng) = engine(engine_id) {
                replay_if_restored(&eng)
                    .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
            }
            Ok(())
        })
        .map_err(|e| e.reason().to_string())
}

/// After a restore: every terrain back as cooked, then the restored edits.
pub(crate) fn replay_if_restored(eng: &Engine) -> Result<(), String> {
    let edits = {
        let shared = log_of(eng);
        let mut log = shared.lock().expect("the edit log");
        if !log.replay {
            return Ok(());
        }
        log.replay = false;
        log.edits.clone()
    };
    let count = eng.render.borrow().terrain.loaded.len();
    let mut replayed = std::collections::BTreeMap::new();
    for i in 0..count {
        let (render, layout, range, collision, field) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            if !t.edited {
                continue;
            }
            t.edited = false;
            t.generation = next_generation();
            t.tiles.borrow_mut().clear();
            t.field = if t.collision {
                Some(t.cooked.field().map_err(|e| e.to_string())?)
            } else {
                None
            };
            (
                t.render,
                t.cooked.layout,
                t.cooked.range,
                t.collision,
                t.field.clone(),
            )
        };
        {
            let mut rs = eng.render.borrow_mut();
            rs.scene.set_terrain_stream(render, layout, range);
            rs.terrain.streamer.forget(i);
        }
        if collision {
            replayed.insert(i, field);
        }
    }
    for (id, brush) in edits {
        let Some(&i) = eng.render.borrow().terrain.by_id.get(&id) else {
            continue;
        };
        let b: Value = serde_json::from_str(&brush).map_err(|e| e.to_string())?;
        if stamp::removed(&b) {
            continue;
        }
        edit_loaded(eng, i, &b, false)?;
        if eng.render.borrow().terrain.loaded[i].collision {
            replayed.insert(i, eng.render.borrow().terrain.loaded[i].field.clone());
        }
    }
    // Physics gets each field's heights once, as the edits left them: the
    // snapshot's heights (a restore within the run that edited them) change
    // nothing, so the restored colliders keep their contacts and handles
    // and the world goes on as the uninterrupted run does.
    for (i, field) in replayed {
        if let Some(f) = field {
            set_heightfield(eng, GEOMETRY_BASE + i as u64, &f)?;
        }
    }
    Ok(())
}

/// The streamed terrains' presentation state, for a presentation
/// checkpoint: the streamer's and the tiles the renderer holds of each.
#[derive(Clone, Default)]
pub(crate) struct TerrainCheckpoint {
    stream: crate::bind::terrain_stream::StreamCheckpoint,
    held: Vec<(
        og_render::TerrainHandle,
        og_core::IdMap<u32, std::sync::Arc<og_render::TerrainTile>>,
    )>,
}

pub(crate) fn checkpoint(rs: &RenderState) -> TerrainCheckpoint {
    TerrainCheckpoint {
        stream: rs.terrain.streamer.checkpoint(),
        held: rs
            .terrain
            .loaded
            .iter()
            .filter_map(|t| Some((t.render, rs.scene.terrain_tiles(t.render)?.clone())))
            .collect(),
    }
}

pub(crate) fn restore(rs: &mut RenderState, c: &TerrainCheckpoint) {
    rs.terrain.streamer.restore(&c.stream);
    for (handle, tiles) in &c.held {
        let now: Vec<u32> = rs
            .scene
            .terrain_tiles(*handle)
            .map(|t| t.keys().copied().collect())
            .unwrap_or_default();
        for t in now {
            if !tiles.contains_key(&t) {
                rs.scene.set_terrain_tile(*handle, t, None);
            }
        }
        for (&t, tile) in tiles.iter() {
            rs.scene.set_terrain_tile(*handle, t, Some(tile.clone()));
        }
    }
}
