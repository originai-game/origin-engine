//! Explicit static-mesh preparation through the same kernels as `og model`.
//! The file owns the choices; processing never guesses a budget or a rig.
use crate::cook::source_schemas::world::model::process as input;
use crate::grammar::Line;
use og_core::ogt::Block;
use og_shape::tools::{self, Atlas, Bake, BakeSource, Decimate, Part, Repair, Scene, Unwrap};
use std::{cell::RefCell, path::Path};

pub(crate) mod projection;
pub(crate) mod transfer;

/// An already imported dependency, including bytes read by its normal importer.
pub(crate) struct TransferInput {
    pub bytes: Vec<u8>,
    pub files: crate::cook::model_program::Read,
}

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
    "project", "{ ... }", "off", "Project registered authored views into prepared UVs through og-bake, preserving other material channels. See model.process.project.";
    "transfer", "{ ... }", "off", "Carry an imported base model's painted maps to named parts in their own unscaled metric frames, through og-bake. Runs after project.";
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
        if !matches!(e.key.as_str(), "joint" | "view")
            && b.entries[..i].iter().any(|v| v.key == e.key)
        {
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

/// External model import settings; authored models already hold native inputs.
pub(crate) fn settings(imported: Option<&Block>) -> Result<Option<input::Process>, String> {
    let sidecar = imported.and_then(|b| b.get("process"));
    if imported.is_some_and(|b| b.get_all("process").count() > 1) {
        return Err("a model declares process once".into());
    }
    let Some(e) = sidecar else {
        return Ok(None);
    };
    let b = e
        .block
        .as_ref()
        .filter(|_| e.values.is_empty())
        .ok_or("process takes a block")?;
    check(b, LINES)?;
    if b.entries.is_empty() {
        return Ok(None);
    }
    let repair = sub(b, "repair", REPAIR)?
        .map(|b| {
            Ok::<_, String>(input::Repair {
                weld: Some(number(b, "weld", 0.0, 1e9)?),
                weld_all: Some(boolean(b, "weldAll", false)?),
                floaters: Some(number(b, "floaters", 0.0, 1e9)?),
                floater_fraction: Some(number(b, "floaterFraction", 0.0, 1.0)?),
                non_manifold: Some(boolean(b, "nonManifold", false)?),
                hole_edges: Some(whole(b, "holeEdges", 0, u32::MAX)?),
                holes_through: Some(boolean(b, "holesThrough", false)?),
                thicken: Some(number(b, "thicken", 0.0, 1e9)?),
                fuse: Some(boolean(b, "fuse", false)?),
                min_edge: Some(number(b, "minEdge", 0.0, 1e9)?),
            })
        })
        .transpose()?;
    let decimate = sub(b, "decimate", DECIMATE)?
        .map(|b| {
            Ok::<_, String>(input::Decimate {
                target: Some(whole(b, "target", 0, u32::MAX)?),
                ratio: Some(number(b, "ratio", 0.0, 1.0)?),
                max_error: Some(number(b, "maxError", 0.0, 1e9)?),
                lock_border: Some(boolean(b, "lockBorder", false)?),
                merge_charts: Some(number(b, "mergeCharts", 0.0, 1e9)?),
                cross_color_borders: Some(boolean(b, "crossColorBorders", false)?),
            })
        })
        .transpose()?;
    let unwrap = sub(b, "unwrap", UNWRAP)?
        .map(|b| {
            Ok::<_, String>(input::Unwrap {
                angle: Some(number(b, "angle", 0.0, 89.999999)?),
                resolution: Some(whole(b, "resolution", 0, 8192)?),
                padding: Some(number(b, "padding", 0.0, 8192.0)?),
                shared: Some(boolean(b, "shared", false)?),
                conformal: Some(boolean(b, "conformal", false)?),
                seams: Some(boolean(b, "seams", false)?),
            })
        })
        .transpose()?;
    let bake = sub(b, "bake", BAKE)?
        .map(|b| {
            Ok::<_, String>(input::Bake {
                size: Some(whole(b, "size", 1024, 8192)?),
                distance: Some(number(b, "distance", 0.0, 1e9)?),
                padding: Some(whole(b, "padding", 4, 8192)?),
                dilate: Some(whole(b, "dilate", 0, u32::MAX)?),
                normals: Some(boolean(b, "normals", true)?),
            })
        })
        .transpose()?;
    let project = sub(b, "project", projection::LINES)?
        .map(projection::settings)
        .transpose()?;
    let transfer = sub(b, "transfer", transfer::LINES)?
        .map(transfer::settings)
        .transpose()?;
    let rig = sub(b, "rig", RIG)?
        .map(|b| {
            let mut joints = Vec::new();
            for e in b.get_all("joint") {
                if e.values.len() != 5 || e.block.is_some() {
                    return Err("joint takes name, parent, x y z".into());
                }
                let position = [e.f32(2), e.f32(3), e.f32(4)]
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;
                if position.iter().any(|v| !v.is_finite()) {
                    return Err("joint positions must be finite".into());
                }
                joints.push(input::Joint {
                    name: e.string(0).map_err(|e| e.to_string())?.into(),
                    parent: e.string(1).map_err(|e| e.to_string())?.into(),
                    position: position.try_into().unwrap(),
                });
            }
            let auto = boolean(b, "auto", false)?;
            if auto == !joints.is_empty() {
                return Err("rig needs either explicit joints or auto true".into());
            }
            Ok::<_, String>(if auto {
                input::Rig::Auto
            } else {
                input::Rig::Joints { joints }
            })
        })
        .transpose()?;
    Ok(Some(input::Process {
        repair,
        decimate,
        unwrap,
        bake,
        project,
        transfer,
        rig,
    }))
}

/// The result embeds maps; read dependencies remain attached to the original source.
pub(crate) fn apply(
    bytes: &[u8],
    resolve: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    b: &input::Process,
    parameters: &[u8],
    cache: Option<&Path>,
    base: Option<&TransferInput>,
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
    let projection = b
        .project
        .as_ref()
        .map(|b| projection::read(b, &resolve_tracked))
        .transpose()?;
    let transfer = b.transfer.as_ref();
    let base = if transfer.is_some() {
        Some(base.ok_or("transfer.source did not import as a model")?)
    } else {
        None
    };
    let mut read = read.into_inner();
    read.sort_by(|a, b| a.0.cmp(&b.0));
    read.dedup();
    let mut inputs = vec![bytes, parameters];
    for (name, data) in &read {
        inputs.extend([name.as_bytes(), data.as_slice()]);
    }
    if let Some(base) = base {
        inputs.push(&base.bytes);
        for (name, bytes) in &base.files {
            inputs.extend([name.as_bytes(), bytes.as_slice()]);
        }
    }
    let key = crate::cook::cache::key(&[crate::format::Area::Model], "model-process-1", &inputs);
    let store = cache.map(|p| crate::cook::cache::Store::new(p.to_path_buf()));
    if let Some((bytes, _)) = store.as_ref().and_then(|s| s.action(&key)) {
        return Ok((bytes, read));
    }
    let repair = b.repair.as_ref();
    let decimate = b.decimate.as_ref();
    let unwrap = b.unwrap.as_ref();
    let bake = b.bake.as_ref();
    let rig = b.rig.as_ref();
    let u = unwrap
        .map(|b| Unwrap {
            angle: b.angle.unwrap_or(0.0),
            resolution: b.resolution.unwrap_or(0),
            padding: b.padding.unwrap_or(0.0),
            shared: b.shared.unwrap_or(false),
            conformal: b.conformal.unwrap_or(false),
            seams: b.seams.unwrap_or(false),
        })
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
                    weld: b.weld.unwrap_or(0.0) * 1000.0,
                    weld_all: b.weld_all.unwrap_or(false),
                    floaters: tools::repair::Floaters {
                        size: b.floaters.unwrap_or(0.0) * 1000.0,
                        fraction: b.floater_fraction.unwrap_or(0.0),
                    },
                    non_manifold: b.non_manifold.unwrap_or(false),
                    max_hole_edges: b.hole_edges.unwrap_or(0) as usize,
                    holes_through: b.holes_through.unwrap_or(false),
                    thicken: b.thicken.unwrap_or(0.0) * 1000.0,
                    fuse: b.fuse.unwrap_or(false),
                    min_edge: b.min_edge.unwrap_or(0.0) * 1000.0,
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
                    target: b.target.unwrap_or(0) as usize,
                    ratio: b.ratio.unwrap_or(0.0),
                    max_error: b.max_error.unwrap_or(0.0) * 1000.0,
                    lock_border: b.lock_border.unwrap_or(false),
                    merge_charts: b.merge_charts.unwrap_or(0.0),
                    cross_color_borders: b.cross_color_borders.unwrap_or(false),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?
            .0;
        }
        if let Some(b) = bake {
            let size = b.size.unwrap_or(1024);
            if size == 0 {
                return Err("bake.size must be positive".into());
            }
            solid.uvs = None;
            tools::unwrap_atlas(
                &mut solid,
                &u,
                &Atlas {
                    size,
                    padding: b.padding.unwrap_or(4),
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
                    distance: b.distance.unwrap_or(0.0) * 1000.0,
                    dilate: b.dilate.unwrap_or(0),
                    no_normals: !b.normals.unwrap_or(true),
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
        let skeleton = match b {
            input::Rig::Auto => {
                og_shape::skin::humanoid(&tools::join(&imported.parts).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?
                    .0
            }
            input::Rig::Joints { joints } => og_shape::skin::Skeleton {
                joints: joints
                    .iter()
                    .map(|j| {
                        let [x, y, z] = j.position.map(f64::from);
                        og_shape::skin::Joint {
                            name: j.name.clone(),
                            parent: j.parent.clone(),
                            at: glam::DVec3::new(x, -z, y) * 1000.0,
                        }
                    })
                    .collect(),
            },
        };
        for solid in &mut solids {
            *solid = og_shape::skin::bind(solid, &skeleton)
                .map_err(|e| e.to_string())?
                .0;
        }
    }
    let mut scene = Scene {
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
    if let Some(projection) = &projection {
        projection::apply(&mut scene, projection)?;
    }
    if let (Some(transfer), Some(base)) = (transfer, base) {
        transfer::apply(&mut scene, transfer, base)?;
    }
    let bytes = og_shape_formats::gltf::write::glb(&scene).map_err(|e| e.to_string())?;
    if let Some(store) = store {
        let _ = store.put_action(&key, &bytes, &[]);
    }
    Ok((bytes, read))
}
