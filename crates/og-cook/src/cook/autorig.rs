//! Auto-rigging layered 2D art: a `type rig2d` file naming `parts` (a
//! layered PSD, or a folder of PNG parts) and a `template` becomes the
//! same canonical rig the DragonBones import and the hand-written text
//! make, with its texture an atlas of the parts packed by the cook
//! (`<rig>#atlas`), so it plays through og.Skeleton2D like any other.
//!
//! ```text
//! type rig2d
//! version 1
//! ---
//! parts "knight.psd"         # or "knight/", a folder of PNGs
//! template humanoid-side    # or humanoid-front, quadruped, creature
//! pivot hand-r 212 340       # optional: a joint, canvas pixels
//! unit 200                   # optional: pixels per world unit
//! filter nearest             # optional: pixel art
//! ```
//!
//! The part list and the layer conventions are the art contract
//! (`docs/art.md` §5): each part is a layer (or a group, merged) named
//! for it, drawn where it sits on the character in its setup pose, and
//! overlapping its parent where they join. The cook reads each part's
//! pixels and alpha ([`parts`]), places each joint at the centroid of the
//! overlap with its parent near the part's joint end (else at that end),
//! builds the template's bones through those joints ([`skeleton`]), and
//! packs the parts into the atlas ([`atlas`]). What does not fit the
//! template is refused, naming the layer, the file and the fix.

use super::Source;
use super::texture::Image;
use og_anim::{Rig2dAsset, SheetFilter};
use og_core::ogt::Document;
use std::path::Path;

mod atlas;
mod clips;
mod creature;
mod front;
mod mesh;
mod parts;
mod quadruped;
mod side;
mod skeleton;
mod template;
#[cfg(test)]
mod tests;

pub(super) use parts::Part;

/// The suffix of the atlas texture a rig with `parts` cooks beside itself.
pub(super) fn atlas_id(rel: &str) -> String {
    format!("{rel}#atlas")
}

/// A path the rig names, relative to its directory when that file or
/// folder exists, else an asset id; a folder keeps its trailing `/`.
fn resolve(rel: &str, root: &Path, path: &str) -> String {
    let dir = path.ends_with('/');
    let id = crate::runtime::asset_id(path.trim_end_matches('/'));
    let mut segs: Vec<&str> = rel.split('/').collect();
    segs.pop();
    for seg in id.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                segs.pop();
            }
            x => segs.push(x),
        }
    }
    let relative = segs.join("/");
    let out = if id.starts_with("./") || id.starts_with("../") || root.join(&relative).exists() {
        relative
    } else {
        id.to_string()
    };
    if dir { format!("{out}/") } else { out }
}

fn string_of(doc: &Document, key: &str) -> Option<String> {
    doc.body
        .get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// The files a rig's `parts` names (the PSD, or each PNG of the folder),
/// as asset ids: part of the rig, not textures of their own.
pub(super) fn part_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(p) = string_of(&doc, "parts") else {
        return Vec::new();
    };
    let id = resolve(rel, root, &p);
    match id.strip_suffix('/') {
        Some(dir) => parts::folder_files(&root.join(dir))
            .into_iter()
            .map(|f| format!("{dir}/{f}"))
            .collect(),
        None => vec![id],
    }
}

/// Whether a rig's text names `parts` (an auto-rig).
pub(super) fn is_autorig(doc: &Document) -> bool {
    doc.body.get("parts").is_some()
}

/// The rig and its atlas from a rig's text with `parts`.
pub(super) fn build(
    s: &Source,
    doc: &Document,
) -> Result<(Rig2dAsset, Image, Vec<String>), String> {
    if let Some(e) = doc.body.entries.iter().find(|e| {
        !matches!(
            e.key.as_str(),
            "parts" | "template" | "pivot" | "unit" | "filter"
        )
    }) {
        return Err(format!(
            "line {}: a rig that names `parts` is made by the cook; `{}` is not one of its lines (parts, template, pivot, unit, filter)",
            e.line, e.key
        ));
    }
    let root = super::flipbook::assets_root(s);
    let path =
        string_of(doc, "parts").ok_or("`parts` names a PSD or a folder: parts \"hero.psd\"")?;
    let id = resolve(&s.rel, &root, &path);
    let template = match doc.body.get("template") {
        None => {
            return Err(format!(
                "a rig with `parts` names its template: `template {}`",
                template::NAMES.join("|")
            ));
        }
        Some(e) => {
            let w = e
                .word(0)
                .map_err(|_| format!("line {}: template <name>", e.line))?;
            template::get(w).ok_or_else(|| {
                format!(
                    "line {}: `{w}` is not a template; one of {}",
                    e.line,
                    template::NAMES.join(", ")
                )
            })?
        }
    };
    let mut notes = Vec::new();
    let art = parts::read(&root, &id, template, &mut notes)?;
    let mut pivots = Vec::new();
    for e in doc.body.entries.iter().filter(|e| e.key == "pivot") {
        let (Ok(name), Ok(x), Ok(y)) = (e.word(0), e.f32(1), e.f32(2)) else {
            return Err(format!(
                "line {}: pivot <part> <x> <y> (canvas pixels)",
                e.line
            ));
        };
        if !art.parts.iter().any(|p| p.name == name) {
            return Err(format!(
                "line {}: pivot for `{name}`, which is not a part of {}; name one of its layers",
                e.line, art.label
            ));
        }
        pivots.push((name.to_string(), [x, y]));
    }
    let packed = atlas::pack(&art.parts).map_err(|e| format!("{}: {e}", art.label))?;
    let mut rig = skeleton::rig(&art, template, &pivots, &packed.rects)?;
    rig.texture = atlas_id(&s.rel);
    rig.size = [packed.image.width, packed.image.height];
    rig.unit = match doc.body.get("unit") {
        Some(e) => e.f32(0).ok().filter(|u| *u > 0.0).ok_or(format!(
            "line {}: unit <pixels per world unit>, above 0",
            e.line
        ))?,
        // The template's height in world units (a humanoid two, a fox
        // 1.2), whatever the art's resolution.
        None => (art.drawn_height() / template.height).max(1.0),
    };
    if doc.body.get("filter").and_then(|e| e.word(0).ok()) == Some("nearest") {
        rig.filter = SheetFilter::Nearest;
    }
    // The template's clip library, scaled to this rig.
    let measure = clips::measure(&rig, template.legs, art.drawn_height());
    rig.clips = match template.name {
        "humanoid-side" => side::clips(&rig, &measure),
        "humanoid-front" => front::clips(&rig, &measure),
        "quadruped" => quadruped::clips(&rig, &measure),
        _ => creature::clips(&rig, &measure),
    };
    notes.push(format!(
        "auto-rigged {} as {}: {} parts, {} clips, a {}x{} atlas",
        art.label,
        template.name,
        art.parts.len(),
        rig.clips.len(),
        packed.image.width,
        packed.image.height
    ));
    Ok((rig, packed.image, notes))
}
