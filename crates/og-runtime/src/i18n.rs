//! The IDL's `i18n` module over og-i18n: an engine's string tables (loaded
//! from the bundle), the locale the host asked for, `t`, and the UI's font
//! chain and direction for the active locale.

use super::engine::{Engine, engine};
use super::{ok, parse, refuse};
use og_core::IdMap;
use og_i18n::{Arg, Catalog, StringTable};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct State {
    catalog: Catalog,
    /// Font paths of the active chain, as last applied to the UI.
    fonts: Vec<String>,
    warned: BTreeSet<String>,
}

thread_local! {
    static STATES: RefCell<IdMap<u32, State>> = const { RefCell::new(IdMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut State) -> T) -> T {
    STATES.with(|s| f(s.borrow_mut().entry(engine).or_default()))
}

pub(crate) fn story_strings(
    engine: u32,
    locale: &str,
    strings: &BTreeMap<String, String>,
) -> Result<(), String> {
    with(engine, |s| s.catalog.extend_base(locale, strings))
}

fn info(engine: u32) -> Value {
    with(engine, |s| {
        let active = s.catalog.active();
        json!({
            "requested": s.catalog.requested().unwrap_or(""),
            "active": active.map_or("", |t| t.locale.as_str()),
            "direction": s.catalog.direction().as_str(),
            "locales": s.catalog.locales(),
            "fonts": s.fonts,
        })
    })
}

/// The active locale's language (its primary subtag, `en` with none): what
/// UI documents format numbers and plurals by.
pub(crate) fn language(engine: u32) -> String {
    {
        let l = tag(engine);
        let primary = l
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if primary.is_empty() {
            "en".to_string()
        } else {
            primary
        }
    }
}

/// The active table's locale, else the requested one (empty: none): what
/// og.i18n's format functions write in.
fn tag(engine: u32) -> String {
    with(engine, |s| {
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
pub(crate) fn request(engine: u32, locale: &str) {
    if !locale.trim().is_empty() {
        with(engine, |s| s.catalog.request(&[locale]));
    }
}

/// Applies the active locale to the UI: its fonts first, then the other
/// tables', each loaded from the bundle; and its direction.
fn apply(eng: &Engine) -> Result<(), String> {
    let (paths, direction) = with(eng.id, |s| {
        let mut paths: Vec<String> = s
            .catalog
            .active()
            .map(|t| t.fonts.clone())
            .unwrap_or_default();
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
    for p in &paths {
        let i = super::assets::load_font(eng, p).map_err(|e| format!("font '{p}': {e}"))?;
        chain.push(super::modules::ui_font(
            eng,
            super::assets::handle(i) as f64,
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
    with(eng.id, |s| s.fonts = paths);
    Ok(())
}

pub fn og_i18n_load(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
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
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_i18n_set_locale(world: u32, locale: String) -> String {
    with(world, |s| s.catalog.request(&[&locale]));
    if let Ok(eng) = engine(world)
        && let Err(e) = apply(&eng)
    {
        note(world, format!("og.i18n.setLocale: {e}"));
    }
    info(world).to_string()
}

pub fn og_i18n_locale(world: u32) -> String {
    info(world).to_string()
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
        Some(text) => text,
        None => {
            note(
                world,
                format!(
                    "og.i18n.t: no table has '{key}': add `string \"{key}\" \"...\"` to the base table"
                ),
            );
            key
        }
    }
}

pub fn og_i18n_has(world: u32, key: String) -> bool {
    with(world, |s| s.catalog.has(&key))
}

pub(crate) fn retire(engine: u32) {
    STATES.with(|s| s.borrow_mut().remove(&engine));
}

pub fn og_i18n_format_compact(world: u32, value: f64) -> String {
    og_i18n::format_compact(value, &tag(world))
}

pub fn og_i18n_format_currency(world: u32, amount: f64, currency: String) -> String {
    og_i18n::format_currency(amount, &currency, &tag(world))
}

/// A format desc's style name and offset.
fn moment(format: Option<&str>) -> (String, i32) {
    let v = parse(format).unwrap_or(Value::Null);
    let style = v["style"].as_str().unwrap_or("").to_string();
    let offset = v["offsetMinutes"]
        .as_f64()
        .map_or(0, |m| m.clamp(-1440.0, 1440.0) as i32);
    (style, offset)
}

pub fn og_i18n_format_date(world: u32, seconds: f64, format: Option<String>) -> String {
    let (style, offset) = moment(format.as_deref());
    let style = og_i18n::DateStyle::named(&style).unwrap_or(og_i18n::DateStyle::Medium);
    og_i18n::format_date(seconds, offset, style, &tag(world))
}

pub fn og_i18n_format_time(world: u32, seconds: f64, format: Option<String>) -> String {
    let (style, offset) = moment(format.as_deref());
    let style = og_i18n::TimeStyle::named(&style).unwrap_or(og_i18n::TimeStyle::Short);
    og_i18n::format_time(seconds, offset, style, &tag(world))
}

pub fn og_i18n_format_duration(world: u32, seconds: f64) -> String {
    og_i18n::format_duration(seconds, &tag(world))
}
