//! The visual UI debugger's reads (`Ui::debug_pick`, `Ui::debug_element`,
//! docs/ui.md §6.4) and what they read. Ways it
//! could fail, each covered below: a pick lands on the element under
//! another (the first drawn, not the last) or on one outside its clip; the
//! box model leaves out the border or the padding, or swaps CSS's order;
//! a computed style names no line or the rule that lost; a binding shows
//! its value from before the change; or a layout boundary goes unreported.

use crate::support;

use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use support::*;

#[path = "ui_debug/doc.rs"]
mod doc;
use doc::*;

fn mounted() -> (Ui, impl Fn(&mut Ui, Value)) {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/dbg.svelte", DOC).unwrap());
    let apply = |ui: &mut Ui, command: Value| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let title = SourceId::new("test", "title");
    apply(
        &mut ui,
        json!({"op": "signal", "source": title, "initial": "Hello"}),
    );
    let mut inputs = std::collections::BTreeMap::new();
    inputs.insert("title".to_string(), title);
    ui.mount_document(
        SourceId::new("test", "dbg"),
        "dbg",
        inputs,
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for _ in 0..2 {
        step(&mut ui);
    }
    (ui, apply)
}

fn step(ui: &mut Ui) {
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
}

#[test]
fn the_debugger_picks_the_topmost_element_and_reads_its_box_styles_and_bindings() {
    let (mut ui, apply) = mounted();
    let card = ui
        .element_bounds("test:dbg/Card")
        .expect("the card is drawn");
    let title = ui.element_bounds("test:dbg/Card/Title").expect("the title");
    // Over the title: the title, drawn over the card; in the card's
    // padding: the card; past everything: nothing.
    let mid = |r: Rect| (r.x + r.width / 2.0, r.y + r.height / 2.0);
    let (x, y) = mid(title);
    assert_eq!(ui.debug_pick(x, y).as_deref(), Some("test:dbg/Card/Title"));
    assert_eq!(
        ui.debug_pick(card.x + 3.0, card.y + card.height - 3.0)
            .as_deref(),
        Some("test:dbg/Card")
    );
    assert_eq!(ui.debug_pick(790.0, 590.0), None);

    let e = ui
        .debug_element("test:dbg/Card")
        .expect("the card's record");
    assert_eq!(e.border_box, card);
    // CSS's order: top right bottom left; the later rule's padding wins.
    let ins = |i: Insets| [i.top, i.right, i.bottom, i.left];
    assert_eq!(ins(e.padding), [8.0, 12.0, 8.0, 16.0]);
    assert_eq!(ins(e.border), [2.0, 2.0, 2.0, 2.0]);
    assert_eq!(ins(e.margin), [10.0, 4.0, 10.0, 4.0]);
    assert_eq!(
        [e.content.x, e.content.y, e.content.width, e.content.height],
        [
            card.x + 18.0,
            card.y + 10.0,
            card.width - 2.0 - 2.0 - 16.0 - 12.0,
            card.height - 2.0 - 2.0 - 16.0
        ]
    );
    assert!(e.boundaries.contains(&"fixed"), "{:?}", e.boundaries);
    let style = |name: &str| e.styles.iter().find(|s| s.name == name).cloned();
    let width = style("width").expect("width is a computed style");
    assert_eq!(
        (width.value.as_str(), width.source.as_str()),
        ("220px", "ui/dbg.svelte:9")
    );
    let padding = style("padding").expect("padding is a computed style");
    assert_eq!(
        (padding.value.as_str(), padding.source.as_str()),
        ("8px 12px 8px 16px", "ui/dbg.svelte:10"),
        "the winning rule's line, not the one it overrode"
    );
    let list = ui.debug_element("test:dbg/List").unwrap();
    assert!(list.boundaries.contains(&"scroll"), "{:?}", list.boundaries);

    let bound = |ui: &Ui| {
        ui.debug_element("test:dbg/Card/Title")
            .unwrap()
            .bindings
            .into_iter()
            .find(|(p, ..)| p == "text")
    };
    assert_eq!(
        bound(&ui),
        Some(("text".into(), "{$title}".into(), "Hello".into()))
    );
    apply(
        &mut ui,
        json!({"op": "set", "source": SourceId::new("test", "title"), "value": "Bye"}),
    );
    step(&mut ui);
    assert_eq!(bound(&ui).map(|b| b.2), Some("Bye".into()), "the value now");
    assert!(ui.debug_element("test:dbg/Nothing").is_none());
}
