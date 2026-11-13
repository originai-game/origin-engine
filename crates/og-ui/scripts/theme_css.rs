//! The default CSS stylesheet generated from the design tokens.

use super::{ROLES, colour, role_colour};
use serde_json::Value;
use std::fmt::Write;

/// Each legacy dressing and its single author-facing selector. Keep the
/// mapping next to the declarations: it is also emitted for migration review.
const PART_RULES: &[(&str, &str, &str)] = &[
    ("Panel::frame", "panel.glass::part(frame)", GLASS_LIFT),
    ("Tooltip::frame", "tooltip::part(frame)", GLASS_FLOAT),
    ("Hud::frame", "hud::part(frame)", GLASS_REGULAR),
    (
        "Minimap::frame",
        "minimap::part(frame)",
        "background: var(--color-minimap); border: var(--stroke-edge) solid var(--color-minimap-edge); box-shadow: var(--shadow-minimap);",
    ),
    (
        "Card::frame",
        "panel::part(frame)",
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
        "border: var(--button-border-width) solid var(--line-strong);",
    ),
    (
        "Field::frame",
        "input::part(frame), textarea::part(frame)",
        "border: none; box-shadow: none;",
    ),
    (
        "Field::frame.invalid",
        "input::part(frame):invalid, textarea::part(frame):invalid",
        "border: var(--stroke-hairline) solid var(--danger);",
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
        "border: var(--stroke-hairline) solid var(--color-slot-selected-edge); box-shadow: var(--shadow-slot-selected);",
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
        "background: var(--color-track); box-shadow: none;",
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
        "border: var(--stroke-regular) solid var(--focus); padding-inline: calc(var(--space-md) - var(--stroke-regular));",
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
        "border: var(--stroke-regular) solid var(--primary);",
    ),
    (
        "Tabs::tab:selected",
        "tabs::part(tab):checked",
        "background: var(--selection); box-shadow: var(--shadow-tab-selected);",
    ),
];

/// The built-in components' shared dress. A component names a part by its
/// role (`Shop.frame`, `Friends.row`); every part of a role is dressed once
/// here from the role tokens (packages/design/tokens/role.css), so a style
/// pack restyles all of them by setting values. Each selector sits in
/// `:where()`: no specificity, so a component's or a pack's rule for its
/// own part wins whatever the order.
const ROLE_RULES: &str = r#"  /* Roles: frame, plate, band, title, row, cell, badge, well, figure, label. */
  :where([part$=".frame"]) {
    box-sizing: border-box;
    padding: var(--frame-pad);
    border: var(--frame-rim-width) solid var(--frame-rim);
    border-radius: var(--frame-radius);
    background: var(--frame-fill);
    box-shadow: var(--frame-shadow);
    color: var(--ink);
    font-family: var(--font-body);
  }
  :where([part$=".frame"]:where(.hud)) {
    padding: var(--plate-pad);
    border: var(--plate-rim-width) solid var(--plate-rim);
    border-radius: var(--plate-radius);
    background: var(--plate-fill);
    box-shadow: var(--plate-shadow);
  }
  /* A frame that only lays its children out wears nothing. */
  :where([part$=".frame"]:where(.bare)) { padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
  :where([part$=".band"]) {
    min-height: var(--band-height);
    padding: var(--band-pad);
    border-bottom: var(--band-rule-width) solid var(--band-rule);
    background: var(--band-fill);
    box-shadow: var(--band-shadow);
  }
  :where([part$=".band"]:where(.lead)) { justify-content: flex-start; background: var(--band-lead-fill); }
  :where([part$=".band"]:where(.small)) { min-height: var(--band-small-height); padding: var(--band-small-pad); background: var(--band-small-fill); }
  :where(.band-end.start) { background: var(--band-end-start-image) center / contain no-repeat; }
  :where(.band-end.finish) { background: var(--band-end-finish-image) center / contain no-repeat; }
  :where([part$=".title"]) {
    margin: 0;
    font-family: var(--title-font);
    font-size: var(--title-size);
    font-weight: var(--title-weight);
    line-height: var(--font-leading-tight);
    letter-spacing: var(--title-tracking);
    text-transform: var(--title-transform);
    color: var(--title-ink);
    text-shadow: var(--title-shadow);
  }
  /* A band sizes its title through the inherited token, not a descendant
     selector: a rule with a combinator makes every class change restyle the
     element's whole subtree. */
  :where([part$=".band"]:where(.lead)) { --title-size: var(--title-lead-size); }
  :where([part$=".band"]:where(.small)) { --title-size: var(--title-small-size); }
  :where([part$=".row"]) { border-radius: var(--row-radius); background: var(--row-fill); color: var(--ink); }
  :where([part$=".row"]:hover) { background: var(--row-hover-fill); }
  :where([part$=".row"]:active) { background: var(--row-pressed-fill); }
  :where([part$=".row"]:where([aria-selected=true], [aria-current=true], [data-selected=true])) {
    background: var(--row-selected-fill);
    box-shadow: var(--row-selected-edge);
    color: var(--ink-selected);
  }
  :where([part$=".row"]:where(:disabled, [aria-disabled=true])) { background: var(--row-fill); color: var(--ink-disabled); }
  :where([part$=".row"]:where([data-done=true])) { color: var(--ink-done); }
  :where([part$=".cell"]) {
    box-sizing: border-box;
    border: var(--cell-rim-width) solid var(--cell-rim);
    border-radius: var(--cell-radius);
    background: var(--cell-fill);
    box-shadow: var(--cell-shadow);
    color: var(--ink);
  }
  :where([part$=".cell"]:hover) { box-shadow: var(--cell-hover-shadow); }
  :where([part$=".cell"]:active) { background: var(--cell-pressed-fill); }
  :where([part$=".cell"]:where([aria-selected=true], [aria-current=true], [data-selected=true])) { border-color: var(--cell-selected-rim); }
  :where([part$=".cell"]:where([data-rarity=common])) { background: var(--cell-common-fill); box-shadow: var(--cell-common-shadow); }
  :where([part$=".cell"]:where([data-rarity=uncommon])) { background: var(--cell-uncommon-fill); box-shadow: var(--cell-uncommon-shadow); }
  :where([part$=".cell"]:where([data-rarity=rare])) { background: var(--cell-rare-fill); box-shadow: var(--cell-rare-shadow); }
  :where([part$=".cell"]:where([data-rarity=epic])) { background: var(--cell-epic-fill); box-shadow: var(--cell-epic-shadow); }
  :where([part$=".cell"]:where([data-rarity=legendary])) { background: var(--cell-legendary-fill); box-shadow: var(--cell-legendary-shadow); }
  :where([part$=".cell"]:where([data-rarity=mythic])) { background: var(--cell-mythic-fill); box-shadow: var(--cell-mythic-shadow); }
  :where([part$=".cell"]:where([data-empty=true])) { box-shadow: var(--cell-empty-shadow); }
  :where([part$=".cell"]:where(:disabled, [aria-disabled=true], [data-locked=true])) { box-shadow: var(--cell-empty-shadow); color: var(--ink-disabled); }
  :where([part$=".badge"]) {
    border-radius: var(--badge-radius);
    background: var(--badge-fill);
    box-shadow: var(--badge-shadow);
    color: var(--badge-ink);
    font-family: var(--figure-font);
  }
  :where([part$=".well"]) { border-radius: var(--well-radius); background: var(--well-fill); box-shadow: var(--well-shadow); }
  :where([part$=".figure"]) { font-family: var(--figure-font); color: var(--figure-ink); text-shadow: var(--figure-shadow); }
  :where([part$=".label"], [part$=".heading"]) {
    font-family: var(--label-font);
    letter-spacing: var(--label-tracking);
    text-transform: var(--label-transform);
    color: var(--ink-label);
  }
  :where([data-trend=up]) { color: var(--trend-up); }
  :where([data-trend=down]) { color: var(--trend-down); }
  :where([data-tone=damage]) { color: var(--tone-damage); }
  :where([data-tone=heal]) { color: var(--tone-heal); }
  :where([data-tone=crit]) { color: var(--tone-crit); }
  /* A phone on its side: bands and titles one step down. */
  @media (height < 500px) {
    :where([part$=".band"]) { min-height: var(--band-small-height); padding-top: 0; padding-bottom: 0; }
    :where([part$=".band"]) { --title-size: var(--title-lead-size); }
    :where([part$=".band"]:where(.small)) { --title-size: var(--title-small-size); }
  }
"#;

// The material owns tint/edge/shadows as one shader operation. Repeating
// these as a background or border would composite them twice.
const GLASS_LIFT: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(var(--glass-blur-thick)) saturate(var(--glass-saturation)); -og-material: glass thick lift;";
const GLASS_FLOAT: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(var(--glass-blur-thick)) saturate(var(--glass-saturation)); -og-material: glass thick float;";
const GLASS_REGULAR: &str = "background: none; border: none; box-shadow: none; backdrop-filter: blur(var(--glass-blur-regular)) saturate(var(--glass-saturation)); -og-material: glass regular float;";

const HEAD: &str = "/* Generated from packages/design/tokens/*.css by crates/og-ui/scripts/gen-theme.sh.\n\
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
        .ok_or("tokens: color is not an object")?
        .clone();
    if dark {
        colors.extend(
            tokens["dark"]["color"]
                .as_object()
                .ok_or("tokens: dark.color is not an object")?
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
    let scheme = |group: &str| -> Result<serde_json::Map<String, Value>, String> {
        let mut map = super::get(tokens, &[group])?
            .as_object()
            .ok_or_else(|| format!("tokens: {group} is not an object"))?
            .clone();
        if dark && let Some(over) = tokens["dark"][group].as_object() {
            map.extend(over.clone());
        }
        Ok(map)
    };
    variables(out, "glass", &Value::Object(scheme("glass")?));
    variables(out, "shadow", &Value::Object(scheme("shadow")?));
    variables(out, "shadow", &tokens["derived"]["shadow"]);
    // Preserve alpha multiplication (not replacement) used by token effects.
    for (name, tint) in super::get(tokens, &["derived", "tint"])?
        .as_object()
        .ok_or("tokens: derived.tint is not an object")?
    {
        let key = super::text(tint, &["color"])?;
        let alpha = super::get(tint, &["alpha"])?
            .as_f64()
            .ok_or_else(|| format!("tokens: derived.tint.{name}.alpha"))?
            as f32;
        let (rgb, a) = role_colour(tokens, dark, "color", key)?;
        writeln!(out, "    --{name}: {};", css_color(rgb, a * alpha)).unwrap();
    }
    let ([r, g, b], alpha) = role_colour(tokens, dark, "color", "shadow")?;
    for name in ["raised", "floating"] {
        let y = super::text(tokens, &["elevation", name, "y"])?;
        let blur = super::text(tokens, &["elevation", name, "blur"])?;
        let opacity = super::get(tokens, &["elevation", name, "alpha"])?
            .as_f64()
            .ok_or_else(|| format!("tokens: elevation.{name}.alpha"))? as f32;
        let a = (alpha * opacity * 255.0).round() as u8;
        writeln!(
            out,
            "    --elevation-{name}: 0 {y} {blur} 0 #{r:02x}{g:02x}{b:02x}{a:02x};"
        )
        .unwrap();
    }
    Ok(())
}

pub fn generate(t: &Value) -> Result<String, String> {
    let mut out = String::from(HEAD);
    out.push_str("  :root {\n");
    for group in ["space", "radius", "font", "density", "motion", "vars"] {
        variables(&mut out, group, &t[group]);
    }
    for group in ["size", "measure", "stroke", "state", "opacity", "filter"] {
        variables(&mut out, group, &t[group]);
    }
    // The controls' own tokens are `--<control>-<property>`, no group prefix.
    for (name, value) in super::get(t, &["controls"])?
        .as_object()
        .ok_or("tokens: controls is not an object")?
        .iter()
        .filter(|(name, _)| !name.starts_with('_'))
    {
        match value {
            Value::String(s) => writeln!(out, "    --{name}: {s};").unwrap(),
            v => writeln!(out, "    --{name}: {v};").unwrap(),
        }
    }
    // The roles' tokens are `--<role>-<property>`, no group prefix.
    for (name, value) in super::get(t, &["role"])?
        .as_object()
        .ok_or("tokens: role is not an object")?
    {
        writeln!(out, "    --{name}: {};", value.as_str().unwrap_or_default()).unwrap();
    }
    for (name, value) in super::get(t, &["derived", "alias"])?
        .as_object()
        .ok_or("tokens: derived.alias is not an object")?
    {
        writeln!(out, "    --{name}: {};", value.as_str().unwrap_or_default()).unwrap();
    }
    for (field, _) in super::STEPS {
        let (key, weight, _) = type_role(field);
        let size = super::text(t, &["font", "scale", key, "size"])?;
        let height = super::text(t, &["font", "scale", key, "lineHeight"])?;
        let name = kebab(field);
        writeln!(out, "    --type-{name}-size: {size};\n    --type-{name}-line-height: {height};\n    --type-{name}-weight: {weight};").unwrap();
        let tracking = t["font"]["scale"][key]["letterSpacing"]
            .as_str()
            .unwrap_or("0em");
        let cjk_tracking = super::text(t, &["font", "cjk", "letterSpacing"])?;
        let features = t["font"]["scale"][key]["fontFeatures"]
            .as_array()
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(|tag| format!("\"{tag}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty())
            .unwrap_or("normal".into());
        writeln!(out, "    --type-{name}-tracking: {tracking};\n    --type-{name}-cjk-tracking: {cjk_tracking};\n    --type-{name}-features: {features};\n    --type-{name}-transform: none;").unwrap();
        let cjk_height = super::unit(t, &["font", "scale", key, "lineHeight"], "px")?
            .max((super::unit(t, &["font", "scale", key, "size"], "px")? * 1.45).round());
        writeln!(out, "    --type-{name}-cjk-line-height: {cjk_height}px;").unwrap();
        writeln!(
            out,
            "    --text-{name}: var(--type-{name}-weight) var(--type-{name}-size)/var(--type-{name}-line-height) var(--font-body);"
        )
        .unwrap();
    }
    for (role, step) in [
        ("display", "display-lg"),
        ("title", "display-sm"),
        ("heading", "xl"),
        ("label", "sm"),
        ("numeric", "lg"),
        ("button", "sm"),
        ("hero", "display-xl2"),
    ] {
        for field in [
            "size",
            "line-height",
            "weight",
            "tracking",
            "cjk-tracking",
            "cjk-line-height",
            "features",
            "transform",
        ] {
            writeln!(
                out,
                "    --type-{role}-{field}: var(--type-{step}-{field});"
            )
            .unwrap();
        }
    }
    for (role, font) in [
        ("display", "display"),
        ("title", "display"),
        ("heading", "display"),
        ("body", "body"),
        ("label", "body"),
        ("caption", "body"),
        ("numeric", "mono"),
        ("button", "body"),
        ("hero", "display"),
    ] {
        writeln!(out, "    --type-{role}-font: var(--font-{font});").unwrap();
    }
    // Preserve the default UI spring's uninterrupted samples using CSS
    // linear(), rather than replacing its response with an arbitrary ease.
    let bounce = super::get(t, &["motion", "named", "ui", "bounce"])?
        .as_f64()
        .ok_or("tokens: motion.named.ui.bounce is not a number")? as f32;
    let spring = og_ui::motion::SpringSpec::new(
        super::unit(t, &["motion", "named", "ui", "duration"], "ms")?,
        bounce,
    )
    .to_tween();
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
    palette(&mut out, t, true)?;
    out.push_str("  }\n  :root[data-scheme=light] {\n");
    palette(&mut out, t, false)?;
    out.push_str("  }\n");
    for (query, landscape) in [
        (
            "((width < 600px) or (height < 600px)) and (orientation: portrait)",
            false,
        ),
        (
            "((width < 600px) or (height < 600px)) and (orientation: landscape)",
            true,
        ),
    ] {
        writeln!(out, "  @media {query} {{ :root {{").unwrap();
        for (field, _) in super::STEPS {
            let (key, _, phone) = type_role(field);
            let factor = if landscape {
                (1.0 + phone) / 2.0
            } else {
                phone
            };
            let size = (super::unit(t, &["font", "scale", key, "size"], "px")? * factor)
                .round()
                .max(12.0);
            let height = (super::unit(t, &["font", "scale", key, "lineHeight"], "px")? * factor)
                .round()
                .max(16.0);
            let name = kebab(field);
            let cjk_height = height.max((size * 1.45).round());
            writeln!(
                out,
                "    --type-{name}-size: {size}px; --type-{name}-line-height: {height}px; --type-{name}-cjk-line-height: {cjk_height}px;"
            )
            .unwrap();
        }
        out.push_str("  } }\n");
    }
    // Check all role colours here too; malformed tokens must never produce
    // a partially valid stylesheet.
    for v in t["color"]
        .as_object()
        .ok_or("tokens: color is not an object")?
        .values()
    {
        colour(v.as_str().ok_or("tokens: color is not a string")?)?;
    }
    out.push_str(FALLBACKS);
    // node_draw draws one focus part per local player. The host exposes
    // that part's player and outline flags as data attributes for matching.
    for host in [
        "button", "input", "textarea", "select", "tabs", "slot", "stepper", "details", "keyboard",
    ] {
        writeln!(out, "  {host}[data-outline=true]::part(focus) {{ outline: var(--state-outline-width) solid var(--line-strong); outline-offset: 0; }}").unwrap();
        for (player, color) in [(1, "info"), (2, "success"), (3, "warn")] {
            writeln!(out, "  {host}[data-player=\"{player}\"]::part(focus):focus-visible {{ outline: var(--state-player-ring-width) solid var(--{color}); outline-offset: var(--state-player-ring-offset-{player}); }}").unwrap();
        }
    }
    for (old, selector, declarations) in PART_RULES {
        writeln!(out, "  /* {old} */\n  {selector} {{ {declarations} }}").unwrap();
    }
    out.push_str(ROLE_RULES);
    out.push_str("}\n");
    Ok(out)
}

fn type_role(field: &str) -> (&'static str, u16, f32) {
    match field {
        "xs" | "caption" | "meta" => ("caption", 400, 1.0),
        "sm" => ("body", 600, 1.0),
        "base" | "body" | "reading" => ("body", 400, 1.0),
        "lg" => ("lg", 600, 1.0),
        "xl" | "xl2" => ("xl", 600, 1.0),
        "xl3" | "xl4" | "display_sm" | "display_md" => ("display-sm", 700, 0.875),
        "xl5" | "xl6" | "display_lg" => ("display-lg", 700, 0.8),
        "xl7" | "xl8" | "display_xl" | "display_xl2" => ("display-2xl", 700, 0.75),
        _ => unreachable!("a known type step"),
    }
}

// Default borders and effects, overridden by the more specific part rules.
const FALLBACKS: &str = r#"
  button { color: var(--ink); }
  button.primary { color: var(--on-primary); }
  button.danger { color: var(--danger); }
  button:disabled { color: var(--ink-faint); }
  button::part(base) { background: var(--control); }
  button::part(base):hover, button::part(base):active { background: var(--control-strong); }
  button.primary::part(base) { background: var(--primary); }
  button.primary::part(base):hover { background: var(--primary-hover); }
  button.primary::part(base):active { background: var(--primary-active); }
  button.quiet::part(base), button.ghost::part(base) { background: transparent; }
  button.ghost::part(base) { padding-inline: calc(var(--space-lg) - var(--button-border-width)); }
  button.ghost.small::part(base) { padding-inline: calc(var(--space-md) - var(--button-border-width)); }
  button.ghost.square::part(base) { padding-inline: 0; }
  button.quiet::part(base):hover, button.quiet::part(base):active,
  button.ghost::part(base):hover, button.ghost::part(base):active { background: var(--control); }
  button.danger::part(base) {
    background: color-mix(in srgb, var(--danger) 10%, var(--color-surface));
  }
  button.danger::part(base):hover, button.danger::part(base):active {
    background: color-mix(in srgb, var(--danger) 15%, var(--color-surface));
  }
  button::part(base):disabled,
  button.primary::part(base):disabled, button.quiet::part(base):disabled,
  button.ghost::part(base):disabled, button.danger::part(base):disabled {
    background: var(--control);
  }
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
  panel::part(frame), tooltip::part(frame), dialog::part(panel),
  select::part(list), input[type=range]::part(bubble) {
    box-sizing: border-box;
    border: var(--panel-border-width) solid var(--line);
    border-radius: var(--panel-radius);
  }
  select::part(list), input[type=range]::part(bubble) {
    box-shadow: var(--dropdown-list-shadow);
  }
  /* Hud */
  hud::part(frame), minimap::part(frame) {
    border: var(--stroke-hairline) solid var(--line-70);
    border-radius: var(--hud-radius);
  }
  joystick::part(base):disabled, dpad::part(base):disabled,
  actionbutton::part(face):disabled {
    opacity: var(--state-touch-disabled-opacity);
  }
  /* Button: untouched token defaults also serve secondary controls. */
  button::part(base), input[type=checkbox]::part(row),
  input[type=radio]::part(option), select::part(option),
  stepper::part(button), details::part(summary),
  keyboard::part(key), keyboard::part(candidate) {
    box-sizing: border-box;
    border: var(--button-border-width) solid var(--line);
    border-radius: var(--button-radius);
    box-shadow: var(--button-shadow);
  }
  stepper::part(button):active, details::part(summary):active,
  keyboard::part(key):active, keyboard::part(candidate):active {
    box-shadow: var(--button-pressed-shadow);
  }
  stepper::part(button):disabled, details::part(summary):disabled,
  keyboard::part(key):disabled, keyboard::part(candidate):disabled {
    border: none; box-shadow: none;
  }
  /* Field: text boxes, checkbox marks and radio wells. */
  input::part(frame), textarea::part(frame), select::part(trigger),
  stepper::part(frame), stepper::part(value), input[type=checkbox]::part(box),
  input[type=radio]::part(ring) {
    box-sizing: border-box;
    border: var(--field-border-width) solid var(--line);
    border-radius: var(--field-radius);
    box-shadow: var(--field-shadow);
  }
  select::part(trigger) { padding-inline: calc(var(--space-md) - var(--field-border-width)); }
  input[type=checkbox]::part(box), input[type=radio]::part(ring) {
    border: var(--stroke-hairline) solid var(--well-rim);
  }
  input[type=checkbox]::part(box):checked,
  input[type=checkbox][aria-checked=mixed]::part(box),
  input[type=radio]::part(ring):checked {
    border: none; box-shadow: none;
  }
  input[type=checkbox][aria-invalid=true]::part(box),
  select[aria-invalid=true]::part(trigger),
  stepper[aria-invalid=true]::part(frame), stepper[aria-invalid=true]::part(value) {
    border: var(--field-border-width) solid var(--danger);
  }
  /* Slot / Avatar / Chip / Badge / Tab / Bar */
  slot::part(frame) {
    border: var(--stroke-hairline) solid var(--line);
    border-radius: var(--slot-radius);
    box-shadow: var(--slot-shadow);
  }
  chip::part(base), input[type=range]::part(knob), input[type=checkbox][switch]::part(track),
  input[type=checkbox][switch]::part(knob) {
    border: var(--stroke-hairline) solid var(--line); border-radius: var(--chip-radius);
  }
  chip[aria-selected=true]::part(base) { border: none; }
  badge::part(base), tabs::part(tab), tabs::part(track), tabs::part(indicator) {
    border: none; border-radius: var(--tabs-radius);
  }
  input[type=radio]::part(card) {
    border: var(--stroke-hairline) solid var(--line); border-radius: var(--radio-card-radius);
    box-shadow: var(--radio-card-shadow);
  }
  input[type=radio]::part(card):active, input[type=radio]::part(card):disabled { box-shadow: none; }
  input[type=radio]::part(ring), input[type=radio]::part(dot),
  input[type=range]::part(knob), input[type=range]::part(core),
  input[type=checkbox][switch]::part(track), input[type=checkbox][switch]::part(knob) {
    border-radius: var(--radius-capsule);
  }
  meter::part(track), meter::part(fill), progress::part(track), progress::part(fill), input[type=range]::part(track),
  input[type=range]::part(fill), input[type=range]::part(tick) {
    border: none; border-radius: var(--meter-radius);
    box-shadow: var(--meter-shadow);
  }
  meter::part(fill), progress::part(fill), input[type=range]::part(fill) { box-shadow: var(--meter-fill-shadow); }
  /* node_draw focus rings: outside the part, gap only for navigation. */
  button::part(focus):focus-visible, input::part(focus):focus-visible,
  textarea::part(focus):focus-visible, select::part(focus):focus-visible,
  tabs::part(focus):focus-visible, slot::part(focus):focus-visible,
  stepper::part(focus):focus-visible, details::part(focus):focus-visible,
  keyboard::part(focus):focus-visible {
    outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset);
  }
  input::part(focus):focus-visible, textarea::part(focus):focus-visible { outline-offset: var(--state-focus-ring-offset-field); }
  input[type=range]::part(focus):focus-visible, input[type=checkbox]::part(focus):focus-visible,
  input[type=radio]::part(focus):focus-visible { outline-offset: var(--state-focus-ring-offset); }
"#;
