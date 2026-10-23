//! The look cook step: a `type look` .ogt (grammar: [`crate::look`]) is
//! resolved, checked and flattened here, so its blob is the whole look.
//!
//! - `extends "<slug>"` names the game's own `assets/looks/<slug>.ogt`
//!   when it has one (a look of that path never extends itself: there the
//!   slug is the built-in preset), else the built-in preset `<slug>`
//!   ([`PRESETS`], the engine's `templates/looks/<slug>.ogt`); `extends
//!   "looks/base.ogt"` (a path) names another look under `assets/`. Bases
//!   chain up to [`MAX_EXTENDS`] deep; a cycle is refused. The look is laid
//!   over its base field by field ([`LookFile::over`]).
//! - Every field is checked against the IDL's records (`og_api::host::RECORDS`,
//!   the reflection every tool reads): the resource and field exist and are
//!   writable, a number is in its range, a variant is one of the enum's, a
//!   colour is three linear numbers or an sRGB `#rrggbb` (written linear), a
//!   handle field names an asset of its kind that the bundle has.
//! - A handle field may name one of the engine's built-in assets
//!   ([`BUILTINS`], `templates/looks/assets/`) as `"@canvas/<name>"` or
//!   `"@palettes/<name>"` (the extension may be left out): the look stores
//!   its bundle id (`@canvas/paper.png`), and the cook adds the built-in to
//!   the game's bundle under that id ([`builtin_references`]), so the
//!   runtime loads it as any other asset. Paths under `assets/` starting
//!   with `@` are the engine's, and refused as a game's own.
//! - The stamp covers every look in the chain, so editing a base re-cooks
//!   the looks built on it.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use crate::look::{LookFile, MAX_EXTENDS, hex_to_linear};
use og_blob::BlobWriter;
use og_core::ogt::{Number, Value};
use std::path::Path;

/// The built-in presets `extends` names by slug: the engine's
/// `templates/looks/<slug>.ogt`, compiled into the cook: one per style of
/// the knowledge package's catalogue (`packages/knowledge/looks/catalogue.json`),
/// shown by fx-showcase's `looks` station with a reference frame each.
pub const PRESETS: &[(&str, &str)] = &[
    (
        "16bit-pixel",
        include_str!("../../../../templates/looks/16bit-pixel.ogt"),
    ),
    (
        "8bit-pixel",
        include_str!("../../../../templates/looks/8bit-pixel.ogt"),
    ),
    (
        "anime-cel-3d",
        include_str!("../../../../templates/looks/anime-cel-3d.ogt"),
    ),
    (
        "black-figure-pottery",
        include_str!("../../../../templates/looks/black-figure-pottery.ogt"),
    ),
    (
        "bold-comic-illustration",
        include_str!("../../../../templates/looks/bold-comic-illustration.ogt"),
    ),
    (
        "cinematic-realism",
        include_str!("../../../../templates/looks/cinematic-realism.ogt"),
    ),
    (
        "classical-painting-collage",
        include_str!("../../../../templates/looks/classical-painting-collage.ogt"),
    ),
    (
        "coarse-pixel",
        include_str!("../../../../templates/looks/coarse-pixel.ogt"),
    ),
    (
        "decorative-storybook",
        include_str!("../../../../templates/looks/decorative-storybook.ogt"),
    ),
    (
        "detailed-doodle",
        include_str!("../../../../templates/looks/detailed-doodle.ogt"),
    ),
    (
        "detailed-pixel",
        include_str!("../../../../templates/looks/detailed-pixel.ogt"),
    ),
    (
        "expressionist-painting",
        include_str!("../../../../templates/looks/expressionist-painting.ogt"),
    ),
    (
        "faceted-low-poly",
        include_str!("../../../../templates/looks/faceted-low-poly.ogt"),
    ),
    (
        "gothic-contour",
        include_str!("../../../../templates/looks/gothic-contour.ogt"),
    ),
    (
        "heavy-ink-shadows",
        include_str!("../../../../templates/looks/heavy-ink-shadows.ogt"),
    ),
    (
        "illuminated-manuscript",
        include_str!("../../../../templates/looks/illuminated-manuscript.ogt"),
    ),
    (
        "inked-comic-3d",
        include_str!("../../../../templates/looks/inked-comic-3d.ogt"),
    ),
    (
        "layered-silhouette",
        include_str!("../../../../templates/looks/layered-silhouette.ogt"),
    ),
    (
        "low-resolution-3d",
        include_str!("../../../../templates/looks/low-resolution-3d.ogt"),
    ),
    (
        "luminous-layered-painting",
        include_str!("../../../../templates/looks/luminous-layered-painting.ogt"),
    ),
    (
        "luminous-pixel",
        include_str!("../../../../templates/looks/luminous-pixel.ogt"),
    ),
    (
        "lush-painted-worlds",
        include_str!("../../../../templates/looks/lush-painted-worlds.ogt"),
    ),
    (
        "miniature-diorama",
        include_str!("../../../../templates/looks/miniature-diorama.ogt"),
    ),
    (
        "minimal-geometric",
        include_str!("../../../../templates/looks/minimal-geometric.ogt"),
    ),
    (
        "monochrome-dither-3d",
        include_str!("../../../../templates/looks/monochrome-dither-3d.ogt"),
    ),
    (
        "monochrome-pixel",
        include_str!("../../../../templates/looks/monochrome-pixel.ogt"),
    ),
    (
        "muted-cinematic-3d",
        include_str!("../../../../templates/looks/muted-cinematic-3d.ogt"),
    ),
    (
        "neon-vector",
        include_str!("../../../../templates/looks/neon-vector.ogt"),
    ),
    (
        "painterly-3d",
        include_str!("../../../../templates/looks/painterly-3d.ogt"),
    ),
    (
        "paper-craft",
        include_str!("../../../../templates/looks/paper-craft.ogt"),
    ),
    (
        "pencil-hatching",
        include_str!("../../../../templates/looks/pencil-hatching.ogt"),
    ),
    (
        "pixel-3d-hybrid",
        include_str!("../../../../templates/looks/pixel-3d-hybrid.ogt"),
    ),
    (
        "rubber-hose-animation",
        include_str!("../../../../templates/looks/rubber-hose-animation.ogt"),
    ),
    (
        "sculpted-clay",
        include_str!("../../../../templates/looks/sculpted-clay.ogt"),
    ),
    (
        "topdown-tile-pixel",
        include_str!("../../../../templates/looks/topdown-tile-pixel.ogt"),
    ),
    (
        "voxel-worlds",
        include_str!("../../../../templates/looks/voxel-worlds.ogt"),
    ),
    (
        "watercolor-wash",
        include_str!("../../../../templates/looks/watercolor-wash.ogt"),
    ),
];

/// The engine's built-in look assets, by bundle id: the canvas textures a
/// look names in `Stylise.paintCanvas` and the palettes it names in
/// `Stylise.palette`, made by `templates/looks/assets/generate.ts` (CC0)
/// and compiled into the cook. A look writes `"@canvas/paper"` or
/// `"@canvas/paper.png"`; a game that names one has it cooked into its
/// bundle under the id, where `og.load.texture`/`og.load.palette` find it
/// too.
pub const BUILTINS: &[(&str, &[u8])] = &[
    (
        "@canvas/linen.png",
        include_bytes!("../../../../templates/looks/assets/canvas/linen.png"),
    ),
    (
        "@canvas/paper.png",
        include_bytes!("../../../../templates/looks/assets/canvas/paper.png"),
    ),
    (
        "@canvas/watercolor.png",
        include_bytes!("../../../../templates/looks/assets/canvas/watercolor.png"),
    ),
    (
        "@palettes/1bit.hex",
        include_bytes!("../../../../templates/looks/assets/palettes/1bit.hex"),
    ),
    (
        "@palettes/bright32.hex",
        include_bytes!("../../../../templates/looks/assets/palettes/bright32.hex"),
    ),
    (
        "@palettes/dusk16.hex",
        include_bytes!("../../../../templates/looks/assets/palettes/dusk16.hex"),
    ),
    (
        "@palettes/handheld4.hex",
        include_bytes!("../../../../templates/looks/assets/palettes/handheld4.hex"),
    ),
    (
        "@palettes/lcd2.hex",
        include_bytes!("../../../../templates/looks/assets/palettes/lcd2.hex"),
    ),
];

/// The engine's built-in UI styles, by bundle id: every file of
/// `templates/styles/<name>/` (its `style.ogt`, atlas, icons, fonts and
/// their import settings), compiled into the cook. A look names one by its
/// slug (`ui "pixel"`); a game that names one has the whole pack cooked into
/// its bundle under `@styles/<name>/`, where `og.load.style` finds it too
/// (`@styles/pixel/style.ogt`). A game's own `assets/styles/<slug>/style.ogt`
/// wins over the built-in of that slug.
pub const STYLES: &[(&str, &[u8])] = &[
    (
        "@styles/cartoon/Fredoka-SemiBold.ttf",
        include_bytes!("../../../../templates/styles/cartoon/Fredoka-SemiBold.ttf"),
    ),
    (
        "@styles/cartoon/Fredoka-SemiBold.ttf.ogt",
        include_bytes!("../../../../templates/styles/cartoon/Fredoka-SemiBold.ttf.ogt"),
    ),
    (
        "@styles/cartoon/LilitaOne-Regular.ttf",
        include_bytes!("../../../../templates/styles/cartoon/LilitaOne-Regular.ttf"),
    ),
    (
        "@styles/cartoon/LilitaOne-Regular.ttf.ogt",
        include_bytes!("../../../../templates/styles/cartoon/LilitaOne-Regular.ttf.ogt"),
    ),
    (
        "@styles/cartoon/atlas.png",
        include_bytes!("../../../../templates/styles/cartoon/atlas.png"),
    ),
    (
        "@styles/cartoon/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/cartoon/atlas.png.ogt"),
    ),
    (
        "@styles/cartoon/icons.png",
        include_bytes!("../../../../templates/styles/cartoon/icons.png"),
    ),
    (
        "@styles/cartoon/icons.png.ogt",
        include_bytes!("../../../../templates/styles/cartoon/icons.png.ogt"),
    ),
    (
        "@styles/cartoon/style.ogt",
        include_bytes!("../../../../templates/styles/cartoon/style.ogt"),
    ),
    (
        "@styles/fantasy/Cinzel-Bold.ttf",
        include_bytes!("../../../../templates/styles/fantasy/Cinzel-Bold.ttf"),
    ),
    (
        "@styles/fantasy/Cinzel-Bold.ttf.ogt",
        include_bytes!("../../../../templates/styles/fantasy/Cinzel-Bold.ttf.ogt"),
    ),
    (
        "@styles/fantasy/Newsreader-Medium.ttf",
        include_bytes!("../../../../templates/styles/fantasy/Newsreader-Medium.ttf"),
    ),
    (
        "@styles/fantasy/Newsreader-Medium.ttf.ogt",
        include_bytes!("../../../../templates/styles/fantasy/Newsreader-Medium.ttf.ogt"),
    ),
    (
        "@styles/fantasy/atlas.png",
        include_bytes!("../../../../templates/styles/fantasy/atlas.png"),
    ),
    (
        "@styles/fantasy/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/fantasy/atlas.png.ogt"),
    ),
    (
        "@styles/fantasy/icons.png",
        include_bytes!("../../../../templates/styles/fantasy/icons.png"),
    ),
    (
        "@styles/fantasy/icons.png.ogt",
        include_bytes!("../../../../templates/styles/fantasy/icons.png.ogt"),
    ),
    (
        "@styles/fantasy/style.ogt",
        include_bytes!("../../../../templates/styles/fantasy/style.ogt"),
    ),
    (
        "@styles/neon/ChakraPetch-BoldItalic.ttf",
        include_bytes!("../../../../templates/styles/neon/ChakraPetch-BoldItalic.ttf"),
    ),
    (
        "@styles/neon/ChakraPetch-BoldItalic.ttf.ogt",
        include_bytes!("../../../../templates/styles/neon/ChakraPetch-BoldItalic.ttf.ogt"),
    ),
    (
        "@styles/neon/Orbitron-Medium.ttf",
        include_bytes!("../../../../templates/styles/neon/Orbitron-Medium.ttf"),
    ),
    (
        "@styles/neon/Orbitron-Medium.ttf.ogt",
        include_bytes!("../../../../templates/styles/neon/Orbitron-Medium.ttf.ogt"),
    ),
    (
        "@styles/neon/Rajdhani-Bold.ttf",
        include_bytes!("../../../../templates/styles/neon/Rajdhani-Bold.ttf"),
    ),
    (
        "@styles/neon/Rajdhani-Bold.ttf.ogt",
        include_bytes!("../../../../templates/styles/neon/Rajdhani-Bold.ttf.ogt"),
    ),
    (
        "@styles/neon/atlas.png",
        include_bytes!("../../../../templates/styles/neon/atlas.png"),
    ),
    (
        "@styles/neon/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/neon/atlas.png.ogt"),
    ),
    (
        "@styles/neon/icons.png",
        include_bytes!("../../../../templates/styles/neon/icons.png"),
    ),
    (
        "@styles/neon/icons.png.ogt",
        include_bytes!("../../../../templates/styles/neon/icons.png.ogt"),
    ),
    (
        "@styles/neon/style.ogt",
        include_bytes!("../../../../templates/styles/neon/style.ogt"),
    ),
    (
        "@styles/paper/InstrumentSans-Regular.ttf",
        include_bytes!("../../../../templates/styles/paper/InstrumentSans-Regular.ttf"),
    ),
    (
        "@styles/paper/InstrumentSans-Regular.ttf.ogt",
        include_bytes!("../../../../templates/styles/paper/InstrumentSans-Regular.ttf.ogt"),
    ),
    (
        "@styles/paper/Newsreader-SemiBold.ttf",
        include_bytes!("../../../../templates/styles/paper/Newsreader-SemiBold.ttf"),
    ),
    (
        "@styles/paper/Newsreader-SemiBold.ttf.ogt",
        include_bytes!("../../../../templates/styles/paper/Newsreader-SemiBold.ttf.ogt"),
    ),
    (
        "@styles/paper/PTMono-Regular.ttf",
        include_bytes!("../../../../templates/styles/paper/PTMono-Regular.ttf"),
    ),
    (
        "@styles/paper/PTMono-Regular.ttf.ogt",
        include_bytes!("../../../../templates/styles/paper/PTMono-Regular.ttf.ogt"),
    ),
    (
        "@styles/paper/atlas.png",
        include_bytes!("../../../../templates/styles/paper/atlas.png"),
    ),
    (
        "@styles/paper/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/paper/atlas.png.ogt"),
    ),
    (
        "@styles/paper/icons.png",
        include_bytes!("../../../../templates/styles/paper/icons.png"),
    ),
    (
        "@styles/paper/icons.png.ogt",
        include_bytes!("../../../../templates/styles/paper/icons.png.ogt"),
    ),
    (
        "@styles/paper/style.ogt",
        include_bytes!("../../../../templates/styles/paper/style.ogt"),
    ),
    (
        "@styles/pixel/DotGothic16-Regular.ttf",
        include_bytes!("../../../../templates/styles/pixel/DotGothic16-Regular.ttf"),
    ),
    (
        "@styles/pixel/DotGothic16-Regular.ttf.ogt",
        include_bytes!("../../../../templates/styles/pixel/DotGothic16-Regular.ttf.ogt"),
    ),
    (
        "@styles/pixel/atlas.png",
        include_bytes!("../../../../templates/styles/pixel/atlas.png"),
    ),
    (
        "@styles/pixel/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/pixel/atlas.png.ogt"),
    ),
    (
        "@styles/pixel/icons.png",
        include_bytes!("../../../../templates/styles/pixel/icons.png"),
    ),
    (
        "@styles/pixel/icons.png.ogt",
        include_bytes!("../../../../templates/styles/pixel/icons.png.ogt"),
    ),
    (
        "@styles/pixel/style.ogt",
        include_bytes!("../../../../templates/styles/pixel/style.ogt"),
    ),
    (
        "@styles/scifi/ChakraPetch-Medium.ttf",
        include_bytes!("../../../../templates/styles/scifi/ChakraPetch-Medium.ttf"),
    ),
    (
        "@styles/scifi/ChakraPetch-Medium.ttf.ogt",
        include_bytes!("../../../../templates/styles/scifi/ChakraPetch-Medium.ttf.ogt"),
    ),
    (
        "@styles/scifi/JetBrainsMono-Regular.ttf",
        include_bytes!("../../../../templates/styles/scifi/JetBrainsMono-Regular.ttf"),
    ),
    (
        "@styles/scifi/JetBrainsMono-Regular.ttf.ogt",
        include_bytes!("../../../../templates/styles/scifi/JetBrainsMono-Regular.ttf.ogt"),
    ),
    (
        "@styles/scifi/Orbitron-Bold.ttf",
        include_bytes!("../../../../templates/styles/scifi/Orbitron-Bold.ttf"),
    ),
    (
        "@styles/scifi/Orbitron-Bold.ttf.ogt",
        include_bytes!("../../../../templates/styles/scifi/Orbitron-Bold.ttf.ogt"),
    ),
    (
        "@styles/scifi/atlas.png",
        include_bytes!("../../../../templates/styles/scifi/atlas.png"),
    ),
    (
        "@styles/scifi/atlas.png.ogt",
        include_bytes!("../../../../templates/styles/scifi/atlas.png.ogt"),
    ),
    (
        "@styles/scifi/icons.png",
        include_bytes!("../../../../templates/styles/scifi/icons.png"),
    ),
    (
        "@styles/scifi/icons.png.ogt",
        include_bytes!("../../../../templates/styles/scifi/icons.png.ogt"),
    ),
    (
        "@styles/scifi/style.ogt",
        include_bytes!("../../../../templates/styles/scifi/style.ogt"),
    ),
];

/// The built-in styles' slugs.
pub fn style_slugs() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = STYLES
        .iter()
        .filter_map(|(id, _)| id.strip_prefix("@styles/")?.strip_suffix("/style.ogt"))
        .collect();
    out.sort();
    out
}

/// What a look's `ui "<name>"` names: an asset id as written (a path or an
/// `.ogt`), else by slug the game's own `styles/<slug>/style.ogt` when
/// `own` answers it exists, else the built-in `@styles/<slug>/style.ogt`.
pub fn resolve_style(name: &str, own: &dyn Fn(&str) -> bool) -> Result<String, String> {
    if name.contains('/') || name.ends_with(".ogt") {
        let id = crate::runtime::asset_id(name);
        if !id.ends_with(".ogt") {
            return Err(format!(
                "ui \"{name}\": a UI style's .ogt (styles/<name>/style.ogt) or a built-in style's slug ({})",
                style_slugs().join(", ")
            ));
        }
        return Ok(id.to_string());
    }
    let mine = format!("styles/{name}/style.ogt");
    if own(&mine) {
        return Ok(mine);
    }
    let builtin = format!("@styles/{name}/style.ogt");
    if STYLES.iter().any(|(id, _)| *id == builtin) {
        return Ok(builtin);
    }
    Err(format!(
        "ui \"{name}\": neither assets/{mine} nor a built-in style ({})",
        style_slugs().join(", ")
    ))
}

/// The files of the built-in style `id` (`@styles/<name>/style.ogt`)
/// names, by bundle id: the whole pack.
pub fn style_files(id: &str) -> Vec<(&'static str, &'static [u8])> {
    let Some(dir) = id
        .strip_suffix("style.ogt")
        .filter(|d| d.starts_with("@styles/"))
    else {
        return Vec::new();
    };
    STYLES
        .iter()
        .filter(|(f, _)| f.starts_with(dir))
        .copied()
        .collect()
}

/// The built-in asset `name` names (`"@canvas/paper"` or its full id
/// `"@canvas/paper.png"`): its bundle id and bytes.
pub fn builtin(name: &str) -> Option<(&'static str, &'static [u8])> {
    BUILTINS
        .iter()
        .find(|(id, _)| *id == name || id.rsplit_once('.').is_some_and(|(stem, _)| stem == name))
        .copied()
}

/// The built-in assets the look `rel` (text `text`) names through its chain
/// of bases, by bundle id with their bytes: the canvases and palettes its
/// fields name, and every file of the built-in style its `ui` names (unless
/// the game has its own of that slug). The cook adds each to the bundle
/// once. A look that does not resolve names none here; its own cook reports
/// why.
pub(super) fn builtin_files(
    rel: &str,
    text: &str,
    root: &Path,
) -> Vec<(&'static str, &'static [u8])> {
    let mut seen = vec![rel.to_string()];
    let mut bases = Vec::new();
    let Ok(look) = flatten(rel, text, root, PRESETS, &mut seen, &mut bases) else {
        return Vec::new();
    };
    let mut out: Vec<(&'static str, &'static [u8])> = Vec::new();
    for b in &look.blocks {
        for (_, v, _) in &b.fields {
            if let [Value::String(p)] = v.as_slice()
                && let Some(f) = builtin(p)
                && !out.contains(&f)
            {
                out.push(f);
            }
        }
    }
    if let Some(ui) = &look.ui
        && let Ok(id) = resolve_style(ui, &|id| root.join(id).is_file())
    {
        out.extend(style_files(&id));
    }
    out
}

/// A look in an `extends` chain: its id (an asset id, or a preset's slug)
/// and its text.
struct Base {
    id: String,
    text: String,
}

fn find_base(
    name: &str,
    from: &str,
    root: &Path,
    presets: &[(&str, &str)],
) -> Result<Base, String> {
    let read = |id: &str| std::fs::read_to_string(root.join(id)).ok();
    if name.contains('/') || name.ends_with(".ogt") {
        let id = crate::runtime::asset_id(name).to_string();
        return read(&id)
            .map(|text| Base {
                id: id.clone(),
                text,
            })
            .ok_or_else(|| format!("extends \"{name}\": there is no assets/{id}"));
    }
    let own = format!("looks/{name}.ogt");
    if own != from
        && let Some(text) = read(&own)
    {
        return Ok(Base { id: own, text });
    }
    if let Some((slug, text)) = presets.iter().find(|(s, _)| *s == name) {
        return Ok(Base {
            id: slug.to_string(),
            text: text.to_string(),
        });
    }
    let names: Vec<&str> = presets.iter().map(|(s, _)| *s).collect();
    Err(format!(
        "extends \"{name}\": neither assets/{own} nor a built-in preset ({})",
        if names.is_empty() {
            "the engine ships none yet".to_string()
        } else {
            names.join(", ")
        }
    ))
}

/// The look `text` (of id `id`) laid over its chain of bases, and the
/// bases it read, nearest first.
fn flatten(
    id: &str,
    text: &str,
    root: &Path,
    presets: &[(&str, &str)],
    seen: &mut Vec<String>,
    bases: &mut Vec<Base>,
) -> Result<LookFile, String> {
    let mut look = LookFile::parse(text).map_err(|e| {
        if seen.len() > 1 {
            format!("{id}: {e}")
        } else {
            e
        }
    })?;
    let Some(name) = look.extends.first().cloned() else {
        return Ok(look);
    };
    if look.extends.len() > 1 {
        return Err(format!(
            "{id}: extends names one look; the chain is built by each look naming its own"
        ));
    }
    if seen.len() > MAX_EXTENDS {
        return Err(format!("extends chains at most {MAX_EXTENDS} looks deep"));
    }
    let base = find_base(&name, id, root, presets)?;
    if seen.contains(&base.id) {
        seen.push(base.id.clone());
        return Err(format!(
            "extends goes round in a circle: {}",
            seen.join(" → ")
        ));
    }
    seen.push(base.id.clone());
    let base_look = flatten(&base.id, &base.text, root, presets, seen, bases)?;
    look.extends.clear();
    let merged = look.over(&base_look, &base.id);
    bases.insert(0, base);
    Ok(merged)
}

/// What a handle column takes: the asset kinds by extension.
fn handle_extensions(column: &str) -> Option<&'static [&'static str]> {
    Some(match column {
        "Texture" => &["png", "jpg", "jpeg"],
        "Lut" => &["cube"],
        "Palette" => &["hex", "gpl"],
        "EnvironmentMap" => &["ogt"],
        _ => return None,
    })
}

fn word(v: &[Value]) -> Option<&str> {
    match v {
        [Value::Word(w)] => Some(w),
        _ => None,
    }
}

fn number(n: f64) -> Value {
    Value::Number(Number::from_f64(n).expect("a finite number"))
}

/// Checks the flattened look against the records and writes colours
/// linear; `known` answers whether the bundle has an asset id.
fn check(look: &mut LookFile, known: &dyn Fn(&str) -> bool) -> Result<(), String> {
    let mut missing: Vec<String> = Vec::new();
    if let Some(name) = &look.ui {
        // The blob names the style by its bundle id, resolved here.
        let id = resolve_style(name, known)?;
        if !known(&id) {
            missing.push(id.clone());
        }
        look.ui = Some(id);
    }
    for block in &mut look.blocks {
        let schema = format!("og.{}", block.resource);
        let record = og_api::host::RECORDS
            .iter()
            .find(|r| r.schema == schema && r.kind == "resource")
            .ok_or_else(|| format!("{}: not a resource of this engine", block.resource))?;
        for (field, values, line) in &mut block.fields {
            let r = &block.resource;
            let at = format!("line {line}: {r}.{field}");
            let decl = record
                .fields
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| {
                    let names: Vec<&str> = record
                        .fields
                        .iter()
                        .filter(|f| !crate::look::ANIMATED_FIELDS.contains(&(r.as_str(), f.name)))
                        .map(|f| f.name)
                        .collect();
                    format!(
                        "{at}: {r} has no field '{field}' (its fields: {})",
                        names.join(", ")
                    )
                })?;
            if decl.readonly {
                return Err(format!("{at} is read only"));
            }
            let range = |n: f64| -> Result<(), String> {
                if decl.min.is_some_and(|m| n < m) || decl.max.is_some_and(|m| n > m) {
                    return Err(format!(
                        "{at}: {n} is outside {} to {}",
                        decl.min.map_or("-∞".to_string(), |m| m.to_string()),
                        decl.max.map_or("∞".to_string(), |m| m.to_string())
                    ));
                }
                Ok(())
            };
            let numbers = |n: usize| -> Result<Vec<f64>, String> {
                let out: Vec<f64> = values
                    .iter()
                    .filter_map(|v| v.as_number().map(Number::as_f64))
                    .filter(|x| x.is_finite())
                    .collect();
                if out.len() != n || values.len() != n {
                    return Err(format!(
                        "{at} takes {n} number{}",
                        if n == 1 { "" } else { "s" }
                    ));
                }
                Ok(out)
            };
            if !decl.variants.is_empty() {
                match word(values) {
                    Some(w) if decl.variants.contains(&w) => {}
                    _ => {
                        return Err(format!("{at} is one of {}", decl.variants.join(", ")));
                    }
                }
                continue;
            }
            if let Some(exts) = handle_extensions(decl.column) {
                let path = match values.as_slice() {
                    [Value::String(s)] if s.starts_with('@') => match builtin(s) {
                        Some((id, _)) => id.to_string(),
                        None => {
                            let kinds: Vec<&str> = BUILTINS
                                .iter()
                                .map(|(id, _)| *id)
                                .filter(|id| {
                                    id.rsplit_once('.').is_some_and(|(_, e)| exts.contains(&e))
                                })
                                .map(|id| id.rsplit_once('.').map_or(id, |(stem, _)| stem))
                                .collect();
                            return Err(format!(
                                "{at}: \"{s}\" is not one of the engine's built-in {}s ({})",
                                decl.column,
                                kinds.join(", ")
                            ));
                        }
                    },
                    [Value::String(s)] => crate::runtime::asset_id(s).to_string(),
                    _ => {
                        return Err(format!(
                            "{at} names a {} by its asset id, as a string",
                            decl.column
                        ));
                    }
                };
                let ext = path
                    .rsplit_once('.')
                    .map_or("", |(_, e)| e)
                    .to_ascii_lowercase();
                if !exts.contains(&ext.as_str()) {
                    return Err(format!(
                        "{at}: \"{path}\" is not a {} (.{})",
                        decl.column,
                        exts.join(", .")
                    ));
                }
                if !known(&path) {
                    missing.push(path.clone());
                }
                *values = vec![Value::String(path)];
                continue;
            }
            match decl.column {
                "bool" => {
                    if !matches!(word(values), Some("true" | "false")) {
                        return Err(format!("{at} is true or false"));
                    }
                }
                "f32" => range(numbers(1)?[0])?,
                "u32" | "i32" => {
                    let n = numbers(1)?[0];
                    if n.fract() != 0.0 || (decl.column == "u32" && n < 0.0) {
                        return Err(format!(
                            "{at} is a whole number{}",
                            if decl.column == "u32" { " from 0" } else { "" }
                        ));
                    }
                    range(n)?;
                }
                "Vector2" => {
                    numbers(2)?;
                }
                "Vector3" => {
                    numbers(3)?;
                }
                "Color" => {
                    let rgb = match values.as_slice() {
                        [Value::String(s)] => hex_to_linear(s).map(Vec::from).ok_or_else(|| {
                            format!("{at}: \"{s}\" is not an sRGB #rrggbb colour")
                        })?,
                        _ => numbers(3).map_err(|_| {
                            format!("{at} is three linear numbers or an sRGB \"#rrggbb\"")
                        })?,
                    };
                    *values = rgb.into_iter().map(number).collect();
                }
                other => return Err(format!("{at} ({other}) is not set from a look")),
            }
        }
    }
    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(format!(
            "the look names asset(s) not under assets/: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

/// The flattened, checked look of `s` and the bases it read.
fn resolve(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    presets: &[(&str, &str)],
) -> Result<(LookFile, Vec<Base>), String> {
    let text = std::str::from_utf8(&s.bytes).map_err(|_| "a look is text (UTF-8)".to_string())?;
    let mut seen = vec![s.rel.clone()];
    let mut bases = Vec::new();
    let mut look = flatten(&s.rel, text, root, presets, &mut seen, &mut bases)?;
    check(&mut look, known)?;
    Ok((look, bases))
}

/// The stamp's dependencies: every base's id and text hash, and whether the
/// assets the look names are in the bundle.
pub(super) fn look_dependencies(s: &Source, root: &Path, known: &dyn Fn(&str) -> bool) -> String {
    let text = String::from_utf8_lossy(&s.bytes);
    let mut seen = vec![s.rel.clone()];
    let mut bases = Vec::new();
    let mut out = Vec::new();
    match flatten(&s.rel, &text, root, PRESETS, &mut seen, &mut bases) {
        Ok(look) => {
            let mut refs: Vec<String> = look
                .ui
                .iter()
                .map(|u| resolve_style(u, known).unwrap_or_else(|_| u.clone()))
                .collect();
            for b in &look.blocks {
                for (_, v, _) in &b.fields {
                    if let [Value::String(p)] = v.as_slice() {
                        refs.push(crate::runtime::asset_id(p).to_string());
                    }
                }
            }
            for r in refs {
                let id = builtin(&r).map_or(crate::runtime::asset_id(&r), |(id, _)| id);
                out.push(format!("{r}={}", known(id)));
            }
        }
        Err(e) => out.push(format!("error {e}")),
    }
    for b in bases {
        out.push(format!(
            "{}={:016x}",
            b.id,
            super::hash_bytes(b.text.as_bytes())
        ));
    }
    out.join("\n")
}

/// Cooks one `type look` file.
pub(super) fn cook_look(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    cook_with(s, root, known, PRESETS, stamp)
}

fn cook_with(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    presets: &[(&str, &str)],
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let (look, _) = resolve(s, root, known, presets).map_err(CookError)?;
    let text = look.to_text();
    let mut w = BlobWriter::new(
        format::Look::type_hash(),
        format::Look::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::LookMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::LookMeta>(),
    )
    .raw("text", text.into_bytes(), 16);
    let fields: usize = look.blocks.iter().map(|b| b.fields.len()).sum();
    Ok((
        w.finish(),
        vec![format!(
            "look '{}': {} ({fields} field(s)){}",
            look.name
                .as_deref()
                .unwrap_or(s.rel.trim_end_matches(".ogt")),
            look.resources().join(", "),
            if look.extends.is_empty() {
                String::new()
            } else {
                format!(", over {}", look.extends.join(" → "))
            }
        )],
    ))
}

/// Cooks one `.hex` or `.gpl` palette.
pub(super) fn cook_palette(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a palette is text (UTF-8)".into()))?;
    let colours = crate::look::parse_palette(&s.rel, text).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Palette::type_hash(),
        format::Palette::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::PaletteMeta {
            count: colours.len() as u32,
        }],
        format::row_hash::<format::PaletteMeta>(),
    )
    .raw(
        "rgb",
        colours.iter().flatten().copied().collect::<Vec<u8>>(),
        16,
    );
    Ok((
        w.finish(),
        vec![format!("{} colour palette", colours.len())],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn source(root: &Path, rel: &str) -> Source {
        Source {
            rel: rel.to_string(),
            path: root.join(rel),
            kind: super::super::Kind::Look,
            bytes: std::fs::read(root.join(rel)).unwrap(),
            settings: None,
            hash: 0,
            gltf: None,
            error: None,
            encoded: Default::default(),
        }
    }

    fn project(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "og-look-{}-{}",
            std::process::id(),
            super::super::hash_bytes(format!("{files:?}").as_bytes())
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (rel, text) in files {
            let p = dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        dir
    }

    const HEAD: &str = "type look\nversion 1\n---\n";
    const PRESET: &str = "type look\nversion 1\n---\nname \"Cel\"\nantialias msaa4\nMaterialStyle {\n  toon true\n  outlineWidth 2\n}\nPostEffects {\n  vignette true\n}\n";

    fn cook(root: &Path, rel: &str, known: &[&str]) -> Result<LookFile, String> {
        let s = source(root, rel);
        let known = |id: &str| known.contains(&id);
        resolve(&s, root, &known, &[("cel", PRESET)]).map(|(l, _)| l)
    }

    #[test]
    fn extends_a_preset_and_the_games_own_look_wins() {
        let root = project(&[(
            "looks/main.ogt",
            &format!(
                "{HEAD}extends \"cel\"\nStylise {{\n  posterize 5\n}}\nMaterialStyle {{\n  outlineWidth 3\n}}\n"
            ),
        )]);
        let l = cook(&root, "looks/main.ogt", &[]).unwrap();
        assert_eq!(l.extends, vec!["cel"]);
        assert_eq!(l.name, None);
        assert_eq!(l.antialias.as_deref(), Some("msaa4"));
        let m = l
            .blocks
            .iter()
            .find(|b| b.resource == "MaterialStyle")
            .unwrap();
        assert_eq!(
            m.get("outlineWidth").unwrap()[0]
                .as_number()
                .unwrap()
                .text(),
            "3"
        );
        assert_eq!(m.get("toon").unwrap()[0].as_word(), Some("true"));
        assert_eq!(
            l.resources(),
            vec!["PostEffects", "Stylise", "MaterialStyle"]
        );

        // The game's own looks/cel.ogt overrides the preset of that slug,
        // and itself extends the preset.
        let root = project(&[
            (
                "looks/cel.ogt",
                &format!("{HEAD}extends \"cel\"\nname \"Our cel\"\n"),
            ),
            ("looks/main.ogt", &format!("{HEAD}extends \"cel\"\n")),
        ]);
        let l = cook(&root, "looks/main.ogt", &[]).unwrap();
        assert_eq!(l.extends, vec!["looks/cel.ogt", "cel"]);
        // A look is not called after its base.
        assert_eq!(l.name, None);
    }

    #[test]
    fn checks_fields_against_the_records() {
        let one = |body: &str, known: &[&str]| {
            let root = project(&[("looks/a.ogt", &format!("{HEAD}{body}"))]);
            cook(&root, "looks/a.ogt", known)
        };
        let l = one("PostEffects {\n  outlineColor \"#ffffff\"\n  aoMethod gtao\n}\nStylise {\n  palette \"palettes/p.hex\"\n  dither bayer8\n}\n", &["palettes/p.hex"]).unwrap();
        let p = l
            .blocks
            .iter()
            .find(|b| b.resource == "PostEffects")
            .unwrap();
        assert_eq!(p.get("outlineColor").unwrap().len(), 3);
        assert!(
            one("Stylise {\n  posterise 4\n}\n", &[])
                .unwrap_err()
                .contains("Stylise has no field 'posterise'")
        );
        assert!(
            one("Stylise {\n  posterize 100\n}\n", &[])
                .unwrap_err()
                .contains("outside 0 to 64")
        );
        assert!(
            one("Stylise {\n  posterize 2.5\n}\n", &[])
                .unwrap_err()
                .contains("whole number")
        );
        assert!(
            one("Stylise {\n  dither bayer3\n}\n", &[])
                .unwrap_err()
                .contains("one of none, bayer2")
        );
        assert!(
            one("Stylise {\n  palette \"palettes/p.png\"\n}\n", &[])
                .unwrap_err()
                .contains("not a Palette")
        );
        assert!(
            one("Stylise {\n  palette \"palettes/p.hex\"\n}\n", &[])
                .unwrap_err()
                .contains("not under assets/: palettes/p.hex")
        );
        assert!(
            one("PostEffects {\n  outline 1\n}\n", &[])
                .unwrap_err()
                .contains("true or false")
        );
        assert!(
            one("Fog {\n  color \"#12\"\n}\n", &[])
                .unwrap_err()
                .contains("not an sRGB")
        );
        // `ui`: an asset id the bundle must have; a slug names the game's
        // own style first, else a built-in; neither names both.
        assert!(
            one("ui \"styles/x/style.ogt\"\n", &[])
                .unwrap_err()
                .contains("styles/x/style.ogt")
        );
        let l = one("ui \"pixel\"\n", &["@styles/pixel/style.ogt"]).unwrap();
        assert_eq!(l.ui.as_deref(), Some("@styles/pixel/style.ogt"));
        let l = one(
            "ui \"pixel\"\n",
            &["styles/pixel/style.ogt", "@styles/pixel/style.ogt"],
        )
        .unwrap();
        assert_eq!(
            l.ui.as_deref(),
            Some("styles/pixel/style.ogt"),
            "the game's own wins"
        );
        let e = one("ui \"velvet\"\n", &[]).unwrap_err();
        assert!(e.contains("velvet") && e.contains("pixel, scifi"), "{e}");
        assert!(
            one("skin \"skins/x/skin.ogt\"\n", &[])
                .unwrap_err()
                .contains("'skin' is now 'ui'")
        );
    }

    #[test]
    fn refuses_a_missing_base_and_a_cycle() {
        let root = project(&[
            ("looks/a.ogt", &format!("{HEAD}extends \"looks/b.ogt\"\n")),
            ("looks/b.ogt", &format!("{HEAD}extends \"a\"\n")),
            ("looks/c.ogt", &format!("{HEAD}extends \"nowhere\"\n")),
        ]);
        assert!(
            cook(&root, "looks/a.ogt", &[])
                .unwrap_err()
                .contains("circle")
        );
        let e = cook(&root, "looks/c.ogt", &[]).unwrap_err();
        assert!(
            e.contains("neither assets/looks/nowhere.ogt nor a built-in preset (cel)"),
            "{e}"
        );
    }

    #[test]
    fn the_presets_are_the_template_looks() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks");
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().to_str().map(str::to_string))
                    .filter_map(|n| n.strip_suffix(".ogt").map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        on_disk.sort();
        let mut built_in: Vec<String> = PRESETS.iter().map(|(s, _)| s.to_string()).collect();
        built_in.sort();
        assert_eq!(built_in, on_disk, "PRESETS lists templates/looks/*.ogt");
        for (slug, text) in PRESETS {
            LookFile::parse(text).unwrap_or_else(|e| panic!("{slug}: {e}"));
        }
    }
    #[test]
    fn names_the_built_in_assets_by_their_reserved_ids() {
        let one = |body: &str, known: &[&str]| {
            let root = project(&[("looks/b.ogt", &format!("{HEAD}{body}"))]);
            cook(&root, "looks/b.ogt", known)
        };
        let l = one(
            "Stylise {\n  paintCanvas \"@canvas/paper\"\n  palette \"@palettes/handheld4.hex\"\n}\n",
            &["@canvas/paper.png", "@palettes/handheld4.hex"],
        )
        .unwrap();
        let st = l.blocks.iter().find(|b| b.resource == "Stylise").unwrap();
        assert_eq!(
            st.get("paintCanvas").unwrap()[0].as_str(),
            Some("@canvas/paper.png")
        );
        assert_eq!(
            st.get("palette").unwrap()[0].as_str(),
            Some("@palettes/handheld4.hex")
        );
        let e = one("Stylise {\n  paintCanvas \"@canvas/burlap\"\n}\n", &[]).unwrap_err();
        assert!(
            e.contains(
                "\"@canvas/burlap\" is not one of the engine's built-in Textures (@canvas/linen, @canvas/paper, @canvas/watercolor)"
            ),
            "{e}"
        );
        let e = one("Stylise {\n  paintCanvas \"@palettes/1bit\"\n}\n", &[]).unwrap_err();
        assert!(e.contains("is not a Texture"), "{e}");
        // Named but not in the bundle: the cook adds the built-ins a look
        // names before it checks the look, so this is a cook without them.
        let e = one("Stylise {\n  palette \"@palettes/lcd2\"\n}\n", &[]).unwrap_err();
        assert!(e.contains("not under assets/: @palettes/lcd2.hex"), "{e}");
    }

    #[test]
    fn the_built_ins_are_the_template_assets() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks/assets");
        let mut on_disk = Vec::new();
        for kind in ["canvas", "palettes"] {
            for e in std::fs::read_dir(dir.join(kind)).unwrap() {
                let name = e.unwrap().file_name().to_string_lossy().to_string();
                on_disk.push(format!("@{kind}/{name}"));
            }
        }
        on_disk.sort();
        let ids: Vec<String> = BUILTINS.iter().map(|(id, _)| id.to_string()).collect();
        assert_eq!(
            ids, on_disk,
            "BUILTINS lists templates/looks/assets, sorted"
        );
        for (id, bytes) in BUILTINS {
            if id.ends_with(".png") {
                assert!(bytes.len() <= 64 * 1024, "{id}: {} B", bytes.len());
                let img = image::load_from_memory(bytes).unwrap();
                assert_eq!((img.width(), img.height()), (256, 256), "{id}");
            } else {
                let colours = crate::look::parse_palette(id, std::str::from_utf8(bytes).unwrap())
                    .unwrap_or_else(|e| panic!("{id}: {e}"));
                let digits: String = id.chars().filter(char::is_ascii_digit).collect();
                let want: usize = if *id == "@palettes/1bit.hex" {
                    2
                } else {
                    digits.parse().unwrap()
                };
                assert_eq!(colours.len(), want, "{id}");
            }
        }
        assert_eq!(
            builtin("@canvas/paper").map(|b| b.0),
            Some("@canvas/paper.png")
        );
        assert_eq!(
            builtin("@canvas/paper.png").map(|b| b.0),
            Some("@canvas/paper.png")
        );
        assert_eq!(builtin("@canvas/pap"), None);
        assert_eq!(builtin("canvas/paper"), None);
    }

    #[test]
    fn every_preset_checks_against_the_records_with_the_built_ins() {
        let builtin_ids: Vec<&str> = BUILTINS.iter().map(|(id, _)| *id).collect();
        for (slug, _) in PRESETS {
            let root = project(&[("looks/x.ogt", &format!("{HEAD}extends \"{slug}\"\n"))]);
            let s = source(&root, "looks/x.ogt");
            let known = |id: &str| builtin_ids.contains(&id);
            resolve(&s, &root, &known, PRESETS).unwrap_or_else(|e| panic!("{slug}: {e}"));
            let text = String::from_utf8_lossy(&s.bytes).to_string();
            for (id, _) in builtin_files(&s.rel, &text, &root) {
                assert!(builtin_ids.contains(&id), "{slug}: {id}");
            }
        }
    }
}
