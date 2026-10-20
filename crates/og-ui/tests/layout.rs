//! Layout of a nested row/column tree with grow, caps, shrink, wrap, percent,
//! alignment and the safe area, against `tests/golden/layout.json`
//! (`OG_UI_GOLDEN=1` re-records it). Text is measured at a fixed 0.5 em per
//! character (`text-body`: 7 px a character, 20 px a line) so the golden
//! does not depend on a font. The rectangles the reasoning below predicts
//! are asserted too, so the golden is checked, not just recorded.

mod support;

use og_ui::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use support::out_dir;

fn image(ui: &mut Ui, id: impl Into<Id>, width: Size, height: Size) {
    ui.image(
        id,
        ImageDesc {
            width,
            height,
            ..ImageDesc::default()
        },
    );
}

fn label(ui: &mut Ui, id: &str, text: &str) {
    ui.label(id, text, LabelDesc::default());
}

fn tree(ui: &mut Ui) {
    let viewport = Viewport {
        width: 400.0,
        height: 300.0,
        dpr: 1.0,
        safe_area: Insets {
            left: 10.0,
            top: 20.0,
            right: 0.0,
            bottom: 0.0,
        },
        coarse_pointer: false,
        ..Viewport::default()
    };
    ui.begin(InputFrame::default(), viewport);
    // A bar: fixed icon, text, a growing spacer and a growing badge capped at 50.
    ui.row(
        "Bar",
        Element {
            width: Size::grow(),
            height: Size::Fixed(40.0),
            padding: Insets::all(4.0),
            gap: 8.0,
            cross: Cross::Center,
            ..Element::default()
        },
        |ui| {
            image(ui, "Icon", Size::Fixed(32.0), Size::Fixed(32.0));
            label(ui, "Title", "Hello");
            ui.column(
                "Spacer",
                Element {
                    width: Size::grow(),
                    ..Element::default()
                },
                |_| {},
            );
            image(
                ui,
                "Badge",
                Size::Grow {
                    min: 0.0,
                    max: 50.0,
                },
                Size::Fixed(10.0),
            );
        },
    );
    // Six 50-wide tiles wrap three to a line in 190 inner pixels; a growing
    // seventh joins the second line and takes what is left of it.
    ui.row(
        "Wrap",
        Element {
            width: Size::Fixed(200.0),
            padding: Insets::all(5.0),
            gap: 10.0,
            wrap: true,
            ..Element::default()
        },
        |ui| {
            for n in 0..6 {
                image(ui, ui_idi("Tile", n), Size::Fixed(50.0), Size::Fixed(20.0));
            }
            image(ui, "Fill", Size::grow(), Size::Fixed(20.0));
        },
    );
    // 98 + 35 + 40 = 173 in 150: the widest (a wrapping label) gives up 23 and wraps.
    ui.row(
        "Shrink",
        Element {
            width: Size::Fixed(150.0),
            ..Element::default()
        },
        |ui| {
            label(ui, "Long", "aaaa bbbb cccc");
            label(ui, "Short", "dd ee");
            image(ui, "Box", Size::Fixed(40.0), Size::Fixed(10.0));
        },
    );
    // Centred both ways; a quarter-width child next to a growing one.
    ui.row(
        "Split",
        Element {
            width: Size::Fixed(200.0),
            ..Element::default()
        },
        |ui| {
            ui.column(
                "Center",
                Element {
                    width: Size::Fixed(100.0),
                    height: Size::Fixed(60.0),
                    align: Align::Center,
                    cross: Cross::Center,
                    ..Element::default()
                },
                |ui| label(ui, "Mid", "Hi"),
            );
            image(ui, "Quarter", Size::percent(0.25), Size::Fixed(10.0));
            image(ui, "Rest", Size::grow(), Size::grow());
        },
    );
    // Free space between three boxes; a stretched column child.
    ui.row(
        "Between",
        Element {
            width: Size::Fixed(200.0),
            align: Align::SpaceBetween,
            ..Element::default()
        },
        |ui| {
            for n in 0..3 {
                image(ui, ui_idi("Dot", n), Size::Fixed(20.0), Size::Fixed(10.0));
            }
        },
    );
    ui.column(
        "Stack",
        Element {
            width: Size::Fixed(120.0),
            padding: Insets::xy(10.0, 0.0),
            cross: Cross::Stretch,
            ..Element::default()
        },
        |ui| image(ui, "Wide", Size::fit(), Size::Fixed(5.0)),
    );
    ui.end();
}

fn rect(ui: &Ui, path: &str) -> [f32; 4] {
    let e = ui
        .frame()
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    [e.x, e.y, e.width, e.height]
}

#[test]
fn nested_rows_and_columns_lay_out_as_the_golden() {
    let mut ui = Ui::new(UiDesc::default());
    tree(&mut ui);
    let frame = ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);

    // The safe area pads the root.
    assert_eq!(rect(&ui, "Bar"), [10.0, 20.0, 390.0, 40.0]);
    // 382 inner − 32 − 35 − 3 gaps of 8 = 291 for spacer and badge: they rise
    // together until the badge stops at its max.
    assert_eq!(rect(&ui, "Badge")[2], 50.0);
    assert_eq!(rect(&ui, "Spacer")[2], 241.0);
    assert_eq!(rect(&ui, "Title"), [14.0 + 32.0 + 8.0, 30.0, 35.0, 20.0]);
    // Two lines of three; the grower fills the second line's last 10 px.
    assert_eq!(rect(&ui, "Wrap"), [10.0, 60.0, 200.0, 60.0]);
    assert_eq!(rect(&ui, "Wrap/Tile#3"), [15.0, 95.0, 50.0, 20.0]);
    assert_eq!(rect(&ui, "Fill"), [15.0 + 170.0 + 10.0, 95.0, 10.0, 20.0]);
    // The long label shrinks to 75 and wraps to two lines.
    assert_eq!(rect(&ui, "Long"), [10.0, 120.0, 75.0, 40.0]);
    assert_eq!(rect(&ui, "Short")[2], 35.0);
    // Centred in 100×60; quarter of 200; the rest grows in both axes.
    assert_eq!(rect(&ui, "Mid"), [10.0 + 43.0, 160.0 + 20.0, 14.0, 20.0]);
    assert_eq!(rect(&ui, "Quarter"), [110.0, 160.0, 50.0, 10.0]);
    assert_eq!(rect(&ui, "Rest"), [160.0, 160.0, 50.0, 60.0]);
    // 140 free between three: 0, 90, 180.
    let xs: Vec<f32> = (0..3)
        .map(|n| rect(&ui, &format!("Dot#{n}"))[0] - 10.0)
        .collect();
    assert_eq!(xs, [0.0, 90.0, 180.0]);
    assert_eq!(rect(&ui, "Wide"), [20.0, 230.0, 100.0, 5.0]);

    let actual: Value = json!({
        "viewport": {"width": 400, "height": 300, "safeArea": {"left": 10, "top": 20}},
        "elements": frame.elements.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
    });
    let text = serde_json::to_string_pretty(&actual).expect("json") + "\n";
    std::fs::write(out_dir().join("layout.json"), &text).expect("write actual");
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/layout.json");
    if std::env::var_os("OG_UI_GOLDEN").is_some() {
        std::fs::write(&golden, &text).expect("record golden");
    }
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(&golden)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_GOLDEN=1 records it)", golden.display())),
    )
    .expect("golden parses");
    assert_eq!(
        actual,
        expected,
        "layout differs from {}; the actual is in {}",
        golden.display(),
        out_dir().join("layout.json").display()
    );
}
