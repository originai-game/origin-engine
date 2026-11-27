//! Native point declarations become scene instances against the same terrain
//! surface used by the terrain writer and navigation baker.
use super::{Document, Source};
use crate::cook::{
    source_content::Graph,
    source_schemas::procedural::{ScatterInput, ScatterTarget},
    systems::terrain,
};
use glam::{Quat, Vec2, Vec3};
use serde_json::json;

fn make(s: &Source, graph: &Graph) -> Result<Document, String> {
    let points: ScatterInput = s.input()?;
    let (model, asset) = match &points.target {
        ScatterTarget::Model { asset } => (true, &asset.id),
        ScatterTarget::Prefab { asset } => (false, &asset.id),
    };
    let asset = &graph.outputs[asset].asset;
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
    let align = points.align.unwrap_or(false);
    if align && points.project.is_none() {
        return Err("scatter align needs project".into());
    }
    let field = points
        .project
        .as_ref()
        .map(|r| terrain::get(s, &graph.outputs[&r.id].asset)?.surface())
        .transpose()?;
    let sampler = field
        .as_ref()
        .map(|f| og_render_data::terrain::TerrainSampler {
            size: Vec2::from_array(f.size),
            columns: f.columns,
            rows: f.rows,
            sample: |c, r| f.heights[(r * f.columns + c) as usize],
        });
    let mut doc = s
        .scene
        .as_ref()
        .ok_or("scatter needs its checked record context")?
        .as_ref()
        .clone();
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
        doc.scene.entities.push(serde_json::from_value(json!({
            "name": format!("point-{i}"),
            "model": model.then(|| json!({"$ogRef": asset})),
            "prefab": (!model).then(|| json!({"$ogRef": asset})),
            "components": {"og.Transform": {"position":p.to_array(), "quaternion":q.to_array(), "scale":scale.to_array()}}
        })).map_err(|e| e.to_string())?);
    }
    Ok(doc)
}

pub(crate) fn expand(sources: &mut [Source], graph: &Graph) {
    for source in sources {
        if source.scene.as_ref().is_none()
            || graph
                .nodes
                .get(&source.rel)
                .is_none_or(|n| n.entry.factory != "scatter")
            || source.error.is_some()
        {
            continue;
        }
        match make(source, graph) {
            Ok(doc) => source.scene = Some(std::sync::Arc::new(doc)),
            Err(error) => source.error = Some(error),
        }
    }
}
