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

pub(super) fn size(d: &Value, k: &str, def: f32) -> Result<f32, String> {
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
