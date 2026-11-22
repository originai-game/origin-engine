//! `og model measure <program.json | mesh>`: measures as JSON
//! (`docs/design.md` §7), with no thresholds. A program gives every part
//! and the whole model at rest (`og_shape::model::program::measures_json`);
//! with `--posed`, at its pose, with how each joint holds and the
//! interference at the pose (`og_shape::model::program::posed_json`); a mesh
//! file gives every part placed, and the parts joined
//! (`og_shape::tools::MeshMeasures`, null for a skinned file, which cannot be
//! joined). With `--axes`, a mesh's report adds `axes`
//! (`og_shape::tools::AxisMeasures`) to each part and for the whole file.
//!
//! Every report but `--posed` also carries the facts a prop pipeline reads, at its top
//! level and in the GLB's frame ([`facts`]): `size` `[x, y, z]` in metres
//! (+Y up), `triangles`, `vertices`, `partCount` and `materials`. In a
//! project whose look sets an art contract (read from its cooked look, as
//! `og check` does), `look` adds how the model sits against it: each
//! material's colour, its nearest palette colour and their CIEDE2000, and
//! the triangles against `ArtShape.propFaces`. Facts only: nothing here
//! refuses on them.

use og_shape::tools::{ColorSpace, Measure, Scene};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use super::args::{Args, IMPORT_OPTIONS};
use super::report::{array, object, string};
use super::source::{Local, Mesh, Source, load_uvs};
use super::{Failure, failed};

/// What to measure besides the defaults. The zero value skips the exact
/// self-intersection count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeasureDesc {
    /// Meshes only: count self-intersecting triangle pairs (exact; costs a
    /// sweep over the triangles).
    pub intersections: bool,
    /// Meshes only: add `axes` to each part's and the whole's measures
    /// (`og_shape::tools::AxisMeasures`: extents, the longest and shortest
    /// axis, the area facing each way, where the mass sits), the numbers
    /// for choosing up and front.
    pub axes: bool,
}

/// The measures of a program or a mesh.
pub fn measure(source: &Source, d: &MeasureDesc) -> Result<String, Failure> {
    match source {
        Source::Program(m) => og_shape::model::program::measures_json(m).map_err(failed),
        Source::Mesh(mesh) => mesh_measures(mesh, d),
    }
}

/// A program's measures at its pose (`og_shape::model::program::posed_json`).
/// A mesh file has no joints: refused.
pub fn measure_posed(source: &Source) -> Result<String, Failure> {
    match source {
        Source::Program(m) => og_shape::model::program::posed_json(m).map_err(failed),
        Source::Mesh(_) => Err(Failure::Usage(
            "--posed measures a program at its pose; a mesh file has no joints".into(),
        )),
    }
}

fn mesh_measures(mesh: &Mesh, d: &MeasureDesc) -> Result<String, Failure> {
    let placed = mesh.placed()?;
    let parts: Vec<String> = placed
        .iter()
        .map(|(name, s)| {
            let m = og_shape::tools::measure(
                s,
                &Measure {
                    intersections: d.intersections,
                    ..Default::default()
                },
            );
            let mut fields = vec![("name", string(name)), ("measures", m.to_json())];
            if d.axes {
                fields.push((
                    "axes",
                    og_shape::tools::axis_measures(std::slice::from_ref(s)).to_json(),
                ));
            }
            object(&fields)
        })
        .collect();
    let whole = if mesh.skinned() {
        "null".to_string()
    } else {
        og_shape::tools::measure(
            &mesh.joined()?,
            &Measure {
                images: &mesh.imported.images,
                intersections: d.intersections,
                ..Default::default()
            },
        )
        .to_json()
    };
    let mut fields = vec![
        ("name", string(&mesh.name)),
        ("format", string(&mesh.format)),
        ("parts", array(&parts)),
        ("whole", whole),
    ];
    if d.axes {
        // Every part placed, taken together: a skinned file too.
        let solids: Vec<_> = placed.into_iter().map(|(_, s)| s).collect();
        fields.push(("axes", og_shape::tools::axis_measures(&solids).to_json()));
    }
    Ok(object(&fields))
}

/// The prop facts of a scene (see the module doc), `look` against
/// `contract` when there is one.
pub fn facts(scene: &Scene, contract: Option<&crate::style_check::Contract>) -> Value {
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    let (mut triangles, mut vertices) = (0usize, 0usize);
    for p in &scene.parts {
        triangles += p.solid.triangles.len();
        vertices += p.solid.positions.len();
        for q in &p.solid.positions {
            for (k, v) in [q.x, q.y, q.z].into_iter().enumerate() {
                lo[k] = lo[k].min(v);
                hi[k] = hi[k].max(v);
            }
        }
    }
    // Kernel millimetres, +Z up, to the GLB's metres, +Y up.
    let d = |k: usize| {
        if hi[k] >= lo[k] {
            (hi[k] - lo[k]) * 0.001
        } else {
            0.0
        }
    };
    let size = [d(0), d(2), d(1)];
    // Each material once, by name (a part's name when it has none), with
    // its sRGB colour: the base colour times its map's mean.
    let mut materials: Vec<(String, [u8; 3])> = Vec::new();
    for p in &scene.parts {
        let name = if p.material.name.is_empty() {
            p.name.clone()
        } else {
            p.material.name.clone()
        };
        if materials.iter().any(|(n, _)| *n == name) {
            continue;
        }
        materials.push((name, colour(&p.material)));
    }
    let mut out = json!({
        "size": size,
        "triangles": triangles,
        "vertices": vertices,
        "partCount": scene.parts.len(),
        "materials": materials.iter().map(|(n, c)| json!({ "material": n, "colour": hex(*c) })).collect::<Vec<_>>(),
    });
    if let Some(c) = contract {
        let palette: Vec<Value> = match &c.palette {
            Some((_, colours)) if !colours.is_empty() => materials
                .iter()
                .map(|(n, rgb)| {
                    let here = crate::commands::compare::lab(*rgb);
                    let (near, de) = colours
                        .iter()
                        .map(|p| (*p, crate::commands::compare::delta_e00(here, crate::commands::compare::lab(*p))))
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .expect("a palette with colours");
                    json!({ "material": n, "colour": hex(*rgb), "nearest": hex(near), "deltaE": de })
                })
                .collect(),
            _ => Vec::new(),
        };
        out["look"] = json!({
            "look": c.look,
            "palette": palette,
            "paletteColours": c.palette.as_ref().map_or(0, |p| p.1.len()),
            "propFaces": c.prop_faces,
            "facesRatio": (c.prop_faces > 0).then(|| triangles as f64 / f64::from(c.prop_faces)),
            "texelDensity": c.density,
            "maxTextureSize": c.max_size,
        });
    }
    out
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// A material's colour as drawn, sRGB: its base colour times the mean of
/// its base-colour map when the map decodes.
fn colour(m: &og_shape::tools::Material) -> [u8; 3] {
    let lin = |c: f64| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            og_core::math::double::pow((c + 0.055) / 1.055, 2.4)
        }
    };
    let srgb = |c: f64| {
        if c <= 0.003_130_8 {
            c * 12.92
        } else {
            1.055 * og_core::math::double::pow(c, 1.0 / 2.4) - 0.055
        }
    };
    let base = m.color.unwrap_or([1.0; 3]).map(|c| c.clamp(0.0, 1.0));
    let mut linear = match m.color_space {
        ColorSpace::Srgb => base.map(lin),
        ColorSpace::Linear => base,
    };
    if let Some(img) = m
        .map
        .as_ref()
        .and_then(|t| og_shape_formats::texture::decode(&t.bytes).ok())
    {
        // The map's mean in linear light, over its opaque texels.
        let (mut sum, mut n) = ([0f64; 3], 0f64);
        for px in img.rgba.as_chunks::<4>().0.iter().filter(|p| p[3] >= 128) {
            for k in 0..3 {
                sum[k] += lin(f64::from(px[k]) / 255.0);
            }
            n += 1.0;
        }
        if n > 0.0 {
            for k in 0..3 {
                linear[k] *= sum[k] / n;
            }
        }
    }
    linear.map(|c| (srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8)
}

/// The project the model is in: `--project`, else the nearest folder
/// above the model (or the current one) holding a project.ogt.
fn project_of(a: &Args, input: &super::Input) -> Result<Option<PathBuf>, Failure> {
    if let Some(p) = a.value(&["--project"])? {
        return Ok(Some(PathBuf::from(p)));
    }
    let start = match input {
        super::Input::Path(p) => std::path::absolute(p)
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf)),
        super::Input::None => None,
    }
    .or_else(|| std::env::current_dir().ok());
    Ok(start.and_then(|s| {
        s.ancestors()
            .find(|d| d.join("project.ogt").is_file())
            .map(Path::to_path_buf)
    }))
}

/// The command line.
pub fn run(a: &Args) -> Result<String, Failure> {
    let mut known = vec!["--intersections", "--posed", "--uvs", "--axes", "--project"];
    known.extend(IMPORT_OPTIONS);
    a.only(&known)?;
    let input = a.input()?;
    let source = load_uvs(&Local, &input, &a.import()?, a.switch("--uvs"))?;
    // At the pose: the kernel's report alone (its bounds are the pose's;
    // the prop facts are the model at rest).
    if a.switch("--posed") {
        return measure_posed(&source);
    }
    let report = measure(
        &source,
        &MeasureDesc {
            intersections: a.switch("--intersections"),
            axes: a.switch("--axes"),
        },
    )?;
    // The project's art contract, when it has one; a look that cannot be
    // read (an uncooked project) is said, not fatal.
    let (contract, look_error) = match project_of(a, &input)? {
        Some(root) => match crate::style_check::contract(&root) {
            Ok(c) => (c, None),
            Err(e) => (None, Some(e)),
        },
        None => (None, None),
    };
    let facts = super::preview::with_scene(&source, None, &[], |scene| {
        Ok(facts(scene, contract.as_ref()))
    })?;
    let mut v: Value = serde_json::from_str(&report).map_err(failed)?;
    if let (Value::Object(o), Value::Object(f)) = (&mut v, facts) {
        o.extend(f);
        if let Some(e) = look_error {
            o.insert("lookError".into(), Value::String(e));
        }
    }
    Ok(v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_shape::generators::solids::{BoxDesc, cuboid};
    use og_shape::tools::{Material, Part};

    /// A box 1 m wide (x), 2 m deep (kernel y) and 3 m tall (kernel z) is
    /// [1, 3, 2] in the GLB's frame; its dark red base colour is nearest the
    /// palette's red, not its blue; its 12 triangles are twice a budget of 6.
    #[test]
    fn facts_are_in_metres_in_the_glbs_frame_against_the_look() {
        let s = cuboid(&BoxDesc {
            width: 1000.0,
            depth: 2000.0,
            height: 3000.0,
        })
        .unwrap();
        let scene = Scene {
            name: "box".into(),
            parts: vec![Part {
                material: Material {
                    name: "paint".into(),
                    color: Some([0.6, 0.1, 0.1]),
                    color_space: ColorSpace::Srgb,
                    ..Default::default()
                },
                ..Part::new("box", &s)
            }],
        };
        let contract = crate::style_check::Contract {
            look: "looks/test.ogt".into(),
            palette: Some(("p".into(), vec![[160, 20, 20], [20, 20, 160]])),
            prop_faces: 6,
            ..Default::default()
        };
        let f = facts(&scene, Some(&contract));
        let size: Vec<f64> = serde_json::from_value(f["size"].clone()).unwrap();
        for (got, want) in size.iter().zip([1.0, 3.0, 2.0]) {
            assert!((got - want).abs() < 1e-9, "{f}");
        }
        assert_eq!(f["triangles"], 12, "{f}");
        assert_eq!(f["materials"][0]["colour"], "#991a1a", "{f}");
        assert_eq!(f["look"]["palette"][0]["nearest"], "#a01414", "{f}");
        assert!(
            f["look"]["palette"][0]["deltaE"].as_f64().unwrap() < 5.0,
            "{f}"
        );
        assert_eq!(f["look"]["facesRatio"], 2.0, "{f}");
        // No contract: no look block.
        assert!(facts(&scene, None).get("look").is_none());
    }
}
