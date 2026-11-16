//! Layout of a nested row/column tree with grow, caps, shrink, wrap, percent,
//! alignment and the safe area, against `tests/golden/layout.json`
//! (`OG_UI_GOLDEN=1` re-records it). Text is measured at a fixed 0.5 em per
//! character (`text-body`: 7 px a character, 20 px a line) so the golden
//! does not depend on a font. The rectangles the reasoning below predicts
//! are asserted too, so the golden is checked, not just recorded.

use crate::support::{self, mount_stores, set_store};

use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use support::out_dir;

/// A UI measuring text at the fixed 0.5 em a character, with `text`
/// defined and mounted (as `test:t`) with the test stores `stores`.
fn mounted(text: &str, stores: &[(&str, Value)]) -> Ui {
    let mut ui = Ui::new(UiDesc::default());
    let doc = UiDocument::parse("ui/layout.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
    let name = doc.name.clone();
    ui.define_document(doc);
    mount_stores(&mut ui, SourceId::new("test", "t"), &name, stores);
    ui
}

/// One frame of the mounted documents at `viewport`.
fn frame(ui: &mut Ui, viewport: Viewport) {
    ui.begin(InputFrame::default(), viewport);
    ui.build_retained_documents(&bind::NoEnv).expect("builds");
    ui.end();
}

/// A list of `count` rows keyed `0`, `1`, …: each row's path is
/// `Item#<n>`, as `ui_idi` named a repeated element.
fn rows(count: u32) -> Value {
    (0..count).map(|n| json!({ "id": n.to_string() })).collect()
}

const TREE: &str = r#"<meta name="component" content="layout">
<column id="Tree">
  <!-- A bar: fixed icon, text, a growing spacer and a growing badge capped at 50. -->
  <row id="Bar">
    <img id="Icon" />
    <text id="Title">Hello</text>
    <column id="Spacer"></column>
    <img id="Badge" />
  </row>
  <!-- Six 50-wide tiles wrap three to a line in 190 inner pixels; a growing
       seventh joins the second line and takes what is left of it. -->
  <row id="Wrap">
    {#each $tiles as tile (tile.id)}<img id="Tile" />{/each}
    <img id="Fill" />
  </row>
  <!-- 98 + 35 + 40 = 173 in 150: the labels give up 23 by their bases. -->
  <row id="Shrink">
    <text id="Long">aaaa bbbb cccc</text>
    <text id="Short">dd ee</text>
    <img id="Box" />
  </row>
  <!-- Centred both ways; a quarter-width child next to a growing one. -->
  <row id="Split">
    <column id="Center"><text id="Mid">Hi</text></column>
    <img id="Quarter" />
    <img id="Rest" />
  </row>
  <!-- Free space between three boxes; a stretched column child. -->
  <row id="Between">
    {#each $dots as dot (dot.id)}<img id="Dot" />{/each}
  </row>
  <column id="Stack"><img id="Wide" /></column>
</column>
<style>
#Tree { width: stretch; height: stretch; align-items: start; }
#Bar { width: stretch; height: 40px; padding: 4px; gap: 8px; align-items: center; }
#Icon { width: 32px; height: 32px; }
#Spacer { flex: 1; }
#Badge { flex: 1; max-width: 50px; height: 10px; }
#Wrap { width: 200px; padding: 5px; gap: 10px; flex-wrap: wrap; }
#Tile { width: 50px; height: 20px; }
#Fill { flex: 1; height: 20px; }
#Shrink { width: 150px; }
#Box { width: 40px; height: 10px; }
#Split { width: 200px; }
#Center { width: 100px; height: 60px; justify-content: center; align-items: center; }
#Quarter { width: 25%; height: 10px; }
#Rest { flex: 1; height: stretch; }
#Between { width: 200px; justify-content: space-between; }
#Dot { width: 20px; height: 10px; }
#Stack { width: 120px; padding: 0 10px; align-items: stretch; }
#Wide { height: 5px; }
</style>
"#;

fn tree() -> Ui {
    let mut ui = mounted(TREE, &[("tiles", rows(6)), ("dots", rows(3))]);
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
    frame(&mut ui, viewport);
    ui
}

fn rect(ui: &Ui, path: &str) -> [f32; 4] {
    let e = ui.element(path).unwrap_or_else(|| {
        let all: Vec<&str> = ui.elements().iter().map(|e| e.path.as_str()).collect();
        panic!("no element {path}: {all:?}")
    });
    [e.x, e.y, e.width, e.height]
}

/// Twelve nested columns around `Box` (140 wide; 100 high when `fixed`,
/// else fitting its text `$text`) with a 30 × 10 `After` beside it.
fn nested(fixed: bool) -> String {
    let open: String = (0..12)
        .map(|i| format!(r#"<column id="Ancestor{i}">"#))
        .collect();
    let height = if fixed { "height: 100px;" } else { "" };
    format!(
        r#"<meta name="component" content="nested">
{open}<column id="Box"><text id="Text">{{$text}}</text></column><img id="After" />{close}
<style>
#Box {{ width: 140px; {height} }}
#After {{ width: 30px; height: 10px; }}
</style>
"#,
        close = "</column>".repeat(12)
    )
}

#[test]
fn fixed_boxes_bound_layout_invalidation_but_fitting_boxes_propagate_it() {
    let vp = Viewport::new(800.0, 600.0);
    for fixed in [true, false] {
        let mut cached = mounted(&nested(fixed), &[("text", json!("short"))]);
        frame(&mut cached, vp);
        let before = rect(&cached, "After");
        let text = "one two three four five six seven eight nine ten eleven twelve";
        set_store(&mut cached, "text", json!(text));
        frame(&mut cached, vp);
        let mut fresh = mounted(&nested(fixed), &[("text", json!(text))]);
        frame(&mut fresh, vp);
        let geometry = |ui: &Ui| {
            ui.elements()
                .iter()
                .map(|e| (e.path.clone(), [e.x, e.y, e.width, e.height]))
                .collect::<Vec<_>>()
        };
        assert_eq!(geometry(&cached), geometry(&fresh));
        assert!(rect(&cached, "Text")[3] > 20.0);
        if fixed {
            assert_eq!(rect(&cached, "After"), before);
            assert!(
                cached.frame().stats.laid_out <= 2,
                "{:?}",
                cached.frame().stats
            );
        } else {
            assert!(rect(&cached, "After")[1] > before[1]);
            assert!(
                cached.frame().stats.laid_out > 12,
                "{:?}",
                cached.frame().stats
            );
        }
    }
}

#[test]
fn fixed_box_size_and_placement_changes_still_invalidate_the_parent() {
    let px = |v: f32| json!(format!("{v}px"));
    let mut ui = mounted(
        r#"<meta name="component" content="placed">
<div id="Root"><div id="Child" style:left={$x} style:width={$w} style:height={$h}></div></div>
<style>#Child { position: absolute; top: 15px; }</style>
"#,
        &[("x", px(20.0)), ("w", px(30.0)), ("h", px(10.0))],
    );
    for (x, width, height) in [(20.0, 30.0, 10.0), (80.0, 30.0, 10.0), (80.0, 70.0, 40.0)] {
        set_store(&mut ui, "x", px(x));
        set_store(&mut ui, "w", px(width));
        set_store(&mut ui, "h", px(height));
        frame(&mut ui, Viewport::new(400.0, 300.0));
        assert_eq!(rect(&ui, "Child"), [x, 15.0, width, height]);
    }
}

#[test]
fn nested_rows_and_columns_lay_out_as_the_golden() {
    let ui = tree();
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
    // Both labels give up the 23 px by their shrink factor times their
    // basis (CSS Flexbox 1 §9.7), 98:35, and wrap to two lines.
    assert_eq!(rect(&ui, "Long"), [10.0, 120.0, 81.0, 40.0]);
    assert_eq!(rect(&ui, "Short")[2], 29.0);
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
        "elements": ui.elements().iter().map(|e| e.to_json()).collect::<Vec<_>>(),
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

/// A screen document's full-bleed art (`position: fixed`) stands on the
/// whole surface, under the safe area its layer keeps clear; an absolute
/// sibling stays in its parent's box; a fixed one sits by its end and
/// bottom insets on the surface's corner.
#[test]
fn a_screen_placed_element_bleeds_under_the_safe_area() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(
        UiDocument::parse(
            "ui/loading.svelte",
            r#"<meta name="screen" content="loading">
<column id="Loading"><div id="Art"></div><div id="Scrim"></div><div id="Corner"></div></column>
<style>
#Loading { width: stretch; height: stretch; }
#Art { position: fixed; inset: 0; }
#Scrim { position: absolute; inset: 0; }
#Corner { position: fixed; right: 0; bottom: 0; width: 50px; height: 20px; }
</style>
"#,
        )
        .unwrap(),
    );
    support::mount_screen(&mut ui, "loading", &[]);
    let viewport = Viewport {
        width: 390.0,
        height: 844.0,
        dpr: 1.0,
        safe_area: Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        },
        ..Viewport::default()
    };
    frame(&mut ui, viewport);
    let r = |p: &str| {
        let e = ui.element(p).expect(p);
        (e.x, e.y, e.width, e.height)
    };
    assert_eq!(r("Loading/Art"), (0.0, 0.0, 390.0, 844.0));
    assert_eq!(r("Loading/Scrim"), (0.0, 47.0, 390.0, 763.0));
    assert_eq!(r("Loading/Corner"), (340.0, 824.0, 50.0, 20.0));
}
