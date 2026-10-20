//! The fields' second round of editing, tick by tick through `InputFrame`s:
//! moves and deletes by words, double and triple clicks, undo and redo, and
//! a drag that scrolls a text area past its edge.
//!
//! Ways these could fail, each covered below:
//! - Alt or Ctrl+arrows move one character, stop inside a word or skip
//!   punctuation, Ctrl+Backspace deletes one character, Ctrl+Home/End stop
//!   at the line → `words_move_and_delete_with_alt_or_ctrl`
//! - a double click selects nothing or the wrong word, a triple click less
//!   than the line, a drag after them extends by characters, a slow second
//!   press or one after a drag counts as a double click →
//!   `double_and_triple_clicks_select_words_and_lines`
//! - undo takes a whole sentence or a character at a time, redo is lost
//!   after an undo, a new edit keeps the redo, a text the game set is undone
//!   into, the history grows without bound or is not saved →
//!   `undo_and_redo_step_through_the_history`
//! - a drag below a text area jumps to the end (or does not scroll), keeps
//!   scrolling past the end, or scrolls while the pointer is inside →
//!   `a_drag_past_a_text_area_scrolls_it_line_by_line`

mod support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

struct Rig {
    ui: Ui,
    value: String,
    area: String,
    vp: Viewport,
}

fn build(ui: &mut Ui, value: &mut String, area: &mut String) {
    ui.column("Form", Element::column(), |ui| {
        ui.text_field(
            "Name",
            value,
            &FieldDesc {
                width: Some(Size::Fixed(300.0)),
                ..FieldDesc::default()
            },
        );
        ui.text_area(
            "Notes",
            area,
            &TextAreaDesc {
                width: Some(Size::Fixed(200.0)),
                rows: 3,
                ..TextAreaDesc::default()
            },
        );
    });
}

impl Rig {
    fn new(value: &str, area: &str) -> Rig {
        let mut scene = og_render::RenderScene::new();
        let mut rig = Rig {
            ui: ui_with_font(add_test_font(&mut scene)),
            value: value.into(),
            area: area.into(),
            vp: Viewport::new(800.0, 600.0),
        };
        rig.idle(1);
        rig
    }

    fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, self.vp);
        build(&mut self.ui, &mut self.value, &mut self.area);
        self.ui.end();
    }

    fn idle(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.step(InputFrame::default());
        }
    }

    fn rect(&self, id: &str) -> (f32, f32, f32, f32) {
        let e = self.ui.frame().element(id).expect(id);
        (e.x, e.y, e.width, e.height)
    }

    fn press(&mut self, at: Vec2, shift: bool) {
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            down: true,
            pressed: true,
            shift,
            ..InputFrame::default()
        });
    }

    fn release(&mut self, at: Vec2) {
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        });
    }

    fn drag(&mut self, at: Vec2) {
        self.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        });
    }

    fn click(&mut self, at: Vec2) {
        self.press(at, false);
        self.release(at);
    }

    fn keys(&mut self, keys: &[UiKey], shift: bool, ctrl: bool, alt: bool) {
        self.step(InputFrame {
            keys: keys.to_vec(),
            shift,
            ctrl,
            alt,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    fn typed(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
    }

    /// Types one character a tick.
    fn type_slowly(&mut self, text: &str) {
        for c in text.chars() {
            self.typed(&c.to_string());
        }
    }

    fn selection(&self) -> String {
        self.ui
            .frame()
            .text_input
            .clone()
            .expect("a focused text control")
            .selection
    }

    /// The field's caret, found by typing a marker there and taking it out.
    fn caret_marked(&mut self) -> String {
        self.typed("|");
        let marked = self.value.clone();
        self.keys(&[UiKey::Backspace], false, false, false);
        marked
    }

    /// Where character `k` of the field's text starts (CSS px), measured
    /// with the field's font and offset.
    fn field_x(&self, k: usize) -> f32 {
        let (x, _, _, _) = self.rect("Name");
        let pad = self.ui.theme().space.md;
        let size = self.ui.theme().type_ramp.body.size;
        let m = &self.ui.fonts().body.metrics;
        let s: String = self.value.chars().take(k).collect();
        x + pad + m.measure(&s, size)
    }

    fn focus_field(&mut self) {
        let (x, y, w, h) = self.rect("Name");
        self.click(Vec2::new(x + w - 2.0, y + h / 2.0));
        // Past the double-click window, so the next press is a first click.
        self.idle(40);
    }
}

#[test]
fn words_move_and_delete_with_alt_or_ctrl() {
    let mut rig = Rig::new("hello brave, new world", "");
    rig.focus_field();
    // From the end: Alt+← to the start of "world", then of "new".
    rig.keys(&[UiKey::Left], false, false, true);
    assert_eq!(rig.caret_marked(), "hello brave, new |world");
    rig.keys(&[UiKey::Left], false, true, false);
    assert_eq!(rig.caret_marked(), "hello brave, |new world");
    // The comma is a word of its own; then "brave".
    rig.keys(&[UiKey::Left, UiKey::Left], false, false, true);
    assert_eq!(rig.caret_marked(), "hello |brave, new world");
    // Ctrl+→ to the end of "brave", Shift extends to the comma's end.
    rig.keys(&[UiKey::Right], false, true, false);
    assert_eq!(rig.caret_marked(), "hello brave|, new world");
    rig.keys(&[UiKey::Right], true, true, false);
    assert_eq!(rig.selection(), ",");
    // Alt+Backspace deletes the word before the caret, Alt+Delete the one after.
    rig.keys(&[UiKey::End], false, false, false);
    rig.keys(&[UiKey::Backspace], false, false, true);
    assert_eq!(rig.value, "hello brave, new ");
    rig.keys(&[UiKey::Home], false, false, false);
    rig.keys(&[UiKey::Delete], false, true, false);
    assert_eq!(rig.value, " brave, new ");
    // Ctrl+End and Ctrl+Home: the text's ends, in an area across lines.
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(Vec2::new(x + w / 2.0, y + 6.0));
    rig.idle(40);
    rig.typed("one\ntwo\nthree");
    rig.keys(&[UiKey::Up], false, false, false);
    rig.keys(&[UiKey::Home], true, true, false);
    assert_eq!(
        rig.selection(),
        "one\ntwo",
        "Shift+Ctrl+Home selects to the start"
    );
    rig.keys(&[UiKey::End], true, true, false);
    assert_eq!(
        rig.selection(),
        "\nthree",
        "Shift+Ctrl+End selects to the end"
    );
}

#[test]
fn double_and_triple_clicks_select_words_and_lines() {
    let mut rig = Rig::new("drag to select words", "");
    rig.focus_field();
    let (_, y, _, h) = rig.rect("Name");
    let cy = y + h / 2.0;
    // Inside "select": a double click selects it, a third press the field.
    let at = Vec2::new(rig.field_x(10), cy);
    rig.click(at);
    assert_eq!(rig.selection(), "");
    rig.click(at + Vec2::new(1.0, 0.0));
    assert_eq!(rig.selection(), "select");
    rig.click(at);
    assert_eq!(rig.selection(), "drag to select words");
    rig.idle(40);
    // A double click, the second press held and dragged left: by words.
    rig.click(at);
    rig.press(at, false);
    assert_eq!(rig.selection(), "select");
    rig.drag(Vec2::new(rig.field_x(1), cy));
    assert_eq!(rig.selection(), "drag to select", "whole words to the left");
    rig.drag(Vec2::new(rig.field_x(19), cy));
    assert_eq!(rig.selection(), "select words", "and to the right");
    rig.release(Vec2::new(rig.field_x(19), cy));
    rig.idle(40);
    // Too slow: two clicks 40 ticks apart are two first clicks.
    rig.click(at);
    rig.idle(40);
    rig.click(at);
    assert_eq!(rig.selection(), "", "a slow second press");
    rig.idle(40);
    // A press that dragged is no first click of a double click.
    rig.press(at, false);
    rig.drag(at + Vec2::new(30.0, 0.0));
    rig.release(at + Vec2::new(30.0, 0.0));
    rig.click(at);
    assert_eq!(rig.selection(), "", "a press after a drag");
    rig.idle(40);
    // A text area: the triple click takes the line between its breaks.
    let (x, y, _, _) = rig.rect("Notes");
    rig.click(Vec2::new(x + 20.0, y + 10.0));
    rig.idle(40);
    rig.typed("first line\nsecond line");
    rig.keys(&[UiKey::Up], false, false, false);
    let pad = rig.ui.theme().space.sm;
    let lh = rig.ui.theme().type_ramp.body.line_height;
    let second = Vec2::new(x + 30.0, y + pad + lh * 1.5);
    for _ in 0..3 {
        rig.click(second);
    }
    assert_eq!(rig.selection(), "second line");
    // A double click on a space selects the spaces.
    rig.idle(40);
    rig.value = "a   b".into();
    rig.focus_field();
    let mid = Vec2::new(rig.field_x(2), cy);
    rig.click(mid);
    rig.click(mid);
    assert_eq!(rig.selection(), "   ");
}

#[test]
fn undo_and_redo_step_through_the_history() {
    let mut rig = Rig::new("", "");
    rig.focus_field();
    rig.type_slowly("one two three");
    assert_eq!(rig.value, "one two three");
    let undo = |rig: &mut Rig| rig.keys(&[UiKey::KeyZ], false, true, false);
    let redo = |rig: &mut Rig| rig.keys(&[UiKey::KeyZ], true, true, false);
    // Typing groups by words with their spaces.
    undo(&mut rig);
    assert_eq!(rig.value, "one two ");
    undo(&mut rig);
    assert_eq!(rig.value, "one ");
    redo(&mut rig);
    assert_eq!(rig.value, "one two ");
    // Ctrl+Y redoes too, and the caret lands after what came back.
    rig.keys(&[UiKey::KeyY], false, true, false);
    assert_eq!(rig.value, "one two three");
    rig.keys(&[UiKey::Left], true, false, true);
    assert_eq!(rig.selection(), "three", "the caret after it");
    rig.keys(&[UiKey::End], false, false, false);
    // Backspaces in a row are one step; a move ends it.
    for _ in 0..3 {
        rig.keys(&[UiKey::Backspace], false, false, false);
    }
    rig.keys(&[UiKey::Left], false, false, false);
    rig.keys(&[UiKey::Backspace], false, false, false);
    assert_eq!(rig.value, "one two h");
    undo(&mut rig);
    assert_eq!(rig.value, "one two th");
    undo(&mut rig);
    assert_eq!(rig.value, "one two three");
    // A cut is a step of its own, and a new edit drops the redo.
    rig.keys(&[UiKey::KeyA], false, true, false);
    rig.keys(&[UiKey::KeyX], false, true, false);
    assert_eq!(rig.value, "");
    undo(&mut rig);
    assert_eq!(rig.value, "one two three");
    assert_eq!(
        rig.selection(),
        "one two three",
        "the cut text selected again"
    );
    rig.typed("x");
    redo(&mut rig);
    assert_eq!(rig.value, "x", "nothing to redo after a new edit");
    // Undo and redo survive a save and restore, and replay the same.
    let saved = rig.ui.save_state();
    undo(&mut rig);
    let once = rig.value.clone();
    rig.value = "x".into();
    rig.ui.restore_state(&saved).expect("restores");
    undo(&mut rig);
    assert_eq!(rig.value, once);
    assert_eq!(once, "one two three");
    // A text the game set starts the history again: nothing to undo into.
    rig.value = "set by the game".into();
    rig.step(InputFrame::default());
    undo(&mut rig);
    assert_eq!(rig.value, "set by the game");
    // The history keeps its last 100 steps.
    for k in 0..150 {
        rig.typed(if k % 2 == 0 { "a " } else { "b " });
    }
    let mut steps = 0;
    while {
        let v = rig.value.clone();
        undo(&mut rig);
        v != rig.value
    } {
        steps += 1;
    }
    assert_eq!(steps, 100, "a bounded history");
}

#[test]
fn a_drag_past_a_text_area_scrolls_it_line_by_line() {
    let lines: Vec<String> = (1..=20).map(|k| format!("line {k}")).collect();
    let mut rig = Rig::new("", &lines.join("\n"));
    let (x, y, w, h) = rig.rect("Notes");
    let start = Vec2::new(x + 4.0, y + 10.0);
    rig.press(start, false);
    // Inside the area: no scroll, the selection follows the pointer.
    rig.drag(start + Vec2::new(0.0, 20.0));
    let inside = rig.selection();
    rig.drag(start + Vec2::new(0.0, 20.0));
    assert_eq!(rig.selection(), inside, "no scroll while inside");
    // 40 px below the area: a little more each tick, not the end at once.
    let below = Vec2::new(x + w - 4.0, y + h + 40.0);
    let mut lens = Vec::new();
    for _ in 0..6 {
        rig.drag(below);
        lens.push(rig.selection().lines().count());
    }
    assert!(
        lens[0] <= 5,
        "the first tick reaches the edge's line: {lens:?}"
    );
    assert!(lens.windows(2).all(|w| w[1] >= w[0]), "{lens:?}");
    assert!(lens[5] > lens[0], "it keeps scrolling while held: {lens:?}");
    assert!(lens[5] < 20, "line by line, not at once: {lens:?}");
    // Held long enough it reaches the last line and stops there.
    for _ in 0..200 {
        rig.drag(below);
    }
    let all = rig.selection();
    assert!(all.ends_with("line 20"), "{all:?}");
    rig.drag(below);
    assert_eq!(rig.selection(), all, "no scrolling past the end");
    // Back above it: it scrolls back up.
    let above = Vec2::new(x + 4.0, y - 40.0);
    for _ in 0..200 {
        rig.drag(above);
    }
    assert!(rig.selection().is_empty() || rig.selection().starts_with("line 1"));
    rig.release(above);
}

/// The `text-fields-2` reference scene (tests/gfx.rs renders it) is in the
/// state its documentation names: "select" double-clicked, the undone
/// field at "one two ", the area scrolled with a selection over the lines
/// the drag passed.
#[test]
fn the_text_fields_scene_is_what_its_frame_shows() {
    let mut scene = og_render::RenderScene::new();
    let font = add_test_font(&mut scene);
    let (ui, area) = text_fields_uis(&font);
    let target = ui.frame().text_input.clone().expect("the words field");
    assert_eq!(target.selection, "select");
    assert_eq!(ui.frame().element("Undo").expect("undo").text, "one two ");
    let selected = area.frame().text_input.clone().expect("the area").selection;
    assert!(
        selected.contains(" 1 of the notes\nLine 2") && selected.lines().count() >= 5,
        "{selected:?}"
    );
    assert!(
        !selected.contains("Line 9"),
        "scrolled part way: {selected:?}"
    );
    // Built twice, the same frames.
    let (again, area2) = text_fields_uis(&font);
    assert_eq!(again.frame().items, ui.frame().items);
    assert_eq!(area2.frame().items, area.frame().items);
}
