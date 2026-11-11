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

use super::api::{add_geometry, uv_repeat};
use super::assets::{self, index_of};
use super::engine::{Engine, engine, running};
use super::extract::{Hierarchy, RenderState};
use super::parse;
use super::wire;
use glam::{DVec2, DVec3, Mat4, Quat, Vec2, Vec3};
use og_ecs::World;
use og_render::geometry::{self as geo, Bvh};
use og_render::{LineLayer, LineMode, Lines, MeshHandle, Points, debug};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::Arc;

mod build;

use super::{modules, timeline};
pub use build::*;

/// The debug views (`render.DebugView`), by variant index.
const VIEWS: [&str; 18] = [
    "none",
    "physics",
    "nav",
    "bounds",
    "lod",
    "cascades",
    "ao",
    "albedo",
    "iblDiffuse",
    "iblSpecular",
    "direct",
    "aoDepth",
    "aoRaw",
    "terrain",
    "terrainLayers",
    "terrainMip",
    "terrainProjection",
    "terrainDensity",
];

/// The renderer's own view for a debug view drawn in place of the lit
/// image (`RenderScene::debug`); the overlay views draw over it.
fn render_view(view: &str) -> og_render::DebugView {
    use og_render::DebugView as V;
    match view {
        "cascades" => V::Cascades,
        "ao" => V::AmbientOcclusion,
        "albedo" => V::Albedo,
        "iblDiffuse" => V::IblDiffuse,
        "iblSpecular" => V::IblSpecular,
        "direct" => V::DirectLight,
        "aoDepth" => V::AoDepth,
        "aoRaw" => V::AoRaw,
        "terrain" => V::Terrain,
        "terrainLayers" => V::TerrainLayers,
        "terrainMip" => V::TerrainMip,
        "terrainProjection" => V::TerrainProjection,
        "terrainDensity" => V::TerrainDensity,
        _ => V::None,
    }
}

/// Explicit coverage for paths with no shading diagnostic or overlay.
pub(crate) fn debug_coverage(view: &str) -> serde_json::Value {
    let supported: &[&str] = match view {
        "none" => return serde_json::Value::Null,
        "physics" => &["colliders"],
        "nav" => &["navigation"],
        "bounds" => &["mesh instances"],
        "lod" => &["og.Model"],
        "terrain" | "terrainLayers" | "terrainMip" | "terrainProjection" | "terrainDensity" => {
            &["terrain"]
        }
        "albedo" => &[
            "standard",
            "physical",
            "custom",
            "terrain",
            "grass",
            "scatter",
            "impostors",
            "decals on standard/physical",
        ],
        "ao" | "aoDepth" | "aoRaw" => &[
            "standard",
            "physical",
            "grass",
            "scatter",
            "water",
            "decals on standard/physical",
        ],
        "cascades" => &[
            "standard lit",
            "physical",
            "terrain",
            "grass",
            "decals on standard/physical",
        ],
        "iblDiffuse" | "iblSpecular" | "direct" => {
            &["standard lit", "terrain", "grass", "decals on standard"]
        }
        _ => &[],
    };
    serde_json::json!({
        "supported": supported,
        "otherPaths": "unsupported",
        "unsupportedShading": "magenta/black checker for 3D surfaces; sprites, 2D fluids, lines, points and UI have no shading diagnostics",
        "overlays": "physics/nav/bounds/lod retain beauty shading; only the listed subjects receive the overlay",
        "decals": "part of the receiving standard/physical surface, not a separate shading pass",
    })
}

/// A debug label (`og.debug.text`).
struct DebugText {
    until: u64,
    position: Vec3,
    text: String,
    color: [f32; 4],
    size: f32,
}

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
    /// between ticks blends from them (`super::interpolate`).
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
    /// to a display frame's blend (`super::interpolate`) until
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

/// The debug view drawn this frame (`none` while the overlay is off):
/// `lod` is drawn by extraction itself, tinting each model by its level.
pub(crate) fn current_view(rs: &RenderState) -> &'static str {
    if rs.geo.disabled {
        return "none";
    }
    VIEWS[rs.geo.view_index(&rs.options)]
}

fn view_of(v: &Value) -> usize {
    match v {
        Value::String(s) => VIEWS.iter().position(|n| n == s).unwrap_or(0),
        Value::Number(n) => (n.as_u64().unwrap_or(0) as usize).min(VIEWS.len() - 1),
        _ => 0,
    }
}

// ── decoding ────────────────────────────────────────────────────────────────

fn num(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

fn flag(v: &Value, k: &str, d: bool) -> bool {
    v[k].as_bool().unwrap_or(d)
}

/// `q` normalised; a zero quaternion is the identity.
fn unit(q: Quat) -> Quat {
    if q.length_squared() < 1e-12 {
        Quat::IDENTITY
    } else {
        q.normalize()
    }
}

fn rgb(v: &Value, d: [f32; 3]) -> [f32; 3] {
    if !v.is_object() {
        return d;
    }
    [num(v, "r", d[0]), num(v, "g", d[1]), num(v, "b", d[2])]
}

fn rgba(v: &Value, d: [f32; 3], alpha: f32) -> [f32; 4] {
    let [r, g, b] = rgb(v, d);
    [r, g, b, alpha]
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

/// The BVH of a mesh, built now when it is missing or older than the mesh.
fn bvh_of(g: &mut GeoState, scene: &og_render::RenderScene, mesh: MeshHandle) -> Option<Rc<Bvh>> {
    let rev = scene.mesh_revision(mesh.index());
    if let Some((r, b)) = g.bvh.get(&mesh.index())
        && *r == rev
    {
        return Some(b.clone());
    }
    let b = Rc::new(Bvh::build(scene.mesh(mesh)?));
    g.bvh.insert(mesh.index(), (rev, b.clone()));
    Some(b)
}

/// A loaded model's primitive takes the BVH its cook built (over the mesh
/// the scene now holds at `mesh`), so no raycast builds one; a primitive
/// cooked with `raycast false` keeps the lazy build.
/// Drops what was cached for a mesh (its raycast BVH, its bounds): a
/// released uncooked model's meshes (og.load.release).
pub(crate) fn forget_mesh(rs: &mut RenderState, mesh: MeshHandle) {
    rs.geo.bvh.remove(&mesh.index());
    rs.geo.bounds.remove(&mesh.index());
}

pub(crate) fn adopt_cooked_bvh(
    rs: &mut RenderState,
    mesh: MeshHandle,
    prim: &og_bundle::runtime::CookedPrimitive,
) -> Result<(), String> {
    let RenderState { geo: g, scene, .. } = rs;
    let Some(data) = scene.mesh(mesh) else {
        return Ok(());
    };
    let Some(bvh) = prim.bvh(data) else {
        return Ok(());
    };
    let bvh = bvh?;
    g.bvh.insert(
        mesh.index(),
        (scene.mesh_revision(mesh.index()), Rc::new(bvh)),
    );
    Ok(())
}

// ── lines and points ────────────────────────────────────────────────────────

fn line_mode(v: &Value) -> LineMode {
    match v {
        Value::String(s) if s == "segments" => LineMode::Segments,
        Value::String(s) if s == "loop" => LineMode::Loop,
        Value::Number(n) if n.as_u64() == Some(1) => LineMode::Segments,
        Value::Number(n) if n.as_u64() == Some(2) => LineMode::Loop,
        _ => LineMode::Strip,
    }
}

/// Starts the immediate queues of a new tick.
fn queue(rs: &mut RenderState, tick: u64) -> &mut GeoState {
    let g = &mut rs.geo;
    if g.tick != tick {
        g.lines.clear();
        g.points.clear();
        g.tick = tick;
    }
    g
}

pub fn og_render_lines(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::LinesDesc = wire::arg(desc.as_deref(), "desc")?;
        let points = finite_points(d.points, "points")?;
        if points.len() < 2 {
            return Err(format!(
                "lines need at least two points, got {}",
                points.len()
            ));
        }
        let alpha = d.opacity.clamp(0.0, 1.0);
        let colors = colors_of(&d.colors, points.len(), alpha)?;
        let screen = d.screen;
        let lines = Lines {
            colors,
            color: color4(&d.color, alpha),
            mode: match d.mode {
                wire::LineMode::Strip => LineMode::Strip,
                wire::LineMode::Segments => LineMode::Segments,
                wire::LineMode::Loop => LineMode::Loop,
            },
            width: size(d.width, "width")?,
            world_width: d.world_units && !screen,
            depth_test: d.depth_test && !screen,
            screen,
            dashed: d
                .dashed
                .then(|| (d.dash_size.max(0.0), d.gap_size.max(0.0))),
            layer: if screen {
                LineLayer::Overlay
            } else {
                LineLayer::Scene
            },
            points,
            transform: Mat4::IDENTITY,
        };
        let t = tick(&eng);
        queue(&mut eng.render.borrow_mut(), t).lines.push(lines);
        Ok(())
    })()
}

pub fn og_render_points(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::PointsDesc = wire::arg(desc.as_deref(), "desc")?;
        let positions = finite_points(d.points, "points")?;
        let alpha = d.opacity.clamp(0.0, 1.0);
        let colors = colors_of(&d.colors, positions.len(), alpha)?;
        let screen = d.screen;
        let points = Points {
            positions,
            colors,
            color: color4(&d.color, alpha),
            size: size(d.size, "size")?,
            size_attenuation: d.size_attenuation && !screen,
            depth_test: d.depth_test && !screen,
            screen,
            layer: if screen {
                LineLayer::Overlay
            } else {
                LineLayer::Scene
            },
            transform: Mat4::IDENTITY,
        };
        let t = tick(&eng);
        queue(&mut eng.render.borrow_mut(), t).points.push(points);
        Ok(())
    })()
}

// ── raycast ─────────────────────────────────────────────────────────────────

/// Every og.Mesh and og.Model part as drawn from the world now, with `h`
/// read again (its slots brought to the read when the record is first
/// composed, [`Hierarchy::capture`]): the og.Mesh rows in query order (`kept`'s while their
/// columns are unchanged), and the og.Model parts in entity order into
/// `out` (cleared).
fn drawn(
    world: &World,
    rs: &RenderState,
    h: &mut Hierarchy,
    kept: &mut Meshes,
    out: &mut Vec<Drawn>,
) -> Rc<Vec<Drawn>> {
    use super::codec::{Rec, Row, Slot};
    out.clear();
    h.capture(world);
    let meshes = match &rs.draw.mesh {
        Some(layout) if kept.still(world, &layout.schema, rs.geometries.len()) => kept.list.clone(),
        Some(layout) => {
            let mut list = Vec::new();
            for r in super::extract::mesh_rows(world, layout) {
                let Some(&mesh) = rs.geometries.get(r.geometry) else {
                    continue;
                };
                list.push(Drawn {
                    entity: r.entity,
                    geometry: r.geometry,
                    mesh,
                    part: None,
                    visible: r.visible,
                    layers: r.layers,
                });
            }
            kept.list = Rc::new(list);
            kept.list.clone()
        }
        None => Rc::new(Vec::new()),
    };
    if let Some(layout) = &rs.draw.model
        && let Ok(id) = world.id_by_name(&layout.schema)
    {
        // Read in place, as extraction reads them (decoding each row to
        // JSON was half of this read).
        let (asset, visible, layers) = (
            Slot::of(layout, "asset"),
            Slot::of(layout, "visible"),
            Slot::of(layout, "layers"),
        );
        let mut rows = super::feel::byte_rows(world, id, layout);
        og_core::sort::sort_by_key(&mut rows, |(e, _)| og_ecs::Entity::from_bits(*e).index());
        // Each part's geometry index through a map (its first position, as
        // a search gave): a search per part made this read, every tick,
        // parts × geometries (Daguanyuan, native: a 8.7 ms tick, 1.8 ms after).
        let mut geometry_of = std::collections::HashMap::with_capacity(rs.geometries.len());
        for (i, g) in rs.geometries.iter().enumerate() {
            geometry_of.entry(*g).or_insert(i);
        }
        for (e, bytes) in &rows {
            let v = Row::new(layout, bytes);
            let Some(parts) = rs.models.get(&(v.num(&asset).unwrap_or(0.0) as u64)) else {
                continue;
            };
            let visible = v.flag(&visible).unwrap_or(true);
            let layers = v.uint(&layers).map_or(1, |x| x as u32);
            for &(mesh, _, local) in parts.iter() {
                out.push(Drawn {
                    entity: *e,
                    geometry: geometry_of.get(&mesh).copied().unwrap_or(usize::MAX),
                    mesh,
                    part: Some(local),
                    visible,
                    layers,
                });
            }
        }
    }
    meshes
}

/// The targets of `drawn`, their world matrices from `h`.
fn compose(h: &mut Hierarchy, drawn: &[Drawn]) -> Vec<Target> {
    drawn
        .iter()
        .map(|d| {
            let m = h.world_of(d.entity);
            Target {
                entity: d.entity,
                geometry: d.geometry,
                mesh: d.mesh,
                transform: d.part.map_or(m, |local| m * local),
                visible: d.visible,
                layers: d.layers,
            }
        })
        .collect()
}

/// The probe's tick start: the drawn meshes as the tick starts, for a
/// raycast from inside a system. Their world matrices wait for the first
/// such raycast ([`Record`]).
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let tick = world.time().tick;
    let mut models = match rs.geo.record.take() {
        Some((_, Record::Drawn(_, models))) => models,
        _ => Vec::new(),
    };
    let mut h = std::mem::take(&mut rs.geo.hierarchy);
    let mut kept = std::mem::take(&mut rs.geo.meshes);
    let meshes = drawn(world, &rs, &mut h, &mut kept, &mut models);
    rs.geo.hierarchy = h;
    rs.geo.meshes = kept;
    rs.geo.record = Some((tick, Record::Drawn(meshes, models)));
}

pub fn og_render_raycast(
    world: u32,
    ray: String,
    desc: Option<String>,
) -> Result<Vec<crate::wire::GeometryHit>, String> {
    (|| {
        let eng = engine(world)?;
        let r: wire::Ray = wire::arg(Some(&ray), "ray")?;
        let d: wire::RaycastGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let (origin, dir) = (r.origin, r.direction);
        if !origin.is_finite() || !dir.is_finite() || dir.length_squared() < 1e-20 {
            return Err(
                "the ray's origin and direction must be finite and its direction not zero".into(),
            );
        }
        let dir = dir.normalize();
        let max = d.max_distance;
        let max = if max > 0.0 && max.is_finite() {
            max
        } else {
            f32::INFINITY
        };
        let (first, invisible) = (d.first, d.invisible);
        // Three.js layers: a mesh is tested when its mask shares a bit
        // (`super::views`).
        let layers = d.layers;
        let list: Rc<Vec<Target>> = if running(eng.id).is_some() {
            let mut rs = eng.render.borrow_mut();
            let GeoState {
                record, hierarchy, ..
            } = &mut rs.geo;
            match record {
                Some((_, r)) => r.targets(hierarchy),
                None => Rc::new(Vec::new()),
            }
        } else {
            let mut rs = eng.render.borrow_mut();
            let mut h = std::mem::take(&mut rs.geo.now);
            let mut models = Vec::new();
            let meshes = drawn(
                eng.world()?,
                &rs,
                &mut h,
                &mut Meshes::default(),
                &mut models,
            );
            let mut targets = compose(&mut h, &meshes);
            targets.extend(compose(&mut h, &models));
            rs.geo.now = h;
            Rc::new(targets)
        };
        let mut rs = eng.render.borrow_mut();
        let RenderState { geo: g, scene, .. } = &mut *rs;
        let mut hits = Vec::new();
        let mut reach = max;
        for t in list
            .iter()
            .filter(|t| (t.visible || invisible) && t.layers & layers != 0)
        {
            let det = t.transform.determinant();
            if !det.is_finite() || det.abs() < 1e-12 {
                continue;
            }
            let Some(bvh) = bvh_of(g, scene, t.mesh) else {
                continue;
            };
            let inv = t.transform.inverse();
            // Not normalised: t along it is the world distance.
            let lo = inv.transform_point3(origin);
            let ld = inv.transform_vector3(dir);
            let normal_m = inv.transpose();
            for hit in bvh.raycast(lo, ld, reach, first) {
                let n = normal_m.transform_vector3(hit.normal).normalize_or_zero();
                let p = origin + dir * hit.t;
                hits.push((
                    hit.t,
                    wire::GeometryHit {
                        entity: t.entity,
                        geometry: if t.geometry == usize::MAX {
                            0
                        } else {
                            assets::handle(t.geometry)
                        },
                        distance: hit.t,
                        point: p,
                        normal: n,
                        uv: Vec2::from_array(hit.uv),
                        face_index: hit.face,
                    },
                ));
                if first {
                    reach = reach.min(hit.t);
                }
            }
        }
        og_core::sort::sort_by(&mut hits, |a, b| a.0.total_cmp(&b.0));
        if first {
            hits.truncate(1);
        }
        Ok(hits.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
    })()
}

// ── og.debug ────────────────────────────────────────────────────────────────

struct DrawDesc {
    color: [f32; 4],
    width: f32,
    until: u64,
    rotation: Quat,
}

fn draw_desc(eng: &Engine, desc: Option<String>) -> DrawDesc {
    let d: wire::DebugDrawDesc = wire::arg(desc.as_deref(), "desc").unwrap_or_default();
    let w = d.width;
    DrawDesc {
        color: color4(&d.color, 1.0),
        width: if w.is_finite() { w.max(0.0) } else { 2.0 },
        // Drawn at the frame after the tick that asked (its world reads
        // the next tick), and `duration` ticks past it.
        until: tick(eng) + 1 + u64::from(d.duration),
        rotation: unit(d.rotation),
    }
}

fn draw(world: u32, desc: Option<String>, shape: impl FnOnce(&DrawDesc) -> Vec<Lines>) {
    let Ok(eng) = engine(world) else {
        return;
    };
    let dd = draw_desc(&eng, desc);
    let sets = shape(&dd);
    let mut rs = eng.render.borrow_mut();
    rs.geo.draws.extend(sets.into_iter().map(|l| (dd.until, l)));
}

fn arg_vec3(text: &str) -> Vec3 {
    wire::arg(Some(text), "point").unwrap_or_default()
}

pub fn og_debug_line(world: u32, from: String, to: String, desc: Option<String>) {
    let (a, b) = (arg_vec3(&from), arg_vec3(&to));
    draw(world, desc, |d| {
        vec![debug::segments(vec![a, b], d.color, d.width)]
    });
}

pub fn og_debug_box(world: u32, center: String, size: String, desc: Option<String>) {
    let (c, s) = (arg_vec3(&center), arg_vec3(&size));
    draw(world, desc, |d| {
        let m = Mat4::from_rotation_translation(d.rotation, c);
        vec![debug::segments(
            debug::box_edges(m, s * 0.5),
            d.color,
            d.width,
        )]
    });
}

pub fn og_debug_sphere(world: u32, center: String, radius: f64, desc: Option<String>) {
    let c = arg_vec3(&center);
    draw(world, desc, |d| {
        vec![debug::segments(
            debug::sphere(c, radius as f32, d.rotation),
            d.color,
            d.width,
        )]
    });
}

pub fn og_debug_arrow(world: u32, from: String, to: String, desc: Option<String>) {
    let (a, b) = (arg_vec3(&from), arg_vec3(&to));
    draw(world, desc, |d| {
        let head = (b - a).length() * 0.2;
        vec![debug::segments(debug::arrow(a, b, head), d.color, d.width)]
    });
}

pub fn og_debug_axes(world: u32, position: String, size: f64, desc: Option<String>) {
    let p = arg_vec3(&position);
    draw(world, desc, |d| {
        let m = Mat4::from_rotation_translation(d.rotation, p);
        debug::axes(m, size as f32, d.width).into()
    });
}

pub fn og_debug_text(world: u32, position: String, text: String, desc: Option<String>) {
    let Ok(eng) = engine(world) else {
        return;
    };
    let p = arg_vec3(&position);
    let d = draw_desc(&eng, desc);
    eng.render.borrow_mut().geo.texts.push(DebugText {
        until: d.until,
        position: p,
        text,
        color: d.color,
        size: if d.width <= 2.0 { 14.0 } else { d.width },
    });
}

pub fn og_debug_clear(world: u32) {
    if let Ok(eng) = engine(world) {
        let mut rs = eng.render.borrow_mut();
        rs.geo.draws.clear();
        rs.geo.texts.clear();
    }
}

pub fn og_debug_set_view(world: u32, view: String) {
    if let Ok(eng) = engine(world) {
        // A DebugView by name (an index is not part of the API).
        eng.render.borrow_mut().geo.view = Some(view_of(&Value::String(view)));
    }
}

pub fn og_debug_view(world: u32) -> &'static str {
    let i = engine(world).map_or(0, |eng| {
        let rs = eng.render.borrow();
        rs.geo.view_index(&rs.options)
    });
    VIEWS[i]
}

pub fn og_debug_set_enabled(world: u32, enabled: bool) {
    if let Ok(eng) = engine(world) {
        eng.render.borrow_mut().geo.disabled = !enabled;
    }
}

pub fn og_debug_enabled(world: u32) -> bool {
    engine(world).is_ok_and(|eng| !eng.render.borrow().geo.disabled)
}

/// The command channel's `debug:on|off|<view>`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "debug",
        og_probe::CommandDesc {
            usage: "debug:on|off|none|physics|nav|bounds|lod|cascades|ao|albedo|iblDiffuse|iblSpecular|direct|aoDepth|aoRaw|terrain|terrainLayers|terrainMip|terrainProjection|terrainDensity".into(),
            doc: "show or hide the debug overlay (og.debug's draws), or pick its view: physics (colliders, joints, contacts), nav (grids, navmeshes, agents' paths), bounds (every drawn mesh's box), lod (every model tinted by its level of detail), cascades (the shadow cascade per pixel), ao (the ambient occlusion term), albedo, iblDiffuse, iblSpecular, direct (the lit materials' lighting terms), aoDepth (the AO pass's depth), aoRaw (the AO before its blur), terrain (projection weights and clipmap cells), terrainLayers (strongest raw layer), terrainMip (normal-map mip footprint), terrainProjection (unobscured XYZ projection weights), terrainDensity (XYZ vertical albedo texel density encoded as d/(1+d)), none".into(),
        },
        move |_world, args| {
            let arg = args.first().map(String::as_str).unwrap_or("on");
            let eng = engine(engine_id)?;
            let mut rs = eng.render.borrow_mut();
            match arg {
                "on" => rs.geo.disabled = false,
                "off" => rs.geo.disabled = true,
                v => {
                    let i = VIEWS.iter().position(|n| *n == v).ok_or_else(|| {
                        format!("debug:{v}: write debug:on, debug:off or a view: {}", VIEWS.join(", "))
                    })?;
                    rs.geo.view = Some(i);
                    rs.geo.disabled = false;
                }
            }
            Ok(())
        },
    );
}

// ── extraction ──────────────────────────────────────────────────────────────

/// The vertices of a geometry handle field as points and colours.
fn vertices_of(rs: &RenderState, v: &Value, alpha: f32) -> Option<(Vec<Vec3>, Vec<[f32; 4]>)> {
    let index = super::fx::handle_index(&v["geometry"])?;
    let mesh = rs.scene.mesh(*rs.geometries.get(index)?)?;
    let points = mesh
        .vertices
        .iter()
        .map(|v| Vec3::from(v.position))
        .collect();
    let colors = mesh
        .vertices
        .iter()
        .map(|v| [v.color[0], v.color[1], v.color[2], v.color[3] * alpha])
        .collect();
    Some((points, colors))
}

/// Lines, points and debug draw into the frame (`super::extract`).
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy, vp: (f32, f32)) {
    rs.scene.frame.css_height = vp.1;
    let now = world.time().tick;
    // Immediate sets of the tick drawn (or the one before it, whose frame
    // this is).
    if rs.geo.tick == now || rs.geo.tick + 1 == now {
        rs.scene.effects.lines.extend(rs.geo.lines.iter().cloned());
        rs.scene
            .effects
            .points
            .extend(rs.geo.points.iter().cloned());
    }
    // og.Lines and og.Points.
    if let Some(layout) = rs.draw.lines.clone() {
        for (e, v) in super::fx::rows(world, Some(&layout)) {
            // Drawn by the cameras its layers admit (the renderer tests them).
            let layers = v["layers"].as_u64().map_or(1, |x| x as u32);
            if !flag(&v, "visible", true) {
                continue;
            }
            let alpha = num(&v, "opacity", 1.0).clamp(0.0, 1.0);
            let Some((points, colors)) = vertices_of(rs, &v, alpha) else {
                continue;
            };
            let dash = num(&v, "dashSize", 0.0);
            let at = rs.scene.effects.lines.len();
            if layers != 1 {
                rs.scene.effects.line_layers.resize(at, 1);
                rs.scene.effects.line_layers.push(layers);
            }
            rs.scene.effects.lines.push(Lines {
                points,
                colors,
                color: rgba(&v["color"], [1.0; 3], 1.0),
                mode: line_mode(&v["mode"]),
                width: num(&v, "width", 1.0).max(0.0),
                world_width: flag(&v, "worldUnits", false),
                depth_test: flag(&v, "depthTest", true),
                screen: false,
                dashed: (dash > 0.0).then(|| (dash, num(&v, "gapSize", 1.0).max(0.0))),
                layer: LineLayer::Scene,
                transform: h.world_of(e),
            });
        }
    }
    if let Some(layout) = rs.draw.points.clone() {
        for (e, v) in super::fx::rows(world, Some(&layout)) {
            // Drawn by the cameras its layers admit (the renderer tests them).
            let layers = v["layers"].as_u64().map_or(1, |x| x as u32);
            if !flag(&v, "visible", true) {
                continue;
            }
            let alpha = num(&v, "opacity", 1.0).clamp(0.0, 1.0);
            let Some((positions, colors)) = vertices_of(rs, &v, alpha) else {
                continue;
            };
            let at = rs.scene.effects.points.len();
            if layers != 1 {
                rs.scene.effects.point_layers.resize(at, 1);
                rs.scene.effects.point_layers.push(layers);
            }
            rs.scene.effects.points.push(Points {
                positions,
                colors,
                color: rgba(&v["color"], [1.0; 3], 1.0),
                size: num(&v, "size", 1.0).max(0.0),
                size_attenuation: flag(&v, "sizeAttenuation", true),
                depth_test: flag(&v, "depthTest", true),
                screen: false,
                layer: LineLayer::Scene,
                transform: h.world_of(e),
            });
        }
    }
    // Debug draw.
    rs.geo.draws.retain(|(until, _)| *until >= now);
    rs.geo.texts.retain(|t| t.until >= now);
    // The lit image's views; a publish build draws none of them.
    rs.scene.frame.debug = if cfg!(feature = "publish") {
        og_render::DebugView::None
    } else {
        render_view(current_view(rs))
    };
    if rs.geo.disabled {
        return;
    }
    let draws: Vec<Lines> = rs.geo.draws.iter().map(|(_, l)| l.clone()).collect();
    rs.scene.effects.lines.extend(draws);
    match VIEWS[rs.geo.view_index(&rs.options)] {
        "physics" => physics_view(rs),
        #[cfg(feature = "nav")]
        "nav" => nav_view(rs),
        "bounds" => bounds_view(rs),
        _ => {}
    }
    texts(rs, vp);
}

fn physics_view(rs: &mut RenderState) {
    let Some(lines) = rs.draw.physics.as_ref().and_then(|p| p.debug_lines()) else {
        return;
    };
    if lines.is_empty() {
        return;
    }
    let mut set = debug::segments(Vec::with_capacity(lines.len() * 2), [1.0; 4], 1.5);
    set.colors.reserve(lines.len() * 2);
    for (a, b, c) in lines {
        set.points.extend([a, b]);
        set.colors.extend([c, c]);
    }
    rs.scene.effects.lines.push(set);
}

#[cfg(feature = "nav")]
fn nav_view(rs: &mut RenderState) {
    let engine_id = rs.draw.engine;
    let Some((navs, nav, profile)) =
        super::e6::with(engine_id, |m| (m.navs.clone(), m.nav.clone(), m.profile))
    else {
        return;
    };
    let plane = super::e6::plane_of(profile);
    let lift = |p: Vec3| match plane {
        og_nav::Plane::Xz => p + Vec3::Y * 0.05,
        og_nav::Plane::Xy => p,
    };
    let mut edges = Vec::new();
    let mut blocked = Vec::new();
    {
        let navs = navs.lock().unwrap_or_else(|e| e.into_inner());
        for n in navs.entries.values() {
            match n {
                og_nav::Nav::Grid(g) => {
                    let s = g.cell_size;
                    let corner = |x: f32, y: f32| {
                        lift(plane.to_world(g.origin + glam::Vec2::new(x, y) * s, 0.0))
                    };
                    let (w, ht) = (g.width as f32, g.height as f32);
                    edges.extend([
                        corner(0.0, 0.0),
                        corner(w, 0.0),
                        corner(w, 0.0),
                        corner(w, ht),
                        corner(w, ht),
                        corner(0.0, ht),
                        corner(0.0, ht),
                        corner(0.0, 0.0),
                    ]);
                    for y in 0..g.height {
                        for x in 0..g.width {
                            if g.cost(x as i32, y as i32) != og_nav::BLOCKED {
                                continue;
                            }
                            let (fx, fy) = (x as f32, y as f32);
                            blocked.extend([
                                corner(fx + 0.15, fy + 0.15),
                                corner(fx + 0.85, fy + 0.85),
                                corner(fx + 0.85, fy + 0.15),
                                corner(fx + 0.15, fy + 0.85),
                            ]);
                        }
                    }
                }
                og_nav::Nav::Mesh(m) => {
                    for tile in &m.tiles {
                        for poly in &tile.polys {
                            let n = poly.verts.len();
                            for i in 0..n {
                                let a = tile.verts[poly.verts[i] as usize];
                                let b = tile.verts[poly.verts[(i + 1) % n] as usize];
                                edges.extend([lift(a), lift(b)]);
                            }
                        }
                    }
                }
                og_nav::Nav::Flow { .. } => {}
            }
        }
    }
    if !edges.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(edges, [0.2, 0.75, 1.0, 1.0], 1.0));
    }
    if !blocked.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(blocked, [1.0, 0.3, 0.25, 1.0], 1.0));
    }
    if let Some(nav) = nav {
        for (_, path) in nav.paths() {
            if path.corners.len() < 2 {
                continue;
            }
            let mut l = debug::segments(
                path.corners.iter().map(|p| lift(*p)).collect(),
                [1.0, 0.85, 0.2, 1.0],
                2.0,
            );
            l.mode = LineMode::Strip;
            rs.scene.effects.lines.push(l);
        }
    }
}

/// A mesh's box in its own space, kept per revision (the bounds view's
/// and the review cameras' `super::review`).
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

fn bounds_view(rs: &mut RenderState) {
    let mut out = Vec::new();
    let instances: Vec<(MeshHandle, Mat4)> = rs
        .scene
        .objects
        .instances
        .iter()
        .map(|i| (i.mesh, i.transform))
        .collect();
    for (mesh, m) in instances {
        let Some((min, max)) = mesh_bounds(rs, mesh) else {
            continue;
        };
        let centre = (min + max) * 0.5;
        out.extend(debug::box_edges(
            m * Mat4::from_translation(centre),
            (max - min) * 0.5,
        ));
    }
    if !out.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(out, [0.3, 1.0, 0.45, 1.0], 1.0));
    }
}

/// Debug labels: each world point projected to the play surface, the
/// label centred above it in the UI's body font.
// Render-only: the labels are drawn into the frame, never read by the tick.
#[allow(clippy::disallowed_methods)]
fn texts(rs: &mut RenderState, vp: (f32, f32)) {
    if rs.geo.texts.is_empty() || vp.0 <= 0.0 || vp.1 <= 0.0 {
        return;
    }
    let cam = &rs.scene.frame.camera;
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let aspect = vp.0 / vp.1;
    let proj = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => {
            glam::camera::rh::proj::directx::perspective(
                fov_y_degrees.to_radians(),
                aspect,
                cam.near,
                cam.far,
            )
        }
        og_render::Projection::Orthographic { height } => {
            let h = height.abs().max(1e-6) * 0.5;
            glam::camera::rh::proj::directx::orthographic(
                -h * aspect,
                h * aspect,
                -h,
                h,
                cam.near,
                cam.far,
            )
        }
    };
    let vpm = proj * view;
    let ui = rs.ui.clone();
    let mut out = Vec::new();
    for t in &rs.geo.texts {
        let clip = vpm * t.position.extend(1.0);
        if clip.w <= 1e-6 {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < 0.0 || ndc.z > 1.0 {
            continue;
        }
        let Some(body) = ui.as_ref().and_then(|u| {
            u.lock()
                .fonts()
                .for_text(og_ui::FontRole::Body, &t.text)
                .handle
        }) else {
            continue;
        };
        let pages = rs
            .ui_pages
            .get(&body)
            .cloned()
            .unwrap_or_else(|| vec![body]);
        let Some(first) = pages.first().and_then(|p| rs.scene.font(*p)) else {
            continue;
        };
        let width = first.measure(&t.text) * t.size;
        let x = (ndc.x * 0.5 + 0.5) * vp.0 - width * 0.5;
        let y = (0.5 - ndc.y * 0.5) * vp.1 - t.size * 0.6;
        for page in pages {
            out.push(og_render::Text {
                font: page,
                text: t.text.clone(),
                position: Vec3::new(x, y, 0.0),
                size: t.size,
                rotation: 0.0,
                color: t.color,
                layer: og_render::screen_layer(og_render::UI_LAYER),
                clip: None,
                tracking: 0.0,
                weight: 0.0,
                glow: None,
                placements: Vec::new(),
            });
        }
    }
    if out.is_empty() {
        return;
    }
    let o = rs
        .scene
        .ui
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(vp.0, vp.1, 0.0));
    o.texts.extend(out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Dispatch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
        og_render_sphere_geometry, og_render_standard_material, og_world_despawn, og_world_insert,
        og_world_remove, og_world_set, og_world_spawn, set_hooks,
    };
    use serde_json::json;

    struct Quiet;
    impl Hooks for Quiet {
        fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
    }

    fn value(text: String) -> Value {
        let v: Value = serde_json::from_str(&text).expect("wire JSON");
        assert_eq!(v["ok"], json!(true), "{text}");
        v["value"].clone()
    }

    /// The tick's raycast record, each transform's bits, against the world
    /// read afresh; and how many world matrices composing it took. The
    /// tick itself composes none: only a raycast from a system needs them.
    fn checked(w: u32, label: &str) -> usize {
        let eng = engine(w).unwrap();
        let mut rs = eng.render.borrow_mut();
        assert_eq!(
            rs.geo.hierarchy.composed, 0,
            "{label}: the tick composed world matrices no raycast asked for"
        );
        assert!(
            rs.geo.hierarchy.pending(),
            "{label}: the tick brought the hierarchy to the world before a raycast asked"
        );
        let GeoState {
            record, hierarchy, ..
        } = &mut rs.geo;
        let list = record.as_mut().expect("a record").1.targets(hierarchy);
        let mut fresh = Hierarchy::of(eng.world().unwrap());
        // The rows kept from earlier ticks are those read afresh.
        let mut models = Vec::new();
        let meshes = drawn(
            eng.world().unwrap(),
            &rs,
            &mut Hierarchy::default(),
            &mut Meshes::default(),
            &mut models,
        );
        let rows = |t: &[(u64, usize, bool, u32)]| t.to_vec();
        assert_eq!(
            rows(
                &list
                    .iter()
                    .map(|t| (t.entity, t.geometry, t.visible, t.layers))
                    .collect::<Vec<_>>()
            ),
            rows(
                &meshes
                    .iter()
                    .chain(&models)
                    .map(|d| (d.entity, d.geometry, d.visible, d.layers))
                    .collect::<Vec<_>>()
            ),
            "{label}: the record's meshes are not the world's"
        );
        for t in list.iter() {
            let want = fresh.world_of(t.entity).to_cols_array().map(f32::to_bits);
            assert_eq!(
                t.transform.to_cols_array().map(f32::to_bits),
                want,
                "{label}: entity {} kept a stale world matrix",
                t.entity
            );
        }
        rs.geo.hierarchy.composed
    }

    /// The raycast record's hierarchy read (the probe's before_tick, every
    /// tick) composes only the world matrices that moved, and stays bit for
    /// bit what a fresh read composes through moves, a moved parent,
    /// reparenting, a removed transform and a despawn. Before it was
    /// incremental, a tick where nothing moved composed all 350; before it
    /// waited for a raycast, every tick composed what moved (`checked`).
    #[test]
    fn the_raycast_record_composes_only_what_moved() {
        set_hooks(Rc::new(Quiet));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
        let geometry = value(crate::host_impl::answer(og_render_sphere_geometry(
            w,
            Some("{}".into()),
        )));
        let material = value(crate::host_impl::answer(og_render_standard_material(
            w,
            Some("{}".into()),
        )));
        let mesh = json!({ "geometry": geometry, "material": material }).to_string();
        let spawn = |x: f64, parent: Option<f64>| {
            let at = json!({ "position": { "x": x, "y": 0.5, "z": -x * 0.5 } }).to_string();
            let e = value(crate::host_impl::answer(og_world_spawn(w, Some(at))))
                .as_f64()
                .unwrap();
            value(crate::host_impl::answer(og_world_insert(
                w,
                e,
                "og.Mesh".into(),
                Some(mesh.clone()),
            )));
            if let Some(p) = parent {
                let v = json!({ "entity": p }).to_string();
                value(crate::host_impl::answer(og_world_insert(
                    w,
                    e,
                    "og.Parent".into(),
                    Some(v),
                )));
            }
            e
        };
        // 200 roots, a child under every other one and a grandchild under
        // every fourth: 350 meshes.
        let mut roots = Vec::new();
        let mut children = Vec::new();
        for i in 0..200 {
            let r = spawn(i as f64 * 1.5, None);
            roots.push(r);
            if i % 2 == 0 {
                let c = spawn(0.25, Some(r));
                children.push(c);
                if i % 4 == 0 {
                    spawn(0.125, Some(c));
                }
            }
        }
        let frame = || value(og_engine_frame(w, 1000.0 / 60.0));
        let step = |label: &str| {
            frame();
            checked(w, label)
        };
        assert_eq!(step("first"), 350);
        assert_eq!(step("still"), 0, "nothing moved");
        // A leaf root: one matrix.
        let to = |x: f64| json!({ "position": { "x": x, "y": 1, "z": 2 } }).to_string();
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[1],
            "og.Transform".into(),
            to(3.0),
        )));
        assert_eq!(step("one root moved"), 1);
        // A root with a child and a grandchild: three.
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[4],
            "og.Transform".into(),
            to(5.0),
        )));
        assert_eq!(step("a parent moved"), 3);
        // A child moved: it and its own child.
        value(crate::host_impl::answer(og_world_set(
            w,
            children[0],
            "og.Transform".into(),
            to(0.5),
        )));
        assert_eq!(step("a child moved"), 2);
        // Reparented: everything again, once.
        let v = json!({ "entity": roots[3] }).to_string();
        value(crate::host_impl::answer(og_world_insert(
            w,
            children[1],
            "og.Parent".into(),
            Some(v),
        )));
        assert_eq!(step("reparented"), 350);
        assert_eq!(step("still after"), 0);
        // A parent without its transform, then despawned.
        value(crate::host_impl::answer(og_world_remove(
            w,
            roots[8],
            "og.Transform".into(),
        )));
        step("transform removed");
        value(crate::host_impl::answer(og_world_despawn(w, roots[8])));
        step("despawned");
        assert_eq!(step("still at the end"), 0);
        // Ticks with no raycast between them: a root moved a tick before
        // one where nothing moves (which alone reads as still), then moved
        // and moved back.
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[1],
            "og.Transform".into(),
            to(9.0),
        )));
        frame();
        assert_eq!(step("moved a tick before a still one"), 1);
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[1],
            "og.Transform".into(),
            to(11.0),
        )));
        frame();
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[1],
            "og.Transform".into(),
            to(9.0),
        )));
        assert_eq!(step("moved and moved back"), 0);
        // Reparented a tick before a still one: everything again.
        let v = json!({ "entity": roots[6] }).to_string();
        value(crate::host_impl::answer(og_world_insert(
            w,
            children[1],
            "og.Parent".into(),
            Some(v),
        )));
        frame();
        assert_eq!(step("reparented a tick before a still one"), 347);
        // A mesh hidden and one given another geometry: the kept rows are
        // read again (`checked` compares them with the world's).
        let hidden = json!({ "visible": false }).to_string();
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[2],
            "og.Mesh".into(),
            hidden,
        )));
        let other = value(crate::host_impl::answer(og_render_sphere_geometry(
            w,
            Some(r#"{"radius":2}"#.into()),
        )));
        let swapped = json!({ "geometry": other }).to_string();
        value(crate::host_impl::answer(og_world_set(
            w,
            roots[5],
            "og.Mesh".into(),
            swapped,
        )));
        step("a mesh hidden, one swapped");
        let eng = engine(w).unwrap();
        let rs = eng.render.borrow();
        let Some((_, Record::Targets(list))) = &rs.geo.record else {
            panic!("composed by checked");
        };
        let of = |e: f64| list.iter().find(|t| t.entity == e as u64).expect("drawn");
        assert!(!of(roots[2]).visible);
        assert_ne!(of(roots[5]).geometry, of(roots[6]).geometry);
    }

    /// A read where nothing moved since the last one brought to the slots
    /// walks no entity and no slot, at a tick's first raycast and at a
    /// raycast between ticks (`perf_raycast_record_20k`: the walk was the
    /// still read's own work beside comparing the columns). One where
    /// something moved, at that read or at a tick with no raycast since,
    /// walks them, and the record stays a fresh read (`checked`).
    #[test]
    fn a_still_read_walks_no_entity() {
        set_hooks(Rc::new(Quiet));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
        let geometry = value(crate::host_impl::answer(og_render_sphere_geometry(
            w,
            Some("{}".into()),
        )));
        let material = value(crate::host_impl::answer(og_render_standard_material(
            w,
            Some("{}".into()),
        )));
        let mesh = json!({ "geometry": geometry, "material": material }).to_string();
        let mut all = Vec::new();
        for i in 0..60 {
            let at = json!({ "position": { "x": i as f64, "y": 0, "z": 0 } }).to_string();
            let e = value(crate::host_impl::answer(og_world_spawn(w, Some(at))))
                .as_f64()
                .unwrap();
            value(crate::host_impl::answer(og_world_insert(
                w,
                e,
                "og.Mesh".into(),
                Some(mesh.clone()),
            )));
            if i % 3 == 2 {
                let v = json!({ "entity": all[i - 1] }).to_string();
                value(crate::host_impl::answer(og_world_insert(
                    w,
                    e,
                    "og.Parent".into(),
                    Some(v),
                )));
            }
            all.push(e);
        }
        let frame = || value(og_engine_frame(w, 1000.0 / 60.0));
        let walked = |label: &str| {
            frame();
            checked(w, label);
            engine(w).unwrap().render.borrow().geo.hierarchy.walked
        };
        assert!(walked("first") >= 60);
        assert_eq!(walked("still"), 0, "nothing moved");
        let to = |x: f64| json!({ "position": { "x": x, "y": 1, "z": 2 } }).to_string();
        value(crate::host_impl::answer(og_world_set(
            w,
            all[4],
            "og.Transform".into(),
            to(3.0),
        )));
        assert!(walked("one moved") >= 60);
        assert_eq!(walked("still after"), 0);
        // Moved at a tick no raycast read, still at the next.
        value(crate::host_impl::answer(og_world_set(
            w,
            all[4],
            "og.Transform".into(),
            to(5.0),
        )));
        frame();
        assert!(walked("moved a tick before a still one") >= 60);
        // Every parented mesh loses its transform: their archetype empties
        // and leaves the query, the others' columns and the parents are
        // unchanged, and they are no longer the slots they were.
        for &e in all.iter().skip(2).step_by(3) {
            value(crate::host_impl::answer(og_world_remove(
                w,
                e,
                "og.Transform".into(),
            )));
        }
        assert!(walked("the last archetype emptied") >= 40);
        assert_eq!(walked("still after it"), 0);
        // A raycast between ticks reads the world as it is.
        let ray = json!({ "origin": { "x": 0, "y": 10, "z": 0 }, "direction": { "x": 0, "y": -1, "z": 0 } })
            .to_string();
        let between = || {
            crate::host_impl::answer(og_render_raycast(w, ray.clone(), None));
            engine(w).unwrap().render.borrow().geo.now.walked
        };
        assert!(between() >= 40);
        assert_eq!(between(), 0, "nothing moved between the raycasts");
        value(crate::host_impl::answer(og_world_set(
            w,
            all[7],
            "og.Transform".into(),
            to(1.0),
        )));
        assert!(between() >= 40);
    }
}
