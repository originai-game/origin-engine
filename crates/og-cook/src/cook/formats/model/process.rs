//! Explicit static-mesh preparation through the same kernels as `og model`.
//! The file owns the choices; processing never guesses a budget or a rig.
use crate::cook::source_schemas::world::model::process as input;
use og_shape::tools::{self, Atlas, Bake, BakeSource, Decimate, Part, Repair, Scene, Unwrap};
use std::{cell::RefCell, path::Path};

pub(crate) mod projection;
pub(crate) mod transfer;

/// An already imported dependency, including bytes read by its normal importer.
pub(crate) struct TransferInput {
    pub bytes: Vec<u8>,
    pub files: crate::cook::model_program::Read,
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
