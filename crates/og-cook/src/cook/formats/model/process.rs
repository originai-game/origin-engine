//! Explicit static-mesh preparation through the same kernels as `og model`.
//! The file owns the choices; processing never guesses a budget or a rig.
use crate::grammar::Line;
use og_core::ogt::{Block, Document};
use og_shape::tools::{self, Atlas, Bake, BakeSource, Decimate, Part, Repair, Scene, Unwrap};
use std::{cell::RefCell, path::Path};

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(Line::new($key, $args, $default, $doc, "")),*]
    };
}

pub(crate) const LINES: &[Line] = lines![
    "repair", "{ ... }", "off", "Run the existing mesh repair kernel before simplification.";
    "decimate", "{ ... }", "off", "Explicit triangle/error budget, independently per material primitive.";
    "unwrap", "{ ... }", "off", "Replace UVs. Existing maps require bake so their appearance is not silently lost.";
    "bake", "{ ... }", "off", "Transfer the source material and normals to a fresh low-mesh atlas. No image generation.";
    "rig", "{ ... }", "off", "Bind explicit joints, or explicitly request the existing humanoid finder, after static processing."
];
pub(crate) const REPAIR: &[Line] = lines![
    "weld", "<metres >= 0>", "0", "Weld distance; zero welds exact duplicates.";
    "weldAll", "true | false", "false", "Also weld interior vertices.";
    "floaters", "<metres >= 0>", "0", "Drop disconnected pieces below this diagonal.";
    "floaterFraction", "<0..1>", "0", "Drop pieces below this share of the largest diagonal.";
    "nonManifold", "true | false", "false", "Split non-manifold sheets.";
    "holeEdges", "<integer >= 0>", "0", "Largest hole rim to close.";
    "holesThrough", "true | false", "false", "Permit a crossing fill of larger holes.";
    "thicken", "<metres >= 0>", "0", "Thickness given to open sheets.";
    "fuse", "true | false", "false", "Self-union closed interpenetrating shells.";
    "minEdge", "<metres >= 0>", "0", "Collapse edges below this length where manifoldness survives."
];
pub(crate) const DECIMATE: &[Line] = lines![
    "target", "<integer >= 0>", "0", "Maximum triangles per material primitive; zero is unset.";
    "ratio", "<0..1>", "0", "Triangle share; zero is unset. With target the smaller count wins.";
    "maxError", "<metres >= 0>", "0", "Quadric RMS displacement limit; zero is unlimited.";
    "lockBorder", "true | false", "false", "Keep open-border vertices fixed.";
    "mergeCharts", "<UV distance >= 0>", "0", "Join UV charts agreeing up to a rigid move.";
    "crossColorBorders", "true | false", "false", "Permit collapse across hard vertex-colour borders."
];
pub(crate) const UNWRAP: &[Line] = lines![
    "angle", "<degrees, 0..90)", "0 (kernel 66)", "Chart projection angle.";
    "resolution", "<integer, 0..8192>", "0 (kernel 1024)", "Grid used to measure padding.";
    "padding", "<texels >= 0>", "0 (kernel 2)", "Chart separation.";
    "shared", "true | false", "false", "Share chart projection directions.";
    "conformal", "true | false", "false", "Use angle-preserving flattening where valid.";
    "seams", "true | false", "false", "Try seam-cut charts before flattening."
];
pub(crate) const BAKE: &[Line] = lines![
    "size", "<integer, 1..8192>", "1024", "Atlas and material-map side in texels.";
    "distance", "<metres >= 0>", "0", "Source search distance; zero uses the kernel's 2% of diagonal.";
    "padding", "<integer, 0..8192>", "4", "Atlas gutters.";
    "dilate", "<integer >= 0>", "0", "Dilation layers; zero fills all uncovered texels.";
    "normals", "true | false", "true", "Transfer source shading normals as a tangent-space normal map."
];
pub(crate) const RIG: &[Line] = lines![
    "auto", "true | false", "false", "Explicitly run the existing humanoid joint finder; exclusive with joint lines.";
    "joint", "\"<name>\" \"<parent or empty>\" <x> <y> <z>", "", "Repeat parent-first joints in model-local metres, Y up. Each processed primitive is bound to the same skeleton."
];

fn check(b: &Block, fields: &[Line]) -> Result<(), String> {
    for (i, e) in b.entries.iter().enumerate() {
        if !fields.iter().any(|f| f.key == e.key) {
            return Err(format!(
                "line {}: unknown model processing field '{}'",
                e.line, e.key
            ));
        }
        if e.key != "joint" && b.entries[..i].iter().any(|v| v.key == e.key) {
            return Err(format!("line {}: '{}' is set twice", e.line, e.key));
        }
    }
    Ok(())
}

fn number(b: &Block, key: &str, default: f64, max: f64) -> Result<f64, String> {
    let Some(e) = b.get(key) else {
        return Ok(default);
    };
    let n = e
        .values
        .first()
        .and_then(|v| v.as_number())
        .map(|v| v.as_f64());
    match n {
        Some(v)
            if e.values.len() == 1
                && e.block.is_none()
                && v.is_finite()
                && (0.0..=max).contains(&v) =>
        {
            Ok(v)
        }
        _ => Err(format!(
            "line {}: {key} takes one finite number in 0..={max}",
            e.line
        )),
    }
}

fn whole(b: &Block, key: &str, default: u32, max: u32) -> Result<u32, String> {
    let v = number(b, key, f64::from(default), f64::from(max))?;
    if v.fract() != 0.0 {
        return Err(format!("{key} is a whole number"));
    }
    Ok(v as u32)
}

fn boolean(b: &Block, key: &str, default: bool) -> Result<bool, String> {
    let Some(e) = b.get(key) else {
        return Ok(default);
    };
    e.values
        .first()
        .and_then(|v| v.as_bool())
        .filter(|_| e.values.len() == 1 && e.block.is_none())
        .ok_or_else(|| format!("line {}: {key} takes true or false", e.line))
}

fn sub<'a>(b: &'a Block, key: &str, fields: &[Line]) -> Result<Option<&'a Block>, String> {
    let Some(e) = b.get(key) else { return Ok(None) };
    let inner = e
        .block
        .as_ref()
        .filter(|_| e.values.is_empty())
        .ok_or_else(|| format!("line {}: {key} takes a block", e.line))?;
    check(inner, fields)?;
    Ok(Some(inner))
}

/// Settings are owned by the model text, or its import.model block, not both.
pub(crate) fn settings(
    text: Option<&str>,
    imported: Option<&Block>,
) -> Result<Option<Block>, String> {
    let doc = text
        .map(Document::parse)
        .transpose()
        .map_err(|e| e.to_string())?;
    let authored = doc.as_ref().and_then(|d| d.body.get("process"));
    let sidecar = imported.and_then(|b| b.get("process"));
    if doc
        .as_ref()
        .is_some_and(|d| d.body.get_all("process").count() > 1)
        || imported.is_some_and(|b| b.get_all("process").count() > 1)
    {
        return Err("a model declares process once".into());
    }
    if authored.is_some() && sidecar.is_some() {
        return Err("process is declared in both model and import settings".into());
    }
    let Some(e) = authored.or(sidecar) else {
        return Ok(None);
    };
    let b = e
        .block
        .as_ref()
        .filter(|_| e.values.is_empty())
        .ok_or("process takes a block")?;
    check(b, LINES)?;
    Ok((!b.entries.is_empty()).then(|| b.clone()))
}

/// The result embeds maps; read dependencies remain attached to the original source.
pub(crate) fn apply(
    bytes: &[u8],
    resolve: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    b: &Block,
    cache: Option<&Path>,
) -> Result<(Vec<u8>, crate::cook::model_program::Read), String> {
    let (json, _) = super::plain::chunks(bytes)?;
    let json: serde_json::Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    if ["animations", "skins"]
        .iter()
        .any(|key| json[*key].as_array().is_some_and(|v| !v.is_empty()))
    {
        return Err("process takes a static model, not existing skins or animation".into());
    }
    let read = RefCell::new(Vec::new());
    let resolve_tracked = |uri: &str| {
        let bytes = resolve(uri)?;
        read.borrow_mut().push((uri.to_string(), bytes.clone()));
        Ok(bytes)
    };
    let imported = og_shape_formats::import(bytes, &Default::default(), &resolve_tracked)
        .map_err(|e| e.to_string())?;
    if imported.parts.is_empty()
        || !imported.skeletons.is_empty()
        || !imported.skipped.is_empty()
        || imported
            .parts
            .iter()
            .any(|p| p.solid.skin.is_some() || !p.clips.is_empty() || p.solid.morphs.is_some())
    {
        return Err("process requires static mesh primitives without skins, clips, morphs or skipped input; animation import remains the animation pipeline".into());
    }
    let mut read = read.into_inner();
    read.sort_by(|a, b| a.0.cmp(&b.0));
    read.dedup();
    let parameters = Document {
        ty: "model-process".into(),
        version: 1,
        body: b.clone(),
    }
    .to_text()
    .map_err(|e| e.to_string())?;
    let mut inputs = vec![bytes, parameters.as_bytes()];
    for (name, data) in &read {
        inputs.extend([name.as_bytes(), data.as_slice()]);
    }
    let key = crate::cook::cache::key(&[crate::format::Area::Model], "model-process-1", &inputs);
    let store = cache.map(|p| crate::cook::cache::Store::new(p.to_path_buf()));
    if let Some((bytes, _)) = store.as_ref().and_then(|s| s.action(&key)) {
        return Ok((bytes, read));
    }
    let repair = sub(b, "repair", REPAIR)?;
    let decimate = sub(b, "decimate", DECIMATE)?;
    let unwrap = sub(b, "unwrap", UNWRAP)?;
    let bake = sub(b, "bake", BAKE)?;
    let rig = sub(b, "rig", RIG)?;
    let u = unwrap
        .map(|b| {
            Ok::<_, String>(Unwrap {
                angle: number(b, "angle", 0.0, 89.999999)?,
                resolution: whole(b, "resolution", 0, 8192)?,
                padding: number(b, "padding", 0.0, 8192.0)?,
                shared: boolean(b, "shared", false)?,
                conformal: boolean(b, "conformal", false)?,
                seams: boolean(b, "seams", false)?,
            })
        })
        .transpose()?
        .unwrap_or_default();
    let mut solids = Vec::new();
    let mut materials = Vec::new();
    for part in &imported.parts {
        let source = tools::apply_transform(part).map_err(|e| e.to_string())?;
        let mut solid = source.clone();
        let mut material = part.material.clone();
        if let Some(b) = repair {
            solid = tools::repair(
                &solid,
                &Repair {
                    weld: number(b, "weld", 0.0, 1e9)? * 1000.0,
                    weld_all: boolean(b, "weldAll", false)?,
                    floaters: tools::repair::Floaters {
                        size: number(b, "floaters", 0.0, 1e9)? * 1000.0,
                        fraction: number(b, "floaterFraction", 0.0, 1.0)?,
                    },
                    non_manifold: boolean(b, "nonManifold", false)?,
                    max_hole_edges: whole(b, "holeEdges", 0, u32::MAX)? as usize,
                    holes_through: boolean(b, "holesThrough", false)?,
                    thicken: number(b, "thicken", 0.0, 1e9)? * 1000.0,
                    fuse: boolean(b, "fuse", false)?,
                    min_edge: number(b, "minEdge", 0.0, 1e9)? * 1000.0,
                    check_intersections: false,
                },
            )
            .map_err(|e| e.to_string())?
            .0;
        }
        if let Some(b) = decimate {
            solid = tools::decimate(
                &solid,
                &Decimate {
                    target: whole(b, "target", 0, u32::MAX)? as usize,
                    ratio: number(b, "ratio", 0.0, 1.0)?,
                    max_error: number(b, "maxError", 0.0, 1e9)? * 1000.0,
                    lock_border: boolean(b, "lockBorder", false)?,
                    merge_charts: number(b, "mergeCharts", 0.0, 1e9)?,
                    cross_color_borders: boolean(b, "crossColorBorders", false)?,
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?
            .0;
        }
        if let Some(b) = bake {
            let size = whole(b, "size", 1024, 8192)?;
            if size == 0 {
                return Err("bake.size must be positive".into());
            }
            solid.uvs = None;
            tools::unwrap_atlas(
                &mut solid,
                &u,
                &Atlas {
                    size,
                    padding: whole(b, "padding", 4, 8192)?,
                    no_turns: false,
                },
            )
            .map_err(|e| e.to_string())?;
            let baked = tools::bake(
                &BakeSource {
                    solid: &source,
                    materials: std::slice::from_ref(&material),
                    material_of: &[],
                },
                &solid,
                &Bake {
                    size,
                    distance: number(b, "distance", 0.0, 1e9)? * 1000.0,
                    dilate: whole(b, "dilate", 0, u32::MAX)?,
                    no_normals: !boolean(b, "normals", true)?,
                },
                &og_shape_formats::texture::CODEC,
            )
            .map_err(|e| e.to_string())?;
            if baked.report.colors_baked {
                solid.colors = None;
            }
            material = baked.material;
        } else if unwrap.is_some() {
            if !material.maps().is_empty() {
                return Err("unwrap on a textured primitive requires bake to retain its authored appearance".into());
            }
            tools::unwrap(&mut solid, &u).map_err(|e| e.to_string())?;
        }
        solids.push(solid);
        materials.push(material);
    }
    if let Some(b) = rig {
        let mut joints = Vec::new();
        for e in b.get_all("joint") {
            if e.values.len() != 5 || e.block.is_some() {
                return Err("joint takes name, parent, x y z".into());
            }
            let p = [e.f32(2), e.f32(3), e.f32(4)]
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            if p.iter().any(|v| !v.is_finite()) {
                return Err("joint positions must be finite".into());
            }
            joints.push(og_shape::skin::Joint {
                name: e.string(0).map_err(|e| e.to_string())?.into(),
                parent: e.string(1).map_err(|e| e.to_string())?.into(),
                at: glam::DVec3::new(p[0] as f64, -(p[2] as f64), p[1] as f64) * 1000.0,
            });
        }
        let auto = boolean(b, "auto", false)?;
        if auto == !joints.is_empty() {
            return Err("rig needs either explicit joints or auto true".into());
        }
        let skeleton = if auto {
            og_shape::skin::humanoid(&tools::join(&imported.parts).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
                .0
        } else {
            og_shape::skin::Skeleton { joints }
        };
        for solid in &mut solids {
            *solid = og_shape::skin::bind(solid, &skeleton)
                .map_err(|e| e.to_string())?
                .0;
        }
    }
    let scene = Scene {
        name: String::new(),
        parts: solids
            .iter()
            .zip(materials)
            .zip(&imported.parts)
            .map(|((solid, material), original)| Part {
                material,
                tangents: bake.is_some(),
                ..Part::new(&original.name, solid)
            })
            .collect(),
    };
    let bytes = og_shape_formats::gltf::write::glb(&scene).map_err(|e| e.to_string())?;
    if let Some(store) = store {
        let _ = store.put_action(&key, &bytes, &[]);
    }
    Ok((bytes, read))
}
