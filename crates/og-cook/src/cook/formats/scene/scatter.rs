//! A point/transform source becomes ordinary scene instances; projected
//! points use the terrain's authoritative CPU surface sampler.
use super::Source;
use crate::cook::{assets_root, generator, systems::terrain};
use crate::grammar::{Line, find};
use glam::{Quat, Vec2, Vec3};
use og_core::ogt::{Block, Document, Entry, Number, Value};
use serde::Deserialize;

pub(crate) const LINES: &[Line] = &[
    Line::new(
        "model",
        "\"<asset>\"",
        "",
        "Model to place at every transform; exclusive with prefab.",
        "",
    ),
    Line::new(
        "prefab",
        "\"<asset>\"",
        "",
        "Prefab to instantiate at every transform; exclusive with model.",
        "",
    ),
    Line::new(
        "point",
        "<x> <y> <z>",
        "",
        "Repeat for authored points, exclusive with generate. Positions use scene/terrain-local metres.",
        "",
    ),
    Line::new(
        "generate",
        "{ ... }",
        "",
        "Returns ScatterSource positions, optional quaternions and scales. May compose shared poissonDisk; no implicit distribution.",
        "",
    ),
    Line::new(
        "project",
        "\"<terrain.ogt>\"",
        "",
        "Add the named terrain surface height to each point's y. Reject off-surface points and holes, not clamp or discard them.",
        "",
    ),
    Line::new(
        "align",
        "true | false",
        "false",
        "Rotate local up to the projected surface normal, before the authored quaternion; needs project.",
        "",
    ),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Points {
    positions: Vec<f32>,
    quaternions: Option<Vec<f32>>,
    scales: Option<Vec<f32>>,
}

fn numbers(e: &Entry) -> Result<Vec<f32>, String> {
    if e.block.is_some() {
        return Err(format!("line {}: {} takes numbers", e.line, e.key));
    }
    e.values
        .iter()
        .map(|v| {
            v.as_number()
                .map(|n| n.as_f32())
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("line {}: finite numbers required", e.line))
        })
        .collect()
}

pub(crate) fn expand(
    s: &Source,
    doc: &Document,
    terrains: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Document, String> {
    if doc.version != 1 {
        return Err("scatter is version 1".into());
    }
    for e in &doc.body.entries {
        if find(LINES, &e.key, &[]).is_none()
            || (e.key != "point" && doc.body.get_all(&e.key).count() != 1)
        {
            return Err(format!(
                "line {}: unknown or duplicate scatter field '{}'",
                e.line, e.key
            ));
        }
    }
    let string = |key| -> Result<Option<String>, String> {
        doc.body
            .get(key)
            .map(|e| {
                if e.values.len() != 1 || e.block.is_some() {
                    return Err(format!("{key} takes one asset path"));
                }
                e.string(0).map(str::to_string).map_err(|e| e.to_string())
            })
            .transpose()
    };
    let model = string("model")?;
    let prefab = string("prefab")?;
    let (target, asset) = match (model, prefab) {
        (Some(v), None) => ("model", v),
        (None, Some(v)) => ("prefab", v),
        _ => return Err("scatter names exactly one model or prefab".into()),
    };
    let root = assets_root(s);
    let points = if let Some(e) = doc.body.get("generate") {
        if !e.values.is_empty() || doc.body.get("point").is_some() {
            return Err("scatter generate is exclusive with point lines".into());
        }
        let spec = generator::Spec::standalone(e.block.as_ref().ok_or("generate is a block")?)?;
        serde_json::from_value(generator::Program::prepare(&root, &s.rel, &spec)?.run()?)
            .map_err(|e| e.to_string())?
    } else {
        let mut positions = Vec::new();
        for e in doc.body.get_all("point") {
            let values = numbers(e)?;
            if values.len() != 3 {
                return Err(format!("line {}: point takes x y z", e.line));
            }
            positions.extend(values);
        }
        Points {
            positions,
            quaternions: None,
            scales: None,
        }
    };
    let count = points.positions.len() / 3;
    if !points.positions.len().is_multiple_of(3)
        || count > 1_000_000
        || points
            .quaternions
            .as_ref()
            .is_some_and(|q| q.len() != count * 4)
        || points.scales.as_ref().is_some_and(|q| q.len() != count * 3)
        || points
            .positions
            .iter()
            .chain(points.quaternions.iter().flatten())
            .chain(points.scales.iter().flatten())
            .any(|v| !v.is_finite())
    {
        return Err("scatter needs finite xyz positions, optional xyzw quaternions and xyz scales, one each per point (at most 1,000,000)".into());
    }
    let project = string("project")?;
    let align = doc
        .body
        .get("align")
        .map(|e| {
            if e.values.len() != 1 || e.block.is_some() {
                return Err("align takes true or false".to_string());
            }
            match e.word(0).map_err(|e| e.to_string())? {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err("align takes true or false".into()),
            }
        })
        .transpose()?
        .unwrap_or(false);
    if align && project.is_none() {
        return Err("scatter align needs project".into());
    }
    let field = project
        .as_ref()
        .map(|rel| {
            let rel = crate::runtime::asset_id(rel);
            let text = terrains(rel)?;
            let doc = terrain::parse(rel, &root, &text)?;
            terrain::field(&doc, &root, &s.fields)
        })
        .transpose()?;
    let sampler = field
        .as_ref()
        .map(|f| og_render_data::terrain::TerrainSampler {
            size: Vec2::from_array(f.size),
            columns: f.columns,
            rows: f.rows,
            sample: |c, r| f.heights[(r * f.columns + c) as usize],
        });
    let mut scene = Document::new("scene", 1);
    for (i, p) in points.positions.as_chunks::<3>().0.iter().enumerate() {
        let mut p = Vec3::from_array(*p);
        let mut q = points
            .quaternions
            .as_ref()
            .map_or(Quat::IDENTITY, |v| Quat::from_slice(&v[i * 4..i * 4 + 4]));
        if q.length_squared() < 1e-12 {
            return Err(format!("scatter point {i}: zero quaternion"));
        }
        q = q.normalize();
        if let (Some(sampler), Some(field)) = (&sampler, &field) {
            let xz = Vec2::new(p.x, p.z);
            let (c, r) = sampler
                .cell_of(xz)
                .ok_or_else(|| format!("scatter point {i}: outside projected terrain"))?;
            if field
                .holes
                .as_ref()
                .is_some_and(|h| h[(r * (field.columns - 1) + c) as usize] != 0)
            {
                return Err(format!("scatter point {i}: in a terrain hole"));
            }
            p.y += sampler.height(xz);
            if align {
                q = Quat::from_rotation_arc(Vec3::Y, sampler.normal(xz)) * q;
            }
        }
        let scale = points
            .scales
            .as_ref()
            .map_or(Vec3::ONE, |v| Vec3::from_slice(&v[i * 3..i * 3 + 3]));
        let mut transform = Block::new();
        let fields: [(&str, &[f32]); 3] = [
            ("position", p.as_ref()),
            ("quaternion", q.as_ref()),
            ("scale", scale.as_ref()),
        ];
        for (key, values) in fields {
            let mut e = Entry::new(key);
            for &v in values {
                e.values.push(
                    Number::from_f64(v as f64)
                        .map_err(|e| e.to_string())?
                        .into(),
                );
            }
            transform.push(e);
        }
        let mut body = Block::new();
        body.push(Entry::new(target).with(Value::String(asset.clone())));
        body.push(Entry::new("Transform").with_block(transform));
        scene.body.push(
            Entry::new("entity")
                .with(Value::String(format!("point-{i}")))
                .with_block(body),
        );
    }
    Ok(scene)
}
