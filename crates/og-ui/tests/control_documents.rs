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

use crate::support::*;
use og_ui::*;
use serde_json::json;

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
