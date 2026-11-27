//! Checked TS pixel-effect recipes rendered into native flipbook sheets.
//! The Forge renderer, packing, impact selection and pixel output are unchanged.
mod color;
mod engine;
mod jsmath;
mod json;
mod layers;

use super::{Source, source_schemas::media::vfx2d::Vfx2d};
pub use engine::{ENGINE_VERSION, LAYER_TYPES, Rendered};
use og_anim::{FlipClip, FlipMode, FlipbookSheet, SheetFilter, SheetFrame};
use serde_json::Value;
use std::sync::Arc;

/// Convert checked recipe data directly to the renderer's ECMAScript values.
fn value(v: &Value) -> json::Val {
    match v {
        Value::Null => json::Val::Null,
        Value::Bool(b) => json::Val::Bool(*b),
        Value::Number(n) => json::Val::Num(n.as_f64().expect("schema-checked number")),
        Value::String(s) => json::Val::Str(s.clone()),
        Value::Array(a) => json::Val::Arr(a.iter().map(value).collect()),
        Value::Object(o) => json::Val::Obj(o.iter().map(|(k, v)| (k.clone(), value(v))).collect()),
    }
}

/// Rendered data retained for tools; no author text is generated or parsed.
pub struct Built {
    pub sheet: FlipbookSheet,
    pub png: Vec<u8>,
    pub rendered: Rendered,
    pub impact: usize,
}

fn frame_at(t: f32, frames: usize, duration: f32, what: &str) -> Result<usize, String> {
    if t > duration * (1.0 + 1e-6) {
        return Err(format!(
            "{what} at {t} s is after the duration, {duration} s"
        ));
    }
    let f = (f64::from(t) * frames as f64 / f64::from(duration) - 1e-6).ceil();
    Ok((f.max(0.0) as usize).min(frames - 1))
}

/// Build a schema-checked recipe with the existing native Forge renderer.
pub fn build(rel: &str, data: &Value) -> Result<Built, String> {
    let fx = Vfx2d::schema()
        .decode(data.clone())
        .map_err(|e| e.to_string())?;
    let r = engine::render(&value(&data["recipe"]));
    let n = r.frames.len();
    let (w, h) = (r.w, r.h);
    if n == 0 || w == 0 || h == 0 || r.fps == 0 {
        return Err("the vfx2d recipe rendered no frames".into());
    }
    let looping = fx.looping.unwrap_or(r.looping);
    let duration = fx.duration.unwrap_or(n as f32 / r.fps as f32);
    if duration <= 0.0 || !duration.is_finite() {
        return Err("duration must be positive and finite".into());
    }
    let impact = match fx.impact {
        Some(t) => frame_at(t, n, duration, "impact")?,
        None => r.impact.min(n - 1),
    };
    let mut events = vec![(impact as u32, "impact".to_string())];
    for e in fx.events {
        if e.name.is_empty() || e.name.len() > 63 {
            return Err("an event name is 1..63 UTF-8 bytes".into());
        }
        events.push((frame_at(e.time, n, duration, &e.name)? as u32, e.name));
    }
    if events.len() > 16 {
        return Err("at most 16 events including impact".into());
    }
    og_core::sort::sort_by_key(&mut events, |e| e.0);
    let cols = if (n as u32).saturating_mul(w) <= 4096 {
        n as u32
    } else {
        ((f64::from(n as u32 * h) / f64::from(w)).sqrt().ceil() as u32).clamp(1, n as u32)
    };
    let rows = (n as u32).div_ceil(cols);
    let (tw, th) = (cols.saturating_mul(w), rows.saturating_mul(h));
    if tw == 0 || th == 0 || tw > 8192 || th > 8192 {
        return Err(format!(
            "the packed vfx2d sheet ({tw}x{th}) exceeds the 8192x8192 sheet limit"
        ));
    }
    let mut pixels = vec![0u8; (tw * th * 4) as usize];
    let origin = |i: usize| ((i as u32 % cols) * w, (i as u32 / cols) * h);
    for (i, f) in r.frames.iter().enumerate() {
        let (ox, oy) = origin(i);
        for y in 0..h {
            let src = &f[(y * w * 4) as usize..((y + 1) * w * 4) as usize];
            let d = (((oy + y) * tw + ox) * 4) as usize;
            pixels[d..d + src.len()].copy_from_slice(src);
        }
    }
    let mut png = Vec::new();
    {
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&pixels, tw, th, image::ExtendedColorType::Rgba8)
            .map_err(|e| format!("the packed sheet: {e}"))?;
    }
    let [px, py] = fx.recipe.pivot.unwrap_or([0.5, 0.5]);
    let clip = rel.rsplit('/').next().unwrap_or(rel).to_string();
    let sheet = FlipbookSheet {
        texture: super::animated::sheet_texture_id(rel),
        size: [tw, th],
        filter: if r.smooth {
            SheetFilter::Linear
        } else {
            SheetFilter::Nearest
        },
        array: false,
        frames: (0..n)
            .map(|i| {
                let (x, y) = origin(i);
                SheetFrame {
                    name: format!("{clip}_{i}"),
                    rect: [x, y, w, h],
                    pivot: [px as f32, (1.0 - py) as f32],
                }
            })
            .collect(),
        clips: vec![FlipClip {
            name: clip,
            frames: (0..n as u32).collect(),
            durations: vec![fx.duration.map_or(1.0 / r.fps as f32, |d| d / n as f32); n],
            mode: if looping {
                FlipMode::Loop
            } else {
                FlipMode::Once
            },
            events,
        }],
    };
    sheet.validate_native().map_err(|e| e.to_string())?;
    Ok(Built {
        sheet,
        png,
        rendered: r,
        impact,
    })
}

/// Materialize the checked recipe and its actual #sheet texture once.
pub(crate) fn prepare(s: &mut Source) -> Result<Source, String> {
    let built = build(&s.rel, &s.input::<Value>()?)?;
    let settings = format!(
        "type import\nversion 1\n---\n# vfx2d renderer {ENGINE_VERSION}\nconform off\ntexture {{\n  kind pixel\n}}\n"
    );
    s.flipbook = Some(built.sheet);
    s.conformed = Some(Arc::new(built.png.clone()));
    Ok(super::flipbook::texture(s, built.png, Some(settings)))
}
