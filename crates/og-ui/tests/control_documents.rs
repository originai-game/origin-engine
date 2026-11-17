//! Control documents (docs/ui.md §2.4): a control element takes its inside
//! from the library document that names it as its host.
//!
//! Ways this could fail:
//! - the host still draws its native widget, or draws nothing
//!   → `a_progress_bar_is_drawn_by_its_control_document`
//! - the document's props do not follow the host's bound attribute
//!   → the same test, after the store moves
//! - a rule for the host where it stands loses to the document's `:host`
//!   rule, or a `::part()` rule loses to the document's own rule for the
//!   part → `outer_rules_outrank_the_control_documents_own`
//! - the host loses its role and value → `the_host_keeps_its_semantics`
//! - a control held by a control document gets no inside of its own, or
//!   the holding document's rules do not reach it
//!   → `a_control_document_nests_another_control`
//! - a nested control's part reaches the page without `exportparts`, under
//!   its inner name, or loses to the holding document's own rule for it;
//!   a renamed part still answers to its inner name outside
//!   → `exportparts_makes_a_nested_part_the_outer_hosts`
//! - a host's document or the style does not reach the element a control
//!   document declares as a standard pseudo-element, loses to the
//!   document's own rule for it, or reaches it by a part name or another
//!   pseudo-element's → `a_standard_pseudo_element_reaches_the_control_documents_element`
//! - a state after the pseudo-element reads the host's state, or the
//!   element is not styled again when its own changes
//!   → `a_state_after_a_pseudo_element_is_its_elements_own`
//!
//! Instance state (docs/ui.md §2 Components in documents):
//! - two instances share one value, or `bind:` does not write it
//!   → `an_instance_holds_its_own_state`
//! - a snapshot loses it → `instance_state_is_carried_by_a_snapshot`
//! - a removed instance's value is found by the next one at its path
//!   → `a_removed_instance_takes_its_state_with_it`

use crate::support::*;
use og_core::glam::Vec2;
use og_ui::*;
use serde_json::json;

#[test]
fn fitted_tabs_keep_their_track_and_full_accessible_labels() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut theme = *ui.theme();
    theme.text_scale = 1.5;
    ui.set_theme(theme);
    let labels = ["Summary", "Detailed graphs", "Campaign scoreboard"];
    let mut d = Doc::mounted(
        ui,
        r#"<meta name="component" content="FittedTabs">
        <div id="Window"><tabs id="Tabs" items={$labels} fit={$fit} fill></tabs></div>
        <style>#Window { width: 324px; height: 180px; }</style>"#,
        &[("labels", json!(labels)), ("fit", json!(true))],
    );
    d.viewport = Viewport::new(390.0, 844.0);
    d.viewport.coarse_pointer = true;
    d.viewport.reduced_motion = true;
    for _ in 0..16 {
        d.step(InputFrame::default());
    }
    let strip = d.get("Strip");
    let track = d.get("Track");
    assert!(track.width <= strip.width + 0.5, "{track:?} in {strip:?}");
    assert!(
        d.ui.tree
            .nodes()
            .iter()
            .find(|n| n.id == "Track")
            .unwrap()
            .placed
            .screen
            .width
            <= strip.width + 0.5,
        "the track itself fits, not only its clipped probe bounds"
    );
    for (i, name) in labels.iter().enumerate() {
        let tab = d.get(&format!("Item#{i}"));
        assert!(
            tab.x >= strip.x - 0.5 && tab.x + tab.width <= strip.x + strip.width + 0.5,
            "every fitted tab is visible without scrolling: {tab:?} in {strip:?}"
        );
        assert_eq!(d.a11y(&format!("Item#{i}")).name, *name);
    }
    assert!(
        d.get("Item#2").width > d.get("Item#0").width,
        "longer labels receive more space when equal tabs cannot fit"
    );
    let label =
        d.ui.probe_elements()
            .into_iter()
            .find(|n| n.path.ends_with("Item#2/Label"))
            .unwrap();
    assert!(
        label.height > 30.0,
        "a long label may wrap before it is ellipsized: {label:?}"
    );
    d.set("fit", json!(false));
    for _ in 0..16 {
        d.step(InputFrame::default());
    }
    assert!(
        d.ui.tree
            .nodes()
            .iter()
            .find(|n| n.id == "Track")
            .unwrap()
            .placed
            .screen
            .width
            > d.get("Strip").width + 1.0,
        "without fit the existing scrollable, equal-width tabs remain"
    );
}

#[test]
fn icon_only_tabs_take_document_accessible_labels() {
    let mut d = Doc::new(
        r#"<meta name="component" content="IconTabs">
        <tabs id="Tabs" items={$items} icons="plus check" aria-labels={$names} />"#,
        &[
            ("items", json!(["", ""])),
            ("names", json!(["Add build", "Completed builds"])),
        ],
    );
    assert_eq!(d.a11y("Item#0").name, "Add build");
    assert_eq!(d.a11y("Item#1").name, "Completed builds");
    d.set("names", json!(["Create loadout", "Saved loadouts"]));
    assert_eq!(d.a11y("Item#0").name, "Create loadout");
}

#[test]
fn native_tabs_natural_width_settles_without_screen_feedback() {
    for (name, css, width) in [
        (
            "scifi",
            include_str!("../../../templates/styles/scifi/base.css"),
            456,
        ),
        (
            "cartoon",
            include_str!("../../../templates/styles/cartoon/base.css"),
            418,
        ),
    ] {
        let directory = format!("styles/{name}");
        let pack = og_ui::style_pack::StylePack::parse(
            &directory,
            vec![(format!("{directory}/base.css"), css.into())],
        )
        .unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        let mut theme = *ui.theme();
        theme.text_scale = 1.5;
        ui.set_theme(theme);
        ui.set_style_pack(&pack).unwrap();
        let mut d = Doc::mounted(
            ui,
            &format!(
                r#"
            <meta name="component" content="TabsRegression">
            <div id="Window"><tabs id="Tabs" items={{labels}} fill="true"></tabs></div>
            <style>#Window {{ width: {width}px; height: 100px; }}</style>
        "#
            ),
            &[("labels", json!(["Summary", "Graphs", "Scoreboard"]))],
        );
        d.viewport = Viewport::new(844.0, 390.0);
        d.viewport.reduced_motion = true;
        d.viewport.coarse_pointer = true;
        let mut widths = Vec::new();
        for tick in 0..132 {
            d.step(InputFrame::default());
            if tick >= 112 {
                widths.push(
                    d.ui.tree
                        .nodes()
                        .iter()
                        .find(|n| n.id == "Track")
                        .unwrap()
                        .placed
                        .screen
                        .width,
                );
            }
        }
        assert!(
            widths.iter().all(|w| (w - widths[0]).abs() < 0.01),
            "{name}: {widths:?}"
        );
        let long = widths[0];
        d.set("labels", json!(["A", "B", "C"]));
        for _ in 0..12 {
            d.step(InputFrame::default());
        }
        let short =
            d.ui.tree
                .nodes()
                .iter()
                .find(|n| n.id == "Track")
                .unwrap()
                .placed
                .screen
                .width;
        assert!(
            short < long,
            "{name}: natural width must shrink after labels change: {long} -> {short}"
        );
    }
}

/// A UI with the library's control document `name` defined.
fn with_control(name: &str) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    let path = engine_dir(&format!("src/ui/controls/{name}.svelte"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    let doc = ui
        .parse_document(&format!("@ui/controls/{name}.svelte"), &text, Vec::new())
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert!(doc.warnings.is_empty(), "{:?}", doc.warnings);
    ui.define_document(doc);
    ui
}

const BAR: &str = r#"<meta name="component" content="t">
<div id="Form"><progress id="P" value={$v}></progress></div>
<style>
#Form { width: 400px; padding: 0; }
</style>"#;

#[test]
fn a_progress_bar_is_drawn_by_its_control_document() {
    let mut d = Doc::mounted(with_control("progress"), BAR, &[("v", json!(0.25))]);
    d.step(InputFrame::default());
    let host = d.get("P");
    let track = d.get("P/Track");
    let fill = d.get("P/Track/Fill");
    assert_eq!(host.width, 400.0, "the host fills its line");
    assert_eq!((track.width, track.height), (host.width, host.height));
    assert_eq!(fill.width, 100.0, "a quarter of the track");
    d.set("v", json!(0.5));
    for _ in 0..60 {
        d.step(InputFrame::default());
    }
    assert_eq!(
        d.get("P/Track/Fill").width,
        200.0,
        "the fill follows the store"
    );
    d.set("v", json!(0));
    for _ in 0..60 {
        d.step(InputFrame::default());
    }
    assert!(d.el("P/Track/Fill").is_none(), "no fill at zero");
}

#[test]
fn outer_rules_outrank_the_control_documents_own() {
    let text = BAR.replace(
        "</style>",
        "progress { height: 24px; }\nprogress::part(fill) { border-radius: 0; background: #ff0000; }\n</style>",
    );
    let mut d = Doc::mounted(with_control("progress"), &text, &[("v", json!(0.5))]);
    d.step(InputFrame::default());
    assert_eq!(d.get("P").height, 24.0, "the host's own document sizes it");
    assert_eq!(d.get("P/Track/Fill").height, 24.0);
    let fill = d.get("P/Track/Fill");
    let red = Srgba::rgb(255, 0, 0).linear();
    let fills = fills_in(&d.ui, Rect::new(fill.x, fill.y, fill.width, fill.height));
    assert!(
        fills.contains(&red),
        "the document's ::part() rule paints the fill: {fills:?}"
    );
}

#[test]
fn the_host_keeps_its_semantics() {
    let mut d = Doc::mounted(with_control("progress"), BAR, &[("v", json!(0.25))]);
    d.step(InputFrame::default());
    let node = d.a11y("P");
    assert_eq!(node.role, "progressbar");
    assert_eq!(node.value, "25%");
}

#[test]
fn a_meter_document_paints_its_hosts_region() {
    // The region is derived once for the host: the document's `region`
    // prop and the host's fact for `meter[region=…]::part(fill)`.
    let text = r#"<meta name="component" content="t">
<div id="Form"><meter id="M" value={$v} min="0" max="100" low="25" high="75" optimum="90"></meter></div>
<style>
#Form { width: 400px; }
meter { width: 200px; }
meter[region=poor]::part(track) { height: 4px; }
</style>"#;
    let mut d = Doc::mounted(with_control("meter"), text, &[("v", json!(80))]);
    let palette = d.ui.theme().palette;
    let painted = |d: &mut Doc, value: f64| {
        d.set("v", json!(value));
        d.step(InputFrame::default());
        let fill = d.get("M/Track/Fill");
        let fills = fills_in(&d.ui, Rect::new(fill.x, fill.y, fill.width, fill.height));
        let width = fill.width;
        let region = [
            ("optimum", palette.success),
            ("suboptimum", palette.warn),
            ("poor", palette.danger),
        ]
        .into_iter()
        .filter(|(_, c)| fills.iter().any(|f| near(*f, *c)))
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
        (width, region, d.get("M/Track").height)
    };
    assert_eq!(painted(&mut d, 80.0), (160.0, vec!["optimum"], 12.0));
    assert_eq!(painted(&mut d, 50.0), (100.0, vec!["suboptimum"], 12.0));
    assert_eq!(
        painted(&mut d, 20.0),
        (40.0, vec!["poor"], 4.0),
        "the host's region is a fact its ::part() rules read"
    );
    assert_eq!(d.a11y("M").role, "meter");
    assert_eq!(d.a11y("M").value, "20");
}

#[test]
fn a_control_defined_after_its_host_is_mounted_takes_it_over() {
    // A screen's load defines the controls it uses after the screen.
    let mut d = Doc::new(BAR, &[("v", json!(0.5))]);
    d.step(InputFrame::default());
    assert!(d.el("P/Track").is_none(), "the fallback draws no document");
    let path = engine_dir("src/ui/controls/progress.svelte");
    let text = std::fs::read_to_string(path).unwrap();
    let doc =
        d.ui.parse_document("@ui/controls/progress.svelte", &text, Vec::new())
            .unwrap();
    d.ui.define_document(doc);
    d.step(InputFrame::default());
    assert_eq!(d.get("P/Track/Fill").width, 200.0);
}

#[test]
fn a_control_document_nests_another_control() {
    // A game's own meter, drawn with a progress bar: the bar inside takes
    // its inside from its control document too, and the meter document's
    // rules reach it as any document's rules reach a control it holds.
    let meter = r#"<meta name="control" content="meter">
<meta name="parts" content="frame">
<meta name="prop:value" content="number">
<meta name="prop:max" content="number" data-default="1">
<div id="Frame" part="frame"><progress id="Bar" value={value / max}></progress></div>
<style>
:host { display: block; inline-size: 200px; }
progress { block-size: 10px; }
progress::part(fill) { border-radius: 0; background: #00ff00; }
</style>"#;
    let mut ui = with_control("progress");
    let doc = ui
        .parse_document("ui/controls/meter.svelte", meter, Vec::new())
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert!(doc.warnings.is_empty(), "{:?}", doc.warnings);
    ui.define_document(doc);
    let text = r#"<meta name="component" content="t">
<div id="Form"><meter id="M" value={$v} max="100"></meter></div>
<style>
#Form { width: 400px; }
</style>"#;
    let mut d = Doc::mounted(ui, text, &[("v", json!(50))]);
    d.step(InputFrame::default());
    let bar = d.get("M/Frame/Bar");
    assert_eq!(
        (bar.width, bar.height),
        (200.0, 10.0),
        "the meter document sizes the bar it holds"
    );
    let fill = d.get("M/Frame/Bar/Track/Fill");
    assert_eq!(fill.width, 100.0, "the bar reads the meter's props");
    let green = Srgba::rgb(0, 255, 0).linear();
    let fills = fills_in(&d.ui, Rect::new(fill.x, fill.y, fill.width, fill.height));
    assert!(
        fills.contains(&green),
        "the meter document's ::part() rule paints the bar's fill: {fills:?}"
    );
    assert_eq!(d.a11y("M").role, "meter", "the outer host is the control");
    d.set("v", json!(25));
    for _ in 0..60 {
        d.step(InputFrame::default());
    }
    assert_eq!(d.get("M/Frame/Bar/Track/Fill").width, 50.0);
}

/// A component that keeps whether it is switched on for itself.
const LAMP: &str = r#"<meta name="component" content="Lamp">
<meta name="state:on" content="bool">
<meta name="prop:label" content="string">
<div id="Lamp">
  <input id="Switch" type="checkbox" bind:checked={on}>
  <p id="Lit" hidden={!on}>{label}</p>
</div>"#;

const LAMPS: &str = r#"<meta name="component" content="t">
<div id="Room">
  <Lamp id="A" label="a"/>
  <Lamp id="B" label="b"/>
</div>"#;

fn lamps() -> Doc {
    let mut ui = ui_with_font(test_font_metrics());
    let lamp = ui.parse_document("lamp.svelte", LAMP, Vec::new()).unwrap();
    ui.define_document(lamp);
    let mut d = Doc::mounted(ui, LAMPS, &[]);
    d.step(InputFrame::default());
    d
}

#[test]
fn an_instance_holds_its_own_state() {
    let mut d = lamps();
    assert!(d.el("A/Lit").is_none() && d.el("B/Lit").is_none());
    d.click("A/Switch");
    assert!(
        d.el("A/Lit").is_some(),
        "the press wrote the instance's state"
    );
    assert!(d.el("B/Lit").is_none(), "the other instance has its own");
    d.click("A/Switch");
    assert!(d.el("A/Lit").is_none());
}

#[test]
fn instance_state_is_carried_by_a_snapshot() {
    let mut d = lamps();
    d.click("B/Switch");
    let saved = d.ui.save_state();
    d.click("B/Switch");
    d.click("A/Switch");
    assert!(d.el("A/Lit").is_some() && d.el("B/Lit").is_none());
    d.ui.restore_state(&saved).unwrap();
    d.step(InputFrame::default());
    assert!(d.el("B/Lit").is_some(), "B is on again");
    assert!(d.el("A/Lit").is_none(), "A is off again");
}

#[test]
fn a_removed_instance_takes_its_state_with_it() {
    // A row's lamp is switched on, the row goes and comes back: the new
    // instance starts from the state's default, not the old one's value.
    let mut ui = ui_with_font(test_font_metrics());
    let lamp = ui.parse_document("lamp.svelte", LAMP, Vec::new()).unwrap();
    ui.define_document(lamp);
    let text = r#"<meta name="component" content="t">
<div id="Room">
  {#each $lamps as lamp (lamp.id)}
    <Lamp id="L" label={lamp.id}/>
  {/each}
</div>"#;
    let rows = json!([{"id": "x"}, {"id": "y"}]);
    let mut d = Doc::mounted(ui, text, &[("lamps", rows)]);
    d.step(InputFrame::default());
    let lit = |d: &Doc| {
        ["x", "y"]
            .into_iter()
            .filter(|row| d.el(&format!("L#{row}/Lit")).is_some())
            .collect::<Vec<_>>()
    };
    d.click("L#x/Switch");
    assert_eq!(lit(&d), ["x"], "each row's lamp has its own state");
    d.edit("lamps", json!({"op": "remove", "key": "x"}));
    d.edit("lamps", json!({"op": "insert", "row": {"id": "x"}}));
    assert!(d.el("L#x/Switch").is_some(), "the row is back");
    assert_eq!(lit(&d), [""; 0], "its lamp starts off");
}

#[test]
fn a_separator_is_drawn_by_the_default_styles_rule() {
    // `<hr>` has no inside and no look in Rust: the default style's `hr`
    // rule is its line, and a document's rule for it wins as on any box.
    let text = r#"<meta name="component" content="t">
<div id="Form">
  <hr id="Line">
  <hr id="Thick" class="thick">
  <div id="Row"><hr id="Post" vertical></div>
</div>
<style>
#Form { width: 400px; padding: 0; }
#Row { display: flex; height: 40px; }
.thick { block-size: 3px; color: #ff0000; }
</style>"#;
    let mut d = Doc::new(text, &[]);
    d.step(InputFrame::default());
    let painted = |d: &Doc, id: &str| {
        let e = d.get(id);
        let fills = fills_in(&d.ui, Rect::new(e.x, e.y, e.width, e.height));
        ((e.width, e.height), fills)
    };
    let line = d.ui.theme().palette.line;
    let (size, fills) = painted(&d, "Line");
    assert_eq!(size, (400.0, 1.0));
    assert!(fills.iter().any(|f| near(*f, line)), "{fills:?}");
    let (size, fills) = painted(&d, "Thick");
    assert_eq!(size, (400.0, 3.0));
    assert!(fills.contains(&Srgba::rgb(255, 0, 0).linear()), "{fills:?}");
    assert_eq!(painted(&d, "Post").0, (1.0, 40.0));
    assert_eq!(d.a11y("Line").role, "separator");
}

/// A game's meter drawn with a progress bar, which exports the bar's fill
/// under `exported`.
fn meter_exporting(exported: &str) -> Ui {
    meter_with(exported, "#00ff00")
}

/// The same, with the meter document's own fill `fill`.
fn meter_with(exported: &str, fill: &str) -> Ui {
    let meter = format!(
        r#"<meta name="control" content="meter">
<meta name="parts" content="frame">
<meta name="prop:value" content="number">
<meta name="prop:max" content="number" data-default="1">
<div id="Frame" part="frame"><progress id="Bar" value={{value / max}} {exported}></progress></div>
<style>
:host {{ display: block; inline-size: 200px; }}
progress {{ block-size: 10px; }}
progress::part(fill) {{ border-radius: 0; background: {fill}; }}
</style>"#
    );
    let mut ui = with_control("progress");
    let doc = ui
        .parse_document("ui/controls/meter.svelte", &meter, Vec::new())
        .unwrap_or_else(|e| panic!("{e:?}"));
    ui.define_document(doc);
    ui
}

/// The colours painted in the bar's fill under a page whose `<style>`
/// is `css`.
fn fill_colours(ui: Ui, css: &str) -> Vec<[f32; 4]> {
    let text = format!(
        r#"<meta name="component" content="t">
<div id="Form"><meter id="M" value={{$v}} max="100"></meter></div>
<style>
#Form {{ width: 400px; }}
{css}
</style>"#
    );
    let mut d = Doc::mounted(ui, &text, &[("v", json!(50))]);
    d.step(InputFrame::default());
    let fill = d.get("M/Frame/Bar/Track/Fill");
    fills_in(&d.ui, Rect::new(fill.x, fill.y, fill.width, fill.height))
}

#[test]
fn exportparts_makes_a_nested_part_the_outer_hosts() {
    let (red, blue, green) = (
        Srgba::rgb(255, 0, 0).linear(),
        Srgba::rgb(0, 0, 255).linear(),
        Srgba::rgb(0, 255, 0).linear(),
    );
    let page =
        "meter::part(bar-fill) { background: #ff0000; } meter::part(fill) { background: #0000ff; }";
    // Not exported: the page reaches the meter's own parts only, and the
    // meter document's rule paints the bar it holds.
    let fills = fill_colours(meter_exporting(""), page);
    assert!(
        fills.contains(&green) && !fills.contains(&red) && !fills.contains(&blue),
        "{fills:?}"
    );
    // Exported under another name: the page reaches it by that name,
    // over the meter document's own rule, and not by the inner name.
    let fills = fill_colours(meter_exporting(r#"exportparts="fill: bar-fill""#), page);
    assert!(fills.contains(&red) && !fills.contains(&blue), "{fills:?}");
    // Exported as it is named, beside another part under another name,
    // which is that part's alone.
    let fills = fill_colours(
        meter_exporting(r#"exportparts="fill, track: bar-track""#),
        "meter::part(fill) { background: #0000ff; } meter::part(bar-track) { background: #ff0000; }",
    );
    assert!(fills.contains(&blue) && !fills.contains(&red), "{fills:?}");
    // `!important` reverses the rank (CSS Cascade 5 §6.1): the meter
    // document's important rule wins over the page's.
    let ui = meter_with(r#"exportparts="fill""#, "#00ff00 !important");
    let fills = fill_colours(ui, "meter::part(fill) { background: #0000ff !important; }");
    assert!(
        fills.contains(&green) && !fills.contains(&blue),
        "{fills:?}"
    );
    // The page's rule is one boundary further out than the meter
    // document's: it wins whatever the specificity of either.
    let fills = fill_colours(
        meter_exporting(r#"exportparts="fill""#),
        ":where(meter)::part(fill) { background: #0000ff; }",
    );
    assert!(fills.contains(&blue), "{fills:?}");
}

#[test]
fn exportparts_is_read_where_a_part_can_be_exported() {
    let mut ui = with_control("progress");
    let refused = |ui: &mut Ui, text: &str| {
        let errors = ui
            .parse_document("ui/t.svelte", text, Vec::new())
            .expect_err(text);
        errors.join("\n")
    };
    let screen =
        r#"<meta name="component" content="t"><div><progress exportparts="fill"></progress></div>"#;
    assert!(refused(&mut ui, screen).contains("control document"));
    let instance = r#"<meta name="control" content="meter"><div><Bar exportparts="fill"/></div>"#;
    assert!(refused(&mut ui, instance).contains("component instance"));
    let list = r#"<meta name="control" content="meter"><div><progress exportparts="fill: :x"></progress></div>"#;
    assert!(refused(&mut ui, list).contains("part names"));
}

/// A UI with a game's control document `text` for `host` defined.
fn with_document(host: &str, text: &str) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    let doc = ui
        .parse_document(&format!("ui/controls/{host}.svelte"), text, Vec::new())
        .unwrap_or_else(|e| panic!("{e:?}"));
    ui.define_document(doc);
    ui
}

/// A select whose control document gives its picker icon an element of
/// its own (`pseudo="picker-icon"`), painted green by its own rule.
fn select_with_icon() -> Ui {
    with_document(
        "select",
        r#"<meta name="control" content="select">
<div id="Box"><div id="Icon" pseudo="picker-icon"></div></div>
<style>
:host { display: block; inline-size: 200px; }
[pseudo="picker-icon"] { inline-size: 40px; block-size: 40px; background: #00ff00; }
</style>"#,
    )
}

/// The colours painted in `target` of a page holding `host` under the
/// page's `<style>` `css`, with the pointer at `pointer` (none: outside
/// the window).
fn reached_colours(
    ui: Ui,
    host: &str,
    target: &str,
    css: &str,
    pointer: Option<Vec2>,
) -> Vec<[f32; 4]> {
    let text = format!(
        r#"<meta name="component" content="t">
<div id="Form">{host}</div>
<style>
#Form {{ width: 400px; }}
{css}
</style>"#
    );
    let mut d = Doc::mounted(ui, &text, &[]);
    let input = InputFrame {
        pointer: pointer.unwrap_or_default(),
        pointer_inside: pointer.is_some(),
        ..InputFrame::default()
    };
    for _ in 0..3 {
        d.step(input.clone());
    }
    let at = d.get(target);
    fills_in(&d.ui, Rect::new(at.x, at.y, at.width, at.height))
}

fn icon_colours(ui: Ui, css: &str) -> Vec<[f32; 4]> {
    let select = r#"<select id="S" class="pick"></select>"#;
    reached_colours(ui, select, "S/Box/Icon", css, None)
}

#[test]
fn a_standard_pseudo_element_reaches_the_control_documents_element() {
    let (red, blue, green) = (
        Srgba::rgb(255, 0, 0).linear(),
        Srgba::rgb(0, 0, 255).linear(),
        Srgba::rgb(0, 255, 0).linear(),
    );
    // The page's rule for the host's pseudo-element wins over the control
    // document's own rule for its element, whatever either's specificity.
    let fills = icon_colours(
        select_with_icon(),
        ":where(select)::picker-icon { background: #ff0000; }",
    );
    assert!(fills.contains(&red) && !fills.contains(&green), "{fills:?}");
    // Its name is apart from parts', and a rule for another host's
    // pseudo-element, or for this one's under another host test, misses.
    let fills = icon_colours(
        select_with_icon(),
        "select::part(picker-icon) { background: #ff0000; }
         .other::picker-icon { background: #0000ff; }
         select::checkmark { background: #0000ff; }",
    );
    assert!(fills.contains(&green), "{fills:?}");
    // The style's rule reaches it too, under the page's.
    let mut ui = select_with_icon();
    ui.set_part_css("select::picker-icon { background: #0000ff; }")
        .unwrap();
    assert!(icon_colours(ui, "").contains(&blue));
    let mut ui = select_with_icon();
    ui.set_part_css("select::picker-icon { background: #0000ff; }")
        .unwrap();
    let fills = icon_colours(ui, "select::picker-icon { background: #ff0000; }");
    assert!(fills.contains(&red) && !fills.contains(&blue), "{fills:?}");
}

#[test]
fn a_state_after_a_pseudo_element_is_its_elements_own() {
    // A file input whose button is its document's: the pointer on the
    // input beside the button leaves it unhovered, on the button hovers it
    // (CSS Pseudo-Elements 4: a pseudo-class after an element-backed
    // pseudo-element reads that element's state).
    let file = r#"<meta name="control" content="input[type=file]">
<div id="Box"><button id="Pick" pseudo="file-selector-button"></button></div>
<style>
:host { display: block; inline-size: 300px; block-size: 40px; }
#Box { display: flex; }
[pseudo="file-selector-button"] { inline-size: 40px; block-size: 40px; }
</style>"#;
    let page = r#"<meta name="component" content="t">
<div id="Form"><input id="F" type="file"></div>
<style>
#Form { width: 400px; }
input::file-selector-button:hover { inline-size: 120px; }
</style>"#;
    let mut d = Doc::mounted(with_document("file", file), page, &[]);
    let mut width = |x: f32| {
        let input = InputFrame {
            pointer: Vec2::new(x, 20.0),
            pointer_inside: true,
            ..InputFrame::default()
        };
        for _ in 0..3 {
            d.step(input.clone());
        }
        d.get("F/Box/Pick").width
    };
    assert_eq!(width(250.0), 40.0, "the pointer on the input beside it");
    assert_eq!(width(20.0), 120.0, "the pointer on the button");
    assert_eq!(width(250.0), 40.0, "the pointer off the button again");
}

#[test]
fn pseudo_is_read_where_an_element_can_be_its_hosts_pseudo_element() {
    let mut ui = ui_with_font(test_font_metrics());
    let refused = |ui: &mut Ui, text: &str| {
        let errors = ui
            .parse_document("ui/t.svelte", text, Vec::new())
            .expect_err(text);
        errors.join("\n")
    };
    let screen = r#"<meta name="component" content="t"><div><div pseudo="backdrop"></div></div>"#;
    assert!(refused(&mut ui, screen).contains("control document"));
    let host = r#"<meta name="control" content="select"><div><div pseudo="backdrop"></div></div>"#;
    assert!(refused(&mut ui, host).contains("a <select> has no ::backdrop"));
    let twice = r#"<meta name="control" content="details"><div><div pseudo="details-content"></div><div pseudo="details-content"></div></div>"#;
    assert!(refused(&mut ui, twice).contains("written twice"));
    let marker = r#"<meta name="control" content="select"><div><div pseudo="marker"></div></div>"#;
    assert!(refused(&mut ui, marker).contains("standard pseudo-element name"));
    let bound = r#"<meta name="control" content="select"><div><div pseudo={x}></div></div>"#;
    assert!(refused(&mut ui, bound).contains("not a binding"));
    let instance =
        r#"<meta name="control" content="select"><div><Icon pseudo="picker-icon"/></div>"#;
    assert!(refused(&mut ui, instance).contains("component instance"));
    let own = r#"<meta name="control" content="select"><div></div><style>:host::picker-icon { color: red; }</style>"#;
    assert!(refused(&mut ui, own).contains("pseudo=\"picker-icon\""));
    let dialog = r#"<meta name="control" content="dialog"><div></div><style>:host::backdrop { color: red; }</style>"#;
    assert!(refused(&mut ui, dialog).contains("pseudo=\"backdrop\""));
    // A popover's backdrop and a file input's button are theirs.
    for host in ["div[popover]", "input[type=file]"] {
        let name = if host.starts_with("div") {
            "backdrop"
        } else {
            "file-selector-button"
        };
        let text = format!(
            r#"<meta name="control" content="{host}"><div><div pseudo="{name}"></div></div>"#
        );
        ui.parse_document("ui/t.svelte", &text, Vec::new())
            .unwrap_or_else(|e| panic!("{host}: {e:?}"));
    }
}
