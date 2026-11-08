//! CSS interaction properties (docs/ui.md §2): `pointer-events`,
//! `visibility` and the cursor keywords.
//!
//! Ways these could fail, each covered below:
//! - `pointer-events: none` taking the pointer anyway, or not handing it
//!   to what is under it, or a child saying `auto` inside it not taking
//!   it → `the_pointer_passes_through_none_and_hidden`
//! - `visibility: hidden` drawing or taking the pointer, or not letting a
//!   child that says `visible` take it → the same test

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use serde_json::json;
use support::*;

const DOC: &str = r#"<meta name="component" content="ix">
<div id="Root">
  <button id="Under">Under</button>
  <div id="Over" onpointerdown={over}><button id="Again">Again</button></div>
  <div id="Ghost"><button id="Lost">Lost</button><button id="Shown">Shown</button></div>
</div>
<style>
#Root { width: 600px; height: 400px; cursor: help; }
#Under { width: 200px; height: 60px; cursor: crosshair; }
#Over { position: absolute; top: 0; inset-inline-start: 0; width: 300px; height: 60px; pointer-events: none; cursor: wait; }
#Again { position: absolute; top: 0; inset-inline-start: 220px; width: 60px; height: 60px; pointer-events: auto; cursor: copy; }
#Ghost { visibility: hidden; cursor: no-drop; }
#Lost { width: 100px; height: 40px; }
#Shown { width: 100px; height: 40px; visibility: visible; cursor: nwse-resize; }
</style>"#;

fn frame(ui: &mut Ui, p: Vec2) -> Cursor {
    ui.begin(
        InputFrame {
            pointer: p,
            pointer_inside: true,
            ..InputFrame::default()
        },
        Viewport::new(800.0, 600.0),
    );
    ui.component("T", "ix", &mut json!({}), &bind::NoEnv);
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    ui.frame().cursor
}

#[test]
fn the_pointer_passes_through_none_and_hidden() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/ix.svelte", DOC).expect("parses"));
    frame(&mut ui, Vec2::ZERO);
    let el = |ui: &Ui, id: &str| {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap();
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    };
    let (under, again, lost, shown) = (
        el(&ui, "Under"),
        el(&ui, "Again"),
        el(&ui, "Lost"),
        el(&ui, "Shown"),
    );
    // Over the button the overlay covers: the overlay passes it on.
    frame(&mut ui, under);
    assert_eq!(frame(&mut ui, under), Cursor::Crosshair);
    // Its child that says auto takes the pointer again.
    frame(&mut ui, again);
    assert_eq!(frame(&mut ui, again), Cursor::Copy);
    // A hidden button takes nothing: the pointer is on the root.
    frame(&mut ui, lost);
    assert_eq!(frame(&mut ui, lost), Cursor::Default);
    // A visible child of a hidden parent takes it.
    frame(&mut ui, shown);
    assert_eq!(frame(&mut ui, shown), Cursor::ResizeNwse);
    let a11y = format!("{:?}", ui.a11y());
    assert!(a11y.contains("Shown") && !a11y.contains("Lost"), "{a11y}");
}
