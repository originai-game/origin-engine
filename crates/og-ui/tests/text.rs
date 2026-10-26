//! Text editing and truncation, tick by tick through `InputFrame`s: the
//! field's selection (Shift with the arrows, a drag, Ctrl/Cmd+A, C and X),
//! the multi-line area, an input method's composition, the caret's blink,
//! the typed-text stream, and labels cut with an ellipsis.
//!
//! Ways these could fail, each covered below:
//! - Shift+arrows move the caret without selecting, typing does not
//!   replace the selection, or Ctrl+C copies nothing or the wrong text →
//!   `shift_arrows_select_and_typing_replaces_the_selection`
//! - a drag selects from the wrong end, or a press does not collapse it →
//!   `a_drag_selects_and_a_press_places_the_caret`
//! - Enter submits a text area instead of breaking the line, ↑/↓ move
//!   focus instead of the caret, the area does not wrap →
//!   `a_text_area_breaks_lines_and_moves_between_them`
//! - a composition is committed early, not shown, does not replace the
//!   selection, or survives the commit →
//!   `a_composition_shows_at_the_caret_until_it_commits`
//! - the caret blinks by anything but ticks, or blinks under reduced motion
//!   → `the_caret_blinks_by_the_tick`
//! - keys type again once a typed-text stream exists (doubled letters) →
//!   `a_typed_text_stream_takes_over_from_the_keys`
//! - an ellipsis cut too early, too late, or the probe reads the cut text →
//!   `labels_cut_with_an_ellipsis_where_they_do_not_fit`
//! - the edit state does not survive a save and restore →
//!   `the_edit_state_is_saved_and_restored`

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

struct Rig {
    ui: Ui,
    value: String,
    area: String,
    vp: Viewport,
}

fn build(ui: &mut Ui, value: &mut String, area: &mut String) -> (Response, Response) {
    let mut r = (Response::default(), Response::default());
    ui.column("Form", Element::column(), |ui| {
        r.0 = ui.text_field(
            "Name",
            value,
            &FieldDesc {
                width: Some(Size::Fixed(300.0)),
                ..FieldDesc::default()
            },
        );
        r.1 = ui.text_area(
            "Notes",
            area,
            &TextAreaDesc {
                width: Some(Size::Fixed(200.0)),
                rows: 3,
                ..TextAreaDesc::default()
            },
        );
    });
    r
}

impl Rig {
    fn new(value: &str) -> Rig {
        let mut scene = og_render::RenderScene::new();
        Rig {
            ui: ui_with_font(add_test_font(&mut scene)),
            value: value.into(),
            area: String::new(),
            vp: Viewport::new(800.0, 600.0),
        }
    }

    fn step(&mut self, input: InputFrame) -> (Response, Response) {
        self.ui.begin(input, self.vp);
        let r = build(&mut self.ui, &mut self.value, &mut self.area);
        self.ui.end();
        r
    }

    fn rect(&self, id: &str) -> (f32, f32, f32, f32) {
        let e = self.ui.frame().element(id).expect(id);
        (e.x, e.y, e.width, e.height)
    }

    /// A click at a point (press, then release a tick later).
    fn click(&mut self, x: f32, y: f32, shift: bool) {
        let at = Vec2::new(x, y);
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            down: true,
            pressed: true,
            shift,
            ..InputFrame::default()
        });
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        });
    }

    fn keys(&mut self, keys: &[UiKey], shift: bool, ctrl: bool) {
        self.step(InputFrame {
            keys: keys.to_vec(),
            shift,
            ctrl,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    fn typed(&mut self, text: &str) {
        let events: Vec<og_input::TextInput> = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
    }

    fn compose(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, true);
        self.step(InputFrame::default().with_text(&events));
    }

    fn focus_field(&mut self) {
        self.step(InputFrame::default());
        let (x, y, w, h) = self.rect("Name");
        self.click(x + w - 2.0, y + h / 2.0, false);
    }

    fn target(&self) -> TextTarget {
        self.ui
            .frame()
            .text_input
            .clone()
            .expect("a focused text control")
    }
}

#[test]
fn shift_arrows_select_and_typing_replaces_the_selection() {
    let mut rig = Rig::new("hello world");
    rig.focus_field();
    // The press at the field's right end put the caret after "world".
    rig.keys(
        &[
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
            UiKey::Left,
        ],
        true,
        false,
    );
    assert_eq!(rig.target().selection, "world");
    rig.keys(&[UiKey::KeyC], false, true);
    assert_eq!(rig.ui.take_clipboard().as_deref(), Some("world"));
    assert_eq!(rig.ui.take_clipboard(), None, "taken once");
    rig.typed("there");
    assert_eq!(rig.value, "hello there");
    assert_eq!(rig.target().selection, "", "typing collapsed the selection");
    // Home with Shift selects back to the start; Ctrl+X cuts it.
    rig.keys(&[UiKey::Home], true, false);
    assert_eq!(rig.target().selection, "hello there");
    rig.keys(&[UiKey::KeyX], false, true);
    assert_eq!(rig.ui.take_clipboard().as_deref(), Some("hello there"));
    assert_eq!(rig.value, "");
    // Ctrl+A selects everything; Backspace deletes the selection.
    rig.typed("abc");
    rig.keys(&[UiKey::KeyA], false, true);
    assert_eq!(rig.target().selection, "abc");
    rig.keys(&[UiKey::Backspace], false, false);
    assert_eq!(rig.value, "");
    // Left without Shift collapses a selection to its start.
    rig.typed("xyz");
    rig.keys(&[UiKey::Left, UiKey::Left], true, false);
    rig.keys(&[UiKey::Left], false, false);
    rig.typed("_");
    assert_eq!(rig.value, "x_yz");
}

#[test]
fn a_drag_selects_and_a_press_places_the_caret() {
    let mut rig = Rig::new("drag to select");
    rig.focus_field();
    let (x, y, _, h) = rig.rect("Name");
    let cy = y + h / 2.0;
    // Press near the start of the text, drag to the right, release.
    let start = Vec2::new(x + 14.0, cy);
    rig.step(InputFrame {
        pointer: start,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    });
    for dx in [20.0, 40.0, 60.0] {
        rig.step(InputFrame {
            pointer: start + Vec2::new(dx, 0.0),
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        });
    }
    let sel = rig.target().selection;
    assert!(
        "drag to select".starts_with(&sel) && sel.len() >= 5,
        "a drag of 60 px from the start selects the start: {sel:?}"
    );
    rig.step(InputFrame {
        pointer: start + Vec2::new(60.0, 0.0),
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    });
    assert_eq!(rig.target().selection, sel, "the release keeps it");
    // A press elsewhere in the field collapses it there; Shift+press extends.
    rig.click(x + 14.0, cy, false);
    assert_eq!(rig.target().selection, "");
    rig.click(x + 14.0 + 60.0, cy, true);
    assert_eq!(
        rig.target().selection,
        sel,
        "Shift+press extends to the press"
    );
}

#[test]
fn a_text_area_breaks_lines_and_moves_between_them() {
    let mut rig = Rig::new("");
    rig.step(InputFrame::default());
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(x + w / 2.0, y + 10.0, false);
    assert!(rig.target().multiline);
    rig.typed("first");
    let (_, r) = rig.step(InputFrame {
        keys: vec![UiKey::Enter],
        ..InputFrame::default()
    });
    assert!(!r.submitted, "Enter breaks the line in an area");
    rig.typed("second");
    assert_eq!(rig.area, "first\nsecond");
    // Up moves to the line above (focus stays), Shift+End selects its end.
    rig.keys(&[UiKey::Up], false, false);
    assert!(rig.ui.frame().text_input.is_some(), "↑ belongs to the area");
    rig.keys(&[UiKey::Home], false, false);
    rig.keys(&[UiKey::End], true, false);
    assert_eq!(rig.target().selection, "first");
    rig.keys(&[UiKey::Down], false, false);
    rig.keys(&[UiKey::End], false, false);
    rig.typed("!");
    assert_eq!(rig.area, "first\nsecond!");
    // A long line wraps between words at the area's width (200 px).
    rig.typed(" and then a much longer line of words that cannot fit");
    let probe = rig.ui.frame().element("Notes").expect("area").text.clone();
    assert!(probe.ends_with("cannot fit"), "the probe reads the value");
}

#[test]
fn a_composition_shows_at_the_caret_until_it_commits() {
    let mut rig = Rig::new("ab");
    rig.focus_field();
    rig.keys(&[UiKey::Left], false, false);
    rig.compose("ni");
    assert_eq!(rig.value, "ab", "nothing is committed while composing");
    assert!(rig.target().composing);
    assert_eq!(
        rig.ui.frame().element("Name").expect("field").text,
        "ab",
        "the probe reads the value, not the composition"
    );
    // The composing text is drawn in the field (the shown text has it at the caret).
    let shown: Vec<String> = rig
        .ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(shown.iter().any(|t| t == "anib"), "{shown:?}");
    rig.compose("你");
    let events = og_input::TextInput::split("你", false);
    rig.step(InputFrame::default().with_text(&events));
    assert_eq!(rig.value, "a你b");
    assert!(!rig.target().composing, "the commit ended the composition");
    // A composition replaces the selection as it starts.
    rig.keys(&[UiKey::KeyA], false, true);
    rig.compose("x");
    assert_eq!(rig.value, "");
    rig.compose("");
    assert!(!rig.target().composing, "an empty composition ends it");
}

#[test]
fn the_caret_blinks_by_the_tick() {
    let caret = |rig: &Rig| {
        rig.ui.frame().items.iter().any(|i| {
            matches!(&i.cmd, DrawCmd::Quad { rect, .. } if rect.width <= 1.01 && rect.height > 10.0)
        })
    };
    let mut rig = Rig::new("blink");
    rig.focus_field();
    // 60 ticks a second: shown for 32 ticks after the last move, then hidden 32.
    let mut seen = Vec::new();
    for _ in 0..70 {
        rig.step(InputFrame::default());
        seen.push(caret(&rig));
    }
    let first_off = seen.iter().position(|s| !s).expect("it hides");
    let back_on = seen[first_off..]
        .iter()
        .position(|s| *s)
        .expect("it shows again")
        + first_off;
    assert!(
        (29..=32).contains(&first_off),
        "hidden after about 32 ticks: {first_off}"
    );
    assert_eq!(back_on - first_off, 32, "for 32 ticks");
    // A move shows it at once.
    rig.keys(&[UiKey::Left], false, false);
    assert!(caret(&rig));
    // Under reduced motion it never hides.
    let mut theme = *rig.ui.theme();
    theme.reduced_motion = true;
    rig.ui.set_theme(theme);
    for _ in 0..80 {
        rig.step(InputFrame::default());
        assert!(caret(&rig));
    }
}

#[test]
fn a_typed_text_stream_takes_over_from_the_keys() {
    let mut rig = Rig::new("");
    rig.focus_field();
    // Before any stream, key presses type (the probe-less fallback).
    rig.step(InputFrame {
        text: "k".into(),
        ..InputFrame::default()
    });
    assert_eq!(rig.value, "k");
    rig.typed("s");
    assert_eq!(rig.value, "ks");
    // From then on a key's text is ignored (the stream brings it), so a key
    // and its typed character arriving in different ticks type once.
    rig.step(InputFrame {
        text: "x".into(),
        ..InputFrame::default()
    });
    assert_eq!(rig.value, "ks");
}

#[test]
fn labels_cut_with_an_ellipsis_where_they_do_not_fit() {
    let mut scene = og_render::RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let long = "A label far too long for the narrow column it sits in";
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.column(
            "Col",
            Element {
                width: Size::Fixed(160.0),
                ..Element::column()
            },
            |ui| {
                ui.label(
                    "One",
                    long,
                    LabelDesc {
                        nowrap: true,
                        ellipsis: true,
                        width: Size::grow(),
                        ..LabelDesc::default()
                    },
                );
                ui.label(
                    "Two",
                    long,
                    LabelDesc {
                        ellipsis: true,
                        max_lines: 2,
                        width: Size::grow(),
                        ..LabelDesc::default()
                    },
                );
                ui.label(
                    "Fits",
                    "Short",
                    LabelDesc {
                        nowrap: true,
                        ellipsis: true,
                        ..LabelDesc::default()
                    },
                );
            },
        );
        ui.end();
    };
    step(&mut ui);
    let texts: Vec<(String, f32, f32)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, bounds, .. } => Some((text.clone(), bounds.width, bounds.y)),
            _ => None,
        })
        .collect();
    // The test font has no "…": its ellipsis is three stops.
    let cut = |t: &str| t.ends_with('…') || t.ends_with("...");
    let one = texts
        .iter()
        .find(|t| t.0.starts_with("A label") && cut(&t.0));
    let one = one.unwrap_or_else(|| panic!("a cut line: {texts:?}"));
    assert!(one.1 <= 160.5, "it fits its 160 px: {one:?}");
    let two: Vec<&(String, f32, f32)> = texts
        .iter()
        .filter(|t| t.2 > one.2 + 1.0 && !t.0.starts_with("Short"))
        .collect();
    assert_eq!(two.len(), 2, "two lines: {two:?}");
    assert!(cut(&two[1].0) && !cut(&two[0].0), "{two:?}");
    assert!(
        texts.iter().any(|t| t.0 == "Short"),
        "a text that fits is not cut"
    );
    assert_eq!(
        ui.frame().element("One").expect("one").text,
        long,
        "the probe reads the whole text"
    );
}

#[test]
fn the_edit_state_is_saved_and_restored() {
    let mut rig = Rig::new("snapshot");
    rig.focus_field();
    rig.keys(&[UiKey::Left, UiKey::Left], true, false);
    rig.compose("zz");
    let saved = rig.ui.save_state();
    let before = rig.target();
    rig.keys(&[UiKey::Home], false, false);
    rig.compose("");
    rig.ui.restore_state(&saved).expect("restores");
    rig.step(InputFrame::default());
    let after = rig.target();
    assert_eq!(after.selection, before.selection);
    assert!(after.composing);
}
