//! Documents (docs/ui.md §2.1), bindings (§2.1), the layer stack and UI
//! actions (§2.1), through og-ui's public API: a shop screen parsed and
//! validated, mounted over a HUD with a viewmodel, its bindings formatted,
//! its controls writing back, Back and focus memory, a probe edit saved
//! back into the text, and the stack carried by a snapshot.
//!
//! Ways it could fail: a binding read once and never again (the gold
//! label stays at its first value), a list item's write landing on the
//! item variable instead of the list entry, Back reaching the HUD under a
//! menu, focus lost when the menu over a screen pops, a write-back that
//! rewrites lines it did not touch, a snapshot that drops the stack.

use crate::support;

use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::{Value, json};
use support::*;

pub const SHOP: &str = r#"<meta name="screen" content="shop" data-layer="menu" data-pause="true" data-under="dim" data-focus="Footer/Buy">
<meta name="prop:gold" content="number" data-doc="The player's gold.">
<meta name="prop:items" content="list" data-doc="What the shop sells.">
<meta name="prop:selected" content="string" data-doc="The selected item.">
<meta name="prop:sound" content="bool" data-doc="Purchase sounds.">
<meta name="prop:volume" content="number" data-doc="Volume, 0 to 1.">
<meta name="action:confirm" content="Buy">
<meta name="action:back" content="Leave">
<meta name="fixture:rich" content='{"gold":1250000,"items":[{"id":"sword","name":"Sword","price":1200,"owned":false},{"id":"shield","name":"Shield","price":800,"owned":true}],"selected":"sword","sound":true,"volume":0.5}'>
<panel id="Shop">
  <row id="Header">
    <text id="Title">Shop</text>
    <spacer />
    <text id="Gold">{formatCompact(gold)}</text>
  </row>
  <column id="Items">
    {#each items as item (item.id)}
      <row id="Item" class:selected={item.id == selected}>
        <text id="Name">{item.name}</text>
        <text id="Price">{formatNumber(item.price)} gold</text>
        <label><input type="checkbox" switch id="Owned" bind:checked={item.owned}> Owned</label>
      </row>
    {/each}
  </column>
  <column id="Preview">{@render preview?.()}</column>
  <input type="range" id="Volume" bind:value={volume} />
  <row id="Footer">
    <button id="Leave" onclick={leave}>Leave</button>
    <button id="Buy" variant="primary" disabled={gold < items.0.price} onclick={buy}>Buy</button>
  </row>
</panel>
<style>
#Shop { width: 520px; padding: 20px; gap: 12px; }
#Header { align-items: center; gap: 8px; }
#Title { font: var(--text-xl); }
#Items { gap: 4px; }
#Item { gap: 8px; opacity: 0.7; }
#Item.selected { opacity: 1; }
#Preview { height: 40px; }
#Footer { gap: 8px; }
</style>
"#;

const HUD: &str = r#"<meta name="screen" content="hud" data-layer="hud">
<meta name="prop:gold" content="number" data-doc="Gold.">
<row id="Bar">
  <button id="Menu" onclick={menu}>Menu</button>
  <text id="Gold">{formatNumber(gold)}</text>
</row>
<style>#Bar { padding: 8px; }</style>
"#;

fn ui() -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/shop.svelte", SHOP).expect("shop parses"));
    ui.define_document(UiDocument::parse("ui/hud.svelte", HUD).expect("hud parses"));
    ui
}

fn tick(ui: &mut Ui, input: InputFrame) {
    ui.begin(input, viewport());
    ui.end();
}

#[test]
fn transition_variables_substitute_after_longhands_win() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "timing.svelte",
            r#"
<meta name="component" content="timing">
<column id="Target" class:expanded={$expanded}/>
<style>
@layer engine-default {
  column { --duration: 2s; --curve: linear; width: 20px; height: 40px;
    transition: width var(--duration) var(--curve); }
  column.expanded { width: 80px; }
}
column { transition-duration: 200ms; }
</style>"#,
        )
        .unwrap(),
    );
    mount_stores(
        &mut ui,
        SourceId::new("test", "timing"),
        "timing",
        &[("expanded", json!(false))],
    );
    let step = |ui: &mut Ui, expanded: bool| {
        set_store(ui, "expanded", json!(expanded));
        ui.begin(InputFrame::default(), viewport());
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    for _ in 0..30 {
        step(&mut ui, false);
    }
    let target = ui.element("test:timing").unwrap();
    assert_eq!(target.width, 20.0);
    for _ in 0..7 {
        step(&mut ui, true);
    }
    let width = ui.element("test:timing").unwrap().width;
    assert!(
        width > 30.0 && width < 70.0,
        "200ms transition, not 2s or instant: {width}"
    );
    for _ in 0..30 {
        step(&mut ui, true);
    }
    assert_eq!(ui.element("test:timing").unwrap().width, 80.0);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

#[test]
fn media_queries_follow_dimensions_preferences_and_nested_intersections() {
    let doc = UiDocument::parse(
        "media.svelte",
        r#"
<meta name="component" content="media">
<column><panel id="Range"/><panel id="Ratio"/><panel id="Nested"/><panel id="Motion"/></column>
<style>
panel { width: 10px; height: 10px; padding: 0px; }
@MEDIA
(300px <= width < 600px) and (min-height: 200px) { #Range { width: 31px; } }
@media (aspect-ratio >= 3/2), (orientation: portrait) { #Ratio { width: 47px; } }
@media (width >= 400px) { @media (height < 500px) { #Nested { width: 63px; } } }
@media (prefers-reduced-motion: reduce) { #Motion { width: 79px; } }
</style>"#,
    )
    .unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(doc);
    mount_stores(&mut ui, SourceId::new("test", "media"), "media", &[]);
    for (w, h, reduced, expected) in [
        (299.5, 240.0, false, [10.0, 10.0, 10.0, 10.0]),
        (300.0, 240.0, false, [31.0, 10.0, 10.0, 10.0]),
        (599.5, 300.0, true, [31.0, 47.0, 63.0, 79.0]),
        (600.0, 500.0, false, [10.0, 10.0, 10.0, 10.0]),
        (400.0, 400.0, false, [31.0, 47.0, 63.0, 10.0]),
        (200.0, 600.0, true, [10.0, 47.0, 10.0, 79.0]),
    ] {
        let mut theme = *ui.theme();
        theme.unit_pin = 1.0;
        theme.reduced_motion = reduced;
        ui.set_theme(theme);
        ui.begin(InputFrame::default(), Viewport::new(w, h));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        for (id, width) in ["Range", "Ratio", "Nested", "Motion"]
            .into_iter()
            .zip(expected)
        {
            let actual = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.id == id)
                .unwrap();
            assert_eq!(actual.width, width, "{id} at {w}x{h}, reduced={reduced}");
        }
    }
}

#[test]
fn invalid_media_queries_report_document_location_and_fix() {
    let errors = UiDocument::parse(
        "invalid-media.svelte",
        "<meta name='component' content='invalid'/><panel/>\n<style>\n@media (20px < width > 40px) { panel { width: 30px; } }\n</style>",
    )
    .unwrap_err()
    .join("\n");
    assert!(errors.contains("invalid-media.svelte:3:1"), "{errors}");
    assert!(
        errors.contains("chained ranges must point in the same direction"),
        "{errors}"
    );
}

#[test]
fn forced_input_density_drives_media_queries_and_retained_updates() {
    use crate::support::Doc;
    let source = r#"<meta name="component" content="t">
<column><div id="Pointer"></div><div id="Hover"></div><div id="Pad"></div><input type="number" value="3" /></column>
<style>
div { width: 10px; height: 10px; }
@media (pointer: coarse) { #Pointer { width: 21px; } }
@media (hover: none) { #Hover { width: 32px; } }
@media (-og-input: pad) { #Pad { width: 43px; } }
</style>"#;
    let mut doc = Doc::new(source, &[]);
    for (density, coarse, pointer, hover, pad) in [
        (DensityMode::Auto, false, 10.0, 10.0, 10.0),
        (DensityMode::Touch, false, 21.0, 32.0, 10.0),
        (DensityMode::Mouse, true, 10.0, 10.0, 10.0),
        (DensityMode::Mouse, false, 10.0, 10.0, 10.0),
        (DensityMode::Mouse, true, 10.0, 10.0, 10.0),
        (DensityMode::Auto, true, 21.0, 32.0, 10.0),
        (DensityMode::Pad, true, 21.0, 32.0, 43.0),
        (DensityMode::Auto, true, 21.0, 32.0, 10.0),
    ] {
        let mut theme = *doc.ui.theme();
        if theme.density != density {
            theme.density = density;
            doc.ui.set_theme(theme);
        }
        doc.viewport.coarse_pointer = coarse;
        doc.step(InputFrame::default());
        for (id, width) in [("Pointer", pointer), ("Hover", hover), ("Pad", pad)] {
            assert_eq!(doc.get(id).width, width, "{density:?} {id}");
        }
        assert_eq!(doc.get("Increment").width, if coarse { 44.0 } else { 24.0 });
    }
    assert!(
        UiDocument::parse(
            "bad.svelte",
            &source.replace("-og-input: pad", "-og-input: touch")
        )
        .is_err()
    );
}

/// Each preference and display feature answers the theme or the surface,
/// each value from its own setting, and its boolean form follows Media
/// Queries 4 §2.4.4 (true unless the value is 0, `none` or one it names
/// false). The host reports no forced or inverted colours, no data saving,
/// scripts, a surface that updates every frame and fills the display
/// area, so those answer one value only.
#[test]
fn preference_and_display_media_features_follow_the_theme_and_surface() {
    use crate::support::Doc;
    // Each query widens its own row to 20px.
    let queries = [
        ("prefers-color-scheme: dark", [false, true]),
        ("prefers-color-scheme: light", [true, false]),
        ("prefers-color-scheme", [true, true]),
        ("prefers-contrast: more", [false, true]),
        ("prefers-contrast: no-preference", [true, false]),
        ("prefers-contrast: less", [false, false]),
        ("prefers-contrast", [false, true]),
        ("prefers-reduced-transparency: reduce", [false, true]),
        ("prefers-reduced-transparency", [false, true]),
        ("resolution: 2dppx", [false, true]),
        ("min-resolution: 192dpi", [false, true]),
        ("resolution >= 75dpcm", [false, true]),
        ("resolution: 1x", [true, false]),
        ("color-gamut: srgb", [true, true]),
        ("color-gamut: p3", [false, true]),
        ("color-gamut: rec2020", [false, false]),
        ("dynamic-range: high", [false, true]),
        ("dynamic-range: standard", [true, true]),
        ("dynamic-range", [true, true]),
        ("hover: hover", [true, false]),
        ("any-hover: none", [false, true]),
        ("hover", [true, false]),
        ("pointer: fine", [true, false]),
        ("any-pointer: coarse", [false, true]),
        ("pointer", [true, true]),
        ("inverted-colors: none", [true, true]),
        ("inverted-colors", [false, false]),
        ("scripting: enabled", [true, true]),
        ("scripting: initial-only", [false, false]),
        ("scripting", [true, true]),
        ("update: fast", [true, true]),
        ("update: slow", [false, false]),
        ("update", [true, true]),
        ("display-mode: fullscreen", [true, true]),
        ("display-mode: browser", [false, false]),
        ("display-state: fullscreen", [true, true]),
        ("display-state: maximized", [false, false]),
        ("display-state: normal", [false, false]),
        ("display-state", [true, true]),
        ("prefers-reduced-data: no-preference", [true, true]),
        ("prefers-reduced-data: reduce", [false, false]),
        ("prefers-reduced-data", [false, false]),
    ];
    let rows: String = (0..queries.len())
        .map(|i| format!("<div id=\"Q{i}\"></div>"))
        .collect();
    let rules: String = queries
        .iter()
        .enumerate()
        .map(|(i, (q, _))| format!("@media ({q}) {{ #Q{i} {{ width: 20px; }} }}\n"))
        .collect();
    let source = format!(
        "<meta name=\"component\" content=\"t\">\n<column>{rows}</column>\n<style>\ndiv {{ width: 10px; height: 1px; }}\n{rules}</style>"
    );
    let mut doc = Doc::new(&source, &[]);
    for (k, [scheme, contrast, transparency, dpr, wide, high, coarse]) in [
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0],
    ]
    .into_iter()
    .enumerate()
    {
        let mut theme = *doc.ui.theme();
        theme.scheme = if scheme > 0.0 {
            Scheme::Dark
        } else {
            Scheme::Light
        };
        theme.contrast = if contrast > 0.0 {
            Contrast::High
        } else {
            Contrast::Standard
        };
        theme.reduced_transparency = transparency > 0.0;
        theme.density = DensityMode::Auto;
        doc.ui.set_theme(theme);
        doc.viewport.dpr = dpr;
        doc.viewport.wide_gamut = wide > 0.0;
        doc.viewport.high_range = high > 0.0;
        doc.viewport.coarse_pointer = coarse > 0.0;
        doc.step(InputFrame::default());
        for (i, (query, expected)) in queries.iter().enumerate() {
            let width = doc.get(&format!("Q{i}")).width;
            assert_eq!(
                width,
                if expected[k] { 20.0 } else { 10.0 },
                "({query}) in setting {k}"
            );
        }
    }
}

/// Media Queries 4's deprecated device features read the play surface,
/// Compatibility's `-webkit-device-pixel-ratio` is `resolution` in dppx and
/// `-webkit-transform-3d` is 1 (3D transforms draw); no host reports a
/// fold, so the posture is `continuous` with one segment each way.
/// `@custom-media` (Media Queries 5 §3) names a query list, `true` or
/// `false`, usable before it is defined and inside another definition;
/// the last definition of a name wins.
#[test]
fn device_compat_posture_and_custom_media_queries_answer_the_surface() {
    use crate::support::Doc;
    // At 800 × 900, 2dppx and a fine pointer; then 500 × 900, 1dppx, coarse.
    let queries = [
        ("(device-width: 800px)", [true, false]),
        ("(min-device-height: 900px)", [true, true]),
        ("(device-aspect-ratio: 8/9)", [true, false]),
        ("(max-device-width: 600px)", [false, true]),
        ("(-webkit-device-pixel-ratio: 2)", [true, false]),
        ("(-webkit-min-device-pixel-ratio: 1.5)", [true, false]),
        ("(-webkit-max-device-pixel-ratio: 1.5)", [false, true]),
        ("(-webkit-transform-3d)", [true, true]),
        ("(-webkit-transform-3d: 0)", [false, false]),
        ("(device-posture: continuous)", [true, true]),
        ("(device-posture: folded)", [false, false]),
        ("(horizontal-viewport-segments: 1)", [true, true]),
        ("(vertical-viewport-segments > 1)", [false, false]),
        ("(--wide)", [true, false]),
        ("not (--wide)", [false, true]),
        ("(--wide-touch)", [false, false]),
        ("(--wide) or (--touch)", [true, true]),
        ("(--always)", [true, true]),
        ("(--never)", [false, false]),
    ];
    let rows: String = (0..queries.len())
        .map(|i| format!("<div id=\"Q{i}\"></div>"))
        .collect();
    let rules: String = queries
        .iter()
        .enumerate()
        .map(|(i, (q, _))| format!("@media {q} {{ #Q{i} {{ width: 20px; }} }}\n"))
        .collect();
    let custom = "@custom-media --wide-touch (--wide) and (--touch);
@custom-media --wide (width >= 900px);
@custom-media --wide (width >= 700px);
@custom-media --touch (pointer: coarse);
@custom-media --always true;
@custom-media --never false;";
    let source = format!(
        "<meta name=\"component\" content=\"t\">\n<column>{rows}</column>\n<style>\ndiv {{ width: 10px; height: 1px; }}\n{rules}{custom}\n</style>"
    );
    let mut doc = Doc::new(&source, &[]);
    for (k, (width, dpr, coarse)) in [(800.0, 2.0, false), (500.0, 1.0, true)]
        .into_iter()
        .enumerate()
    {
        doc.viewport.width = width;
        doc.viewport.dpr = dpr;
        doc.viewport.coarse_pointer = coarse;
        doc.step(InputFrame::default());
        for (i, (query, expected)) in queries.iter().enumerate() {
            let got = doc.get(&format!("Q{i}")).width;
            let want = if expected[k] { 20.0 } else { 10.0 };
            assert_eq!(got, want, "{query} in setting {k}");
        }
    }
    let error = |css: &str| {
        UiDocument::parse(
            "ui/t.svelte",
            &format!("<meta name=\"component\" content=\"t\"><div></div><style>{css}</style>"),
        )
        .unwrap_err()
        .join("\n")
    };
    assert!(error("@media (--nope) { div { width: 1px; } }").contains("names no @custom-media"));
    let cycle = error("@custom-media --a (--a); @media (--a) { div { width: 1px; } }");
    assert!(cycle.contains("refers to itself"), "{cycle}");
}

/// CSS Cascade 4 §7.3's keywords: `inherit` takes the parent's computed
/// value, `initial` the property's initial one (`width: auto`, so an empty
/// flex item shrinks to 0; `font-size: medium`, the root's size), `unset`
/// inherits an inherited property and is `initial` otherwise, and `revert`
/// rolls an author rule back to the user agent's (a `p`'s 1em margins).
#[test]
fn css_wide_keywords_inherit_reset_and_revert() {
    use crate::support::Doc;
    let text = r#"<meta name="component" content="t">
<div id="Root"><div id="Row">
  <div id="Inherit" class="c"></div><div id="Initial" class="c"></div><div id="Unset" class="c"></div>
  <div id="SizeUnset" class="c"></div><div id="SizeInitial" class="c"></div><div id="SizeInherit" class="c"></div>
</div><p id="Zero">x</p><p id="Revert">x</p><div id="After"></div></div>
<style>
#Row { display: flex; width: 300px; font-size: 20px; }
.c { width: 30px; height: 1px; font-size: 10px; }
#Inherit { width: inherit; }
#Initial { width: initial; }
#Unset { width: unset; }
#SizeUnset { font-size: unset; width: 2em; }
#SizeInitial { font-size: initial; width: 2em; }
#SizeInherit { font-size: inherit; width: 2em; }
p { margin: 0; font-size: 15px; line-height: 20px; }
#Revert { margin: revert; }
#After { height: 1px; }
</style>"#;
    let d = Doc::new(text, &[]);
    let root = d.ui.theme().type_ramp.body.size;
    for (id, width) in [
        ("Inherit", 300.0),
        ("Initial", 0.0),
        ("Unset", 0.0),
        ("SizeUnset", 40.0),
        ("SizeInitial", 2.0 * root),
        ("SizeInherit", 40.0),
    ] {
        assert_eq!(d.get(id).width, width, "{id}");
    }
    // `revert`: 15px above and below #Revert, none at #Zero.
    let (zero, revert, after) = (d.get("Zero"), d.get("Revert"), d.get("After"));
    assert_eq!(revert.y - (zero.y + zero.height), 15.0);
    assert_eq!(after.y - (revert.y + revert.height), 15.0);
}

/// `@container style()` (CSS Conditional 5 §6.2) reads the nearest
/// ancestor's computed custom property, a bound one re-read when its store
/// changes, an unset one never matching, and its ranges compare property
/// values and literals; a container name alone applies under that
/// container (§6.1); `scroll-state()` (§6.3) reads the
/// nearest scroll-state container's edges and last scroll direction, and
/// follows a wheel.
#[test]
fn container_style_and_scroll_state_queries_follow_their_container() {
    use crate::support::Doc;
    let text = r#"<meta name="component" content="t">
<div id="Root">
  <div id="Box" style:--t={$t}><div id="Styled"></div><div id="Missing"></div>
    <div id="Inner"><div id="Shadowed"></div></div></div>
  <div id="Ranged"><div id="Gt"></div><div id="Lt"></div><div id="Between"></div><div id="Props"></div><div id="Length"></div><div id="Mixed"></div><div id="Named"></div><div id="Unnamed"></div></div>
  <div id="Scroller"><div id="Fill"></div></div>
</div>
<style>
div div { height: 1px; width: 10px; }
#Inner { --t: light; }
#Ranged { --n: 5; --gap: 12px; --low: 2; container-name: ranged; }
@container style(--n > 3) { #Gt { width: 51px; } }
@container style(--n < 3) { #Lt { width: 52px; } }
@container style(2 < --n <= 5) { #Between { width: 53px; } }
@container style(--low < --n) { #Props { width: 54px; } }
@container style(--gap >= calc(0.125in)) { #Length { width: 55px; } }
@container style(--gap > 3) { #Mixed { width: 56px; } }
@container ranged { #Named { width: 57px; } }
@container nowhere { #Unnamed { width: 58px; } }
@container style(--t: dark) { #Styled, #Shadowed { width: 31px; } }
@container style(--missing) { #Missing { width: 32px; } }
#Scroller { container-type: scroll-state; overflow-y: auto; scrollbar-width: none; height: 50px; width: 100px; }
#Fill { height: 200px; }
@container scroll-state(scrollable: bottom) { #Fill { width: 41px; } }
@container scroll-state(scrollable: top) { #Fill { width: 42px; } }
@container scroll-state(scrolled: bottom) { #Fill { margin-left: 3px; } }
</style>"#;
    let mut d = Doc::new(text, &[("t", json!("dark"))]);
    // The fill's width, and its inset from the scroller's left.
    let fill = |d: &Doc| (d.get("Fill").width, d.get("Fill").x - d.get("Scroller").x);
    assert_eq!(d.get("Styled").width, 31.0);
    assert_eq!(
        d.get("Shadowed").width,
        10.0,
        "the nearest container's --t is light"
    );
    assert_eq!(d.get("Missing").width, 10.0);
    // Ranges compare numbers of one type (12px ≥ 0.125in = 12px; 12px
    // against 3 is false); a name alone needs only the named container.
    for (id, width) in [
        ("Gt", 51.0),
        ("Lt", 10.0),
        ("Between", 53.0),
        ("Props", 54.0),
        ("Length", 55.0),
        ("Mixed", 10.0),
        ("Named", 57.0),
        ("Unnamed", 10.0),
    ] {
        assert_eq!(d.get(id).width, width, "{id}");
    }
    d.set("t", json!("light"));
    assert_eq!(d.get("Styled").width, 10.0);
    d.set("t", json!("dark"));
    assert_eq!(d.get("Styled").width, 31.0);
    // At the top: it can scroll down only, and has not scrolled.
    assert_eq!(fill(&d), (41.0, 0.0));
    let s = d.get("Scroller");
    d.step(InputFrame {
        wheel: og_core::glam::Vec2::new(0.0, 1000.0),
        ..InputFrame::hover(s.x + 10.0, s.y + 10.0)
    });
    for _ in 0..4 {
        d.step(InputFrame::hover(s.x + 10.0, s.y + 10.0));
    }
    // At the bottom, scrolled down: it can scroll up only.
    assert_eq!(fill(&d), (42.0, 3.0));
}

/// `@supports` (CSS Conditional 3 §2, 4 §2, 5 §2): a declaration the
/// cascade takes holds, one it refuses does not; `selector()`,
/// `font-format()`, `font-tech()`, `at-rule()` (not `@charset`, which is
/// no at-rule, nor print's `@page`), `named-feature()` (CSS Color 4 gamut
/// mapping only) and `env()`, with `not`, `and` and `or`. `@charset` is
/// the stylesheet's encoding marker only, and the rules after it apply.
/// `@import` (CSS Cascade 5 §2) brings a sheet into the cascade under its
/// `layer()`, `supports()` and media conditions.
#[test]
fn supports_charset_and_import_conditions_gate_their_rules() {
    let css = r#"@charset "utf-8";
@layer defaults, base;
@import "base.css" layer(base);
@import "wide.css" (width > 500px);
@import "narrow.css" (width < 500px);
@import "grid.css" supports(display: grid);
@import "none.css" supports(display: blorp);
@import "at.css" supports(at-rule(@layer));
#Base { width: 12px; }
@layer base { #Layered { width: 13px; } }
@supports (display: grid) { #Decl { width: 21px; } }
@supports (display: blorp) { #Bad { width: 22px; } }
@supports not (display: blorp) { #Not { width: 23px; } }
@supports (display: grid) and (gap: 4px) { #And { width: 24px; } }
@supports (display: blorp) or (color: red) { #Or { width: 25px; } }
@supports selector(:has(+ a)) { #Selector { width: 26px; } }
@supports font-format(woff2) and font-tech(variations) { #Font { width: 27px; } }
@supports font-format(eot) { #Eot { width: 28px; } }
@supports at-rule(@container) and at-rule(@custom-media) { #AtRule { width: 41px; } }
@supports at-rule(@charset) or at-rule(@page) { #NoAtRule { width: 42px; } }
@supports named-feature(gamut-mapping) { #Named { width: 43px; } }
@supports named-feature(anchor-position-follows-transforms) or named-feature(nope) { #NoNamed { width: 44px; } }
@supports env(safe-area-inset-top) and env(-og-hud-inset-left) { #Env { width: 45px; } }
@supports env(viewport-segment-width) { #NoEnv { width: 46px; } }
@layer defaults { div div { height: 1px; width: 10px; } }"#;
    let ids = [
        "Base", "Layered", "Wide", "Narrow", "Grid", "None", "Decl", "Bad", "Not", "And", "Or",
        "Selector", "Font", "Eot", "At", "AtRule", "NoAtRule", "Named", "NoNamed", "Env", "NoEnv",
    ];
    let body: String = ids
        .iter()
        .map(|id| format!("<div id=\"{id}\"></div>"))
        .collect();
    let text = format!(
        "<meta name=\"component\" content=\"t\">\n<div id=\"Root\">{body}</div>\n<style>\n{css}\n</style>"
    );
    let imports = vec![
        (
            "ui/base.css".to_string(),
            "#Base { width: 99px; } #Layered { width: 98px; }".to_string(),
        ),
        (
            "ui/wide.css".to_string(),
            "#Wide { width: 31px; }".to_string(),
        ),
        (
            "ui/narrow.css".to_string(),
            "#Narrow { width: 32px; }".to_string(),
        ),
        (
            "ui/grid.css".to_string(),
            "#Grid { width: 33px; }".to_string(),
        ),
        (
            "ui/none.css".to_string(),
            "#None { width: 34px; }".to_string(),
        ),
        ("ui/at.css".to_string(), "#At { width: 35px; }".to_string()),
    ];
    let doc = UiDocument::parse_with_imports("ui/t.svelte", &text, imports)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut ui = ui_with_font(test_font_metrics());
    let name = doc.name.clone();
    ui.define_document(doc);
    mount_stores(&mut ui, SourceId::new("test", "t"), &name, &[]);
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let els = ui.probe_elements();
    let width = |id: &str| els.iter().find(|e| e.id == id).unwrap().width;
    for (id, expected) in [
        // An unlayered rule beats the imported layer; the later `@layer
        // base` rule beats the imported one inside it.
        ("Base", 12.0),
        ("Layered", 13.0),
        ("Wide", 31.0),
        ("Narrow", 10.0),
        ("Grid", 33.0),
        ("None", 10.0),
        ("Decl", 21.0),
        ("Bad", 10.0),
        ("Not", 23.0),
        ("And", 24.0),
        ("Or", 25.0),
        ("Selector", 26.0),
        ("Font", 27.0),
        ("Eot", 10.0),
        ("At", 35.0),
        ("AtRule", 41.0),
        ("NoAtRule", 10.0),
        ("Named", 43.0),
        ("NoNamed", 10.0),
        ("Env", 45.0),
        ("NoEnv", 10.0),
    ] {
        assert_eq!(width(id), expected, "{id}");
    }
}

#[test]
fn core_screen_readouts_preserve_units_and_unbroken_ranks() {
    struct Messages(std::collections::BTreeMap<String, String>, Value);
    impl bind::Env for Messages {
        fn translate(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn lookup(&self, root: &str) -> Option<Value> {
            self.1.get(root.trim_start_matches('$')).cloned()
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine/src/ui/screens");
    let mut failures = Vec::new();
    for name in ["leaderboard", "results", "calibration", "save-load"] {
        let source = std::fs::read_to_string(root.join(format!("{name}.svelte"))).unwrap();
        let doc = UiDocument::parse(name, &source).unwrap();
        let mut vm = doc.fixtures[0].1.clone();
        if name == "calibration" {
            vm["step"] = json!("offset");
        }
        if name == "results" {
            let stats = vm["stats"].as_array_mut().unwrap();
            stats.push(json!({"id":"fraction", "name":"Ratio", "value":0.375}));
            stats.push(
                json!({"id":"percentage", "name":"Completion", "format":"percent", "value":0.125}),
            );
            stats.push(json!({"id":"rate", "name":"Rate", "format":"percent", "count":true, "decimals":1, "value":0.625}));
        }
        let table = og_i18n::StringTable::parse(
            &std::fs::read_to_string(root.join(format!("{name}.en.ogt"))).unwrap(),
        )
        .unwrap();
        let env = Messages(
            table
                .entries
                .into_iter()
                .map(|(k, (v, _))| (k, v))
                .collect(),
            vm.clone(),
        );
        for width in [390.0, 1280.0] {
            let mut ui = ui_with_font(test_font());
            ui.define_document(doc.clone());
            ui.push_screen(name, vm.clone()).unwrap();
            // A counted stat (the fixture's score) is on its way up from 0
            // at first and has landed after its 900 ms.
            let mut counting = false;
            for tick in 0..if name == "results" { 60 } else { 30 } {
                ui.begin(InputFrame::default(), Viewport::new(width, 844.0));
                ui.build_layers(&env);
                ui.end();
                if tick == 5 {
                    counting = !ui.frame().items.iter().any(|item| {
                        matches!(item.cmd.unstyled(), DrawCmd::Text { text, .. } if text == "48,210" || text == "62.5%")
                    });
                }
            }
            let lines: Vec<_> = ui
                .frame()
                .items
                .iter()
                .filter_map(|item| {
                    if let DrawCmd::Text { text, .. } = item.cmd.unstyled() {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let valid = match name {
                "leaderboard" => lines.contains(&"1284th") && !lines.contains(&"1284t"),
                "results" => {
                    counting
                        && ["62%", "48,210", "0.375", "12.5%", "62.5%"]
                            .iter()
                            .all(|value| lines.contains(value))
                }
                "calibration" => lines.contains(&"24 ms") && !lines.contains(&"24"),
                "save-load" => lines.contains(&"Back"),
                _ => unreachable!(),
            };
            if !valid {
                failures.push(format!("{name}/{width}: {lines:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The tutorial screen points at an element of the document under it:
/// with a target the screen dims around that element, which stays
/// pressable while the rest is not, and without one nothing is dimmed.
#[test]
fn tutorial_lights_a_target_in_the_document_under_it() {
    struct Env(std::collections::BTreeMap<String, String>);
    impl bind::Env for Env {
        fn translate(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine/src/ui/screens");
    let source = std::fs::read_to_string(root.join("tutorial.svelte")).unwrap();
    let doc = UiDocument::parse("tutorial", &source).unwrap();
    let table = og_i18n::StringTable::parse(
        &std::fs::read_to_string(root.join("tutorial.en.ogt")).unwrap(),
    )
    .unwrap();
    let messages: std::collections::BTreeMap<_, _> = table
        .entries
        .into_iter()
        .map(|(k, (v, _))| (k, v))
        .collect();
    let hud = r#"<meta name="screen" content="hud" data-layer="hud">
<div id="Hud"><button id="Dodge" onclick={dodge}>Dodge</button><button id="Block" onclick={block}>Block</button></div>
<style>
#Hud { width: stretch; height: stretch; }
#Dodge { position: absolute; left: 400px; top: 300px; width: 96px; height: 48px; }
#Block { position: absolute; left: 40px; top: 40px; width: 96px; height: 48px; }
</style>"#;
    let mut ui = ui_with_font(test_font());
    ui.define_document(UiDocument::parse("hud.svelte", hud).unwrap());
    ui.define_document(doc.clone());
    let mut vm = doc.fixtures[0].1.clone();
    vm["target"] = json!("Dodge");
    ui.push_screen("hud", Value::Null).unwrap();
    let stores: Vec<(&str, Value)> = vm
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, value)| (name.as_str(), value.clone()))
        .collect();
    register_screen(&mut ui, "tutorial", &stores);
    let env = Env(messages);
    let step = |ui: &mut Ui, env: &Env, input: InputFrame| -> Vec<String> {
        ui.begin(input, Viewport::new(1280.0, 720.0));
        ui.build_layers(env);
        ui.end();
        ui.take_doc_events()
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| e.name)
            .collect()
    };
    let settle = |ui: &mut Ui, env: &Env| {
        for _ in 0..30 {
            step(ui, env, InputFrame::default());
        }
    };
    // A press and its release at the centre of path: the events it raised.
    let click = |ui: &mut Ui, env: &Env, path: &str| -> Vec<String> {
        let e = ui
            .element(path)
            .unwrap_or_else(|| panic!("no {path} among {:?}", paths(ui)));
        let at = og_core::glam::Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
        let mut names = Vec::new();
        for (pressed, released) in [(true, false), (false, true)] {
            names.extend(step(
                ui,
                env,
                InputFrame {
                    pointer: at,
                    pointer_inside: true,
                    down: pressed,
                    pressed,
                    released,
                    ..InputFrame::default()
                },
            ));
        }
        names
    };
    let rect = |ui: &Ui, path: &str| {
        let e = ui
            .element(path)
            .unwrap_or_else(|| panic!("no {path} among {:?}", paths(ui)));
        Rect::new(e.x, e.y, e.width, e.height)
    };
    fn paths(ui: &Ui) -> Vec<&str> {
        ui.elements().iter().map(|e| e.path.as_str()).collect()
    }

    settle(&mut ui, &env);
    let target = rect(&ui, "hud/Hud/Dodge");
    let hole = rect(&ui, "Spot~spotlight/Middle/Hole");
    assert!(
        hole.x <= target.x
            && hole.y <= target.y
            && hole.right() >= target.right()
            && hole.bottom() >= target.bottom(),
        "the hole {hole:?} holds the target {target:?}"
    );
    let card = ui
        .elements()
        .iter()
        .find(|e| e.kind == "popover" && e.path.ends_with("tutorial/Tutorial/Spot"))
        .map(|e| Rect::new(e.x, e.y, e.width, e.height))
        .expect("the card is the spotlight's callout");
    assert!(
        card.y >= target.bottom() || card.bottom() <= target.y,
        "the card {card:?} stands clear of the target {target:?}"
    );
    assert!(ui.element("tutorial/Tutorial/Spot/card/Title").is_some());
    assert!(ui.element("tutorial/Tutorial/Card").is_none());
    assert_eq!(
        click(&mut ui, &env, "hud/Hud/Block"),
        Vec::<String>::new(),
        "a press on the dimmed screen stops at the shade"
    );
    assert_eq!(
        click(&mut ui, &env, "hud/Hud/Dodge"),
        ["dodge"],
        "the lit element takes its press"
    );
    assert_eq!(
        click(&mut ui, &env, "tutorial/Tutorial/Spot/card/Foot/Skip"),
        ["skip"]
    );

    // Without a target nothing is dimmed and the card stands at x and y.
    set_store(&mut ui, "target", json!(""));
    settle(&mut ui, &env);
    assert!(
        !paths(&ui).iter().any(|p| p.starts_with("Spot~spotlight")),
        "{:?}",
        paths(&ui)
    );
    let card = rect(&ui, "tutorial/Tutorial/Card");
    assert_eq!((card.x, card.y), (64.0, 120.0));
    assert_eq!(click(&mut ui, &env, "hud/Hud/Block"), ["block"]);
    assert_eq!(
        click(&mut ui, &env, "tutorial/Tutorial/Card/card/Foot/Skip"),
        ["skip"]
    );
}

#[test]
fn lexical_css_inherits_computed_values_and_cascades_before_substitution() {
    let source = r#"<meta name="screen" content="cascade" data-layer="hud">
<column id="Root" class="parent">
  <button id="Child" class="size">Child</button>
  <button id="Fallback">Fallback</button>
  <button id="Invalid">Invalid</button>
  <column id="Branch"><button id="Nested">Nested</button></column>
  <button id="Sibling">Sibling</button>
  <Badge id="Component"/>
  <Badge id="Override" --width="66px"/>
  <Badge id="Bound" --width={$wide}/>
  <text id="Rich" spans={$spans}>ABC</text>
  <text id="Plain">ABC</text>
</column>
<style>
.parent { --width: 32px !important; --saved: var(--width); --a: var(--b); --b: var(--a); --ramp: var(--text-xl3); }
#Root { --width: 73px; width: 300px; }
.size { --width: 97px; width: var(--saved); }
:where(#Child) { width: 122px; }
#Fallback { width: var(--a, 58px); }
#Invalid { opacity: 0.25; }
#Invalid { opacity: var(--missing); }
#Branch { --width: 44px; }
#Nested, #Sibling { width: var(--width); }
#Rich::part(emphasis) { font: var(--ramp); }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("cascade.svelte", source).unwrap());
    ui.define_document(
        UiDocument::parse(
            "badge.svelte",
            r#"<meta name="component" content="Badge">
<button id="Badge">Badge</button><style>#Badge { width: var(--width); }</style>"#,
        )
        .unwrap(),
    );
    ui.push_screen(
        "cascade",
        json!({"wide":"84px","spans":[{"name":"emphasis","position":0,"length":3,"properties":{}}]}),
    )
    .unwrap();
    // A diagnostic is reported in the frame its element resolves.
    let mut warnings = Vec::new();
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default());
        warnings.extend(ui.frame().warnings.iter().cloned());
    }
    for (path, width) in [
        ("Child", 32.0),
        ("Fallback", 58.0),
        ("Branch/Nested", 44.0),
        ("Sibling", 32.0),
        ("Component", 32.0),
        ("Override", 66.0),
        ("Bound", 84.0),
    ] {
        assert_eq!(
            ui.element(&format!("cascade/Root/{path}")).unwrap().width,
            width,
            "{path}"
        );
    }
    assert!(
        ui.element("cascade/Root/Rich").unwrap().height
            > ui.element("cascade/Root/Plain").unwrap().height
    );
    assert!(
        warnings
            .iter()
            .any(|e| e.contains("--a") && e.contains("cycle"))
    );
    assert!(
        warnings
            .iter()
            .any(|e| e.contains("opacity") && e.contains("--missing"))
    );
}

/// A component is styled through what it declares: its props, its parts
/// and the custom properties it reads, which an instance sets with a style
/// prop. A user's rule does not reach its root.
#[test]
fn a_document_rule_stops_at_a_component_instance() {
    let parse = |body: &str| {
        UiDocument::parse(
            "user.svelte",
            &format!(r#"<meta name="component" content="user">{body}"#),
        )
        .map(|_| ())
        .map_err(|e| e.join("\n"))
    };
    for rule in ["#Mine", ".loud", "Badge", "#Root > Badge", "[data-tone]"] {
        let error = parse(&format!(
            r#"<column id="Root"><Badge id="Mine" class="loud" data-tone="x"/></column>
<style>{rule} {{ --width: 66px; }}</style>"#
        ))
        .unwrap_err();
        assert!(
            error.contains("selects a <Badge> instance") && error.contains("--name"),
            "{rule}: {error}"
        );
    }
    // A rule that only sweeps over it passes it by.
    parse(
        r#"<column id="Root"><Badge id="Mine"/></column>
<style>* { --width: 66px; } #Root > * { --width: 66px; }</style>"#,
    )
    .unwrap();
    // One spelling each: `--name` on an instance, `style:--name` on an element.
    let error = parse(r#"<column><Badge style:--width="66px"/></column>"#).unwrap_err();
    assert!(error.contains("write --width="), "{error}");
    let error = parse(r#"<column --width="66px"></column>"#).unwrap_err();
    assert!(error.contains("write style:--width="), "{error}");
}

#[test]
fn boolean_attribute_selectors_follow_svelte_presence() {
    let source = r#"<meta name="component" content="t">
<div>
<div id="Shown" hidden={false}>show</div><div id="Hidden" hidden={true}>hide</div>
<button id="Disabled" disabled={$on}>button</button>
<input id="Checked" type="checkbox" checked={$on}/>
<details id="Open" open={$on}><summary>details</summary></details>
<div id="Data" data-value={false} aria-atomic={false}></div>
<div id="Ancestor"><button disabled={$on}>child</button></div>
</div>
<style>
#Shown, #Hidden { height: 20px; }
[hidden] { display: none; }
#Disabled, #Checked, #Open, #Data, #Ancestor { width: 100px; min-width: 0; }
#Disabled[disabled], #Checked[checked], #Open[open], #Data[data-value="false"][aria-atomic="false"], #Ancestor:has(> [disabled]) { width: 120px; }
#Disabled[disabled="false"], #Checked[checked="false"], #Open[open="false"] { width: 190px; }
#Checked { height: 24px; min-height: 0; }
#Checked:default { height: 40px; }
</style>"#;
    crate::support::with_doc(source, &[("on", json!(false))], |d| {
        assert!(d.el("Shown").is_some(), "false hidden is absent");
        assert!(d.el("Hidden").is_none(), "true hidden is present");
        assert_eq!(
            d.get("Data").width,
            120.0,
            "data false is not a boolean attribute"
        );
        for on in [false, true, false] {
            d.set("on", json!(on));
            for id in ["Disabled", "Checked", "Open", "Ancestor"] {
                assert_eq!(
                    d.get(id).width,
                    if on { 120.0 } else { 100.0 },
                    "{id}, on={on}"
                );
            }
            assert_eq!(
                d.get("Checked").height,
                if on { 40.0 } else { 24.0 },
                ":default on={on}"
            );
        }
    });
}

#[test]
fn inline_bindings_and_inheritance_are_isolated_from_author_variables() {
    let source = r#"<meta name="screen" content="isolated" data-layer="hud">
<div id="Root" style:width={$width} style:color={$color} style:--author={$width}>
  <div id="Inherited" />
  <div id="Author" />
  <div id="Normal" style:width={$width} />
  <div id="Important" style:width|important={$width} />
  <div id="Optional" style:width={$optional} />
  <text id="InheritedColor">inherited</text>
  <text id="BoundColor" style:color={$color}>bound</text>
  <text id="HiddenColor">hidden</text>
</div>
<style>
#Root { --og-v-width: 999px; --og-v-color: red; --og-style-color: red; }
#Root { --og-style-width: 999px; width: 14px; }
#Inherited { width: inherit; height: 10px; }
#Author { width: var(--author); height: 10px; }
#Normal, #Important { width: 80px !important; height: 10px; }
#Optional { width: 64px; height: 10px; }
#InheritedColor { color: inherit; }
#BoundColor { --og-style-color: red !important; }
#HiddenColor { color: var(--og-v-color, #00ff00); }
@layer overrides { #Important { width: 93px !important; } }
</style>"#;
    let mut ui = ui_with_font(test_font());
    ui.define_document(UiDocument::parse("isolated.svelte", source).unwrap());
    mount_screen(
        &mut ui,
        "isolated",
        &[
            ("width", json!("138px")),
            ("color", json!("#0000ff")),
            ("optional", json!("104px")),
        ],
    );
    for (width, color, rgb) in [
        (138.0, "#0000ff", [0.0, 0.0, 1.0, 1.0]),
        (220.0, "#ff0000", [1.0, 0.0, 0.0, 1.0]),
    ] {
        set_store(&mut ui, "width", json!(format!("{width}px")));
        set_store(&mut ui, "color", json!(color));
        set_store(
            &mut ui,
            "optional",
            json!(if width == 138.0 { Some("104px") } else { None }),
        );
        for _ in 0..3 {
            tick(&mut ui, InputFrame::default());
        }
        for id in ["Root", "Root/Inherited", "Root/Author", "Root/Important"] {
            assert_eq!(
                ui.element(&format!("isolated/{id}")).unwrap().width,
                width,
                "{id}"
            );
        }
        assert_eq!(ui.element("isolated/Root/Normal").unwrap().width, 80.0);
        assert_eq!(
            ui.element("isolated/Root/Optional").unwrap().width,
            if width == 138.0 { 104.0 } else { 64.0 }
        );
        for (label, expected) in [
            ("inherited", rgb),
            ("bound", rgb),
            ("hidden", [0.0, 1.0, 0.0, 1.0]),
        ] {
            let actual = ui.frame().items.iter().find_map(|item| {
                let cmd = match &item.cmd {
                    DrawCmd::StyledText { text, .. } => text.as_ref(),
                    cmd => cmd,
                };
                match cmd {
                    DrawCmd::Text { text, color, .. } if text == label => Some(*color),
                    _ => None,
                }
            });
            assert_eq!(actual, Some(expected), "{label}");
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }
}

#[test]
fn scoped_css_motion_uses_shared_curves_emits_sound_and_restores() {
    let mut ui = Ui::new(UiDesc::default());
    let source = r#"<meta name="screen" content="motion" data-layer="hud">
<panel id="Card"/>
<style>
#Card { width: 40px; height: 20px; animation: reveal 800ms cubic-bezier(0,1,0,1) forwards; }
@keyframes reveal {
  from { translate: 0 0; -og-sound: error; }
  to { translate: 80px 0; }
}
</style>"#;
    ui.define_document(UiDocument::parse("motion.svelte", source).unwrap());
    ui.push_screen("motion", Value::Null).unwrap();
    let mut sounds = 0;
    for _ in 0..7 {
        tick(&mut ui, InputFrame::default());
        sounds += ui
            .events()
            .iter()
            .filter(|e| e.kind == "sound" && e.name == "error")
            .count();
    }
    // x=u³ and y=1-(1-u)³: at 1/8 duration, y=7/8.
    assert!((ui.element("motion/Card").unwrap().x - 70.0).abs() < 0.001);
    assert_eq!(sounds, 1);
    let saved = ui.save_state();
    let continued: Vec<_> = (0..12)
        .map(|_| {
            tick(&mut ui, InputFrame::default());
            ui.element("motion/Card").unwrap().x
        })
        .collect();
    ui.restore_state(&saved).unwrap();
    for expected in continued {
        tick(&mut ui, InputFrame::default());
        assert_eq!(ui.element("motion/Card").unwrap().x, expected);
        assert!(
            ui.events().is_empty(),
            "restoring does not replay the entrance sound"
        );
    }
}

#[test]
fn keyed_document_rows_keep_identity_after_reorder_and_build_the_empty_branch() {
    let document = UiDocument::parse(
        "rows.svelte",
        r#"<meta name="screen" content="rows">
<column id="Root">
  {#each $items as item (item.id)}
    <button id="Row" onclick={choose}>{item.name}</button>
  {:else}
    <text id="Empty">No rows</text>
  {/each}
</column>
"#,
    )
    .unwrap();

    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(document);
    let a = json!({"id": "sword", "name": "Sword"});
    let b = json!({"id": "shield", "name": "Shield"});
    mount_screen(&mut ui, "rows", &[("items", json!([a, b]))]);
    tick(&mut ui, InputFrame::default());
    assert!(
        ui.doc_built()
            .iter()
            .any(|e| e.path == "rows/Root/Row#sword")
    );
    assert!(!ui.doc_built().iter().any(|e| e.path.ends_with("/Empty")));

    edit_store(
        &mut ui,
        "items",
        json!({"op": "move", "key": "shield", "before": "sword"}),
    );
    tick(&mut ui, InputFrame::default());
    let paths: Vec<_> = ui
        .doc_built()
        .into_iter()
        .map(|e| e.path)
        .filter(|path| path.contains("Row#"))
        .collect();
    assert_eq!(paths, ["rows/Root/Row#shield", "rows/Root/Row#sword"]);

    for key in ["shield", "sword"] {
        edit_store(&mut ui, "items", json!({"op": "remove", "key": key}));
    }
    tick(&mut ui, InputFrame::default());
    assert!(ui.doc_built().iter().any(|e| e.path == "rows/Root/Empty"));
    assert!(!ui.doc_built().iter().any(|e| e.path.contains("Row#")));
}

#[test]
fn length_tokens_follow_the_active_style_in_layout_and_states() {
    let source = r#"<meta name="screen" content="tokens" data-layer="hud">
<column id="Root">
  <button id="Child">Token</button>
  <grid id="Tracks" />
</column>
<style>
#Root {
  width: stretch; min-width: var(--size-sm); max-width: var(--size-xl);
  padding: var(--space-xs) 3px var(--space-md) 7px;
  gap: var(--space-sm);
  background: var(--panel);
  border-radius: var(--radius-lg);
  translate: var(--space-xs) var(--space-sm);
}
  #Child {
    height: var(--size-lg);
    border-radius: var(--radius-sm);
  }
    #Child:hover {
      border-radius: var(--radius-xl);
      translate: var(--space-md) 0;
    }
  #Tracks {
    grid-template-columns: var(--size-sm) minmax(var(--size-xs), var(--size-lg));
    grid-template-rows: var(--size-md);
  }
</style>
"#;
    let mut symbolic = ui_with_font(test_font_metrics());
    symbolic.define_document(UiDocument::parse("tokens.ogt", source).unwrap());
    symbolic.push_screen("tokens", Value::Null).unwrap();
    for factor in [1.0, 2.0] {
        let mut theme = Theme {
            reduced_motion: true,
            ..Theme::default()
        };
        for v in &mut theme.tokens.space {
            *v *= factor;
        }
        for v in &mut theme.tokens.size {
            *v *= factor;
        }
        for v in &mut theme.tokens.radius {
            *v *= factor;
        }
        symbolic.set_theme(theme);
        let mut literal = source.to_string();
        for (name, value) in [
            ("space-xs", 4.0),
            ("space-sm", 8.0),
            ("space-md", 12.0),
            ("size-xs", 16.0),
            ("size-sm", 24.0),
            ("size-md", 32.0),
            ("size-lg", 48.0),
            ("size-xl", 64.0),
            ("radius-sm", 6.0),
            ("radius-lg", 18.0),
            ("radius-xl", 24.0),
        ] {
            literal = literal.replace(&format!("var(--{name})"), &format!("{}px", value * factor));
        }
        let mut expected = ui_with_font(test_font_metrics());
        expected.set_theme(theme);
        expected.define_document(UiDocument::parse("tokens.ogt", &literal).unwrap());
        expected.push_screen("tokens", Value::Null).unwrap();
        for _ in 0..3 {
            tick(&mut symbolic, InputFrame::default());
            tick(&mut expected, InputFrame::default());
        }
        assert_eq!(
            symbolic.frame().items,
            expected.frame().items,
            "style factor {factor}"
        );
        let button = expected.element("tokens/Root/Child").unwrap();
        let (x, y) = (
            button.x + button.width * 0.5,
            button.y + button.height * 0.5,
        );
        for _ in 0..3 {
            tick(&mut symbolic, InputFrame::hover(x, y));
            tick(&mut expected, InputFrame::hover(x, y));
        }
        assert_eq!(
            symbolic.frame().items,
            expected.frame().items,
            "hover, style factor {factor}"
        );
    }
    for (original, missing) in [
        ("--radius-lg", "--radius-missing"),
        ("--panel", "--missing-colour"),
    ] {
        let doc = UiDocument::parse("bad.svelte", &source.replace(original, missing)).unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(doc);
        ui.push_screen("tokens", Value::Null).unwrap();
        // Reported every frame, not only the one the element resolves in.
        for frame in 0..3 {
            tick(&mut ui, InputFrame::default());
            assert!(
                ui.frame()
                    .warnings
                    .iter()
                    .any(|error| error.contains(missing) && error.contains("computed-value")),
                "frame {frame}: {:?}",
                ui.frame().warnings
            );
        }
    }
}

/// A \-og-burst\ whose \ar()\ does not resolve is reported every frame
/// the element keeps it, as its other style warnings are.
#[test]
fn an_unresolved_burst_variable_is_reported_every_frame() {
    let source = r#"<meta name="screen" content="bursts" data-layer="hud">
<column id="Root"><button id="Shell">Pop</button></column>
<style>
@keyframes pop { 50% { -og-burst: var(--burst-missing); } }
#Shell { animation: pop 1s infinite; }
</style>
"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("bursts.svelte", source).unwrap());
    ui.push_screen("bursts", Value::Null).unwrap();
    // Once each frame: not again in a frame that builds the element.
    for frame in 0..3 {
        tick(&mut ui, InputFrame::default());
        let said = ui.frame().warnings.iter();
        assert_eq!(
            said.filter(|w| w.contains("-og-burst") && w.contains("--burst-missing"))
                .count(),
            1,
            "frame {frame}: {:?}",
            ui.frame().warnings
        );
    }
}

fn key(code: &'static str) -> InputFrame {
    let ui_key = UiKey::ALL.into_iter().find(|k| k.code() == code);
    InputFrame {
        keys: ui_key.into_iter().collect(),
        keys_held: vec![code],
        keys_pressed: vec![code],
        ..InputFrame::default()
    }
}

/// The stack is ordered by layer kind, as CommonUI's layer groups are, and
/// the active layer is the top one that takes input, not the topmost: a
/// toast over a menu leaves Back, focus and pop with the menu, a modal
/// stays over a menu pushed after it, and a layer hears when it becomes
/// and stops being the active one.
#[test]
fn the_active_layer_is_the_top_input_taker_of_a_kind_ordered_stack() {
    let mut ui = ui();
    let screen = |name: &str, layer: &str| {
        format!(
            "<meta name=\"screen\" content=\"{name}\" data-layer=\"{layer}\"><column id=\"Root\"><button id=\"Ok\">Ok</button></column>"
        )
    };
    for (n, l) in [("notice", "toast"), ("confirm", "modal")] {
        let src = screen(n, l);
        ui.define_document(UiDocument::parse(&format!("ui/{n}.ogt"), &src).unwrap());
    }
    let names =
        |ui: &Ui| -> Vec<String> { ui.layer_stack().into_iter().map(|l| l.screen).collect() };
    ui.push_screen("hud", json!({ "gold": 1 })).unwrap();
    ui.push_screen("shop", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    ui.take_doc_events();
    // A toast over the menu: the menu keeps Back and focus.
    ui.push_screen("notice", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "shop/Shop/Footer/Buy");
    // A modal pushed now sits under the toast and over the menu.
    ui.push_screen("confirm", Value::Null).unwrap();
    assert_eq!(names(&ui), ["hud", "shop", "confirm", "notice"]);
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "confirm/Root/Ok", "the modal is active");
    let events = ui.take_doc_events();
    let fired = |kind: &str| -> Vec<String> {
        events
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.mount.clone())
            .collect()
    };
    assert_eq!(fired("deactivate"), ["shop"], "{events:?}");
    assert_eq!(fired("activate"), ["confirm"], "{events:?}");
    // A menu pushed after the modal goes under it.
    let src = screen("options", "menu");
    ui.define_document(UiDocument::parse("ui/options.ogt", &src).unwrap());
    ui.push_screen("options", Value::Null).unwrap();
    assert_eq!(names(&ui), ["hud", "shop", "options", "confirm", "notice"]);
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(
        ui.focus_path(0),
        "confirm/Root/Ok",
        "the modal stays active"
    );
    // Back pops the modal, not the toast, then the menu under it; the shop
    // returns with its focus.
    ui.take_doc_events();
    for _ in 0..2 {
        tick(&mut ui, key("Escape"));
        tick(&mut ui, InputFrame::default());
        tick(&mut ui, InputFrame::default());
        assert!(!ui.back(), "a menu layer took Back");
    }
    assert_eq!(names(&ui), ["hud", "shop", "notice"]);
    assert_eq!(ui.focus_path(0), "shop/Shop/Footer/Buy", "focus memory");
    let activated: Vec<String> = ui
        .take_doc_events()
        .into_iter()
        .filter(|e| e.kind == "activate")
        .map(|e| e.mount)
        .collect();
    assert_eq!(activated, ["options", "shop"]);
    // pop_screen pops the active layer too.
    assert_eq!(ui.pop_screen().as_deref(), Some("shop"));
}

#[test]
fn dump_ui_names_each_elements_layer_source_bindings_and_states() {
    let mut ui = ui();
    ui.push_screen("hud", json!({ "gold": 1500 })).unwrap();
    let vm = UiDocument::parse("x", SHOP)
        .unwrap()
        .fixture("rich")
        .cloned()
        .unwrap();
    ui.push_screen("shop", vm).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    ui.set_doc_prop("shop/Shop/Header/Title", "text", "\"Bazaar\"")
        .unwrap();
    tick(&mut ui, InputFrame::default());
    let els = ui.probe_elements();
    let find = |p: &str| els.iter().find(|e| e.path == p).unwrap().to_json();
    // Same id under two layers: each answers with its own layer and line.
    let shop_gold = find("shop/Shop/Header/Gold");
    let hud_gold = find("hud/Bar/Gold");
    assert_eq!(shop_gold["layer"], json!("shop"));
    assert_eq!(hud_gold["layer"], json!("hud"));
    assert_eq!(
        shop_gold["source"],
        json!("ui/shop.svelte:14"),
        "{shop_gold}"
    );
    assert_eq!(hud_gold["source"], json!("ui/hud.svelte:5"), "{hud_gold}");
    assert_eq!(
        shop_gold["bindings"],
        json!([{ "prop": "text", "expr": "{formatCompact(gold)}", "value": "1.25M" }]),
    );
    assert_eq!(
        find("shop/Shop/Header/Title")["edits"],
        json!({ "text": "\"Bazaar\"" })
    );
    assert!(
        ui.doc_built()
            .iter()
            .any(|e| e.path == "shop/Shop/Items/Item#shield")
    );
    let extra = ui.probe_extra();
    let layers = extra["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[1]["screen"], json!("shop"));
    assert_eq!(layers[1]["focused"], json!("shop/Shop/Footer/Buy"));
    assert_eq!(
        extra["edits"],
        json!([{ "path": "shop/Shop/Header/Title", "prop": "text", "value": "\"Bazaar\"" }])
    );
}

/// The CSS dialect is a strict subset: an unknown property or a value without its unit is
/// an error that names the form to write.
#[test]
fn css_outside_the_subset_is_an_error_that_names_the_fix() {
    let err = |body: &str| {
        let t = format!(
            "<meta name=\"screen\" content=\"s\"><column id=\"Root\"/><style>#Root {{ {body} }}</style>"
        );
        UiDocument::parse("ui/s.svelte", &t).unwrap_err().join("\n")
    };
    assert!(err("  float: center").contains("inline-start"));
    assert!(err("  margin: 4px 2px 1px 0 9px").contains("1 to 4"));
    assert!(err("  gap: 12").contains("12px"));
    assert!(err("  font-size: big").contains("font-size: 'big'"));
    assert!(err("  padding: 4px !urgent").contains("declaration"));
    assert!(err("-og-frame: rive(\"ui/chest.riv#Chest\")").contains("-og-frame"));
    for bad in ["ui/chest.riv", "chest", "ui/chest.riv#", "ui/chest.riv#A#B"] {
        let source = format!("<meta name=\"component\" content=\"s\"><og-rive src=\"{bad}\"/>");
        let e = UiDocument::parse("ui/s.svelte", &source)
            .unwrap_err()
            .join("\n");
        assert!(e.contains("src takes <file>.riv#<Artboard>"), "{bad}: {e}");
    }
    // Container syntax outside the specs names what is wrong.
    assert!(err("  container-type: size scroll-state size").contains("container-type"));
    let rule = |rule: &str| {
        let t = format!(
            "<meta name=\"screen\" content=\"s\"><column id=\"Root\"/><style>{rule}</style>"
        );
        UiDocument::parse("ui/s.svelte", &t).unwrap_err().join("\n")
    };
    for (bad, needle) in [
        (
            "@container style(color: red) { #Root { gap: 4px; } }",
            "custom properties",
        ),
        ("@container (width > 20cqw) { #Root { gap: 4px; } }", "cqw"),
        (
            "@container (prefers-reduced-motion) { #Root { gap: 4px; } }",
            "prefers-reduced-motion",
        ),
    ] {
        let e = rule(bad);
        assert!(e.contains(needle), "{bad}: {e}");
    }
}

/// Probe edits preserve authored structure and existing stylesheet rules.
#[test]
fn a_prop_edit_writes_a_css_declaration_into_a_svelte_document() {
    const DOC: &str = "<meta name=\"screen\" content=\"s\">\n<column id=\"Root\"><text id=\"Title\">Hi</text></column>\n<style>#Root { width: stretch; max-width: 640px; translate: 0 -4px; }</style>\n";
    let mut doc = UiDocument::parse("ui/s.svelte", DOC).unwrap();
    let root = doc.nodes.iter().position(|n| n.id == "Root").unwrap();
    doc.set_prop(root, "x", "6").unwrap();
    doc.set_prop(root, "gap", "12").unwrap();
    let title = doc.nodes.iter().position(|n| n.id == "Title").unwrap();
    doc.set_prop(title, "color", "\"#ffcc00\"").unwrap();
    let t = &doc.text;
    assert!(t.contains("translate: 6px -4px"), "{t}");
    assert!(t.contains("gap: 12px"), "{t}");
    assert!(t.contains("width: stretch; max-width: 640px"), "{t}");
    assert!(t.contains("color: #ffcc00"), "{t}");
    assert!(t.contains(">Hi</text>"), "{t}");
}

/// A Rive artboard of 100 × 50: a 40 × 20 red rectangle `Lid` centred at
/// x 25 that an animation moves to x 75 over a second, looping.
fn chest() -> std::sync::Arc<og_ui::rive::Artboard> {
    use og_ui::rive::*;
    let node = |name: &str, parent, kind, values| Node {
        name: name.into(),
        parent,
        kind,
        values,
    };
    let key = |seconds, value| Key {
        seconds,
        value,
        color: 0,
        ease: Ease::Linear,
    };
    std::sync::Arc::new(Artboard {
        name: "Chest".into(),
        size: og_core::glam::Vec2::new(100.0, 50.0),
        fit: Fit::None,
        clip: true,
        nodes: vec![
            node(
                "Chest",
                0,
                Kind::Artboard { paints: vec![] },
                Values::default(),
            ),
            node(
                "Lid",
                0,
                Kind::Shape {
                    paths: vec![2],
                    paints: vec![3],
                    hidden: false,
                    blend: og_draw::UiBlend::Normal,
                },
                Values {
                    x: 25.0,
                    y: 25.0,
                    ..Values::default()
                },
            ),
            node(
                "box",
                1,
                Kind::Path {
                    outline: Outline::Rect,
                    closed: true,
                },
                Values {
                    width: 40.0,
                    height: 20.0,
                    ..Values::default()
                },
            ),
            node(
                "fill",
                1,
                Kind::Fill {
                    rule: og_draw::UiFillRule::NonZero,
                    shader: 4,
                    visible: true,
                },
                Values::default(),
            ),
            node(
                "red",
                3,
                Kind::Solid,
                Values {
                    color: 0xFFFF_0000,
                    ..Values::default()
                },
            ),
        ],
        draws: vec![1],
        animations: vec![Animation {
            name: "open".into(),
            fps: 60,
            duration: 60,
            speed: 1.0,
            looping: Loop::Loop,
            work: None,
            tracks: vec![Track {
                node: 1,
                prop: Prop::X,
                keys: vec![key(0.0, 25.0), key(1.0, 75.0)],
            }],
        }],
        play: Some(0),
        ..Artboard::default()
    })
}

const CHEST_SCREEN: &str = "<meta name=\"screen\" content=\"loot\"><og-rive id=\"Chest\" src=\"ui/chest.riv#Chest\"/><style>#Chest { width: 100px; height: 50px; }</style>";

/// A document element whose frame is a Rive artboard plays it on the tick
/// in its box: its shapes draw over its background, its objects are
/// elements with the artboard as their source, and its player is saved
/// with the UI's state and runs on after a restore as if never stopped.
///
/// Ways it could fail: a player advanced per build rather than per tick or
/// restarted each frame (the lid never moves or jumps), a restore that
/// starts it again (the restored run lags the unbroken one), refused bytes
/// silently reset, or the write-back losing its src attribute.
#[test]
fn a_rive_frame_plays_draws_names_its_objects_and_is_saved() {
    let doc = UiDocument::parse("ui/loot.ogt", CHEST_SCREEN).expect("parses");
    assert_eq!(doc.rive_ids(), ["ui/chest.riv#Chest"]);
    let fresh = |board: Option<std::sync::Arc<og_ui::rive::Artboard>>| {
        let mut ui = ui_with_font(test_font_metrics());
        if let Some(b) = board {
            ui.define_rive("ui/chest.riv#Chest", b);
        }
        ui.define_document(doc.clone());
        ui.push_screen("loot", Value::Null).unwrap();
        ui
    };
    // Where the lid is drawn: its shape's centre, x.
    let lid = |ui: &Ui| -> Vec<f32> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. }
                    if matches!(shape.geometry, og_draw::UiGeometry::Path(_)) =>
                {
                    Some(shape.center.x)
                }
                _ => None,
            })
            .collect()
    };
    let mut whole = fresh(Some(chest()));
    tick(&mut whole, InputFrame::default());
    let at = whole
        .probe_elements()
        .into_iter()
        .find(|e| e.path == "loot/Chest")
        .expect("the element");
    let first = lid(&whole);
    assert_eq!(first.len(), 1, "one shape, drawn once");
    for _ in 0..30 {
        tick(&mut whole, InputFrame::default());
    }
    let moved = lid(&whole);
    // 30 ticks of 1/60 s: half way, 25 → 50 in the artboard, at the box's x.
    assert!(
        (moved[0] - first[0] - 25.0).abs() < 0.6,
        "{first:?} → {moved:?}"
    );
    let els = whole.probe_elements();
    let obj = els
        .iter()
        .find(|e| e.kind == "rive")
        .expect("the lid as an element");
    assert_eq!(obj.path, "loot/Chest/Chest/Lid");
    assert_eq!(obj.detail["source"], json!("ui/chest.riv#Chest/Lid"));
    assert!(obj.x >= at.x && obj.x + obj.width <= at.x + at.width + 0.01);

    // Saved, run on, and restored into a UI that knows the artboard: the
    // next ticks match the unbroken run's.
    let saved = whole.save_state();
    for _ in 0..10 {
        tick(&mut whole, InputFrame::default());
    }
    let mut back = fresh(Some(chest()));
    back.restore_state(&saved).expect("restores");
    for _ in 0..10 {
        tick(&mut back, InputFrame::default());
    }
    assert_eq!(
        lid(&back),
        lid(&whole),
        "it runs on from where it was saved"
    );

    // Restored where the artboard is another: an error, nothing drawn, and
    // the bytes kept for the save rather than reset.
    let other = std::sync::Arc::new(og_ui::rive::Artboard {
        animations: Vec::new(),
        play: None,
        ..(*chest()).clone()
    });
    let mut wrong = fresh(Some(other));
    wrong.restore_state(&saved).expect("the UI state reads");
    tick(&mut wrong, InputFrame::default());
    assert!(lid(&wrong).is_empty(), "not drawn");
    assert!(
        wrong
            .frame()
            .warnings
            .iter()
            .any(|w| w.contains("does not fit")),
        "reported"
    );
    let mut again = fresh(Some(chest()));
    again.restore_state(&wrong.save_state()).expect("restores");
    tick(&mut again, InputFrame::default());
    let mut check = fresh(Some(chest()));
    check.restore_state(&saved).expect("restores");
    tick(&mut check, InputFrame::default());
    assert_eq!(lid(&again), lid(&check), "the refused bytes were kept");

    // No artboard of that id: reported, nothing drawn.
    let mut none = fresh(None);
    tick(&mut none, InputFrame::default());
    assert!(lid(&none).is_empty());
    assert!(
        none.frame()
            .warnings
            .iter()
            .any(|w| w.contains("names no Rive artboard"))
    );

    // An edit writes the frame back in the form it is written in.
    let mut edited = doc.clone();
    let chest_node = edited.nodes.iter().position(|n| n.id == "Chest").unwrap();
    edited
        .set_prop(chest_node, "rive", "\"ui/chest.riv#Lid\"")
        .unwrap();
    assert!(
        edited.text.contains("src=\"ui/chest.riv#Lid\""),
        "{}",
        edited.text
    );
    // Replacing the attribute removes the old dependency.
    assert_eq!(edited.rive_ids(), ["ui/chest.riv#Lid"]);
}

#[test]
fn pushed_screens_match_sibling_and_structural_selectors_on_instantiated_rows() {
    let document = UiDocument::parse(
        "structure.svelte",
        r#"<meta name="screen" content="structure">
<column id="Root">
  {#each $items as item (item.id)}<text id="Item" class={item.kind}>{item.id}</text>{/each}
  {#if $note}<text id="Note">{$note}</text>{/if}
  <column id="Box"></column>
</column>
<style>
  text:nth-child(2n+1 of .gem) { width: 111px; }
  text:nth-last-child(2) { width: 222px; }
  #Note ~ #Box:empty { height: 17px; }
</style>
"#,
    )
    .unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(document);
    let row = |id: &str, kind: &str| json!({"id": id, "kind": kind});
    let measure = |ui: &Ui, text: &str| {
        let e = ui.probe_elements();
        let e = e.iter().find(|e| e.text == text || e.id == text).unwrap();
        (e.width.round(), e.height.round())
    };
    let (g1, c1, g2, g3) = (
        row("g1", "gem"),
        row("c1", "coin"),
        row("g2", "gem"),
        row("g3", "gem"),
    );
    mount_screen(
        &mut ui,
        "structure",
        &[("items", json!([g1, c1, g2, g3])), ("note", json!("Note"))],
    );
    tick(&mut ui, InputFrame::default());
    // Gem ranks 1, 2, 3 (the coin is not counted); the note is second last.
    assert_eq!(measure(&ui, "g1").0, 111.0);
    assert_eq!(measure(&ui, "g3").0, 111.0);
    assert_ne!(measure(&ui, "g2").0, 111.0);
    assert_eq!(measure(&ui, "Note").0, 222.0);
    assert_eq!(measure(&ui, "Box").1, 17.0);
    // The order becomes c1, g3, g1, g2, and the note goes.
    for (key, before) in [("c1", "g1"), ("g3", "g1")] {
        edit_store(
            &mut ui,
            "items",
            json!({"op": "move", "key": key, "before": before}),
        );
    }
    set_store(&mut ui, "note", Value::Null);
    tick(&mut ui, InputFrame::default());
    // Without the note, g2 is second last but its filtered rank wins.
    assert_eq!(measure(&ui, "g3").0, 111.0);
    assert_eq!(measure(&ui, "g2").0, 111.0);
    let (g1_width, c1_width) = (measure(&ui, "g1").0, measure(&ui, "c1").0);
    assert!(g1_width != 111.0 && g1_width != 222.0, "{g1_width}");
    assert!(c1_width != 111.0 && c1_width != 222.0, "{c1_width}");
    assert_ne!(measure(&ui, "Box").1, 17.0);
}

/// A pushed viewmodel's list is iterated by its rows' `id`s: rows that
/// each have a unique string `id` are a keyed list, and a screen that
/// iterates any other list is told what the rows need.
///
/// Ways it could fail: rows with `id`s held as one value (the each finds
/// no keyed list), a list without `id`s drawn empty with no reason given.
#[test]
fn a_pushed_viewmodel_s_list_is_iterated_by_its_rows_ids() {
    let screen = |name: &str| {
        UiDocument::parse(
            &format!("{name}.svelte"),
            &format!(
                r#"<meta name="screen" content="{name}">
<column id="Root">
  {{#each $rows as row (row.id)}}<text id="Row">{{row.name}}</text>{{/each}}
</column>
"#
            ),
        )
        .unwrap()
    };
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(screen("keyed"));
    ui.define_document(screen("plain"));
    ui.push_screen(
        "keyed",
        json!({"rows": [{"id": "a", "name": "ash"}, {"id": "o", "name": "oak"}]}),
    )
    .unwrap();
    tick(&mut ui, InputFrame::default());
    let texts: Vec<String> = ui
        .probe_elements()
        .iter()
        .filter(|e| !e.text.is_empty())
        .map(|e| e.text.clone())
        .collect();
    assert_eq!(texts, ["ash", "oak"]);
    ui.pop_screen();
    tick(&mut ui, InputFrame::default());
    ui.push_screen("plain", json!({"rows": [{"name": "ash"}, {"name": "oak"}]}))
        .unwrap();
    ui.begin(InputFrame::default(), viewport());
    let refused = ui.build_retained_documents(&bind::NoEnv).unwrap_err();
    ui.end();
    assert_eq!(
        refused,
        "plain.svelte:3: each over '$rows' needs a keyed list; give each of its rows a unique string id"
    );
}

/// A screen that declares no props reads its viewmodel's fields bare
/// (`{title}`), as one that declares them does, and a field the push
/// leaves out reads nothing.
///
/// Ways it could fail: a bare name refused as no row variable (the screen
/// draws nothing), an item variable or a resource taken for a field, a
/// field left out refused.
#[test]
fn a_screen_without_props_reads_its_viewmodel_s_fields_bare() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "plain.svelte",
            r#"<meta name="screen" content="plain">
<column id="Root">
  <text id="Title">{title}</text>
  {#if note}<text id="Note">{note}</text>{/if}
  {#each rows as row (row.id)}<text id="Row">{row.name}</text>{/each}
</column>
"#,
        )
        .unwrap(),
    );
    ui.push_screen(
        "plain",
        json!({"title": "Plain", "rows": [{"id": "a", "name": "ash"}]}),
    )
    .unwrap();
    ui.begin(InputFrame::default(), viewport());
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let texts: Vec<String> = ui
        .probe_elements()
        .iter()
        .filter(|e| !e.text.is_empty())
        .map(|e| e.text.clone())
        .collect();
    assert_eq!(texts, ["Plain", "ash"]);
}

/// A pushed screen's viewmodel is read back as it is now: what was
/// pushed, rows in their order, with what the screen's controls wrote.
///
/// Ways it could fail: the pushed object answered after a control wrote
/// (the write is lost to the game), keyed rows read back out of order or
/// not at all, a screen that is not on the stack answering.
#[test]
fn a_pushed_screen_s_viewmodel_reads_back_with_what_its_controls_wrote() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "options.svelte",
            r#"<meta name="screen" content="options">
<column id="Root"><input type="checkbox" id="On" bind:checked={$on}></column>
"#,
        )
        .unwrap(),
    );
    let pushed = json!({
        "on": false,
        "rows": [{"id": "b", "n": 2}, {"id": "a", "n": 1}],
        "title": "Options",
    });
    assert_eq!(ui.screen_vm("options"), None);
    ui.push_screen("options", pushed.clone()).unwrap();
    assert_eq!(ui.screen_vm("options"), Some(pushed.clone()));
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "options/Root/On");
    tick(&mut ui, key("Space"));
    tick(&mut ui, InputFrame::default());
    let mut written = pushed;
    written["on"] = json!(true);
    assert_eq!(ui.screen_vm("options"), Some(written));
    ui.pop_screen();
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.screen_vm("options"), None);
}

/// Container queries restyle, before paint, the kinds that build
/// differently: a virtual list's and grid's own box, and a Rive frame that
/// starts, stops and starts again with its container's width.
///
/// Ways it could fail: lists and grids without a field baseline (their
/// padding never changes after the first build, or a removed padding
/// sticks), a Rive player advanced twice in a tick or left playing after its
/// frame is queried away, conditions read one frame late.
#[test]
fn container_queries_restyle_lists_grids_and_rive_frames_before_paint() {
    let doc = UiDocument::parse(
        "ui/shelf.ogt",
        r#"<meta name="screen" content="loot">
<column id="Shelf">
  <column id="ChestFrame"><og-rive id="Chest" src="ui/chest.riv#Chest"/></column>
  <list id="Log" row-height="20" height="80">{#each $rows as row (row.id)}<text>{row.id}</text>{/each}</list>
  <cells id="Bag" cell-width="30" cell-height="30" height="70">{#each $rows as row (row.id)}<panel/>{/each}</cells>
</column>
<style>
#Shelf { width: 50%; padding: 0px; container: shelf / inline-size; }
#ChestFrame { width: 100px; height: 50px; }
#Chest { width: 100%; height: 100%; display: none; }
#Log { padding: 2px; }
@container shelf (width >= 400px) {
  #Chest { display: block; }
  #Log { padding: 7px; }
  #Bag { padding: 5px; }
}
</style>"#,
    )
    .expect("parses");
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_rive("ui/chest.riv#Chest", chest());
    ui.define_document(doc);
    let rows: Vec<Value> = ["a", "b", "c", "d", "e", "f", "g"]
        .map(|id| json!({ "id": id }))
        .into();
    ui.push_screen("loot", json!({ "rows": rows })).unwrap();
    let frame = |ui: &mut Ui, width: f32| {
        ui.begin(InputFrame::default(), Viewport::new(width, 600.0));
        ui.end();
    };
    let row = |ui: &Ui, path: &str| -> (f32, f32) {
        let els = ui.probe_elements();
        let first = els
            .iter()
            .find(|e| e.path.starts_with(&format!("loot/Shelf/{path}/")))
            .unwrap_or_else(|| panic!("a {path} item"));
        let me = els
            .iter()
            .find(|e| e.path == format!("loot/Shelf/{path}"))
            .unwrap();
        (first.x - me.x, first.y - me.y)
    };
    let rive_objects = |ui: &Ui| {
        ui.probe_elements()
            .iter()
            .filter(|e| e.kind == "rive")
            .count()
    };
    // 1000 wide: Shelf is 500, so the query holds on the very first frame.
    frame(&mut ui, 1000.0);
    assert_eq!(row(&ui, "Log"), (7.0, 7.0));
    assert_eq!(row(&ui, "Bag"), (5.0, 5.0));
    assert!(rive_objects(&ui) > 0, "the chest plays");
    // 600 wide: Shelf is 300; the list keeps its unconditional 2px, the
    // grid loses its padding, the chest stops.
    frame(&mut ui, 600.0);
    assert_eq!(row(&ui, "Log"), (2.0, 2.0));
    assert_eq!(row(&ui, "Bag"), (0.0, 0.0));
    assert_eq!(rive_objects(&ui), 0, "the chest stopped");
    frame(&mut ui, 820.0);
    assert_eq!(row(&ui, "Log"), (7.0, 7.0));
    assert!(rive_objects(&ui) > 0, "the chest plays again");
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// An element with a trigger on an action is built in a frame the action
/// fires, not every frame: idle frames build nothing, and a fired action
/// reaches its listener, written in its layer's context although its
/// ancestors are not built.
#[test]
fn action_listeners_build_only_when_their_action_fires() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut doc = String::from(
        r#"<meta name="screen" content="pad" data-layer="hud">
<meta name="action:confirm" content="Buy">
<column id="Pad">
  <button id="Buy" onconfirm={buy}>Buy</button>
"#,
    );
    for i in 0..40 {
        doc.push_str(&format!("  <text id=\"Line{i}\">Line {i}</text>\n"));
    }
    doc.push_str("</column>\n");
    ui.define_document(UiDocument::parse("pad.svelte", &doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "pad"),
        "pad",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let frame = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), viewport());
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let emitted: Vec<String> = ui
            .take_doc_events()
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| e.name)
            .collect();
        (ui.frame().stats.reconciled, emitted)
    };
    frame(&mut ui);
    frame(&mut ui);
    assert_eq!(frame(&mut ui), (0, vec![]), "an idle frame builds nothing");
    ui.fire_action("confirm");
    assert_eq!(frame(&mut ui), (1, vec!["buy".to_string()]));
    assert_eq!(frame(&mut ui), (0, vec![]));
}

/// A disabled control's input action activates nothing, as its press does
/// not (docs/ui.md §6.1: disabled emits no activation); an enabled one in
/// the same layer still answers.
#[test]
fn a_disabled_controls_action_sends_nothing() {
    let mut ui = ui_with_font(test_font_metrics());
    let doc = r#"<meta name="screen" content="pad" data-layer="menu">
<meta name="action:confirm" content="Buy">
<meta name="action:details" content="Sell">
<column id="Pad">
  <button id="Buy" disabled onconfirm={buy}>Buy</button>
  <button id="Sell" ondetails={sell}>Sell</button>
</column>
"#;
    ui.define_document(UiDocument::parse("pad.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "pad"),
        "pad",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let emitted = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), viewport());
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        ui.take_doc_events()
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| e.name)
            .collect::<Vec<_>>()
    };
    emitted(&mut ui);
    ui.fire_action("confirm");
    assert_eq!(emitted(&mut ui), Vec::<String>::new());
    ui.fire_action("details");
    assert_eq!(emitted(&mut ui), vec!["sell".to_string()]);
}

/// `aria-disabled` is inert: its button stays in the tree and sends nothing
/// when pressed or by its action, and one beside it still answers.
#[test]
fn an_aria_disabled_button_sends_nothing_when_pressed() {
    let mut ui = ui_with_font(test_font_metrics());
    let doc = r#"<meta name="screen" content="map" data-layer="menu">
<meta name="action:details" content="Look">
<column id="Map">
  <button id="Locked" aria-disabled={true} onclick={pick} ondetails={peek}>Locked</button>
  <button id="Open" onclick={enter} ondetails={look}>Open</button>
</column>
"#;
    ui.define_document(UiDocument::parse("map.svelte", doc).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "map"),
        "map",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, viewport());
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        ui.take_doc_events()
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| e.name)
            .collect::<Vec<_>>()
    };
    step(&mut ui, InputFrame::default());
    let click = |ui: &mut Ui, id: &str| {
        let e = ui.element(id).cloned().unwrap();
        let at = og_core::glam::Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
        let mut sent = Vec::new();
        for (pressed, released) in [(true, false), (false, true)] {
            sent.extend(step(
                ui,
                InputFrame {
                    pointer: at,
                    pointer_inside: true,
                    down: pressed,
                    pressed,
                    released,
                    ..InputFrame::default()
                },
            ));
        }
        sent.extend(step(ui, InputFrame::default()));
        sent
    };
    assert!(ui.element("Locked").unwrap().disabled);
    assert_eq!(click(&mut ui, "Locked"), Vec::<String>::new());
    assert_eq!(click(&mut ui, "Open"), vec!["enter".to_string()]);
    ui.fire_action("details");
    assert_eq!(
        step(&mut ui, InputFrame::default()),
        vec!["look".to_string()]
    );
}

/// A container query that resizes a slider and a disclosure: settlement
/// replaces their fields, and what their widgets build inside (the knob at
/// its value, the summary row) follows the new box in that frame, as a
/// fresh build at that size draws it.
#[test]
fn queried_controls_follow_their_new_box_in_the_frame_it_changes() {
    let mount = || {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            doc::UiDocument::parse(
                "shelf.svelte",
                r#"<meta name="component" content="shelf">
<column id="Shelf">
  <input id="Range" type="range" value="0.4">
  <details id="More" summary="More" open><text>In</text></details>
</column>
<style>
#Shelf { width: 60%; padding: 0px; container-type: inline-size; }
@container (width > 250px) { #Range, #More { width: 180px; } }
</style>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            reactive::SourceId::new("test", "shelf"),
            "shelf",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        ui
    };
    let frame = |ui: &mut Ui, width: f32| {
        ui.begin(InputFrame::default(), Viewport::new(width, 300.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        format!("{:?}", ui.frame().items)
    };
    let mut resized = mount();
    frame(&mut resized, 400.0);
    frame(&mut resized, 400.0);
    let mut fresh = mount();
    frame(&mut fresh, 700.0);
    assert_eq!(frame(&mut resized, 700.0), frame(&mut fresh, 700.0));
}

/// The first frame settles every element that reads its container's
/// box, a reader whose child reads it too among them: each slot (and the
/// stack in it) has its container-unit size in that frame.
///
/// Ways it could fail: a reader resolved only as the parent of another
/// reader keeps the fields of the build before its container was
/// measured (0 × 0 until the next frame).
#[test]
fn a_container_reader_and_its_reading_child_settle_in_the_first_frame() {
    let mut text = String::from("<meta name=\"component\" content=\"hand\">\n<row id=\"Frame\">\n");
    for i in 0..6 {
        text.push_str(&format!(
            "<panel id=\"S{i}\"><panel id=\"T{i}\"/></panel>\n"
        ));
    }
    text.push_str(
        "</row>\n<style>\n\
         #Frame { width: 400px; height: 200px; padding: 0px; gap: 0px; container-type: size; }\n\
         panel { padding: 0px; }\n\
         #Frame > panel { flex: none; width: 10cqw; height: 50cqh; }\n\
         #Frame > panel > panel { width: 5cqw; height: 10px; }\n\
         </style>",
    );
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(doc::UiDocument::parse("hand.svelte", &text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "hand"),
        "hand",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(500.0, 300.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let elements = ui.probe_elements();
    let size = |id: &str| {
        let e = elements
            .iter()
            .find(|e| e.path.ends_with(&format!("/{id}")))
            .unwrap();
        (e.width, e.height)
    };
    for i in 0..6 {
        assert_eq!(size(&format!("S{i}")), (40.0, 100.0), "slot S{i}");
        assert_eq!(size(&format!("T{i}")), (20.0, 10.0), "stack T{i}");
    }
}

/// A Rive frame plays on without building its element, and an element
/// hidden by a store is kept while hidden and shown again by changing its
/// fields; one hidden from the start is built when first shown.
///
/// Ways it could fail: the Rive element rebuilt every frame (idle
/// reconciles 1), its player not advanced when it is not built (the lid
/// stops), a hidden element dropped and never shown again (its lines stay
/// gone), its subtree rebuilt on show, the never-built element's show mark
/// dropped because it holds no node.
#[test]
fn rive_frames_and_hidden_elements_change_fields_not_builds() {
    let mount = |closed_at_start: bool| {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_rive("ui/chest.riv#Chest", chest());
        let mut doc = String::from(
            "<meta name=\"component\" content=\"loot\">\n<column id=\"Root\">\n<og-rive id=\"Chest\" src=\"ui/chest.riv#Chest\"/>\n<column id=\"Menu\" hidden={$closed}>\n",
        );
        for i in 0..30 {
            doc.push_str(&format!("<text id=\"L{i}\">Line {i}</text>\n"));
        }
        doc.push_str("</column>\n</column>\n<style>#Chest { width: 100px; height: 50px; }</style>");
        ui.define_document(UiDocument::parse("loot.svelte", &doc).unwrap());
        let closed = reactive::SourceId::new("test", "closed");
        let mut inputs = std::collections::BTreeMap::new();
        inputs.insert("closed".to_string(), closed.clone());
        set(
            &mut ui,
            json!({"op": "signal", "source": closed, "initial": closed_at_start}),
        );
        ui.mount_document(
            reactive::SourceId::new("test", "loot"),
            "loot",
            inputs,
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        ui
    };
    fn set(ui: &mut Ui, command: Value) {
        reactive::wire::decode(
            &ui.apply_reactive(&reactive::wire::encode(&json!([command])).unwrap()),
        )
        .unwrap();
    }
    let close = |ui: &mut Ui, on: bool| {
        let closed = reactive::SourceId::new("test", "closed");
        set(ui, json!({"op": "set", "source": closed, "value": on}));
    };
    // (reconciled, lines shown, lid x)
    let frame = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), viewport());
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let lines = ui
            .probe_elements()
            .iter()
            .filter(|e| e.path.starts_with("test:loot/Menu/L"))
            .count();
        let lid = ui.frame().items.iter().find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if matches!(shape.geometry, og_draw::UiGeometry::Path(_)) =>
            {
                Some(shape.center.x)
            }
            _ => None,
        });
        (ui.frame().stats.reconciled, lines, lid.unwrap_or(f32::NAN))
    };
    let mut ui = mount(false);
    frame(&mut ui);
    let shown = frame(&mut ui).1;
    assert!(shown > 0);
    let (reconciled, _, a) = frame(&mut ui);
    let (_, _, b) = frame(&mut ui);
    assert_eq!(reconciled, 0, "an idle frame builds nothing, Rive included");
    assert!(b > a, "the lid moves on without a build: {a} → {b}");
    close(&mut ui, true);
    assert_eq!(frame(&mut ui).0..2, 1..2, "hiding changes the one element");
    assert_eq!(frame(&mut ui).1, 0);
    close(&mut ui, false);
    assert_eq!(frame(&mut ui).0..2, 1..2, "showing changes the one element");
    assert_eq!(frame(&mut ui).1, shown);

    let mut ui = mount(true);
    frame(&mut ui);
    assert_eq!(frame(&mut ui).1, 0);
    close(&mut ui, false);
    assert_eq!(frame(&mut ui).1, shown, "built when first shown");
}

/// A scroll-driven animation (`animation-timeline: view()`) follows its
/// subject through its scroller: a channel property (translate) and a
/// computed one (width) at the view progress, with nothing rebuilt while
/// the scroller rests. Ways it could fail: the animation playing on ticks,
/// progress over the scroll range instead of the subject's cover range,
/// the computed value left behind for good, or an idle scroll-driven
/// element binding every frame.
#[test]
fn view_timelines_follow_the_subject_through_its_scroller() {
    let text = r#"<meta name="component" content="t">
<div id="Root"><div id="Box"><div id="Top"></div><div id="Card"></div><div class="gap"></div></div></div>
<style>
#Box { height: 100px; width: 300px; overflow: hidden auto; scrollbar-width: none; }
#Top { height: 50px; }
.gap { height: 300px; }
#Card { height: 50px; width: 100px; animation: slide linear both; animation-timeline: view(); }
@keyframes slide { from { translate: 0 0; width: 20px; } to { translate: 150px 0; width: 170px; } }
</style>"#;
    crate::support::with_doc(text, &[], |d| {
        let b = d.get("Box");
        let settle = |d: &mut crate::support::Doc| {
            for _ in 0..40 {
                d.step(InputFrame::hover(b.x + 10.0, b.y + 10.0));
            }
        };
        // Its cover range runs from offset -50 (its top at the port's
        // bottom) to 100: at rest it is a third of the way through.
        settle(d);
        let c = d.get("Card");
        assert!((c.x - b.x - 50.0).abs() < 0.01, "{c:?}");
        assert!((c.width - 70.0).abs() < 0.01, "{c:?}");
        d.step(InputFrame {
            wheel: og_core::glam::Vec2::new(0.0, 25.0),
            ..InputFrame::hover(b.x + 10.0, b.y + 10.0)
        });
        settle(d);
        let c = d.get("Card");
        assert!((c.x - b.x - 75.0).abs() < 0.01, "{c:?}");
        assert!((c.width - 95.0).abs() < 0.01, "{c:?}");
        // The pointer leaves; then nothing changes.
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        let idle = d.ui.frame().stats;
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    });
}

/// A declared prop bound two ways writes the mount's store of its name,
/// as `bind:checked={$on}` writes a store: a built-in screen reads and
/// writes its props bare (`bind:open={open}`, `bind:value={volume}`).
///
/// Ways it could fail: the bare name bound as a value read once (the box
/// flips and the store stays), a write refused because the name is no
/// store, or the prop's default written over the store.
#[test]
fn a_bare_prop_bound_two_ways_writes_the_mount_s_store() {
    let text = r#"<meta name="component" content="t">
<meta name="prop:on" content="bool" data-default="false">
<meta name="prop:volume" content="number" data-default="0">
<column id="Form">
  <input type="checkbox" id="Box" bind:checked={on} />
  <input type="range" id="Volume" min="0" max="10" bind:value={volume} />
</column>
<style>#Volume { width: 200px; }</style>"#;
    let mut d = Doc::new(text, &[("on", json!(false)), ("volume", json!(2))]);
    d.step(InputFrame::default());
    assert_eq!(d.read("on"), json!(false));
    d.click("Box");
    assert_eq!(d.read("on"), json!(true), "the box writes its prop");
    // The game's write shows, and the next click writes over it.
    d.set("on", json!(false));
    d.click("Box");
    assert_eq!(d.read("on"), json!(true));
    // A press on the track's far end moves the knob there.
    let track = d.get("Volume");
    d.click_at(og_core::glam::Vec2::new(
        track.x + track.width - 2.0,
        track.y + track.height / 2.0,
    ));
    assert_eq!(d.read("volume"), json!(10));
}
