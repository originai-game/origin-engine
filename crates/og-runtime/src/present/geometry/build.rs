//! Buffer geometry, primitives and procedural modelling calls.

use super::*;

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

pub fn og_render_buffer_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let d: wire::BufferGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let mesh = mesh_of_desc(&d)?;
        make(world, mesh, d.physics, d.dynamic)
    })()
}

pub fn og_render_update_geometry(
    world: u32,
    geometry: f64,
    desc: Option<String>,
) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::BufferGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
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
    })()
}

pub fn og_render_geometry_info(
    world: u32,
    geometry: f64,
) -> Result<crate::host::wire::GeometryInfo, String> {
    (|| {
        let eng = engine(world)?;
        let index = index_of(geometry).ok_or_else(|| format!("{geometry} is not a geometry"))?;
        let rs = eng.render.borrow();
        let mesh = rs
            .geometries
            .get(index)
            .and_then(|h| rs.scene.mesh(*h))
            .ok_or_else(|| format!("geometry {geometry} is not alive"))?;
        let i = geo::info(mesh);
        Ok(wire::GeometryInfo {
            vertices: i.vertices,
            triangles: i.triangles,
            min: i.min,
            max: i.max,
            dynamic: rs.geo.dynamic.contains(&index),
        })
    })()
}

/// `v` (field `k`), refused unless finite and at least 0.
pub(super) fn size(v: f32, k: &str) -> Result<f32, String> {
    if !(v.is_finite() && v >= 0.0) {
        return Err(format!("{k} must be finite and at least 0, got {v}"));
    }
    Ok(v)
}

/// `v` (field `k`), refused unless finite.
fn angle(v: f32, k: &str) -> Result<f32, String> {
    if !v.is_finite() {
        return Err(format!("{k} must be finite, got {v}"));
    }
    Ok(v)
}

/// A segment count, at least `min` and at most 65536.
fn segments(n: u32, min: u32) -> u32 {
    n.min(1 << 16).max(min)
}

/// A three.js primitive's desc: its `uvRepeat`.
trait Primitive {
    fn uv_repeat(&self) -> Vec2;
}

macro_rules! primitive_desc {
    ($($t:ident),*) => {$(
        impl Primitive for wire::$t {
            fn uv_repeat(&self) -> Vec2 {
                self.uv_repeat
            }
        }
    )*};
}

primitive_desc!(
    CylinderGeometryDesc,
    ConeGeometryDesc,
    CapsuleGeometryDesc,
    TorusGeometryDesc,
    RingGeometryDesc,
    CircleGeometryDesc,
    TubeGeometryDesc
);

/// A three.js primitive, made with its desc's `uvRepeat`.
fn primitive<D: wire::FromJson + Default + Primitive>(
    world: u32,
    desc: Option<String>,
    build: impl FnOnce(D) -> Result<og_render::MeshData, String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let d: D = wire::arg(desc.as_deref(), "desc")?;
        let repeat = d.uv_repeat();
        let mut mesh = build(d)?;
        uv_repeat(repeat, &mut mesh)?;
        make(world, mesh, true, false)
    })()
}

pub fn og_render_cylinder_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::CylinderGeometryDesc| {
        Ok(geo::primitives::cylinder(&geo::primitives::Cylinder {
            radius_top: size(d.radius_top, "radiusTop")?,
            radius_bottom: size(d.radius_bottom, "radiusBottom")?,
            height: size(d.height, "height")?,
            radial_segments: segments(d.radial_segments, 3),
            height_segments: segments(d.height_segments, 1),
            open_ended: d.open_ended,
            theta_start: angle(d.theta_start, "thetaStart")?,
            theta_length: angle(d.theta_length, "thetaLength")?,
        }))
    })
}

pub fn og_render_cone_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::ConeGeometryDesc| {
        Ok(geo::primitives::cone(
            size(d.radius, "radius")?,
            size(d.height, "height")?,
            segments(d.radial_segments, 3),
            segments(d.height_segments, 1),
            d.open_ended,
        ))
    })
}

pub fn og_render_capsule_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::CapsuleGeometryDesc| {
        Ok(geo::primitives::capsule(
            size(d.radius, "radius")?,
            size(d.height, "height")?,
            segments(d.cap_segments, 1),
            segments(d.radial_segments, 3),
            segments(d.height_segments, 1),
        ))
    })
}

pub fn og_render_torus_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::TorusGeometryDesc| {
        Ok(geo::primitives::torus(
            size(d.radius, "radius")?,
            size(d.tube, "tube")?,
            segments(d.radial_segments, 2),
            segments(d.tubular_segments, 3),
            angle(d.arc, "arc")?,
        ))
    })
}

pub fn og_render_ring_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::RingGeometryDesc| {
        Ok(geo::primitives::ring(
            size(d.inner_radius, "innerRadius")?,
            size(d.outer_radius, "outerRadius")?,
            segments(d.theta_segments, 3),
            segments(d.phi_segments, 1),
            angle(d.theta_start, "thetaStart")?,
            angle(d.theta_length, "thetaLength")?,
        ))
    })
}

pub fn og_render_circle_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::CircleGeometryDesc| {
        Ok(geo::primitives::circle(
            size(d.radius, "radius")?,
            segments(d.segments, 3),
            angle(d.theta_start, "thetaStart")?,
            angle(d.theta_length, "thetaLength")?,
        ))
    })
}

pub fn og_render_tube_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    primitive(world, desc, |d: wire::TubeGeometryDesc| {
        if d.path != 0 {
            let h = d.path;
            let p = super::timeline::path_of(world, h).ok_or_else(|| {
                format!("desc.path {h} is not a live path (og.path.create or og.path.find)")
            })?;
            return Ok(geo::primitives::tube_along(
                |u| {
                    let s = p.sample(u);
                    (s.position, s.tangent)
                },
                segments(d.tubular_segments, 1),
                size(d.radius, "radius")?,
                segments(d.radial_segments, 3),
                p.desc().closed,
            ));
        }
        let points = finite_points(d.points, "points")?;
        if points.len() < 2 {
            return Err(format!(
                "a tube needs at least two points, got {}",
                points.len()
            ));
        }
        Ok(geo::primitives::tube(
            &points,
            segments(d.tubular_segments, 1),
            size(d.radius, "radius")?,
            segments(d.radial_segments, 3),
            d.closed,
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

/// A procedural geometry's desc: the fields every one carries.
trait Made {
    /// `uvRepeat`, `physics`, `dynamic`.
    fn made(&self) -> (Vec2, bool, bool);
}

macro_rules! made {
    ($($t:ident),*) => {$(
        impl Made for wire::$t {
            fn made(&self) -> (Vec2, bool, bool) {
                (self.uv_repeat, self.physics, self.dynamic)
            }
        }
    )*};
}

made!(
    ShapeGeometryDesc,
    ExtrudeGeometryDesc,
    LatheGeometryDesc,
    PolyhedronGeometryDesc,
    SolidGeometryDesc,
    SweepGeometryDesc,
    CsgGeometryDesc,
    MergeGeometriesDesc
);

/// A procedural geometry: built from its desc, refused when it holds no
/// triangle, made with the desc's `uvRepeat`, `physics` and `dynamic`.
fn generated<D: wire::FromJson + Default + Made>(
    world: u32,
    desc: Option<String>,
    build: impl FnOnce(&D) -> Result<og_geom::Mesh, String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let d: D = wire::arg(desc.as_deref(), "desc")?;
        let mesh = build(&d)?;
        if mesh.indices.is_empty() {
            return Err("the geometry made holds no triangle".into());
        }
        if mesh.positions.iter().flatten().any(|x| !x.is_finite()) {
            return Err("the geometry made holds a position that is not finite".into());
        }
        let mut data = mesh_data(mesh);
        let (repeat, physics, dynamic) = d.made();
        if !repeat.is_finite() {
            return Err(format!("uvRepeat must be finite, got {repeat}"));
        }
        og_render::shapes::repeat_uv(&mut data, repeat);
        make(world, data, physics, dynamic)
    })()
}

/// Points as og-geom reads them, refused when one is not finite.
fn dvec2s(points: &[Vec2], what: &str) -> Result<Vec<DVec2>, String> {
    if let Some(i) = points.iter().position(|p| !p.is_finite()) {
        return Err(format!("{what}[{i}] is not a finite point"));
    }
    Ok(points.iter().map(|p| p.as_dvec2()).collect())
}

/// A ShapePoints as og-geom reads one.
fn shape_points(s: &wire::ShapePoints, what: &str) -> Result<og_geom::ShapePoints, String> {
    Ok(og_geom::ShapePoints {
        points: dvec2s(&s.shape, &format!("{what}.shape"))?,
        holes: s
            .holes
            .iter()
            .enumerate()
            .map(|(i, h)| dvec2s(h, &format!("{what}.holes[{i}]")))
            .collect::<Result<_, _>>()?,
    })
}

/// The refusal of an og.Shape or og.Path2 that crossed as a handle: the
/// TypeScript runtime flattens them to points (geom.ts).
fn unflattened(what: &str) -> String {
    format!("{what}: an og.Shape or og.Path2 crosses as its points (ShapePoints); got a number")
}

/// ShapeGeometryDesc.shapes and ExtrudeGeometryDesc.shapes as they read.
type Shapes =
    wire::OneOf<u32, wire::OneOf<Vec<u32>, wire::OneOf<Vec<Vec2>, Vec<wire::ShapePoints>>>>;

/// `shapes` (`desc.shapes`): the outline's points or a list of ShapePoints,
/// as the TypeScript runtime sends an og.Shape, a list of them or points;
/// absent, `default`.
fn shapes_of(
    shapes: &Option<Shapes>,
    default: &[[f64; 2]],
) -> Result<Vec<og_geom::ShapePoints>, String> {
    use wire::OneOf::{A, B};
    match shapes {
        None => Ok(vec![og_geom::ShapePoints {
            points: default.iter().map(|p| DVec2::new(p[0], p[1])).collect(),
            holes: Vec::new(),
        }]),
        Some(A(_)) => Err(unflattened("desc.shapes")),
        Some(B(A(list))) if list.is_empty() => Ok(Vec::new()),
        Some(B(A(_))) => Err(unflattened("desc.shapes")),
        Some(B(B(A(points)))) => Ok(vec![og_geom::ShapePoints {
            points: dvec2s(points, "desc.shapes")?,
            holes: Vec::new(),
        }]),
        Some(B(B(B(list)))) => list
            .iter()
            .enumerate()
            .map(|(i, s)| shape_points(s, &format!("desc.shapes[{i}]")))
            .collect(),
    }
}

/// A live og.path as og-geom samples one: its point and unit tangent at a
/// fraction of its length.
fn path_sampler(world: u32, h: u64, what: &str) -> Result<Option<Arc<og_timeline::Path>>, String> {
    if h == 0 {
        return Ok(None);
    }
    super::timeline::path_of(world, h)
        .map(Some)
        .ok_or_else(|| format!("{what} {h} is not a live path (og.path.create or og.path.find)"))
}

fn sample_path(p: &og_timeline::Path, t: f64) -> (DVec3, DVec3) {
    let s = p.sample(t as f32);
    (s.position.as_dvec3(), s.tangent.as_dvec3())
}

fn fsize(v: f32, k: &str) -> Result<f64, String> {
    Ok(f64::from(size(v, k)?))
}

fn fnum(v: f64, k: &str) -> Result<f64, String> {
    if !v.is_finite() {
        return Err(format!("{k} must be finite, got {v}"));
    }
    Ok(v)
}

pub fn og_render_shape_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::ShapeGeometryDesc| {
        let shapes = shapes_of(&d.shapes, &[[0.0, 0.5], [-0.5, -0.5], [0.5, -0.5]])?;
        Ok(og_geom::shape_geometry(&shapes))
    })
}

pub fn og_render_extrude_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::ExtrudeGeometryDesc| {
        let shapes = shapes_of(
            &d.shapes,
            &[[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]],
        )?;
        let thickness = fsize(d.bevel_thickness, "bevelThickness")?;
        let o = og_geom::Extrude {
            depth: fsize(d.depth, "depth")?,
            steps: segments(d.steps, 1),
            bevel_enabled: d.bevel_enabled,
            bevel_thickness: thickness,
            bevel_size: fnum(d.bevel_size.map_or(thickness - 0.1, f64::from), "bevelSize")?,
            bevel_offset: fnum(f64::from(d.bevel_offset), "bevelOffset")?,
            bevel_segments: segments(d.bevel_segments, 1),
        };
        let path = path_sampler(world, d.extrude_path, "desc.extrudePath")?;
        let sample = |t: f64| sample_path(path.as_deref().expect("a path"), t);
        let along = path.as_ref().map(|p| og_geom::extrude::ExtrudePath {
            sample: &sample,
            closed: p.desc().closed,
        });
        Ok(og_geom::extrude(&shapes, &o, along.as_ref()))
    })
}

pub fn og_render_lathe_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    let given = parse(desc.as_deref()).is_ok_and(|v| !v["points"].is_null());
    generated(world, desc, |d: &wire::LatheGeometryDesc| {
        let mut l = og_geom::Lathe::default();
        // Unset or null, three.js's default profile (the IDL's doc); an
        // empty list is refused below.
        if given {
            l.points = d.points.iter().map(|p| p.as_dvec2()).collect();
            if l.points.iter().any(|p| !p.is_finite()) {
                return Err("points holds a point that is not finite".into());
            }
        }
        if l.points.len() < 2 {
            return Err(format!(
                "a lathe needs at least two points, got {}",
                l.points.len()
            ));
        }
        l.segments = segments(d.segments, 1);
        l.phi_start = fnum(f64::from(d.phi_start), "phiStart")?;
        l.phi_length = fnum(f64::from(d.phi_length), "phiLength")?;
        Ok(og_geom::lathe(&l))
    })
}

pub fn og_render_polyhedron_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::PolyhedronGeometryDesc| {
        if d.vertices.iter().any(|v| !v.is_finite()) {
            return Err("vertices holds something that is not a finite number".into());
        }
        if d.indices.is_empty() {
            return Err("a polyhedron needs at least one face (indices)".into());
        }
        og_geom::polyhedron(&og_geom::Polyhedron {
            vertices: d.vertices.iter().map(|&v| f64::from(v)).collect(),
            indices: d.indices.clone(),
            radius: fsize(d.radius, "radius")?,
            detail: d.detail.min(64),
        })
    })
}

fn solid(
    world: u32,
    desc: Option<String>,
    make: fn(f64, u32) -> og_geom::Mesh,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::SolidGeometryDesc| {
        Ok(make(fsize(d.radius, "radius")?, d.detail.min(64)))
    })
}

pub fn og_render_icosahedron_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    solid(world, desc, og_geom::icosahedron)
}

pub fn og_render_dodecahedron_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    solid(world, desc, og_geom::dodecahedron)
}

pub fn og_render_octahedron_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    solid(world, desc, og_geom::octahedron)
}

pub fn og_render_tetrahedron_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    solid(world, desc, og_geom::tetrahedron)
}

fn keys(list: &[wire::SweepKey]) -> Vec<og_geom::Key> {
    list.iter()
        .map(|x| og_geom::Key {
            t: f64::from(x.t),
            value: f64::from(x.value),
        })
        .collect()
}

pub fn og_render_sweep_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::SweepGeometryDesc| {
        use wire::OneOf::{A, B};
        let path = path_sampler(world, d.path, "desc.path")?
            .ok_or("a sweep needs a path (og.path.create or og.path.find)")?;
        // Its points, or a ShapePoints whose holes are not swept.
        let section = match &d.section {
            None => Vec::new(),
            Some(A(_)) => return Err(unflattened("desc.section")),
            Some(B(A(points))) => dvec2s(points, "desc.section")?,
            Some(B(B(s))) => dvec2s(&s.shape, "desc.section.shape")?,
        };
        let s = og_geom::Sweep {
            section,
            steps: segments(d.steps, 1),
            scale: keys(&d.scale),
            twist: keys(&d.twist),
            caps: d.caps,
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
    v: &wire::GeometryPart,
    what: &str,
) -> Result<(og_geom::Mesh, og_geom::Placement, og_geom::Mirror), String> {
    let mesh = geom_of(world, v.geometry as f64, &format!("{what}.geometry"))?;
    let (p, scale) = (v.position, v.scale);
    if !p.is_finite() || !scale.is_finite() {
        return Err(format!("{what}'s position and scale must be finite"));
    }
    let q = unit(v.quaternion);
    let mirror = v.mirror;
    Ok((
        mesh,
        og_geom::Placement {
            position: p.as_dvec3(),
            quaternion: q.as_dquat(),
            scale: scale.as_dvec3(),
        },
        og_geom::Mirror {
            x: mirror == wire::MirrorAxis::X,
            y: mirror == wire::MirrorAxis::Y,
            z: mirror == wire::MirrorAxis::Z,
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

pub fn og_render_csg_geometry(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::CsgGeometryDesc| {
        let op = match d.op {
            wire::CsgOp::Union => og_geom::CsgOp::Union,
            wire::CsgOp::Subtract => og_geom::CsgOp::Subtract,
            wire::CsgOp::Intersect => og_geom::CsgOp::Intersect,
        };
        let a = placed(&part_of(world, &d.a, "desc.a")?, "desc.a")?;
        let b = placed(&part_of(world, &d.b, "desc.b")?, "desc.b")?;
        let none = og_geom::Placement::default();
        og_geom::csg(&a, none, &b, none, op)
    })
}

pub fn og_render_merge_geometries(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    generated(world, desc, |d: &wire::MergeGeometriesDesc| {
        let list = &d.parts;
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
