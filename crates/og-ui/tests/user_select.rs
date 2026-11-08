//! Selectable read-only text (CSS `user-select`, docs/ui.md §2): a press
//! and drag over text that allows it selects, across elements in the
//! tree's order, and Ctrl or Cmd+C copies it.
//!
//! Ways these could fail, each covered below:
//! - a wrapped line copied without the space its wrap dropped, a
//!   paragraph run into the next, or a drag across elements selecting
//!   them in drawing order instead of the tree's →
//!   `a_drag_selects_across_paragraphs_and_copies_their_source_text`
//! - a double press not selecting the word, a triple press not the line,
//!   `all` selecting by characters, `none` or a control's label selected,
//!   or a press elsewhere keeping the selection →
//!   `presses_select_words_lines_and_whole_elements_where_allowed`
//! - a long press on a touch screen not selecting (or a touch drag
//!   selecting instead of scrolling) → `a_long_touch_selects_a_word`
//! - the selection painted by repainting the screen (or not repainted
//!   where it moved), or lost over a save and restore →
//!   `a_selection_repaints_only_what_it_moved_and_survives_a_restore`
//! - its fill off the glyphs or over them →
//!   `the_selection_matches_its_reference_frames` (GPU)

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use support::*;

const DOC: &str = USER_SELECT_DOC;

struct Rig {
    ui: Ui,
    vp: Viewport,
    mounted: bool,
}

impl Rig {
    fn new(vp: Viewport, mounted: bool, font: UiFont) -> Rig {
        Rig::with(DOC, vp, mounted, font)
    }

    fn with(doc: &str, vp: Viewport, mounted: bool, font: UiFont) -> Rig {
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/sel.svelte", doc).expect("parses"));
        if mounted {
            ui.mount_document(
                SourceId::new("test", "sel"),
                "sel",
                BTreeMap::new(),
                BTreeMap::new(),
                &bind::NoEnv,
            )
            .expect("mounts");
        }
        let mut rig = Rig { ui, vp, mounted };
        rig.idle(2);
        rig
    }

    fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, self.vp);
        if self.mounted {
            self.ui.build_retained_documents(&bind::NoEnv).unwrap();
        } else {
            self.ui.component("T", "sel", &mut json!({}), &bind::NoEnv);
        }
        self.ui.end();
        assert!(
            self.ui.frame().warnings.is_empty(),
            "{:?}",
            self.ui.frame().warnings
        );
    }

    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.step(InputFrame::default());
        }
    }

    fn at(&mut self, p: Vec2, down: bool, pressed: bool, released: bool) {
        self.step(InputFrame {
            pointer: p,
            pointer_inside: true,
            down,
            pressed,
            released,
            ..InputFrame::default()
        });
    }

    fn el(&self, id: &str) -> UiElement {
        self.ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("{id}"))
    }

    /// A point `fx` across and `fy` down element `id`.
    fn point(&self, id: &str, fx: f32, fy: f32) -> Vec2 {
        let e = self.el(id);
        Vec2::new(e.x + e.width * fx, e.y + e.height * fy)
    }

    fn drag(&mut self, from: Vec2, to: Vec2) {
        self.at(from, true, true, false);
        for k in 1..=4 {
            let t = k as f32 / 4.0;
            self.at(from + (to - from) * t, true, false, false);
        }
        self.at(to, false, false, true);
    }

    /// `n` presses in a row at `p`.
    fn presses(&mut self, p: Vec2, n: usize) {
        for _ in 0..n {
            self.at(p, true, true, false);
            self.at(p, false, false, true);
        }
    }

    fn selection(&self) -> Value {
        self.ui.probe_extra()["selection"].clone()
    }

    fn selected(&self) -> String {
        self.selection()["text"].as_str().unwrap_or("").to_owned()
    }

    fn copy(&mut self) -> Option<String> {
        let _ = self.ui.take_clipboard();
        self.step(InputFrame {
            keys: vec![UiKey::KeyC],
            ctrl: true,
            ..InputFrame::default()
        });
        self.ui.take_clipboard()
    }
}

fn desktop() -> Viewport {
    Viewport::new(800.0, 600.0)
}

#[test]
fn a_drag_selects_across_paragraphs_and_copies_their_source_text() {
    for mounted in [false, true] {
        let mut rig = Rig::new(desktop(), mounted, test_font_metrics());
        // The narrow paragraph wraps: its lines are its words.
        let wrap = rig.el("Wrap");
        let second = rig.el("Second");
        assert!(wrap.height > second.height * 3.0, "{wrap:?}");
        // From before its first glyph to past the second's last, down
        // through both: every character of both, the wraps' spaces back.
        rig.drag(rig.point("Wrap", 0.0, 0.1), rig.point("Second", 1.0, 0.5));
        assert_eq!(
            rig.selected(),
            "The quick brown fox jumps over the lazy dog\nSecond paragraph",
            "mounted {mounted}"
        );
        assert_eq!(rig.selection()["anchor"]["at"], 0);
        assert!(
            rig.selection()["focus"]["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("Second")),
            "{}",
            rig.selection()
        );
        assert_eq!(rig.copy().as_deref(), Some(rig.selected().as_str()));
        // Back up from the second's middle to the first's: the same
        // order, whichever way the drag went; part of each.
        let p = rig.point("Second", 0.5, 0.5);
        rig.drag(p, rig.point("Wrap", 0.5, 0.95));
        let text = rig.selected();
        let (head, tail) = text.split_once('\n').expect("two paragraphs");
        assert!(
            !head.is_empty() && "The quick brown fox jumps over the lazy dog".ends_with(head),
            "{text:?}"
        );
        assert!(
            !tail.is_empty() && tail.len() < "Second paragraph".len(),
            "{text:?}"
        );
        assert!("Second paragraph".starts_with(tail), "{text:?}");
        // The selection survives the drag's end and a frame with no input.
        rig.idle(2);
        assert_eq!(rig.selected(), text);
    }
}

#[test]
fn presses_select_words_lines_and_whole_elements_where_allowed() {
    let mut rig = Rig::new(desktop(), true, test_font_metrics());
    // A double press in the second paragraph's first word.
    let p = rig.point("Second", 0.05, 0.5);
    rig.presses(p, 2);
    assert_eq!(rig.selected(), "Second");
    // A third: the line.
    rig.at(p, true, true, false);
    rig.at(p, false, false, true);
    assert_eq!(rig.selected(), "Second paragraph");
    // A press elsewhere (the column's padding) clears it.
    rig.idle(40);
    rig.presses(Vec2::new(4.0, 4.0), 1);
    assert_eq!(rig.selection(), Value::Null);
    // `all`: one press takes the whole element.
    rig.presses(rig.point("Code", 0.5, 0.5), 1);
    assert_eq!(rig.selected(), "ID-1234");
    // `none`, and a control's label: a press there selects nothing (and
    // the button still takes its press).
    rig.idle(40);
    rig.drag(rig.point("Off", 0.0, 0.5), rig.point("Off", 1.0, 0.5));
    assert_eq!(rig.selection(), Value::Null);
    rig.idle(40);
    rig.presses(rig.point("Go", 0.5, 0.5), 2);
    assert_eq!(rig.selection(), Value::Null);
    // The I-beam over selectable text, the arrow over what is not.
    rig.at(rig.point("Second", 0.5, 0.5), false, false, false);
    assert_eq!(rig.ui.frame().cursor, Cursor::Text);
    rig.at(rig.point("Off", 0.5, 0.5), false, false, false);
    assert_eq!(rig.ui.frame().cursor, Cursor::Default);
}

#[test]
fn a_long_touch_selects_a_word() {
    let vp = Viewport {
        coarse_pointer: true,
        ..desktop()
    };
    let mut rig = Rig::new(vp, true, test_font_metrics());
    let p = rig.point("Second", 0.05, 0.5);
    // A tap selects nothing.
    rig.presses(p, 1);
    assert_eq!(rig.selection(), Value::Null);
    // Held still for half a second: the word.
    rig.at(p, true, true, false);
    for _ in 0..40 {
        rig.at(p, true, false, false);
    }
    assert_eq!(rig.selected(), "Second");
    // Dragged on from there: by words, to the line's end.
    rig.at(rig.point("Second", 0.99, 0.5), true, false, false);
    rig.at(rig.point("Second", 0.99, 0.5), false, false, true);
    assert_eq!(rig.selected(), "Second paragraph");
    // A touch that moves at once is a scroll, not a selection.
    rig.idle(40);
    rig.drag(rig.point("Wrap", 0.1, 0.2), rig.point("Wrap", 0.1, 0.8));
    assert_eq!(rig.selection(), Value::Null);
}

#[test]
fn a_selection_repaints_only_what_it_moved_and_survives_a_restore() {
    // A long page: what a drag in one paragraph repaints must not grow
    // with it.
    let paras: String = (0..200)
        .map(|k| format!("<p id=\"P{k}\">Paragraph number {k} of the page</p>"))
        .collect();
    let doc = format!(
        "<meta name=\"component\" content=\"sel\">\n<column id=\"Doc\">{paras}</column>\n<style>\n#Doc {{ user-select: text; padding: 8px; }}\n</style>"
    );
    let mut rig = Rig::with(
        &doc,
        Viewport::new(800.0, 6000.0),
        true,
        test_font_metrics(),
    );
    let (a, b) = (rig.point("P3", 0.0, 0.5), rig.point("P3", 0.9, 0.5));
    rig.at(a, true, true, false);
    let mut painted = Vec::new();
    for k in 1..=6 {
        rig.at(a + (b - a) * (k as f32 / 6.0), true, false, false);
        painted.push(rig.ui.frame().stats.painted);
    }
    assert!(!rig.selected().is_empty());
    // The paragraph whose range moved, and nothing else.
    assert!(painted.iter().all(|&n| n <= 1), "{painted:?}");
    // On to the next paragraph: the one it reaches is painted too.
    rig.at(rig.point("P4", 0.5, 0.5), true, false, false);
    assert!(rig.ui.frame().stats.painted <= 2);
    rig.at(rig.point("P4", 0.5, 0.5), false, false, true);
    let before = rig.selection();
    let bytes = rig.ui.save_state();
    let mut again = Rig::with(
        &doc,
        Viewport::new(800.0, 6000.0),
        true,
        test_font_metrics(),
    );
    again.ui.restore_state(&bytes).expect("restores");
    again.idle(1);
    assert_eq!(again.selection(), before);
}

/// `references/<tier>/user-select-<width>.png` (`support::user_select_scene`,
/// which Chrome draws too: tests/web/reference-gfx.ts): a drag's selection
/// over a wrapped paragraph and into the next at 1280 and 390 wide.
#[test]
#[ignore = "requires GPU"]
fn the_selection_matches_its_reference_frames() {
    use og_render::{RenderTarget, Renderer};
    with_test_font(|files| {
        for name in ["user-select-1280", "user-select-390"] {
            let (scene, desc) = reference_scene(name, files).unwrap();
            let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
            let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            assert_eq!(stats.missing_glyphs, 0);
            let image = renderer.read_offscreen().unwrap();
            let tier = renderer.info().backend_tier();
            let actual = out_dir().join(format!("{name}-{tier}.png"));
            image::save_buffer(
                &actual,
                &image.pixels,
                image.width,
                image.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references");
            let own = dir.join(&tier).join(format!("{name}.png"));
            if std::env::var_os("OG_UI_CAPTURE").is_some() {
                record_reference(&own, image.width, image.height, &image.pixels);
            }
            let reference = if own.exists() {
                own
            } else {
                dir.join("metal").join(format!("{name}.png"))
            };
            let expected = image::open(&reference)
                .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1)", reference.display()))
                .to_rgba8();
            match compare(
                &image.pixels,
                expected.as_raw(),
                renderer.info(),
                &reference,
            ) {
                Ok((rmse, outliers)) => println!(
                    "{name} {tier}: RMSE {rmse:.5}, outliers {:.3}%",
                    outliers * 100.0
                ),
                Err(e) => panic!(
                    "{} differs from {}: {e}",
                    actual.display(),
                    reference.display()
                ),
            }
        }
    });
}
