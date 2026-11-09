//! The default document stylesheet. Independent of DressSet so removing
//! the legacy runtime does not change this generator's output.

use super::{ROLES, colour, role_colour};
use serde_json::Value;
use std::fmt::Write;

/// Each legacy dressing and its single author-facing selector. Keep the
/// mapping next to the declarations: it is also emitted for migration review.
const DRESSINGS: &[(&str, &str, &str)] = &[
    ("Panel::frame", "panel::part(frame)", GLASS_LIFT),
    ("Sheet::frame", "sheet::part(frame)", GLASS_LIFT),
    ("Radial::frame", "radial::part(frame)", GLASS_LIFT),
    ("Menu::frame", "menu::part(frame)", GLASS_FLOAT),
    ("Popover::frame", "popover::part(frame)", GLASS_FLOAT),
    ("Tooltip::frame", "tooltip::part(frame)", GLASS_FLOAT),
    ("Chart::tooltip", "chart::part(tooltip)", GLASS_FLOAT),
    ("Hud::frame", "hud::part(frame)", GLASS_REGULAR),
    (
        "Minimap::frame",
        "minimap::part(frame)",
        "background: #16161899; border: 0.5px solid #ffffff14; box-shadow: 0 6px 16px 0 #00000073;",
    ),
    (
        "ScrollArea::frame",
        "scroll::part(frame)",
        "border: none; box-shadow: none;",
    ),
    ("ActionBar::hint", "actionbar::part(hint)", GLASS_REGULAR),
    (
        "Card::frame",
        "card::part(frame)",
        "box-shadow: none; border-radius: var(--radius-card);",
    ),
    (
        "Button::base",
        "button::part(base)",
        "border: none; box-shadow: none;",
    ),
    (
        "Button::base.ghost",
        "button.ghost::part(base)",
        "border: 1px solid var(--line-strong);",
    ),
    (
        "Field::frame",
        "input::part(frame), textarea::part(frame)",
        "border: none; box-shadow: none;",
    ),
    (
        "Field::frame.invalid",
        "input[aria-invalid=true]::part(frame), textarea[aria-invalid=true]::part(frame)",
        "border: 1px solid var(--danger);",
    ),
    ("Chip::base", "chip::part(base)", "border: none;"),
    (
        "Slot::frame",
        "slot::part(frame)",
        "border: none; box-shadow: none;",
    ),
    (
        "Slot::frame:selected",
        "slot[aria-selected=true]::part(frame)",
        "border: 1px solid #ffffff8c; box-shadow: 0 0 8px 0 #ffffff1f, 0 6px 14px 0 #00000066;",
    ),
    (
        "Meter::track",
        "meter::part(track), progress::part(track)",
        "box-shadow: none;",
    ),
    (
        "Meter::fill",
        "meter::part(fill), progress::part(fill)",
        "box-shadow: none;",
    ),
    (
        "Slider::track",
        "input[type=range]::part(track)",
        "background: #707075; box-shadow: none;",
    ),
    (
        "Toggle::track",
        "input[type=checkbox][switch]::part(track)",
        "box-shadow: none;",
    ),
    (
        "Toggle::row",
        "input[type=checkbox][switch]::part(row)",
        "border: none; box-shadow: none;",
    ),
    (
        "Checkbox::row",
        "input[type=checkbox]::part(row)",
        "border: none; box-shadow: none;",
    ),
    (
        "Radio::option",
        "input[type=radio]::part(option)",
        "border: none; box-shadow: none;",
    ),
    (
        "Dropdown::trigger.open",
        "select[aria-expanded=true]::part(trigger)",
        "border: 2px solid var(--focus);",
    ),
    (
        "Dropdown::option",
        "select::part(option)",
        "border: none; box-shadow: none;",
    ),
    (
        "Radio::card",
        "input[type=radio]::part(card)",
        "border: none;",
    ),
    (
        "Radio::card:selected",
        "input[type=radio]::part(card):checked",
        "background: var(--selection);",
    ),
    (
        "Radio::chosen",
        "input[type=radio]::part(chosen)",
        "border: none;",
    ),
    (
        "Tabs::tab:selected",
        "tabs::part(tab):checked",
        "background: var(--selection); box-shadow: 0 2px 4px 0 #00000073, inset 0 1px 0 0 #ffffff26;",
    ),
];

// The material owns tint/edge/shadows as one shader operation. Repeating
// these as a background or border would composite them twice.
const GLASS_LIFT: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(32px) saturate(var(--glass-saturation)); -og-material: glass thick lift;";
const GLASS_FLOAT: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(32px) saturate(var(--glass-saturation)); -og-material: glass thick float;";
const GLASS_REGULAR: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(22px) saturate(var(--glass-saturation)); -og-material: glass regular float;";

const HEAD: &str = "/* Generated from packages/design/tokens.json by crates/og-ui/scripts/gen-theme.sh.\n\
 * Do not edit: tests/theme.rs checks drift.\n\
 * @layer og-default is the engine's lowest-priority author layer; packs\n\
 * override it in later layers or with unlayered rules. All defaults,\n\
 * including root tokens, belong to this layer. Dark is the initial scheme;\n\
 * the host sets data-scheme=light on the root for the light palette.\n\
 * Selectors use markup tags, host classes/attributes, ::part(name), states.\n\
 * Engine extension: -og-material: glass <weight> <elevation> preserves\n\
 * the renderer's luminance-limited backdrop, lit edge, CSS-space tint,\n\
 * elevation and accessibility solid fallback. Standard backdrop-filter\n\
 * cannot express that material; it supplies blur/saturation, not a second\n\
 * backdrop. The material owns its tint, border and elevation shadows.\n\
 * All --* values below are ordinary CSS variables, not engine hooks.\n\
 * --well-rim is the generated 3:1 unchecked-mark rim in each scheme.\n\
 * --motion-ui-easing samples the existing UI spring at 60 Hz; CSS\n\
 * transitions restart from the current value, not the old spring velocity.\n\
 * No default uses atlas border-image, procedural segments or ornaments.\n\
 * Borders paint inside the existing part box (box-sizing: border-box);\n\
 * focus uses an outside outline and never changes the part's geometry.\n\
 */\n@layer og-default {\n";

fn css_color(rgb: [u8; 3], alpha: f32) -> String {
    let [r, g, b] = rgb;
    format!("rgb({r} {g} {b} / {alpha})")
}

fn kebab(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_uppercase() {
            out.push('-');
            out.extend(c.to_lowercase());
        } else {
            out.push(if c == '_' { '-' } else { c });
        }
    }
    out
}

fn variables(out: &mut String, prefix: &str, value: &Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map.iter().filter(|(key, _)| !key.starts_with('_')) {
                variables(out, &format!("{prefix}-{}", kebab(key)), value);
            }
        }
        Value::String(s) => writeln!(out, "    --{prefix}: {s};").unwrap(),
        Value::Number(n) => writeln!(out, "    --{prefix}: {n};").unwrap(),
        _ => {}
    }
}

fn palette(out: &mut String, tokens: &Value, dark: bool) -> Result<(), String> {
    let mut colors = tokens["color"]
        .as_object()
        .ok_or("tokens.json: color is not an object")?
        .clone();
    if dark {
        colors.extend(
            tokens["dark"]["color"]
                .as_object()
                .ok_or("tokens.json: dark.color is not an object")?
                .clone(),
        );
    }
    variables(out, "color", &Value::Object(colors));
    for (role, source, key) in ROLES {
        let (rgb, a) = role_colour(tokens, dark, source, key)?;
        writeln!(out, "    --{}: {};", kebab(role), css_color(rgb, a)).unwrap();
    }
    // Same contrast rule as dress::well_rim, evaluated from the input
    // tokens rather than the already generated theme compiled into og-ui.
    let c = |key| -> Result<og_ui::Srgba, String> {
        let (rgb, alpha) = role_colour(tokens, dark, "color", key)?;
        Ok(og_ui::Srgba { rgb, alpha }.fade(1.0))
    };
    let mut rim = c("border-strong")?;
    for ground in [c("surface-raised")?, c("surface")?] {
        if rim.contrast(ground) < 3.0 {
            rim = rim.readable_on(ground, 3.0).unwrap_or(c("fg")?);
        }
    }
    writeln!(out, "    --well-rim: {};", css_color(rim.rgb, rim.alpha)).unwrap();
    writeln!(
        out,
        "    --glass-saturation: {};",
        if dark { "120%" } else { "200%" }
    )
    .unwrap();
    // Preserve alpha multiplication (not replacement) used by token effects.
    for (name, key, alpha) in [
        ("shadow-30", "shadow", 0.3),
        ("shadow-35", "shadow", 0.35),
        ("shadow-45", "shadow", 0.45),
        ("sheen-50", "sheen", 0.5),
        ("sheen-60", "sheen", 0.6),
        ("line-70", "border", 0.7),
        ("danger-45", "danger", 0.45),
    ] {
        let (rgb, a) = role_colour(tokens, dark, "color", key)?;
        writeln!(out, "    --{name}: {};", css_color(rgb, a * alpha)).unwrap();
    }
    let ([r, g, b], alpha) = role_colour(tokens, dark, "color", "shadow")?;
    for (name, y, blur, opacity) in [("raised", 1, 3, 0.3), ("floating", 4, 12, 0.35)] {
        let a = (alpha * opacity * 255.0).round() as u8;
        writeln!(
            out,
            "    --elevation-{name}: 0 {y}px {blur}px 0 #{r:02x}{g:02x}{b:02x}{a:02x};"
        )
        .unwrap();
    }
    Ok(())
}

pub fn generate(text: &str) -> Result<String, String> {
    let t: Value = serde_json::from_str(text).map_err(|e| format!("tokens.json: {e}"))?;
    let mut out = String::from(HEAD);
    out.push_str("  :root {\n");
    for group in ["space", "radius", "font", "density", "motion", "vars"] {
        variables(&mut out, group, &t[group]);
    }
    // Preserve the default UI spring's uninterrupted samples using CSS
    // linear(), rather than replacing its response with an arbitrary ease.
    let spring = og_ui::motion::MotionTokens::default().ui.to_tween();
    let mut state = og_tween::SpringState::default();
    let mut samples = vec!["0".to_owned()];
    for _ in 0..600 {
        state = spring.step(state, 1.0, 1.0 / 60.0);
        samples.push(state.value.to_string());
        if og_tween::Spring::settled(state, 1.0) {
            break;
        }
    }
    if !og_tween::Spring::settled(state, 1.0) {
        return Err("default UI spring did not settle within ten seconds".into());
    }
    writeln!(
        out,
        "    --motion-ui-duration: {}ms;",
        (samples.len() - 1) as f32 * 1000.0 / 60.0
    )
    .unwrap();
    writeln!(
        out,
        "    --motion-ui-easing: linear({});",
        samples.join(", ")
    )
    .unwrap();
    palette(&mut out, &t, true)?;
    out.push_str("  }\n  :root[data-scheme=light] {\n");
    palette(&mut out, &t, false)?;
    out.push_str("  }\n");
    // Check all role colours here too; malformed tokens must never produce
    // a partially valid stylesheet.
    for v in t["color"]
        .as_object()
        .ok_or("tokens.json: color is not an object")?
        .values()
    {
        colour(v.as_str().ok_or("tokens.json: color is not a string")?)?;
    }
    out.push_str(FALLBACKS);
    // node_draw draws one focus part per local player. The host exposes
    // that part's player and outline flags as data attributes for matching.
    for host in [
        "button", "input", "textarea", "select", "tabs", "slot", "stepper", "details", "keyboard",
    ] {
        writeln!(out, "  {host}[data-outline=true]::part(focus) {{ outline: 1px solid var(--line-strong); outline-offset: 0; }}").unwrap();
        for (player, color, gap) in [(1, "info", 3), (2, "success", 5), (3, "warn", 7)] {
            writeln!(out, "  {host}[data-player=\"{player}\"]::part(focus):focus-visible {{ outline: var(--space-xxs) solid var(--{color}); outline-offset: calc(var(--space-xxs) * {gap}); }}").unwrap();
        }
    }
    for (old, selector, declarations) in DRESSINGS {
        writeln!(out, "  /* {old} */\n  {selector} {{ {declarations} }}").unwrap();
    }
    out.push_str("}\n");
    Ok(out)
}

// Effective token defaults, before DRESSINGS replaces individual lists.
// Class membership comes from the widget's PartRef, not the markup name.
const FALLBACKS: &str = r#"
  /* Hover/press values are supplied by the widget, blended by the cascade. */
  button::part(base), input[type=checkbox]::part(row), input[type=radio]::part(option),
  select::part(option), stepper::part(button), details::part(summary),
  keyboard::part(key), keyboard::part(candidate), input::part(frame),
  textarea::part(frame), select::part(trigger), input[type=checkbox]::part(box),
  input[type=radio]::part(ring), input[type=radio]::part(card), slot::part(frame),
  chip::part(base), tabs::part(tab), input[type=range]::part(knob),
  input[type=checkbox][switch]::part(track), input[type=checkbox][switch]::part(knob) {
    transition: background var(--motion-ui-duration) var(--motion-ui-easing),
      border-color var(--motion-ui-duration) var(--motion-ui-easing),
      box-shadow var(--motion-ui-duration) var(--motion-ui-easing),
      color var(--motion-ui-duration) var(--motion-ui-easing),
      opacity var(--motion-ui-duration) var(--motion-ui-easing);
  }
  /* Panel / Dialog / Card / Tooltip */
  panel::part(frame), sheet::part(frame), radial::part(frame),
  menu::part(frame), popover::part(frame), tooltip::part(frame),
  chart::part(tooltip), card::part(frame), dialog::part(panel),
  details::part(root), details::part(content), scroll::part(frame),
  select::part(list), input[type=range]::part(bubble) {
    box-sizing: border-box;
    border: 1px solid var(--line);
    border-radius: var(--radius-panel);
  }
  select::part(list), input[type=range]::part(bubble) {
    box-shadow: var(--elevation-floating);
  }
  /* Hud */
  hud::part(frame), actionbar::part(hint), minimap::part(frame),
  joystick::part(base), dpad::part(base), actionbutton::part(face) {
    border: 1px solid var(--line-70);
    border-radius: var(--radius-button);
  }
  /* Button: untouched token defaults also serve secondary controls. */
  button::part(base), input[type=checkbox]::part(row),
  input[type=radio]::part(option), select::part(option),
  stepper::part(button), details::part(summary),
  keyboard::part(key), keyboard::part(candidate) {
    box-sizing: border-box;
    border: 1px solid var(--line);
    border-radius: var(--radius-button);
    box-shadow: inset 0 1px 0 0 var(--sheen-60);
  }
  stepper::part(button):active, details::part(summary):active,
  keyboard::part(key):active, keyboard::part(candidate):active {
    box-shadow: inset 0 1px 3px 0 var(--shadow-45);
  }
  stepper::part(button):disabled, details::part(summary):disabled,
  keyboard::part(key):disabled, keyboard::part(candidate):disabled {
    border: none; box-shadow: none;
  }
  /* Field: text boxes, checkbox marks and radio wells. */
  input::part(frame), textarea::part(frame), select::part(trigger),
  stepper::part(frame), stepper::part(value), input[type=checkbox]::part(box),
  input[type=radio]::part(box), input[type=radio]::part(ring) {
    box-sizing: border-box;
    border: 1.5px solid var(--line);
    border-radius: var(--radius-button);
    box-shadow: inset 0 1px 3px 0 var(--shadow-30);
  }
  input[type=checkbox]::part(box), input[type=radio]::part(box), input[type=radio]::part(ring) {
    border: 1px solid var(--well-rim);
  }
  input[type=checkbox]::part(box):checked,
  input[type=checkbox][aria-checked=mixed]::part(box),
  input[type=radio]::part(box):checked, input[type=radio]::part(ring):checked {
    border: none; box-shadow: none;
  }
  input[type=checkbox][aria-invalid=true]::part(box),
  input[type=radio][aria-invalid=true]::part(box),
  select[aria-invalid=true]::part(trigger),
  stepper[aria-invalid=true]::part(frame), stepper[aria-invalid=true]::part(value) {
    border: 1.5px solid var(--danger);
  }
  /* Slot / Avatar / Chip / Badge / Tab / Bar */
  slot::part(frame) {
    border: 1px solid var(--line);
    border-radius: var(--radius-control);
    box-shadow: inset 0 1px 3px 0 var(--shadow-30);
  }
  avatar::part(frame) { border: 1.5px solid var(--line-strong); border-radius: var(--radius-control); }
  chip::part(base), input[type=range]::part(knob), input[type=checkbox][switch]::part(track),
  input[type=checkbox][switch]::part(knob) {
    border: 1px solid var(--line); border-radius: var(--radius-button);
  }
  chip[aria-selected=true]::part(base) { border: none; }
  badge::part(base), tabs::part(tab), tabs::part(track), tabs::part(indicator) {
    border: none; border-radius: var(--radius-button);
  }
  input[type=radio]::part(card) {
    border: 1px solid var(--line); border-radius: var(--radius-panel);
    box-shadow: var(--elevation-raised);
  }
  input[type=radio]::part(card):active, input[type=radio]::part(card):disabled { box-shadow: none; }
  input[type=radio]::part(ring), input[type=radio]::part(dot),
  input[type=range]::part(knob), input[type=range]::part(core),
  input[type=checkbox][switch]::part(track), input[type=checkbox][switch]::part(knob) {
    border-radius: 9999px;
  }
  meter::part(track), meter::part(fill), progress::part(track), progress::part(fill), input[type=range]::part(track),
  input[type=range]::part(fill), input[type=range]::part(tick) {
    border: none; border-radius: var(--radius-capsule);
    box-shadow: inset 0 1px 2px 0 var(--shadow-35);
  }
  meter::part(fill), progress::part(fill), input[type=range]::part(fill) { box-shadow: inset 0 1px 0 0 var(--sheen-50); }
  /* node_draw focus rings: outside the part, gap only for navigation. */
  button::part(focus):focus-visible, input::part(focus):focus-visible,
  textarea::part(focus):focus-visible, select::part(focus):focus-visible,
  tabs::part(focus):focus-visible, slot::part(focus):focus-visible,
  stepper::part(focus):focus-visible, details::part(focus):focus-visible,
  keyboard::part(focus):focus-visible {
    outline: 2px solid var(--focus); outline-offset: var(--space-xxs);
  }
  input::part(focus):focus-visible, textarea::part(focus):focus-visible { outline-offset: 0; }
  input[type=range]::part(focus):focus-visible, input[type=checkbox]::part(focus):focus-visible,
  input[type=radio]::part(focus):focus-visible { outline-offset: var(--space-xxs); }
"#;
