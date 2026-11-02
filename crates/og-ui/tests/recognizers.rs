//! Tap, double tap and long press (docs/ui.md §8.5), explicit focus moves
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

/// What the elements answered on one tick, by id.
#[derive(Default, Debug, Clone, PartialEq)]
struct Seen {
    tapped: Vec<String>,
    double: Vec<String>,
    long: Vec<String>,
    clicked: Vec<String>,
    progress: f32,
}

/// Two boxes side by side: `Tap` (tap only) and `Dbl` (tap and double tap),
/// and under them `Long`, a button with a long press inside a card `Card`
/// whose own spec is `card`.
fn screen(ui: &mut Ui, card: Option<GestureSpec>, inner: GestureSpec, seen: &mut Seen) {
    let note = |seen: &mut Seen, id: &str, r: Response| {
        if r.tapped {
            seen.tapped.push(id.into());
        }
        if r.double_tapped {
            seen.double.push(id.into());
        }
        if r.long_pressed {
            seen.long.push(id.into());
        }
        if r.clicked {
            seen.clicked.push(id.into());
        }
    };
    ui.open("Row", Element::row());
    for (id, spec) in [
        (
            "Tap",
            GestureSpec {
                tap: true,
                ..GestureSpec::default()
            },
        ),
        (
            "Dbl",
            GestureSpec {
                tap: true,
                double_tap: true,
                ..GestureSpec::default()
            },
        ),
    ] {
        ui.open_with(
            id,
            Element {
                interactive: true,
                ..fixed(100.0, 100.0)
            },
            Props {
                gestures: Some(spec),
                ..Props::default()
            },
        );
        let r = ui.close();
        note(seen, id, r);
    }
    ui.close();
    ui.open_with(
        "Card",
        Element {
            padding: Insets::all(20.0),
            ..Element::default()
        },
        Props {
            gestures: card,
            ..Props::default()
        },
    );
    ui.open_with(
        "Long",
        Element {
            interactive: true,
            focusable: true,
            ..fixed(100.0, 100.0)
        },
        Props {
            gestures: Some(inner),
            ..Props::default()
        },
    );
    let r = ui.close();
    note(seen, "Long", r);
    seen.progress = r.long_press;
    ui.close();
    let card_info = ui.gesture_at("Card");
    if card_info.long_pressed {
        seen.long.push("Card".into());
    }
    if card_info.tapped {
        seen.tapped.push("Card".into());
    }
}

struct Rig {
    ui: Ui,
    card: Option<GestureSpec>,
    inner: GestureSpec,
}

impl Rig {
    fn new(card: Option<GestureSpec>, inner: GestureSpec) -> Rig {
        let mut r = Rig {
            ui: Ui::new(UiDesc::default()),
            card,
            inner,
        };
        r.step(InputFrame::default());
        r
    }

    fn step(&mut self, input: InputFrame) -> Seen {
        let mut seen = Seen::default();
        self.ui.begin(input, vp());
        screen(&mut self.ui, self.card, self.inner, &mut seen);
        self.ui.end();
        seen
    }

    fn center(&self, path: &str) -> Vec2 {
        let (x, y) = self.ui.frame().element(path).expect(path).center();
        Vec2::new(x, y)
    }
}

fn long_spec() -> GestureSpec {
    GestureSpec {
        long_press: true,
        tap: true,
        ..GestureSpec::default()
    }
}

#[test]
fn a_tap_answers_on_release_and_waits_for_a_double_only_where_one_is_declared() {
    let mut rig = Rig::new(None, long_spec());
    let tap = rig.center("Row/Tap");
    rig.step(press(tap));
    let up = rig.step(release(tap));
    assert_eq!(
        up.tapped,
        ["Tap"],
        "a tap-only element answers on the release"
    );
    assert_eq!(up.clicked, ["Tap"]);

    // The double-tap element: nothing on the release, a tap once its window
    // (300 ms: 18 ticks at 60 Hz) has passed with no second tap.
    let dbl = rig.center("Row/Dbl");
    rig.step(press(dbl));
    let up = rig.step(release(dbl));
    assert!(up.tapped.is_empty(), "it waits: {up:?}");
    assert_eq!(up.clicked, ["Dbl"], "the click does not wait");
    let mut when = None;
    for k in 1..=30 {
        let s = rig.step(at(dbl));
        if !s.tapped.is_empty() {
            assert_eq!(s.tapped, ["Dbl"]);
            assert!(when.is_none(), "one tap");
            when = Some(k);
        }
    }
    assert_eq!(when, Some(19), "the tick after the 18-tick window");

    // Two taps within the window: a double tap, and no tap at all.
    let mut all = Seen::default();
    for input in [press(dbl), release(dbl), at(dbl), press(dbl), release(dbl)] {
        let s = rig.step(input);
        all.tapped.extend(s.tapped);
        all.double.extend(s.double);
    }
    for _ in 0..30 {
        let s = rig.step(at(dbl));
        all.tapped.extend(s.tapped);
        all.double.extend(s.double);
    }
    assert_eq!(all.double, ["Dbl"]);
    assert!(
        all.tapped.is_empty(),
        "a double tap is not also a tap: {all:?}"
    );
}

#[test]
fn a_long_press_fires_at_its_time_once_and_takes_the_press() {
    let mut rig = Rig::new(None, long_spec());
    let p = rig.center("Card/Long");
    let mut fired = Vec::new();
    let s = rig.step(press(p));
    assert!(s.long.is_empty());
    let mut half = 0.0;
    // Held 500 ms: 30 ticks after the press's.
    for k in 1..=40 {
        let s = rig.step(hold(p));
        if k == 15 {
            half = s.progress;
        }
        if !s.long.is_empty() {
            fired.push((k, s.long.clone()));
        }
    }
    assert_eq!(fired, [(30, vec!["Long".to_string()])]);
    assert!((half - 0.5).abs() < 0.01, "progress halfway: {half}");
    let up = rig.step(release(p));
    assert!(
        up.clicked.is_empty() && up.tapped.is_empty(),
        "the long press took the press: {up:?}"
    );

    // Travel past the slop (10 px) fails it, and the release taps nothing.
    rig.step(press(p));
    rig.step(hold(p + Vec2::new(12.0, 0.0)));
    let mut any = Seen::default();
    for _ in 0..40 {
        let s = rig.step(hold(p));
        any.long.extend(s.long);
    }
    let up = rig.step(release(p));
    assert!(
        any.long.is_empty() && up.tapped.is_empty(),
        "{any:?} {up:?}"
    );
    assert_eq!(
        up.clicked,
        ["Long"],
        "the click is a press and release inside it"
    );

    // A short press is a tap.
    rig.step(press(p));
    let up = rig.step(release(p));
    assert_eq!(up.tapped, ["Long"]);
}

#[test]
fn nested_recognisers_go_to_the_higher_priority_then_the_innermost() {
    let card = |priority: i32| GestureSpec {
        long_press: true,
        tap: true,
        priority,
        ..GestureSpec::default()
    };
    // Equal priorities: the button inside wins both.
    let mut rig = Rig::new(Some(card(0)), long_spec());
    let p = rig.center("Card/Long");
    rig.step(press(p));
    let up = rig.step(release(p));
    assert_eq!(up.tapped, ["Long"], "the innermost of equals");
    rig.step(press(p));
    let mut long = Vec::new();
    for _ in 0..35 {
        long.extend(rig.step(hold(p)).long);
    }
    assert_eq!(long, ["Long"]);
    rig.step(release(p));

    // The card ranked higher: it takes the press inside the button.
    let mut rig = Rig::new(Some(card(1)), long_spec());
    rig.step(press(p));
    let up = rig.step(release(p));
    assert_eq!(up.tapped, ["Card"]);
    rig.step(press(p));
    let mut long = Vec::new();
    for _ in 0..35 {
        long.extend(rig.step(hold(p)).long);
    }
    assert_eq!(long, ["Card"]);
    let up = rig.step(release(p));
    assert!(
        up.clicked.is_empty(),
        "the card's long press took the button's press"
    );

    // A recogniser of one gesture leaves the others to the innermost: the
    // card long-presses only, the button still taps.
    let mut rig = Rig::new(
        Some(GestureSpec {
            long_press: true,
            priority: 5,
            ..GestureSpec::default()
        }),
        GestureSpec {
            tap: true,
            ..GestureSpec::default()
        },
    );
    rig.step(press(p));
    let up = rig.step(release(p));
    assert_eq!(up.tapped, ["Long"]);
}

#[test]
fn keys_tap_a_focused_recogniser_and_a_waiting_tap_survives_a_snapshot() {
    let mut rig = Rig::new(None, long_spec());
    rig.step(keys(&[UiKey::Tab]));
    rig.step(InputFrame::default());
    let s = rig.step(keys(&[UiKey::Enter]));
    assert_eq!(s.tapped, ["Long"], "Enter taps the focused control");

    // A tap waiting for its double, saved and restored mid-wait.
    let dbl = rig.center("Row/Dbl");
    rig.step(press(dbl));
    rig.step(release(dbl));
    for _ in 0..5 {
        rig.step(at(dbl));
    }
    let saved = rig.ui.save_state();
    let mut copy = Rig {
        ui: Ui::new(UiDesc::default()),
        card: None,
        inner: long_spec(),
    };
    copy.ui.restore_state(&saved).expect("restores");
    let (mut a, mut b) = (None, None);
    for k in 0..20 {
        if !rig.step(at(dbl)).tapped.is_empty() {
            a.get_or_insert(k);
        }
        if !copy.step(at(dbl)).tapped.is_empty() {
            b.get_or_insert(k);
        }
    }
    assert!(a.is_some());
    assert_eq!(a, b, "the restored copy taps on the same tick");
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
fn without_a_group_focus_wraps_as_before_and_explicit_moves_win() {
    let mut f = Focus::new(None, NavLinks::default());
    assert_eq!(f.go(UiKey::Tab), "T0");
    assert_eq!(
        f.go(UiKey::Left),
        "Side",
        "today's rule: wraps to the row's far end"
    );
    let mut f = Focus::new(
        None,
        NavLinks {
            right: "Below".into(),
            left: "Top/Tools".into(),
            next: "Top/Side".into(),
            ..NavLinks::default()
        },
    );
    assert_eq!(f.go(UiKey::Tab), "T0");
    assert_eq!(f.go(UiKey::Right), "Below", "the explicit move, not T1");
    assert_eq!(f.go(UiKey::Up), "T0");
    assert_eq!(f.go(UiKey::Tab), "Side", "Tab follows next");
    assert_eq!(f.go(UiKey::Tab), "Below", "from Side, reading order");
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
