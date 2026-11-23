//! A cell's (or a building's) meshes as one glTF binary: a primitive per
//! material, and levels of detail when its grammars mark parts with
//! `detail`.
//!
//! Level 0 draws every part. A part of detail `d` above 0 is drawn in the
//! levels under `d` and left out from level `d` on (a sill of `detail 1`
//! is gone from level 1); the model has a level for each detail its parts
//! name, as glTF `MSFT_lod` nodes, and the cook keeps them as its levels.

use og_core::ogt::{Block, Value};
use og_shape::generators::Mesh;

use super::build::Acc;
use super::parse::Material;
use crate::cook::model_text;
use crate::cook::vox::Bin;

fn primitive(bin: &mut Bin, m: &Mesh, material: usize) -> serde_json::Value {
    let count = m.positions.len();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in &m.positions {
        for a in 0..3 {
            lo[a] = lo[a].min(p[a]);
            hi[a] = hi[a].max(p[a]);
        }
    }
    let mut add = |data: &[u8], ty: &str, extra: serde_json::Value| -> usize {
        let mut acc = serde_json::json!({ "componentType": 5126, "count": count, "type": ty });
        if let Some(o) = extra.as_object() {
            for (k, v) in o {
                acc[k] = v.clone();
            }
        }
        bin.add(data, Some(34962), acc)
    };
    let attributes = serde_json::json!({
        "POSITION": add(bytemuck::cast_slice(&m.positions), "VEC3", serde_json::json!({ "min": lo, "max": hi })),
        "NORMAL": add(bytemuck::cast_slice(&m.normals), "VEC3", serde_json::json!({})),
        "TEXCOORD_0": add(bytemuck::cast_slice(&m.uvs), "VEC2", serde_json::json!({})),
        "COLOR_0": add(bytemuck::cast_slice(&m.colors), "VEC4", serde_json::json!({})),
    });
    let indices = bin.add(
        bytemuck::cast_slice(&m.indices),
        Some(34963),
        serde_json::json!({ "componentType": 5125, "count": m.indices.len(), "type": "SCALAR" }),
    );
    serde_json::json!({ "attributes": attributes, "indices": indices, "material": material })
}

/// Joins meshes into one.
fn joined<'a>(parts: impl Iterator<Item = &'a Mesh>) -> Mesh {
    let mut out = Mesh::default();
    for m in parts {
        let first = out.positions.len() as u32;
        out.positions.extend(&m.positions);
        out.normals.extend(&m.normals);
        out.uvs.extend(&m.uvs);
        out.colors.extend(&m.colors);
        out.indices.extend(m.indices.iter().map(|i| first + i));
    }
    out
}

/// The glb of `acc`, its materials `materials` (each map named by asset
/// id, given to the glTF as `uri` makes it).
pub(super) fn write(
    acc: &Acc,
    materials: &[&Material],
    uri: &dyn Fn(&str) -> String,
) -> Result<Vec<u8>, String> {
    let mut images = Vec::new();
    let mut mats = Vec::new();
    for m in materials {
        let mut fields = Block::new();
        for e in &m.fields.entries {
            let mut e = e.clone();
            if matches!(e.key.as_str(), "map" | "normalMap")
                && let Some(Value::String(id)) = e.values.first()
            {
                e.values[0] = Value::String(uri(crate::runtime::asset_id(id)));
            }
            fields.push(e);
        }
        mats.push(
            model_text::material(&m.name, &fields, &mut images, 0)
                .map_err(|e| format!("material `{}`: {e}", m.name))?,
        );
    }
    let index = |name: &str| {
        materials
            .iter()
            .position(|m| m.name == name)
            .expect("a used material")
    };
    let top = acc.keys().map(|(_, d)| *d).max().unwrap_or(0);
    let mut bin = Bin::default();
    let mut meshes = Vec::new();
    for level in 0..=top {
        // Each material's parts drawn at this level, joined.
        let mut prims = Vec::new();
        for m in materials {
            let mesh = joined(
                acc.iter()
                    .filter(|((name, d), _)| name == &m.name && (*d == 0 || level < *d))
                    .map(|(_, mesh)| mesh),
            );
            if !mesh.indices.is_empty() {
                prims.push(primitive(&mut bin, &mesh, index(&m.name)));
            }
        }
        if prims.is_empty() {
            break;
        }
        meshes.push(serde_json::json!({ "name": format!("lod {level}"), "primitives": prims }));
    }
    let mut nodes = vec![serde_json::json!({ "name": "district", "mesh": 0 })];
    if meshes.len() > 1 {
        let ids: Vec<usize> = (1..meshes.len()).collect();
        for i in &ids {
            nodes.push(serde_json::json!({ "name": format!("lod {i}"), "mesh": i }));
        }
        nodes[0]["extensions"] = serde_json::json!({ "MSFT_lod": { "ids": ids } });
    }
    let mut json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook type district" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": mats,
    });
    if !images.is_empty() {
        let (imgs, texs) = model_text::textures(&images);
        json["images"] = imgs;
        json["textures"] = texs;
        json["samplers"] = serde_json::json!([{ "wrapS": 10497, "wrapT": 10497 }]);
    }
    let mut used = model_text::extensions_used(&json);
    if json["nodes"][0]["extensions"].is_object() && !used.contains(&"MSFT_lod".to_string()) {
        used.push("MSFT_lod".into());
    }
    if !used.is_empty() {
        used.sort();
        json["extensionsUsed"] = used.into();
    }
    bin.finish(json)
}
