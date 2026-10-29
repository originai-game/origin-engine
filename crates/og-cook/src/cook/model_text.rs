//! `type model`: a model written as text, from og-geom's generators and
//! og-render's primitives (the runtime's own geometry descs, by the name
//! of their `og.render` call and with their fields' names and defaults),
//! placed as `og.render.mergeGeometries` places parts, a material per
//! part, and named pivots as nodes. The cook turns it into a glTF binary
//! here, so the rest of the cook (meshopt, the LOD chain with its measured
//! errors, the raycast BVH, the node graph) sees a `.glb`, as it does a
//! `.vox`: the blob is the model blob a glTF cooks to, `og.load.gltf`
//! loads it and a scene's `model` line names it.
//!
//! ```text
//! type model
//! version 1
//! ---
//! material "brass" {
//!   color 0.78 0.56 0.2         # linear, as glTF's baseColorFactor
//!   metalness 1
//!   roughness 0.35
//! }
//! pivot "axle" {                 # a node: animation and code find it by name
//!   position 0 0.6 0
//! }
//! part "gear" {
//!   pivot "axle"                 # hangs from the pivot (default: the model's root)
//!   material "brass"
//!   extrudeGeometry {
//!     shape {
//!       absarc 0 0 1 0 6.283185307179586 false
//!       hole {
//!         absarc 0 0 0.3 0 6.283185307179586 true
//!       }
//!     }
//!     depth 0.2
//!     bevelThickness 0.05
//!     bevelSize 0.04
//!   }
//! }
//! part "bolt" {                  # one mesh, drawn at every placement (instanced)
//!   material "brass"
//!   cylinderGeometry {
//!     radiusTop 0.05
//!     radiusBottom 0.05
//!     height 0.3
//!   }
//!   at {
//!     position 0.6 0 0
//!   }
//!   at {
//!     position -0.6 0 0
//!   }
//! }
//! ```
//!
//! A part's geometry is one block named by its `og.render` call:
//! `boxGeometry`, `planeGeometry`, `sphereGeometry`, `cylinderGeometry`,
//! `coneGeometry`, `capsuleGeometry`, `torusGeometry`, `shapeGeometry`,
//! `extrudeGeometry`, `latheGeometry`, `polyhedronGeometry`, the four
//! solids, `sweepGeometry`, `csgGeometry` (operands `a` and `b`, each a
//! geometry block placed) and `mergeGeometries` (`part` blocks, each a
//! geometry block placed). Shapes are og.Shape's calls one a line (`moveTo`,
//! `lineTo`, `quadraticCurveTo`, `bezierCurveTo`, `splineThru`, `arc`,
//! `absarc`, `ellipse`, `absellipse`, `closePath`, `points` for
//! setFromPoints) with `hole { }` blocks, flattened as og.Shape flattens
//! them ([`og_geom::path2`]); paths (`extrudePath`, a sweep's `path`) are
//! og.path's `kind`, `closed`, `tension` and `point` lines.
//!
//! A placement (`position`, `quaternion`, `scale`, `mirror`) is the part's
//! node transform, so a part placed several times (`at` blocks) is one
//! mesh under several nodes, which the renderer draws instanced. A mirrored
//! placement (`mirror`, or a negative scale) is baked into its own mesh
//! with its triangles turned, as mergeGeometries bakes one.

use super::vox::Bin;
use glam::{DQuat, DVec2, DVec3};
use og_core::ogt::{Block, Document, Entry};
use og_geom::{Mesh, Mirror, Path2, Placement, Shape, ShapePoints};
use std::collections::BTreeMap;

/// The `.ogt` type of a model written as text.
pub const MODEL_TYPE: &str = "model";

/// The geometry blocks a part may hold, by their `og.render` call.
pub const GEOMETRIES: [&str; 17] = [
    "boxGeometry",
    "planeGeometry",
    "sphereGeometry",
    "cylinderGeometry",
    "coneGeometry",
    "capsuleGeometry",
    "torusGeometry",
    "shapeGeometry",
    "extrudeGeometry",
    "latheGeometry",
    "polyhedronGeometry",
    "icosahedronGeometry",
    "dodecahedronGeometry",
    "octahedronGeometry",
    "tetrahedronGeometry",
    "sweepGeometry",
    "csgGeometry",
];

const MERGE: &str = "mergeGeometries";

fn at(e: &Entry, msg: impl std::fmt::Display) -> String {
    format!("line {}: {}: {msg}", e.line, e.key)
}

/// The numbers of an entry, every one finite.
fn numbers(e: &Entry) -> Result<Vec<f64>, String> {
    e.values
        .iter()
        .map(|v| {
            v.as_number()
                .map(|n| n.as_f64())
                .filter(|x| x.is_finite())
                .ok_or_else(|| at(e, "takes numbers"))
        })
        .collect()
}

fn exactly(e: &Entry, n: usize) -> Result<Vec<f64>, String> {
    let v = numbers(e)?;
    if v.len() != n {
        return Err(at(
            e,
            format!("takes {n} number{}", if n == 1 { "" } else { "s" }),
        ));
    }
    Ok(v)
}

/// The fields of one desc block: each key once, every key known.
struct Fields<'a> {
    block: &'a Block,
    what: &'a str,
}

impl<'a> Fields<'a> {
    fn new(block: &'a Block, what: &'a str, keys: &[&str]) -> Result<Fields<'a>, String> {
        let mut seen = std::collections::BTreeSet::new();
        for e in &block.entries {
            if !keys.contains(&e.key.as_str()) {
                return Err(format!(
                    "line {}: {what} has no field '{}' (its fields: {})",
                    e.line,
                    e.key,
                    keys.join(", ")
                ));
            }
            let repeats = matches!(
                (what, e.key.as_str()),
                ("extrudeGeometry" | "shapeGeometry", "shape")
                    | ("mergeGeometries", "part")
                    | ("sweepGeometry", "scale" | "twist")
                    | ("part", "at")
            );
            if !repeats && !seen.insert(e.key.as_str()) {
                return Err(format!("line {}: {what}.{} is given twice", e.line, e.key));
            }
        }
        Ok(Fields { block, what })
    }

    fn get(&self, k: &str) -> Option<&'a Entry> {
        self.block.get(k)
    }

    fn num(&self, k: &str, default: f64) -> Result<f64, String> {
        match self.get(k) {
            Some(e) => Ok(exactly(e, 1)?[0]),
            None => Ok(default),
        }
    }

    /// A size: finite and at least 0.
    fn size(&self, k: &str, default: f64) -> Result<f64, String> {
        let v = self.num(k, default)?;
        if v < 0.0 {
            return Err(format!("{}.{k} must be at least 0, got {v}", self.what));
        }
        Ok(v)
    }

    /// A segment count as [`Fields::uint`] reads it, scaled by `detail`
    /// (1: as written; under it, the generator is built coarser for the art
    /// contract's face budget, [`to_glb_within`]), never below `floor` (the
    /// count that still keeps the form) nor above what the text says.
    fn segments(
        &self,
        k: &str,
        (default, min, max): (u32, u32, u32),
        floor: u32,
        detail: f64,
    ) -> Result<u32, String> {
        let n = self.uint(k, default, min, max)?;
        Ok(scaled(n, floor, detail))
    }

    fn uint(&self, k: &str, default: u32, min: u32, max: u32) -> Result<u32, String> {
        let v = self.num(k, default as f64)?;
        if v.fract() != 0.0 || v < min as f64 || v > max as f64 {
            return Err(format!(
                "{}.{k} is a whole number from {min} to {max}, got {v}",
                self.what
            ));
        }
        Ok(v as u32)
    }

    fn flag(&self, k: &str, default: bool) -> Result<bool, String> {
        match self.get(k) {
            Some(e) => match e.values.as_slice() {
                [v] => v.as_bool().ok_or_else(|| at(e, "takes true or false")),
                _ => Err(at(e, "takes true or false")),
            },
            None => Ok(default),
        }
    }

    fn vec3(&self, k: &str, default: DVec3) -> Result<DVec3, String> {
        match self.get(k) {
            Some(e) => {
                let v = exactly(e, 3)?;
                Ok(DVec3::new(v[0], v[1], v[2]))
            }
            None => Ok(default),
        }
    }

    fn block(&self, k: &str) -> Result<Option<&'a Block>, String> {
        match self.get(k) {
            Some(e) => e
                .block
                .as_ref()
                .map(Some)
                .ok_or_else(|| at(e, "is a block: `{` on its line, `}` on its own")),
            None => Ok(None),
        }
    }
}

/// `n` segments at `detail`: `n × detail` rounded, held between the lower
/// of `floor` and `n`, and `n`.
fn scaled(n: u32, floor: u32, detail: f64) -> u32 {
    if detail >= 1.0 {
        return n;
    }
    ((n as f64 * detail).round() as u32).clamp(floor.min(n), n)
}

/// A polyhedron's `detail` (subdivisions, each edge cut in `detail + 1`)
/// at `detail` as [`scaled`] holds a segment count: down to the solid
/// itself.
fn subdivisions(d: u32, detail: f64) -> u32 {
    scaled(d + 1, 1, detail) - 1
}

// ── shapes and paths ────────────────────────────────────────────────────────

fn word_flag(e: &Entry, i: usize) -> Result<bool, String> {
    match e.values.get(i) {
        None => Ok(false),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| at(e, format!("value {} is true or false (clockwise)", i + 1))),
    }
}

/// The leading numbers of an entry, then optional flags (`clockwise`).
fn args(e: &Entry, n: usize, max: usize) -> Result<Vec<f64>, String> {
    if e.values.len() < n || e.values.len() > max {
        return Err(at(
            e,
            format!(
                "takes {n} numbers{}",
                if max > n {
                    " and its optional flags"
                } else {
                    ""
                }
            ),
        ));
    }
    e.values[..n]
        .iter()
        .map(|v| {
            v.as_number()
                .map(|x| x.as_f64())
                .filter(|x| x.is_finite())
                .ok_or_else(|| at(e, format!("its first {n} values are numbers")))
        })
        .collect()
}

/// og.Path2's calls, one a line.
fn path2(block: &Block, holes: Option<&mut Vec<Path2>>) -> Result<Path2, String> {
    let mut p = Path2::new();
    let mut holes = holes;
    for e in &block.entries {
        let v2 = |x: f64, y: f64| DVec2::new(x, y);
        match e.key.as_str() {
            "moveTo" => {
                let a = args(e, 2, 2)?;
                p.move_to(a[0], a[1]);
            }
            "lineTo" => {
                let a = args(e, 2, 2)?;
                p.line_to(a[0], a[1]);
            }
            "quadraticCurveTo" => {
                let a = args(e, 4, 4)?;
                p.quadratic_curve_to(a[0], a[1], a[2], a[3]);
            }
            "bezierCurveTo" => {
                let a = args(e, 6, 6)?;
                p.bezier_curve_to(v2(a[0], a[1]), v2(a[2], a[3]), a[4], a[5]);
            }
            "splineThru" | "points" => {
                let a = numbers(e)?;
                if a.is_empty() || a.len() % 2 != 0 {
                    return Err(at(e, "takes x y pairs"));
                }
                let pts: Vec<DVec2> = a.chunks(2).map(|c| v2(c[0], c[1])).collect();
                if e.key == "points" {
                    p.set_from_points(&pts);
                } else {
                    p.spline_thru(&pts);
                }
            }
            "arc" | "absarc" => {
                let a = args(e, 5, 6)?;
                let cw = word_flag(e, 5)?;
                if e.key == "arc" {
                    p.arc(a[0], a[1], a[2], a[3], a[4], cw);
                } else {
                    p.absarc(a[0], a[1], a[2], a[3], a[4], cw);
                }
            }
            "ellipse" | "absellipse" => {
                let a = args(e, 6, 8)?;
                let cw = word_flag(e, 6)?;
                let rotation = match e.values.get(7) {
                    None => 0.0,
                    Some(v) => v
                        .as_number()
                        .map(|n| n.as_f64())
                        .filter(|x| x.is_finite())
                        .ok_or_else(|| at(e, "its eighth value is the rotation, a number"))?,
                };
                let (c, r) = (v2(a[0], a[1]), v2(a[2], a[3]));
                if e.key == "ellipse" {
                    p.ellipse(c, r, a[4], a[5], cw, rotation);
                } else {
                    p.absellipse(c, r, a[4], a[5], cw, rotation);
                }
            }
            "closePath" => {
                if !e.values.is_empty() {
                    return Err(at(e, "takes no values"));
                }
                p.close_path();
            }
            "hole" => match (holes.as_deref_mut(), &e.block) {
                (Some(h), Some(b)) => h.push(path2(b, None)?),
                (None, _) => {
                    return Err(at(
                        e,
                        "a hole is cut in a shape, not in a hole or a section",
                    ));
                }
                (_, None) => return Err(at(e, "is a block of og.Path2 calls")),
            },
            k => {
                return Err(format!(
                    "line {}: '{k}' is not an og.Shape call (moveTo, lineTo, quadraticCurveTo, bezierCurveTo, splineThru, arc, absarc, ellipse, absellipse, closePath, points, hole)",
                    e.line
                ));
            }
        }
    }
    Ok(p)
}

fn shape_of(e: &Entry, divisions: u32) -> Result<ShapePoints, String> {
    let b = e
        .block
        .as_ref()
        .ok_or_else(|| at(e, "is a block of og.Shape calls"))?;
    let mut holes = Vec::new();
    let path = path2(b, Some(&mut holes))?;
    Ok(Shape { path, holes }.extract_points(divisions))
}

/// `shape` blocks, or the default when there is none.
fn shapes(f: &Fields, divisions: u32, default: &[[f64; 2]]) -> Result<Vec<ShapePoints>, String> {
    let list: Vec<ShapePoints> = f
        .block
        .get_all("shape")
        .map(|e| shape_of(e, divisions))
        .collect::<Result<_, _>>()?;
    if list.is_empty() {
        return Ok(vec![ShapePoints {
            points: default.iter().map(|p| DVec2::new(p[0], p[1])).collect(),
            holes: Vec::new(),
        }]);
    }
    Ok(list)
}

/// An og.path from its `kind`, `closed`, `tension` and `point` lines.
fn path(block: &Block) -> Result<og_timeline::Path, String> {
    let mut d = og_timeline::PathDesc::default();
    for e in &block.entries {
        match e.key.as_str() {
            "kind" => {
                let w = e.word(0).map_err(|x| at(e, x))?;
                d.kind = og_timeline::PathKind::from_name(w)
                    .ok_or_else(|| at(e, format!("'{w}' is not catmullRom, bezier or linear")))?;
            }
            "closed" => {
                d.closed = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| at(e, "takes true or false"))?
            }
            "tension" => d.tension = exactly(e, 1)?[0] as f32,
            "point" => {
                let v = exactly(e, 3)?;
                d.points
                    .push(glam::Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32));
            }
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a path field (kind, closed, tension, point)",
                    e.line
                ));
            }
        }
    }
    og_timeline::Path::new(d)
}

fn sample(p: &og_timeline::Path, t: f64) -> (DVec3, DVec3) {
    let s = p.sample(t as f32);
    (s.position.as_dvec3(), s.tangent.as_dvec3())
}

// ── geometries ──────────────────────────────────────────────────────────────

/// An og-render primitive as og-geom's mesh (no colours).
fn from_render(m: og_render::MeshData) -> Mesh {
    Mesh {
        positions: m.vertices.iter().map(|v| v.position).collect(),
        normals: m.vertices.iter().map(|v| v.normal).collect(),
        uvs: m.vertices.iter().map(|v| v.uv).collect(),
        colors: Vec::new(),
        indices: m.indices,
    }
}

const PHYSICS: [&str; 3] = ["physics", "dynamic", "uvRepeat"];

fn keys<'a>(own: &[&'a str]) -> Vec<&'a str> {
    let mut k = own.to_vec();
    k.extend(PHYSICS);
    k
}

/// The one geometry block of `block` (a part, an operand or a merged part).
fn geometry_entry<'a>(block: &'a Block, what: &str, line: usize) -> Result<&'a Entry, String> {
    let found: Vec<&Entry> = block
        .entries
        .iter()
        .filter(|e| GEOMETRIES.contains(&e.key.as_str()) || e.key == MERGE)
        .collect();
    match found.as_slice() {
        [e] => Ok(e),
        [] => Err(format!(
            "line {line}: {what} has no geometry: one block named by its og.render call ({}, {MERGE})",
            GEOMETRIES.join(", ")
        )),
        [_, b, ..] => Err(format!(
            "line {}: {what} has a second geometry; combine geometries with csgGeometry or mergeGeometries",
            b.line
        )),
    }
}

/// A geometry block, built.
fn geometry(e: &Entry, detail: f64) -> Result<Mesh, String> {
    let empty = Block::new();
    let b = e.block.as_ref().unwrap_or(&empty);
    let name = e.key.as_str();
    let tau = og_geom::trig::TAU;
    let f32s = |x: f64| x as f32;
    let mut mesh = match name {
        "boxGeometry" => {
            let f = Fields::new(b, name, &keys(&["width", "height", "depth"]))?;
            from_render(og_render::shapes::cuboid(glam::Vec3::new(
                f32s(f.size("width", 1.0)?),
                f32s(f.size("height", 1.0)?),
                f32s(f.size("depth", 1.0)?),
            )))
        }
        "planeGeometry" => {
            let f = Fields::new(b, name, &keys(&["width", "height"]))?;
            from_render(og_render::shapes::plane_xy(
                f32s(f.size("width", 1.0)?),
                f32s(f.size("height", 1.0)?),
            ))
        }
        "sphereGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&["radius", "widthSegments", "heightSegments"]),
            )?;
            let r = f32s(f.size("radius", 1.0)?);
            let mut m = og_render::shapes::sphere(
                f.segments("widthSegments", (32, 3, 512), 5, detail)?,
                f.segments("heightSegments", (16, 2, 512), 3, detail)?,
            );
            for v in &mut m.vertices {
                for c in &mut v.position {
                    *c *= r * 2.0;
                }
            }
            from_render(m)
        }
        "cylinderGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&[
                    "radiusTop",
                    "radiusBottom",
                    "height",
                    "radialSegments",
                    "heightSegments",
                    "openEnded",
                    "thetaStart",
                    "thetaLength",
                ]),
            )?;
            from_render(og_render::geometry::primitives::cylinder(
                &og_render::geometry::primitives::Cylinder {
                    radius_top: f32s(f.size("radiusTop", 1.0)?),
                    radius_bottom: f32s(f.size("radiusBottom", 1.0)?),
                    height: f32s(f.size("height", 1.0)?),
                    radial_segments: f.segments("radialSegments", (32, 3, 1 << 16), 5, detail)?,
                    height_segments: f.segments("heightSegments", (1, 1, 1 << 16), 1, detail)?,
                    open_ended: f.flag("openEnded", false)?,
                    theta_start: f32s(f.num("thetaStart", 0.0)?),
                    theta_length: f32s(f.num("thetaLength", tau)?),
                },
            ))
        }
        "coneGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&[
                    "radius",
                    "height",
                    "radialSegments",
                    "heightSegments",
                    "openEnded",
                ]),
            )?;
            from_render(og_render::geometry::primitives::cone(
                f32s(f.size("radius", 1.0)?),
                f32s(f.size("height", 1.0)?),
                f.segments("radialSegments", (32, 3, 1 << 16), 5, detail)?,
                f.segments("heightSegments", (1, 1, 1 << 16), 1, detail)?,
                f.flag("openEnded", false)?,
            ))
        }
        "capsuleGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&[
                    "radius",
                    "height",
                    "capSegments",
                    "radialSegments",
                    "heightSegments",
                ]),
            )?;
            from_render(og_render::geometry::primitives::capsule(
                f32s(f.size("radius", 1.0)?),
                f32s(f.size("height", 1.0)?),
                f.segments("capSegments", (4, 1, 1 << 16), 1, detail)?,
                f.segments("radialSegments", (8, 3, 1 << 16), 5, detail)?,
                f.segments("heightSegments", (1, 1, 1 << 16), 1, detail)?,
            ))
        }
        "torusGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&["radius", "tube", "radialSegments", "tubularSegments", "arc"]),
            )?;
            from_render(og_render::geometry::primitives::torus(
                f32s(f.size("radius", 1.0)?),
                f32s(f.size("tube", 0.4)?),
                f.segments("radialSegments", (12, 2, 1 << 16), 3, detail)?,
                f.segments("tubularSegments", (48, 3, 1 << 16), 6, detail)?,
                f32s(f.num("arc", tau)?),
            ))
        }
        "shapeGeometry" => {
            let f = Fields::new(b, name, &keys(&["shape", "curveSegments"]))?;
            let d = f.segments("curveSegments", (12, 1, 1 << 12), 2, detail)?;
            og_geom::shape_geometry(&shapes(&f, d, &[[0.0, 0.5], [-0.5, -0.5], [0.5, -0.5]])?)
        }
        "extrudeGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&[
                    "shape",
                    "depth",
                    "steps",
                    "bevelEnabled",
                    "bevelThickness",
                    "bevelSize",
                    "bevelOffset",
                    "bevelSegments",
                    "curveSegments",
                    "extrudePath",
                ]),
            )?;
            let d = f.segments("curveSegments", (12, 1, 1 << 12), 2, detail)?;
            let s = shapes(&f, d, &[[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]])?;
            let thickness = f.size("bevelThickness", 0.2)?;
            let o = og_geom::Extrude {
                depth: f.size("depth", 1.0)?,
                steps: f.uint("steps", 1, 1, 1 << 16)?,
                bevel_enabled: f.flag("bevelEnabled", true)?,
                bevel_thickness: thickness,
                // three.js's default, as the runtime's: thickness - 0.1.
                bevel_size: f.num("bevelSize", thickness - 0.1)?,
                bevel_offset: f.num("bevelOffset", 0.0)?,
                bevel_segments: f.segments("bevelSegments", (3, 1, 1 << 12), 1, detail)?,
            };
            match f.block("extrudePath")? {
                Some(pb) => {
                    let p = path(pb).map_err(|x| format!("{name}.extrudePath: {x}"))?;
                    let s_at = |t: f64| sample(&p, t);
                    let along = og_geom::extrude::ExtrudePath {
                        sample: &s_at,
                        closed: p.desc().closed,
                    };
                    og_geom::extrude(&s, &o, Some(&along))
                }
                None => og_geom::extrude(&s, &o, None),
            }
        }
        "latheGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&["points", "segments", "phiStart", "phiLength"]),
            )?;
            let mut l = og_geom::Lathe::default();
            if let Some(e) = f.get("points") {
                let a = numbers(e)?;
                if a.len() < 4 || a.len() % 2 != 0 {
                    return Err(at(e, "takes x y pairs, at least two"));
                }
                l.points = a.chunks(2).map(|c| DVec2::new(c[0], c[1])).collect();
            }
            l.segments = f.segments("segments", (12, 1, 1 << 16), 5, detail)?;
            l.phi_start = f.num("phiStart", 0.0)?;
            l.phi_length = f.num("phiLength", tau)?;
            og_geom::lathe(&l)
        }
        "polyhedronGeometry" => {
            let f = Fields::new(b, name, &keys(&["vertices", "indices", "radius", "detail"]))?;
            let vertices = f
                .get("vertices")
                .map(numbers)
                .transpose()?
                .unwrap_or_default();
            let indices: Vec<u32> = f
                .get("indices")
                .map(|e| {
                    numbers(e)?
                        .into_iter()
                        .map(|x| {
                            (x >= 0.0 && x.fract() == 0.0 && x <= u32::MAX as f64)
                                .then_some(x as u32)
                                .ok_or_else(|| at(e, "takes whole numbers"))
                        })
                        .collect::<Result<Vec<u32>, String>>()
                })
                .transpose()?
                .unwrap_or_default();
            if indices.is_empty() {
                return Err(format!("{name} needs at least one face (indices)"));
            }
            og_geom::polyhedron(&og_geom::Polyhedron {
                vertices,
                indices,
                radius: f.size("radius", 1.0)?,
                detail: subdivisions(f.uint("detail", 0, 0, 64)?, detail),
            })?
        }
        "icosahedronGeometry"
        | "dodecahedronGeometry"
        | "octahedronGeometry"
        | "tetrahedronGeometry" => {
            let f = Fields::new(b, name, &keys(&["radius", "detail"]))?;
            let make = match name {
                "icosahedronGeometry" => og_geom::icosahedron,
                "dodecahedronGeometry" => og_geom::dodecahedron,
                "octahedronGeometry" => og_geom::octahedron,
                _ => og_geom::tetrahedron,
            };
            make(
                f.size("radius", 1.0)?,
                subdivisions(f.uint("detail", 0, 0, 64)?, detail),
            )
        }
        "sweepGeometry" => {
            let f = Fields::new(
                b,
                name,
                &keys(&[
                    "section",
                    "path",
                    "steps",
                    "scale",
                    "twist",
                    "caps",
                    "curveSegments",
                ]),
            )?;
            let d = f.segments("curveSegments", (12, 1, 1 << 12), 2, detail)?;
            let section = match f.block("section")? {
                Some(sb) => path2(sb, None)?.get_points(d),
                None => {
                    return Err(format!(
                        "{name} needs a section (a block of og.Path2 calls)"
                    ));
                }
            };
            let pb = f
                .block("path")?
                .ok_or_else(|| format!("{name} needs a path (kind, closed, point lines)"))?;
            let p = path(pb).map_err(|x| format!("{name}.path: {x}"))?;
            let keys_of = |k: &str| -> Result<Vec<og_geom::Key>, String> {
                b.get_all(k)
                    .map(|e| {
                        let v = exactly(e, 2)?;
                        Ok(og_geom::Key {
                            t: v[0],
                            value: v[1],
                        })
                    })
                    .collect()
            };
            let s = og_geom::Sweep {
                section,
                steps: f.segments("steps", (64, 1, 1 << 16), 6, detail)?,
                scale: keys_of("scale")?,
                twist: keys_of("twist")?,
                caps: f.flag("caps", true)?,
            };
            let s_at = |t: f64| sample(&p, t);
            og_geom::sweep(
                &s,
                &og_geom::extrude::ExtrudePath {
                    sample: &s_at,
                    closed: p.desc().closed,
                },
            )?
        }
        "csgGeometry" => {
            let f = Fields::new(b, name, &keys(&["op", "a", "b"]))?;
            let op = match f.get("op") {
                None => og_geom::CsgOp::Union,
                Some(e) => match e.word(0).map_err(|x| at(e, x))? {
                    "union" => og_geom::CsgOp::Union,
                    "subtract" => og_geom::CsgOp::Subtract,
                    "intersect" => og_geom::CsgOp::Intersect,
                    w => return Err(at(e, format!("'{w}' is not union, subtract or intersect"))),
                },
            };
            let operand = |k: &str| -> Result<Mesh, String> {
                let e = f
                    .get(k)
                    .ok_or_else(|| format!("{name} needs operands a and b"))?;
                let ob = e
                    .block
                    .as_ref()
                    .ok_or_else(|| at(e, "is a placed geometry block"))?;
                placed_geometry(ob, &format!("{name}.{k}"), e.line, detail)
            };
            let none = Placement::default();
            og_geom::csg(&operand("a")?, none, &operand("b")?, none, op)?
        }
        MERGE => {
            let f = Fields::new(b, name, &keys(&["part"]))?;
            let parts: Vec<Mesh> = b
                .get_all("part")
                .enumerate()
                .map(|(i, e)| {
                    let pb = e
                        .block
                        .as_ref()
                        .ok_or_else(|| at(e, "is a placed geometry block"))?;
                    placed_geometry(pb, &format!("{name}.part[{i}]"), e.line, detail)
                })
                .collect::<Result<_, _>>()?;
            let _ = f;
            if parts.is_empty() {
                return Err(format!("{name} needs at least one part"));
            }
            let refs: Vec<og_geom::Part> = parts
                .iter()
                .map(|m| og_geom::Part {
                    mesh: m,
                    placement: Placement::default(),
                    mirror: Mirror::default(),
                })
                .collect();
            og_geom::merge(&refs)?
        }
        _ => unreachable!("geometry_entry names only geometries"),
    };
    // uvRepeat, as og.render's geometry descs take it (og_render::shapes::repeat_uv).
    if let Some(e) = b.get("uvRepeat") {
        let r = exactly(e, 2)?;
        for uv in &mut mesh.uvs {
            uv[0] *= r[0] as f32;
            uv[1] *= r[1] as f32;
        }
    }
    if mesh.indices.is_empty() {
        return Err(format!("line {}: {name} holds no triangle", e.line));
    }
    if mesh.positions.iter().flatten().any(|x| !x.is_finite()) {
        return Err(format!(
            "line {}: {name} holds a position that is not finite",
            e.line
        ));
    }
    Ok(mesh)
}

/// A placement block's `position`, `quaternion`, `scale` and `mirror`.
fn placement(f: &Fields) -> Result<(Placement, Mirror), String> {
    let q = match f.get("quaternion") {
        Some(e) => {
            let v = exactly(e, 4)?;
            let q = DQuat::from_xyzw(v[0], v[1], v[2], v[3]);
            if q.length_squared() < 1e-12 {
                return Err(at(e, "is not a rotation (zero length)"));
            }
            q.normalize()
        }
        None => DQuat::IDENTITY,
    };
    let mirror = match f.get("mirror") {
        None => Mirror::default(),
        Some(e) => match e.word(0).map_err(|x| at(e, x))? {
            "none" => Mirror::default(),
            "x" => Mirror {
                x: true,
                ..Default::default()
            },
            "y" => Mirror {
                y: true,
                ..Default::default()
            },
            "z" => Mirror {
                z: true,
                ..Default::default()
            },
            w => return Err(at(e, format!("'{w}' is not none, x, y or z"))),
        },
    };
    Ok((
        Placement {
            position: f.vec3("position", DVec3::ZERO)?,
            quaternion: q,
            scale: f.vec3("scale", DVec3::ONE)?,
        },
        mirror,
    ))
}

const PLACEMENT: [&str; 4] = ["position", "quaternion", "scale", "mirror"];

/// A geometry block with its placement, baked into one mesh (a CSG
/// operand, a merged part).
fn placed_geometry(block: &Block, what: &str, line: usize, detail: f64) -> Result<Mesh, String> {
    let g = geometry_entry(block, what, line)?;
    let mut allowed: Vec<&str> = PLACEMENT.to_vec();
    allowed.push(g.key.as_str());
    let f = Fields::new(block, what, &allowed)?;
    let (p, m) = placement(&f)?;
    let mesh = geometry(g, detail).map_err(|e| format!("{what}: {e}"))?;
    og_geom::merge(&[og_geom::Part {
        mesh: &mesh,
        placement: p,
        mirror: m,
    }])
    .map_err(|e| format!("{what}: {}", e.replacen("part 0: ", "", 1)))
}

// ── the model ───────────────────────────────────────────────────────────────

/// A material as glTF writes it.
fn material(name: &str, b: &Block) -> Result<serde_json::Value, String> {
    let f = Fields::new(
        b,
        "material",
        &[
            "color",
            "opacity",
            "metalness",
            "roughness",
            "emissive",
            "emissiveIntensity",
            "doubleSided",
            "alphaTest",
        ],
    )?;
    let c = f.vec3("color", DVec3::ONE)?;
    let opacity = f.num("opacity", 1.0)?;
    let unit = |k: &str, v: f64| -> Result<f64, String> {
        if (0.0..=1.0).contains(&v) {
            Ok(v)
        } else {
            Err(format!("material '{name}': {k} is 0 to 1, got {v}"))
        }
    };
    let e = f.vec3("emissive", DVec3::ZERO)? * f.size("emissiveIntensity", 1.0)?;
    let mut m = serde_json::json!({
        "name": name,
        "pbrMetallicRoughness": {
            "baseColorFactor": [c.x, c.y, c.z, unit("opacity", opacity)?],
            "metallicFactor": unit("metalness", f.num("metalness", 0.0)?)?,
            "roughnessFactor": unit("roughness", f.num("roughness", 1.0)?)?,
        },
        "doubleSided": f.flag("doubleSided", false)?,
    });
    if e != DVec3::ZERO {
        let peak = e.max_element();
        m["emissiveFactor"] = serde_json::json!(if peak > 1.0 {
            (e / peak).to_array()
        } else {
            e.to_array()
        });
        if peak > 1.0 {
            m["extensions"] = serde_json::json!({ "KHR_materials_emissive_strength": { "emissiveStrength": peak } });
        }
    }
    if let Some(cut) = f.get("alphaTest") {
        m["alphaMode"] = "MASK".into();
        m["alphaCutoff"] = unit("alphaTest", exactly(cut, 1)?[0])?.into();
    } else if opacity < 1.0 {
        m["alphaMode"] = "BLEND".into();
    }
    Ok(m)
}

fn name_of(e: &Entry, what: &str) -> Result<String, String> {
    match e.values.as_slice() {
        [v] => v.as_str().map(str::to_string).ok_or_else(|| {
            at(
                e,
                format!("names the {what} in quotes: `{} \"name\" {{`", e.key),
            )
        }),
        _ => Err(at(
            e,
            format!("names the {what} in quotes: `{} \"name\" {{`", e.key),
        )),
    }
}

fn node(name: &str, p: &Placement) -> serde_json::Value {
    let mut n = serde_json::json!({ "name": name });
    if p.position != DVec3::ZERO {
        n["translation"] = serde_json::json!(p.position.to_array());
    }
    if p.quaternion != DQuat::IDENTITY {
        n["rotation"] = serde_json::json!(p.quaternion.to_array());
    }
    if p.scale != DVec3::ONE {
        n["scale"] = serde_json::json!(p.scale.to_array());
    }
    n
}

/// What a `type model` text builds, before it is written as a glb.
pub(super) struct Built {
    pub json: serde_json::Value,
    pub bin: Bin,
    /// Its triangles as drawn: each part's times its placements.
    pub drawn: usize,
}

/// A `type model` text as a glTF binary.
pub fn to_glb(text: &str) -> Result<Vec<u8>, String> {
    let b = build(text)?;
    b.bin.finish(b.json)
}

/// A `type model` text as a glTF binary within a face `budget` (the art
/// contract's `ArtShape.propFaces` or `characterFaces`; 0 is none): when
/// the text as written draws more triangles, its generators are built at
/// fewer segments (every count scaled by one factor, each held at the
/// count that still keeps its form: a sphere 5 × 3, a cylinder, cone or
/// lathe 5 round), the most the budget holds. A recipe is built at the
/// style's count, not built dense and then decimated: a 24 × 12 sphere
/// cut to a 200-triangle budget by edge collapses keeps its poles' fans
/// and loses its outline first, while the same sphere built 8 × 5 is the
/// faceted form itself. Also the factor (1 as written); what is still
/// past the budget at the floors is left to the decimation.
pub fn to_glb_within(text: &str, budget: u32) -> Result<(Vec<u8>, f64), String> {
    let b = build(text)?;
    if budget == 0 || b.drawn <= budget as usize {
        return Ok((b.bin.finish(b.json)?, 1.0));
    }
    // The drawn count only falls with the factor: the largest factor on a
    // 1/256 grid whose model fits, else the floors.
    let (mut lo, mut hi) = (0u32, 256u32);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if build_at(text, mid as f64 / 256.0)?.drawn <= budget as usize {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let detail = lo as f64 / 256.0;
    let b = build_at(text, detail)?;
    Ok((b.bin.finish(b.json)?, detail))
}

pub(super) fn build(text: &str) -> Result<Built, String> {
    build_at(text, 1.0)
}

/// [`build`] with every segment count at `detail` ([`Fields::segments`]).
fn build_at(text: &str, detail: f64) -> Result<Built, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != MODEL_TYPE {
        return Err(format!("expected `type model`, not `type {}`", doc.ty));
    }
    let mut materials: Vec<serde_json::Value> = Vec::new();
    let mut material_index: BTreeMap<String, usize> = BTreeMap::new();
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    let mut children: Vec<Vec<usize>> = Vec::new();
    let mut pivots: BTreeMap<String, usize> = BTreeMap::new();
    let mut roots: Vec<usize> = Vec::new();
    let mut bin = Bin::default();
    let mut meshes: Vec<serde_json::Value> = Vec::new();
    // A mesh already written, by its bits and material: a part repeated
    // under another name draws the same mesh.
    let mut written: BTreeMap<(Vec<u32>, usize), usize> = BTreeMap::new();
    let mut drawn = 0;
    let add_node = |n: serde_json::Value,
                    parent: Option<usize>,
                    nodes: &mut Vec<serde_json::Value>,
                    children: &mut Vec<Vec<usize>>,
                    roots: &mut Vec<usize>| {
        nodes.push(n);
        children.push(Vec::new());
        let i = nodes.len() - 1;
        match parent {
            Some(p) => children[p].push(i),
            None => roots.push(i),
        }
        i
    };
    for e in &doc.body.entries {
        let block = e.block.as_ref();
        match e.key.as_str() {
            "material" => {
                let name = name_of(e, "material")?;
                let b = block.ok_or_else(|| at(e, "is a block"))?;
                if material_index.contains_key(&name) {
                    return Err(at(e, format!("'{name}' is defined twice")));
                }
                materials.push(material(&name, b).map_err(|x| format!("line {}: {x}", e.line))?);
                material_index.insert(name, materials.len() - 1);
            }
            "pivot" => {
                let name = name_of(e, "pivot")?;
                let empty = Block::new();
                let b = block.unwrap_or(&empty);
                let mut allowed = PLACEMENT[..3].to_vec();
                allowed.push("parent");
                let f = Fields::new(b, "pivot", &allowed)?;
                let parent = match f.get("parent") {
                    Some(pe) => {
                        let p = pe.string(0).map_err(|x| at(pe, x))?;
                        Some(*pivots.get(p).ok_or_else(|| {
                            at(pe, format!("'{p}' is not a pivot defined above this one"))
                        })?)
                    }
                    None => None,
                };
                if pivots.contains_key(&name) {
                    return Err(at(e, format!("'{name}' is defined twice")));
                }
                let (p, _) = placement(&f)?;
                if p.scale.min_element() <= 0.0 {
                    return Err(at(
                        e,
                        "a pivot's scale is positive (mirror a part, not a pivot)",
                    ));
                }
                let i = add_node(
                    node(&name, &p),
                    parent,
                    &mut nodes,
                    &mut children,
                    &mut roots,
                );
                pivots.insert(name, i);
            }
            "part" => {
                let name = name_of(e, "part")?;
                let b = block.ok_or_else(|| at(e, "is a block"))?;
                let what = format!("part '{name}'");
                let g = geometry_entry(b, &what, e.line)?;
                let mut allowed: Vec<&str> = PLACEMENT.to_vec();
                allowed.extend(["material", "pivot", "at", g.key.as_str()]);
                let f = Fields::new(b, "part", &allowed)?;
                let material = match f.get("material") {
                    Some(me) => {
                        let m = me.string(0).map_err(|x| at(me, x))?;
                        *material_index.get(m).ok_or_else(|| {
                            at(
                                me,
                                format!("'{m}' is not a material defined above this part"),
                            )
                        })?
                    }
                    None => {
                        // glTF's default material (og-cook adds it last).
                        usize::MAX
                    }
                };
                let pivot = match f.get("pivot") {
                    Some(pe) => {
                        let p = pe.string(0).map_err(|x| at(pe, x))?;
                        Some(*pivots.get(p).ok_or_else(|| {
                            at(pe, format!("'{p}' is not a pivot defined above this part"))
                        })?)
                    }
                    None => None,
                };
                let ats: Vec<&Entry> = b.get_all("at").collect();
                let own = PLACEMENT.iter().any(|k| f.get(k).is_some());
                if own && !ats.is_empty() {
                    return Err(at(
                        e,
                        "places itself by its own position or by `at` blocks, not both",
                    ));
                }
                let mut places: Vec<(Placement, Mirror)> = Vec::new();
                if ats.is_empty() {
                    places.push(placement(&f)?);
                }
                for a in &ats {
                    let ab = a
                        .block
                        .as_ref()
                        .ok_or_else(|| at(a, "is a placement block"))?;
                    places.push(placement(&Fields::new(ab, "at", &PLACEMENT)?)?);
                }
                let mesh = geometry(g, detail).map_err(|x| format!("{what}: {x}"))?;
                drawn += mesh.indices.len() / 3 * places.len();
                for (k, (p, m)) in places.iter().enumerate() {
                    // A mirror is baked (the node keeps a positive scale and
                    // its triangles their winding); the rest is the node's.
                    let mirrors = m.x || m.y || m.z || p.scale.min_element() < 0.0;
                    let (geo, local) = if mirrors {
                        let baked = og_geom::merge(&[og_geom::Part {
                            mesh: &mesh,
                            placement: *p,
                            mirror: *m,
                        }])
                        .map_err(|x| format!("{what}: {}", x.replacen("part 0: ", "", 1)))?;
                        (std::borrow::Cow::Owned(baked), Placement::default())
                    } else {
                        if p.scale.min_element() == 0.0 {
                            return Err(format!(
                                "{what}: its placement is singular (a scale component is 0)"
                            ));
                        }
                        (std::borrow::Cow::Borrowed(&mesh), *p)
                    };
                    let key = (geo.bits(), material);
                    let mi = match written.get(&key) {
                        Some(&i) => i,
                        None => {
                            meshes.push(write_mesh(&mut bin, &name, &geo, material));
                            written.insert(key, meshes.len() - 1);
                            meshes.len() - 1
                        }
                    };
                    let label = if places.len() == 1 {
                        name.clone()
                    } else {
                        format!("{name}[{k}]")
                    };
                    let mut n = node(&label, &local);
                    n["mesh"] = mi.into();
                    add_node(n, pivot, &mut nodes, &mut children, &mut roots);
                }
            }
            // `conform off`: the art contract leaves it as made;
            // `conform character`: it is held to ArtShape.characterFaces
            // (cook/conform).
            "conform"
                if e.block.is_none()
                    && matches!(
                        e.values.as_slice(),
                        [v] if matches!(v.as_word(), Some("off" | "character"))
                    ) => {}
            k => {
                return Err(format!(
                    "line {}: '{k}' is not part of a model (material, pivot, part, conform off, conform character)",
                    e.line
                ));
            }
        }
    }
    if meshes.is_empty() {
        return Err("a model needs at least one part".into());
    }
    // Parts without a material draw glTF's default, after the model's own.
    for m in &mut meshes {
        for p in m["primitives"].as_array_mut().into_iter().flatten() {
            if p["material"] == serde_json::json!(usize::MAX) {
                p.as_object_mut().expect("an object").remove("material");
            }
        }
    }
    for (i, c) in children.iter().enumerate() {
        if !c.is_empty() {
            nodes[i]["children"] = serde_json::json!(c);
        }
    }
    let mut json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook type model" },
        "scene": 0,
        "scenes": [{ "nodes": roots }],
        "nodes": nodes,
        "meshes": meshes,
    });
    if !materials.is_empty() {
        json["materials"] = materials.into();
    }
    if json.to_string().contains("KHR_materials_emissive_strength") {
        json["extensionsUsed"] = serde_json::json!(["KHR_materials_emissive_strength"]);
    }
    Ok(Built { json, bin, drawn })
}

/// One mesh's attributes in the binary chunk, as a glTF mesh.
fn write_mesh(bin: &mut Bin, name: &str, m: &Mesh, material: usize) -> serde_json::Value {
    let count = m.positions.len();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in &m.positions {
        for a in 0..3 {
            lo[a] = lo[a].min(p[a]);
            hi[a] = hi[a].max(p[a]);
        }
    }
    let pos = bin.add(
        bytemuck::cast_slice(&m.positions),
        Some(34962),
        serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi }),
    );
    let mut attributes = serde_json::json!({ "POSITION": pos });
    if m.normals.len() == count {
        attributes["NORMAL"] = bin
            .add(
                bytemuck::cast_slice(&m.normals),
                Some(34962),
                serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
            )
            .into();
    }
    if m.uvs.len() == count {
        attributes["TEXCOORD_0"] = bin
            .add(
                bytemuck::cast_slice(&m.uvs),
                Some(34962),
                serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC2" }),
            )
            .into();
    }
    if m.colors.len() == count {
        attributes["COLOR_0"] = bin
            .add(
                bytemuck::cast_slice(&m.colors),
                Some(34962),
                serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
            )
            .into();
    }
    let idx = bin.add(
        bytemuck::cast_slice(&m.indices),
        Some(34963),
        serde_json::json!({ "componentType": 5125, "count": m.indices.len(), "type": "SCALAR" }),
    );
    serde_json::json!({
        "name": name,
        "primitives": [{ "attributes": attributes, "indices": idx, "material": material }]
    })
}
