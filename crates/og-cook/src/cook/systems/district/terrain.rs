//! The street footprint as a cook-time TerrainStamp. Block interiors are
//! not streets: subtract them before projecting onto the height field.

use glam::{DVec2, Vec3};
use og_core::ogt::{Block, Entry, Number, Value};
use og_shape::generators::{
    earcut::earcut,
    layout::{Layout, Poly},
};

use super::{build, parse::District};

pub(super) fn carve(d: &District, plan: &Layout, depth: f64) -> Entry {
    let mut pieces: Vec<Poly> = build::roads(d, plan)
        .into_iter()
        // Start with convex pieces: a half-plane cut of a concave region
        // can split it into disconnected polygons.
        .flat_map(|points| {
            let xy: Vec<f64> = points.iter().flat_map(|p| p.to_array()).collect();
            earcut(&xy, &[])
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| Poly::new(t.map(|i| points[i]).to_vec(), None).ccw())
                .collect::<Vec<_>>()
        })
        .collect();
    for block in &plan.blocks {
        let block = block.poly.clone().ccw();
        let mut outside = Vec::new();
        for mut inside in pieces {
            for i in 0..block.len() {
                if inside.len() < 3 {
                    break;
                }
                let p = block.points[i];
                let edge = block.points[(i + 1) % block.len()] - p;
                let normal = -edge.perp().normalize();
                let cut = p.dot(normal);
                let part = inside.clip(-normal, -cut, None).tidy();
                if part.len() >= 3 && part.area().abs() > 1e-9 {
                    outside.push(part);
                }
                inside = inside.clip(normal, cut, None).tidy();
            }
        }
        pieces = outside;
    }
    let num = |x| Value::Number(Number::from_f64(x).expect("finite street coordinate"));
    let mut entries = vec![Entry::new("depth").with(num(depth))];
    for p in pieces {
        let xy: Vec<f64> = p.points.iter().flat_map(|p| p.to_array()).collect();
        for t in earcut(&xy, &[]).as_chunks::<3>().0 {
            let mut e = Entry::new("triangle");
            for i in t {
                let DVec2 { x, y: z } = p.points[*i];
                e.values.extend([num(x), num(0.0), num(z)]);
            }
            entries.push(e);
        }
    }
    Entry::new("district-carve").with_block(Block { entries })
}

/// Applies the scan's street stamps before quantisation and tile/LOD
/// construction, so terrain sampling, collisions and rendering agree.
pub(crate) fn apply(
    settings: Option<&og_core::ogt::Document>,
    rel: &str,
    field: &mut super::super::terrain::Field,
) -> Result<(), String> {
    let Some(settings) = settings else {
        return Ok(());
    };
    for e in settings
        .body
        .get_all("district-carve")
        .filter(|e| e.values.first().and_then(Value::as_str) == Some(rel))
    {
        let b = e.block.as_ref().ok_or("a district carve takes a block")?;
        let depth = b
            .get("depth")
            .and_then(|e| e.values.first())
            .and_then(Value::as_number)
            .map(|n| n.as_f64() as f32)
            .ok_or("a district carve states its depth")?;
        let triangles = b
            .get_all("triangle")
            .map(|e| {
                let p: Vec<f32> = e
                    .values
                    .iter()
                    .filter_map(Value::as_number)
                    .map(|n| n.as_f64() as f32)
                    .collect();
                if p.len() != 9 || !p.iter().all(|x| x.is_finite()) {
                    return Err("a district carve triangle has nine finite coordinates".to_string());
                }
                Ok([
                    Vec3::from_slice(&p[..3]),
                    Vec3::from_slice(&p[3..6]),
                    Vec3::from_slice(&p[6..]),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let grid = og_render_data::terrain_stamp::Grid {
            size: field.size.into(),
            columns: field.columns,
            rows: field.rows,
        };
        let Some(([c, r, w, _], values)) =
            og_render_data::terrain_stamp::patch(grid, &triangles, false, depth)
        else {
            return Err(format!(
                "district streets cover no height samples of terrain `{}`",
                rel
            ));
        };
        for (i, v) in values.into_iter().enumerate() {
            if !v.is_nan() {
                let at = ((r + i as u32 / w) * field.columns + c + i as u32 % w) as usize;
                field.heights[at] = field.heights[at].min(v);
            }
        }
    }
    Ok(())
}
