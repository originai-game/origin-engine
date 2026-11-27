//! Checked `file.look` TS patches, including embedded presets, compose into
//! native look values. No author text parser or runtime text payload remains.
//!
//! - `extends: ["<slug>"]` names a built-in preset ([`PRESETS`]); typed
//!   content refs name project looks. There is no same-slug fallback.
//!   Bases chain up to [`MAX_EXTENDS`] deep; a cycle is refused. A look lies
//!   over its base field by field ([`LookFile::over`]).
//! - Every field is checked against the IDL's records (`og_records::RECORDS`,
//!   the reflection every tool reads): the resource and field exist and are
//!   writable, a number is in its range, a variant is one of the enum's, a
//!   colour is three linear numbers or an sRGB `#rrggbb` (written linear), a
//!   handle field names an asset of its kind that the bundle has.
//! - A handle field may name one of the engine's built-in assets
//!   ([`BUILTINS`], `templates/looks/assets/`) by its complete ID: the look stores
//!   its bundle id (`@canvas/paper.png`), and the cook adds the built-in to
//!   the game's bundle under that id ([`builtin_references`]), so the
//!   runtime loads it as any other asset. Paths under `assets/` starting
//!   with `@` are the engine's, and refused as a game's own.
//! - `mood: "<word>"` in a preset names the mood it is drawn in; in any other
//!   look it asks for a mood: when it differs from the preset's own, the
//!   mood's layer ([`MOOD_LAYERS`], `templates/looks/moods/`) is laid over
//!   the preset the chain reaches ([`lay_mood`]) before the looks above it,
//!   so the game's own fields still win. A chain that reaches no preset
//!   lays the layer under its deepest look.
//! - `variant: "<name>"` asks for one of the variants of the preset the
//!   chain reaches ([`VARIANT_LAYERS`], `templates/looks/variants/<slug>/`):
//!   its layer is laid over the preset ([`lay_variant`]) field by field,
//!   before the mood's, and the mood then reads the variant's own layer
//!   (`moods/<mood>/<slug>.<variant>.ts`) first. The nearest look's
//!   variant wins for the whole chain; a chain that reaches no preset, or a
//!   preset without that variant, is refused.
//! - The stamp covers every look in the chain, so editing a base re-cooks
//!   the looks built on it.

pub use super::look_assets::{BUILTINS, MOOD_LAYERS, PRESETS, VARIANT_LAYERS};
pub use super::look_styles::{
    StyleFile, fonts_dir, style_licences, style_path, styles, styles_dir,
};
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets, stamp};
use crate::format::{self, Area, BlobKind};
use crate::look::{LookFile, MAX_EXTENDS, hex_to_linear};
use og_blob::BlobWriter;
use og_core::ogt::{Number, Value};
use std::collections::BTreeMap;
use std::path::Path;

mod author;
pub(crate) use author::{Prepared, project as prepare};

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

/// UI checks consume the same checked TS composition as the cook, not a
/// second author parser. Project bootstrap itself never evaluates content.
pub fn project_ui_style(root: &Path) -> Result<Option<String>, String> {
    let project = crate::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let Prepared { looks, .. } = prepare(root, &project).map_err(|e| e.to_string())?;
    let id = match &project.metadata.look {
        Some(crate::cook::source_schemas::procedural::LookBase::Preset(slug)) => {
            format!("@looks/{slug}")
        }
        Some(crate::cook::source_schemas::procedural::LookBase::Asset(reference)) => {
            reference.id.clone()
        }
        None => return Ok(None),
    };
    let ui = match looks.get(&id).and_then(|look| look.ui.as_deref()) {
        Some(ui) => ui,
        None => return Ok(None),
    };
    if ui == "default" {
        return Ok(None);
    }
    resolve_style(ui, &|id| root.join("assets").join(id).is_dir()).map(Some)
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
        for f in styles()
            .iter()
            .filter(|f| f.id() == r || f.id().strip_suffix(".ts") == Some(r))
        {
            let f = f.pair();
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// The built-in asset's complete id (`"@canvas/paper.png"`) and bytes.
pub fn builtin(name: &str) -> Option<(&'static str, &'static [u8])> {
    BUILTINS.iter().find(|(id, _)| *id == name).copied()
}

/// The built-in assets the checked look names through its chain
/// of bases, by bundle id with their bytes: the canvases and palettes its
/// fields name, and every file of the built-in style its `ui` names (unless
/// the game has its own of that slug). The cook adds each to the bundle
/// once. A look that does not resolve names none here; its own cook reports
/// why.
pub(crate) fn builtin_files(look: &LookFile, root: &Path) -> Vec<(&'static str, &'static [u8])> {
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

fn builtin_looks() -> Result<&'static BTreeMap<String, LookFile>, String> {
    static LOOKS: std::sync::OnceLock<Result<BTreeMap<String, LookFile>, String>> =
        std::sync::OnceLock::new();
    LOOKS
        .get_or_init(|| {
            PRESETS
                .iter()
                .map(|(id, text)| ((*id).to_owned(), *text))
                .chain(
                    VARIANT_LAYERS.iter().map(|(slug, variant, text)| {
                        (format!("@variants/{slug}/{variant}"), *text)
                    }),
                )
                .chain(MOOD_LAYERS.iter().map(|(mood, key, text)| {
                    (
                        key.map_or_else(
                            || format!("@moods/{mood}"),
                            |key| format!("@moods/{mood}/{key}"),
                        ),
                        *text,
                    )
                }))
                .map(|(id, text)| author::builtin(&id, text).map(|look| (id, look)))
                .collect()
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// Compose checked patches. Strings select engine presets; content refs were
/// normalized to canonical IDs by the typed declaration adapter, never paths.
fn flatten(id: &str, patches: &BTreeMap<String, LookFile>) -> Result<LookFile, String> {
    flatten_asked(id, patches, &mut vec![id.into()], None, None)
}

fn flatten_asked(
    id: &str,
    patches: &BTreeMap<String, LookFile>,
    seen: &mut Vec<String>,
    asked: Option<&str>,
    asked_variant: Option<&str>,
) -> Result<LookFile, String> {
    let mut look = patches
        .get(id)
        .or(builtin_looks()?.get(id))
        .cloned()
        .ok_or_else(|| format!("extends: unknown look {id}"))?;
    // The mood asked here: the one asked from above (the nearest look's
    // wins for the whole chain), else this look's own.
    let asked = asked.map(str::to_string).or_else(|| look.mood.clone());
    let asked_variant = asked_variant
        .map(str::to_string)
        .or_else(|| look.variant.clone());
    let Some(name) = look.extends.first().cloned() else {
        let is_preset = PRESETS.iter().any(|(s, _)| *s == id);
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
            && !is_preset
        {
            let layer = lay_mood(&LookFile::default(), m, &[])?;
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
    if seen.contains(&name) {
        seen.push(name.clone());
        return Err(format!(
            "extends goes round in a circle: {}",
            seen.join(" → ")
        ));
    }
    seen.push(name.clone());
    let mut base_look = flatten_asked(
        &name,
        patches,
        seen,
        asked.as_deref(),
        asked_variant.as_deref(),
    )?;
    let is_preset = PRESETS.iter().any(|(s, _)| *s == name);
    if let Some(v) = asked_variant.as_deref()
        && is_preset
    {
        base_look = lay_variant(&base_look, &name, v)?;
    }
    if let Some(m) = asked.as_deref()
        && is_preset
        && base_look.mood.as_deref() != Some(m)
    {
        let own = asked_variant.as_deref().map(|v| format!("{name}.{v}"));
        let keys: Vec<&str> = own.as_deref().into_iter().chain([name.as_str()]).collect();
        base_look = lay_mood(&base_look, m, &keys)?;
    }
    look.extends.clear();
    let merged = look.over(&base_look, &name);
    Ok(merged)
}

/// The built-in presets flattened, as the knowledge package's `presets/`
/// carries them for the art brief (`packages/knowledge/scripts/brief.ts`,
/// docs/art.md §2.2), by file name with its JSON: `<slug>.json` for each
/// preset, its looks laid as a game's look asking for each variant (none
/// and every one of the preset's) in each mood (its own and every one of
/// [`crate::look::MOODS`]); `moods.json`, each mood's layer laid as under a
/// chain that reaches no preset; and `palettes.json`, the built-in palettes'
/// text by file name. This flattener is the only one; briefs read checked JSON.
pub fn preset_documents() -> Result<Vec<(String, String)>, String> {
    let entry = |variant: Option<&str>, mood: Option<&str>, look: &LookFile| serde_json::json!({ "variant": variant, "mood": mood, "look": look_json(look) });
    let asked = |slug: Option<&str>, variant: Option<&str>, mood: Option<&str>| {
        flatten(
            "@asked",
            &BTreeMap::from([(
                "@asked".into(),
                LookFile {
                    extends: slug.map(str::to_owned).into_iter().collect(),
                    variant: variant.map(str::to_owned),
                    mood: mood.map(str::to_owned),
                    ..Default::default()
                },
            )]),
        )
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

/// Knowledge reads checked cook results, never TS source or another parser.
pub(crate) const REPORT: &str = ".og/cook/look.json";

pub(crate) fn write_report(
    root: &Path,
    project: &crate::cook::source_project::Project,
    graph: &crate::cook::source_content::Graph,
    sources: &[Source],
    known: &dyn Fn(&str) -> bool,
) -> Result<(), CookError> {
    let Some(look_id) = project.metadata.look.as_ref() else {
        return Ok(());
    };
    let id = match look_id {
        crate::cook::source_schemas::procedural::LookBase::Preset(slug) => format!("@looks/{slug}"),
        crate::cook::source_schemas::procedural::LookBase::Asset(reference) => {
            graph.outputs[&reference.id].asset.clone()
        }
    };
    let source = sources
        .iter()
        .find(|s| s.rel == id && s.kind == &LOOK)
        .ok_or_else(|| CookError(format!("{id}: missing project look")))?;
    let look = resolve(source, known).map_err(CookError)?;
    let mut palettes = BTreeMap::new();
    for block in &look.blocks {
        for (field, values, _) in &block.fields {
            if !matches!(
                (block.resource.as_str(), field.as_str()),
                ("ArtPalette", "colors") | ("Stylise", "palette")
            ) {
                continue;
            }
            let [Value::String(id)] = values.as_slice() else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            let source = sources
                .iter()
                .find(|s| &s.rel == id)
                .ok_or_else(|| CookError(format!("{id}: missing look palette")))?;
            let text = std::str::from_utf8(&source.bytes).map_err(|e| CookError(e.to_string()))?;
            let file = source.path.to_string_lossy();
            crate::look::parse_palette(&file, text).map_err(CookError)?;
            palettes.insert(id.clone(), serde_json::json!({"format": if file.ends_with(".gpl") { "gpl" } else { "hex" }, "text": text}));
        }
    }
    let preset = look
        .extends
        .iter()
        .find(|id| PRESETS.iter().any(|(s, _)| s == id));
    let mut fingerprints = project.fingerprints.clone();
    let mut seen = std::collections::BTreeSet::new();
    let mut todo = vec![id.clone()];
    while let Some(id) = todo.pop() {
        let asset = graph.outputs.get(&id).map_or(id.as_str(), |o| &o.asset);
        let id = graph.outputs.get(&id).map_or(id.as_str(), |o| &o.entry);
        if !seen.insert(id.to_owned()) {
            continue;
        }
        if let Some(node) = graph.nodes.get(id) {
            for (file, hash) in &node.fingerprints {
                if fingerprints
                    .insert(file.clone(), hash.clone())
                    .is_some_and(|before| before != *hash)
                {
                    return Err(CookError(format!(
                        "{file}: changed during look cook; run og cook again"
                    )));
                }
            }
            todo.extend(node.references.iter().map(|r| r.target.clone()));
        }
        if let Some(source) = sources.iter().find(|s| s.rel == asset) {
            if let Ok(path) = source.path.strip_prefix(root)
                && source.path.is_file()
                && !graph.nodes.contains_key(id)
            {
                fingerprints.insert(
                    path.to_string_lossy().replace('\\', "/"),
                    super::integrity(&source.bytes),
                );
            }
            let module = graph
                .nodes
                .get(id)
                .map(|node| project.module_for(&node.entry.file));
            for (file, bytes) in &source.settings.files {
                let file = module.map_or_else(
                    || file.clone(),
                    |m| {
                        if m.root.is_empty() {
                            file.clone()
                        } else {
                            format!("{}/{file}", m.root)
                        }
                    },
                );
                fingerprints.insert(file, super::integrity(bytes));
            }
        }
    }
    let document = serde_json::json!({
        "id": id, "profile": if matches!(project.input.create.as_ref().and_then(|c| c.form.as_ref()), Some(crate::cook::source_schemas::project::Form::TwoD)) {"2d"} else {"3d"}, "preset": preset,
        "look": look_json(&look), "palettes": palettes, "fingerprints": fingerprints,
    });
    let path = root.join(REPORT);
    std::fs::create_dir_all(path.parent().expect("report directory"))
        .and_then(|_| std::fs::write(&path, serde_json::to_vec_pretty(&document).expect("JSON")))
        .map_err(|e| CookError(format!("{}: {e}", path.display())))
}

/// The variants' layer `variant` of the preset `slug`
/// (`variants/<slug>/<variant>.ts`) laid over `base`, the preset
/// flattened: field by field, the layer winning, its name, mood and
/// antialias where it gives them. Returns the laid look, carrying the
/// variant, and the layer as a base for the stamp.
fn lay_variant(base: &LookFile, slug: &str, variant: &str) -> Result<LookFile, String> {
    let Some(_) = VARIANT_LAYERS
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
    let layer = &builtin_looks()?[&id];
    if !layer.extends.is_empty() || layer.variant.is_some() {
        return Err(format!(
            "{id}: a variant's layer names neither extends nor variant"
        ));
    }
    let mut out = layer.over(base, slug);
    out.extends = base.extends.clone();
    out.name = layer.name.clone().or_else(|| base.name.clone());
    out.variant = Some(variant.to_string());
    Ok(out)
}

/// The moods' layer for the first of `keys` that has its own
/// (`moods/<mood>/<key>.ts`: a preset's variant `<slug>.<variant>`, then
/// the preset `<slug>`), else every style's (`moods/<mood>.ts`), with its
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
fn lay_mood(base: &LookFile, mood: &str, keys: &[&str]) -> Result<LookFile, String> {
    let (id, _) = mood_layer(mood, keys)
        .ok_or_else(|| format!("mood \"{mood}\": one of {}", crate::look::MOODS.join(", ")))?;
    let layer = &builtin_looks()?[&id];
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
                            // Preserve the composed layers' six-decimal precision.
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
    Ok(out)
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

/// A bound canonical ref, or an engine built-in checked by extension.
fn handle_path(at: &str, column: &str, exts: &[&str], values: &[Value]) -> Result<String, String> {
    let path = match values {
        [Value::String(s)] if s.starts_with('@') => match builtin(s) {
            Some((id, _)) => id.to_string(),
            None => {
                let kinds: Vec<&str> = BUILTINS
                    .iter()
                    .map(|(id, _)| *id)
                    .filter(|id| id.rsplit_once('.').is_some_and(|(_, e)| exts.contains(&e)))
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
    if path.starts_with('@') && !exts.contains(&ext.as_str()) {
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

pub(crate) fn bind(
    sources: &mut [Source],
    graph: &crate::cook::source_content::Graph,
) -> Result<(), CookError> {
    for source in sources.iter_mut().filter(|s| s.kind == &LOOK) {
        let mut look = LookFile::from_bytes(&source.bytes).map_err(CookError)?;
        for id in look.ui.iter_mut().chain(
            look.blocks
                .iter_mut()
                .flat_map(|b| b.fields.iter_mut())
                .flat_map(|(_, values, _)| values.iter_mut())
                .filter_map(|v| match v {
                    Value::String(s) => Some(s),
                    _ => None,
                }),
        ) {
            if let Some(output) = graph.outputs.get(id.as_str()) {
                *id = output.asset.clone();
            }
        }
        source.bytes = look.to_bytes().map_err(CookError)?;
        source.hash = super::hash_bytes(&source.bytes);
    }
    Ok(())
}

/// The flattened, checked look of `s` and the bases it read.
fn resolve(s: &Source, known: &dyn Fn(&str) -> bool) -> Result<LookFile, String> {
    let mut look = LookFile::from_bytes(&s.bytes)?;
    check(&mut look, known)?;
    Ok(look)
}

/// The stamp's dependencies: every base's id and text hash, and whether the
/// assets the look names are in the bundle.
pub(crate) fn look_dependencies(s: &Source, known: &dyn Fn(&str) -> bool) -> String {
    let mut out = Vec::new();
    match LookFile::from_bytes(&s.bytes) {
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
    out.join("\n")
}

/// The look `s` flattened over its chain and checked, every block: what
/// conformance reads beside the contract (`Stylise`'s paint fields, which
/// painted 2D styles apply in texel space, `conform/texture2d.rs`).
pub(crate) fn flattened(
    s: &Source,
    _root: &Path,
    known: &dyn Fn(&str) -> bool,
) -> Result<LookFile, String> {
    resolve(s, known)
}

/// The art contract of the look `s` (og-project.json's `look`): the look
/// flattened over its chain and checked, with only its art resources'
/// blocks (`crate::look::ART_RESOURCES`), which the cook reads before any
/// asset of the game cooks. An error is the look's own (its cook reports it
/// too).
pub(crate) fn contract(
    s: &Source,
    _root: &Path,
    known: &dyn Fn(&str) -> bool,
) -> Result<LookFile, String> {
    let mut look = resolve(s, known)?;
    look.blocks
        .retain(|b| crate::look::ART_RESOURCES.contains(&b.resource.as_str()));
    Ok(look)
}

/// Cooks one checked look patch.
pub(crate) fn cook_look(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let look = resolve(s, known).map_err(CookError)?;
    let data = look.to_bytes().map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Look::type_hash(),
        format::Look::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::LookMeta {
            data_bytes: data.len() as u32,
        }],
        format::row_hash::<format::LookMeta>(),
    )
    .raw("data", data, 16);
    let fields: usize = look.blocks.iter().map(|b| b.fields.len()).sum();
    Ok((
        w.finish(),
        vec![format!(
            "look '{}': {} ({fields} field(s)){}",
            look.name.as_deref().unwrap_or(&s.rel),
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
    types: &[],
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
    let Cx { known, .. } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = look_dependencies(s, &|id| known.contains(id));
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
        cook_look(s, &|id| known.contains(id), p.stamp)
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
