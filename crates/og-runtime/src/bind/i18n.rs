//! The IDL's `i18n` module over og-i18n: an engine's string tables (loaded
//! from the bundle), the locale the host asked for, `t`, and the UI's font
//! chain and direction for the active locale.

use crate::engine::{Engine, engine};
use crate::host::wire::LocaleInfo;
use crate::parse;
use og_i18n::{Arg, BundleFontPacks, BundleStrings, Catalog, StringTable};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct State {
    catalog: Catalog,
    /// Font paths of the active chain, as last applied to the UI.
    fonts: Vec<String>,
    warned: BTreeSet<String>,
    /// The bundle's font packs (docs/ui.md §3.4), read once it is open
    /// (`None`: not yet).
    packs: Option<Vec<og_i18n::FontPack>>,
    /// Packs loaded beyond the locale's own: a frame drew a character only
    /// they cover. Kept across locale changes.
    loaded: Vec<String>,
    /// Pack faces of the chain still being fetched (the web reads a pack's
    /// faces on demand, not at boot): the chain is applied again as they land.
    pending: Vec<String>,
    /// The style pack's faces still being fetched, likewise.
    styling: Vec<String>,
    /// The characters the last UI frame drew that no loaded face has:
    /// while one is left and a face is pending, the first frame waits
    /// ([`fonts_pending`]).
    missed: Vec<char>,
}

/// og.i18n on one engine: its string tables, the active locale and its
/// fonts.
pub(crate) type ModuleState = RefCell<State>;

fn with<T>(engine: u32, f: impl FnOnce(&mut State) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.i18n.borrow_mut()),
        // An engine that is gone, or not made yet, has none: a fresh one.
        Err(_) => f(&mut State::default()),
    }
}

/// Adds a document's or a dialogue's authored strings to the base table.
/// The base table can take over from a loaded translation (a game that
/// loaded its `ar` table before its first document, with no locale asked
/// for, had `ar` active until the documents' English base arrived), so the
/// UI then takes the active locale's fonts and direction again. Without a
/// table before, nothing was applied and nothing is.
pub(crate) fn base_strings(
    eng: &Engine,
    locale: &str,
    strings: &BTreeMap<String, String>,
) -> Result<(), String> {
    authored(eng, locale, strings, Authored::Base)
}

/// What authored strings are to the catalog.
#[derive(Clone, Copy)]
enum Authored {
    /// The game's base language.
    Base,
    /// A translation of it: a component's table of another locale.
    Translation,
    /// The engine's own UI (`@ui/...`): no language of the game.
    Ui,
}

/// Authored strings of `locale` into the catalog, as `kind` says.
fn authored(
    eng: &Engine,
    locale: &str,
    strings: &BTreeMap<String, String>,
    kind: Authored,
) -> Result<(), String> {
    let active = |s: &mut State| s.catalog.active().map(|t| t.locale.clone());
    let moved = with(eng.id, |s| -> Result<bool, String> {
        let before = active(s);
        match kind {
            Authored::Base => s.catalog.extend_base(locale, strings)?,
            Authored::Translation => s.catalog.extend_translation(locale, strings)?,
            Authored::Ui => s.catalog.extend_ui(locale, strings)?,
        }
        Ok(before.is_some() && active(s) != before)
    })?;
    if moved {
        apply(eng)?;
    }
    Ok(())
}

/// The cooked string table `id` of the engine's bundle.
pub(crate) fn read_table(eng: &Engine, id: &str) -> Result<StringTable, String> {
    let assets = eng.assets.borrow();
    let (bytes, bundle) = assets.blob(id)?;
    bundle.strings(id, bytes.bytes()).map_err(|e| e.to_string())
}

/// A UI's string table `id` as authored strings of its locale, beside the
/// game's tables (a document's, a TSX component's): a game's table of the
/// same locale keeps its own entries. The engine's own (`@ui/...`) are no
/// language of the game; of the game's, English is the base and a table of
/// another locale is a translation of it, unless it says `base true`.
fn add_base(eng: &Engine, id: &str, table: StringTable) -> Result<(), String> {
    let kind = if id.starts_with("@ui/") {
        Authored::Ui
    } else if table.base
        || table
            .locale
            .split(['-', '_'])
            .next()
            .is_some_and(|l| l.eq_ignore_ascii_case("en"))
    {
        Authored::Base
    } else {
        Authored::Translation
    };
    let strings = table
        .entries
        .into_iter()
        .map(|(key, (text, _))| (key, text))
        .collect();
    authored(eng, &table.locale, &strings, kind)
}

/// The string tables a world's TSX components import (op 12 of the UI's
/// batches), registered as base strings; one that does not read is a
/// warning, as a missing key is.
pub(crate) fn component_tables(eng: &Engine, ids: Vec<String>) {
    for id in ids {
        if let Err(e) = read_table(eng, &id).and_then(|t| add_base(eng, &id, t)) {
            note(eng.id, format!("og.ui: the string table {id}: {e}"));
        }
    }
}

fn info(engine: u32) -> LocaleInfo {
    with(engine, |s| {
        let active = s.catalog.active();
        LocaleInfo {
            requested: s.catalog.requested().unwrap_or("").to_string(),
            active: active.map_or("", |t| t.locale.as_str()).to_string(),
            direction: s.catalog.direction().as_str().to_string(),
            locales: s.catalog.locales(),
            fonts: s.fonts.clone(),
        }
    })
}

/// The full formatting locale for documents; plural rules derive their
/// language themselves, while Chinese scripts and pseudo tags must survive.
pub(crate) fn language(engine: u32) -> String {
    let locale = tag(engine);
    if locale.is_empty() {
        "en".into()
    } else {
        locale
    }
}

/// The active table's locale, else the requested one (empty: none): what
/// og.i18n's format functions write in.
fn tag(engine: u32) -> String {
    with(engine, |s| {
        if let Some(requested) = s.catalog.requested()
            && og_i18n::is_pseudo_locale(requested)
        {
            return requested.to_string();
        }
        s.catalog
            .active()
            .map(|t| t.locale.clone())
            .or_else(|| s.catalog.requested().map(str::to_string))
            .unwrap_or_default()
    })
}

/// The active locale writes right to left (og.ui.frameDirection).
pub(crate) fn rtl(engine: u32) -> bool {
    direction(engine) == og_i18n::Direction::Rtl
}

/// The active locale's writing direction, which text without a direction
/// of its own takes.
pub(crate) fn direction(engine: u32) -> og_i18n::Direction {
    with(engine, |s| s.catalog.direction())
}

/// The host's locale at creation (`og run --locale`, the browser's language).
pub(crate) fn request(eng: &Engine, locale: &str) {
    if !locale.trim().is_empty() {
        eng.modules.i18n.borrow_mut().catalog.request(&[locale]);
    }
}

/// Applies the active locale to the UI: its fonts first, then the other
/// tables', each loaded from the bundle; and its direction.
pub(crate) fn apply(eng: &Engine) -> Result<(), String> {
    let locale = language(eng.id);
    let (mut paths, _) = with(eng.id, |s| {
        let mut paths: Vec<String> = s
            .catalog
            .active()
            .map(|t| t.fonts.clone())
            .unwrap_or_default();
        // The locale's own font pack after the table's own fonts, then the
        // packs a frame needed; no game code names them.
        let packs = s.packs.as_deref().unwrap_or_default();
        let own = og_i18n::font_pack::for_locale(packs, &locale).map(|p| p.name.clone());
        for name in own.iter().chain(&s.loaded) {
            for p in packs.iter().filter(|p| &p.name == name) {
                for f in &p.faces {
                    if !paths.contains(f) {
                        paths.push(f.clone());
                    }
                }
            }
        }
        for l in s.catalog.locales() {
            let mut c = s.catalog.clone();
            c.request(&[&l]);
            if let Some(t) = c.active() {
                for f in &t.fonts {
                    if !paths.contains(f) {
                        paths.push(f.clone());
                    }
                }
            }
        }
        (paths, s.catalog.direction())
    });
    // The default face draws a role the game gave no font.
    let unset = |field: &str| {
        let state = eng.state.borrow();
        state.ui_desc.get(field).is_none_or(Value::is_null)
    };
    if unset("font")
        && eng
            .assets
            .borrow()
            .bundle_for(og_ui_next::pack::DEFAULT_FONT_ASSET)
            .is_ok_and(|(bundle, _)| bundle.output(og_ui_next::pack::DEFAULT_FONT_ASSET).is_ok())
    {
        paths.insert(0, og_ui_next::pack::DEFAULT_FONT_ASSET.to_string());
    }
    // The same fonts for TSX text (present::faces): the default face
    // whenever the bundle has it, then the locale's in order, so a family
    // no file backs and a character no family has still draw.
    let mut faces = Vec::with_capacity(paths.len() + 1);
    let mut pending = Vec::new();
    let mut drawn = Vec::with_capacity(paths.len());
    for p in &paths {
        let id = og_bundle::runtime::asset_id(p);
        if eng.assets.borrow().pending(id) {
            pending.push(p.clone());
            continue;
        }
        drawn.push(p.clone());
        let i = crate::world::assets::load_font(eng, p).map_err(|e| format!("font '{p}': {e}"))?;
        if p == og_ui_next::pack::DEFAULT_FONT_ASSET {
            faces.insert(0, i);
        } else {
            faces.push(i);
        }
    }
    let default = og_ui_next::pack::DEFAULT_FONT_ASSET;
    let has_default = eng
        .assets
        .borrow()
        .bundle_for(default)
        .is_ok_and(|(bundle, _)| bundle.output(default).is_ok());
    if has_default {
        if eng
            .assets
            .borrow()
            .pending(og_bundle::runtime::asset_id(default))
        {
            if !pending.iter().any(|p| p == default) {
                pending.push(default.to_string());
            }
        } else {
            let i = crate::world::assets::load_font(eng, default)
                .map_err(|e| format!("font '{default}': {e}"))?;
            // Right behind the pack's faces, before the locale's: a
            // character a pack face lacks (→ in Fredoka) takes the default
            // body face, as a browser takes its default font.
            faces.insert(0, i);
        }
    }
    // The style pack's files, in its order, by the same rule: one still
    // being fetched joins when it lands, and text draws from the next
    // face meanwhile.
    let (mut style, mut styling) = (Vec::new(), Vec::new());
    for p in crate::present::faces::style_files(eng) {
        if eng
            .assets
            .borrow()
            .pending(og_bundle::runtime::asset_id(&p))
        {
            styling.push(p);
            continue;
        }
        style.push(
            crate::world::assets::load_font(eng, &p).map_err(|e| format!("font '{p}': {e}"))?,
        );
    }
    with(eng.id, |s| {
        s.fonts = drawn;
        s.pending = pending;
        s.styling = styling;
    });
    crate::present::faces::configure(eng, &style, &faces);
    Ok(())
}

/// Reads the bundle's font packs once it is open and applies the locale's
/// chain with them: the first UI frame of a game with packs installed
/// draws in them with no call of its own. Cheap after the first time.
pub(crate) fn ensure_packs(eng: &Engine) {
    if let Some(waiting) = with(eng.id, |s| {
        s.packs.as_ref().map(|_| {
            s.pending
                .iter()
                .chain(&s.styling)
                .cloned()
                .collect::<Vec<_>>()
        })
    }) {
        // A pack face fetched since: the chain takes it.
        let landed = {
            let assets = eng.assets.borrow();
            waiting
                .iter()
                .any(|p| !assets.pending(og_bundle::runtime::asset_id(p)))
        };
        if landed {
            if let Err(e) = apply(eng) {
                note(eng.id, format!("font packs: {e}"));
            }
            // Behind the play gate no tick lays the frame out again: what
            // the landed faces draw is no longer missing, the rest still is
            // (a Latin face landing first did not release a frame that
            // still lacked its Han).
            let missed = with(eng.id, |s| std::mem::take(&mut s.missed));
            let left: Vec<char> = {
                let rs = eng.render.borrow();
                missed
                    .into_iter()
                    .filter(|&c| !rs.ui_faces.draws(c))
                    .collect()
            };
            with(eng.id, |s| s.missed = left);
        }
        return;
    }
    let packs = {
        let assets = eng.assets.borrow();
        let Some(i) = assets.default_bundle else {
            return;
        };
        if assets.entries[i].status != crate::world::assets::Status::Ready {
            return;
        }
        let ids: Vec<String> = match &assets.entries[i].payload {
            crate::world::assets::Payload::Bundle { bundle, .. } => bundle
                .manifest()
                .assets
                .iter()
                .filter(|a| a.kind == "font-pack")
                .flat_map(|a| a.outputs.iter().map(|o| o.id.clone()))
                .collect(),
            _ => Vec::new(),
        };
        let mut packs = Vec::new();
        let mut dictionaries = Vec::new();
        let mut segmentation = Vec::new();
        for id in ids {
            let read = assets.blob(&id).and_then(|(bytes, bundle)| {
                let font = bundle
                    .font_pack(&id, bytes.bytes())
                    .map_err(|e| e.to_string())?;
                let pack = |kind| {
                    bundle
                        .font_pack_dictionaries(&id, bytes.bytes(), kind)
                        .map_err(|e| e.to_string())
                };
                Ok((font, pack("hyphenation")?, pack("segmentation")?))
            });
            match read {
                Ok((p, h, w)) => {
                    packs.push(p);
                    dictionaries.extend(h);
                    segmentation.extend(w);
                }
                Err(e) => note(eng.id, format!("font pack {id}: {e}")),
            }
        }
        (packs, dictionaries, segmentation)
    };
    let (packs, dictionaries, segmentation) = packs;
    // The hyphenation and word dictionaries the cook shipped for the
    // game's locales: a UI made later finds them installed.
    for (language, bytes) in dictionaries {
        og_i18n::hyphenation::install(&language, bytes.into());
    }
    for (attribute, bytes) in segmentation {
        let added = og_text::text::add_segmentation(&attribute, &bytes);
        if let Err(e) = added {
            note(eng.id, format!("font packs: {e}"));
        }
    }
    with(eng.id, |s| s.packs = Some(packs));
    if let Err(e) = apply(eng) {
        note(eng.id, format!("font packs: {e}"));
    }
}

/// Of the characters a frame drew that their faces lack, those an
/// installed pack not yet in the chain covers load that pack (the
/// locale's CJK forms first) and are answered as drawn from next frame;
/// the rest are returned, still missing.
pub(crate) fn resolve_missing(eng: &Engine, mut missing: Vec<char>) -> Vec<char> {
    // Only the active style can supply a fallback. Its tiny cmap metadata
    // is resident with the layer; the face's bytes are not requested until
    // a character actually needs them.
    let style = eng
        .state
        .borrow()
        .ui_desc
        .get("style")
        .and_then(Value::as_f64)
        .and_then(crate::world::assets::index_of);
    let mut style_added = false;
    if let Some(i) = style {
        let mut assets = eng.assets.borrow_mut();
        if let Some(style) = assets.styles.get_mut(i) {
            let mut fresh = BTreeSet::new();
            missing.retain(|c| {
                if let Some(font) = style
                    .fallback_fonts
                    .iter()
                    .find(|f| f.coverage.iter().any(|r| r.contains(&(*c as u32))))
                {
                    if !style.files.contains(&font.id) {
                        fresh.insert(font.id.clone());
                    }
                    return false;
                }
                true
            });
            style_added = !fresh.is_empty();
            style.files.extend(fresh);
        }
    }
    if style_added && let Err(e) = apply(eng) {
        note(eng.id, format!("style fonts: {e}"));
    }
    let locale = language(eng.id);
    let (out, added) = with(eng.id, |s| {
        let Some(packs) = s.packs.as_deref().filter(|p| !p.is_empty()) else {
            return (missing, false);
        };
        let own = og_i18n::font_pack::for_locale(packs, &locale).map(|p| p.name.clone());
        let order = og_i18n::font_pack::fallback_order(packs, &locale);
        // Packs this frame loads: their characters draw from the next one.
        let mut fresh: Vec<String> = Vec::new();
        let mut left = Vec::new();
        for c in missing {
            // Only a pack not yet in the chain answers it; one already in
            // it lacks the outline after all (a cmap entry, no glyph).
            let pack = order
                .iter()
                .map(|&i| &packs[i])
                .filter(|p| Some(&p.name) != own.as_ref())
                .find(|p| p.covers(c));
            match pack {
                Some(p) if !s.loaded.contains(&p.name) => {
                    s.loaded.push(p.name.clone());
                    fresh.push(p.name.clone());
                }
                Some(p) if fresh.contains(&p.name) => {}
                _ => left.push(c),
            }
        }
        (left, !fresh.is_empty())
    });
    if added && let Err(e) = apply(eng) {
        note(eng.id, format!("font packs: {e}"));
    }
    out
}

/// [`resolve_missing`] for characters no face of the TSX UI's collection
/// draws: what is left when the packs that cover the others load.
pub(crate) fn resolve_missing_chars(eng: &Engine, missing: Vec<char>) -> Vec<char> {
    let left = resolve_missing(eng, missing);
    // A face still being fetched may have the rest: none is missing yet.
    if with(eng.id, |s| !s.pending.is_empty() || !s.styling.is_empty()) {
        return Vec::new();
    }
    left
}

/// Whether the locale's chain still waits for a font-pack face being
/// fetched (the web reads them after the boot; never on native).
pub(crate) fn faces_pending(eng: &Engine) -> bool {
    ensure_packs(eng);
    with(eng.id, |s| !s.pending.is_empty())
}

/// The characters the UI frame just laid out has that no loaded face draws.
pub(crate) fn frame_missed(eng: &Engine, missed: &[char]) {
    // A frame that laid nothing out again reports none: the set stands
    // until a frame finds others or a landed face draws them (ensure_packs).
    if !missed.is_empty() {
        with(eng.id, |s| s.missed = missed.to_vec());
    }
}

/// Faces and glyphs every engine's frame still waits for: the web Worker
/// draws the first frame again until none is left, so a boot never shows
/// text missing while its face is fetched. A face being fetched counts
/// only while the frame lacks a character: a frame its loaded faces draw
/// whole does not wait for the rest.
pub fn fonts_pending() -> u32 {
    let engines = crate::engine::engine_ids();
    // A face that landed since the last tick joins the chain now: behind the
    // play gate no tick runs (the UI's end calls ensure_packs), so counting
    // it as pending held the first frame, and with it the gate, for good.
    for &id in &engines {
        if let Ok(eng) = engine(id) {
            ensure_packs(&eng);
        }
    }
    let faces: u32 = engines
        .iter()
        .map(|&id| {
            with(id, |s| {
                if !s.missed.is_empty() {
                    (s.pending.len() + s.styling.len()) as u32
                } else {
                    0
                }
            })
        })
        .sum();
    faces
}

pub fn og_i18n_load(world: u32, path: String) -> Result<crate::host::wire::LocaleInfo, String> {
    (|| -> Result<LocaleInfo, String> {
        let eng = engine(world)?;
        let id = og_bundle::runtime::asset_id(&path).to_string();
        let table = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .strings(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        with(world, |s| s.catalog.insert(table));
        apply(&eng)?;
        Ok(info(world))
    })()
}

pub fn og_i18n_set_locale(world: u32, locale: String) -> crate::host::wire::LocaleInfo {
    with(world, |s| s.catalog.request(&[&locale]));
    if let Ok(eng) = engine(world)
        && let Err(e) = apply(&eng)
    {
        note(world, format!("og.i18n.setLocale: {e}"));
    }
    info(world)
}

pub fn og_i18n_locale(world: u32) -> crate::host::wire::LocaleInfo {
    info(world)
}

fn note(world: u32, text: String) {
    let first = with(world, |s| s.warned.insert(text.clone()));
    if first
        && let Ok(eng) = engine(world)
        && let Ok(mut st) = eng.state.try_borrow_mut()
    {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
    }
}

pub fn og_i18n_t(world: u32, key: String, args: Option<String>) -> String {
    let args: BTreeMap<String, Arg> = parse(args.as_deref())
        .ok()
        .and_then(|v| v.as_object().cloned())
        .map(|o| {
            og_core::sort::map(o.into_iter().map(|(k, v)| {
                let a = match &v {
                    Value::Number(n) => Arg::Num(n.as_f64().unwrap_or(0.0)),
                    Value::String(s) => Arg::Str(s.clone()),
                    Value::Bool(b) => Arg::Str(b.to_string()),
                    other => Arg::Str(other.to_string()),
                };
                (k, a)
            }))
        })
        .unwrap_or_default();
    match with(world, |s| s.catalog.t(&key, &args)) {
        Some(text) => shown(world, text),
        None => {
            note(
                world,
                format!(
                    "og.i18n.t: no table has '{key}': add `string \"{key}\" \"...\"` to the base table"
                ),
            );
            shown(world, key)
        }
    }
}

/// The pseudo-locale lengthens the text, as a document's t() does.
fn shown(world: u32, text: String) -> String {
    if og_i18n::is_pseudo_locale(&tag(world)) {
        og_i18n::pseudo_text(&text)
    } else {
        text
    }
}

/// A label written either as a key or as the text itself (a flow node's
/// `og-flow-title`): its translation when a table has it, else the text as
/// written, without a warning.
pub(crate) fn label(world: u32, text: &str) -> String {
    let found = with(world, |s| s.catalog.t(text, &BTreeMap::new()));
    shown(world, found.unwrap_or_else(|| text.to_string()))
}

pub fn og_i18n_has(world: u32, key: String) -> bool {
    with(world, |s| s.catalog.has(&key))
}

pub fn og_i18n_format_number(world: u32, value: f64, format: Option<String>) -> String {
    let f: crate::host::wire::DecimalFormat =
        crate::host::wire::arg(format.as_deref(), "format").unwrap_or_default();
    let decimals = f.decimals.map(|n| n.min(20) as u8);
    let grouping = f.grouping.unwrap_or(true);
    og_i18n::format_decimal(value, decimals, grouping, &tag(world))
}

pub fn og_i18n_format_compact(world: u32, value: f64, decimals: Option<u32>) -> String {
    og_i18n::format::format_compact_with_precision(
        value,
        decimals.map(|n| n.min(20) as u8),
        &tag(world),
    )
}

pub fn og_i18n_format_currency(world: u32, amount: f64, currency: String) -> String {
    og_i18n::format_currency(amount, &currency, &tag(world))
}

pub fn og_i18n_format_date(world: u32, time_ms: f64, format: Option<String>) -> String {
    let f: crate::host::wire::DateFormat =
        crate::host::wire::arg(format.as_deref(), "format").unwrap_or_default();
    let style = og_i18n::DateStyle::named(f.style.as_str()).unwrap_or(og_i18n::DateStyle::Medium);
    let offset = f.offset_minutes.clamp(-1440, 1440);
    og_i18n::format_date(time_ms, offset, style, &tag(world))
}

pub fn og_i18n_format_time(world: u32, time_ms: f64, format: Option<String>) -> String {
    let f: crate::host::wire::TimeFormat =
        crate::host::wire::arg(format.as_deref(), "format").unwrap_or_default();
    let style = og_i18n::TimeStyle::named(f.style.as_str()).unwrap_or(og_i18n::TimeStyle::Short);
    let offset = f.offset_minutes.clamp(-1440, 1440);
    og_i18n::format_time(time_ms, offset, style, &tag(world))
}

pub fn og_i18n_format_duration(world: u32, seconds: f64, style: Option<String>) -> String {
    if style.as_deref() == Some("short") {
        og_i18n::format::format_duration_short(seconds, &tag(world))
    } else {
        og_i18n::format_duration(seconds, &tag(world))
    }
}
