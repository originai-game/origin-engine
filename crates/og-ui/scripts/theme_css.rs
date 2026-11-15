//! The default CSS stylesheet generated from the design tokens.

use super::{ROLES, role_colour};
use serde_json::Value;
use std::fmt::Write;

/// Each legacy dressing and its single author-facing selector. Keep the
/// mapping next to the declarations: it is also emitted for migration review.
const PART_RULES: &[(&str, &str, &str)] = &[
    ("Panel::frame", "panel.glass::part(frame)", GLASS_LIFT),
    ("Tooltip::frame", "tooltip::part(frame)", GLASS_FLOAT),
    ("Hud::frame", "hud::part(frame)", GLASS_REGULAR),
    (
        "Card::frame",
        "panel::part(frame)",
        "box-shadow: var(--panel-shadow, none); border-radius: var(--radius-card);",
    ),
    (
        "Button::base",
        "button::part(base)",
        "border-width: var(--button-rim-width, 0px); border-style: solid; border-color: var(--button-rim, var(--line)); \
         box-shadow: var(--button-base-shadow, none); corner-shape: var(--button-corner-shape); \
         border-image-source: var(--button-image, none); \
         border-image-slice: var(--button-image-slice, 100%); \
         border-image-width: var(--button-image-width, 1); \
         font-family: var(--button-font, revert-layer); \
         font-size: var(--button-font-size, revert-layer); \
         letter-spacing: var(--button-tracking, revert-layer); \
         text-transform: var(--button-transform, revert-layer); \
         text-shadow: var(--button-text-shadow, revert-layer); \
         min-height: var(--button-min-height, revert-layer);",
    ),
    (
        "Button::base.ghost",
        "button.ghost::part(base)",
        "border-width: var(--button-border-width); border-style: solid; border-color: var(--button-ghost-rim); border-image-source: none;",
    ),
    (
        "Field::frame",
        "input::part(frame), textarea::part(frame)",
        "border-width: var(--field-rim-width, 0px); border-style: solid; border-color: var(--field-rim, var(--line)); \
         box-shadow: var(--field-frame-shadow, none);",
    ),
    (
        "Field::frame.invalid",
        "input::part(frame):invalid, textarea::part(frame):invalid",
        "border-width: var(--stroke-hairline); border-style: solid; border-color: var(--field-invalid-rim);",
    ),
    (
        "Slot::frame",
        "slot::part(frame)",
        "background: var(--slot-fill, revert-layer); \
         border-width: var(--slot-rim-width, 0px); border-style: solid; border-color: var(--slot-rim, var(--line)); \
         box-shadow: var(--slot-shadow); corner-shape: var(--slot-corner-shape);",
    ),
    (
        "Slot::frame:selected",
        "slot[aria-selected=true]::part(frame)",
        "border-width: var(--stroke-hairline); border-style: solid; border-color: var(--slot-selected-rim); box-shadow: var(--slot-selected-shadow);",
    ),
    (
        "Meter::track",
        "meter::part(track), progress::part(track)",
        "background: var(--meter-track-fill, revert-layer); \
         border-width: var(--meter-track-rim-width, 0px); border-style: solid; border-color: var(--meter-track-rim, transparent); \
         box-shadow: var(--meter-track-shadow, none);",
    ),
    (
        "Meter::fill",
        "meter::part(fill), progress::part(fill)",
        "box-shadow: none;",
    ),
    (
        "Slider::track",
        "input[type=range]::part(track)",
        "background: var(--slider-track-fill); box-shadow: var(--slider-track-shadow, none); \
         border-width: var(--slider-track-rim-width, 0px); border-style: solid; border-color: var(--slider-track-rim, transparent);",
    ),
    (
        "Toggle::track",
        "input[type=checkbox][switch]::part(track)",
        "box-shadow: var(--switch-track-shadow, none);",
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
        "border-width: var(--stroke-regular); border-style: solid; border-color: var(--field-focus-rim, var(--focus)); padding-inline: calc(var(--space-md) - var(--stroke-regular));",
    ),
    (
        "Dropdown::option",
        "select::part(option)",
        "border: none; box-shadow: none;",
    ),
    (
        "Radio::card",
        "input[type=radio]::part(card)",
        "border-width: var(--radio-card-rim-width, 0px); border-style: solid; border-color: var(--radio-card-rim, var(--line)); \
         background: var(--radio-card-fill, revert-layer); corner-shape: var(--radio-card-corner-shape);",
    ),
    (
        "Radio::chosen",
        "input[type=radio]::part(chosen)",
        "border-width: var(--stroke-regular); border-style: solid; border-color: var(--radio-chosen-rim); background: var(--radio-chosen-fill, revert-layer);",
    ),
    (
        "Tabs::tab:selected",
        "tabs::part(tab):checked",
        "background: var(--tabs-tab-selected-fill); box-shadow: var(--tabs-tab-selected-shadow); \
         color: var(--tabs-tab-selected-ink, revert-layer);",
    ),
];

/// The built-in components' shared dress. A component names a part by its
/// role (`Shop.frame`, `Friends.row`); every part of a role is dressed once
/// here from the role tokens (packages/design/tokens/role.css), so a style
/// pack restyles all of them by setting values. Each selector sits in
/// `:where()`: no specificity, so a component's or a pack's rule for its
/// own part wins whatever the order.
const ROLE_RULES: &str = r#"  /* Roles: frame, plate, band, title, row, cell, badge, well, figure, label,
     heading, name, picture, tab. */
  :where([part$=".frame"]) {
    box-sizing: border-box;
    padding: var(--frame-pad);
    border: var(--frame-rim-width) solid var(--frame-rim);
    border-radius: var(--frame-radius); corner-shape: var(--frame-corner-shape);
    background: var(--frame-fill);
    box-shadow: var(--frame-shadow);
    color: var(--ink);
    font-family: var(--font-body);
  }
  :where([part$=".frame"]:where(.hud)) {
    padding: var(--plate-pad);
    border: var(--plate-rim-width) solid var(--plate-rim);
    border-radius: var(--plate-radius); corner-shape: var(--plate-corner-shape);
    background: var(--plate-fill);
    box-shadow: var(--plate-shadow);
  }
  /* A frame that only lays its children out wears nothing. */
  :where([part$=".frame"]:where(.bare)) { padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
  /* Frost: a frame blurs the scene behind it (`--frame-glass`, a plate
     `--plate-glass`); a bare frame and one inside another do not. */
  :where([part$=".frame"]) { backdrop-filter: var(--frame-glass, none); }
  :where([part$=".frame"]:where(.hud)) { backdrop-filter: var(--plate-glass, none); }
  :where([part$=".frame"]:where(.bare), [part$=".frame"] [part$=".frame"]) { backdrop-filter: none; }
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
  :where([part$=".row"]) { border-radius: var(--row-radius); corner-shape: var(--row-corner-shape); background: var(--row-fill); color: var(--ink); }
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
    border-radius: var(--cell-radius); corner-shape: var(--cell-corner-shape);
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
    border-radius: var(--badge-radius); corner-shape: var(--badge-corner-shape);
    background: var(--badge-fill);
    box-shadow: var(--badge-shadow);
    color: var(--badge-ink);
    font-family: var(--figure-font);
  }
  :where([part$=".well"]) { border-radius: var(--well-radius); corner-shape: var(--well-corner-shape); background: var(--well-fill); box-shadow: var(--well-shadow); }
  :where([part$=".figure"]) { font-family: var(--figure-font); color: var(--figure-ink); text-shadow: var(--figure-shadow); }
  :where([part$=".label"], [part$=".heading"]) {
    font-family: var(--label-font);
    letter-spacing: var(--label-tracking);
    text-transform: var(--label-transform);
    color: var(--ink-label);
  }
  /* A name (a player's, an item's, a character's), unset in the default
     style: the component's own type stands. */
  :where([part$=".name"]) {
    font-family: var(--name-font, revert-layer);
    font-size: var(--name-size, revert-layer);
    letter-spacing: var(--name-tracking, revert-layer);
    text-transform: var(--name-transform, revert-layer);
    color: var(--name-ink, revert-layer);
    text-shadow: var(--name-shadow, revert-layer);
  }
  :where([part$=".name"]:where([aria-selected=true], [aria-current=true], [data-selected=true])) {
    color: var(--name-selected-ink, var(--name-ink, revert-layer));
  }
  /* A picture (a portrait, an avatar, a jacket, art), unset by default. */
  :where([part$=".picture"]) {
    background: var(--picture-fill, revert-layer);
    border: var(--picture-rim-width, 0px) solid var(--picture-rim, transparent);
    border-radius: var(--picture-radius, revert-layer);
    box-shadow: var(--picture-shadow, revert-layer);
  }
  /* A component's tab wears the core tabs' hooks. */
  :where([part$=".tab"]) {
    background: var(--tabs-tab-fill, revert-layer);
    color: var(--tabs-tab-ink, revert-layer);
    font-family: var(--tabs-font, revert-layer);
    letter-spacing: var(--tabs-tracking, revert-layer);
    text-transform: var(--tabs-transform, revert-layer);
  }
  :where([part$=".tab"]:hover) { background: var(--tabs-tab-hover-fill, var(--tabs-tab-fill, revert-layer)); }
  :where([part$=".tab"]:where([aria-selected=true], [aria-current=true], [data-selected=true])) {
    background: var(--tabs-tab-selected-fill, revert-layer);
    color: var(--tabs-tab-selected-ink, revert-layer);
  }
  /* A role part on a team (`data-team=a|b`) wears the team's colour as its
     ink and rim. */
  :where(:is([part$=".frame"], [part$=".plate"], [part$=".band"], [part$=".title"], [part$=".row"], [part$=".cell"],
              [part$=".badge"], [part$=".well"], [part$=".figure"], [part$=".label"], [part$=".heading"],
              [part$=".name"], [part$=".picture"], [part$=".tab"])[data-team=a]) {
    color: var(--team1); border-color: var(--team1);
  }
  :where(:is([part$=".frame"], [part$=".plate"], [part$=".band"], [part$=".title"], [part$=".row"], [part$=".cell"],
              [part$=".badge"], [part$=".well"], [part$=".figure"], [part$=".label"], [part$=".heading"],
              [part$=".name"], [part$=".picture"], [part$=".tab"])[data-team=b]) {
    color: var(--team2); border-color: var(--team2);
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
// A pack's `--*-fill` and `--*-glass` (`none`: a solid surface) replace them.
const GLASS_LIFT: &str = "background: var(--panel-fill, none); border: none; box-shadow: var(--panel-shadow, none); backdrop-filter: var(--panel-glass, blur(var(--glass-blur-thick)) saturate(var(--glass-saturation))); -og-material: glass thick lift;";
const GLASS_FLOAT: &str = "background: var(--tooltip-fill, none); border-width: var(--tooltip-rim-width, 0px); border-style: solid; border-color: var(--tooltip-rim, var(--line)); border-radius: var(--tooltip-radius); corner-shape: var(--tooltip-corner-shape); box-shadow: var(--tooltip-shadow, none); backdrop-filter: var(--tooltip-glass, blur(var(--glass-blur-thick)) saturate(var(--glass-saturation))); -og-material: glass thick float;";
const GLASS_REGULAR: &str = "background: var(--hud-fill, none); border-width: var(--hud-rim-width, 0px); border-style: solid; border-color: var(--hud-rim, var(--line-70)); box-shadow: var(--hud-shadow, none); backdrop-filter: var(--hud-glass, blur(var(--glass-blur-regular)) saturate(var(--glass-saturation))); -og-material: glass regular float;";

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
 * --well-rim is each scheme's 3:1 unchecked-mark rim (--color-well-rim).\n\
 * --motion-ui-easing samples the existing UI spring at 60 Hz; CSS\n\
 * transitions restart from the current value, not the old spring velocity.\n\
 * No default uses atlas border-image, procedural segments or ornaments.\n\
 * Borders paint inside the existing part box (box-sizing: border-box);\n\
 * focus uses an outside outline and never changes the part's geometry.\n\
 */\n@layer og-default {\n";

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
    // The seeds and the colours derived from them stay expressions: a seed
    // a pack or og.ui.configure sets restyles every colour that reads it.
    variables(out, "seed", &Value::Object(scheme("seed")?));
    variables(out, "color", &Value::Object(scheme("color")?));
    for (role, source, key) in ROLES {
        let value = match *source {
            // The linear fills are not variables of their own: the role
            // takes the scheme's authored value.
            "linear" => {
                let fills = &tokens["linear"];
                let pick = |m: &Value| m["color"][*key].as_str().map(str::to_owned);
                (if dark { pick(&fills["dark"]) } else { None })
                    .or_else(|| pick(fills))
                    .ok_or_else(|| format!("the tokens have no linear colour `{key}`"))?
            }
            _ => format!("var(--color-{key})"),
        };
        writeln!(out, "    --{}: {value};", kebab(role)).unwrap();
    }
    writeln!(out, "    --well-rim: var(--color-well-rim);").unwrap();
    for k in 1..=8 {
        writeln!(out, "    --team{k}: var(--color-team{k});").unwrap();
    }
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
            .ok_or_else(|| format!("tokens: derived.tint.{name}.alpha"))?;
        writeln!(
            out,
            "    --{name}: rgb(from var(--color-{key}) r g b / calc(alpha * {alpha}));"
        )
        .unwrap();
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
    for dark in [false, true] {
        for key in t["color"]
            .as_object()
            .ok_or("tokens: color is not an object")?
            .keys()
        {
            role_colour(t, dark, "color", key)?;
        }
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
  /* Control hooks (docs/ui.md §3.1): every value a style pack restyles a
     core control with is a token. One the default style declares (state.css)
     is the value it dresses with; one it does not falls back to the hook it
     refines (`--button-hover-rim` to `--button-rim`) and last to the
     control's own value (`revert-layer`), so the default style is unchanged
     until a pack sets it. A rim a control does not draw today takes a
     `-rim-width` beside its colour, as the role tokens do. */
  button { color: var(--button-ink); }
  button.primary { color: var(--button-primary-ink); }
  button.danger { color: var(--button-danger-ink); }
  button.ghost { color: var(--button-ghost-ink); }
  button[aria-pressed=true] { color: var(--button-selected-ink); }
  button:disabled { color: var(--button-disabled-ink); }
  button.primary:disabled { color: var(--button-primary-disabled-ink, var(--button-disabled-ink)); }
  /* A button's padding: its inline padding by size, or the whole shorthand,
     and the face's when held (a label that sinks with it). */
  button::part(base) { padding: var(--button-pad, 0 var(--button-pad-inline, var(--space-lg))); }
  button.small::part(base) { padding: var(--button-pad, 0 var(--button-pad-inline, var(--space-md))); }
  button::part(base):active {
    padding: var(--button-pressed-pad, var(--button-pad, 0 var(--button-pad-inline, var(--space-lg))));
  }
  button.small::part(base):active {
    padding: var(--button-pressed-pad, var(--button-pad, 0 var(--button-pad-inline, var(--space-md))));
  }
  button::part(base) { background: var(--button-fill); }
  button::part(base):hover {
    background: var(--button-hover-fill);
    color: var(--button-hover-ink, revert-layer);
    border-color: var(--button-hover-rim, var(--button-rim, var(--line)));
    box-shadow: var(--button-hover-shadow, var(--button-base-shadow, none));
    border-image-source: var(--button-image-hover, var(--button-image, none));
  }
  button::part(base):active {
    background: var(--button-pressed-fill);
    color: var(--button-pressed-ink, revert-layer);
    border-color: var(--button-pressed-rim, var(--button-hover-rim, var(--button-rim, var(--line))));
    box-shadow: var(--button-base-pressed-shadow, var(--button-base-shadow, none));
    border-image-source: var(--button-image-pressed, var(--button-image, none));
  }
  /* Spin cells sit inside the field's border, the last one at its end. */
  input[type=number]::part(button) {
    background: var(--stepper-button-fill, var(--control)); margin-block: var(--field-border-width);
  }
  input[type=number].inc::part(button) { margin-inline-end: var(--field-border-width); }
  input[type=number]::part(button):hover {
    background: var(--stepper-button-hover-fill, var(--stepper-button-fill, var(--control-strong)));
  }
  input[type=number]::part(button):active {
    background: var(--stepper-button-pressed-fill, var(--stepper-button-fill, var(--control-strong)));
  }
  input[type=number]::part(button):disabled {
    background: var(--stepper-button-disabled-fill, var(--stepper-button-fill, var(--control)));
  }
  stepper::part(button) { background: var(--stepper-button-fill, revert-layer); }
  stepper::part(button):hover { background: var(--stepper-button-hover-fill, var(--stepper-button-fill, revert-layer)); }
  stepper::part(button):active { background: var(--stepper-button-pressed-fill, var(--stepper-button-fill, revert-layer)); }
  stepper::part(button):disabled { background: var(--stepper-button-disabled-fill, var(--stepper-button-fill, revert-layer)); }
  stepper::part(button), input[type=number]::part(button) {
    color: var(--stepper-button-ink, revert-layer);
    border-color: var(--stepper-button-rim, var(--line));
  }
  stepper::part(button):hover, input[type=number]::part(button):hover {
    border-color: var(--stepper-button-hover-rim, var(--stepper-button-rim, var(--line)));
  }
  button.primary::part(base) {
    background: var(--button-primary-fill);
    border-color: var(--button-primary-rim, var(--button-rim, var(--line)));
    box-shadow: var(--button-primary-shadow, var(--button-base-shadow, none));
    border-image-source: var(--button-primary-image, var(--button-image, none));
  }
  button.primary::part(base):hover {
    background: var(--button-primary-hover-fill);
    color: var(--button-primary-hover-ink, revert-layer);
    border-color: var(--button-primary-hover-rim, var(--button-primary-rim, var(--button-hover-rim, var(--button-rim, var(--line)))));
    box-shadow: var(--button-primary-shadow, var(--button-hover-shadow, var(--button-base-shadow, none)));
    border-image-source: var(--button-primary-image-hover, var(--button-primary-image, var(--button-image-hover, var(--button-image, none))));
  }
  button.primary::part(base):active {
    background: var(--button-primary-pressed-fill);
    color: var(--button-primary-pressed-ink, revert-layer);
    border-color: var(--button-primary-hover-rim, var(--button-primary-rim, var(--button-pressed-rim, var(--button-rim, var(--line)))));
    border-image-source: var(--button-primary-image-pressed, var(--button-primary-image, var(--button-image-pressed, var(--button-image, none))));
  }
  button.quiet::part(base), button.ghost::part(base) { background: transparent; }
  button.ghost::part(base) {
    padding-inline: calc(var(--button-pad-inline, var(--space-lg)) - var(--button-border-width));
  }
  button.ghost.small::part(base), button.ghost.small::part(base):active {
    padding-inline: calc(var(--button-pad-inline, var(--space-md)) - var(--button-border-width));
  }
  button.square::part(base), button.square::part(base):active,
  button.ghost.square::part(base), button.ghost.square::part(base):active { padding-inline: 0; }
  button.quiet::part(base):hover { background: var(--button-quiet-hover-fill); }
  button.quiet::part(base):active { background: var(--button-quiet-pressed-fill); }
  button.ghost::part(base):hover { background: var(--button-ghost-hover-fill); }
  button.ghost::part(base):active { background: var(--button-ghost-pressed-fill); }
  button.danger::part(base) {
    background: var(--button-danger-fill);
    border-color: var(--button-danger-rim, var(--button-rim, var(--line)));
    box-shadow: var(--button-danger-shadow, var(--button-base-shadow, none));
    text-shadow: var(--button-danger-text-shadow, var(--button-text-shadow, revert-layer));
  }
  button.danger::part(base):hover { background: var(--button-danger-hover-fill); }
  button.danger::part(base):active { background: var(--button-danger-pressed-fill); }
  button.danger::part(base):hover, button.danger::part(base):active { color: var(--button-danger-ink); }
  /* A button that is on (`aria-pressed`: a toggle, a chip) wears the
     selection, whatever its variant, at rest, hovered and held. */
  button[aria-pressed=true]::part(base), button[aria-pressed=true]::part(base):hover,
  button[aria-pressed=true]::part(base):active {
    background: var(--button-selected-fill);
    color: var(--button-selected-ink);
    border-color: var(--button-selected-rim, var(--button-rim, var(--line)));
    box-shadow: var(--button-selected-shadow, var(--button-base-shadow, none));
  }
  button::part(base):disabled, button.quiet::part(base):disabled, button.ghost::part(base):disabled {
    background: var(--button-disabled-fill);
  }
  button.primary::part(base):disabled { background: var(--button-primary-disabled-fill); }
  button.danger::part(base):disabled { background: var(--button-danger-disabled-fill); }
  button::part(base):disabled, button.primary::part(base):disabled, button.danger::part(base):disabled {
    border-color: var(--button-disabled-rim, var(--button-rim, var(--line)));
    box-shadow: var(--button-disabled-shadow, var(--button-base-shadow, none));
    border-image-source: var(--button-image-disabled, var(--button-image, none));
  }
  button.primary::part(base):disabled {
    color: var(--button-primary-disabled-ink, revert-layer);
    border-image-source: var(--button-primary-image-disabled, var(--button-image-disabled, var(--button-image, none)));
  }
  /* A quiet button is text: no plate, rim, shadow or border image. */
  button.quiet::part(base), button.quiet::part(base):hover, button.quiet::part(base):active,
  button.quiet::part(base):disabled {
    border-width: 0; box-shadow: none; border-image-source: none;
  }
  /* Hover/press values are supplied by the widget, blended by the cascade. */
  button::part(base), input[type=checkbox]::part(row), input[type=radio]::part(option),
  select::part(option), stepper::part(button), input[type=number]::part(button),
  details::part(summary), keyboard::part(key), keyboard::part(candidate), input::part(frame),
  textarea::part(frame), select::part(trigger), input[type=checkbox]::part(box),
  input[type=radio]::part(ring), input[type=radio]::part(card), slot::part(frame),
  tabs::part(tab), input[type=range]::part(knob),
  input[type=checkbox][switch]::part(track), input[type=checkbox][switch]::part(knob) {
    transition: background var(--motion-ui-duration) var(--motion-ui-easing),
      border-color var(--motion-ui-duration) var(--motion-ui-easing),
      box-shadow var(--motion-ui-duration) var(--motion-ui-easing),
      color var(--motion-ui-duration) var(--motion-ui-easing),
      opacity var(--motion-ui-duration) var(--motion-ui-easing);
  }
  /* Panel / Dialog / Card / Tooltip */
  panel::part(frame), tooltip::part(frame), dialog::part(panel) {
    box-sizing: border-box;
    border-width: var(--panel-border-width); border-style: solid; border-color: var(--panel-rim);
    border-radius: var(--panel-radius);
  }
  panel::part(frame), dialog::part(panel) {
    background: var(--panel-fill, revert-layer);
    corner-shape: var(--panel-corner-shape);
    backdrop-filter: var(--panel-glass, none);
  }
  dialog::part(panel) { box-shadow: var(--panel-shadow, revert-layer); }
  dialog::part(backdrop) { background: var(--dialog-backdrop, revert-layer); }
  select::part(list) {
    box-sizing: border-box;
    border-width: var(--dropdown-list-rim-width); border-style: solid; border-color: var(--dropdown-list-rim);
    border-radius: var(--dropdown-list-radius);
    background: var(--dropdown-list-fill, revert-layer);
    box-shadow: var(--dropdown-list-shadow);
  }
  select::part(option):hover { background: var(--dropdown-option-hover-fill, revert-layer); }
  select::part(option):active { background: var(--dropdown-option-pressed-fill, var(--dropdown-option-hover-fill, revert-layer)); }
  select::part(option):checked {
    background: var(--dropdown-option-selected-fill, revert-layer);
    color: var(--dropdown-option-selected-ink, revert-layer);
  }
  /* A group's label and a disclosure's summary read as labels. */
  select::part(group), details::part(summary) {
    font-family: var(--label-font); letter-spacing: var(--label-tracking); text-transform: var(--label-transform);
  }
  input[type=range]::part(bubble) {
    box-sizing: border-box;
    border-width: var(--panel-border-width); border-style: solid; border-color: var(--slider-bubble-rim);
    border-radius: var(--slider-bubble-radius);
    background: var(--slider-bubble-fill, revert-layer);
    box-shadow: var(--slider-bubble-shadow);
  }
  /* The value bubble's text and the checkbox's tick take these parts' ink. */
  input[type=range]::part(bubble) { color: var(--slider-bubble-ink); }
  input[type=checkbox]::part(mark) { color: var(--checkbox-mark-ink); }
  /* Hud */
  hud::part(frame) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--line-70);
    border-radius: var(--hud-radius);
    corner-shape: var(--hud-corner-shape);
  }
  joystick::part(base):disabled, dpad::part(base):disabled {
    opacity: var(--state-touch-disabled-opacity);
  }
  joystick::part(base), dpad::part(base) {
    background: var(--touch-plate-fill, revert-layer);
    border-color: var(--touch-plate-rim, revert-layer);
  }
  joystick::part(rim) { background: var(--touch-plate-rim, revert-layer); }
  joystick::part(knob) { background: var(--touch-knob-fill, revert-layer); }
  joystick::part(direction), dpad::part(indicator) { background: var(--touch-active-fill, revert-layer); }
  /* Button: untouched token defaults also serve secondary controls. */
  button::part(base), input[type=checkbox]::part(row),
  input[type=radio]::part(option), select::part(option),
  stepper::part(button), input[type=number]::part(button), details::part(summary),
  keyboard::part(key), keyboard::part(candidate) {
    box-sizing: border-box;
    border-width: var(--button-border-width); border-style: solid; border-color: var(--line);
    border-radius: var(--button-radius);
    box-shadow: var(--button-shadow);
  }
  stepper::part(button):active, input[type=number]::part(button):active,
  details::part(summary):active,
  keyboard::part(key):active, keyboard::part(candidate):active {
    box-shadow: var(--button-pressed-shadow);
  }
  stepper::part(button):disabled, input[type=number]::part(button):disabled,
  details::part(summary):disabled,
  keyboard::part(key):disabled, keyboard::part(candidate):disabled {
    border: none; box-shadow: none;
  }
  input[type=checkbox]::part(row):hover { background: var(--checkbox-row-hover-fill, revert-layer); }
  input[type=radio]::part(option):hover {
    background: var(--radio-option-hover-fill, var(--checkbox-row-hover-fill, revert-layer));
  }
  /* Details: its summary a plate, its content's ink. */
  details::part(summary) {
    background: var(--details-summary-fill, revert-layer);
    color: var(--details-summary-ink, revert-layer);
    font-family: var(--details-summary-font, var(--label-font));
    border-width: var(--details-summary-border, var(--button-border-width); border-style: solid; border-color: var(--line));
    border-bottom: var(--details-summary-rule, var(--details-summary-border, var(--button-border-width) solid var(--line)));
    box-shadow: var(--details-summary-shadow, var(--button-shadow));
  }
  details::part(summary):hover {
    background: var(--details-summary-hover-fill, var(--details-summary-fill, revert-layer));
  }
  details[aria-expanded=true]::part(summary) {
    background: var(--details-summary-open-fill, var(--details-summary-fill, revert-layer));
    color: var(--details-summary-open-ink, var(--details-summary-ink, revert-layer));
  }
  details::part(content) { color: var(--details-content-ink, revert-layer); }
  /* Keyboard: its panel, keys and candidates. */
  keyboard::part(panel) { background: var(--keyboard-panel-fill, revert-layer); }
  keyboard::part(key), keyboard::part(candidate) {
    background: var(--keyboard-key-fill, revert-layer);
    border-color: var(--keyboard-key-rim);
    box-shadow: var(--keyboard-key-shadow, var(--button-shadow));
    color: var(--keyboard-key-ink, revert-layer);
    font-family: var(--keyboard-key-font, revert-layer);
  }
  keyboard::part(key):hover, keyboard::part(candidate):hover {
    background: var(--keyboard-key-hover-fill, var(--keyboard-key-fill, revert-layer));
    border-color: var(--keyboard-key-hover-rim, var(--keyboard-key-rim));
  }
  keyboard::part(key):active, keyboard::part(candidate):active {
    background: var(--keyboard-key-pressed-fill, var(--keyboard-key-fill, revert-layer));
    border-color: var(--keyboard-key-pressed-rim, var(--keyboard-key-rim));
  }
  /* Field: text boxes, checkbox marks and radio wells. */
  input::part(frame), textarea::part(frame), select::part(trigger),
  stepper::part(frame), stepper::part(value), input[type=checkbox]::part(box),
  input[type=radio]::part(ring) {
    box-sizing: border-box;
    border-width: var(--field-border-width); border-style: solid; border-color: var(--line);
    border-radius: var(--field-radius);
    box-shadow: var(--field-shadow);
  }
  input::part(frame), textarea::part(frame), select::part(trigger), stepper::part(frame) {
    background: var(--field-fill, revert-layer);
    color: var(--field-ink, revert-layer);
    font-family: var(--field-font, revert-layer);
    corner-shape: var(--field-corner-shape);
  }
  select::part(trigger), stepper::part(frame), stepper::part(value) {
    border-width: var(--field-rim-width, var(--field-border-width)); border-style: solid; border-color: var(--field-rim, var(--line));
  }
  input::part(frame):hover, textarea::part(frame):hover, select::part(trigger):hover {
    background: var(--field-hover-fill, var(--field-fill, revert-layer));
    border-color: var(--field-hover-rim, var(--field-rim, var(--line)));
  }
  input::part(frame):focus-visible, textarea::part(frame):focus-visible, select::part(trigger):focus-visible {
    border-color: var(--field-focus-rim, var(--field-rim, var(--line)));
  }
  select::part(trigger):focus-visible { box-shadow: var(--field-focus-shadow, var(--field-shadow)); }
  input::part(frame):focus-visible, textarea::part(frame):focus-visible {
    box-shadow: var(--field-focus-shadow, var(--field-frame-shadow, none));
  }
  input::part(frame):disabled, textarea::part(frame):disabled, select::part(trigger):disabled, stepper::part(frame):disabled {
    background: var(--field-disabled-fill, var(--field-fill, revert-layer));
    border-color: var(--field-disabled-rim, var(--field-rim, var(--line)));
  }
  input::part(text), textarea::part(text) { color: var(--field-ink, revert-layer); font-family: var(--field-font, revert-layer); }
  input::part(placeholder), textarea::part(placeholder) { color: var(--field-placeholder-ink, revert-layer); }
  input::part(selection), textarea::part(selection) { background: var(--field-selection-fill, revert-layer); }
  input::part(caret), textarea::part(caret) { background: var(--field-caret-fill, revert-layer); }
  select::part(trigger) { padding-inline: calc(var(--space-md) - var(--field-border-width)); }
  input[type=checkbox]::part(box) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--checkbox-box-rim);
    background: var(--checkbox-box-fill, revert-layer);
    box-shadow: var(--checkbox-box-shadow);
    border-radius: var(--checkbox-radius, var(--field-radius));
    corner-shape: var(--checkbox-corner-shape);
  }
  input[type=radio]::part(ring) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--radio-ring-rim);
    background: var(--radio-ring-fill, revert-layer);
    box-shadow: var(--radio-ring-shadow);
    corner-shape: var(--radio-corner-shape);
  }
  input[type=checkbox]::part(box):hover {
    border-color: var(--checkbox-box-hover-rim, var(--checkbox-box-rim));
    background: var(--checkbox-box-hover-fill, var(--checkbox-box-fill, revert-layer));
  }
  input[type=radio]::part(ring):hover {
    border-color: var(--radio-ring-hover-rim, var(--checkbox-box-hover-rim, var(--radio-ring-rim)));
  }
  input[type=checkbox]::part(box):checked,
  input[type=checkbox][aria-checked=mixed]::part(box) {
    border-width: var(--checkbox-checked-rim-width, 0px); border-style: solid; border-color: var(--checkbox-checked-rim, transparent);
    background: var(--checkbox-checked-fill, revert-layer);
    box-shadow: var(--checkbox-checked-shadow, none);
  }
  input[type=radio]::part(ring):checked {
    border-width: var(--radio-checked-rim-width, 0px); border-style: solid; border-color: var(--radio-checked-rim, transparent);
    background: var(--radio-checked-fill, revert-layer);
    box-shadow: var(--radio-checked-shadow, none);
  }
  input[type=radio]::part(dot) { background: var(--radio-dot-fill, revert-layer); corner-shape: var(--radio-corner-shape); }
  input[type=checkbox]::part(box):disabled, input[type=radio]::part(ring):disabled {
    background: var(--checkbox-disabled-fill, revert-layer);
    border-color: var(--checkbox-disabled-rim, var(--checkbox-box-rim));
  }
  input[type=checkbox]::part(box):checked:disabled, input[type=checkbox][aria-checked=mixed]::part(box):disabled,
  input[type=radio]::part(ring):checked:disabled {
    background: var(--checkbox-checked-disabled-fill, var(--checkbox-checked-fill, revert-layer));
    border-color: var(--checkbox-disabled-rim, var(--checkbox-checked-rim, transparent));
    box-shadow: none;
  }
  input[type=checkbox][aria-invalid=true]::part(box),
  select[aria-invalid=true]::part(trigger),
  stepper[aria-invalid=true]::part(frame), stepper[aria-invalid=true]::part(value) {
    border-width: var(--field-border-width); border-style: solid; border-color: var(--field-invalid-rim);
  }
  /* Slot / Avatar / Badge / Tab / Bar */
  slot::part(frame) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--line);
    border-radius: var(--slot-radius);
    box-shadow: var(--slot-shadow);
  }
  slot::part(frame):hover {
    border-color: var(--slot-hover-rim, var(--slot-rim, var(--line)));
    box-shadow: var(--slot-hover-shadow, none);
  }
  slot::part(frame):active { background: var(--slot-pressed-fill, var(--slot-fill, revert-layer)); }
  input[type=range]::part(knob) {
    border-width: var(--slider-knob-rim-width, var(--stroke-hairline)); border-style: solid; border-color: var(--slider-knob-rim);
    border-radius: var(--chip-radius);
    background: var(--slider-knob-fill, revert-layer);
    box-shadow: var(--slider-knob-shadow, revert-layer);
  }
  input[type=range]::part(knob):hover { box-shadow: var(--slider-knob-hover-shadow, var(--slider-knob-shadow, revert-layer)); }
  input[type=range]::part(knob):active { background: var(--slider-knob-pressed-fill, var(--slider-knob-fill, revert-layer)); }
  input[type=range]::part(knob):disabled { background: var(--slider-knob-disabled-fill, var(--slider-knob-fill, revert-layer)); }
  input[type=range]::part(core) { background: var(--slider-core-fill, revert-layer); }
  input[type=range]::part(knob), input[type=range]::part(core) { corner-shape: var(--slider-knob-corner-shape); }
  input[type=checkbox][switch]::part(track) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--switch-track-rim); border-radius: var(--chip-radius);
    background: var(--switch-track-fill, revert-layer);
  }
  input[type=checkbox][switch]::part(knob) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--switch-knob-rim); border-radius: var(--chip-radius);
    background: var(--switch-knob-fill, revert-layer);
    box-shadow: var(--switch-knob-shadow, revert-layer);
  }
  input[type=checkbox][switch]::part(track):checked {
    border-color: var(--switch-checked-rim, var(--switch-track-rim));
    background: var(--switch-checked-fill, revert-layer);
    box-shadow: var(--switch-checked-shadow, var(--switch-track-shadow, none));
  }
  input[type=checkbox][switch]::part(track), input[type=checkbox][switch]::part(knob) {
    corner-shape: var(--switch-corner-shape);
  }
  badge::part(base), tabs::part(tab), tabs::part(track), tabs::part(indicator) {
    border: none; border-radius: var(--tabs-radius);
  }
  /* A badge: a neutral one's fill and ink, each tone's, and a rim in the
     badge's ink (its tone's) a pack gives a width. */
  badge::part(base) {
    border-width: var(--badge-rim-width, 0px); border-style: solid; border-color: var(--badge-rim, currentColor);
    font-family: var(--badge-font, revert-layer);
    letter-spacing: var(--badge-tracking, revert-layer);
    text-transform: var(--badge-transform, revert-layer);
  }
  badge[tone=neutral]::part(base) {
    background: var(--badge-neutral-fill, revert-layer);
    color: var(--badge-neutral-ink, revert-layer);
  }
  badge[tone=primary]::part(base) {
    background: var(--badge-primary-fill, revert-layer);
    color: var(--badge-primary-ink, revert-layer);
  }
  badge[tone=info]::part(base) { background: var(--badge-info-fill, revert-layer); color: var(--badge-info-ink, revert-layer); }
  badge[tone=success]::part(base) { background: var(--badge-success-fill, revert-layer); color: var(--badge-success-ink, revert-layer); }
  badge[tone=warn]::part(base) { background: var(--badge-warn-fill, revert-layer); color: var(--badge-warn-ink, revert-layer); }
  badge[tone=danger]::part(base) { background: var(--badge-danger-fill, revert-layer); color: var(--badge-danger-ink, revert-layer); }
  /* Tabs: the track a well, each tab's fill, ink and type, the indicator. */
  tabs::part(track) {
    background: var(--tabs-track-fill, revert-layer);
    border: var(--tabs-track-border, none);
    border-bottom: var(--tabs-track-rule, var(--tabs-track-border, none));
    box-shadow: var(--tabs-track-shadow, revert-layer);
  }
  tabs::part(tab) {
    background: var(--tabs-tab-fill, revert-layer);
    color: var(--tabs-tab-ink, revert-layer);
    font-family: var(--tabs-font, revert-layer);
    font-size: var(--tabs-font-size, revert-layer);
    letter-spacing: var(--tabs-tracking, revert-layer);
    text-transform: var(--tabs-transform, revert-layer);
  }
  tabs::part(tab):hover {
    background: var(--tabs-tab-hover-fill, var(--tabs-tab-fill, revert-layer));
    color: var(--tabs-tab-hover-ink, var(--tabs-tab-ink, revert-layer));
  }
  tabs::part(tab):active { background: var(--tabs-tab-pressed-fill, var(--tabs-tab-hover-fill, var(--tabs-tab-fill, revert-layer))); }
  tabs::part(tab):disabled { color: var(--tabs-tab-disabled-ink, revert-layer); }
  tabs::part(indicator) {
    background: var(--tabs-indicator-fill, revert-layer);
    height: var(--tabs-indicator-height, revert-layer);
    box-shadow: var(--tabs-indicator-glow, revert-layer);
    border-radius: var(--tabs-indicator-radius, var(--tabs-radius));
  }
  input[type=radio]::part(card) {
    border-width: var(--stroke-hairline); border-style: solid; border-color: var(--line); border-radius: var(--radio-card-radius);
    box-shadow: var(--radio-card-shadow);
  }
  input[type=radio]::part(card):hover {
    border-color: var(--radio-card-hover-rim, var(--radio-card-rim, var(--line)));
    background: var(--radio-card-hover-fill, var(--radio-card-fill, revert-layer));
  }
  input[type=radio]::part(card):checked {
    background: var(--radio-card-checked-fill);
    border-color: var(--radio-card-checked-rim, var(--radio-card-rim, var(--line)));
    box-shadow: var(--radio-card-checked-shadow, var(--radio-card-shadow));
    color: var(--radio-card-checked-ink, revert-layer);
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
  input[type=range]::part(fill) { background: var(--slider-fill, revert-layer); box-shadow: var(--slider-fill-shadow); }
  input[type=range]::part(tick) { background: var(--slider-tick-fill, revert-layer); }
  meter::part(fill) { background: var(--meter-fill, revert-layer); }
  meter[region=suboptimum]::part(fill) { background: var(--meter-suboptimum-fill, revert-layer); }
  meter[region=poor]::part(fill) { background: var(--meter-poor-fill, revert-layer); }
  meter.health::part(fill) { background: var(--meter-health-fill, var(--meter-fill, revert-layer)); }
  meter.mana::part(fill) { background: var(--meter-mana-fill, var(--meter-fill, revert-layer)); }
  progress::part(fill) { background: var(--progress-fill, var(--meter-fill, revert-layer)); }
  /* Text and the surface's extras. */
  h1::part(flourish), h2::part(flourish), h3::part(flourish) { background: var(--heading-flourish, revert-layer); }
  scroll::part(bar) {
    background: var(--scrollbar-fill, revert-layer);
    border-radius: var(--scrollbar-radius, revert-layer);
  }
  og-icon::part(secondary) { color: var(--icon-secondary-ink, revert-layer); }
  popover::part(arrow) { background: var(--popover-arrow-fill, revert-layer); }
  og-spotlight::part(edge) { background: var(--spotlight-edge, revert-layer); }
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
  /* Unset, a ring follows its control's corners and their shape. */
  button::part(focus) { border-radius: var(--button-focus-radius, revert-layer); corner-shape: var(--button-focus-corner-shape, revert-layer); }
"#;
