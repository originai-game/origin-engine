use super::*;
use std::collections::BTreeMap;

// ── ui: decoding descs ──────────────────────────────────────────────────────

/// A linear component as the 8-bit sRGB channel og-ui draws, clamped to 0
/// to 1: the API's `Color` is linear RGB (the `conventions` guide §6) and
/// og-ui's [`Srgba`] is a colour as a style authors it. `color.fromHex` of
/// an 8-bit channel comes back as that channel.
pub(crate) fn srgb8(linear: f32) -> u8 {
    let c = linear.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * og_core::math::pow(c, 1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

// ── ui: the calls ───────────────────────────────────────────────────────────

/// The engine whose UI frame is open; outside a frame, logs a warning that
/// `call` does nothing there on the engine that opened one last.
pub(crate) fn building(call: &str) -> Option<std::rc::Rc<Engine>> {
    let building = BUILDING.with(Cell::get);
    let target = building.or_else(|| LAST.with(Cell::get));
    let eng = target.and_then(|id| engine(id).ok())?;
    if building.is_none() {
        if let Ok(mut st) = eng.state.try_borrow_mut() {
            let text = format!(
                "{call} was called outside a system: og.ui calls build the frame the engine opens before the game's systems and closes after them, so they do nothing anywhere else"
            );
            if st.ui_warned.insert(text.clone()) {
                st.log
                    .push(json!({ "tick": 0, "warning": true, "text": text }));
            }
        }
        return None;
    }
    Some(eng)
}

pub fn og_ui_viewport(world: u32) -> crate::host::wire::Viewport {
    // In og-ui units, as the frame lays out (og.ui.frameViewport).
    let css = viewport_of(world);
    let unit = engine(world).map_or(1.0, |e| unit_of(&e, &css));
    viewport_wire(&css.zoomed(unit))
}

pub fn og_ui_theme(world: u32) -> crate::host::wire::UiTheme {
    let settings = match engine(world) {
        Ok(eng) => {
            let v = viewport_of(world);
            super::settings(&eng, &v, super::unit_of(&eng, &v))
        }
        Err(_) => super::UiSettings::of(&Value::Null, String::new(), (false, false)),
    };
    theme_wire(world, &settings)
}

/// A viewport as og.ui.viewport answers it, with its HUD band (at most
/// [`og_ui_model::HUD_MAX_ASPECT`] wide).
fn viewport_wire(v: &Viewport) -> crate::host::wire::Viewport {
    let [small, large] = v.extremes();
    wire::Viewport {
        width: v.width,
        height: v.height,
        dpr: v.dpr,
        safe_area: wire::Insets {
            left: v.safe_area.left,
            top: v.safe_area.top,
            right: v.safe_area.right,
            bottom: v.safe_area.bottom,
        },
        small_width: Some(small[0]),
        small_height: Some(small[1]),
        large_width: Some(large[0]),
        large_height: Some(large[1]),
        coarse_pointer: v.coarse_pointer,
        reduced_motion: Some(v.reduced_motion),
        high_contrast: Some(v.high_contrast),
        covered: Some(rect_wire(&v.covered)),
        breakpoint: match v.breakpoint() {
            og_ui_model::Breakpoint::PhonePortrait => wire::Breakpoint::PhonePortrait,
            og_ui_model::Breakpoint::PhoneLandscape => wire::Breakpoint::PhoneLandscape,
            og_ui_model::Breakpoint::Tablet => wire::Breakpoint::Tablet,
            og_ui_model::Breakpoint::Desktop => wire::Breakpoint::Desktop,
        },
        hud: Some(rect_wire(&v.hud_rect(og_ui_model::HUD_MAX_ASPECT))),
    }
}

pub fn og_ui_frame_viewport() -> String {
    let v = with_frame("og.ui.frameViewport", UiFrame::viewport).unwrap_or_default();
    // The UI kit asks many times a tick (each component that adapts to the
    // surface) and the viewport rarely changes: the last answer is kept with
    // what it was made from, as og_ui_frame_theme's is. Built each call, it
    // was a fifth of a kit frame in the dev build.
    answered(|a| {
        let last = a.map(|a| &mut a.viewport);
        if let Some(Some((lv, text))) = last.as_deref()
            && *lv == v
        {
            return text.clone();
        }
        let text = wire::text_of(&viewport_wire(&v));
        if let Some(last) = last {
            *last = Some((v, text.clone()));
        }
        text
    })
}

/// The frame answers an engine's UI gave last, each with what it was made
/// from (`State::ui_answers`).
#[derive(Default)]
pub(crate) struct UiAnswers {
    viewport: Option<(Viewport, String)>,
    theme: Option<(super::UiSettings, String, String)>,
}

/// Runs `f` on the kept answers of the engine whose UI frame is open (else
/// the one opened last); `None` when there is no such engine or its state
/// is in use, and the answer is then made without being kept.
fn answered<T>(f: impl FnOnce(Option<&mut UiAnswers>) -> T) -> T {
    let eng = ui_engine().and_then(|id| engine(id).ok());
    match eng.as_ref().map(|e| e.state.try_borrow_mut()) {
        Some(Ok(mut st)) => f(Some(&mut st.ui_answers)),
        _ => f(None),
    }
}

pub fn og_ui_frame_cursor() -> String {
    // The last completed frame's: the one the host shows now.
    ui_engine()
        .and_then(|id| engine(id).ok())
        .map_or("default", |eng| crate::host::ui_cursor::shape(&eng).keyword)
        .to_string()
}

pub fn og_ui_frame_pointer() -> crate::host::wire::UiPointer {
    with_frame("og.ui.framePointer", |f| {
        let i = &f.input;
        wire::UiPointer {
            x: i.pointer.x,
            y: i.pointer.y,
            inside: i.pointer_inside,
            down: i.down,
            just_pressed: i.pressed,
            just_released: i.released,
            shift: i.shift,
            ctrl: i.ctrl,
            alt: i.alt,
        }
    })
    .unwrap_or_default()
}

pub fn og_ui_frame_direction() -> String {
    let rtl = with_frame("og.ui.frameDirection", |f| f.rtl).unwrap_or(false);
    if rtl { "rtl" } else { "ltr" }.to_owned()
}

pub fn og_ui_has_icon(name: String) -> bool {
    building("og.ui.hasIcon").is_some_and(|eng| eng.render.borrow().ui_icons.find(&name).is_some())
}

pub fn og_ui_seconds() -> f64 {
    with_frame("og.ui.seconds", UiFrame::seconds).unwrap_or(0.0)
}

pub fn og_ui_frame_theme() -> String {
    let id = ui_engine();
    let settings = id.and_then(|id| engine(id).ok()).map_or_else(
        || super::UiSettings::of(&Value::Null, String::new(), (false, false)),
        |eng| {
            let v = viewport_of(eng.id);
            super::settings(&eng, &v, super::unit_of(&eng, &v))
        },
    );
    // The UI kit asks once a tick and the theme rarely changes: the last
    // answer is kept with the settings (the style among them) and the seeds
    // it was made from.
    let seeds = crate::present::ui_next::configured_seeds(id.unwrap_or(u32::MAX));
    answered(|a| {
        let last = a.map(|a| &mut a.theme);
        if let Some(Some((s, k, text))) = last.as_deref()
            && *s == settings
            && *k == seeds
        {
            return text.clone();
        }
        let text = wire::text_of(&theme_wire(id.unwrap_or(u32::MAX), &settings));
        if let Some(last) = last {
            *last = Some((settings, seeds, text.clone()));
        }
        text
    })
}

/// og.ui.theme's answer: the pack's theme in the settings' scheme and
/// contrast, by its own names, with the settings.
fn theme_wire(engine: u32, s: &super::UiSettings) -> crate::host::wire::UiTheme {
    let theme = crate::present::ui_next::pack_theme(engine, s.dark, s.high_contrast);
    let group = |key: &str| theme.get(key).and_then(Value::as_object);
    // Colours by token name; a nested group (`brand: { 500 }`) as
    // `brand-500`, normalised to hex.
    let mut colors = BTreeMap::new();
    fn flatten(prefix: &str, v: &Value, out: &mut BTreeMap<String, String>) {
        match v {
            Value::String(c) => {
                let c = og_css::parse_color(c).map_or_else(|| c.clone(), hex);
                out.insert(prefix.to_string(), c);
            }
            Value::Object(o) => {
                for (k, v) in o {
                    let name = if k == "DEFAULT" {
                        prefix.to_string()
                    } else {
                        format!("{prefix}-{k}")
                    };
                    flatten(&name, v, out);
                }
            }
            _ => {}
        }
    }
    for (name, v) in group("colors").into_iter().flatten() {
        flatten(name, v, &mut colors);
    }
    let numbers = |key: &str| -> BTreeMap<String, f32> {
        group(key)
            .into_iter()
            .flatten()
            .filter_map(|(k, v)| {
                let n = v.as_f64().or_else(|| v.as_array()?.first()?.as_f64())?;
                Some((k.clone(), n as f32))
            })
            .collect()
    };
    // A font size is `[size, lineHeight]` or `[size, { lineHeight }]`.
    let line_height = group("fontSize")
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| {
            let second = v.as_array()?.get(1)?;
            let lh = second
                .as_f64()
                .or_else(|| second.get("lineHeight")?.as_f64())?;
            Some((k.clone(), lh as f32))
        })
        .collect();
    wire::UiTheme {
        scheme: if s.dark {
            wire::Scheme::Dark
        } else {
            wire::Scheme::Light
        },
        colors,
        radius: numbers("radius"),
        font_size: numbers("fontSize"),
        line_height,
        tracking: numbers("tracking"),
        font_weight: numbers("fontWeight"),
        blur: numbers("blur"),
        durations: (*crate::present::ui_next::durations(engine)).clone(),
        style: s
            .style
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string(),
        contrast: if s.high_contrast {
            wire::Contrast::High
        } else {
            wire::Contrast::Standard
        },
        reduced_motion: s.reduced_motion,
        text_scale: s.text_scale,
        unit: s.unit,
        color_vision: s.color_vision,
        color_vision_filter: s.color_vision_filter,
        color_vision_strength: s.color_vision_strength,
    }
}

/// A colour as `#rrggbb`, or `#rrggbbaa` when it is translucent.
fn hex(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    og_core::color::hex_string([r, g, b, (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8])
}

/// A rectangle as the IDL's `UiRect`.
pub(super) fn rect_wire(r: &og_ui_model::Rect) -> crate::host::wire::UiRect {
    wire::UiRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

/// The engine whose UI frame is open, else the one opened last.
pub(crate) fn ui_engine() -> Option<u32> {
    BUILDING.with(Cell::get).or_else(|| LAST.with(Cell::get))
}
