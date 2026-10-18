//! The IDL's `i18n` module over og-i18n: an engine's string tables (loaded
//! from the bundle), the locale the host asked for, `t`, and the UI's font
//! chain and direction for the active locale.

use super::engine::{Engine, engine};
use super::{ok, parse, refuse};
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
    static STATES: RefCell<BTreeMap<u32, State>> = const { RefCell::new(BTreeMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut State) -> T) -> T {
    STATES.with(|s| f(s.borrow_mut().entry(engine).or_default()))
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
            ..ui.fonts().clone()
        };
        ui.set_fonts(fonts);
        ui.set_direction(direction);
    }
    with(eng.id, |s| s.fonts = paths);
    Ok(())
}

pub fn og_i18n_load(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let id = og_cook::runtime::asset_id(&path).to_string();
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
            o.into_iter()
                .map(|(k, v)| {
                    let a = match &v {
                        Value::Number(n) => Arg::Num(n.as_f64().unwrap_or(0.0)),
                        Value::String(s) => Arg::Str(s.clone()),
                        Value::Bool(b) => Arg::Str(b.to_string()),
                        other => Arg::Str(other.to_string()),
                    };
                    (k, a)
                })
                .collect()
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
