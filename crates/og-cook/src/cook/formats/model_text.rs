//! Checked model inputs share the geometry kernels and one glTF writer.
mod input;
mod native;
mod plant_input;
mod surface;

pub(crate) use input::Prepared;
pub(crate) use surface::material;

use super::vox::Bin;
use glam::{DQuat, DVec3};
use og_shape::generators::{Mesh, Mirror, Placement};
use std::collections::BTreeMap;

const DIFFUSE_TRANSMISSION: &str = "KHR_materials_diffuse_transmission";
const MSFT_LOD: &str = "MSFT_lod";

/// Only nondefault foliage fields are written; the loader supplies the rest.
fn foliage_extras(dome: f64, s: &og_render_data::foliage::LeafSeason) -> Option<serde_json::Value> {
    let d = og_render_data::foliage::LeafSeason::EVERGREEN;
    let mut o = serde_json::Map::new();
    if dome > 0.0 {
        o.insert("domeNormals".into(), dome.into());
    }
    if s.season > 0.0 {
        o.insert("season".into(), s.season.into());
    }
    if s.autumn_color != d.autumn_color {
        o.insert("autumnColor".into(), s.autumn_color.to_vec().into());
    }
    if s.autumn_color2 != d.autumn_color2 {
        o.insert("autumnColor2".into(), s.autumn_color2.to_vec().into());
    }
    if s.shift != 0.0 {
        o.insert("seasonShift".into(), s.shift.into());
    }
    (!o.is_empty()).then(|| serde_json::json!({ "ogFoliage": o }))
}

pub(crate) fn textures(images: &[String]) -> (serde_json::Value, serde_json::Value) {
    let imgs: Vec<serde_json::Value> = images
        .iter()
        .map(|u| serde_json::json!({ "uri": u }))
        .collect();
    let texs: Vec<serde_json::Value> = (0..images.len())
        .map(|i| serde_json::json!({ "source": i, "sampler": 0 }))
        .collect();
    (imgs.into(), texs.into())
}

/// The extensions a model's JSON uses, by name, sorted.
pub(crate) fn extensions_used(json: &serde_json::Value) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    let nodes = json["nodes"].as_array().into_iter().flatten();
    for m in json["materials"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(nodes)
    {
        for k in m["extensions"]
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
        {
            out.insert(k.clone());
        }
    }
    out.into_iter().collect()
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

pub(crate) struct Built {
    pub json: serde_json::Value,
    pub bin: Bin,
    /// Triangles as drawn: each part's times its placements.
    pub drawn: usize,
}

fn write_mesh(bin: &mut Bin, name: &str, m: &Mesh, material: usize) -> serde_json::Value {
    serde_json::json!({ "name": name, "primitives": [write_primitive(bin, m, material, None)] })
}

/// Foliage carries its wind pivot/anchor and the leaf flag beside attributes.
fn write_primitive(
    bin: &mut Bin,
    m: &Mesh,
    material: usize,
    wind: Option<(&[[f32; 8]], bool)>,
) -> serde_json::Value {
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
    let mut vec4 = |v: &[[f32; 4]]| {
        bin.add(
            bytemuck::cast_slice(v),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
        )
    };
    if m.colors.len() == count {
        attributes["COLOR_0"] = vec4(&m.colors).into();
    }
    if m.tangents.len() == count {
        // Complete missing frames in mixed merges before writing glTF.
        let mut mesh = og_render_data::MeshData {
            vertices: (0..count)
                .map(|i| og_render_data::Vertex {
                    position: m.positions[i],
                    normal: m.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                    uv: m.uvs.get(i).copied().unwrap_or([0.0; 2]),
                    tangent: m.tangents[i],
                    ..Default::default()
                })
                .collect(),
            indices: m.indices.clone(),
        };
        mesh.generate_tangents();
        let tangents: Vec<_> = mesh.vertices.iter().map(|v| v.tangent).collect();
        attributes["TANGENT"] = vec4(&tangents).into();
    }
    if let Some((w, _)) = wind {
        let half = |at: usize| -> Vec<[f32; 4]> {
            w.iter()
                .map(|v| [v[at], v[at + 1], v[at + 2], v[at + 3]])
                .collect()
        };
        attributes["_OG_PIVOT"] = vec4(&half(0)).into();
        attributes["_OG_ANCHOR"] = vec4(&half(4)).into();
    }
    let idx = bin.add(
        bytemuck::cast_slice(&m.indices),
        Some(34963),
        serde_json::json!({ "componentType": 5125, "count": m.indices.len(), "type": "SCALAR" }),
    );
    let mut p =
        serde_json::json!({ "attributes": attributes, "indices": idx, "material": material });
    if matches!(wind, Some((_, true))) {
        p["extras"] = serde_json::json!({ "ogFoliage": { "leaves": true } });
    }
    p
}
