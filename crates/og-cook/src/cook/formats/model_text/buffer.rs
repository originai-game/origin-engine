//! Attribute-array sources use the runtime's buffer geometry validation.
//! Script expansion happens before model import, so the normal model cook
//! still owns placements, materials, LODs, mesh optimisation and collisions.
use super::*;
use crate::cook::{Source, assets_root, generator, model};
use crate::grammar::Line;
use og_core::ogt::Number;
use std::path::Path;

pub const LINES: &[Line] = &[
    Line::new(
        "positions",
        "<x y z> ...",
        "required",
        "Finite vertex positions in metres; at least one triangle.",
        "",
    ),
    Line::new(
        "normals",
        "<x y z> ...",
        "computed",
        "One normal per vertex, validated and normalised by bufferGeometry.",
        "",
    ),
    Line::new(
        "uvs",
        "<u v> ...",
        "0 0",
        "One UV per vertex, v down the image.",
        "",
    ),
    Line::new(
        "colors",
        "<r g b> ... | <r g b a> ...",
        "1 1 1 1",
        "Linear vertex colours, one RGB or RGBA value per vertex.",
        "",
    ),
    Line::new(
        "tangents",
        "<x y z w> ...",
        "generated when needed",
        "One glTF tangent frame per vertex; w is its handedness.",
        "",
    ),
    Line::new(
        "indices",
        "<u32> ...",
        "consecutive vertices",
        "Counter-clockwise triangle index triples within the vertex count.",
        "",
    ),
    Line::new(
        "generate",
        "{ ... }",
        "",
        "Deterministic script returns MeshSource attributes, exclusive with authored attribute arrays.",
        "",
    ),
    Line::new(
        "uvRepeat",
        "<u> <v>",
        "1 1",
        "Scale UVs after reading the source.",
        "",
    ),
    Line::new(
        "physics",
        "true | false",
        "true",
        "Runtime descriptor field; model collision is owned by model import settings.",
        "",
    ),
    Line::new(
        "dynamic",
        "true | false",
        "false",
        "Runtime descriptor field; a cooked model is static source data.",
        "",
    ),
];

pub(super) fn parse(b: &Block) -> Result<og_render_data::MeshData, String> {
    for e in &b.entries {
        if crate::grammar::find(LINES, &e.key, &[]).is_none() || b.get_all(&e.key).count() != 1 {
            return Err(at(e, "unknown or duplicate bufferGeometry field"));
        }
    }
    if b.get("generate").is_some() {
        return Err("bufferGeometry.generate must be expanded by cook".into());
    }
    let floats = |key: &str| -> Result<Option<Vec<f32>>, String> {
        b.get(key)
            .map(|e| {
                if e.block.is_some() {
                    return Err(at(e, "takes an attribute array, not a block"));
                }
                Ok(numbers(e)?.into_iter().map(|v| v as f32).collect())
            })
            .transpose()
    };
    let positions = floats("positions")?.ok_or("bufferGeometry needs positions")?;
    let normals = floats("normals")?;
    let uvs = floats("uvs")?;
    let colors = floats("colors")?;
    let tangents = floats("tangents")?;
    let indices = b
        .get("indices")
        .map(|e| {
            if e.block.is_some() {
                return Err(at(e, "takes triangle indices, not a block"));
            }
            e.values
                .iter()
                .map(|v| {
                    v.as_number()
                        .and_then(|n| n.as_u64().ok())
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| at(e, "indices are u32 integers"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    og_render_data::geometry::buffer_geometry(&og_render_data::geometry::BufferAttributes {
        positions: &positions,
        normals: normals.as_deref(),
        uvs: uvs.as_deref(),
        colors: colors.as_deref(),
        tangents: tangents.as_deref(),
        indices: indices.as_deref(),
    })
}

fn expand_block(b: &mut Block, root: &Path, rel: &str) -> Result<bool, String> {
    let mut changed = false;
    for e in &mut b.entries {
        let Some(body) = e.block.as_mut() else {
            continue;
        };
        if e.key == "bufferGeometry" && body.get("generate").is_some() {
            let entries: Vec<_> = body.get_all("generate").collect();
            if entries.len() != 1 || !entries[0].values.is_empty() {
                return Err("bufferGeometry has one generate block".into());
            }
            let spec = generator::Spec::standalone(
                entries[0].block.as_ref().ok_or("generate is a block")?,
            )?;
            let value = generator::Program::prepare(root, rel, &spec)?.run()?;
            let attrs = value
                .as_object()
                .ok_or("MeshSource is an attribute object")?;
            let mut generated = Block::new();
            for (key, value) in attrs {
                if !matches!(
                    key.as_str(),
                    "positions" | "normals" | "uvs" | "colors" | "tangents" | "indices"
                ) {
                    return Err(format!("MeshSource: unknown attribute '{key}'"));
                }
                if value.is_null() {
                    continue;
                }
                let mut entry = Entry::new(key);
                for v in value
                    .as_array()
                    .ok_or_else(|| format!("MeshSource.{key} is a numeric array"))?
                {
                    entry.values.push(
                        Number::from_f64(
                            v.as_f64()
                                .ok_or_else(|| format!("MeshSource.{key} is a numeric array"))?,
                        )
                        .map_err(|e| e.to_string())?
                        .into(),
                    );
                }
                generated.push(entry);
            }
            for field in &body.entries {
                if field.key == "generate" {
                    continue;
                }
                if !matches!(field.key.as_str(), "uvRepeat" | "physics" | "dynamic") {
                    return Err(at(field, "generated attributes cannot also be authored"));
                }
                generated.push(field.clone());
            }
            parse(&generated)?;
            *body = generated;
            changed = true;
        } else if matches!(
            e.key.as_str(),
            "part" | "a" | "b" | "csgGeometry" | "mergeGeometries"
        ) {
            changed |= expand_block(body, root, rel)?;
        }
    }
    Ok(changed)
}

pub(crate) fn expand(sources: &mut [Source]) {
    for s in sources {
        if s.kind != &model::MODEL || !s.rel.ends_with(".ogt") || s.error.is_some() {
            continue;
        }
        let result = (|| {
            let mut doc =
                Document::parse(&String::from_utf8_lossy(&s.bytes)).map_err(|e| e.to_string())?;
            if expand_block(&mut doc.body, &assets_root(s), &s.rel)? {
                s.bytes = doc.to_text().map_err(|e| e.to_string())?.into_bytes();
            }
            Ok::<_, String>(())
        })();
        if let Err(e) = result {
            s.error = Some(e);
        }
    }
}
