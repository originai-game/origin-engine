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
use glam::{Mat4, Quat, Vec3};
use og_ecs::World;
use og_render::geometry::{self as geo, Bvh};
use og_render::{LineLayer, LineMode, Lines, MeshHandle, Points, debug};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

/// The debug views (`render.DebugView`), by variant index.
const VIEWS: [&str; 4] = ["none", "physics", "nav", "bounds"];

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
        // Every mesh is on layer 1 until per-object layers land.
        if d["layers"].as_u64().unwrap_or(1) & 1 == 0 {
            return Ok(json!([]));
        }
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
        for t in list.iter().filter(|t| t.visible || invisible) {
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
                        "entity": t.entity as f64,
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
            usage: "debug:on|off|none|physics|nav|bounds".into(),
            doc: "show or hide the debug overlay (og.debug's draws), or pick its view: physics (colliders, joints, contacts), nav (grids, navmeshes, agents' paths), bounds (every drawn mesh's box), none".into(),
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
            if !flag(&v, "visible", true) {
                continue;
            }
            let alpha = num(&v, "opacity", 1.0).clamp(0.0, 1.0);
            let Some((points, colors)) = vertices_of(rs, &v, alpha) else {
                continue;
            };
            let dash = num(&v, "dashSize", 0.0);
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
            if !flag(&v, "visible", true) {
                continue;
            }
            let alpha = num(&v, "opacity", 1.0).clamp(0.0, 1.0);
            let Some((positions, colors)) = vertices_of(rs, &v, alpha) else {
                continue;
            };
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
