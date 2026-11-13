//! Runtime geometry, lines, points, debug draw and the mesh raycast (depth
//! S1): `og.render.bufferGeometry` and the three.js primitives over
//! `og_render::geometry`, the immediate `og.render.lines`/`points` and the
//! retained og.Lines/og.Points, `og.debug` with its views, and
//! `og.render.raycast` through a BVH per geometry.
//!
//! The state lives in the engine's `RenderState` ([`GeoState`]): the
//! queues are rendering, never simulation, so neither a snapshot nor a
//! replay carries them. The one piece the tick feeds is the raycast's
//! record of the drawn meshes as each tick starts ([`before_tick`]), which
//! answers a raycast from inside a system (a system cannot hand the world's
//! transforms over, as og.physics.castRay's recorded colliders).

use crate::engine::{Engine, engine, running};
use crate::host::api::{add_geometry, uv_repeat};
use crate::host::wire::{self, FromJson};
use crate::parse;
use crate::present::extract::{Hierarchy, RenderState};
use crate::world::assets::{self, index_of};
use glam::{DVec2, DVec3, Mat4, Quat, Vec2, Vec3};
use og_ecs::World;
use og_render::geometry::{self as geo, Bvh};
use og_render::{LineLayer, LineMode, Lines, MeshHandle, Points, debug};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::Arc;

mod build;

use crate::{bind::modules, bind::timeline};
pub use build::*;

mod debug_draw;
mod frame;
mod lines;
mod raycast;
mod views;

pub use debug_draw::*;
pub(crate) use frame::*;
pub use lines::*;
pub use raycast::*;
pub(crate) use views::*;

/// A drawn mesh as the raycast tests it.
#[derive(Clone)]
struct Target {
    entity: u64,
    /// Its index in `RenderState::geometries` (`usize::MAX`: none, a model
    /// part that is not a geometry handle).
    geometry: usize,
    mesh: MeshHandle,
    transform: Mat4,
    visible: bool,
    /// Its og.Mesh's or og.Model's three.js layer mask.
    layers: u32,
}

#[derive(Default)]
pub(crate) struct GeoState {
    /// Geometry indices made `dynamic`, and those registered with physics.
    dynamic: BTreeSet<usize>,
    physics: BTreeSet<usize>,
    /// A BVH per mesh handle index, with the mesh revision it was built at.
    bvh: HashMap<usize, (u64, Rc<Bvh>)>,
    /// The bounds per mesh handle index (the debug view), by revision.
    bounds: HashMap<usize, (u64, Vec3, Vec3)>,
    /// Immediate lines and points of the tick `tick`.
    lines: Vec<Lines>,
    points: Vec<Points>,
    tick: u64,
    /// Debug draws, each with the last tick it shows at.
    draws: Vec<(u64, Lines)>,
    texts: Vec<DebugText>,
    /// The debug view's index once set (`None`: RenderOptions.debugView).
    view: Option<usize>,
    disabled: bool,
    /// The drawn meshes as the current tick started (`before_tick`), with
    /// that tick.
    record: Option<(u64, Record)>,
    /// The transforms and parents as the record's tick started: the
    /// record's world matrices are composed from them, and a display frame
    /// between ticks blends from them (`crate::present::interpolate`).
    hierarchy: Hierarchy,
    /// Those of the last raycast between ticks, which reads the world as
    /// it is (its own, so the record's stay as its tick started).
    now: Hierarchy,
    /// The record's og.Mesh half and what it was read from.
    meshes: Meshes,
}

impl GeoState {
    /// Moves the debug draws and labels still showing by `d` (og.WorldOrigin
    /// moved the world's zero): they were placed in the world as it was.
    pub(crate) fn shift(&mut self, d: glam::DVec3) {
        let moved = |p: Vec3| (p.as_dvec3() + d).as_vec3();
        for (_, l) in &mut self.draws {
            if l.screen {
                continue;
            }
            if l.transform == Mat4::IDENTITY {
                l.points.iter_mut().for_each(|p| *p = moved(*p));
            } else {
                let t = l.transform.w_axis.truncate();
                l.transform.w_axis = moved(t).extend(l.transform.w_axis.w);
            }
        }
        for t in &mut self.texts {
            t.position = moved(t.position);
        }
    }
}

impl GeoState {
    /// The tick the raycast record was read as it started.
    pub(crate) fn record_tick(&self) -> Option<u64> {
        self.record.as_ref().map(|r| r.0)
    }

    /// The record's hierarchy (the transforms as its tick started), lent
    /// to a display frame's blend (`crate::present::interpolate`) until
    /// [`GeoState::put_hierarchy`].
    pub(crate) fn take_hierarchy(&mut self) -> Hierarchy {
        std::mem::take(&mut self.hierarchy)
    }

    pub(crate) fn put_hierarchy(&mut self, h: Hierarchy) {
        self.hierarchy = h;
    }

    /// The record and its hierarchy, for a presentation checkpoint: what
    /// a display frame after a restore blends from and a raycast between
    /// ticks reads, as the snapshot's tick started.
    pub(crate) fn record_checkpoint(&self) -> RecordCheckpoint {
        RecordCheckpoint {
            record: self.record.clone(),
            hierarchy: self.hierarchy.clone(),
        }
    }

    pub(crate) fn restore_record(&mut self, c: &RecordCheckpoint) {
        self.record = c.record.clone();
        self.hierarchy = c.hierarchy.clone();
    }
}

/// [`GeoState::record_checkpoint`].
#[derive(Clone, Default)]
pub(crate) struct RecordCheckpoint {
    record: Option<(u64, Record)>,
    hierarchy: Hierarchy,
}

/// The drawn og.Mesh rows of the last tick start, kept while the columns
/// they were read from, the entities and the geometries are unchanged:
/// a tick where no og.Mesh changed reads four columns' bytes, not every
/// row (20,000 still meshes: 0.33 ms a tick).
#[derive(Default)]
struct Meshes {
    /// Per og.Mesh archetype in query order: its entities and its
    /// geometry, visible and layers columns as read.
    columns: Vec<(Vec<og_ecs::Entity>, [Vec<u8>; 3])>,
    /// `RenderState::geometries`' length then (they are only appended).
    geometries: usize,
    list: Rc<Vec<Drawn>>,
}

impl Meshes {
    /// Whether the og.Mesh columns are as `columns` holds; else holds them
    /// now. False when they cannot be read by column (`mesh_rows`' JSON
    /// rows), so the rows are read every time.
    fn still(&mut self, world: &World, schema: &str, geometries: usize) -> bool {
        let id = world.id_by_name(schema).ok();
        let fields = ["geometry", "visible", "layers"]
            .map(|n| id.and_then(|id| world.field_by_id(id, n).ok()));
        let q = og_ecs::QueryDesc {
            read: vec![schema.to_string()],
            ..og_ecs::QueryDesc::default()
        };
        let (Ok(all), [Some(g), Some(v), Some(l)]) = (world.query(&q), fields) else {
            self.columns.clear();
            return false;
        };
        let mut still = geometries == self.geometries;
        let mut n = 0;
        for (k, a) in all.enumerate() {
            let (Ok(g), Ok(v), Ok(l)) = (a.read_bytes(g), a.read_bytes(v), a.read_bytes(l)) else {
                self.columns.clear();
                return false;
            };
            n = k + 1;
            if self.columns.len() <= k {
                self.columns.push(Default::default());
            }
            let (ents, cols) = &mut self.columns[k];
            if ents.as_slice() == a.entities() && cols[0] == *g && cols[1] == *v && cols[2] == *l {
                continue;
            }
            still = false;
            ents.clear();
            ents.extend_from_slice(a.entities());
            for (c, b) in cols.iter_mut().zip([&g, &v, &l]) {
                c.clear();
                c.extend_from_slice(b);
            }
        }
        if n != self.columns.len() {
            still = false;
            self.columns.truncate(n);
        }
        self.geometries = geometries;
        still
    }
}

/// A drawn mesh as a tick started, before its world matrix: an og.Mesh
/// (`part`: `None`) or one part of an og.Model (its transform within the
/// model).
#[derive(Clone, Copy)]
struct Drawn {
    entity: u64,
    geometry: usize,
    mesh: MeshHandle,
    part: Option<Mat4>,
    visible: bool,
    layers: u32,
}

/// The raycast record of a tick: its drawn meshes as read when it started,
/// then, from the first raycast of the tick on, the targets composed from
/// them. Most ticks raycast nothing from a system, and composing every
/// drawn mesh's world matrix each tick was a tenth of a frame of the
/// reference scene on the retired WebGL2 tier
/// (the former test sample
#[derive(Clone)]
enum Record {
    /// The og.Mesh rows (shared with the next tick's while they hold) and
    /// the og.Model parts.
    Drawn(Rc<Vec<Drawn>>, Vec<Drawn>),
    Targets(Rc<Vec<Target>>),
}

impl Record {
    /// The targets, composed from `h` (the transforms as the tick started)
    /// on the first call.
    fn targets(&mut self, h: &mut Hierarchy) -> Rc<Vec<Target>> {
        if let Record::Drawn(meshes, models) = self {
            let mut targets = compose(h, meshes);
            targets.extend(compose(h, models));
            *self = Record::Targets(Rc::new(targets));
        }
        match self {
            Record::Targets(t) => t.clone(),
            Record::Drawn(..) => unreachable!("composed above"),
        }
    }
}

impl GeoState {
    fn view_index(&self, options: &Value) -> usize {
        self.view.unwrap_or_else(|| view_of(&options["debugView"]))
    }
}

// ── decoding ────────────────────────────────────────────────────────────────

/// `q` normalised; a zero quaternion is the identity.
fn unit(q: Quat) -> Quat {
    if q.length_squared() < 1e-12 {
        Quat::IDENTITY
    } else {
        q.normalize()
    }
}

fn color4(c: &wire::Color, alpha: f32) -> [f32; 4] {
    [c.r, c.g, c.b, alpha]
}

/// `points`, every one finite.
fn finite_points(points: Vec<Vec3>, key: &str) -> Result<Vec<Vec3>, String> {
    if points.iter().any(|p| !p.is_finite()) {
        return Err(format!("{key} holds a point that is not finite"));
    }
    Ok(points)
}

fn colors_of(list: &[wire::Color], n: usize, alpha: f32) -> Result<Vec<[f32; 4]>, String> {
    if !list.is_empty() && list.len() != n {
        return Err(format!(
            "colors holds {} colours for {n} points: give one per point or none",
            list.len()
        ));
    }
    Ok(list.iter().map(|c| color4(c, alpha)).collect())
}

fn mesh_of_desc(d: &wire::BufferGeometryDesc) -> Result<og_render::MeshData, String> {
    geo::buffer_geometry(&geo::BufferAttributes {
        positions: &d.positions,
        normals: d.normals.as_deref(),
        uvs: d.uvs.as_deref(),
        colors: d.colors.as_deref(),
        tangents: d.tangents.as_deref(),
        indices: d.indices.as_deref(),
    })
}

/// The world's tick, inside a system or between ticks.
fn tick(eng: &Engine) -> u64 {
    if let Some(fr) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { fr.ctx() }.time().tick;
    }
    eng.world().map(|w| w.time().tick).unwrap_or(0)
}

// ── extraction ──────────────────────────────────────────────────────────────

/// A mesh's box in its own space, kept per revision (the bounds view's
/// and the review cameras' `crate::author::review`).
pub(crate) fn mesh_bounds(rs: &mut RenderState, mesh: MeshHandle) -> Option<(Vec3, Vec3)> {
    let rev = rs.scene.mesh_revision(mesh.index());
    if let Some(&(r, a, b)) = rs.geo.bounds.get(&mesh.index())
        && r == rev
    {
        return Some((a, b));
    }
    let i = geo::info(rs.scene.mesh(mesh)?);
    rs.geo.bounds.insert(mesh.index(), (rev, i.min, i.max));
    Some((i.min, i.max))
}

#[cfg(test)]
mod tests;
