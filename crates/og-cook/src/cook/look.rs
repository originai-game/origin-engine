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
//! - Every field is checked against the IDL's records (`og_records::RECORDS`,
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

pub use super::look_assets::{BUILTINS, PRESETS};
pub use super::look_styles::{StyleFile, style_licences, styles, styles_dir};
use super::{CookError, Source};
use crate::format::{self, BlobKind};
use crate::look::{LookFile, MAX_EXTENDS, hex_to_linear};
use og_blob::BlobWriter;
use og_core::ogt::{Number, Value};
use std::path::Path;

/// The built-in styles' slugs.
pub fn style_slugs() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = styles()
        .iter()
        .filter_map(|f| f.id().strip_prefix("@styles/")?.strip_suffix("/style.ogt"))
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
    if styles().iter().any(|f| f.id() == builtin) {
        return Ok(builtin);
    }
    if styles_dir().is_none() {
        return Err(format!(
            "ui \"{name}\": neither assets/{mine} nor a built-in style: the engine's built-in styles were not found (no styles/ in the @origingame/engine package; OG_STYLES_DIR names another directory)"
        ));
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
    styles()
        .iter()
        .filter(|f| f.id().starts_with(dir))
        .map(StyleFile::pair)
        .collect()
}

/// The shared built-in CJK companions (`@styles/cjk/`, each with its
/// import settings) a style's text names as `cjk` and `cjkDisplay` for the
/// locales in `shown` (each companion is 1 to 3 MB of outlines, so a game
/// carries only those of the locales it shows: [`shown_cjk`]).
pub fn companion_files(
    style_text: &str,
    shown: &[og_ui::CjkLocale],
) -> Vec<(&'static str, &'static [u8])> {
    let Ok(file) = og_ui::parse_style(style_text) else {
        return Vec::new();
    };
    let mut out: Vec<(&'static str, &'static [u8])> = Vec::new();
    for (_, r) in file.companions().iter().filter(|(l, _)| shown.contains(l)) {
        let settings = format!("{r}.ogt");
        for f in styles()
            .iter()
            .filter(|f| f.id() == r || f.id() == settings)
        {
            let f = f.pair();
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// The built-in files of other packs a style's text draws with: its body,
/// display, numeric and fallback fonts and its icon sheet where they name a
/// built-in `@styles/` file (each with its import settings). A generated
/// style (`og style new`) shares the packs' fonts and icons this way; they
/// join the bundle whatever the game's languages.
pub fn shared_files(style_text: &str) -> Vec<(&'static str, &'static [u8])> {
    let Ok(file) = og_ui::parse_style(style_text) else {
        return Vec::new();
    };
    let icons = file.icons.as_ref().map(|i| &i.0);
    let mut out: Vec<(&'static str, &'static [u8])> = Vec::new();
    for r in file.fonts[..4].iter().flatten().chain(icons) {
        let settings = format!("{r}.ogt");
        for f in styles()
            .iter()
            .filter(|f| f.id() == r || f.id() == settings)
        {
            let f = f.pair();
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// The CJK forms the project under `root` (its assets directory) shows:
/// those of each `type strings` table's Chinese, Japanese or Korean
/// `locale` (`zh-TW` traditional, `zh` simplified). A style's companions
/// of a locale join the bundle only then.
pub fn shown_cjk(root: &Path) -> Vec<og_ui::CjkLocale> {
    fn walk(dir: &Path, out: &mut Vec<og_ui::CjkLocale>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
                continue;
            }
            if p.extension().is_none_or(|x| x != "ogt") {
                continue;
            }
            let Ok(t) = std::fs::read_to_string(&p) else {
                continue;
            };
            if !t.lines().any(|l| l.trim() == "type strings") {
                continue;
            }
            for l in t.lines() {
                let Some(tag) = l.trim_start().strip_prefix("locale \"") else {
                    continue;
                };
                let tag = tag.split('"').next().unwrap_or("");
                let primary = tag
                    .split(['-', '_'])
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if matches!(primary.as_str(), "zh" | "ja" | "ko") {
                    let l = og_ui::CjkLocale::of_locale(tag);
                    if !out.contains(&l) {
                        out.push(l);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
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
        let files = style_files(&id);
        let style = files
            .iter()
            .find(|(f, _)| *f == id)
            .map(|(_, b)| String::from_utf8_lossy(b).to_string());
        out.extend(files);
        if let Some(text) = style {
            for f in shared_files(&text) {
                if !out.contains(&f) {
                    out.push(f);
                }
            }
            out.extend(companion_files(&text, &shown_cjk(root)));
        }
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
        "Texture" => &["png", "jpg", "jpeg", "webp", "tga", "psd", "dds"],
        "Lut" => &["cube"],
        "Palette" => &["hex", "gpl"],
        "EnvironmentMap" => &["ogt"],
        "Post" => &["ogt"],
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

/// The asset id a handle field's value names: a string under `assets/`,
/// or one of the engine's built-ins (`"@canvas/paper"`), of the column's
/// kinds by extension.
fn handle_path(at: &str, column: &str, exts: &[&str], values: &[Value]) -> Result<String, String> {
    let path = match values {
        [Value::String(s)] if s.starts_with('@') => match builtin(s) {
            Some((id, _)) => id.to_string(),
            None => {
                let kinds: Vec<&str> = BUILTINS
                    .iter()
                    .map(|(id, _)| *id)
                    .filter(|id| id.rsplit_once('.').is_some_and(|(_, e)| exts.contains(&e)))
                    .map(|id| id.rsplit_once('.').map_or(id, |(stem, _)| stem))
                    .collect();
                return Err(format!(
                    "{at}: \"{s}\" is not one of the engine's built-in {column}s ({})",
                    kinds.join(", ")
                ));
            }
        },
        [Value::String(s)] => crate::runtime::asset_id(s).to_string(),
        _ => {
            return Err(format!(
                "{at} names a {column} by its asset id, as a string"
            ));
        }
    };
    let ext = path
        .rsplit_once('.')
        .map_or("", |(_, e)| e)
        .to_ascii_lowercase();
    if !exts.contains(&ext.as_str()) {
        return Err(format!(
            "{at}: \"{path}\" is not a {column} (.{})",
            exts.join(", .")
        ));
    }
    Ok(path)
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
        let record = og_records::RECORDS
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
            if values.is_empty() && decl.list == 0 {
                return Err(format!("{at} has no value"));
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
            if decl.list > 0 {
                // A list on its one line: up to its capacity of numbers, or
                // of asset ids for a list of handles.
                let n = decl.list as usize;
                if values.len() > n {
                    return Err(format!(
                        "{at} holds at most {n}, got {}: a list is never truncated",
                        values.len()
                    ));
                }
                if let Some(exts) = handle_extensions(decl.column) {
                    let mut paths = Vec::with_capacity(values.len());
                    for v in values.iter() {
                        let path = handle_path(&at, decl.column, exts, std::slice::from_ref(v))?;
                        if !known(&path) {
                            missing.push(path.clone());
                        }
                        paths.push(Value::String(path));
                    }
                    *values = paths;
                    continue;
                }
                for v in values.iter() {
                    let x = v
                        .as_number()
                        .map(Number::as_f64)
                        .filter(|x| x.is_finite())
                        .ok_or_else(|| format!("{at} is a list of {} numbers", decl.column))?;
                    let whole = matches!(decl.column, "u32" | "i32");
                    if whole && (x.fract() != 0.0 || (decl.column == "u32" && x < 0.0)) {
                        return Err(format!("{at}: {x} is not a {}", decl.column));
                    }
                }
                continue;
            }
            if let Some(exts) = handle_extensions(decl.column) {
                let path = handle_path(&at, decl.column, exts, values)?;
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

/// The look `s` flattened over its chain and checked, every block: what
/// conformance reads beside the contract (`Stylise`'s paint fields, which
/// painted 2D styles apply in texel space, `conform/texture2d.rs`).
pub(super) fn flattened(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
) -> Result<LookFile, String> {
    resolve(s, root, known, PRESETS).map(|(look, _)| look)
}

/// The art contract of the look `s` (project.ogt's `look`): the look
/// flattened over its chain and checked, with only its art resources'
/// blocks (`crate::look::ART_RESOURCES`), which the cook reads before any
/// asset of the game cooks. An error is the look's own (its cook reports it
/// too).
pub(super) fn contract(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
) -> Result<LookFile, String> {
    let (mut look, _) = resolve(s, root, known, PRESETS)?;
    look.blocks
        .retain(|b| crate::look::ART_RESOURCES.contains(&b.resource.as_str()));
    Ok(look)
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
mod tests;
