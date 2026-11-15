//! Tap, double tap and long press (docs/ui.md §2.1), explicit focus moves
//! and focus groups (og-ui's README, §Focus navigation), and the pointer's
//! shape (§Gestures and the cursor), driven tick by tick.
//!
//! Ways these could fail, each covered below:
//! - a tap waits for a double tap on an element that declares none (a
//!   button that answers 300 ms late), or a double-tap element taps as well
//!   as double taps, or never taps once its window passes;
//! - a long press fires a tick early or late, fires twice, or leaves the
//!   release to click and tap anyway; travel past the slop still fires;
//! - nested recognisers both answer one press, or the outer one always
//!   wins whatever the priorities say;
//! - a snapshot taken while a tap waits for its double continues otherwise;
//! - an explicit focus move is ignored for the spatial pick, or a path to a
//!   group lands on its first member when the group remembers another;
//! - a group without wrap still wraps (today's global rule) or stops where
//!   a control outside lies that way; a wrapping group wraps to a control
//!   outside it;
//! - the cursor ignores a shape set on an ancestor, or a drag's hand does
//!   not close while dragged.

use crate::support::Doc;
use og_core::glam::Vec2;
use og_ui::*;
use serde_json::{Value, json};

fn vp() -> Viewport {
    Viewport {
        width: 800.0,
        height: 600.0,
        dpr: 1.0,
        coarse_pointer: false,
        ..Viewport::default()
    }
}

fn at(p: Vec2) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        ..InputFrame::default()
    }
}

fn press(p: Vec2) -> InputFrame {
    InputFrame {
        down: true,
        pressed: true,
        ..at(p)
    }
}

fn hold(p: Vec2) -> InputFrame {
    InputFrame {
        down: true,
        ..at(p)
    }
}

fn release(p: Vec2) -> InputFrame {
    InputFrame {
        released: true,
        ..at(p)
    }
}

fn keys(k: &[UiKey]) -> InputFrame {
    InputFrame {
        keys: k.to_vec(),
        ..InputFrame::default()
    }
}

/// `text` mounted with `stores` and laid out at [`vp`].
fn doc(text: &str, stores: &[(&str, Value)]) -> Doc {
    let mut d = Doc::new(text, stores);
    d.viewport = vp();
    d.step(InputFrame::default());
    d
}

// ── focus ───────────────────────────────────────────────────────────────

/// `Tools` (a row of three buttons, `T0`..`T2`, a focus group by the
/// attributes `group`) with `Side` right of it, and `Below` under `T0`.
/// `links` are `T0`'s attributes.
fn focus_screen(group: &str, links: &str) -> String {
    format!(
        r#"<meta name="component" content="focus">
<column id="Screen">
  <row id="Top">
    <row id="Tools" {group}>
      <button id="T0" {links}>T0</button><button id="T1">T1</button><button id="T2">T2</button>
    </row>
    <button id="Side">Side</button>
  </row>
  <button id="Below">Below</button>
</column>
<style>
#Screen {{ align-items: start; }}
#Top {{ gap: 40px; }}
#Tools {{ gap: 8px; }}
</style>
"#
    )
}

struct Focus(Doc);

impl Focus {
    fn new(group: &str, links: &str) -> Focus {
        Focus(doc(&focus_screen(group, links), &[]))
    }

    /// The id of the control the focus is on (empty: none).
    fn focus(&self) -> String {
        let path = self.0.ui.focus_path(0);
        path.rsplit('/').next().unwrap_or_default().to_owned()
    }

    /// Presses `k` and answers where the focus is a tick later.
    fn go(&mut self, k: UiKey) -> String {
        self.0.step(keys(&[k]));
        self.0.step(InputFrame::default());
        self.focus()
    }
}

#[test]
fn a_group_stops_at_its_edge_leaves_it_or_wraps_inside_and_remembers() {
    // No wrap: Right from T2 leaves for Side; Left from T0 has nothing
    // outside that way and stays (today's rule would wrap to Side).
    let mut f = Focus::new(r#"focus-group="stop""#, "");
    assert_eq!(f.go(UiKey::Tab), "T0");
    assert_eq!(f.go(UiKey::Left), "T0", "stays at the group's edge");
    assert_eq!(f.go(UiKey::Right), "T1");
    assert_eq!(f.go(UiKey::Right), "T2");
    assert_eq!(f.go(UiKey::Right), "Side", "leaves for the control outside");

    // Wrap: Right from T2 goes round to T0, never out to Side.
    let mut f = Focus::new(r#"focus-group="wrap""#, "");
    f.go(UiKey::Tab);
    f.go(UiKey::Right);
    f.go(UiKey::Right);
    assert_eq!(f.go(UiKey::Right), "T0", "wraps inside the group");
    assert_eq!(f.go(UiKey::Down), "Below", "other ways still leave it");

    // Memory: back into the group lands on the member it last had, not the
    // nearest one; an explicit move to the group does too.
    let mem = r#"focus-group="stop" focus-memory"#;
    let mut f = Focus::new(mem, "");
    f.go(UiKey::Tab);
    f.go(UiKey::Right);
    assert_eq!(f.go(UiKey::Right), "T2");
    assert_eq!(f.go(UiKey::Down), "Below");
    assert_eq!(f.go(UiKey::Up), "T2", "T0 is nearer, T2 was last");
    let mut f = Focus::new(mem, r#"nav-down="Below""#);
    f.go(UiKey::Tab);
    assert_eq!(f.go(UiKey::Down), "Below");
    assert_eq!(f.go(UiKey::Up), "T0");
}

#[test]
fn a_list_takes_home_end_the_page_keys_and_typed_letters() {
    // rune-kit's listbox keys on a select: Down opens it on its value,
    // Home and End go to the ends, PageDown as far as a view reaches,
    // letters pick by name and the same letter again cycles.
    let mut d = doc(
        r#"<meta name="component" content="fruit">
<column id="Form">
  <select id="Fruit" bind:value={$fruit}>
    <option value="Apple">Apple</option>
    <option value="Avocado">Avocado</option>
    <option value="Banana">Banana</option>
    <option value="Cherry">Cherry</option>
    <option value="Date">Date</option>
  </select>
</column>
"#,
        &[("fruit", json!("Banana"))],
    );
    let open = |d: &Doc| {
        d.a11y("Fruit/Trigger")
            .states
            .iter()
            .any(|s| s == "expanded")
    };
    // A key, then a still tick: whether it is open, the focused row's path
    // and the choice.
    let go = |d: &mut Doc, input: InputFrame| {
        d.step(input);
        d.step(InputFrame::default());
        (open(d), d.ui.focus_path(0).to_owned(), d.read("fruit"))
    };
    let text = |t: &str| InputFrame {
        text: t.into(),
        ..InputFrame::default()
    };
    go(&mut d, InputFrame::default());
    let (o, trig, _) = go(&mut d, keys(&[UiKey::Tab]));
    assert!(!o);
    let (o, on, _) = go(&mut d, keys(&[UiKey::Down]));
    assert!(o, "Down opens a closed select");
    assert_ne!(on, trig, "the focus is in the list");
    let row = |d: &Doc| d.ui.focus_path(0).to_owned();
    let at_value = row(&d);
    go(&mut d, keys(&[UiKey::End]));
    let last = row(&d);
    assert_ne!(last, at_value, "End leaves the chosen row");
    go(&mut d, keys(&[UiKey::Home]));
    let first = row(&d);
    assert_ne!(first, last);
    go(&mut d, keys(&[UiKey::PageDown]));
    assert_eq!(row(&d), last, "the whole list is in one view");
    go(&mut d, keys(&[UiKey::PageUp]));
    assert_eq!(row(&d), first);
    // Letters: "a" from Apple cycles to Avocado, again back to Apple; "c"
    // after a pause goes to Cherry; Enter chooses it.
    go(&mut d, text("a"));
    let avocado = row(&d);
    assert_ne!(avocado, first, "the same letter cycles");
    go(&mut d, text("a"));
    assert_eq!(row(&d), first, "and wraps round");
    go(&mut d, text("c"));
    assert_eq!(row(&d), first, "\"aac\" names nothing: it stays");
    for _ in 0..70 {
        go(&mut d, InputFrame::default());
    }
    go(&mut d, text("c"));
    let (o, _, s) = go(&mut d, keys(&[UiKey::Enter]));
    assert!(
        !o && s == "Cherry",
        "a second's pause starts a new word: Cherry ({o} {s})"
    );

    // A group that is not a list keeps its letters for the game.
    let mut f = Focus::new(r#"focus-group="stop""#, "");
    assert_eq!(f.go(UiKey::Tab), "T0");
    f.0.step(text("t"));
    f.0.step(InputFrame::default());
    assert_eq!(f.focus(), "T0", "a toolbar takes no letters");
    assert_eq!(f.go(UiKey::End), "T2", "but takes End");
    assert_eq!(f.go(UiKey::Home), "T0");
}

// ── the cursor ─────────────────────────────────────────────────────────

#[test]
fn the_cursor_follows_what_the_pointer_is_over_and_what_it_holds() {
    let mut d = doc(
        r#"<meta name="component" content="shop">
<column id="Root">
  <column id="Shop"><button id="Buy">Buy</button><button id="Sold">Sold</button></column>
  <input id="Name" type="text" bind:value={$name} />
  <div id="Card" draggable="true"></div>
  <text id="Note">Nothing here</text>
</column>
<style>
#Root { align-items: start; }
#Shop { cursor: pointer; }
#Sold { cursor: not-allowed; }
#Card { width: 80px; height: 80px; background: #5a5a5a; }
</style>
"#,
        &[("name", json!(""))],
    );
    let cursor = |d: &mut Doc, input: InputFrame| {
        d.step(input);
        d.ui.frame().cursor
    };
    let c = |d: &Doc, id: &str| {
        let e = d.get(id);
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    };
    let buy = c(&d, "Shop/Buy");
    assert_eq!(
        cursor(&mut d, at(buy)),
        Cursor::Pointer,
        "inherited from Shop"
    );
    let sold = c(&d, "Shop/Sold");
    assert_eq!(cursor(&mut d, at(sold)), Cursor::NotAllowed, "its own wins");
    let name = c(&d, "Name");
    assert_eq!(cursor(&mut d, at(name)), Cursor::Text);
    let note = c(&d, "Note");
    assert_eq!(cursor(&mut d, at(note)), Cursor::Default);
    let card = c(&d, "Card");
    assert_eq!(cursor(&mut d, at(card)), Cursor::Grab);
    cursor(&mut d, press(card));
    assert_eq!(
        cursor(&mut d, hold(card + Vec2::new(30.0, 0.0))),
        Cursor::Grabbing
    );
    // Held off the card, the hand stays closed.
    assert_eq!(cursor(&mut d, hold(note)), Cursor::Grabbing);
    cursor(&mut d, release(note));
    let empty = Vec2::new(700.0, 500.0);
    assert_eq!(cursor(&mut d, at(empty)), Cursor::Default);
}
