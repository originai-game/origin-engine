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
//! transforms over, as og.physics.raycast's recorded colliders).

use super::api::{add_geometry, uv_repeat};
use super::assets::{self, index_of};
use super::engine::{Engine, engine, running};
use super::extract::{Hierarchy, RenderState};
use super::{ok, ok_void, parse, refuse};
use glam::{DVec2, DVec3, Mat4, Quat, Vec3};
use og_ecs::World;
use og_render::geometry::{self as geo, Bvh};
use og_render::{LineLayer, LineMode, Lines, MeshHandle, Points, debug};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::Arc;

/// The debug views (`render.DebugView`), by variant index.
const VIEWS: [&str; 5] = ["none", "physics", "nav", "bounds", "lod"];

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
    /// The drawn meshes as the current tick started (`before_tick`).
    record: Option<(u64, Rc<Vec<Target>>)>,
    hierarchy: Hierarchy,
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

fn uint(v: &Value, k: &str, d: u32, min: u32) -> u32 {
    v[k].as_u64().map_or(d, |x| x.min(1 << 16) as u32).max(min)
}

fn flag(v: &Value, k: &str, d: bool) -> bool {
    v[k].as_bool().unwrap_or(d)
}

fn vec3(v: &Value) -> Vec3 {
    Vec3::new(num(v, "x", 0.0), num(v, "y", 0.0), num(v, "z", 0.0))
}

fn quat(v: &Value) -> Quat {
    if !v.is_object() {
        return Quat::IDENTITY;
    }
    let q = Quat::from_xyzw(
        num(v, "x", 0.0),
        num(v, "y", 0.0),
        num(v, "z", 0.0),
        num(v, "w", 1.0),
    );
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

/// A list of Vector3, every one finite.
fn points_of(v: &Value, key: &str) -> Result<Vec<Vec3>, String> {
    let list = v[key].as_array().map(Vec::as_slice).unwrap_or_default();
    let out: Vec<Vec3> = list.iter().map(vec3).collect();
    if out.iter().any(|p| !p.is_finite()) {
        return Err(format!("{key} holds a point that is not finite"));
    }
    Ok(out)
}

fn colors_of(v: &Value, n: usize, alpha: f32) -> Result<Vec<[f32; 4]>, String> {
    let list = v["colors"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    if !list.is_empty() && list.len() != n {
        return Err(format!(
            "colors holds {} colours for {n} points: give one per point or none",
            list.len()
        ));
    }
    Ok(list.iter().map(|c| rgba(c, [1.0; 3], alpha)).collect())
}

/// A Float32Array attribute (a plain array on the wire; the TypeScript
/// runtime converts typed arrays).
fn floats(d: &Value, key: &str) -> Result<Option<Vec<f32>>, String> {
    match &d[key] {
        Value::Null => Ok(None),
        Value::Array(a) => a
            .iter()
            .map(|x| {
                x.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| format!("{key} holds something that is not a number"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Err(format!("{key} is not an array of numbers")),
    }
}

fn mesh_of_desc(d: &Value) -> Result<og_render::MeshData, String> {
    let positions = floats(d, "positions")?.unwrap_or_default();
    let normals = floats(d, "normals")?;
    let uvs = floats(d, "uvs")?;
    let colors = floats(d, "colors")?;
    let tangents = floats(d, "tangents")?;
    let indices: Option<Vec<u32>> = match &d["indices"] {
        Value::Null => None,
        Value::Array(a) => Some(
            a.iter()
                .map(|x| {
                    x.as_u64()
                        .filter(|&i| i <= u64::from(u32::MAX))
                        .map(|i| i as u32)
                        .ok_or("indices holds something that is not an unsigned integer")
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        _ => return Err("indices is not an array of numbers".into()),
    };
    geo::buffer_geometry(&geo::BufferAttributes {
        positions: &positions,
        normals: normals.as_deref(),
        uvs: uvs.as_deref(),
        colors: colors.as_deref(),
        tangents: tangents.as_deref(),
        indices: indices.as_deref(),
    })
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
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
pub(crate) fn adopt_cooked_bvh(
    rs: &mut RenderState,
    mesh: MeshHandle,
    prim: &og_cook::runtime::CookedPrimitive,
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

// ── geometry ────────────────────────────────────────────────────────────────

/// Adds a made geometry: drawn, its BVH built, and (`physics`) a Collider.mesh.
fn make(
    world: u32,
    mesh: og_render::MeshData,
    physics: bool,
    dynamic: bool,
) -> Result<Value, String> {
    let physics = physics && !mesh.indices.is_empty();
    let handle = add_geometry(world, mesh, physics)?;
    let eng = engine(world)?;
    let mut rs = eng.render.borrow_mut();
    let index = rs.geometries.len() - 1;
    let mesh = rs.geometries[index];
    let RenderState { geo: g, scene, .. } = &mut *rs;
    bvh_of(g, scene, mesh);
    if dynamic {
        g.dynamic.insert(index);
    }
    if physics {
        g.physics.insert(index);
    }
    Ok(handle)
}

pub fn og_render_buffer_geometry(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d = parse(desc.as_deref())?;
        let mesh = mesh_of_desc(&d)?;
        make(
            world,
            mesh,
            flag(&d, "physics", true),
            flag(&d, "dynamic", false),
        )
    })())
}

pub fn og_render_update_geometry(world: u32, geometry: f64, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let index = index_of(geometry).ok_or_else(|| format!("{geometry} is not a geometry"))?;
        let mesh_data = mesh_of_desc(&d)?;
        let physics = {
            let mut rs = eng.render.borrow_mut();
            let handle = *rs
                .geometries
                .get(index)
                .ok_or_else(|| format!("geometry {geometry} is not alive"))?;
            if !rs.geo.dynamic.contains(&index) {
                return Err(format!(
                    "geometry {geometry} is static: make it with og.render.bufferGeometry({{ dynamic: true }}) to update it"
                ));
            }
            let physics = rs.geo.physics.contains(&index) && !mesh_data.indices.is_empty();
            let for_physics = physics.then(|| super::modules::mesh_of(&mesh_data));
            rs.scene.set_mesh(handle, mesh_data);
            let RenderState { geo: g, scene, .. } = &mut *rs;
            bvh_of(g, scene, handle);
            for_physics
        };
        if let Some(m) = physics {
            super::modules::set_mesh(&eng, assets::handle(index), m)?;
        }
        Ok(())
    })())
}

pub fn og_render_geometry_info(world: u32, geometry: f64) -> String {
    wire((|| {
        let eng = engine(world)?;
        let index = index_of(geometry).ok_or_else(|| format!("{geometry} is not a geometry"))?;
        let rs = eng.render.borrow();
        let mesh = rs
            .geometries
            .get(index)
            .and_then(|h| rs.scene.mesh(*h))
            .ok_or_else(|| format!("geometry {geometry} is not alive"))?;
        let i = geo::info(mesh);
        let v = |p: Vec3| json!({ "x": p.x, "y": p.y, "z": p.z });
        Ok(json!({
            "vertices": i.vertices,
            "triangles": i.triangles,
            "min": v(i.min),
            "max": v(i.max),
            "dynamic": rs.geo.dynamic.contains(&index),
        }))
    })())
}

fn size(d: &Value, k: &str, def: f32) -> Result<f32, String> {
    let v = num(d, k, def);
    if !(v.is_finite() && v >= 0.0) {
        return Err(format!("{k} must be finite and at least 0, got {v}"));
    }
    Ok(v)
}

fn angle(d: &Value, k: &str, def: f32) -> Result<f32, String> {
    let v = num(d, k, def);
    if !v.is_finite() {
        return Err(format!("{k} must be finite, got {v}"));
    }
    Ok(v)
}

fn primitive(
    world: u32,
    desc: Option<String>,
    build: impl FnOnce(&Value) -> Result<og_render::MeshData, String>,
) -> String {
    wire((|| {
        let d = parse(desc.as_deref())?;
        let mut mesh = build(&d)?;
        uv_repeat(&d, &mut mesh)?;
        make(world, mesh, true, false)
    })())
}

const TAU: f32 = std::f32::consts::TAU;

pub fn og_render_cylinder_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::cylinder(&geo::primitives::Cylinder {
            radius_top: size(d, "radiusTop", 1.0)?,
            radius_bottom: size(d, "radiusBottom", 1.0)?,
            height: size(d, "height", 1.0)?,
            radial_segments: uint(d, "radialSegments", 32, 3),
            height_segments: uint(d, "heightSegments", 1, 1),
            open_ended: flag(d, "openEnded", false),
            theta_start: angle(d, "thetaStart", 0.0)?,
            theta_length: angle(d, "thetaLength", TAU)?,
        }))
    })
}

pub fn og_render_cone_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::cone(
            size(d, "radius", 1.0)?,
            size(d, "height", 1.0)?,
            uint(d, "radialSegments", 32, 3),
            uint(d, "heightSegments", 1, 1),
            flag(d, "openEnded", false),
        ))
    })
}

pub fn og_render_capsule_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::capsule(
            size(d, "radius", 1.0)?,
            size(d, "height", 1.0)?,
            uint(d, "capSegments", 4, 1),
            uint(d, "radialSegments", 8, 3),
            uint(d, "heightSegments", 1, 1),
        ))
    })
}

pub fn og_render_torus_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::torus(
            size(d, "radius", 1.0)?,
            size(d, "tube", 0.4)?,
            uint(d, "radialSegments", 12, 2),
            uint(d, "tubularSegments", 48, 3),
            angle(d, "arc", TAU)?,
        ))
    })
}

pub fn og_render_ring_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::ring(
            size(d, "innerRadius", 0.5)?,
            size(d, "outerRadius", 1.0)?,
            uint(d, "thetaSegments", 32, 3),
            uint(d, "phiSegments", 1, 1),
            angle(d, "thetaStart", 0.0)?,
            angle(d, "thetaLength", TAU)?,
        ))
    })
}

pub fn og_render_circle_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        Ok(geo::primitives::circle(
            size(d, "radius", 1.0)?,
            uint(d, "segments", 32, 3),
            angle(d, "thetaStart", 0.0)?,
            angle(d, "thetaLength", TAU)?,
        ))
    })
}

pub fn og_render_tube_geometry(world: u32, desc: Option<String>) -> String {
    primitive(world, desc, |d| {
        let path = d["path"].as_f64().filter(|&h| h != 0.0);
        if let Some(h) = path {
            let p = super::timeline::path_of(world, h as u64).ok_or_else(|| {
                format!("desc.path {h} is not a live path (og.path.create or og.path.find)")
            })?;
            return Ok(geo::primitives::tube_along(
                |u| {
                    let s = p.sample(u);
                    (s.position, s.tangent)
                },
                uint(d, "tubularSegments", 64, 1),
                size(d, "radius", 1.0)?,
                uint(d, "radialSegments", 8, 3),
                p.desc().closed,
            ));
        }
        let points = points_of(d, "points")?;
        if points.len() < 2 {
            return Err(format!(
                "a tube needs at least two points, got {}",
                points.len()
            ));
        }
        Ok(geo::primitives::tube(
            &points,
            uint(d, "tubularSegments", 64, 1),
            size(d, "radius", 1.0)?,
            uint(d, "radialSegments", 8, 3),
            flag(d, "closed", false),
        ))
    })
}

// ── procedural modelling (og-geom) ──────────────────────────────────────────

/// An og-geom mesh as the renderer holds one (tangents left to the
/// renderer, which generates them where a normal map needs them).
pub(crate) fn mesh_data(m: og_geom::Mesh) -> og_render::MeshData {
    let vertices = (0..m.positions.len())
        .map(|i| og_render::Vertex {
            position: m.positions[i],
            normal: m.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
            uv: m.uvs.get(i).copied().unwrap_or([0.0; 2]),
            color: m.colors.get(i).copied().unwrap_or([1.0; 4]),
            tangent: [0.0; 4],
        })
        .collect();
    og_render::MeshData {
        vertices,
        indices: m.indices,
    }
}

/// A live geometry's mesh, as og-geom reads one (colours only when some
/// vertex is not white).
fn geom_of(world: u32, geometry: f64, what: &str) -> Result<og_geom::Mesh, String> {
    let eng = engine(world)?;
    let index = index_of(geometry).ok_or_else(|| format!("{what} {geometry} is not a geometry"))?;
    let rs = eng.render.borrow();
    let mesh = rs
        .geometries
        .get(index)
        .and_then(|h| rs.scene.mesh(*h))
        .ok_or_else(|| format!("{what} {geometry} is not alive"))?;
    let v = &mesh.vertices;
    let colored = v.iter().any(|x| x.color != [1.0; 4]);
    Ok(og_geom::Mesh {
        positions: v.iter().map(|x| x.position).collect(),
        normals: v.iter().map(|x| x.normal).collect(),
        uvs: v.iter().map(|x| x.uv).collect(),
        colors: if colored {
            v.iter().map(|x| x.color).collect()
        } else {
            Vec::new()
        },
        indices: mesh.indices.clone(),
    })
}

/// A procedural geometry: built, refused when it holds no triangle, made
/// with the desc's `physics` and `dynamic`.
fn generated(
    world: u32,
    desc: Option<String>,
    build: impl FnOnce(&Value) -> Result<og_geom::Mesh, String>,
) -> String {
    wire((|| {
        let d = parse(desc.as_deref())?;
        let mesh = build(&d)?;
        if mesh.indices.is_empty() {
            return Err("the geometry made holds no triangle".into());
        }
        if mesh.positions.iter().flatten().any(|x| !x.is_finite()) {
            return Err("the geometry made holds a position that is not finite".into());
        }
        let mut data = mesh_data(mesh);
        uv_repeat(&d, &mut data)?;
        make(
            world,
            data,
            flag(&d, "physics", true),
            flag(&d, "dynamic", false),
        )
    })())
}

fn dvec2_list(v: &Value, what: &str) -> Result<Vec<DVec2>, String> {
    let list = v
        .as_array()
        .ok_or_else(|| format!("{what} is not a list of points"))?;
    let out: Vec<DVec2> = list
        .iter()
        .map(|p| {
            DVec2::new(
                p["x"].as_f64().unwrap_or(0.0),
                p["y"].as_f64().unwrap_or(0.0),
            )
        })
        .collect();
    if out.iter().any(|p| !p.is_finite()) {
        return Err(format!("{what} holds a point that is not finite"));
    }
    Ok(out)
}

/// `shapes` as the TypeScript runtime sends them (og.Shape and og.Path2
/// flattened to `{ shape, holes }`), or a plain points array.
fn shapes_of(d: &Value, default: &[[f64; 2]]) -> Result<Vec<og_geom::ShapePoints>, String> {
    let v = &d["shapes"];
    let fallback = || og_geom::ShapePoints {
        points: default.iter().map(|p| DVec2::new(p[0], p[1])).collect(),
        holes: Vec::new(),
    };
    let Some(list) = v.as_array() else {
        return if v.is_null() {
            Ok(vec![fallback()])
        } else {
            Err("shapes is not an og.Shape, a list of them, or a list of points".into())
        };
    };
    if list.first().is_some_and(|p| p.get("x").is_some()) {
        return Ok(vec![og_geom::ShapePoints {
            points: dvec2_list(v, "shapes")?,
            holes: Vec::new(),
        }]);
    }
    list.iter()
        .enumerate()
        .map(|(i, s)| {
            let holes = match s["holes"].as_array() {
                Some(h) => h
                    .iter()
                    .map(|x| dvec2_list(x, &format!("shapes[{i}].holes")))
                    .collect::<Result<_, _>>()?,
                None => Vec::new(),
            };
            Ok(og_geom::ShapePoints {
                points: dvec2_list(&s["shape"], &format!("shapes[{i}]"))?,
                holes,
            })
        })
        .collect()
}

/// A live og.path as og-geom samples one: its point and unit tangent at a
/// fraction of its length.
fn path_sampler(
    world: u32,
    v: &Value,
    what: &str,
) -> Result<Option<Arc<og_timeline::Path>>, String> {
    let Some(h) = v.as_f64().filter(|&h| h != 0.0) else {
        return Ok(None);
    };
    super::timeline::path_of(world, h as u64)
        .map(Some)
        .ok_or_else(|| format!("{what} {h} is not a live path (og.path.create or og.path.find)"))
}

fn sample_path(p: &og_timeline::Path, t: f64) -> (DVec3, DVec3) {
    let s = p.sample(t as f32);
    (s.position.as_dvec3(), s.tangent.as_dvec3())
}

fn fsize(d: &Value, k: &str, def: f64) -> Result<f64, String> {
    Ok(f64::from(size(d, k, def as f32)?))
}

fn fnum(d: &Value, k: &str, def: f64) -> Result<f64, String> {
    let v = d[k].as_f64().unwrap_or(def);
    if !v.is_finite() {
        return Err(format!("{k} must be finite, got {v}"));
    }
    Ok(v)
}

pub fn og_render_shape_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let shapes = shapes_of(d, &[[0.0, 0.5], [-0.5, -0.5], [0.5, -0.5]])?;
        Ok(og_geom::shape_geometry(&shapes))
    })
}

pub fn og_render_extrude_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let shapes = shapes_of(d, &[[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]])?;
        let thickness = fsize(d, "bevelThickness", 0.2)?;
        let o = og_geom::Extrude {
            depth: fsize(d, "depth", 1.0)?,
            steps: uint(d, "steps", 1, 1),
            bevel_enabled: flag(d, "bevelEnabled", true),
            bevel_thickness: thickness,
            bevel_size: fnum(d, "bevelSize", thickness - 0.1)?,
            bevel_offset: fnum(d, "bevelOffset", 0.0)?,
            bevel_segments: uint(d, "bevelSegments", 3, 1),
        };
        let path = path_sampler(world, &d["extrudePath"], "desc.extrudePath")?;
        let sample = |t: f64| sample_path(path.as_deref().expect("a path"), t);
        let along = path.as_ref().map(|p| og_geom::extrude::ExtrudePath {
            sample: &sample,
            closed: p.desc().closed,
        });
        Ok(og_geom::extrude(&shapes, &o, along.as_ref()))
    })
}

pub fn og_render_lathe_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let mut l = og_geom::Lathe::default();
        if !d["points"].is_null() {
            l.points = dvec2_list(&d["points"], "points")?;
        }
        if l.points.len() < 2 {
            return Err(format!(
                "a lathe needs at least two points, got {}",
                l.points.len()
            ));
        }
        l.segments = uint(d, "segments", 12, 1);
        l.phi_start = fnum(d, "phiStart", 0.0)?;
        l.phi_length = fnum(d, "phiLength", og_geom::trig::TAU)?;
        Ok(og_geom::lathe(&l))
    })
}

fn detail(d: &Value) -> u32 {
    d["detail"].as_u64().map_or(0, |x| x.min(64) as u32)
}

pub fn og_render_polyhedron_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let vertices: Vec<f64> = d["vertices"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|x| {
                x.as_f64()
                    .filter(|v| v.is_finite())
                    .ok_or("vertices holds something that is not a finite number")
            })
            .collect::<Result<_, _>>()?;
        let indices: Vec<u32> = d["indices"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|x| {
                x.as_u64()
                    .filter(|&i| i <= u64::from(u32::MAX))
                    .map(|i| i as u32)
                    .ok_or("indices holds something that is not an unsigned integer")
            })
            .collect::<Result<_, _>>()?;
        if indices.is_empty() {
            return Err("a polyhedron needs at least one face (indices)".into());
        }
        og_geom::polyhedron(&og_geom::Polyhedron {
            vertices,
            indices,
            radius: fsize(d, "radius", 1.0)?,
            detail: detail(d),
        })
    })
}

fn solid(world: u32, desc: Option<String>, make: fn(f64, u32) -> og_geom::Mesh) -> String {
    generated(world, desc, |d| {
        Ok(make(fsize(d, "radius", 1.0)?, detail(d)))
    })
}

pub fn og_render_icosahedron_geometry(world: u32, desc: Option<String>) -> String {
    solid(world, desc, og_geom::icosahedron)
}

pub fn og_render_dodecahedron_geometry(world: u32, desc: Option<String>) -> String {
    solid(world, desc, og_geom::dodecahedron)
}

pub fn og_render_octahedron_geometry(world: u32, desc: Option<String>) -> String {
    solid(world, desc, og_geom::octahedron)
}

pub fn og_render_tetrahedron_geometry(world: u32, desc: Option<String>) -> String {
    solid(world, desc, og_geom::tetrahedron)
}

fn keys(d: &Value, k: &str) -> Vec<og_geom::Key> {
    d[k].as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|x| og_geom::Key {
            t: x["t"].as_f64().unwrap_or(0.0),
            value: x["value"].as_f64().unwrap_or(0.0),
        })
        .collect()
}

pub fn og_render_sweep_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let path = path_sampler(world, &d["path"], "desc.path")?
            .ok_or("a sweep needs a path (og.path.create or og.path.find)")?;
        // A Shape or Path2 crosses as `{ shape, holes }`; its holes are not swept.
        let section = match &d["section"] {
            v if v.get("shape").is_some() => dvec2_list(&v["shape"], "section")?,
            Value::Null => Vec::new(),
            v => dvec2_list(v, "section")?,
        };
        let s = og_geom::Sweep {
            section,
            steps: uint(d, "steps", 64, 1),
            scale: keys(d, "scale"),
            twist: keys(d, "twist"),
            caps: flag(d, "caps", true),
        };
        let sample = |t: f64| sample_path(&path, t);
        og_geom::sweep(
            &s,
            &og_geom::extrude::ExtrudePath {
                sample: &sample,
                closed: path.desc().closed,
            },
        )
    })
}

/// A GeometryPart: its geometry's mesh and where it goes.
fn part_of(
    world: u32,
    v: &Value,
    what: &str,
) -> Result<(og_geom::Mesh, og_geom::Placement, og_geom::Mirror), String> {
    let mesh = geom_of(
        world,
        v["geometry"].as_f64().unwrap_or(0.0),
        &format!("{what}.geometry"),
    )?;
    let p = vec3(&v["position"]);
    let s = &v["scale"];
    let scale = Vec3::new(num(s, "x", 1.0), num(s, "y", 1.0), num(s, "z", 1.0));
    if !p.is_finite() || !scale.is_finite() {
        return Err(format!("{what}'s position and scale must be finite"));
    }
    let q = quat(&v["quaternion"]);
    let mirror = match &v["mirror"] {
        Value::String(s) if s == "x" => 1,
        Value::String(s) if s == "y" => 2,
        Value::String(s) if s == "z" => 3,
        Value::Number(n) => n.as_u64().unwrap_or(0),
        _ => 0,
    };
    Ok((
        mesh,
        og_geom::Placement {
            position: p.as_dvec3(),
            quaternion: q.as_dquat(),
            scale: scale.as_dvec3(),
        },
        og_geom::Mirror {
            x: mirror == 1,
            y: mirror == 2,
            z: mirror == 3,
        },
    ))
}

/// A part placed and mirrored, as one mesh.
fn placed(
    part: &(og_geom::Mesh, og_geom::Placement, og_geom::Mirror),
    what: &str,
) -> Result<og_geom::Mesh, String> {
    og_geom::merge(&[og_geom::Part {
        mesh: &part.0,
        placement: part.1,
        mirror: part.2,
    }])
    .map_err(|e| e.replacen("part 0", what, 1))
}

pub fn og_render_csg_geometry(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let op = match &d["op"] {
            Value::String(s) if s == "subtract" => og_geom::CsgOp::Subtract,
            Value::String(s) if s == "intersect" => og_geom::CsgOp::Intersect,
            Value::Number(n) if n.as_u64() == Some(1) => og_geom::CsgOp::Subtract,
            Value::Number(n) if n.as_u64() == Some(2) => og_geom::CsgOp::Intersect,
            _ => og_geom::CsgOp::Union,
        };
        let a = placed(&part_of(world, &d["a"], "desc.a")?, "desc.a")?;
        let b = placed(&part_of(world, &d["b"], "desc.b")?, "desc.b")?;
        let none = og_geom::Placement::default();
        og_geom::csg(&a, none, &b, none, op)
    })
}

pub fn og_render_merge_geometries(world: u32, desc: Option<String>) -> String {
    generated(world, desc, |d| {
        let list = d["parts"].as_array().map(Vec::as_slice).unwrap_or_default();
        if list.is_empty() {
            return Err("mergeGeometries needs at least one part".into());
        }
        let parts = list
            .iter()
            .enumerate()
            .map(|(i, p)| part_of(world, p, &format!("desc.parts[{i}]")))
            .collect::<Result<Vec<_>, _>>()?;
        let refs: Vec<og_geom::Part> = parts
            .iter()
            .map(|(mesh, placement, mirror)| og_geom::Part {
                mesh,
                placement: *placement,
                mirror: *mirror,
            })
            .collect();
        og_geom::merge(&refs)
            .map_err(|e| e.replacen("part ", "desc.parts[", 1).replacen(':', "]:", 1))
    })
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

pub fn og_render_lines(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let points = points_of(&d, "points")?;
        if points.len() < 2 {
            return Err(format!(
                "lines need at least two points, got {}",
                points.len()
            ));
        }
        let alpha = num(&d, "opacity", 1.0).clamp(0.0, 1.0);
        let colors = colors_of(&d, points.len(), alpha)?;
        let screen = flag(&d, "screen", false);
        let lines = Lines {
            colors,
            color: rgba(&d["color"], [1.0; 3], alpha),
            mode: line_mode(&d["mode"]),
            width: size(&d, "width", 1.0)?,
            world_width: flag(&d, "worldUnits", false) && !screen,
            depth_test: flag(&d, "depthTest", true) && !screen,
            screen,
            dashed: flag(&d, "dashed", false).then(|| {
                (
                    num(&d, "dashSize", 3.0).max(0.0),
                    num(&d, "gapSize", 1.0).max(0.0),
                )
            }),
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
    })())
}

pub fn og_render_points(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let positions = points_of(&d, "points")?;
        let alpha = num(&d, "opacity", 1.0).clamp(0.0, 1.0);
        let colors = colors_of(&d, positions.len(), alpha)?;
        let screen = flag(&d, "screen", false);
        let points = Points {
            positions,
            colors,
            color: rgba(&d["color"], [1.0; 3], alpha),
            size: size(&d, "size", 1.0)?,
            size_attenuation: flag(&d, "sizeAttenuation", true) && !screen,
            depth_test: flag(&d, "depthTest", true) && !screen,
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
    })())
}

// ── raycast ─────────────────────────────────────────────────────────────────

/// Every og.Mesh and og.Model part as drawn from the world now.
fn targets(world: &World, rs: &RenderState, h: &mut Hierarchy) -> Vec<Target> {
    let mut out = Vec::new();
    h.refresh(world);
    if let Some(layout) = &rs.draw.mesh {
        for r in super::extract::mesh_rows(world, layout) {
            let Some(&mesh) = rs.geometries.get(r.geometry) else {
                continue;
            };
            out.push(Target {
                entity: r.entity,
                geometry: r.geometry,
                mesh,
                transform: h.world_of(r.entity),
                visible: r.visible,
                layers: r.layers,
            });
        }
    }
    if let Some(layout) = &rs.draw.model {
        for (e, v) in super::fx::rows(world, Some(layout)) {
            let asset = v["asset"].as_f64().unwrap_or(0.0) as u64;
            let Some(parts) = rs.models.get(&asset) else {
                continue;
            };
            let m = h.world_of(e);
            let visible = flag(&v, "visible", true);
            let layers = v["layers"].as_u64().map_or(1, |x| x as u32);
            for &(mesh, _, local) in parts.iter() {
                out.push(Target {
                    entity: e,
                    geometry: rs
                        .geometries
                        .iter()
                        .position(|g| *g == mesh)
                        .unwrap_or(usize::MAX),
                    mesh,
                    transform: m * local,
                    visible,
                    layers,
                });
            }
        }
    }
    out
}

/// The probe's tick start: the drawn meshes as the tick starts, for a
/// raycast from inside a system.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let tick = world.time().tick;
    let mut h = std::mem::take(&mut rs.geo.hierarchy);
    let list = targets(world, &rs, &mut h);
    rs.geo.hierarchy = h;
    rs.geo.record = Some((tick, Rc::new(list)));
}

pub fn og_render_raycast(world: u32, ray: String, desc: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        let r = parse(Some(&ray))?;
        let d = parse(desc.as_deref())?;
        let origin = vec3(&r["origin"]);
        let dir = vec3(&r["direction"]);
        if !origin.is_finite() || !dir.is_finite() || dir.length_squared() < 1e-20 {
            return Err(
                "the ray's origin and direction must be finite and its direction not zero".into(),
            );
        }
        let dir = dir.normalize();
        let max = num(&d, "maxDistance", 0.0);
        let max = if max > 0.0 && max.is_finite() {
            max
        } else {
            f32::INFINITY
        };
        let first = flag(&d, "first", false);
        let invisible = flag(&d, "invisible", false);
        // Three.js layers: a mesh is tested when its mask shares a bit
        // (`super::views`).
        let layers = d["layers"].as_u64().map_or(1, |x| x as u32);
        let list: Rc<Vec<Target>> = if running(eng.id).is_some() {
            let rs = eng.render.borrow();
            match &rs.geo.record {
                Some((_, l)) => l.clone(),
                None => Rc::new(Vec::new()),
            }
        } else {
            let mut rs = eng.render.borrow_mut();
            let mut h = std::mem::take(&mut rs.geo.hierarchy);
            let l = targets(eng.world()?, &rs, &mut h);
            rs.geo.hierarchy = h;
            Rc::new(l)
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
                    json!({
                        "entity": t.entity,
                        "geometry": if t.geometry == usize::MAX { 0.0 } else { assets::handle(t.geometry) as f64 },
                        "distance": hit.t,
                        "point": { "x": p.x, "y": p.y, "z": p.z },
                        "normal": { "x": n.x, "y": n.y, "z": n.z },
                        "uv": { "x": hit.uv[0], "y": hit.uv[1] },
                        "faceIndex": hit.face,
                    }),
                ));
                if first {
                    reach = reach.min(hit.t);
                }
            }
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        if first {
            hits.truncate(1);
        }
        Ok(Value::Array(hits.into_iter().map(|(_, v)| v).collect()))
    })())
}

// ── og.debug ────────────────────────────────────────────────────────────────

struct DrawDesc {
    color: [f32; 4],
    width: f32,
    until: u64,
    rotation: Quat,
}

fn draw_desc(eng: &Engine, desc: Option<String>) -> DrawDesc {
    let d = parse(desc.as_deref()).unwrap_or(Value::Null);
    let w = num(&d, "width", 2.0);
    DrawDesc {
        color: rgba(&d["color"], [1.0, 1.0, 0.0], 1.0),
        width: if w.is_finite() { w.max(0.0) } else { 2.0 },
        // Drawn at the frame after the tick that asked (its world reads
        // the next tick), and `duration` ticks past it.
        until: tick(eng) + 1 + d["duration"].as_u64().unwrap_or(0),
        rotation: quat(&d["rotation"]),
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
    vec3(&parse(Some(text)).unwrap_or(Value::Null))
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
        let v = parse(Some(&view)).unwrap_or(Value::String(view));
        eng.render.borrow_mut().geo.view = Some(view_of(&v));
    }
}

pub fn og_debug_view(world: u32) -> String {
    let i = engine(world).map_or(0, |eng| {
        let rs = eng.render.borrow();
        rs.geo.view_index(&rs.options)
    });
    json!(VIEWS[i]).to_string()
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
            usage: "debug:on|off|none|physics|nav|bounds|lod".into(),
            doc: "show or hide the debug overlay (og.debug's draws), or pick its view: physics (colliders, joints, contacts), nav (grids, navmeshes, agents' paths), bounds (every drawn mesh's box), lod (every model tinted by its level of detail), none".into(),
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
    rs.scene.css_height = vp.1;
    let now = world.time().tick;
    // Immediate sets of the tick drawn (or the one before it, whose frame
    // this is).
    if rs.geo.tick == now || rs.geo.tick + 1 == now {
        rs.scene.lines.extend(rs.geo.lines.iter().cloned());
        rs.scene.points.extend(rs.geo.points.iter().cloned());
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
            let at = rs.scene.lines.len();
            if layers != 1 {
                rs.scene.line_layers.resize(at, 1);
                rs.scene.line_layers.push(layers);
            }
            rs.scene.lines.push(Lines {
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
            let at = rs.scene.points.len();
            if layers != 1 {
                rs.scene.point_layers.resize(at, 1);
                rs.scene.point_layers.push(layers);
            }
            rs.scene.points.push(Points {
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
    if rs.geo.disabled {
        return;
    }
    let draws: Vec<Lines> = rs.geo.draws.iter().map(|(_, l)| l.clone()).collect();
    rs.scene.lines.extend(draws);
    match VIEWS[rs.geo.view_index(&rs.options)] {
        "physics" => physics_view(rs),
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
    rs.scene.lines.push(set);
}

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
            .lines
            .push(debug::segments(edges, [0.2, 0.75, 1.0, 1.0], 1.0));
    }
    if !blocked.is_empty() {
        rs.scene
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
            rs.scene.lines.push(l);
        }
    }
}

fn bounds_view(rs: &mut RenderState) {
    let mut out = Vec::new();
    let instances: Vec<(MeshHandle, Mat4)> = rs
        .scene
        .instances
        .iter()
        .map(|i| (i.mesh, i.transform))
        .collect();
    for (mesh, m) in instances {
        let rev = rs.scene.mesh_revision(mesh.index());
        let (min, max) = match rs.geo.bounds.get(&mesh.index()) {
            Some(&(r, a, b)) if r == rev => (a, b),
            _ => {
                let Some(data) = rs.scene.mesh(mesh) else {
                    continue;
                };
                let i = geo::info(data);
                rs.geo.bounds.insert(mesh.index(), (rev, i.min, i.max));
                (i.min, i.max)
            }
        };
        let centre = (min + max) * 0.5;
        out.extend(debug::box_edges(
            m * Mat4::from_translation(centre),
            (max - min) * 0.5,
        ));
    }
    if !out.is_empty() {
        rs.scene
            .lines
            .push(debug::segments(out, [0.3, 1.0, 0.45, 1.0], 1.0));
    }
}

/// Debug labels: each world point projected to the play surface, the
/// label centred above it in the UI's body font.
fn texts(rs: &mut RenderState, vp: (f32, f32)) {
    if rs.geo.texts.is_empty() || vp.0 <= 0.0 || vp.1 <= 0.0 {
        return;
    }
    let cam = &rs.scene.camera;
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
    /// read afresh; and how many world matrices that tick's read composed.
    fn checked(w: u32, label: &str) -> usize {
        let eng = engine(w).unwrap();
        let rs = eng.render.borrow();
        let (_, list) = rs.geo.record.as_ref().expect("a record");
        let mut fresh = Hierarchy::of(eng.world().unwrap());
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
    /// incremental, a tick where nothing moved composed all 350.
    #[test]
    fn the_raycast_record_composes_only_what_moved() {
        set_hooks(Rc::new(Quiet));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
        let geometry = value(og_render_sphere_geometry(w, Some("{}".into())));
        let material = value(og_render_standard_material(w, Some("{}".into())));
        let mesh = json!({ "geometry": geometry, "material": material }).to_string();
        let spawn = |x: f64, parent: Option<f64>| {
            let at = json!({ "position": { "x": x, "y": 0.5, "z": -x * 0.5 } }).to_string();
            let e = value(og_world_spawn(w, Some(at))).as_f64().unwrap();
            value(og_world_insert(w, e, "og.Mesh".into(), Some(mesh.clone())));
            if let Some(p) = parent {
                let v = json!({ "entity": p }).to_string();
                value(og_world_insert(w, e, "og.Parent".into(), Some(v)));
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
        value(og_world_set(w, roots[1], "og.Transform".into(), to(3.0)));
        assert_eq!(step("one root moved"), 1);
        // A root with a child and a grandchild: three.
        value(og_world_set(w, roots[4], "og.Transform".into(), to(5.0)));
        assert_eq!(step("a parent moved"), 3);
        // A child moved: it and its own child.
        value(og_world_set(w, children[0], "og.Transform".into(), to(0.5)));
        assert_eq!(step("a child moved"), 2);
        // Reparented: everything again, once.
        let v = json!({ "entity": roots[3] }).to_string();
        value(og_world_insert(w, children[1], "og.Parent".into(), Some(v)));
        assert_eq!(step("reparented"), 350);
        assert_eq!(step("still after"), 0);
        // A parent without its transform, then despawned.
        value(og_world_remove(w, roots[8], "og.Transform".into()));
        step("transform removed");
        value(og_world_despawn(w, roots[8]));
        step("despawned");
        assert_eq!(step("still at the end"), 0);
    }
}
