//! Pixel-art 2D effects: an `.ogt` of `type vfx2d` whose body is a Pixel
//! VFX Forge recipe (Scenario's JSON, layers keyed by their own `type`),
//! inline or named by `recipe "<path>"`. The cook renders it with the
//! Forge's renderer, ported to Rust (`vfx2d/engine.rs`, the same frames as
//! the Forge), and the effect becomes an ordinary flipbook sheet: its frames
//! packed into a texture of its own, `<id>#sheet` (pixel art, `filter
//! nearest`), and one clip named after the file with the recipe's fps and
//! loop and a FlipbookEvent `impact` on the impact frame. The rest of the
//! cook and the runtime see a sheet.
//!
//! ```text
//! type vfx2d
//! version 1
//! ---
//! duration 0.8                 # seconds the clip lasts (default: frames / fps)
//! loop false                   # the clip's mode: loop or once (default: the recipe's)
//! impact 0.2                   # = event 0.2 "impact" (default: the recipe's impact, else
//!                              #   the frame with the largest jump in coverage in the first 60%)
//! event 0.5 "linger"           # a FlipbookEvent on the frame that starts at or after 0.5 s
//! recipe "slash.json"          # the recipe, relative to this file, or an asset id
//! ```
//!
//! or, instead of `recipe`, the JSON itself from a line starting with `{`
//! to the end of the file. The shared lines (`duration`, `loop`, `event`,
//! `impact`) mean what they mean in a `type vfx` composed effect.

mod color;
mod engine;
mod jsmath;
mod json;
mod layers;

pub use engine::{ENGINE_VERSION, LAYER_TYPES, Rendered};

use super::Source;
use og_core::ogt::{Block, Document, Entry, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The `.ogt` type of a pixel effect.
pub const VFX2D_TYPE: &str = "vfx2d";
/// The event an `impact` line sends.
const IMPACT: &str = "impact";
/// Named events in one effect at most (with the impact).
const MAX_EVENTS: usize = 16;
/// An event name's bytes at most (og.FlipbookEvent.name).
const MAX_EVENT_NAME: usize = 63;

/// Renders a recipe's JSON text: the Forge's frames.
pub fn render_json(text: &str) -> Result<Rendered, String> {
    Ok(engine::render(&json::parse(text)?))
}

/// What a `type vfx2d` file says.
struct Effect {
    recipe: json::Val,
    /// The recipe file's asset id, when it names one.
    file: Option<String>,
    duration: Option<f32>,
    looping: Option<bool>,
    impact: Option<f32>,
    events: Vec<(f32, String)>,
}

fn seconds(e: &Entry) -> Result<f32, String> {
    match e.f32(0) {
        Ok(t) if t >= 0.0 && t.is_finite() && e.values.len() == 1 => Ok(t),
        _ => Err(format!("line {}: {} <seconds>, 0 or more", e.line, e.key)),
    }
}

/// The recipe file `effect` names, without reading it.
fn recipe_line(doc: &Document) -> Result<Option<String>, String> {
    doc.body
        .get("recipe")
        .map(|e| {
            e.string(0)
                .map(str::to_string)
                .map_err(|_| format!("line {}: recipe \"<path>\"", e.line))
        })
        .transpose()
}

/// Splits a vfx2d text into its `.ogt` lines and its inline JSON (from the
/// first body line that starts with `{`).
fn split(text: &str) -> (String, Option<&str>) {
    let mut offset = 0;
    let mut body = false;
    for line in text.split_inclusive('\n') {
        if body && line.trim_start().starts_with('{') {
            return (text[..offset].to_string(), Some(&text[offset..]));
        }
        if line.trim_end() == "---" {
            body = true;
        }
        offset += line.len();
    }
    (text.to_string(), None)
}

fn parse(
    text: &str,
    resolve: &dyn Fn(&str) -> String,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Effect, String> {
    let (lines, inline) = split(text);
    let doc = Document::parse(&lines).map_err(|e| e.to_string())?;
    if doc.ty != VFX2D_TYPE {
        return Err(format!(
            "a pixel effect is `type {VFX2D_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    if doc.version != 1 {
        return Err(format!(
            "vfx2d version {} (this engine reads 1)",
            doc.version
        ));
    }
    let mut fx = Effect {
        recipe: json::Val::Null,
        file: None,
        duration: None,
        looping: None,
        impact: None,
        events: Vec::new(),
    };
    for e in &doc.body.entries {
        let line = e.line;
        match e.key.as_str() {
            "duration" => {
                let d = seconds(e)?;
                if d <= 0.0 {
                    return Err(format!("line {line}: duration is above 0 seconds"));
                }
                fx.duration = Some(d);
            }
            "loop" => {
                fx.looping = Some(match e.word(0) {
                    Ok("true") if e.values.len() == 1 => true,
                    Ok("false") if e.values.len() == 1 => false,
                    _ => return Err(format!("line {line}: loop true | false")),
                })
            }
            "impact" => {
                if fx.impact.is_some() {
                    return Err(format!("line {line}: one impact line at most"));
                }
                fx.impact = Some(seconds(e)?);
            }
            "event" => {
                let (Ok(t), Ok(name)) = (e.f32(0), e.string(1)) else {
                    return Err(format!("line {line}: event <seconds> \"<name>\""));
                };
                if t < 0.0 || !t.is_finite() || e.values.len() != 2 {
                    return Err(format!("line {line}: event <seconds> \"<name>\""));
                }
                if name.is_empty() || name.len() > MAX_EVENT_NAME {
                    return Err(format!(
                        "line {line}: an event's name is 1 to {MAX_EVENT_NAME} bytes"
                    ));
                }
                fx.events.push((t, name.to_string()));
            }
            "recipe" => {}
            k => {
                return Err(format!(
                    "line {line}: unknown line `{k}` (duration, loop, impact, event, recipe, then the recipe's JSON)"
                ));
            }
        }
    }
    let recipe_text = match (recipe_line(&doc)?, inline) {
        (Some(_), Some(_)) => {
            return Err("a `recipe` line and an inline recipe: give one".into());
        }
        (None, None) => {
            return Err(
                "no recipe: `recipe \"<file>.json\"`, or the recipe's JSON from a line starting with `{`"
                    .into(),
            );
        }
        (Some(p), None) => {
            let id = resolve(&p);
            let bytes = read(&id)?;
            fx.file = Some(id.clone());
            String::from_utf8(bytes).map_err(|_| format!("recipe `{id}` is not UTF-8"))?
        }
        (None, Some(j)) => j.to_string(),
    };
    fx.recipe = json::parse(&recipe_text).map_err(|e| match &fx.file {
        Some(id) => format!("{id}: {e}"),
        None => e,
    })?;
    match fx.recipe.get("layers") {
        Some(json::Val::Arr(l)) if !l.is_empty() => {}
        _ => return Err("the recipe has no layers".into()),
    }
    Ok(fx)
}

/// A built pixel effect.
pub struct Built {
    /// The flipbook sheet's text.
    pub text: String,
    /// The packed frames, PNG.
    pub png: Vec<u8>,
    /// The rendered frames.
    pub rendered: Rendered,
    /// The clip's impact frame.
    pub impact: usize,
    /// The recipe file it read, by asset id.
    pub read: Option<String>,
}

/// The clip's frame for a time: the frame that starts at or after it (the
/// last frame for a time inside it).
fn frame_at(t: f32, frames: usize, duration: f32, what: &str) -> Result<usize, String> {
    if t > duration * (1.0 + 1e-6) {
        return Err(format!(
            "{what} at {t} s is after the duration, {duration} s"
        ));
    }
    let f = (f64::from(t) * frames as f64 / f64::from(duration) - 1e-6).ceil();
    Ok((f.max(0.0) as usize).min(frames - 1))
}

/// Builds pixel effect `rel` from its text.
pub fn build(
    rel: &str,
    text: &str,
    resolve: &dyn Fn(&str) -> String,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Built, String> {
    let fx = parse(text, resolve, read)?;
    let r = engine::render(&fx.recipe);
    let n = r.frames.len();
    let (w, h) = (r.w, r.h);
    let looping = fx.looping.unwrap_or(r.looping);
    let duration = fx.duration.unwrap_or(n as f32 / r.fps as f32);
    let impact = match fx.impact {
        Some(t) => frame_at(t, n, duration, "impact")?,
        None => r.impact,
    };
    let mut events = vec![(impact, IMPACT.to_string())];
    for (t, name) in &fx.events {
        events.push((
            frame_at(*t, n, duration, &format!("event \"{name}\""))?,
            name.clone(),
        ));
    }
    if events.len() > MAX_EVENTS {
        return Err(format!("{} events (at most {MAX_EVENTS})", events.len()));
    }
    og_core::sort::sort_by_key(&mut events, |e| e.0);
    // The Forge's sheet: one row when it fits in 4096 texels.
    let cols = if n as u32 * w <= 4096 {
        n as u32
    } else {
        (f64::from(n as u32 * h) / f64::from(w)).sqrt().ceil() as u32
    };
    let rows = (n as u32).div_ceil(cols);
    let (tw, th) = (cols * w, rows * h);
    let mut sheet = vec![0u8; (tw * th * 4) as usize];
    let origin = |i: usize| ((i as u32 % cols) * w, (i as u32 / cols) * h);
    for (i, f) in r.frames.iter().enumerate() {
        let (ox, oy) = origin(i);
        for y in 0..h {
            let src = &f[(y * w * 4) as usize..((y + 1) * w * 4) as usize];
            let d = (((oy + y) * tw + ox) * 4) as usize;
            sheet[d..d + src.len()].copy_from_slice(src);
        }
    }
    let mut png = Vec::new();
    {
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&sheet, tw, th, image::ExtendedColorType::Rgba8)
            .map_err(|e| format!("the packed sheet: {e}"))?;
    }
    // The recipe's pivot is a canvas point from the top left; a sheet's
    // is from the bottom left.
    let pivot = |i: usize, d: f64| match fx.recipe.get("pivot") {
        Some(json::Val::Arr(p)) => p.get(i).map_or(d, json::Val::num),
        _ => d,
    };
    let (px, py) = (pivot(0, 0.5), 1.0 - pivot(1, 0.5));
    let num = |v: f64| Value::Number(og_core::ogt::Number::from_f64(v).expect("finite"));
    let clip = rel
        .rsplit('/')
        .next()
        .unwrap_or(rel)
        .trim_end_matches(".ogt")
        .to_string();
    let mut out = Document::new("flipbook", 1);
    out.body
        .push(Entry::new("texture").with(Value::String(super::animated::sheet_texture_id(rel))));
    out.body
        .push(Entry::new("filter").with(Value::Word("nearest".into())));
    let mut names = Entry::new("frames");
    for i in 0..n {
        let name = format!("{clip}_{i}");
        let (ox, oy) = origin(i);
        let mut e = Entry::new("frame").with(Value::String(name.clone()));
        for v in [ox, oy, w, h] {
            e = e.with(num(f64::from(v)));
        }
        out.body.push(e.with(num(px)).with(num(py)));
        names = names.with(Value::String(name));
    }
    let mut block = Block::new();
    block.push(names);
    match fx.duration {
        Some(d) => {
            let mut durations = Entry::new("durations");
            for _ in 0..n {
                durations = durations.with(num(f64::from(d / n as f32)));
            }
            block.push(durations);
        }
        None => block.push(Entry::new("fps").with(num(f64::from(r.fps)))),
    }
    block.push(Entry::new("mode").with(Value::Word(if looping { "loop" } else { "once" }.into())));
    for (f, name) in &events {
        block.push(
            Entry::new("event")
                .with(num(*f as f64))
                .with(Value::String(name.clone())),
        );
    }
    out.body.push(
        Entry::new("clip")
            .with(Value::String(clip))
            .with_block(block),
    );
    let text = out.to_text().map_err(|e| e.reason().to_string())?;
    Ok(Built {
        text,
        png,
        rendered: r,
        impact,
        read: fx.file,
    })
}

/// Whether an `.ogt`'s text is a pixel effect (its header says so).
pub fn is_vfx2d(text: &str) -> bool {
    Document::parse_header(text).is_ok_and(|d| d.ty == VFX2D_TYPE)
}

/// Expands every pixel effect before the cook hashes anything, as
/// `animated::expand` does a sheet of animated sources: its text becomes
/// the sheet's (the packed frames its texture, `Source::conformed`), the
/// frames a texture source of their own (`<id>#sheet`, pixel art; the
/// renderer's version is part of its import text, so a renderer change
/// cooks it again), and the recipe file it names is consumed.
pub(super) fn expand(sources: &mut Vec<Source>, consumed: &mut BTreeSet<String>) {
    let mut extra = Vec::new();
    for s in sources.iter_mut() {
        if s.kind != super::Kind::Flipbook || s.error.is_some() {
            continue;
        }
        let text = String::from_utf8_lossy(&s.bytes).into_owned();
        if !is_vfx2d(&text) {
            continue;
        }
        let root = super::flipbook::assets_root(s);
        let resolve = |p: &str| super::flipbook::resolve(s, p);
        let read = |id: &str| {
            std::fs::read(root.join(id)).map_err(|_| format!("recipe `{id}` is not in assets/"))
        };
        // The recipe it names is part of it, built or not.
        if let Ok(Some(p)) = Document::parse(&split(&text).0)
            .map_err(|e| e.to_string())
            .and_then(|d| recipe_line(&d))
        {
            consumed.insert(resolve(&p));
        }
        let built = match build(&s.rel, &text, &resolve, &read) {
            Ok(b) => b,
            Err(e) => {
                s.error = Some(e);
                continue;
            }
        };
        let settings = format!(
            "type import\nversion 1\n---\n# vfx2d renderer {ENGINE_VERSION}\ntexture {{\n  kind pixel\n}}\n"
        );
        let doc = Document::parse(&settings).expect("the import text");
        s.bytes = built.text.into_bytes();
        s.conformed = Some(Arc::new(built.png.clone()));
        extra.push(Source {
            rel: super::animated::sheet_texture_id(&s.rel),
            path: s.path.clone(),
            kind: super::Kind::Texture,
            bytes: built.png,
            settings: Some((settings, doc)),
            hash: 0,
            gltf: None,
            error: None,
            encoded: Default::default(),
            whole: Default::default(),
            conformed: None,
            fracture: None,
        });
    }
    sources.extend(extra);
}
