//! The probe's `ui:` commands (material, action), and the hot reload of
//! an edited style in place.

use crate::engine::{Engine, engine};
use crate::host::wire;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The styles one engine loaded: their cooked hashes, by load index (a
/// hot reload re-applies an edited one).
#[derive(Default)]
pub(crate) struct Docs {
    styles: BTreeMap<usize, u64>,
}

pub(crate) type ModuleState = RefCell<Docs>;

fn with<T>(engine: u32, f: impl FnOnce(&mut Docs) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.docs.borrow_mut()),
        // An engine that is gone, or not made yet, has none: a fresh one.
        Err(_) => f(&mut Docs::default()),
    }
}

fn hash_of(eng: &Engine, id: &str) -> u64 {
    let a = eng.assets.borrow();
    a.bundle_for(id)
        .ok()
        .and_then(|(b, _)| b.manifest().outputs().find(|o| o.id == id).map(|o| o.hash))
        .unwrap_or(0)
}

pub(super) fn sheet_faces(
    eng: &Engine,
    faces: &[og_ui::doc::at_rules::FontFace],
    source: &str,
    owner: &str,
) -> Result<Vec<og_ui::FamilyFace>, String> {
    let mut out = Vec::new();
    for face in faces {
        let Some(src) = face
            .src
            .iter()
            .find(|s| !s.starts_with("local:") && eng.assets.borrow().bundle_for(s).is_ok())
        else {
            continue;
        };
        let i = crate::world::assets::load_font(eng, src)
            .map_err(|e| format!("{source}: @font-face {}: {src}: {e}", face.family))?;
        let mut font = super::ui_config::ui_font(eng, crate::world::assets::handle(i) as f64)?;
        font.metrics = face.metrics(font.metrics);
        let weight = |w: f32| w.round().clamp(1.0, 1000.0) as u16;
        // `auto`: a variable font's axis range (CSS Fonts 4 §4.5), else
        // the normal value.
        let axis = |tag: &[u8; 4], normal: f32| {
            font.metrics
                .file
                .as_ref()
                .and_then(|f| {
                    f.axes()
                        .iter()
                        .find(|a| a.tag == *tag)
                        .map(|a| (a.min, a.max))
                })
                .unwrap_or((normal, normal))
        };
        let (w0, w1) = face.weight.unwrap_or_else(|| axis(b"wght", 400.0));
        let (s0, s1) = face.stretch.unwrap_or_else(|| axis(b"wdth", 100.0));
        out.push(og_ui::FamilyFace {
            family: face.family.clone(),
            document: owner.to_owned(),
            weight: (weight(w0), weight(w1)),
            stretch: (
                og_ui::FontQuery::width_tenths(s0),
                og_ui::FontQuery::width_tenths(s1),
            ),
            style: match face.style.as_str() {
                "italic" => og_ui::FaceStyle::Italic,
                "oblique" => og_ui::FaceStyle::Oblique,
                _ => og_ui::FaceStyle::Normal,
            },
            font,
            ..og_ui::FamilyFace::default()
        });
    }
    Ok(out)
}

/// A development build: puts the default style a development cook from a
/// checkout carries (`@ui/base`, the source tree's when it is not the
/// compiled one; og-cook's `ui_components::base_css`) in effect for the
/// process, or the compiled one again when the bundle carries none. A
/// pack-less UI takes its part sheet at once; packs and documents read
/// before keep theirs until they are read again. Answers whether the
/// default style changed.
#[cfg(not(feature = "publish"))]
pub(crate) fn apply_base(eng: &Engine) -> Result<bool, String> {
    const PACK: &str = "@ui/base";
    let css = {
        let assets = eng.assets.borrow();
        match assets.blob(PACK) {
            Ok((bytes, bundle)) => {
                let text = bundle
                    .style_text(PACK, bytes.bytes())
                    .map_err(|e| e.to_string())?;
                let files: Vec<(String, String)> = serde_json::from_str(&text)
                    .map_err(|e| format!("{PACK}: CSS pack sources: {e}"))?;
                files.into_iter().next().map(|(_, css)| css)
            }
            Err(_) => None,
        }
    };
    let changed = og_ui::set_base_css(css.as_deref())
        .map_err(|e| format!("{PACK}: {e} (the default style stays)"))?;
    if changed
        && let Some(ui) = eng.state.borrow().ui.clone()
        && ui.lock().styled_by(None)
    {
        ui.lock().reset_part_css();
    }
    Ok(changed)
}

/// After a hot reload (`og dev`): every style the UI is configured with
/// that changed is applied again, the UI's state kept. In a development
/// build an edited default style ([`apply_base`]) reads every style again.
/// `fresh_ui`: the UI is a new one (a hot reload's rebuilt world), so the
/// configured style is applied to it whatever changed. Each step's time
/// goes to `laps`.
pub(crate) fn refresh(
    eng: &Engine,
    fresh_ui: bool,
    laps: &mut Vec<(&'static str, f64)>,
) -> Vec<String> {
    let mut failed = Vec::new();
    if eng.state.borrow().ui.is_none() {
        return failed;
    }
    let mut lap = {
        let mut t = crate::author::probe::now_ms();
        move |laps: &mut Vec<(&'static str, f64)>, name| {
            let now = crate::author::probe::now_ms();
            laps.push((name, now - t));
            t = now;
        }
    };
    #[cfg(not(feature = "publish"))]
    let rebased = apply_base(eng).unwrap_or_else(|e| {
        failed.push(format!("og dev: {e}"));
        false
    });
    #[cfg(feature = "publish")]
    let rebased = false;
    lap(laps, "base");
    // Styles: an edited one is parsed again and the UI configured again.
    let styles: Vec<(usize, String)> = eng
        .assets
        .borrow()
        .styles
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.id.clone()))
        .collect();
    let mut changed = false;
    for (i, id) in styles {
        let now = hash_of(eng, &id);
        let before = with(eng.id, |d| d.styles.get(&i).copied());
        if before.is_none_or(|b| b == now) && !rebased {
            continue;
        }
        match super::ui_config::read_style(eng, &id) {
            Ok(style) => {
                eng.assets.borrow_mut().styles[i] = style;
                with(eng.id, |d| d.styles.insert(i, now));
                changed = true;
            }
            Err(e) => failed.push(format!("og dev: {e} (the old style stays)")),
        }
    }
    lap(laps, "styles");
    if (changed || (fresh_ui && configured_style(eng).is_some()))
        && let Err(reason) = super::og_ui_configure(eng.id, Some("{}".into()))
    {
        failed.push(format!("og dev: the style did not apply: {reason}"));
    }
    failed
}

/// The style the last og.ui.configure named (its UiDesc, as configure
/// merged and read it).
fn configured_style(eng: &Engine) -> Option<u64> {
    let st = eng.state.borrow();
    wire::value::<wire::UiDesc>(&st.ui_desc, "desc").ok()?.style
}

/// The style hashes as loaded (so the first refresh compares against them).
pub(crate) fn note_styles(eng: &Engine) {
    let ids: Vec<(usize, String)> = eng
        .assets
        .borrow()
        .styles
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.id.clone()))
        .collect();
    for (i, id) in ids {
        let h = hash_of(eng, &id);
        with(eng.id, |d| {
            d.styles.entry(i).or_insert(h);
        });
    }
}

// ── live edits (Tune and the probe) ─────────────────────────────────────────

// ── the probe's ui: commands ────────────────────────────────────────────────

const USAGE: &str = "ui:material:<asset>.<parameter>=<numbers> | ui:action:<name>";

pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "ui",
        og_probe::CommandDesc {
            usage: USAGE.into(),
            doc: "adjust the UI live (docs/ui.md §6): set a UI material's parameter, fire a UI action".into(),
        },
        move |_world, args| command(engine_id, args),
    );
}

fn command(engine_id: u32, args: &[String]) -> Result<(), String> {
    let verb = args.first().map(String::as_str).unwrap_or("");
    let rest = args.get(1..).map(|a| a.join(":")).unwrap_or_default();
    let eng = engine(engine_id)?;
    match verb {
        "material" => {
            let (target, value) = rest.split_once('=').ok_or_else(|| {
                format!("write ui:material:<asset>.<parameter>=<numbers>; {USAGE}")
            })?;
            let (path, parameter) = target
                .rsplit_once('.')
                .filter(|(p, n)| !p.is_empty() && !n.is_empty())
                .ok_or_else(|| format!("'{target}': write <asset>.<parameter>; {USAGE}"))?;
            crate::present::ui_draw::tune_material(&eng, path, parameter, value)
        }
        "action" => {
            // The tree takes it at the next tick's end, as a fired action.
            eng.render
                .borrow_mut()
                .ui_next
                .get_or_insert_with(|| Box::new(crate::present::ui_next::Next::new()))
                .fired
                .push(rest);
            Ok(())
        }
        _ => Err(format!("'{verb}' is not a ui command; {USAGE}")),
    }
}
