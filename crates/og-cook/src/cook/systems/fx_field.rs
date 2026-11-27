//! Vector fields are native lattices or captured texture inputs, never text.
use super::{Source, field, source_content};
use og_render_data::{MAX_FIELD_CELLS, VectorField};

pub(crate) fn bake(
    sources: &[Source],
    fields: &field::Fields,
    output: &source_content::Output,
    strength: f32,
    size: f32,
) -> Result<VectorField, String> {
    let id = &output.asset;
    let (cells, vectors) = if output.kind == "field" {
        let data = &field::get(fields, id)?.data;
        let [x, y, z] = data.dimensions;
        if x != y || y != z || data.channels != 3 || !(1..=MAX_FIELD_CELLS).contains(&x) {
            return Err(format!(
                "particle field '{id}' needs a cubic 3-channel lattice, 1..={MAX_FIELD_CELLS} samples a side"
            ));
        }
        (
            x,
            data.values
                .as_chunks::<3>()
                .0
                .iter()
                .map(|v| glam::Vec3::new(v[0], v[1], v[2]))
                .collect(),
        )
    } else {
        let source = sources
            .iter()
            .find(|s| &s.rel == id)
            .ok_or_else(|| format!("particle field texture '{id}' has no captured image"))?;
        let image = image::load_from_memory(&source.bytes)
            .map_err(|e| format!("field '{id}': {e}"))?
            .to_rgba8();
        let n = image.height();
        if !(1..=MAX_FIELD_CELLS).contains(&n) || image.width() != n * n {
            return Err(format!(
                "field '{id}' is {}x{}: a grid is N² wide and N high, N in 1..={MAX_FIELD_CELLS}",
                image.width(),
                n
            ));
        }
        let mut vectors = Vec::new();
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let p = image.get_pixel(x + z * n, n - 1 - y);
                    vectors.push(
                        glam::Vec3::new(f32::from(p[0]), f32::from(p[1]), f32::from(p[2])) / 255.0
                            * 2.0
                            - glam::Vec3::ONE,
                    );
                }
            }
        }
        (n, vectors)
    };
    Ok(VectorField::Grid {
        cells,
        vectors,
        strength,
        size,
    })
}
