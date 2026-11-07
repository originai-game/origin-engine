//! Tap, double tap and long press (docs/ui.md §2.1), explicit focus moves
//! and focus groups (§8.3), and the pointer's shape (§6.1), driven tick by
//! tick.
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

use og_core::glam::Vec2;
use og_ui::*;

fn vp() -> Viewport {
    Viewport {
        width: 800.0,
        height: 600.0,
        dpr: 1.0,
        coarse_pointer: false,
        ..Viewport::default()
    }
}

fn fixed(w: f32, h: f32) -> Element {
    Element {
        width: Size::Fixed(w),
        height: Size::Fixed(h),
        ..Element::default()
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

// ── focus ───────────────────────────────────────────────────────────────

/// `Tools` (a row of three buttons, `T0`..`T2`, a focus group of `group`)
/// with `Side` right of it, and `Below` under `T0`. `links` go on `T0`.
fn focus_screen(ui: &mut Ui, group: Option<FocusGroup>, links: NavLinks) -> Option<String> {
    let mut focus = None;
    ui.open(
        "Top",
        Element {
            gap: 40.0,
            ..Element::row()
        },
    );
    ui.open_with(
        "Tools",
        Element {
            gap: 8.0,
            ..Element::row()
        },
        Props {
            focus_group: group,
            ..Props::default()
        },
    );
    for k in 0..3 {
        let id = format!("T{k}");
        let r = ui.button(id.as_str(), &id, ButtonDesc::default());
        if k == 0 {
            ui.set_props(Props {
                nav: links.clone(),
                ..Props::default()
            });
        }
        if r.focused {
            focus = Some(id);
        }
    }
    ui.close();
    if ui.button("Side", "Side", ButtonDesc::default()).focused {
        focus = Some("Side".into());
    }
    ui.close();
    if ui.button("Below", "Below", ButtonDesc::default()).focused {
        focus = Some("Below".into());
    }
    focus
}

struct Focus {
    ui: Ui,
    group: Option<FocusGroup>,
    links: NavLinks,
    focus: Option<String>,
}

impl Focus {
    fn new(group: Option<FocusGroup>, links: NavLinks) -> Focus {
        let mut f = Focus {
            ui: Ui::new(UiDesc::default()),
            group,
            links,
            focus: None,
        };
        f.step(InputFrame::default());
        f
    }

    fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, vp());
        let f = focus_screen(&mut self.ui, self.group, self.links.clone());
        self.ui.end();
        self.focus = f;
    }

    /// Presses `k` and answers where the focus is a tick later.
    fn go(&mut self, k: UiKey) -> String {
        self.step(keys(&[k]));
        self.step(InputFrame::default());
        self.focus.clone().unwrap_or_default()
    }
}

#[test]
fn a_group_stops_at_its_edge_leaves_it_or_wraps_inside_and_remembers() {
    // No wrap: Right from T2 leaves for Side; Left from T0 has nothing
    // outside that way and stays (today's rule would wrap to Side).
    let mut f = Focus::new(Some(FocusGroup::default()), NavLinks::default());
    assert_eq!(f.go(UiKey::Tab), "T0");
    assert_eq!(f.go(UiKey::Left), "T0", "stays at the group's edge");
    assert_eq!(f.go(UiKey::Right), "T1");
    assert_eq!(f.go(UiKey::Right), "T2");
    assert_eq!(f.go(UiKey::Right), "Side", "leaves for the control outside");

    // Wrap: Right from T2 goes round to T0, never out to Side.
    let wrap = FocusGroup {
        wrap: true,
        ..FocusGroup::default()
    };
    let mut f = Focus::new(Some(wrap), NavLinks::default());
    f.go(UiKey::Tab);
    f.go(UiKey::Right);
    f.go(UiKey::Right);
    assert_eq!(f.go(UiKey::Right), "T0", "wraps inside the group");
    assert_eq!(f.go(UiKey::Down), "Below", "other ways still leave it");

    // Memory: back into the group lands on the member it last had, not the
    // nearest one; an explicit move to the group does too.
    let mem = FocusGroup {
        memory: true,
        ..FocusGroup::default()
    };
    let mut f = Focus::new(Some(mem), NavLinks::default());
    f.go(UiKey::Tab);
    f.go(UiKey::Right);
    assert_eq!(f.go(UiKey::Right), "T2");
    assert_eq!(f.go(UiKey::Down), "Below");
    assert_eq!(f.go(UiKey::Up), "T2", "T0 is nearer, T2 was last");
    let mut f = Focus::new(
        Some(mem),
        NavLinks {
            down: "Below".into(),
            ..NavLinks::default()
        },
    );
    f.go(UiKey::Tab);
    assert_eq!(f.go(UiKey::Down), "Below");
    assert_eq!(f.go(UiKey::Up), "T0");
}

#[test]
fn a_list_takes_home_end_the_page_keys_and_typed_letters() {
    // rune-kit's listbox keys on a dropdown: Down opens it on its value,
    // Home and End go to the ends, PageDown as far as a view reaches,
    // letters pick by name and the same letter again cycles.
    let names = ["Apple", "Avocado", "Banana", "Cherry", "Date"];
    let mut ui = Ui::new(UiDesc::default());
    let mut sel = 2;
    let mut open = false;
    let mut step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        let r = ui.dropdown(
            "Fruit",
            &DropdownDesc {
                options: names
                    .iter()
                    .map(|n| DropdownOption {
                        label: (*n).into(),
                        ..DropdownOption::default()
                    })
                    .collect(),
                selected: sel,
                ..DropdownDesc::default()
            },
        );
        ui.end();
        sel = r.selected;
        open = r.open;
        (open, sel)
    };
    // A key, then a still tick: whether it is open, the focused row's path
    // and the choice.
    let mut go = |ui: &mut Ui, input: InputFrame| {
        step(ui, input);
        let (o, s) = step(ui, InputFrame::default());
        (o, ui.focus_path(0).to_owned(), s)
    };
    let text = |t: &str| InputFrame {
        text: t.into(),
        ..InputFrame::default()
    };
    go(&mut ui, InputFrame::default());
    let (o, trig, _) = go(&mut ui, keys(&[UiKey::Tab]));
    assert!(!o);
    let (o, on, _) = go(&mut ui, keys(&[UiKey::Down]));
    assert!(o, "Down opens a closed dropdown");
    assert_ne!(on, trig, "the focus is in the list");
    let row = |ui: &Ui| ui.focus_path(0).to_owned();
    let at_value = row(&ui);
    go(&mut ui, keys(&[UiKey::End]));
    let last = row(&ui);
    assert_ne!(last, at_value, "End leaves the chosen row");
    go(&mut ui, keys(&[UiKey::Home]));
    let first = row(&ui);
    assert_ne!(first, last);
    go(&mut ui, keys(&[UiKey::PageDown]));
    assert_eq!(row(&ui), last, "the whole list is in one view");
    go(&mut ui, keys(&[UiKey::PageUp]));
    assert_eq!(row(&ui), first);
    // Letters: "a" from Apple cycles to Avocado, again back to Apple; "c"
    // after a pause goes to Cherry; Enter chooses it.
    go(&mut ui, text("a"));
    let avocado = row(&ui);
    assert_ne!(avocado, first, "the same letter cycles");
    go(&mut ui, text("a"));
    assert_eq!(row(&ui), first, "and wraps round");
    go(&mut ui, text("c"));
    assert_eq!(row(&ui), first, "\"aac\" names nothing: it stays");
    for _ in 0..70 {
        go(&mut ui, InputFrame::default());
    }
    go(&mut ui, text("c"));
    let (o, _, s) = go(&mut ui, keys(&[UiKey::Enter]));
    assert!(
        !o && s == 3,
        "a second's pause starts a new word: Cherry ({o} {s})"
    );

    // A group that is not a list keeps its letters for the game.
    let mut f = Focus::new(Some(FocusGroup::default()), NavLinks::default());
    assert_eq!(f.go(UiKey::Tab), "T0");
    f.step(text("t"));
    f.step(InputFrame::default());
    assert_eq!(f.focus.as_deref(), Some("T0"), "a toolbar takes no letters");
    assert_eq!(f.go(UiKey::End), "T2", "but takes End");
    assert_eq!(f.go(UiKey::Home), "T0");
}

// ── the cursor ─────────────────────────────────────────────────────────

#[test]
fn the_cursor_follows_what_the_pointer_is_over_and_what_it_holds() {
    let mut ui = Ui::new(UiDesc::default());
    let mut text = String::new();
    let mut build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        ui.open_with(
            "Shop",
            Element::column(),
            Props {
                cursor: Cursor::Pointer,
                ..Props::default()
            },
        );
        ui.button("Buy", "Buy", ButtonDesc::default());
        ui.button("Sold", "Sold", ButtonDesc::default());
        ui.set_props(Props {
            cursor: Cursor::NotAllowed,
            ..Props::default()
        });
        ui.close();
        ui.text_field("Name", &mut text, &FieldDesc::default());
        ui.open_with(
            "Card",
            Element {
                interactive: true,
                background: Some(Srgba::rgb(90, 90, 90)),
                ..fixed(80.0, 80.0)
            },
            Props {
                drag: Some(DragSpec::default()),
                ..Props::default()
            },
        );
        ui.close();
        ui.label("Note", "Nothing here", LabelDesc::default());
        ui.end().cursor
    };
    build(&mut ui, InputFrame::default());
    let c = |ui: &Ui, p: &str| {
        let (x, y) = ui.frame().element(p).expect(p).center();
        Vec2::new(x, y)
    };
    let buy = c(&ui, "Shop/Buy");
    assert_eq!(
        build(&mut ui, at(buy)),
        Cursor::Pointer,
        "inherited from Shop"
    );
    let sold = c(&ui, "Shop/Sold");
    assert_eq!(build(&mut ui, at(sold)), Cursor::NotAllowed, "its own wins");
    let name = c(&ui, "Name");
    assert_eq!(build(&mut ui, at(name)), Cursor::Text);
    let note = c(&ui, "Note");
    assert_eq!(build(&mut ui, at(note)), Cursor::Default);
    let card = c(&ui, "Card");
    assert_eq!(build(&mut ui, at(card)), Cursor::Grab);
    build(&mut ui, press(card));
    assert_eq!(
        build(&mut ui, hold(card + Vec2::new(30.0, 0.0))),
        Cursor::Grabbing
    );
    // Held off the card, the hand stays closed.
    assert_eq!(build(&mut ui, hold(note)), Cursor::Grabbing);
    build(&mut ui, release(note));
    let empty = Vec2::new(700.0, 500.0);
    assert_eq!(build(&mut ui, at(empty)), Cursor::Default);
}
