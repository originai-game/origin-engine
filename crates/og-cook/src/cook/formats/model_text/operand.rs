//! Asset operands are resolved after their source model imports, then
//! enter the existing geometry mechanisms as validated attribute buffers.
use super::*;
use crate::grammar::Line;
use og_core::ogt::Number;

pub(crate) const LINES: &[Line] = &[
    Line::new(
        "source",
        "\"<model asset>\"",
        "required",
        "Asset-root relative model, imported before this model. Cycles are refused.",
        "",
    ),
    Line::new(
        "node",
        "\"<node name>\"",
        "all visible",
        "Select a mesh node by name, otherwise merge the visible default scene at rest. Node placements, UVs, colours and tangent frames are retained. The surrounding part owns the material; skins and animation are not copied into a static geometry operand.",
        "",
    ),
];

fn reference(e: &Entry) -> Result<(String, Option<String>), String> {
    let b = e.block.as_ref().ok_or("modelGeometry is a block")?;
    let keys: Vec<_> = LINES.iter().map(|f| f.key).collect();
    Fields::new(b, "modelGeometry", &keys)?;
    if !e.values.is_empty() {
        return Err(at(e, "takes no values before its block"));
    }
    let string = |key| -> Result<Option<String>, String> {
        b.get(key)
            .map(|e| {
                if e.values.len() != 1 || e.block.is_some() {
                    return Err(at(e, "takes one string"));
                }
                e.string(0).map(str::to_string).map_err(|e| e.to_string())
            })
            .transpose()
    };
    let id = string("source")?.ok_or("modelGeometry needs source")?;
    Ok((
        crate::cook::normalise_rel(crate::runtime::asset_id(&id)),
        string("node")?,
    ))
}

fn walk(b: &mut Block, f: &mut dyn FnMut(&mut Entry) -> Result<(), String>) -> Result<(), String> {
    for e in &mut b.entries {
        if e.key == "modelGeometry" {
            f(e)?;
        } else if matches!(
            e.key.as_str(),
            "part"
                | "a"
                | "b"
                | "csgGeometry"
                | "mergeGeometries"
                | "plant"
                | "leafGeometry"
                | "flowerGeometry"
                | "stamp"
        ) && let Some(b) = e.block.as_mut()
        {
            walk(b, f)?;
        }
    }
    Ok(())
}

pub(crate) fn references(bytes: &[u8]) -> Vec<String> {
    let Ok(mut doc) = Document::parse(&String::from_utf8_lossy(bytes)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let _ = walk(&mut doc.body, &mut |e| {
        if let Ok((id, _)) = reference(e) {
            out.push(id);
        }
        Ok(())
    });
    out
}

pub(crate) fn expand(
    bytes: &[u8],
    resolve: impl Fn(&str, Option<&str>) -> Result<Mesh, String>,
) -> Result<Vec<u8>, String> {
    let mut doc = Document::parse(&String::from_utf8_lossy(bytes)).map_err(|e| e.to_string())?;
    walk(&mut doc.body, &mut |e| {
        let (id, node) = reference(e)?;
        let mesh = resolve(&id, node.as_deref())?;
        let mut body = Block::new();
        let fields: [(&str, &[f32]); 5] = [
            ("positions", bytemuck::cast_slice(&mesh.positions)),
            ("normals", bytemuck::cast_slice(&mesh.normals)),
            ("uvs", bytemuck::cast_slice(&mesh.uvs)),
            ("colors", bytemuck::cast_slice(&mesh.colors)),
            ("tangents", bytemuck::cast_slice(&mesh.tangents)),
        ];
        for (key, values) in fields {
            if values.is_empty() {
                continue;
            }
            let mut entry = Entry::new(key);
            for &v in values {
                entry.values.push(
                    Number::from_f64(v as f64)
                        .map_err(|e| e.to_string())?
                        .into(),
                );
            }
            body.push(entry);
        }
        let mut indices = Entry::new("indices");
        indices
            .values
            .extend(mesh.indices.into_iter().map(|v| Number::from(v).into()));
        body.push(indices);
        *e = Entry::new("bufferGeometry").with_block(body);
        Ok(())
    })?;
    doc.to_text()
        .map(String::into_bytes)
        .map_err(|e| e.to_string())
}
