//! Authored local edits use the live terrain brush and stamp kernels.
use super::terrain::Field;
use crate::cook::{
    formats::geometry,
    source_schemas::procedural::terrain::{BrushMode, StampMode, TerrainEdit},
};
use glam::{Vec2, Vec3};
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

pub(crate) fn prepare(
    input: Vec<TerrainEdit>,
    layers: usize,
    models: &geometry::Models<'_>,
) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for edit in input {
        match edit {
            TerrainEdit::Brush {
                mode,
                center,
                radius,
                strength,
                falloff,
                layer,
            } => {
                let mode = match mode {
                    BrushMode::Raise => 0,
                    BrushMode::Lower => 1,
                    BrushMode::Flatten => 2,
                    BrushMode::Smooth => 3,
                    BrushMode::Paint => 4,
                    BrushMode::Hole => 5,
                    BrushMode::Fill => 6,
                };
                let layer = layer.unwrap_or(0) as usize;
                if mode == 4 && layer >= layers.max(1) {
                    return Err(
                        "paint edit.layer must name an existing terrain material layer".into(),
                    );
                }
                steps.push(Step::Brush(Brush {
                    mode,
                    center: Vec3::from_array(center),
                    radius,
                    strength: strength.unwrap_or(0.0),
                    falloff: falloff.unwrap_or(0.0),
                    layer,
                    pages: layers.div_ceil(4).max(1),
                }));
            }
            TerrainEdit::Stamp {
                geometry: input,
                transform,
                mode,
                depth,
            } => {
                let mesh = og_shape::generators::merge(&[og_shape::generators::Part {
                    mesh: &geometry::build(&input, models)?,
                    placement: geometry::placement(transform.as_ref())?,
                    mirror: Default::default(),
                }])?;
                let triangles = mesh
                    .indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|t| t.map(|i| Vec3::from_array(mesh.positions[i as usize])))
                    .collect();
                steps.push(Step::Stamp {
                    triangles,
                    hole: matches!(mode, Some(StampMode::Hole)),
                    depth: depth.unwrap_or(0.0),
                });
            }
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
