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
//! (+Y up), `triangles`, `vertices`, `partCount`, `colours` (distinct
//! sRGB colours over untextured materials and every map's opaque texels)
//! and `materials`. In a
//! project whose look sets an art contract (read from its cooked look, as
//! `og check` does), `look` adds how the model sits against it: each
//! material's colour, its nearest palette colour and their CIEDE2000, and
//! the triangles against `ArtShape.propFaces` and `characterFaces`, and
//! the colours against `ArtPalette.maxColors`. A fact the look does not
//! state is null, never 0 or an empty list, and `look.absent` gives the
//! reason ("the look states no palette (ArtPalette.colors)"). Facts only: nothing here
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
    // Every colour the model shows: untextured materials' colours and
    // each map's opaque texels (sRGB), as ArtPalette.maxColors counts.
    let mut colours = std::collections::BTreeSet::new();
    for p in &scene.parts {
        match p
            .material
            .map
            .as_ref()
            .and_then(|t| og_shape_formats::texture::decode(&t.bytes).ok())
        {
            Some(img) => colours.extend(
                img.rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|px| px[3] >= 128)
                    .map(|px| [px[0], px[1], px[2]]),
            ),
            None => {
                colours.insert(colour(&p.material));
            }
        }
    }
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
        "colours": colours.len(),
        "materials": materials.iter().map(|(n, c)| json!({ "material": n, "colour": hex(*c) })).collect::<Vec<_>>(),
    });
    if let Some(c) = contract {
        out["look"] = look_facts(c, &materials, triangles, colours.len());
    }
    out
}

/// The model against the look's art contract. A fact the look does not
/// state (its field at the zero value, which constrains nothing) is null,
/// and `absent` says why, so no 0 reads as a budget of nothing.
fn look_facts(
    c: &crate::style_check::Contract,
    materials: &[(String, [u8; 3])],
    triangles: usize,
    colours: usize,
) -> Value {
    use crate::commands::compare::{delta_e00, lab};
    let mut absent = serde_json::Map::new();
    let mut stated = |key: &str, set: bool, why: &str| {
        if !set {
            absent.insert(
                key.into(),
                Value::String(format!("the look states no {why}")),
            );
        }
        set
    };
    let palette = match &c.palette {
        Some((_, p)) if stated("palette", !p.is_empty(), "palette (ArtPalette.colors)") => Some((
            p.len(),
            materials
                .iter()
                .map(|(n, rgb)| {
                    let (near, de) = p
                        .iter()
                        .map(|q| (*q, delta_e00(lab(*rgb), lab(*q))))
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .expect("a palette with colours");
                    json!({ "material": n, "colour": hex(*rgb), "nearest": hex(near), "deltaE": de })
                })
                .collect::<Vec<_>>(),
        )),
        Some(_) => None,
        None => {
            stated("palette", false, "palette (ArtPalette.colors)");
            None
        }
    };
    let budget = |n: u32, of: usize| (n > 0).then(|| (n, of as f64 / f64::from(n)));
    let prop = budget(c.prop_faces, triangles);
    stated(
        "propFaces",
        prop.is_some(),
        "prop triangle budget (ArtShape.propFaces)",
    );
    let character = budget(c.character_faces, triangles);
    stated(
        "characterFaces",
        character.is_some(),
        "character triangle budget (ArtShape.characterFaces)",
    );
    let max_colors = budget(c.max_colors, colours);
    stated(
        "maxColors",
        max_colors.is_some(),
        "colour budget (ArtPalette.maxColors)",
    );
    let density = (c.density > 0.0).then_some(c.density);
    stated(
        "texelDensity",
        density.is_some(),
        "texel density (ArtTexels.density)",
    );
    let max_size = (c.max_size > 0).then_some(c.max_size);
    stated(
        "maxTextureSize",
        max_size.is_some(),
        "largest texture (ArtTexels.maxSize)",
    );
    json!({
        "look": c.look,
        "palette": palette.as_ref().map(|p| &p.1),
        "paletteColours": palette.as_ref().map(|p| p.0),
        "propFaces": prop.map(|p| p.0),
        "facesRatio": prop.map(|p| p.1),
        "characterFaces": character.map(|p| p.0),
        "characterFacesRatio": character.map(|p| p.1),
        "maxColors": max_colors.map(|p| p.0),
        "coloursRatio": max_colors.map(|p| p.1),
        "texelDensity": density,
        "maxTextureSize": max_size,
        "absent": absent,
    })
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// A material's colour as drawn, sRGB: its base colour times the mean of
/// its base-colour map when the map decodes.
pub(super) fn colour(m: &og_shape::tools::Material) -> [u8; 3] {
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
pub fn project_of(a: &Args, input: &super::Input) -> Result<Option<PathBuf>, Failure> {
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
            max_colors: 4,
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
        // One flat colour of a budget of 4; no character budget set.
        assert_eq!(f["colours"], 1, "{f}");
        assert_eq!(f["look"]["coloursRatio"], 0.25, "{f}");
        assert!(f["look"]["characterFacesRatio"].is_null(), "{f}");
        assert!(f["look"]["characterFaces"].is_null(), "{f}");
        assert!(
            f["look"]["absent"]["characterFaces"]
                .as_str()
                .unwrap()
                .contains("states no"),
            "{f}"
        );
        assert!(f["look"]["absent"].get("palette").is_none(), "{f}");
        // A look that states nothing: every fact null, each with a reason,
        // never a budget of 0.
        let none = facts(&scene, Some(&crate::style_check::Contract::default()));
        for k in [
            "palette",
            "paletteColours",
            "propFaces",
            "maxColors",
            "texelDensity",
        ] {
            assert!(none["look"][k].is_null(), "{k}: {none}");
        }
        assert_eq!(
            none["look"]["absent"].as_object().unwrap().len(),
            6,
            "{none}"
        );
        // No contract: no look block.
        assert!(facts(&scene, None).get("look").is_none());
    }
}
