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
//! - word moves by character again instead of the dictionary's words
//!   (Thai and Chinese, written without spaces, a stop per cluster or none
//!   in the run), ← stepping back in the text in a right-to-left paragraph,
//!   or an arrow skipping a run at a bidi edge instead of stepping through
//!   it from the side it enters → `words_and_arrows_follow_the_layout`
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
//! - a field held as wide as its placeholder (a chat input pushing Send off
//!   a phone at large text), or its placeholder cut with no ellipsis →
//!   `a_field_shrinks_in_its_row_and_cuts_its_placeholder_with_an_ellipsis`

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use serde_json::json;
use support::*;

struct Rig {
    d: Doc,
}

const FORM: &str = r#"<meta name="component" content="t">
<column id="Form">
  <input id="Name" bind:value={$value} />
  <textarea id="Notes" rows="3" bind:value={$area} />
</column>
<style>
#Name { width: 300px; }
#Notes { width: 200px; }
</style>"#;

impl Rig {
    fn new(value: &str, area: &str) -> Rig {
        let mut d = Doc::with_font(
            FORM,
            &[("value", json!(value)), ("area", json!(area))],
            test_font(),
        );
        d.viewport = Viewport::new(800.0, 600.0);
        let mut rig = Rig { d };
        rig.idle(1);
        rig
    }

    /// The field's text: what it wrote back to its store.
    fn value(&mut self) -> String {
        self.d
            .read("value")
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    /// The area's text.
    fn area(&mut self) -> String {
        self.d.read("area").as_str().unwrap_or_default().to_string()
    }

    /// A text the game sets the field to.
    fn set_value(&mut self, value: &str) {
        self.d.set("value", json!(value));
    }

    fn step(&mut self, input: InputFrame) {
        self.d.step(input);
    }

    fn idle(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.step(InputFrame::default());
        }
    }
    fn rect(&self, id: &str) -> (f32, f32, f32, f32) {
        let e = self.d.get(id);
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
        self.d
            .ui
            .frame()
            .text_input
            .clone()
            .expect("a focused text control")
            .selection
    }

    /// The field's caret, found by typing a marker there and taking it out.
    fn caret_marked(&mut self) -> String {
        self.typed("|");
        let marked = self.value();
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
    rig.d.viewport.mac_keys = true;
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
    assert_eq!(rig.value(), "new world");
    rig.chord(&[UiKey::KeyZ], false, true, false, false);
    assert_eq!(rig.value(), "new world", "Control+Z is no undo on a Mac");
    rig.chord(&[UiKey::KeyZ], false, false, true, false);
    assert_eq!(rig.value(), "hello brave, new world");
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
    rig.d.viewport.mac_keys = true;
    rig.focus_field();
    // K kills to the end; Y yanks it back, at the caret, as often as asked.
    ctrl(&mut rig, UiKey::KeyA);
    option_right(&mut rig);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.value(), "hello");
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.caret_marked(), " brave new world|hello");
    ctrl(&mut rig, UiKey::KeyE);
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value(), " brave new worldhello brave new world");
    // A yank is one undo step.
    rig.chord(&[UiKey::KeyZ], false, false, true, false);
    assert_eq!(rig.value(), " brave new worldhello");
    // W in a row: each word before the last, the buffer both of them.
    let mut rig = Rig::new("one two three", "");
    rig.d.viewport.mac_keys = true;
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyW);
    ctrl(&mut rig, UiKey::KeyW);
    assert_eq!(rig.value(), "one ");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value(), "one two three");
    // A move between two kills ends the chain: the second replaces the
    // buffer.
    ctrl(&mut rig, UiKey::KeyA);
    option_right(&mut rig);
    ctrl(&mut rig, UiKey::KeyK);
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.value(), "");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value(), "one");
    // In an area, K at a line's end kills its break and adds it to the
    // kill before; a field yanks that without the break.
    let (x, y, w, _) = rig.rect("Notes");
    rig.click(Vec2::new(x + w / 2.0, y + 6.0));
    rig.idle(40);
    rig.typed("ab\ncd");
    rig.chord(&[UiKey::Up], false, false, true, false);
    ctrl(&mut rig, UiKey::KeyK);
    ctrl(&mut rig, UiKey::KeyK);
    assert_eq!(rig.area(), "cd");
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.area(), "ab\ncd");
    rig.set_value("x");
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyY);
    assert_eq!(rig.value(), "xab");
    // The buffer is UI state: a save and restore keeps it.
    let saved = rig.d.ui.save_state();
    let mut other = Rig::new("y", "");
    other.d.viewport.mac_keys = true;
    other.d.ui.restore_state(&saved).expect("restores");
    // The UI state holds the mount's stores too ("xab"): the game sets
    // its own text again.
    other.set_value("y");
    ctrl(&mut other, UiKey::KeyY);
    assert_eq!(other.value(), "yab");
    // T transposes the characters about the caret, the two before it at
    // the end; at the start nothing.
    let mut rig = Rig::new("abcd", "");
    rig.d.viewport.mac_keys = true;
    rig.focus_field();
    ctrl(&mut rig, UiKey::KeyA);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.value(), "abcd");
    ctrl(&mut rig, UiKey::KeyF);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.caret_marked(), "ba|cd");
    ctrl(&mut rig, UiKey::KeyE);
    ctrl(&mut rig, UiKey::KeyT);
    assert_eq!(rig.caret_marked(), "badc|");
}

/// Word moves stop at dictionary words in Thai and Chinese (the
/// dictionaries the cook ships for those locales), and the arrows move on
/// screen: on in the text in a right-to-left paragraph, and into a
/// right-to-left run from the side the caret enters it. A fixed-advance
/// face has every character, so each script is laid out.
#[test]
fn words_and_arrows_follow_the_layout() {
    for attr in ["thaidict", "cjdict"] {
        let bytes = og_i18n::segmenter::compiled(attr).expect(attr);
        og_text::text::add_segmentation(attr, &bytes).expect(attr);
    }
    let rig = |value: &str| {
        let mut d = Doc::with_font(
            FORM,
            &[("value", json!(value)), ("area", json!(""))],
            UiFont::unrendered(0.5),
        );
        d.viewport = Viewport::new(800.0, 600.0);
        let mut rig = Rig { d };
        rig.idle(1);
        rig.focus_field();
        rig
    };
    // The caret after `n` presses of `key` from the line's start, in one
    // tick (the marker typed to find it would reset its place on screen).
    let after = |rig: &mut Rig, key: UiKey, n: usize, ctrl: bool| {
        let mut keys = vec![UiKey::Home];
        keys.extend(std::iter::repeat_n(key, n));
        rig.keys(&keys, false, ctrl, false);
        rig.caret_marked()
    };
    let mut thai = rig("ภาษาไทยง่ายมาก");
    let stops: Vec<String> = (1..=3)
        .map(|n| after(&mut thai, UiKey::Right, n, true))
        .collect();
    assert_eq!(
        stops,
        ["ภาษา|ไทยง่ายมาก", "ภาษาไทย|ง่ายมาก", "ภาษาไทยง่าย|มาก"]
    );
    let mut chinese = rig("我们喜欢学习中文");
    let stops: Vec<String> = (1..=3)
        .map(|n| after(&mut chinese, UiKey::Right, n, true))
        .collect();
    assert_eq!(
        stops,
        [
            "我们|喜欢学习中文",
            "我们喜欢|学习中文",
            "我们喜欢学习|中文"
        ]
    );
    // A right-to-left paragraph starts at the right: ← goes on in the text,
    // by words and by clusters.
    let mut arabic = rig("مرحبا بالعالم");
    assert_eq!(after(&mut arabic, UiKey::Left, 1, true), "مرحبا| بالعالم");
    assert_eq!(after(&mut arabic, UiKey::Left, 2, false), "مر|حبا بالعالم");
    assert_eq!(after(&mut arabic, UiKey::Right, 1, false), "|مرحبا بالعالم");
    // → into a right-to-left run from its left: through its last letter
    // first, then out of its right edge, its first letter's place.
    let mut mixed = rig("ab مرحبا cd");
    let stops: Vec<String> = (3..=10)
        .map(|n| after(&mut mixed, UiKey::Right, n, false))
        .collect();
    assert_eq!(
        stops,
        [
            "ab |مرحبا cd",
            "ab مرحب|ا cd",
            "ab مرح|با cd",
            "ab مر|حبا cd",
            "ab م|رحبا cd",
            "ab |مرحبا cd",
            "ab مرحبا |cd",
            "ab مرحبا c|d",
        ]
    );
}
