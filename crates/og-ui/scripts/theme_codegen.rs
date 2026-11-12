//! Writes `src/theme.rs`, the engine's default style, from the design
//! language's tokens (`packages/design/tokens.json`): the 33 colour roles
//! of both schemes, the space, radius, type, density and motion values and
//! the default style's part dressings. `scripts/gen-theme.sh` runs it;
//! `tests/theme.rs` fails when `src/theme.rs` is not exactly its output.

use serde_json::Value;
use std::fmt::Write;

#[path = "theme_css.rs"]
mod theme_css;
pub use theme_css::generate as generate_css;

/// Each og-ui role and the token it is drawn from: `color` (the scheme's
/// colour, `dark.color` over `color`) or `linear` (the fill restated for
/// the UI pass's linear-light blending, `linear.dark.color` over
/// `linear.color`).
const ROLES: &[(&str, &str, &str)] = &[
    ("backdrop", "color", "bg"),
    ("well", "color", "bg-subtle"),
    ("panel", "color", "surface"),
    ("panel_hover", "color", "surface-hover"),
    ("panel_raised", "color", "surface-raised"),
    ("line", "color", "border"),
    ("line_strong", "color", "border-strong"),
    ("ink", "color", "fg"),
    ("ink_muted", "color", "fg-muted"),
    ("ink_faint", "color", "fg-subtle"),
    ("ink_inverse", "color", "fg-inverse"),
    ("control", "linear", "fill"),
    ("control_strong", "linear", "fill-strong"),
    ("primary", "color", "accent"),
    ("primary_hover", "color", "accent-hover"),
    ("primary_active", "color", "accent-active"),
    ("on_primary", "color", "accent-fg"),
    ("primary_soft", "linear", "accent-soft"),
    ("highlight", "color", "accent-ink"),
    ("selection", "color", "border-strong"),
    ("on_selection", "color", "fg"),
    ("accent", "color", "accent-ink"),
    ("scene_ink", "color", "fg"),
    ("scene_plate", "color", "surface-raised"),
    ("scene_shadow", "color", "shadow"),
    ("focus", "color", "focus"),
    ("knob", "color", "knob"),
    ("hud", "color", "hud"),
    ("scrim", "linear", "scrim"),
    ("shadow", "color", "shadow"),
    ("sheen", "color", "sheen"),
    ("success", "color", "success"),
    ("warn", "color", "warn"),
    ("danger", "color", "danger"),
    ("info", "color", "info"),
    ("health", "color", "health"),
    ("mana", "color", "mana"),
    ("xp", "color", "xp"),
    ("gold", "color", "gold"),
];

/// og-ui's type steps and the token scale's names.
const STEPS: &[(&str, &str)] = &[
    ("xl2", "2xl"),
    ("xl3", "3xl"),
    ("xl4", "4xl"),
    ("xl5", "5xl"),
    ("xl6", "6xl"),
    ("xl7", "7xl"),
    ("xl8", "8xl"),
    ("base", "base"),
    ("body", "body"),
    ("caption", "caption"),
    ("display_xl2", "display-2xl"),
    ("display_lg", "display-lg"),
    ("display_md", "display-md"),
    ("display_sm", "display-sm"),
    ("display_xl", "display-xl"),
    ("lg", "lg"),
    ("meta", "meta"),
    ("reading", "reading"),
    ("sm", "sm"),
    ("xl", "xl"),
    ("xs", "xs"),
];

fn get<'a>(v: &'a Value, path: &[&str]) -> Result<&'a Value, String> {
    let mut at = v;
    for k in path {
        at = at
            .get(k)
            .ok_or_else(|| format!("tokens.json has no {}", path.join(".")))?;
    }
    Ok(at)
}

fn text<'a>(v: &'a Value, path: &[&str]) -> Result<&'a str, String> {
    get(v, path)?
        .as_str()
        .ok_or_else(|| format!("tokens.json: {} is not a string", path.join(".")))
}

/// A number with a unit suffix (`12px`, `120ms`, `-0.01em`).
fn unit(v: &Value, path: &[&str], suffix: &str) -> Result<f32, String> {
    let s = text(v, path)?;
    s.strip_suffix(suffix)
        .and_then(|n| n.trim().parse::<f32>().ok())
        .ok_or_else(|| {
            format!(
                "tokens.json: {} is `{s}`, not a number of {suffix}",
                path.join(".")
            )
        })
}

/// `#rrggbb`, `#rrggbbaa` or `rgb(r g b / a)`.
fn colour(s: &str) -> Result<([u8; 3], f32), String> {
    if let Some(h) = s.strip_prefix('#').filter(|h| h.len() == 6) {
        let n = u32::from_str_radix(h, 16).map_err(|_| format!("`{s}` is not a colour"))?;
        return Ok(([(n >> 16) as u8, (n >> 8) as u8, n as u8], 1.0));
    }
    if let Some(h) = s.strip_prefix('#').filter(|h| h.len() == 8) {
        let n = u32::from_str_radix(h, 16).map_err(|_| format!("`{s}` is not a colour"))?;
        let a = f32::from(n as u8) / 255.0;
        return Ok(([(n >> 24) as u8, (n >> 16) as u8, (n >> 8) as u8], a));
    }
    let inner = s
        .strip_prefix("rgb(")
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| format!("`{s}` is not a colour (#rrggbb or rgb(r g b / a))"))?;
    let (rgb, a) = match inner.split_once('/') {
        Some((rgb, a)) => (
            rgb,
            a.trim()
                .parse::<f32>()
                .map_err(|_| format!("`{s}`: alpha"))?,
        ),
        None => (inner, 1.0),
    };
    let c: Vec<u8> = rgb
        .split_whitespace()
        .map(|p| p.parse::<u8>().map_err(|_| format!("`{s}`: a channel")))
        .collect::<Result<_, _>>()?;
    let [r, g, b] = c[..] else {
        return Err(format!("`{s}` has not three channels"));
    };
    Ok(([r, g, b], a))
}

/// A float as Rust source: `14.0`, `0.169`, `-0.01`.
fn num(x: f32) -> String {
    if x == x.trunc() && x.abs() < 1e7 {
        format!("{x:.1}")
    } else {
        format!("{x}")
    }
}

fn bezier(s: &str) -> Result<[f32; 4], String> {
    let inner = s
        .strip_prefix("cubic-bezier(")
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| format!("`{s}` is not a cubic-bezier()"))?;
    let n: Vec<f32> = inner
        .split(',')
        .map(|p| p.trim().parse::<f32>().map_err(|_| format!("`{s}`")))
        .collect::<Result<_, _>>()?;
    let [a, b, c, d] = n[..] else {
        return Err(format!("`{s}` has not four numbers"));
    };
    Ok([a, b, c, d])
}

fn role_colour(t: &Value, dark: bool, source: &str, key: &str) -> Result<([u8; 3], f32), String> {
    let (base, over): (&[&str], &[&str]) = match source {
        "linear" => (&["linear", "color"], &["linear", "dark", "color"]),
        _ => (&["color"], &["dark", "color"]),
    };
    let pick = |p: &[&str]| {
        get(t, p)
            .ok()
            .and_then(|m| m.get(key))
            .and_then(Value::as_str)
    };
    let s = (if dark { pick(over) } else { None })
        .or_else(|| pick(base))
        .ok_or_else(|| format!("tokens.json has no {source} colour `{key}`"))?;
    colour(s)
}

fn palette(t: &Value, dark: bool, out: &mut String) -> Result<(), String> {
    for (field, source, key) in ROLES {
        let (rgb, a) = role_colour(t, dark, source, key)?;
        let [r, g, b] = rgb;
        if a >= 1.0 {
            writeln!(out, "    {field}: c(0x{r:02x}, 0x{g:02x}, 0x{b:02x}),").unwrap();
        } else {
            writeln!(
                out,
                "    {field}: ca(0x{r:02x}, 0x{g:02x}, 0x{b:02x}, {}),",
                num(a)
            )
            .unwrap();
        }
    }
    Ok(())
}

fn density(t: &Value, name: &str, fallback: Option<&str>, out: &mut String) -> Result<(), String> {
    for (field, key) in [
        ("control_lg", "control-lg"),
        ("control_md", "control-md"),
        ("control_sm", "control-sm"),
        ("gutter", "gutter"),
        ("header", "header"),
        ("row", "row"),
        ("section", "section"),
    ] {
        let v = match unit(t, &["density", name, key], "px") {
            Ok(v) => v,
            Err(e) => match fallback {
                Some(f) => unit(t, &["density", f, key], "px").map_err(|_| e)?,
                None => return Err(e),
            },
        };
        writeln!(out, "    {field}: {},", num(v)).unwrap();
    }
    Ok(())
}

/// `src/theme.rs` from `tokens.json`'s text.
pub fn generate(tokens: &str) -> Result<String, String> {
    let t: Value = serde_json::from_str(tokens).map_err(|e| format!("tokens.json: {e}"))?;
    let mut o = String::new();
    o.push_str(HEAD);

    o.push_str("/// The default style's light scheme.\npub const LIGHT: Palette = Palette {\n");
    palette(&t, false, &mut o)?;
    o.push_str("};\n\n/// The default style's dark scheme (the default: games run on a dark\n/// screen more often than not).\npub const DARK: Palette = Palette {\n");
    palette(&t, true, &mut o)?;
    o.push_str("};\n\n");

    o.push_str(SPACE_HEAD);
    for (field, key) in [
        ("xl3", "3xl"),
        ("lg", "lg"),
        ("md", "md"),
        ("sm", "sm"),
        ("xl", "xl"),
        ("xs", "xs"),
        ("xxl", "xxl"),
        ("xxs", "xxs"),
    ] {
        writeln!(o, "    {field}: {},", num(unit(&t, &["space", key], "px")?)).unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(RADIUS_HEAD);
    for (field, key) in [
        ("small", "control"),
        ("control", "button"),
        ("bar", "capsule"),
        ("panel", "panel"),
    ] {
        writeln!(
            o,
            "    {field}: {},",
            num(unit(&t, &["radius", key], "px")?)
        )
        .unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(TYPE_HEAD);
    o.push_str("pub const TYPE_RAMP: TypeRamp = TypeRamp {\n");
    let cjk_tracking = unit(&t, &["font", "cjk", "letterSpacing"], "em")?;
    for (field, key) in STEPS {
        let size = unit(&t, &["font", "scale", key, "size"], "px")?;
        let lh = unit(&t, &["font", "scale", key, "lineHeight"], "px")?;
        let track = match get(&t, &["font", "scale", key, "letterSpacing"]) {
            Ok(_) => unit(&t, &["font", "scale", key, "letterSpacing"], "em")?,
            Err(_) => 0.0,
        };
        let cjk = match get(&t, &["font", "cjk", "lineHeight", key]) {
            Ok(_) => unit(&t, &["font", "cjk", "lineHeight", key], "px")?,
            Err(_) => lh,
        };
        // `fontFeatures` (CSS `font-feature-settings` tags, on only),
        // none when absent.
        let features = match get(&t, &["font", "scale", key, "fontFeatures"]) {
            Ok(v) => {
                let tags = v
                    .as_array()
                    .and_then(|a| a.iter().map(Value::as_str).collect::<Option<Vec<_>>>())
                    .ok_or_else(|| {
                        format!("font.scale.{key}.fontFeatures is not a list of tags")
                    })?;
                og_ui::shape::Features::parse(tags)
                    .map_err(|e| format!("font.scale.{key}.fontFeatures: {e}"))?
                    .bits()
            }
            Err(_) => 0,
        };
        let features = match features {
            0 => "crate::shape::Features::NONE".to_string(),
            b => format!("crate::shape::Features::from_bits({b})"),
        };
        writeln!(
            o,
            "    {field}: TypeStep {{\n        size: {},\n        line_height: {},\n        tracking_em: {},\n        cjk_line_height: {},\n        cjk_tracking_em: {},\n        features: {features},\n        weight: 0,\n        uppercase: false,\n    }},",
            num(size),
            num(lh),
            num(track),
            num(cjk),
            num(cjk_tracking)
        )
        .unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(DENSITY_HEAD);
    o.push_str("/// A mouse and keyboard: the tightest sizes (the tokens' `compact`).\npub const MOUSE: Density = Density {\n");
    density(&t, "compact", None, &mut o)?;
    o.push_str("};\n\n/// A gamepad across a room: larger controls and rows, wider gutters, so\n/// a focused control reads from a couch (the tokens' `pad`).\npub const PAD: Density = Density {\n");
    density(&t, "pad", None, &mut o)?;
    o.push_str("};\n\n/// Touch: every control at least 44 CSS px, a finger's target (the\n/// tokens' `touch`, its gutter and section `compact`'s).\npub const TOUCH: Density = Density {\n");
    density(&t, "touch", Some("compact"), &mut o)?;
    o.push_str("};\n\n");

    o.push_str(DURATIONS_HEAD);
    for k in ["base", "bounce", "fast", "instant", "slow", "spring"] {
        writeln!(
            o,
            "    {k}: {},",
            num(unit(&t, &["motion", "duration", k], "ms")?)
        )
        .unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(TIMING_HEAD);
    let number = |path: &[&str]| -> Result<f32, String> {
        get(&t, path)?
            .as_f64()
            .map(|n| n as f32)
            .ok_or_else(|| format!("tokens.json: {} is not a number", path.join(".")))
    };
    for (field, value) in [
        ("magnet", number(&timing("hover.magnet"))?),
        ("press_damping", number(&timing("press.damping"))?),
        ("cascade_step_ms", unit(&t, &timing("cascade.step"), "ms")?),
        ("cascade_cap_ms", unit(&t, &timing("cascade.cap"), "ms")?),
        ("toast_hold_ms", unit(&t, &timing("toast.hold"), "ms")?),
        (
            "toast_danger_ms",
            unit(&t, &timing("toast.holdDanger"), "ms")?,
        ),
        ("screen_slows", number(&timing("screen.slow"))?),
    ] {
        writeln!(o, "    {field}: {},", num(value)).unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(SPRING_HEAD);
    for (name, key, doc) in [
        (
            "SPRING_DEFAULT",
            "default",
            "Overlays, menus and highlights: no overshoot.",
        ),
        ("SPRING_SHEET", "sheet", "Sheets and drawers."),
        (
            "SPRING_MOMENTUM",
            "momentum",
            "A release that carried velocity (a fling, a throw).",
        ),
    ] {
        let damping = get(&t, &["motion", "spring", key, "damping"])?
            .as_f64()
            .ok_or_else(|| format!("tokens.json: motion.spring.{key}.damping"))?
            as f32;
        let response = unit(&t, &["motion", "spring", key, "response"], "ms")?;
        writeln!(
            o,
            "/// {doc}\npub const {name}: Spring = Spring {{\n    damping: {},\n    response_ms: {},\n}};",
            num(damping),
            num(response)
        )
        .unwrap();
    }
    o.push('\n');
    let fmt4 = |b: [f32; 4]| {
        format!(
            "[{}, {}, {}, {}]",
            num(b[0]),
            num(b[1]),
            num(b[2]),
            num(b[3])
        )
    };
    writeln!(
        o,
        "/// Leaving: cubic Bézier control points (x1, y1, x2, y2).\npub const EASE_EXIT: [f32; 4] = {};\n/// Entering: cubic Bézier control points (x1, y1, x2, y2).\npub const EASE_OUT: [f32; 4] = {};\n",
        fmt4(bezier(text(&t, &["motion", "easing", "exit"])?)?),
        fmt4(bezier(text(&t, &["motion", "easing", "out"])?)?)
    )
    .unwrap();
    let press: f32 = text(&t, &["vars", "press-scale"])?
        .parse()
        .map_err(|_| "tokens.json: vars.press-scale".to_string())?;
    writeln!(
        o,
        "/// How far a pressed control shrinks.\npub const PRESS_SCALE: f32 = {};\n",
        num(press)
    )
    .unwrap();

    o.push_str(STATE_HEAD);
    let state = |k: &str| -> Result<f32, String> {
        get(&t, &["state", k])?
            .as_f64()
            .map(|n| n as f32)
            .ok_or_else(|| format!("tokens.json: state.{k} is not a number"))
    };
    for (field, value) in [
        ("hover_wash", state("hover-wash")?),
        ("control_press_scale", state("control-press-scale")?),
        ("lift", unit(&t, &["state", "lift"], "px")?),
        ("disabled_opacity", state("disabled-opacity")?),
        ("touch_target", unit(&t, &["size", "touch"], "px")?),
    ] {
        writeln!(o, "    {field}: {},", num(value)).unwrap();
    }
    o.push_str("};\n\n");

    for name in ["ui", "panel", "emphasis", "playful"] {
        let bounce = get(&t, &["motion", "named", name, "bounce"])?
            .as_f64()
            .ok_or_else(|| format!("tokens.json: motion.named.{name}.bounce"))?
            as f32;
        writeln!(
            o,
            "/// The `{name}` transition's spring: perceptual duration (ms) and bounce.\npub const MOTION_{}: [f32; 2] = [{}, {}];",
            name.to_uppercase(),
            num(unit(&t, &["motion", "named", name, "duration"], "ms")?),
            num(bounce)
        )
        .unwrap();
    }

    Ok(o)
}

const HEAD: &str = r##"//! The engine's default style: the colour roles of a game UI and the values
//! of the design language the engine shares with origin-agent
//! (`packages/design`, docs/ui.md §3): the neutral ladder, a
//! neutral primary set apart by fill and weight (2026-11-02), hue only where
//! it means something, flat glass on floating chrome only, and borderless
//! controls told apart by tone.
//!
//! **Generated** from `packages/design/tokens.json` by
//! `crates/og-ui/scripts/gen-theme.sh`; never edit it by hand (`tests/theme.rs`
//! fails on drift). The default style is origin-agent's look and the one a
//! game draws in until it picks another (`crate::style_file`, `type style`,
//! usually the one its look names); a game may ship it as it is.
//!
//! Lengths are og-ui units (CSS pixels at a unit scale of 1), durations
//! milliseconds, colours sRGB as authored (straight alpha); the translucent
//! fills are the tokens' `linear` values, restated for the UI pass's
//! linear-light blending. `crate::style` converts them for the renderer.

pub use og_css::Srgba;

const fn c(r: u8, g: u8, b: u8) -> Srgba {
    Srgba {
        rgb: [r, g, b],
        alpha: 1.0,
    }
}

const fn ca(r: u8, g: u8, b: u8, alpha: f32) -> Srgba {
    Srgba { rgb: [r, g, b], alpha }
}

macro_rules! palette {
    ($($(#[doc = $doc:literal])* $field:ident $name:literal,)*) => {
        /// The colour roles of a game UI. Every widget reads its colours
        /// here; a style sets every one of them (`palette { … }`), and
        /// `[color=<role>]`, `{ token: "<role>" }` name them.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct Palette {
            $($(#[doc = $doc])* pub $field: Srgba,)*
        }

        impl Palette {
            /// Every role's name (the IDL's ThemeColor), in this order.
            pub const NAMES: &'static [&'static str] = &[$($name,)*];

            /// A role by its name.
            pub fn get_mut(&mut self, name: &str) -> Option<&mut Srgba> {
                match name {
                    $($name => Some(&mut self.$field),)*
                    _ => None,
                }
            }

            /// A role by its name.
            pub fn get(&self, name: &str) -> Option<Srgba> {
                match name {
                    $($name => Some(self.$field),)*
                    _ => None,
                }
            }
        }
    };
}

palette! {
    /// The screen behind the UI where nothing else is: a menu's ground, a
    /// dialog's surroundings once the scrim is over them.
    backdrop "backdrop",
    /// A recessed ground inside a panel: wells, map frames, empty slots.
    well "well",
    /// Panels and cards.
    panel "panel",
    /// A panel under the pointer (list rows, selectable cards).
    panel_hover "panelHover",
    /// Dialogs, menus, tooltips: what sits above a panel.
    panel_raised "panelRaised",
    /// Hairlines and frames.
    line "line",
    /// Stronger frames: high contrast, map borders, scrollbars.
    line_strong "lineStrong",
    /// The main ink.
    ink "ink",
    /// Secondary ink: captions, keys, inactive tabs.
    ink_muted "inkMuted",
    /// The faintest ink that still reads: placeholders, hints.
    ink_faint "inkFaint",
    /// Ink on a ground of `ink` (a selected chip).
    ink_inverse "inkInverse",
    /// Neutral control fills: secondary buttons, tracks, fields.
    control "control",
    /// A neutral control hovered or pressed; selection.
    control_strong "controlStrong",
    /// The one directed action: primary buttons, a slider's fill, a switch
    /// that is on, the caret.
    primary "primary",
    /// `primary` hovered.
    primary_hover "primaryHover",
    /// `primary` pressed.
    primary_active "primaryActive",
    /// Ink on `primary`.
    on_primary "onPrimary",
    /// A translucent wash of `primary` (a selected row).
    primary_soft "primarySoft",
    /// Emphasised ink: a speaker's name, a price, a link, the player's dot.
    highlight "highlight",
    /// Selected rows, tabs and cards; independent of a primary action.
    selection "selection",
    /// Ink on the selection ground.
    on_selection "onSelection",
    /// Decorative emphasis, independent of selection and primary actions.
    accent "accent",
    /// HUD text drawn directly over the scene.
    scene_ink "sceneInk",
    /// The scheme's plate behind HUD content.
    scene_plate "scenePlate",
    /// The scheme's separation shadow under scene text.
    scene_shadow "sceneShadow",
    /// The focus ring of keyboard and gamepad navigation.
    focus "focus",
    /// Slider and switch knobs.
    knob "knob",
    /// HUD capsules over the game (scores, timers, toasts).
    hud "hud",
    /// Behind a dialog.
    scrim "scrim",
    /// Drop shadows.
    shadow "shadow",
    /// The passing sheen of a skeleton.
    sheen "sheen",
    /// Success.
    success "success",
    /// Warnings.
    warn "warn",
    /// Danger and errors.
    danger "danger",
    /// Information.
    info "info",
    /// A health meter.
    health "health",
    /// A mana meter.
    mana "mana",
    /// An experience meter.
    xp "xp",
    /// Currency: coins, gold, prices that can be paid.
    gold "gold",
}

"##;

const SPACE_HEAD: &str = r#"/// The spacing scale, og-ui units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space {
    pub xl3: f32,
    pub lg: f32,
    pub md: f32,
    pub sm: f32,
    pub xl: f32,
    pub xs: f32,
    pub xxl: f32,
    pub xxs: f32,
}

pub const SPACE: Space = Space {
"#;

const RADIUS_HEAD: &str = r#"/// The corner radii, og-ui units. A style sets them; the default's are
/// the tokens' `control` (small), `button` (controls), `capsule` (bars,
/// round at any height) and `panel`. A radius at or past half an
/// element's short side draws it round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    /// Small parts: checkboxes, slots, tiles, the minimap.
    pub small: f32,
    /// Controls: buttons, fields, chips, badges, tabs, switches.
    pub control: f32,
    /// Bars: progress, meters, slider tracks, scrollbars.
    pub bar: f32,
    /// Panels, cards, dialogs, tooltips.
    pub panel: f32,
}

pub const RADIUS: Radius = Radius {
"#;

const TYPE_HEAD: &str = r#"/// One step of the type ramp: size and leading in og-ui units, Latin
/// tracking in em, and the leading Chinese, Japanese and Korean text takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStep {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub cjk_line_height: f32,
    /// Tracking of text with CJK characters, in em (CSS `letter-spacing`, which spaces Han too); 0 in the tokens.
    pub cjk_tracking_em: f32,
    /// OpenType features the step turns on (small caps, tabular figures;
    /// the tokens' `fontFeatures`), applied where its font ships them.
    pub features: crate::shape::Features,
    /// The weight (100 to 900) the step's text takes the nearest real face
    /// of (a style's `fonts { body 600 "…" }`); 0: the role's own face.
    pub weight: u16,
    /// Locale-sensitive uppercase before measurement; source text is retained.
    pub uppercase: bool,
}

/// The type ramp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeRamp {
    pub xl2: TypeStep,
    pub xl3: TypeStep,
    pub xl4: TypeStep,
    pub xl5: TypeStep,
    pub xl6: TypeStep,
    pub xl7: TypeStep,
    pub xl8: TypeStep,
    pub base: TypeStep,
    /// The default step.
    pub body: TypeStep,
    pub caption: TypeStep,
    pub display_xl2: TypeStep,
    pub display_lg: TypeStep,
    pub display_md: TypeStep,
    pub display_sm: TypeStep,
    pub display_xl: TypeStep,
    pub lg: TypeStep,
    pub meta: TypeStep,
    pub reading: TypeStep,
    pub sm: TypeStep,
    pub xl: TypeStep,
    pub xs: TypeStep,
}

impl TypeRamp {
    /// Every step, to change them all (a text scale).
    pub fn steps_mut(&mut self) -> [&mut TypeStep; 21] {
        let TypeRamp {
            xl2,
            xl3,
            xl4,
            xl5,
            xl6,
            xl7,
            xl8,
            base,
            body,
            caption,
            display_xl2,
            display_lg,
            display_md,
            display_sm,
            display_xl,
            lg,
            meta,
            reading,
            sm,
            xl,
            xs,
        } = self;
        [
            xl2, xl3, xl4, xl5, xl6, xl7, xl8, base, body, caption, display_xl2, display_lg,
            display_md, display_sm, display_xl, lg, meta, reading, sm, xl, xs,
        ]
    }
}

"#;

const DENSITY_HEAD: &str = r#"/// Control, row and header heights and the gutter, og-ui units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density {
    pub control_lg: f32,
    pub control_md: f32,
    pub control_sm: f32,
    pub gutter: f32,
    pub header: f32,
    pub row: f32,
    pub section: f32,
}

"#;

const DURATIONS_HEAD: &str = r#"/// Motion durations, milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Durations {
    pub base: f32,
    pub bounce: f32,
    pub fast: f32,
    pub instant: f32,
    pub slow: f32,
    pub spring: f32,
}

pub const DURATIONS_MS: Durations = Durations {
"#;

/// The path of a row of `motion.timing` (`hover.magnet`).
fn timing(k: &str) -> Vec<&str> {
    ["motion", "timing"]
        .into_iter()
        .chain(k.split('.'))
        .collect()
}

const TIMING_HEAD: &str = r#"/// The timing table's own numbers (the design language's `motion.timing`;
/// its other rows name the durations, eases and springs above). A style's
/// motion block overrides each (docs/ui.md §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    /// How much a pad's virtual cursor slows and settles over a control, 0–1.
    pub magnet: f32,
    /// The press squash's spring back: its damping over `instant`.
    pub press_damping: f32,
    /// A cascade's step between children and the most it spreads over, ms.
    pub cascade_step_ms: f32,
    pub cascade_cap_ms: f32,
    /// How long a toast stays, and an error's toast, ms.
    pub toast_hold_ms: f32,
    pub toast_danger_ms: f32,
    /// A screen transition covers and reveals over this many `slow` durations.
    pub screen_slows: f32,
}

pub const TIMING: Timing = Timing {
"#;

const STATE_HEAD: &str = r#"/// How a control answers (the tokens' `state`; CSS `--state-*`,
/// `--touch-target`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    /// How far hover (half) and press (whole) move a control's fill
    /// towards its wash.
    pub hover_wash: f32,
    /// What a pressed control row shrinks to (a button's part takes
    /// `PRESS_SCALE`).
    pub control_press_scale: f32,
    /// The hover and focus rise, og-ui units (negative is up).
    pub lift: f32,
    /// A disabled control's opacity.
    pub disabled_opacity: f32,
    /// The smallest side a finger's target gets, og-ui units.
    pub touch_target: f32,
}

pub const STATE: State = State {
"#;

const SPRING_HEAD: &str = r#"/// A spring: damping ratio (1 critically damped, below 1 overshoots) and
/// response (the time to reach the target, not a duration), milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub response_ms: f32,
}

"#;
