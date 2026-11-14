//! The IDL's `i18n` module over og-i18n: an engine's string tables (loaded
//! from the bundle), the locale the host asked for, `t`, and the UI's font
//! chain and direction for the active locale.

use crate::engine::{Engine, engine};
use crate::host::wire::LocaleInfo;
use crate::parse;
use og_i18n::{Arg, Catalog, StringTable};
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
    let active = |s: &mut State| s.catalog.active().map(|t| t.locale.clone());
    let moved = with(eng.id, |s| -> Result<bool, String> {
        let before = active(s);
        s.catalog.extend_base(locale, strings)?;
        Ok(before.is_some() && active(s) != before)
    })?;
    if moved {
        apply(eng)?;
    }
    Ok(())
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

/// A key's message in the active locale with no arguments, `None` when no
/// table has it (a document's `key` prop).
pub(crate) fn message(engine: u32, key: &str) -> Option<String> {
    with(engine, |s| s.catalog.t(key, &BTreeMap::new()))
}

/// The host's locale at creation (`og run --locale`, the browser's language).
pub(crate) fn request(eng: &Engine, locale: &str) {
    if !locale.trim().is_empty() {
        eng.modules.i18n.borrow_mut().catalog.request(&[locale]);
    }
}

/// Applies the active locale to the UI: its fonts first, then the other
/// tables', each loaded from the bundle; and its direction.
fn apply(eng: &Engine) -> Result<(), String> {
    let locale = language(eng.id);
    let (paths, direction) = with(eng.id, |s| {
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
    let mut chain = Vec::with_capacity(paths.len());
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
        chain.push(crate::bind::modules::ui_font(
            eng,
            crate::world::assets::handle(i) as f64,
        )?);
    }
    if let Some(ui) = eng.state.borrow().ui.clone() {
        let mut ui = ui.lock();
        let fonts = og_ui::Fonts {
            chain,
            cjk_locale: og_ui::CjkLocale::of_locale(&tag(eng.id)),
            ..ui.fonts().clone()
        };
        ui.set_fonts(fonts);
        ui.set_direction(direction);
        ui.set_locale(&tag(eng.id));
    }
    with(eng.id, |s| {
        s.fonts = drawn;
        s.pending = pending;
    });
    Ok(())
}

/// Reads the bundle's font packs once it is open and applies the locale's
/// chain with them: the first UI frame of a game with packs installed
/// draws in them with no call of its own. Cheap after the first time.
pub(crate) fn ensure_packs(eng: &Engine) {
    if let Some(waiting) = with(eng.id, |s| s.packs.as_ref().map(|_| s.pending.clone())) {
        // A pack face fetched since: the chain takes it.
        let landed = {
            let assets = eng.assets.borrow();
            waiting
                .iter()
                .any(|p| !assets.pending(og_bundle::runtime::asset_id(p)))
        };
        if landed && let Err(e) = apply(eng) {
            note(eng.id, format!("font packs: {e}"));
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
        for id in ids {
            let read = assets.blob(&id).and_then(|(bytes, bundle)| {
                bundle
                    .font_pack_text(&id, bytes.bytes())
                    .map_err(|e| e.to_string())
            });
            match read.and_then(|t| og_i18n::FontPack::parse(&t)) {
                Ok(p) => packs.push(p),
                Err(e) => note(eng.id, format!("font pack {id}: {e}")),
            }
        }
        packs
    };
    let any = !packs.is_empty();
    with(eng.id, |s| s.packs = Some(packs));
    if any && let Err(e) = apply(eng) {
        note(eng.id, format!("font packs: {e}"));
    }
}

/// Of the characters a frame drew that their faces lack, those an
/// installed pack not yet in the chain covers load that pack (the
/// locale's CJK forms first) and are answered as drawn from next frame;
/// the rest are returned, still missing.
pub(crate) fn resolve_missing(
    eng: &Engine,
    missing: Vec<(og_render::FontHandle, Vec<char>)>,
) -> Vec<(og_render::FontHandle, Vec<char>)> {
    let locale = language(eng.id);
    let (out, added) = with(eng.id, |s| {
        let Some(packs) = s.packs.as_deref().filter(|p| !p.is_empty()) else {
            return (missing, false);
        };
        let own = og_i18n::font_pack::for_locale(packs, &locale).map(|p| p.name.clone());
        let order = og_i18n::font_pack::fallback_order(packs, &locale);
        // Packs this frame loads: their characters draw from the next one.
        let mut fresh: Vec<String> = Vec::new();
        let mut out = Vec::new();
        for (face, chars) in missing {
            let mut left = Vec::new();
            for c in chars {
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
            if !left.is_empty() {
                out.push((face, left));
            }
        }
        (out, !fresh.is_empty())
    });
    if added && let Err(e) = apply(eng) {
        note(eng.id, format!("font packs: {e}"));
    }
    out
}

/// Pack faces and glyphs every engine's chain still waits for: the web Worker draws
/// the first frame again until none is left, so a boot never shows a
/// locale's text missing while its pack is fetched.
pub fn fonts_pending() -> u32 {
    let engines = crate::engine::engine_ids();
    let faces: u32 = engines
        .iter()
        .map(|&id| with(id, |s| s.pending.len() as u32))
        .sum();
    // And their glyphs still rendering: the first frame shows the text whole.
    faces
        + engines
            .into_iter()
            .map(crate::present::glyphs::glyphs_pending)
            .sum::<u32>()
}

pub fn og_i18n_load(world: u32, path: String) -> Result<crate::host::wire::LocaleInfo, String> {
    (|| -> Result<LocaleInfo, String> {
        let eng = engine(world)?;
        let id = og_bundle::runtime::asset_id(&path).to_string();
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .strings_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let table = StringTable::parse(&text).map_err(|e| format!("{id}: {e}"))?;
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
    // The pseudo-locale lengthens the text, as a document's t() does.
    let shown = |text: String| {
        if og_i18n::is_pseudo_locale(&tag(world)) {
            og_i18n::pseudo_text(&text)
        } else {
            text
        }
    };
    match with(world, |s| s.catalog.t(&key, &args)) {
        Some(text) => shown(text),
        None => {
            note(
                world,
                format!(
                    "og.i18n.t: no table has '{key}': add `string \"{key}\" \"...\"` to the base table"
                ),
            );
            shown(key)
        }
    }
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
