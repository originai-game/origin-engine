//! og.sight, fog of war: the world's og.SightMap as an
//! `og_env::sight::SightMap`, cast again as each tick starts
//! ([`before_tick`]) from the og.Sight eyes and the og.SightBlocker
//! regions where they stand then, so every system of the tick and the
//! frame drawn after it read one answer. What each team has explored is
//! world state: the `og.sight` state block saves it, and the world hash
//! covers it. The records are lazy (crate::engine::lazy): a world that
//! names none has no block and casts nothing.

use crate::bind::physics_ext::Reader;
use crate::engine::{Engine, engine};
use crate::host::wire::{self, FromJson, SightPlane, record};
use glam::{Vec2, Vec3};
use og_ecs::{Entity, StateBlock, World};
use og_env::sight::{MAX_TEAM, SightMap, Viewer};
use og_env::volume::Placement;
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const MAP: &str = "og.SightMap";
const SIGHT: &str = "og.Sight";
const BLOCKER: &str = "og.SightBlocker";
/// Registered together (crate::engine::lazy).
pub(crate) const RECORDS: [&str; 3] = [MAP, SIGHT, BLOCKER];
/// The state block's name.
const BLOCK: &str = "og.sight";

/// The world's map and whose it is.
#[derive(Default)]
struct State {
    map: SightMap,
    /// The og.SightMap's entity (0: the world has none).
    entity: u64,
    flat: bool,
    /// The levels last drawn: the map's revision, the team, the cells.
    drawn: Option<(u64, u32, Arc<[u8]>)>,
    /// Counts what replaces the map from outside the tick (a restore), so
    /// the drawn levels are never another map's.
    epoch: u64,
}

type Shared = Arc<Mutex<State>>;

fn lock(s: &Shared) -> MutexGuard<'_, State> {
    s.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// og.sight on one engine: the state its world's block holds (none until
/// the records are registered).
#[derive(Default)]
pub(crate) struct ModuleState(std::cell::RefCell<Option<Shared>>);

fn shared(eng: &Engine) -> Option<Shared> {
    eng.modules.sight.0.borrow().clone()
}

/// Adds the og.sight state block to `world`, keeping the one it has.
pub(crate) fn attach(mount: crate::engine::Mount<'_>, world: &mut World) -> Result<(), String> {
    let state = Shared::default();
    match world.add_state_block(BLOCK, Box::new(Block(state.clone()))) {
        Ok(()) => {
            *mount.modules.sight.0.borrow_mut() = Some(state);
            Ok(())
        }
        Err(e) if e.reason().contains("already added") => Ok(()),
        Err(e) => Err(e.reason().to_string()),
    }
}

/// A world point in the map's plane.
fn on_plane(flat: bool, p: Vec3) -> Vec2 {
    if flat {
        Vec2::new(p.x, p.y)
    } else {
        Vec2::new(p.x, p.z)
    }
}

/// Lays the state's map over the world's og.SightMap (the first in entity
/// order with an og.Volume); without one the map and what it held are
/// dropped.
fn fit(eng: &Engine, r: &Reader<'_>, state: &mut State) -> Result<(), String> {
    let (maps, volumes) = {
        let st = eng.state.borrow();
        (
            st.layouts.get(MAP).cloned(),
            st.layouts.get("og.Volume").cloned(),
        )
    };
    let found = r.rows(maps.as_ref())?.into_iter().find_map(|(e, v)| {
        let v = record::SightMap::from_json(&v).ok()?;
        let at = placement(r, e)?;
        let volume = r.volume(volumes.as_ref(), Entity::from_bits(e))?;
        // Its size without its place: a map that moves (the world's zero
        // shifting) keeps its grid, and so what its teams explored.
        Some((e, v, at.position, volume.half_extent(&at) * 2.0))
    });
    let Some((e, v, centre, size)) = found else {
        let epoch = state.epoch + 1;
        *state = State {
            epoch,
            ..State::default()
        };
        return Ok(());
    };
    let flat = v.plane == SightPlane::Flat;
    if (state.entity, state.flat) != (e, flat) {
        state.map = SightMap::default();
        state.epoch += 1;
    }
    state.entity = e;
    state.flat = flat;
    state
        .map
        .fit(on_plane(flat, centre), on_plane(flat, size), v.cell);
    Ok(())
}

fn placement(r: &Reader<'_>, e: u64) -> Option<Placement> {
    let (position, quaternion, scale) = r.pose(Entity::from_bits(e))?;
    Some(Placement {
        position,
        quaternion,
        scale,
    })
}

/// As each tick starts: the map laid, the blockers on it, every og.Sight
/// cast, and what each team sees added to what it explored.
pub(crate) fn before_tick(eng: &Engine, world: &World) {
    let Some(shared) = shared(eng) else {
        return;
    };
    let r = Reader::World(world);
    let mut state = lock(&shared);
    if fit(eng, &r, &mut state).is_err() || state.entity == 0 {
        return;
    }
    let (sights, blockers, volumes) = {
        let st = eng.state.borrow();
        (
            st.layouts.get(SIGHT).cloned(),
            st.layouts.get(BLOCKER).cloned(),
            st.layouts.get("og.Volume").cloned(),
        )
    };
    let flat = state.flat;
    state.map.begin();
    for (e, v) in r.rows(blockers.as_ref()).unwrap_or_default() {
        let on = record::SightBlocker::from_json(&v).is_ok_and(|b| b.enabled);
        let (Some(at), true) = (placement(&r, e), on) else {
            continue;
        };
        let Some(volume) = r.volume(volumes.as_ref(), Entity::from_bits(e)) else {
            continue;
        };
        let (lo, hi) = volume.bounds(&at);
        // Cut at the blocker's own height (its depth on the flat plane).
        let lift = |p: Vec2| {
            if flat {
                Vec3::new(p.x, p.y, at.position.z)
            } else {
                Vec3::new(p.x, at.position.y, p.y)
            }
        };
        state
            .map
            .block(on_plane(flat, lo), on_plane(flat, hi), |p| {
                volume.outside(&at, lift(p)) <= 0.0
            });
    }
    for (e, v) in r.rows(sights.as_ref()).unwrap_or_default() {
        let (Ok(v), Some(at)) = (record::Sight::from_json(&v), placement(&r, e)) else {
            continue;
        };
        state.map.see(Viewer {
            team: v.team,
            at: on_plane(flat, at.position),
            radius: v.radius,
        });
    }
    state.map.finish();
}

/// The fog of war a frame draws: the map's cells for og.SightMap's team,
/// in the world's coordinates (`None` without a map, or one not drawn).
pub(crate) fn extract(
    world: &World,
    engine_id: u32,
    layout: Option<&crate::host::codec::Layout>,
) -> Option<og_render::FogOfWar> {
    let shared = shared(&*engine(engine_id).ok()?)?;
    let mut state = lock(&shared);
    if state.entity == 0 {
        return None;
    }
    let l = layout?;
    let id = world.id_by_name(&l.schema).ok()?;
    let bytes = world.get_bytes(Entity::from_bits(state.entity), id).ok()?;
    let v: record::SightMap = l.read(&bytes).ok()?;
    let (columns, rows) = state.map.size();
    if !v.drawn || columns == 0 {
        return None;
    }
    let key = state.map.revision() + (state.epoch << 40);
    let levels = match &state.drawn {
        Some((k, team, levels)) if (*k, *team) == (key, v.team) => levels.clone(),
        _ => {
            let levels: Arc<[u8]> = Arc::from(state.map.levels(v.team));
            state.drawn = Some((key, v.team, levels.clone()));
            levels
        }
    };
    Some(og_render::FogOfWar {
        min: state.map.min(),
        cell: state.map.cell(),
        columns,
        rows,
        levels,
        plane: if state.flat {
            og_render::SightPlane::Flat
        } else {
            og_render::SightPlane::Ground
        },
        color: Vec3::new(v.color.r, v.color.g, v.color.b),
        explored: v.explored,
    })
}

/// The engine's map for a binding: laid now when no tick has laid it yet
/// (a reveal in the game's setup). `None`: the world has no og.SightMap.
fn with_map<T>(
    world: u32,
    f: impl FnOnce(&mut State) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let eng = engine(world)?;
    let Some(shared) = shared(&eng) else {
        return Ok(None);
    };
    let mut state = lock(&shared);
    if state.entity == 0 {
        // Inside a system this reads og.SightMap, og.Volume and
        // og.Transform through its declarations; where it may not, the
        // map waits for the tick to lay it.
        if let Ok(r) = Reader::of(&eng, world) {
            let _ = fit(&eng, &r, &mut state);
        }
    }
    if state.entity == 0 {
        return Ok(None);
    }
    f(&mut state).map(Some)
}

fn point_of(point: &str) -> Result<Vec3, String> {
    let p: Vec3 = wire::arg(Some(point), "point")?;
    if p.is_finite() {
        Ok(p)
    } else {
        Err("point is not finite".to_string())
    }
}

fn team_of(team: u32) -> Result<u32, String> {
    if team <= MAX_TEAM {
        Ok(team)
    } else {
        Err(format!("team {team} is over {MAX_TEAM}"))
    }
}

const NO_MAP: &str = "the world has no og.SightMap with an og.Volume";

/// `og.sight.visible`.
pub fn og_sight_visible(world: u32, team: u32, point: String) -> Result<bool, String> {
    let p = point_of(&point)?;
    let seen = with_map(world, |s| Ok(s.map.visible(team, on_plane(s.flat, p))))?;
    Ok(seen.unwrap_or(false))
}

/// `og.sight.explored`.
pub fn og_sight_explored(world: u32, team: u32, point: String) -> Result<bool, String> {
    let p = point_of(&point)?;
    let seen = with_map(world, |s| Ok(s.map.explored(team, on_plane(s.flat, p))))?;
    Ok(seen.unwrap_or(false))
}

/// `og.sight.reveal`.
pub fn og_sight_reveal(world: u32, team: u32, point: String, radius: f64) -> Result<(), String> {
    let (p, team) = (point_of(&point)?, team_of(team)?);
    if !radius.is_finite() {
        return Err("radius is not finite".to_string());
    }
    with_map(world, |s| {
        s.map.reveal(team, on_plane(s.flat, p), radius as f32);
        Ok(())
    })?
    .ok_or_else(|| NO_MAP.to_string())
}

/// `og.sight.forget`.
pub fn og_sight_forget(world: u32, team: u32) -> Result<(), String> {
    let team = team_of(team)?;
    with_map(world, |s| {
        s.map.forget(Some(team));
        Ok(())
    })?
    .ok_or_else(|| NO_MAP.to_string())
}

/// What the block saves: the map's entity, its plane and its cells.
struct Saved {
    entity: u64,
    flat: bool,
    map: SightMap,
}

struct Block(Shared);

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        let s = lock(&self.0);
        out.extend_from_slice(&s.entity.to_le_bytes());
        out.push(u8::from(s.flat));
        s.map.save(out);
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let invalid =
            |e: &str| og_core::Error::new(og_core::ErrorKind::Invalid, format!("{BLOCK}: {e}"));
        let (head, rest) = bytes
            .split_at_checked(9)
            .ok_or_else(|| invalid("its bytes end early"))?;
        let (map, used) = SightMap::load(rest).map_err(|e| invalid(&e))?;
        if used != rest.len() {
            return Err(invalid("bytes follow the map"));
        }
        Ok(Box::new(Saved {
            entity: u64::from_le_bytes(head[..8].try_into().expect("eight bytes")),
            flat: head[8] != 0,
            map,
        }))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut s = lock(&self.0);
        let epoch = s.epoch + 1;
        *s = match decoded.and_then(|d| d.downcast::<Saved>().ok()) {
            Some(saved) => State {
                map: saved.map,
                entity: saved.entity,
                flat: saved.flat,
                ..State::default()
            },
            None => State::default(),
        };
        s.epoch = epoch;
    }

    /// The map lies where its entity stands: it moves with a root.
    fn shift(&mut self, shift: &og_ecs::Shift) {
        let mut s = lock(&self.0);
        let d = shift.of(s.entity).as_vec3();
        let d = on_plane(s.flat, d);
        s.map.shift(d);
    }
}
