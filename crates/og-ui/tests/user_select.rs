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
//! - `::selection`'s color, decoration or shadow painting the whole line
//!   (or not the selected glyphs) →
//!   `selection_ink_lines_and_shadows_cover_only_the_selected_run`
//! - its fill off the glyphs or over them →
//!   `the_selection_matches_its_reference_frames` (GPU)

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::Value;
use std::collections::BTreeMap;
use support::*;

const DOC: &str = USER_SELECT_DOC;

/// The document `sel`, mounted, and the viewport its frames lay out in.
struct Rig {
    ui: Ui,
    vp: Viewport,
}

impl Rig {
    fn new(vp: Viewport, font: UiFont) -> Rig {
        Rig::with(DOC, vp, font)
    }

    fn with(doc: &str, vp: Viewport, font: UiFont) -> Rig {
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/sel.svelte", doc).expect("parses"));
        ui.mount_document(
            SourceId::new("test", "sel"),
            "sel",
            BTreeMap::new(),
            BTreeMap::new(),
            &bind::NoEnv,
        )
        .expect("mounts");
        let mut rig = Rig { ui, vp };
        rig.idle(2);
        rig
    }

    fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, self.vp);
        self.ui.build_retained_documents(&bind::NoEnv).unwrap();
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
    let mut rig = Rig::new(desktop(), test_font_metrics());
    // The narrow paragraph wraps: its lines are its words.
    let wrap = rig.el("Wrap");
    let second = rig.el("Second");
    assert!(wrap.height > second.height * 3.0, "{wrap:?}");
    // From before its first glyph to past the second's last, down
    // through both: every character of both, the wraps' spaces back.
    rig.drag(rig.point("Wrap", 0.0, 0.1), rig.point("Second", 1.0, 0.5));
    assert_eq!(
        rig.selected(),
        "The quick brown fox jumps over the lazy dog\nSecond paragraph"
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

#[test]
fn presses_select_words_lines_and_whole_elements_where_allowed() {
    let mut rig = Rig::new(desktop(), test_font_metrics());
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
fn selection_ink_lines_and_shadows_cover_only_the_selected_run() {
    let doc = DOC.replace(
        "</style>",
        "p::selection { color: #ff0000; text-decoration: underline #00ff00; text-shadow: 2px 2px #0000ff; }\n</style>",
    );
    let mut rig = Rig::with(&doc, desktop(), test_font());
    rig.presses(rig.point("Second", 0.05, 0.5), 2);
    assert_eq!(rig.selected(), "Second");
    let e = rig.el("Second");
    let red = Srgba::rgb(255, 0, 0).linear();
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02);
    let (mut red_clips, mut other_clips, mut lines, mut shadows) = (vec![], vec![], vec![], 0);
    for item in &rig.ui.frame().items {
        let (cmd, shadowed) = match &item.cmd {
            DrawCmd::StyledText { text, effects } => (
                text.as_ref(),
                effects
                    .shadows
                    .iter()
                    .any(|s| s.color[2] > 0.9 && s.offset.x > 1.0),
            ),
            c => (c, false),
        };
        match cmd {
            DrawCmd::Text {
                color,
                clip,
                bounds,
                ..
            } if bounds.y >= e.y - 1.0 && bounds.y < e.y + e.height => {
                if near(*color, red) {
                    red_clips.push(*clip);
                    shadows += shadowed as usize;
                } else {
                    other_clips.push(*clip);
                }
            }
            DrawCmd::Quad { rect, color, .. } if near(*color, Srgba::rgb(0, 255, 0).linear()) => {
                lines.push(*rect)
            }
            _ => {}
        }
    }
    // The selected word in red with its blue shadow, clipped to it.
    assert!(!red_clips.is_empty(), "no selected ink");
    assert!(shadows > 0, "no selection shadow");
    for c in &red_clips {
        assert!(
            c.x >= e.x - 1.0 && c.right() < e.x + e.width * 0.6,
            "{c:?} {e:?}"
        );
    }
    // The rest of the line keeps its own ink, outside the selected run.
    let run = red_clips[0];
    assert!(
        other_clips.iter().any(|c| c.x >= run.right() - 0.5),
        "{other_clips:?}"
    );
    // One green underline under the word, not the line.
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        (lines[0].x - run.x).abs() < 1.0 && lines[0].width < e.width * 0.6,
        "{lines:?}"
    );
}

/// Vertical text takes the player's selection down its lines: a press
/// picks the word under it along the column, the selection's fill runs
/// down that column, and `::selection`'s ink covers only the word. A
/// justified line reaches its box's end.
#[test]
fn a_vertical_text_selects_down_its_lines() {
    let doc = r#"<meta name="component" content="sel">
<div id="Doc"><p id="V">alpha beta gamma delta epsilon zeta</p><p id="J">alpha beta gamma delta epsilon zeta</p></div>
<style>
#Doc { user-select: text; padding: 16px; display: flex; gap: 40px; }
p { writing-mode: vertical-rl; height: 200px; margin: 0; }
#J { text-align: justify; }
p::selection { color: #ff0000; }
</style>"#;
    let mut rig = Rig::with(doc, desktop(), test_font());
    let e = rig.el("V");
    assert!(e.width > e.height / 10.0, "wraps into columns: {e:?}");
    // Its first line is its rightmost column; a press a little down it
    // takes the first word, one further down the second.
    let at = |rig: &Rig, fy: f32| rig.point("V", 0.9, fy);
    rig.presses(at(&rig, 0.02), 2);
    assert_eq!(rig.selected(), "alpha");
    rig.presses(at(&rig, 0.2), 2);
    assert_eq!(rig.selected(), "beta");
    let red = Srgba::rgb(255, 0, 0).linear();
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02);
    let (mut red_clips, mut fills) = (vec![], vec![]);
    for item in &rig.ui.frame().items {
        let cmd = match &item.cmd {
            DrawCmd::StyledText { text, .. } => text.as_ref(),
            c => c,
        };
        match cmd {
            DrawCmd::Text { color, clip, .. } if near(*color, red) => red_clips.push(*clip),
            _ => {
                if let Some((rect, _)) = cmd.filled()
                    && rect.x >= e.x - 1.0
                    && rect.right() <= e.x + e.width + 1.0
                {
                    fills.push(rect);
                }
            }
        }
    }
    // The fill: one band down the first column, the word's length.
    assert_eq!(fills.len(), 1, "{fills:?}");
    let f = fills[0];
    assert!(
        f.height > f.width && f.x > e.x + e.width / 2.0,
        "{f:?} {e:?}"
    );
    assert!(
        f.y > e.y + 5.0 && f.bottom() < e.y + e.height * 0.5,
        "{f:?}"
    );
    // The word redrawn in red, cut to its stretch down the line.
    assert!(!red_clips.is_empty(), "no selected ink");
    for c in &red_clips {
        assert!(
            (c.y - f.y).abs() < 1.0 && (c.bottom() - f.bottom()).abs() < 1.0,
            "{c:?} {f:?}"
        );
    }
    // Justified, the first line's last word ends at the box's bottom.
    let j = rig.el("J");
    rig.presses(rig.point("J", 0.9, 0.5), 3);
    let line_end = rig
        .ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match it.cmd.filled() {
            Some((rect, _)) if rect.x >= j.x - 1.0 && rect.right() <= j.x + j.width + 1.0 => {
                Some(rect.bottom())
            }
            _ => None,
        })
        .fold(0.0, f32::max);
    assert!(
        (line_end - (j.y + j.height)).abs() < 1.5,
        "{line_end} {j:?}"
    );
}

/// A vertical rich paragraph takes the player's selection as a plain one
/// does: a double press picks the word under it down its column (across
/// its spans), its fill runs down that column, `::selection`'s ink, line
/// and shadow cover only that stretch, the glyphs' own ink is cut there
/// (not drawn under it), and the copy is the source text.
#[test]
fn a_vertical_rich_paragraph_selects_down_its_columns() {
    let doc = r#"<meta name="component" content="sel">
<div id="Doc"><p id="V">alpha <b>beta</b> gamma delta epsilon zeta</p></div>
<style>
#Doc { user-select: text; padding: 16px; }
p { writing-mode: vertical-rl; height: 200px; margin: 0; }
p::selection { color: #ff0000; text-decoration: underline #00ff00; text-shadow: 2px 2px #0000ff; }
</style>"#;
    let mut rig = Rig::with(doc, desktop(), test_font());
    let e = rig.el("V");
    assert_eq!(e.kind, "richtext");
    rig.presses(rig.point("V", 0.9, 0.02), 2);
    assert_eq!(rig.selected(), "alpha");
    rig.presses(rig.point("V", 0.9, 0.22), 2);
    assert_eq!(rig.selected(), "beta");
    let red = Srgba::rgb(255, 0, 0).linear();
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02);
    let green = Srgba::rgb(0, 255, 0).linear();
    let (mut red_clips, mut other_clips, mut fills, mut lines, mut shadows) =
        (vec![], vec![], vec![], vec![], 0);
    for item in &rig.ui.frame().items {
        let (cmd, shadowed) = match &item.cmd {
            DrawCmd::StyledText { text, effects } => (
                text.as_ref(),
                effects
                    .shadows
                    .iter()
                    .any(|s| s.color[2] > 0.9 && s.offset.x > 1.0),
            ),
            c => (c, false),
        };
        match cmd {
            DrawCmd::Text { color, clip, .. } if near(*color, red) => {
                red_clips.push(*clip);
                shadows += shadowed as usize;
            }
            // Where its own ink shows: its glyphs' box within its clip.
            DrawCmd::Text { clip, bounds, .. } => other_clips.push(bounds.intersect(clip)),
            DrawCmd::Quad { rect, color, .. } if near(*color, green) => lines.push(*rect),
            _ => {
                if let Some((rect, _)) = cmd.filled()
                    && rect.x >= e.x - 1.0
                    && rect.right() <= e.x + e.width + 1.0
                {
                    fills.push(rect);
                }
            }
        }
    }
    assert_eq!(fills.len(), 1, "{fills:?}");
    let f = fills[0];
    assert!(
        f.height > f.width && f.x >= e.x + e.width / 2.0,
        "{f:?} {e:?}"
    );
    assert!(!red_clips.is_empty(), "no selected ink");
    assert!(shadows > 0, "no selection shadow");
    for c in &red_clips {
        assert!(
            c.y >= f.y - 1.0 && c.bottom() <= f.bottom() + 1.0,
            "{c:?} {f:?}"
        );
    }
    // The glyphs' own ink stops at the stretch: a translucent ::selection
    // colour would blend with it.
    for c in &other_clips {
        let inside = c.y.max(f.y) < c.bottom().min(f.bottom()) - 0.5
            && c.x.max(f.x) < c.right().min(f.right());
        assert!(!inside, "own ink under the selection: {c:?} {f:?}");
    }
    // One green line down the word, beside it.
    assert_eq!(lines.len(), 1, "{lines:?}");
    let l = lines[0];
    assert!(
        l.height > l.width && l.y >= f.y - 1.0 && l.bottom() <= f.bottom() + 1.0,
        "{l:?} {f:?}"
    );
    // A drag from the first word down past the end copies the source.
    let from = rig.point("V", 0.9, 0.0);
    rig.at(from, true, true, false);
    rig.at(rig.point("V", 0.02, 0.99), true, false, false);
    rig.at(rig.point("V", 0.02, 0.99), false, false, true);
    assert_eq!(rig.selected(), "alpha beta gamma delta epsilon zeta");
}

#[test]
fn a_long_touch_selects_a_word() {
    let vp = Viewport {
        coarse_pointer: true,
        ..desktop()
    };
    let mut rig = Rig::new(vp, test_font_metrics());
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
    let mut rig = Rig::with(&doc, Viewport::new(800.0, 6000.0), test_font_metrics());
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
    let mut again = Rig::with(&doc, Viewport::new(800.0, 6000.0), test_font_metrics());
    again.ui.restore_state(&bytes).expect("restores");
    again.idle(1);
    assert_eq!(again.selection(), before);
}
