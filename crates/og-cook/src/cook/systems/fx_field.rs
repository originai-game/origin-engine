//! An emitter's `field "<texture>" <strength> <size>` (og_bundle::fx):
//! the cook reads the image, an N² × N grid of vectors (pixel (x + z·N, y
//! from the bottom) is cell (x, y, z), rgb 0..1 as -1..1, N 1 to 16), and
//! writes the line as `fieldGrid <N> <size> <strength>` and the vectors,
//! which the runtime reads back without the image.
//! Canonical TS field IDs instead consume the checked lattice snapshot,
//! preserving its float vectors without image remapping or text decoding.

use super::Source;
use og_core::ogt::{Document, Value};

/// The texture ids of an emitter's field lines.
fn fields(text: &str) -> Vec<(usize, String, Vec<f32>)> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    doc.body
        .entries
        .iter()
        .filter(|e| e.key == "field")
        .filter_map(|e| {
            let id = e.values.first()?.as_str()?.to_string();
            let rest = e.values[1..]
                .iter()
                .filter_map(Value::as_number)
                .map(|n| n.as_f32())
                .collect();
            Some((e.line, id, rest))
        })
        .collect()
}

fn read(s: &Source, id: &str) -> Result<Vec<u8>, String> {
    let path = crate::cook::assets_root(s).join(crate::runtime::asset_id(id));
    std::fs::read(&path).map_err(|e| format!("field {id}: {e}"))
}

/// The bytes' hash of every field texture, for the emitter's stamp.
pub(crate) fn stamp(s: &Source, text: &str) -> String {
    fields(text)
        .iter()
        .map(|(_, id, _)| {
            let h = s.fields.get(id).map_or_else(
                || read(s, id).map_or(0, |b| og_core::hash::hash_bytes(&b)),
                |field| field.hash,
            );
            format!("field {id}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `text` with each `field "<texture>"` line baked into a `fieldGrid` line.
pub(crate) fn bake(s: &Source, text: &str) -> Result<String, String> {
    let found = fields(text);
    if found.is_empty() {
        return Ok(text.to_string());
    }
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    for (line, id, rest) in found {
        let [strength, size] = rest[..] else {
            return Err(format!(
                "line {line}: field \"<texture>\" <strength> <size>"
            ));
        };
        if !(size.is_finite() && size > 0.0) {
            return Err(format!("line {line}: field size {size} (above 0)"));
        }
        let (n, vectors) = if let Some(field) = s.fields.get(&id) {
            let data = &field.data;
            let [x, y, z] = data.dimensions;
            if x != y
                || y != z
                || data.channels != 3
                || !(1..=og_render_data::MAX_FIELD_CELLS).contains(&x)
            {
                return Err(format!(
                    "line {line}: particle field {id} needs a cubic 3-channel lattice, 1..={} samples a side",
                    og_render_data::MAX_FIELD_CELLS
                ));
            }
            (x, data.values.clone())
        } else {
            let bytes = read(s, &id)?;
            let img = image::load_from_memory(&bytes)
                .map_err(|e| format!("line {line}: field {id}: {e}"))?
                .to_rgba8();
            let n = img.height();
            if !(1..=og_render_data::MAX_FIELD_CELLS).contains(&n) || img.width() != n * n {
                return Err(format!(
                    "line {line}: field {id} is {}x{}: a grid of N cells a side is N² wide and N high (N 1 to {})",
                    img.width(),
                    img.height(),
                    og_render_data::MAX_FIELD_CELLS
                ));
            }
            let mut vectors = Vec::new();
            for z in 0..n {
                for y in 0..n {
                    for x in 0..n {
                        let p = img.get_pixel(x + z * n, n - 1 - y);
                        for c in &p.0[..3] {
                            let v = f32::from(*c) / 255.0 * 2.0 - 1.0;
                            vectors.push(v);
                        }
                    }
                }
            }
            (n, vectors)
        };
        let mut out = format!("{} {n} {size} {strength}", og_bundle::fx::FIELD_GRID);
        for value in vectors {
            out.push_str(&format!(" {value}"));
        }
        let slot = lines
            .get_mut(line - 1)
            .ok_or_else(|| format!("line {line}: field line not found"))?;
        let indent: String = slot.chars().take_while(|c| c.is_whitespace()).collect();
        *slot = format!("{indent}{out}");
    }
    let mut baked = lines.join("\n");
    baked.push('\n');
    Ok(baked)
}
