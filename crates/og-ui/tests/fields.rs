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
//! - words by character classes again (a CJK run one word, an apostrophe
//!   ending a word, a Thai run one word), a double click on a Han character
//!   taking its neighbours, IME commits of Han text undone all at once →
//!   `words_are_uax_29_words`
//! - Command taken for Control (or the other way round) under mac keys:
//!   Cmd+←/→ moving by words or characters, Option+← by characters, Ctrl+A
//!   selecting all, Cmd+Backspace one character, Ctrl+Z undoing; or Command
//!   moving by words on other platforms →
//!   `mac_keys_tell_command_from_control`
//! - a drag below a text area jumps to the end (or does not scroll), keeps
//!   scrolling past the end, or scrolls while the pointer is inside →
//!   `a_drag_past_a_text_area_scrolls_it_line_by_line`
//! - the caret, Delete or Backspace stepping into a grapheme cluster (a
//!   base and its combining mark, an emoji and its skin tone, a family
//!   joined by ZWJ), a press landing between a base and its mark →
//!   `the_caret_and_backspace_step_by_grapheme_clusters`
//! - ← moving back in the text in a right-to-left run (by characters or
//!   with Alt by words), a selection collapsing to the wrong side there →
//!   `arrows_move_as_they_point_in_right_to_left_runs`
//! - the emacs keys beyond Control+A and E doing nothing under mac keys
//!   (Ctrl+F, B, N, P, D, H, K, U, W), or acting off a Mac →
//!   `mac_keys_honour_the_emacs_keys`
//! - Control+K, U and W deleting what they take instead of killing it,
//!   Control+Y yanking nothing, kills in a row replacing each other (or a
//!   move between not ending the chain), a line break yanked into a field,
//!   the buffer lost by a save and restore, Control+T not transposing →
//!   `mac_keys_keep_an_emacs_kill_buffer`
//! - ← and → stopping at a bidi run's edge, stepping back and forth across
//!   it, or forgetting which of its two places the caret took →
//!   `arrows_move_visually_across_a_bidi_edge`
//! - a field held as wide as its placeholder (a chat input pushing Send off
//!   a phone at large text), or its placeholder cut with no ellipsis →
//!   `a_field_shrinks_in_its_row_and_cuts_its_placeholder_with_an_ellipsis`

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
        let mut rig = Rig {
            ui: ui_with_font(test_font()),
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
        let e = self.ui.element(id).expect(id);
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

    /// Keys with Command (`meta`) apart from Control.
    fn chord(&mut self, keys: &[UiKey], shift: bool, ctrl: bool, meta: bool, alt: bool) {
        self.step(InputFrame {
            keys: keys.to_vec(),
            shift,
            ctrl,
            meta,
            alt,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    fn typed(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
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

    fn focus_field(&mut self) {
        let (x, y, w, h) = self.rect("Name");
        self.click(Vec2::new(x + w - 2.0, y + h / 2.0));
        // Past the double-click window, so the next press is a first click.
        self.idle(40);
    }
}

#[test]
fn mac_keys_tell_command_from_control() {
    let mut rig = Rig::new("hello brave, new world", "");
    rig.vp.mac_keys = true;
    rig.focus_field();
    // Command+← and →: the line's ends.
    rig.chord(&[UiKey::Left], false, false, true, false);
    assert_eq!(rig.caret_marked(), "|hello brave, new world");
    rig.chord(&[UiKey::Right], false, false, true, false);
    assert_eq!(rig.caret_marked(), "hello brave, new world|");
    // Option+←: by words; Control+← one character (no word move on a Mac).
    rig.chord(&[UiKey::Left], false, false, false, true);
    assert_eq!(rig.caret_marked(), "hello brave, new |world");
    rig.chord(&[UiKey::Left], false, true, false, false);
    assert_eq!(rig.caret_marked(), "hello brave, new| world");
    // Shift+Command+←: selects to the line's start.
    rig.chord(&[UiKey::Left], true, false, true, false);
    assert_eq!(rig.selection(), "hello brave, new");
    // Control+A and E: the line's ends (emacs), not select all.
    rig.chord(&[UiKey::KeyE], false, true, false, false);
    assert_eq!(rig.selection(), "");
    assert_eq!(rig.caret_marked(), "hello brave, new world|");
    rig.chord(&[UiKey::KeyA], false, true, false, false);
    assert_eq!(rig.selection(), "", "Control+A is no select all on a Mac");
    assert_eq!(rig.caret_marked(), "|hello brave, new world");
    // Command+A selects all; Command+→ then Command+Backspace deletes to
    // the line's start; Control+Z undoes nothing, Command+Z the delete.
    rig.chord(&[UiKey::KeyA], false, false, true, false);
    assert_eq!(rig.selection(), "hello brave, new world");
    rig.chord(&[UiKey::Right], false, false, true, false);
    rig.chord(&[UiKey::Left, UiKey::Left], false, false, false, true);
    rig.chord(&[UiKey::Backspace], false, false, true, false);
    assert_eq!(rig.value, "new world");
    rig.chord(&[UiKey::KeyZ], false, true, false, false);
    assert_eq!(rig.value, "new world", "Control+Z is no undo on a Mac");
    rig.chord(&[UiKey::KeyZ], false, false, true, false);
    assert_eq!(rig.value, "hello brave, new world");
    // Command+↑ and ↓ in an area: the text's ends; Command+← its line's start.
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(Vec2::new(x + w / 2.0, y + 6.0));
    rig.idle(40);
    rig.typed("one\ntwo\nthree");
    rig.chord(&[UiKey::Up], false, false, true, false);
    rig.chord(&[UiKey::Down], true, false, true, false);
    assert_eq!(rig.selection(), "one\ntwo\nthree");
    rig.chord(&[UiKey::Left], false, false, true, false);
    rig.chord(&[UiKey::Right], true, false, false, true);
    assert_eq!(
        rig.selection(),
        "three",
        "the last line's start, then a word"
    );

    // Other platforms: Command moves one character and still selects all;
    // Control moves by words.
    let mut rig = Rig::new("hello brave, new world", "");
    rig.focus_field();
    rig.chord(&[UiKey::Left], false, false, true, false);
    assert_eq!(rig.caret_marked(), "hello brave, new worl|d");
    rig.chord(&[UiKey::Left], false, true, false, false);
    assert_eq!(rig.caret_marked(), "hello brave, new |world");
    rig.chord(&[UiKey::KeyA], false, false, true, false);
    assert_eq!(rig.selection(), "hello brave, new world");
    rig.chord(&[UiKey::KeyE], false, true, false, false);
    assert_eq!(
        rig.selection(),
        "hello brave, new world",
        "Ctrl+E is nothing"
    );
}

#[test]
fn mac_keys_keep_an_emacs_kill_buffer() {
    let ctrl = |rig: &mut Rig, k: UiKey| rig.chord(&[k], false, true, false, false);
    let option_right = |rig: &mut Rig| rig.chord(&[UiKey::Right], false, false, false, true);
    let mut rig = Rig::new("hello brave new world", "");
    rig.vp.mac_keys = true;
    rig.focus_field();
    // K kills to the end; Y yanks it back, at the caret, as often as asked.
    ctrl(&mut rig, UiKey::KeyA);
    option_right(&mut rig);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.value, "hello");
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.caret_marked(), " brave new world|hello");
    ctrl(&mut rig, UiKey::KeyE);
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value, " brave new worldhello brave new world");
    // A yank is one undo step.
    rig.chord(&[UiKey::KeyZ], false, false, true, false);
    assert_eq!(rig.value, " brave new worldhello");
    // W in a row: each word before the last, the buffer both of them.
    let mut rig = Rig::new("one two three", "");
    rig.vp.mac_keys = true;
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyW);
    ctrl(&mut rig, UiKey::KeyW);
    assert_eq!(rig.value, "one ");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value, "one two three");
    // A move between two kills ends the chain: the second replaces the
    // buffer.
    ctrl(&mut rig, UiKey::KeyA);
    option_right(&mut rig);
    ctrl(&mut rig, UiKey::KeyK);
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.value, "");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value, "one");
    // In an area, K at a line's end kills its break and adds it to the
    // kill before; a field yanks that without the break.
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(Vec2::new(x + w / 2.0, y + 6.0));
    rig.idle(40);
    rig.typed("ab\ncd");
    rig.chord(&[UiKey::Up], false, false, true, false);
    ctrl(&mut rig, UiKey::KeyK);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.area, "cd");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.area, "ab\ncd");
    rig.value = "x".into();
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value, "xab");
    // The buffer is UI state: a save and restore keeps it.
    let saved = rig.ui.save_state();
    let mut other = Rig::new("y", "");
    other.vp.mac_keys = true;
    other.ui.restore_state(&saved).expect("restores");
    other.step(InputFrame::default());
    ctrl(&mut other, UiKey::KeyY);
    assert_eq!(other.value, "yab");
    // T transposes the characters about the caret, the two before it at
    // the end; at the start nothing.
    let mut rig = Rig::new("abcd", "");
    rig.vp.mac_keys = true;
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.value, "abcd");
    ctrl(&mut rig, UiKey::KeyF);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.caret_marked(), "ba|cd");
    ctrl(&mut rig, UiKey::KeyE);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.caret_marked(), "badc|");
}
