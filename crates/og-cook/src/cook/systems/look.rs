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
//! - `mood "<word>"` in a preset names the mood it is drawn in; in any other
//!   look it asks for a mood: when it differs from the preset's own, the
//!   mood's layer ([`MOOD_LAYERS`], `templates/looks/moods/`) is laid over
//!   the preset the chain reaches ([`lay_mood`]) before the looks above it,
//!   so the game's own fields still win. A chain that reaches no preset
//!   lays the layer under its deepest look.
//! - `variant "<name>"` asks for one of the variants of the preset the
//!   chain reaches ([`VARIANT_LAYERS`], `templates/looks/variants/<slug>/`):
//!   its layer is laid over the preset ([`lay_variant`]) field by field,
//!   before the mood's, and the mood then reads the variant's own layer
//!   (`moods/<mood>/<slug>.<variant>.ogt`) first. The nearest look's
//!   variant wins for the whole chain; a chain that reaches no preset, or a
//!   preset without that variant, is refused.
//! - The stamp covers every look in the chain, so editing a base re-cooks
//!   the looks built on it.

pub use super::look_assets::{BUILTINS, MOOD_LAYERS, PRESETS, VARIANT_LAYERS};
pub use super::look_styles::{
    StyleFile, fonts_dir, style_licences, style_path, styles, styles_dir,
};
use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets, assets_root, stamp,
};
use crate::format::{self, Area, BlobKind};
use crate::look::{LookFile, MAX_EXTENDS, hex_to_linear};
use og_blob::BlobWriter;
use og_core::ogt::{Number, Value};
use std::path::Path;

/// The built-in styles' slugs: the directories with a pack.json.
pub fn style_slugs() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = styles()
        .iter()
        .filter_map(|f| {
            let (dir, file) = f.id().strip_prefix("@styles/")?.rsplit_once('/')?;
            (!dir.contains('/') && file == "pack.json").then_some(dir)
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// A look's UI pack directory, either an explicit asset ID or a slug.
/// A game's `styles/<slug>` takes precedence over `@styles/<slug>`.
pub fn resolve_style(name: &str, own: &dyn Fn(&str) -> bool) -> Result<String, String> {
    if name.ends_with(".ogt") {
        return Err("style.ogt is removed; a UI style is a directory with a pack.json".into());
    }
    if name.contains('/') {
        let id = crate::runtime::asset_id(name);
        return Ok(id.trim_end_matches('/').to_string());
    }
    let mine = format!("styles/{name}");
    if own(&mine) {
        return Ok(mine);
    }
    // An engine theme (`templates/themes/<name>/theme.json`) before a
    // built-in style of the same name.
    if crate::cook::ui_themes::builtin_recipe(name).is_some() {
        return Ok(format!("@themes/{name}"));
    }
    let builtin = format!("@styles/{name}");
    if style_slugs().contains(&name) {
        return Ok(builtin);
    }
    if crate::cook::ui_themes::themes_dir().is_none() && styles_dir().is_none() {
        return Err(format!(
            "ui \"{name}\": neither assets/{mine} nor a built-in theme or style: the engine's built-in themes and styles were not found (no themes/ or styles/ in the @origingame/engine package; OG_THEMES_DIR / OG_STYLES_DIR names another directory)"
        ));
    }
    let mut available: Vec<String> = crate::cook::ui_themes::builtin_slugs();
    for slug in style_slugs() {
        available.push(slug.to_string());
    }
    available.sort();
    available.dedup();
    Err(format!(
        "ui \"{name}\": neither assets/{mine} nor a built-in theme or style ({})",
        available.join(", ")
    ))
}

/// The UI style id resolved from the look at `assets/<rel>` (flattening its
/// `extends` chain against the built-in presets), or `None` when the look does
/// not name a non-default UI style or cannot be read.
pub fn look_ui_style(assets_root: &Path, rel: &str) -> Option<String> {
    let text = std::fs::read_to_string(assets_root.join(rel)).ok()?;
    let mut seen = vec![rel.to_string()];
    let mut bases = Vec::new();
    let look = flatten(rel, &text, assets_root, PRESETS, &mut seen, &mut bases).ok()?;
    let ui = look.ui.as_deref()?;
    if ui == "default" {
        return None;
    }
    resolve_style(ui, &|id| assets_root.join(id).is_dir()).ok()
}

/// The files of a built-in pack directory, by bundle ID.
pub fn style_files(id: &str) -> Vec<(&'static str, &'static [u8])> {
    if !id.starts_with("@styles/") {
        return Vec::new();
    }
    let dir = format!("{}/", id.trim_end_matches('/'));
    styles()
        .iter()
        .filter(|f| f.id().starts_with(&dir))
        .map(StyleFile::pair)
        .collect()
}

/// Built-in assets a `pack.json` (at `source`) names, its fonts, images,
/// icons and sounds under `@styles/…`, together with their import settings.
/// A pack that does not parse names none (its own cook reports why).
pub fn pack_files(source: &str, text: &str) -> Vec<(&'static str, &'static [u8])> {
    let Ok(mut layer) = og_ui_next::pack::Layer::parse(text) else {
        return Vec::new();
    };
    layer.resolve(source.rsplit_once('/').map_or("", |(dir, _)| dir));
    let mut out: Vec<(&'static str, &'static [u8])> = Vec::new();
    for r in layer.assets() {
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
pub(crate) fn builtin_files(
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
        && ui != "default"
        && let Ok(id) = resolve_style(ui, &|id| root.join(id).is_dir())
    {
        let files = match id.strip_prefix("@themes/") {
            Some(theme) => crate::cook::ui_themes::builtin_files(
                theme,
                &crate::cook::ui_themes::cache_dir(root),
            )
            .unwrap_or_default(),
            None => style_files(&id),
        };
        for (source, bytes) in &files {
            if !crate::cook::ui_packs::is_pack(source) {
                continue;
            }
            for f in pack_files(source, &String::from_utf8_lossy(bytes)) {
                if !out.contains(&f) {
                    out.push(f);
                }
            }
        }
        // A pack is its pack.json and the assets it names; CSS has no reader.
        out.extend(files.into_iter().filter(|(id, _)| !id.ends_with(".css")));
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
    flatten_asked(id, text, root, presets, seen, bases, None, None)
}

/// [`flatten`] with the mood and variant a look above asked for (`asked`,
/// `asked_variant`), which win over this look's own `mood` and `variant`.
#[allow(clippy::too_many_arguments)]
fn flatten_asked(
    id: &str,
    text: &str,
    root: &Path,
    presets: &[(&str, &str)],
    seen: &mut Vec<String>,
    bases: &mut Vec<Base>,
    asked: Option<&str>,
    asked_variant: Option<&str>,
) -> Result<LookFile, String> {
    let mut look = LookFile::parse(text).map_err(|e| {
        if seen.len() > 1 {
            format!("{id}: {e}")
        } else {
            e
        }
    })?;
    // The mood asked here: the one asked from above (the nearest look's
    // wins for the whole chain), else this look's own.
    let asked = asked.map(str::to_string).or_else(|| look.mood.clone());
    let asked_variant = asked_variant
        .map(str::to_string)
        .or_else(|| look.variant.clone());
    let Some(name) = look.extends.first().cloned() else {
        let is_preset = presets.iter().any(|(s, _)| *s == id);
        if is_preset && look.variant.is_some() {
            return Err(format!(
                "{id}: a preset names no variant; a look extending it asks for one"
            ));
        }
        if let Some(v) = asked_variant.as_deref()
            && !is_preset
        {
            return Err(format!(
                "variant \"{v}\": a variant is one of a preset's, and this look's chain reaches no preset"
            ));
        }
        // A chain that reaches no preset: the mood lies under its root.
        if let Some(m) = asked.as_deref()
            && !presets.iter().any(|(s, _)| *s == id)
        {
            let (layer, layer_base) = lay_mood(&LookFile::default(), m, &[])?;
            bases.insert(0, layer_base);
            let mut merged = look.over(&layer, "");
            merged.extends.clear();
            merged.mood = Some(m.to_string());
            return Ok(merged);
        }
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
    let mut base_look = flatten_asked(
        &base.id,
        &base.text,
        root,
        presets,
        seen,
        bases,
        asked.as_deref(),
        asked_variant.as_deref(),
    )?;
    let is_preset = presets.iter().any(|(s, _)| *s == base.id);
    if let Some(v) = asked_variant.as_deref()
        && is_preset
    {
        let (layered, layer_base) = lay_variant(&base_look, &base.id, v)?;
        base_look = layered;
        bases.insert(0, layer_base);
    }
    if let Some(m) = asked.as_deref()
        && is_preset
        && base_look.mood.as_deref() != Some(m)
    {
        let own = asked_variant.as_deref().map(|v| format!("{}.{v}", base.id));
        let keys: Vec<&str> = own
            .as_deref()
            .into_iter()
            .chain([base.id.as_str()])
            .collect();
        let (layered, layer_base) = lay_mood(&base_look, m, &keys)?;
        base_look = layered;
        bases.insert(0, layer_base);
    }
    look.extends.clear();
    let merged = look.over(&base_look, &base.id);
    bases.insert(0, base);
    Ok(merged)
}

/// The built-in presets flattened, as the knowledge package's `presets/`
/// carries them for the art brief (`packages/knowledge/scripts/brief.ts`,
/// docs/art.md §2.2), by file name with its JSON: `<slug>.json` for each
/// preset, its looks laid as a game's look asking for each variant (none
/// and every one of the preset's) in each mood (its own and every one of
/// [`crate::look::MOODS`]); `moods.json`, each mood's layer laid as under a
/// chain that reaches no preset; and `palettes.json`, the built-in palettes'
/// text by file name. This flattener is the only one: the brief lays only a
/// game's own looks over these, field by field.
pub fn preset_documents() -> Result<Vec<(String, String)>, String> {
    let entry = |variant: Option<&str>, mood: Option<&str>, look: &LookFile| serde_json::json!({ "variant": variant, "mood": mood, "look": look_json(look) });
    let asked = |slug: Option<&str>, variant: Option<&str>, mood: Option<&str>| {
        let mut text = String::from("type look\nversion 1\n---\n");
        for (key, v) in [("extends", slug), ("variant", variant), ("mood", mood)] {
            if let Some(v) = v {
                text.push_str(&format!("{key} \"{v}\"\n"));
            }
        }
        // The asking look's id is the preset's own path, which never
        // extends itself, so the slug is the built-in preset and no game's.
        let id = format!("looks/{}.ogt", slug.unwrap_or("asked"));
        let (mut seen, mut bases) = (vec![id.clone()], Vec::new());
        flatten(&id, &text, Path::new(""), PRESETS, &mut seen, &mut bases)
    };
    let pretty = |v: serde_json::Value| {
        serde_json::to_string_pretty(&v).expect("JSON values serialise") + "\n"
    };
    let mut out = Vec::new();
    for (slug, _) in PRESETS {
        let variants = VARIANT_LAYERS.iter().filter(|(s, _, _)| s == slug);
        let mut looks = Vec::new();
        for variant in std::iter::once(None).chain(variants.map(|(_, v, _)| Some(*v))) {
            for mood in std::iter::once(None).chain(crate::look::MOODS.map(Some)) {
                let look = asked(Some(slug), variant, mood).map_err(|e| format!("{slug}: {e}"))?;
                looks.push(entry(variant, mood, &look));
            }
        }
        let doc = serde_json::json!({
            "about": format!("The look preset {slug} flattened by og-cook's look step for each variant and mood a game's look may ask of it (`og api gen` writes it from templates/looks; never edit it)."),
            "slug": slug,
            "looks": looks,
        });
        out.push((format!("{slug}.json"), pretty(doc)));
    }
    let mut moods = Vec::new();
    for mood in crate::look::MOODS {
        let look = asked(None, None, Some(mood))?;
        moods.push(entry(None, Some(mood), &look));
    }
    out.push((
        "moods.json".into(),
        pretty(serde_json::json!({
            "about": "Each mood's layer as og-cook's look step lays it under a look whose chain reaches no preset (`og api gen` writes it from templates/looks/moods; never edit it).",
            "looks": moods,
        })),
    ));
    let mut palettes = serde_json::Map::new();
    for (id, bytes) in BUILTINS {
        if let Some(name) = id.strip_prefix("@palettes/") {
            let text = std::str::from_utf8(bytes).map_err(|_| format!("{id} is not text"))?;
            palettes.insert(name.into(), text.into());
        }
    }
    out.push((
        "palettes.json".into(),
        pretty(serde_json::json!({
            "about": "The engine's built-in palettes (`@palettes/<name>` in a look), each file's text by name (`og api gen` writes it from templates/looks/assets/palettes; never edit it).",
            "palettes": palettes,
        })),
    ));
    Ok(out)
}

/// A flattened look as the presets' JSON holds it: its top-level fields and
/// its blocks, each field's values as written (numbers, strings, `true` and
/// `false` as booleans, other words as strings).
fn look_json(look: &LookFile) -> serde_json::Value {
    let value = |v: &Value| match v {
        Value::Number(n) => {
            let x = n.as_f64();
            if x.fract() == 0.0 && x.abs() < 9e15 {
                serde_json::json!(x as i64)
            } else {
                serde_json::json!(x)
            }
        }
        Value::String(s) => serde_json::json!(s),
        Value::Word(w) if w == "true" || w == "false" => serde_json::json!(w == "true"),
        Value::Word(w) => serde_json::json!(w),
    };
    let blocks: serde_json::Map<String, serde_json::Value> = look
        .blocks
        .iter()
        .map(|b| {
            let fields: serde_json::Map<String, serde_json::Value> = b
                .fields
                .iter()
                .map(|(f, v, _)| (f.clone(), v.iter().map(value).collect()))
                .collect();
            (b.resource.clone(), fields.into())
        })
        .collect();
    let seeds: serde_json::Map<String, serde_json::Value> = look
        .ui_seeds
        .iter()
        .map(|(n, h)| (n.clone(), h.clone().into()))
        .collect();
    serde_json::json!({
        "name": look.name,
        "mood": look.mood,
        "variant": look.variant,
        "ui": look.ui,
        "antialias": look.antialias,
        "uiSeeds": seeds,
        "blocks": blocks,
    })
}

/// The variants' layer `variant` of the preset `slug`
/// (`variants/<slug>/<variant>.ogt`) laid over `base`, the preset
/// flattened: field by field, the layer winning, its name, mood and
/// antialias where it gives them. Returns the laid look, carrying the
/// variant, and the layer as a base for the stamp.
fn lay_variant(base: &LookFile, slug: &str, variant: &str) -> Result<(LookFile, Base), String> {
    let Some((_, _, text)) = VARIANT_LAYERS
        .iter()
        .find(|(s, v, _)| *s == slug && *v == variant)
    else {
        let names: Vec<&str> = VARIANT_LAYERS
            .iter()
            .filter(|(s, _, _)| *s == slug)
            .map(|(_, v, _)| *v)
            .collect();
        return Err(if names.is_empty() {
            format!("variant \"{variant}\": the preset {slug} has no variants")
        } else {
            format!(
                "variant \"{variant}\": the preset {slug} takes one of {}",
                names.join(", ")
            )
        });
    };
    let id = format!("@variants/{slug}/{variant}");
    let layer = LookFile::parse(text).map_err(|e| format!("{id}: {e}"))?;
    if !layer.extends.is_empty() || layer.variant.is_some() {
        return Err(format!(
            "{id}: a variant's layer names neither extends nor variant"
        ));
    }
    let mut out = layer.over(base, slug);
    out.extends = base.extends.clone();
    out.name = layer.name.clone().or_else(|| base.name.clone());
    out.variant = Some(variant.to_string());
    Ok((
        out,
        Base {
            id,
            text: text.to_string(),
        },
    ))
}

/// The moods' layer for the first of `keys` that has its own
/// (`moods/<mood>/<key>.ogt`: a preset's variant `<slug>.<variant>`, then
/// the preset `<slug>`), else every style's (`moods/<mood>.ogt`), with its
/// id.
fn mood_layer(mood: &str, keys: &[&str]) -> Option<(String, &'static str)> {
    let own = keys.iter().find_map(|s| {
        MOOD_LAYERS
            .iter()
            .find(|(m, l, _)| *m == mood && *l == Some(*s))
            .map(|(_, _, t)| (format!("@moods/{mood}/{s}"), *t))
    });
    own.or_else(|| {
        MOOD_LAYERS
            .iter()
            .find(|(m, l, _)| *m == mood && l.is_none())
            .map(|(_, _, t)| (format!("@moods/{mood}"), *t))
    })
}

/// The resources and fields a mood's layer may set: the frame's grading,
/// light and palette, never the art contract's sizes, shapes or lines.
fn mood_sets(resource: &str, field: &str) -> bool {
    matches!(
        resource,
        "PostEffects" | "RenderSettings" | "Environment" | "Fog" | "Sky"
    ) || (resource, field) == ("Stylise", "palette")
        || (resource, field) == ("ArtPalette", "colors")
}

/// The numbers of a field's default (its JSON: `1`, `[0,0,0]`); empty when
/// it is not numbers.
fn default_numbers(json: &str) -> Vec<f64> {
    json.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|t| t.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default()
}

/// `base` laid in `mood` (docs/art.md §2.10): the mood's layer for `keys`
/// over it, a number moving the base's value (else the field's default) by
/// the layer's distance from the default, clamped to the field's range and
/// rounded for a whole-number field; any other value replacing the base's.
/// Returns the laid look, carrying the mood, and the layer as a base for the
/// stamp.
fn lay_mood(base: &LookFile, mood: &str, keys: &[&str]) -> Result<(LookFile, Base), String> {
    let (id, text) = mood_layer(mood, keys)
        .ok_or_else(|| format!("mood \"{mood}\": one of {}", crate::look::MOODS.join(", ")))?;
    let layer = LookFile::parse(text).map_err(|e| format!("{id}: {e}"))?;
    let mut out = base.clone();
    for block in &layer.blocks {
        let schema = format!("og.{}", block.resource);
        let record = og_records::RECORDS
            .iter()
            .find(|r| r.schema == schema && r.kind == "resource")
            .ok_or_else(|| format!("{id}: {} is not a resource of this engine", block.resource))?;
        for (field, values, _) in &block.fields {
            if !mood_sets(&block.resource, field) {
                return Err(format!(
                    "{id}: {}.{field}: a mood sets the frame's grading, light and palette only",
                    block.resource
                ));
            }
            let decl = record
                .fields
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| format!("{id}: {} has no field '{field}'", block.resource))?;
            let target = match out.blocks.iter_mut().find(|b| b.resource == block.resource) {
                Some(b) => b,
                None => {
                    out.blocks.push(crate::look::LookBlock {
                        resource: block.resource.clone(),
                        fields: Vec::new(),
                    });
                    out.blocks.last_mut().expect("just pushed")
                }
            };
            let layer_numbers: Option<Vec<f64>> = values
                .iter()
                .map(|v| v.as_number().map(Number::as_f64))
                .collect();
            let default = default_numbers(decl.default);
            let numeric = matches!(decl.column, "f32" | "u32" | "i32" | "Vector2" | "Vector3");
            let value = match layer_numbers {
                Some(l) if numeric && !l.is_empty() && l.len() == default.len() => {
                    let have: Vec<f64> = target
                        .get(field)
                        .and_then(|v| {
                            v.iter()
                                .map(|x| x.as_number().map(Number::as_f64))
                                .collect::<Option<Vec<f64>>>()
                        })
                        .filter(|v| v.len() == default.len())
                        .unwrap_or_else(|| default.clone());
                    have.iter()
                        .zip(&l)
                        .zip(&default)
                        .map(|((h, l), d)| {
                            let mut x = h + (l - d);
                            if let Some(lo) = decl.min {
                                x = x.max(lo);
                            }
                            if let Some(hi) = decl.max {
                                x = x.min(hi);
                            }
                            if matches!(decl.column, "u32" | "i32") {
                                x = x.round();
                            }
                            // Six decimals: the layer's sums are written as text.
                            number((x * 1e6).round() / 1e6)
                        })
                        .collect()
                }
                _ => values.clone(),
            };
            match target.fields.iter_mut().find(|(n, _, _)| n == field) {
                Some(slot) => slot.1 = value,
                None => target.fields.push((field.clone(), value, 0)),
            }
        }
    }
    out.mood = Some(mood.to_string());
    Ok((
        out,
        Base {
            id,
            text: text.to_string(),
        },
    ))
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
    if look.ui.as_deref() == Some("default") {
        look.ui = None;
    }
    if let Some(name) = &look.ui {
        // The blob names the style by its bundle id, resolved here: its
        // directory, which the bundle holds as that directory's pack.json.
        let has = |dir: &str| known(&format!("{dir}/pack.json"));
        let id = resolve_style(name, &has)?;
        if !has(&id) {
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
pub(crate) fn look_dependencies(s: &Source, root: &Path, known: &dyn Fn(&str) -> bool) -> String {
    let text = String::from_utf8_lossy(&s.bytes);
    let mut seen = vec![s.rel.clone()];
    let mut bases = Vec::new();
    let mut out = Vec::new();
    match flatten(&s.rel, &text, root, PRESETS, &mut seen, &mut bases) {
        Ok(look) => {
            let mut refs: Vec<String> = look
                .ui
                .iter()
                .filter(|u| u.as_str() != "default")
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
pub(crate) fn flattened(
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
pub(crate) fn contract(
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
pub(crate) fn cook_look(
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
pub(crate) fn cook_palette(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
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

pub(crate) static LUT: KindDef = KindDef {
    name: "lut",
    extensions: &["cube"],
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_lut,
};

fn plan_lut<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Look], &[b"lut", &hash]);
    outs.push((s.rel.clone(), format::Lut::NAME, st, Box::new(LutJob(s))));
}

/// A colour table's one output.
struct LutJob<'a>(&'a Source);

impl Job for LutJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let LutJob(s) = self;
        let text = std::str::from_utf8(&s.bytes)
            .map_err(|_| CookError("a .cube LUT is text (UTF-8)".into()))?;
        let lut = crate::fx::parse_cube(text).map_err(CookError)?;
        let mut w = og_blob::BlobWriter::new(
            format::Lut::type_hash(),
            format::Lut::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::LutMeta { size: lut.size }],
            format::row_hash::<format::LutMeta>(),
        )
        .raw("rgba", lut.rgba, 16);
        Ok((w.finish(), vec![format!("{0}³ colour table", lut.size)]))
    }
}

pub(crate) static LOOK: KindDef = KindDef {
    name: "look",
    extensions: &[],
    types: &[crate::look::LOOK_TYPE],
    by_content: false,
    streams: true,
    plan: plan_look,
};

fn plan_look<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        known, assets_dir, ..
    } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = look_dependencies(s, assets_dir, &|id| known.contains(id));
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    let st = stamp(&[Area::Look], &[b"look 1", &hash, deps.as_bytes()]);
    outs.push((s.rel.clone(), format::Look::NAME, st, Box::new(LookJob(s))));
}

/// A look's one output: the flattened look.
struct LookJob<'a>(&'a Source);

impl Job for LookJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let LookJob(s) = self;
        let Exec { known, .. } = ex;
        cook_look(s, &assets_root(s), &|id| known.contains(id), p.stamp)
    }
}

pub(crate) static PALETTE: KindDef = KindDef {
    name: "palette",
    extensions: &["hex", "gpl"],
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_palette,
};

fn plan_palette<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Look], &[b"palette 1", &hash]);
    outs.push((
        s.rel.clone(),
        format::Palette::NAME,
        st,
        Box::new(PaletteJob(s)),
    ));
}

/// A palette's one output.
struct PaletteJob<'a>(&'a Source);

impl Job for PaletteJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PaletteJob(s) = self;
        cook_palette(s, p.stamp)
    }
}

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&LUT, &LOOK, &PALETTE];

#[cfg(test)]
mod tests;
