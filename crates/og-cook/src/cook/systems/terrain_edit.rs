//! Authored local edits use the live terrain brush and stamp kernels.
use super::{terrain::Field, terrain_schema};
use crate::cook::model_text;
use glam::{Vec2, Vec3};
use og_core::ogt::Block;
use og_render_data::{
    TerrainData,
    terrain::{
        edit::{self, Brush},
        stamp,
    },
};

pub(crate) enum Step {
    Brush(Brush),
    Stamp {
        triangles: Vec<[Vec3; 3]>,
        hole: bool,
        depth: f32,
    },
}

pub(crate) fn parse(body: &Block, layers: usize) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for e in body
        .entries
        .iter()
        .filter(|e| matches!(e.key.as_str(), "edit" | "stamp"))
    {
        let b = e
            .block
            .as_ref()
            .filter(|_| e.values.is_empty())
            .ok_or_else(|| format!("line {}: {} takes a block", e.line, e.key))?;
        let number = |key: &str, default: f32| -> Result<f32, String> {
            let Some(f) = b.get(key) else {
                return Ok(default);
            };
            if f.values.len() != 1 || f.block.is_some() {
                return Err(format!("line {}: {key} takes one finite number", f.line));
            }
            f.f32(0).map_err(|e| e.to_string()).and_then(|v| {
                if v.is_finite() {
                    Ok(v)
                } else {
                    Err(format!("{key} must be finite"))
                }
            })
        };
        if e.key == "edit" {
            terrain_schema::check(b, terrain_schema::EDIT, "terrain edit")?;
            let modes = [
                "raise", "lower", "flatten", "smooth", "paint", "hole", "fill",
            ];
            let mode = b.get("mode").ok_or("terrain edit needs mode")?;
            let mode = modes
                .iter()
                .position(|v| {
                    mode.values.len() == 1 && mode.block.is_none() && mode.word(0).ok() == Some(*v)
                })
                .ok_or("terrain edit mode is raise, lower, flatten, smooth, paint, hole or fill")?;
            let center = match b.get("center") {
                Some(e) if e.values.len() == 3 && e.block.is_none() => Vec3::new(
                    e.f32(0).map_err(|e| e.to_string())?,
                    e.f32(1).map_err(|e| e.to_string())?,
                    e.f32(2).map_err(|e| e.to_string())?,
                ),
                _ => return Err("terrain edit needs center x y z in terrain-local metres".into()),
            };
            let radius = number("radius", 0.0)?;
            let strength = number("strength", 0.0)?;
            let falloff = number("falloff", 0.0)?;
            let layer = number("layer", 0.0)?;
            if !center.is_finite()
                || radius <= 0.0
                || strength < 0.0
                || !(0.0..=1.0).contains(&falloff)
                || layer < 0.0
                || layer.fract() != 0.0
                || (mode == 4 && layer as usize >= layers.max(1))
            {
                return Err("terrain edit needs finite centre, positive radius, nonnegative strength, falloff 0..1 and a valid whole layer index".into());
            }
            steps.push(Step::Brush(Brush {
                mode: mode as u32,
                center,
                radius,
                strength,
                falloff,
                layer: layer as usize,
                pages: layers.div_ceil(4).max(1),
            }));
        } else {
            terrain_schema::check(b, &terrain_schema::stamp_lines(), "terrain stamp")?;
            let depth = number("depth", 0.0)?;
            let hole = match b.get("mode") {
                None => false,
                Some(e) if e.values.len() == 1 && e.block.is_none() => {
                    match e.word(0).map_err(|e| e.to_string())? {
                        "carve" => false,
                        "hole" => true,
                        _ => return Err("terrain stamp mode is carve or hole".into()),
                    }
                }
                Some(_) => return Err("terrain stamp mode takes one word".into()),
            };
            if depth < 0.0 {
                return Err("terrain stamp depth must be nonnegative".into());
            }
            let mut geometry = b.clone();
            geometry
                .entries
                .retain(|e| !matches!(e.key.as_str(), "mode" | "depth"));
            let mesh = model_text::placed_geometry(&geometry, "terrain stamp", e.line, 1.0)?;
            let triangles = mesh
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| t.map(|i| Vec3::from_array(mesh.positions[i as usize])))
                .collect();
            steps.push(Step::Stamp {
                triangles,
                hole,
                depth,
            });
        }
    }
    Ok(steps)
}

pub(crate) fn apply(mut field: Field, steps: &[Step]) -> Result<Field, String> {
    if steps.is_empty() {
        return Ok(field);
    }
    let mut data = TerrainData {
        size: Vec2::from_array(field.size),
        columns: field.columns,
        rows: field.rows,
        heights: field.heights.into(),
        control: field.control,
        holes: field.holes.map(Into::into),
    };
    for step in steps {
        match step {
            Step::Brush(b) => {
                let c = Vec2::new(b.center.x, b.center.z);
                let half = data.size * 0.5;
                if c.x + b.radius < -half.x
                    || c.x - b.radius > half.x
                    || c.y + b.radius < -half.y
                    || c.y - b.radius > half.y
                {
                    return Err("terrain edit brush misses the terrain".into());
                }
                edit::apply(&mut data, b);
            }
            Step::Stamp {
                triangles,
                hole,
                depth,
            } => {
                let (rect, values) = stamp::patch(stamp::Grid::of(&data), triangles, *hole, *depth)
                    .ok_or("terrain stamp has no projected samples on the terrain")?;
                stamp::apply(&mut data, *hole, rect, &values)?;
            }
        }
    }
    if data.heights.iter().any(|v| !v.is_finite()) {
        return Err("terrain edits produced non-finite heights".into());
    }
    field.heights = data.heights.to_vec();
    field.control = data.control;
    field.holes = data.holes.map(|v| v.to_vec());
    Ok(field)
}
