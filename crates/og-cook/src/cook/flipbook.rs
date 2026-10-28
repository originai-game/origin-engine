//! Flipbook sheets: an `.ogt` of `type flipbook` (grid frames, `frame`
//! lines, clips) with an optional `atlas "x.json"` — an Aseprite export
//! (frames with durations, `meta.frameTags` as clips) or a TexturePacker
//! JSON (hash or array; pivots) — into the canonical sheet text
//! (`og_anim::FlipbookSheet::to_text`) with the texture's size filled in.
//!
//! A sheet with `array true` also cooks into a texture array: its frames,
//! cut out of the texture, are the layers of `<id>#array` (an
//! `og.TextureArray` whose layers are the textures `<id>#array#layer<n>`,
//! pixel art under `filter nearest`, colour otherwise), so the runtime
//! draws frame `n` as layer `n`. Every frame must then have one size, and
//! a sheet has at most [`super::texture_array::MAX_LAYERS`] of them.

use super::{CookError, Source};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::ogt::{Block, Document, Entry, Value};
use std::path::PathBuf;

/// The project's `assets/` directory, from a source's path and asset id.
pub(super) fn assets_root(s: &Source) -> PathBuf {
    let mut p = s.path.clone();
    for _ in s.rel.split('/') {
        p.pop();
    }
    p
}

/// A path the sheet names: relative to the sheet's directory when that file
/// exists (`hero.png`, `../art/hero.png`), else an asset id
/// (`sprites/hero.png`).
pub(super) fn resolve(s: &Source, path: &str) -> String {
    let id = crate::runtime::asset_id(path);
    let mut parts: Vec<&str> = s.rel.split('/').collect();
    parts.pop();
    for seg in id.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            x => parts.push(x),
        }
    }
    let relative = parts.join("/");
    if id.starts_with("./") || id.starts_with("../") || assets_root(s).join(&relative).is_file() {
        relative
    } else {
        id.to_string()
    }
}

fn string_of(doc: &Document, key: &str) -> Option<String> {
    doc.body
        .get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// The hashes of what the sheet depends on (its texture and atlas files),
/// for the cook stamp: editing either recooks the sheet.
pub(super) fn dependency_hashes(s: &Source) -> String {
    let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    let root = assets_root(s);
    ["texture", "atlas"]
        .iter()
        .filter_map(|k| string_of(&doc, k))
        .map(|p| {
            let id = resolve(s, &p);
            let h = std::fs::read(root.join(&id))
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{id}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The bytes of the texture `id` the sheet names: the image the art
/// contract conformed it to (`conform/texture2d.rs`, whose cells the
/// sheet's text then follows), else the file.
fn texture_bytes(s: &Source, id: &str) -> Result<Vec<u8>, String> {
    match &s.conformed {
        Some(b) => Ok(b.to_vec()),
        None => std::fs::read(assets_root(s).join(id))
            .map_err(|_| format!("texture `{id}` is not in assets/")),
    }
}

/// Splits a name into text and number runs for a natural order ("run 2" before "run 10").
fn natural(name: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut num: Option<u64> = None;
    for ch in name.chars() {
        if let Some(d) = ch.to_digit(10) {
            num = Some(
                num.unwrap_or(0)
                    .saturating_mul(10)
                    .saturating_add(u64::from(d)),
            );
        } else {
            if let Some(n) = num.take() {
                out.push((std::mem::take(&mut text), n));
            }
            text.push(ch);
        }
    }
    out.push((text, num.unwrap_or(0)));
    out
}

struct AtlasFrame {
    name: String,
    rect: [u32; 4],
    duration: Option<f32>,
    pivot: Option<[f32; 2]>,
}

fn json_u32(v: &serde_json::Value, k: &str) -> Result<u32, String> {
    v[k].as_f64()
        .filter(|x| *x >= 0.0 && x.fract() == 0.0)
        .map(|x| x as u32)
        .ok_or_else(|| format!("`{k}` is not a whole number"))
}

/// Frames and tags of an Aseprite or TexturePacker JSON, in sheet order.
fn read_atlas(text: &str) -> Result<(Vec<AtlasFrame>, Vec<Entry>), String> {
    let json: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let frame_of = |name: String, v: &serde_json::Value| -> Result<AtlasFrame, String> {
        let f = &v["frame"];
        let rect = [
            json_u32(f, "x")?,
            json_u32(f, "y")?,
            json_u32(f, "w")?,
            json_u32(f, "h")?,
        ];
        if v["rotated"].as_bool() == Some(true) {
            return Err(format!(
                "frame `{name}` is rotated in the atlas; export without rotation"
            ));
        }
        // TexturePacker pivots run from the top left; sheets from the bottom left.
        let pivot = v["pivot"].as_object().and_then(|p| {
            Some([
                p.get("x")?.as_f64()? as f32,
                1.0 - p.get("y")?.as_f64()? as f32,
            ])
        });
        Ok(AtlasFrame {
            name,
            rect,
            duration: v["duration"].as_f64().map(|ms| (ms / 1000.0) as f32),
            pivot,
        })
    };
    let mut frames = Vec::new();
    match &json["frames"] {
        serde_json::Value::Array(list) => {
            for (i, v) in list.iter().enumerate() {
                let name = v["filename"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| i.to_string());
                frames.push(frame_of(name, v).map_err(|e| format!("frames[{i}]: {e}"))?);
            }
        }
        serde_json::Value::Object(map) => {
            // A hash: in the natural order of its names (Aseprite numbers them).
            let mut names: Vec<&String> = map.keys().collect();
            names.sort_by_key(|n| natural(n));
            for n in names {
                frames.push(frame_of(n.clone(), &map[n]).map_err(|e| format!("frame `{n}`: {e}"))?);
            }
        }
        _ => return Err("no `frames` (an Aseprite or TexturePacker JSON export)".into()),
    }
    // Aseprite tags become clips with the frames' own durations.
    let mut clips = Vec::new();
    for t in json["meta"]["frameTags"].as_array().into_iter().flatten() {
        let name = t["name"].as_str().ok_or("a frame tag without a name")?;
        let (from, to) = (json_u32(t, "from")?, json_u32(t, "to")?);
        if from > to || to as usize >= frames.len() {
            return Err(format!(
                "tag `{name}` runs {from}..{to} of {} frames",
                frames.len()
            ));
        }
        let mut idx: Vec<u32> = (from..=to).collect();
        let mode = match t["direction"].as_str().unwrap_or("forward") {
            "forward" => "loop",
            "reverse" => {
                idx.reverse();
                "loop"
            }
            "pingpong" | "pingpong_reverse" => "pingPong",
            d => return Err(format!("tag `{name}`: direction `{d}`")),
        };
        let mut block = Block::new();
        block.push(idx.iter().fold(Entry::new("frames"), |e, i| {
            e.with(Value::String(frames[*i as usize].name.clone()))
        }));
        let durations = idx
            .iter()
            .map(|i| frames[*i as usize].duration.unwrap_or(0.1));
        block.push(durations.fold(Entry::new("durations"), |e, d| {
            e.with(Value::Number(
                og_core::ogt::Number::from_f32(d).expect("finite"),
            ))
        }));
        block.push(Entry::new("mode").with(Value::Word(mode.into())));
        clips.push(
            Entry::new("clip")
                .with(Value::String(name.into()))
                .with_block(block),
        );
    }
    Ok((frames, clips))
}

/// The canonical sheet of a flipbook source.
pub(super) fn sheet(s: &Source) -> Result<og_anim::FlipbookSheet, String> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| "a flipbook sheet is text (UTF-8)".to_string())?;
    let mut doc = Document::parse(text).map_err(|e| e.reason().to_string())?;
    let root = assets_root(s);
    let texture = string_of(&doc, "texture").ok_or("a flipbook sheet needs `texture \"path\"`")?;
    let texture_id = resolve(s, &texture);
    let image = texture_bytes(s, &texture_id)?;
    let (w, h) = image::load_from_memory(&image)
        .map(|i| (i.width(), i.height()))
        .map_err(|e| format!("texture `{texture_id}`: {e}"))?;
    // The resolved texture id and its size.
    doc.body
        .entries
        .retain(|e| e.key != "texture" && e.key != "size" && e.key != "conform");
    doc.body.entries.insert(
        0,
        Entry::new("size")
            .with(Value::Number(
                og_core::ogt::Number::from_f64(f64::from(w)).expect("finite"),
            ))
            .with(Value::Number(
                og_core::ogt::Number::from_f64(f64::from(h)).expect("finite"),
            )),
    );
    doc.body
        .entries
        .insert(0, Entry::new("texture").with(Value::String(texture_id)));
    if let Some(atlas) = string_of(&doc, "atlas") {
        let id = resolve(s, &atlas);
        let text = std::fs::read_to_string(root.join(&id))
            .map_err(|_| format!("atlas `{id}` is not in assets/"))?;
        let (frames, tags) = read_atlas(&text).map_err(|e| format!("atlas `{id}`: {e}"))?;
        let pivot = doc.body.get("pivot").cloned();
        doc.body.entries.retain(|e| e.key != "atlas");
        let at = doc
            .body
            .entries
            .iter()
            .position(|e| e.key == "clip")
            .unwrap_or(doc.body.entries.len());
        let mut lines = Vec::new();
        for f in &frames {
            let num = |v: f64| Value::Number(og_core::ogt::Number::from_f64(v).expect("finite"));
            let mut e = Entry::new("frame").with(Value::String(f.name.clone()));
            for v in f.rect {
                e = e.with(num(f64::from(v)));
            }
            let p = f.pivot.or_else(|| {
                pivot
                    .as_ref()
                    .and_then(|p| Some([p.f32(0).ok()?, p.f32(1).ok()?]))
            });
            if let Some([x, y]) = p {
                e = e.with(num(f64::from(x))).with(num(f64::from(y)));
            }
            lines.push(e);
        }
        // A clip the sheet declares itself wins over a tag of the same name.
        let own: Vec<String> = doc
            .body
            .get_all("clip")
            .filter_map(|e| {
                e.values
                    .first()
                    .and_then(|v| v.as_word().or(v.as_str()))
                    .map(str::to_string)
            })
            .collect();
        for t in tags {
            let name = t
                .values
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !own.contains(&name) {
                lines.push(t);
            }
        }
        doc.body.entries.splice(at..at, lines);
    }
    let canonical = doc.to_text().map_err(|e| e.reason().to_string())?;
    og_anim::FlipbookSheet::parse(&canonical).map_err(|e| e.reason().to_string())
}

/// Cooks a flipbook sheet blob.
pub(super) fn cook(
    s: &Source,
    motion: Option<&super::conform::mesh::Motion>,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let mut sheet = sheet(s).map_err(CookError)?;
    // The art contract's motion (docs/art.md §2.3, `conform::mesh`).
    let conformed = motion.map(|m| m.flipbook(&mut sheet)).unwrap_or_default();
    let text = sheet.to_text();
    let note = format!(
        "{} frames, {} clips ({}) over {} ({}x{})",
        sheet.frames.len(),
        sheet.clips.len(),
        sheet
            .clips
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        sheet.texture,
        sheet.size[0],
        sheet.size[1]
    );
    let mut w = BlobWriter::new(
        format::Flipbook::type_hash(),
        format::Flipbook::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[FlipbookMeta {
            text_bytes: text.len() as u32,
        }],
        row_hash::<FlipbookMeta>(),
    )
    .raw("text", text.into_bytes(), 4);
    let mut notes = vec![note];
    notes.extend(conformed);
    Ok((w.finish(), notes))
}

/// The texture array output of an `array true` sheet.
pub(super) fn array_id(rel: &str) -> String {
    format!("{rel}#array")
}

/// The texture kind an `array true` sheet's layers cook as.
fn array_kind(sheet: &og_anim::FlipbookSheet) -> u32 {
    match sheet.filter {
        og_anim::SheetFilter::Nearest => format::kind::PIXEL,
        og_anim::SheetFilter::Linear => format::kind::COLOR,
    }
}

/// Checks that an `array true` sheet's frames can be an array's layers:
/// at least one, at most the array limit, all of one size, inside the
/// texture.
pub(super) fn check_array(sheet: &og_anim::FlipbookSheet) -> Result<(), String> {
    let max = super::texture_array::MAX_LAYERS;
    let Some(first) = sheet.frames.first() else {
        return Err("an `array true` sheet needs frames".into());
    };
    if sheet.frames.len() > max {
        return Err(format!(
            "an `array true` sheet has at most {max} frames (the array's layers), not {}",
            sheet.frames.len()
        ));
    }
    for f in &sheet.frames {
        let [x, y, w, h] = f.rect;
        if [w, h] != [first.rect[2], first.rect[3]] {
            return Err(format!(
                "frame `{}` is {w}x{h} and frame `{}` {}x{}: an `array true` sheet's frames are one size",
                f.name, first.name, first.rect[2], first.rect[3]
            ));
        }
        if w == 0 || h == 0 || x + w > sheet.size[0] || y + h > sheet.size[1] {
            return Err(format!(
                "frame `{}` ({x} {y} {w} {h}) is not inside the {}x{} texture",
                f.name, sheet.size[0], sheet.size[1]
            ));
        }
    }
    Ok(())
}

/// The texture array blob of an `array true` sheet.
pub(super) fn cook_array(
    s: &Source,
    sheet: &og_anim::FlipbookSheet,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    check_array(sheet).map_err(CookError)?;
    let [_, _, w, h] = sheet.frames[0].rect;
    let blob = super::texture_array::array_blob(
        &array_id(&s.rel),
        (w, h),
        sheet.frames.len(),
        array_kind(sheet),
        opts,
        stamp,
    );
    Ok((
        blob,
        vec![format!(
            "texture array: {} frames of {w}x{h} cut from {}",
            sheet.frames.len(),
            sheet.texture
        )],
    ))
}

/// Layer `n` of an `array true` sheet: frame `n` cut out of the texture.
pub(super) fn cook_layer(
    s: &Source,
    sheet: &og_anim::FlipbookSheet,
    n: usize,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    check_array(sheet).map_err(CookError)?;
    let bytes = texture_bytes(s, &sheet.texture).map_err(CookError)?;
    let img = super::texture::decode(&bytes)
        .map_err(|e| CookError(format!("texture `{}`: {e}", sheet.texture)))?;
    let [x, y, w, h] = sheet.frames[n].rect;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for row in y..y + h {
        let start = ((row * img.width + x) * 4) as usize;
        rgba.extend_from_slice(&img.rgba[start..start + (w * 4) as usize]);
    }
    let frame = super::texture::Image {
        width: w,
        height: h,
        rgba,
    };
    super::texture::cook(&frame, array_kind(sheet), opts, stamp)
}
