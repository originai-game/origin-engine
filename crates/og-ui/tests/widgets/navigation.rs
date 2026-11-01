use super::*;

/// A 3×3 grid of buttons; records which one reports focus and which clicked.
#[derive(Default)]
struct Grid {
    focus: Option<String>,
    clicks: Vec<String>,
}

fn grid(ui: &mut Ui, s: &mut Grid) {
    s.focus = None;
    ui.column(
        "Grid",
        Element {
            gap: 8.0,
            ..Element::default()
        },
        |ui| {
            for r in 0..3u32 {
                ui.row(
                    ui_idi("R", r),
                    Element {
                        gap: 8.0,
                        ..Element::default()
                    },
                    |ui| {
                        for c in 0..3u32 {
                            let name = format!("B{r}{c}");
                            let resp = ui.button(name.as_str(), &name, ButtonDesc::default());
                            if resp.focused {
                                s.focus = Some(name.clone());
                            }
                            if resp.clicked {
                                s.clicks.push(name);
                            }
                        }
                    },
                );
            }
        },
    );
}

#[test]
fn arrows_move_to_the_aligned_neighbour_and_wrap_at_the_edges() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B00"),
        "the first arrow focuses the first control"
    );
    rig.key(UiKey::Right, b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B01"));
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B11"),
        "down stays in the column"
    );
    rig.key(UiKey::Down, b);
    rig.key(UiKey::Down, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B01"),
        "down from the last row wraps to the top of the column"
    );
    rig.key(UiKey::Left, b);
    rig.key(UiKey::Left, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B02"),
        "left from the first column wraps to the end of the row"
    );
    rig.key(UiKey::Enter, b);
    assert_eq!(
        rig.state.clicks,
        vec!["B02".to_string()],
        "Enter presses the focused control"
    );
    rig.key(UiKey::Space, b);
    assert_eq!(rig.state.clicks.len(), 2, "Space presses it too");
}

#[test]
fn tab_walks_reading_order_and_shift_tab_walks_back() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    let mut seen = Vec::new();
    for _ in 0..10 {
        rig.key(UiKey::Tab, b);
        rig.idle(1, b);
        seen.push(rig.state.focus.clone().unwrap_or_default());
    }
    assert_eq!(
        seen,
        [
            "B00", "B01", "B02", "B10", "B11", "B12", "B20", "B21", "B22", "B00"
        ],
        "Tab in reading order, wrapping"
    );
    rig.step(
        InputFrame {
            keys: vec![UiKey::Tab],
            shift: true,
            ..InputFrame::default()
        },
        b,
    );
    rig.idle(1, b);
    assert_eq!(
        rig.state.focus.as_deref(),
        Some("B22"),
        "Shift+Tab wraps backwards"
    );
}

#[test]
fn the_gamepad_dpad_and_stick_navigate_with_repeat_by_tick() {
    let mut rig = Rig::new(Grid::default());
    let b = &mut grid;
    rig.idle(1, b);
    let pad = |pressed: u32, stick: Vec2| InputFrame {
        gamepad: GamepadFrame {
            pressed,
            stick,
            ..Default::default()
        },
        ..InputFrame::default()
    };
    rig.step(pad(GamepadFrame::RIGHT, Vec2::ZERO), b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B00"));
    rig.step(pad(GamepadFrame::RIGHT, Vec2::ZERO), b);
    rig.idle(1, b);
    assert_eq!(rig.state.focus.as_deref(), Some("B01"), "d-pad right");
    // Hold the stick right: one move now, then after 400 ms (24 ticks) one
    // every 180 ms (11 ticks).
    let mut moves = Vec::new();
    let mut last = rig.state.focus.clone();
    for tick in 0..48 {
        rig.step(pad(0, Vec2::new(0.9, 0.1)), b);
        if rig.state.focus != last {
            moves.push(tick);
            last = rig.state.focus.clone();
        }
    }
    assert_eq!(
        moves,
        vec![0, 24, 35, 46],
        "stick: at once, after the slow delay, then every fast step"
    );
    rig.step(pad(GamepadFrame::A, Vec2::ZERO), b);
    assert_eq!(rig.state.clicks.len(), 1, "A presses the focused control");
}

#[derive(Default)]
struct Dialog {
    open: bool,
    under: u32,
    ok: u32,
    dismissed: u32,
    locked: bool,
    focus: Vec<String>,
}

fn dialog_screen(ui: &mut Ui, s: &mut Dialog) {
    s.focus.clear();
    if ui.button("Open", "Open", ButtonDesc::default()).clicked {
        s.open = true;
    }
    let r = ui.button("Under", "Under", ButtonDesc::default());
    if r.clicked {
        s.under += 1;
    }
    let desc = ModalDesc {
        title: "Quit?".into(),
        locked: s.locked,
        ..ModalDesc::default()
    };
    let m = ui.modal("Confirm", s.open, &desc, |ui| {
        for name in ["Cancel", "Ok"] {
            let r = ui.button(name, name, ButtonDesc::default());
            if r.focused {
                s.focus.push(name.into());
            }
            if r.clicked {
                if name == "Ok" {
                    s.ok += 1;
                }
                s.open = false;
            }
        }
    });
    if m.dismissed {
        s.dismissed += 1;
        s.open = false;
    }
}

#[test]
fn a_dialog_traps_focus_blocks_what_is_under_it_and_closes_on_back() {
    let mut rig = Rig::new(Dialog::default());
    let b = &mut dialog_screen;
    rig.idle(1, b);
    rig.key(UiKey::Tab, b);
    rig.key(UiKey::Enter, b);
    assert!(rig.state.open, "Enter on Open opens the dialog");
    rig.idle(30, b);
    assert_eq!(
        rig.state.focus,
        vec!["Cancel".to_string()],
        "focus moved into the dialog"
    );
    for _ in 0..5 {
        rig.key(UiKey::Tab, b);
    }
    rig.idle(1, b);
    assert!(
        rig.state.focus == vec!["Ok".to_string()] || rig.state.focus == vec!["Cancel".to_string()],
        "Tab stays inside: {:?}",
        rig.state.focus
    );
    // The Under button sits under the backdrop: a press there hits the
    // backdrop, which dismisses.
    let under = rig.center("Under");
    rig.step(press(under), b);
    rig.step(release(under), b);
    assert_eq!(
        rig.state.under, 0,
        "the press did not reach the control under the dialog"
    );
    assert_eq!(rig.state.dismissed, 1, "a press on the backdrop dismisses");
    rig.idle(30, b);
    assert!(
        rig.ui.frame().element("Confirm").is_none(),
        "gone after its exit"
    );

    // Locked: neither Back nor the backdrop closes it.
    rig.state.locked = true;
    rig.state.open = true;
    rig.idle(30, b);
    rig.key(UiKey::Escape, b);
    rig.step(press(under), b);
    rig.step(release(under), b);
    assert_eq!(rig.state.dismissed, 1, "locked");
    rig.state.locked = false;
    rig.idle(1, b);
    rig.key(UiKey::Escape, b);
    assert_eq!(rig.state.dismissed, 2, "Back dismisses an unlocked dialog");
    rig.idle(30, b);
    rig.key(UiKey::Enter, b);
    assert!(
        rig.state.open,
        "focus came back to Open, so Enter opens it again"
    );
}

/// A dialog takes input from the first frame it shows, while it still
/// eases in (a menu pressed at once), but a press on its backdrop
/// dismisses only once it has entered: the second press of a double click
/// on what opened it does not close it. Ways it could fail: the content is
/// inert while entering (the press is dropped), or the backdrop dismisses
/// on the opening double click.
#[test]
fn a_dialog_takes_a_press_while_it_enters_and_its_backdrop_waits() {
    let mut rig = Rig::new(Dialog::default());
    let b = &mut dialog_screen;
    rig.idle(1, b);
    let open = rig.center("Open");
    rig.click("Open", b);
    assert!(rig.state.open);
    // Two ticks in, far from settled: the second press of a double click
    // lands on the backdrop over Open.
    rig.step(press(open), b);
    rig.step(release(open), b);
    assert_eq!(
        rig.state.dismissed, 0,
        "the backdrop waits for the entrance"
    );
    assert!(rig.state.open);
    let ok = rig.center("Ok");
    rig.step(press(ok), b);
    rig.step(release(ok), b);
    assert_eq!(rig.state.ok, 1, "pressed while it eases in");
    assert!(!rig.state.open);
}

/// A dialog opened from another gives the focus back to the control that
/// opened it when it closes, and the outer one then gives it back to its
/// own opener: a submenu returns to its row. Ways it could fail: the focus
/// lands on the outer dialog's first control (one return remembered for
/// the base only), or the outer dialog's close forgets its opener.
#[test]
fn nested_dialogs_give_the_focus_back_to_what_opened_each() {
    #[derive(Default)]
    struct Nest {
        outer: bool,
        inner: bool,
    }
    let mut rig = Rig::new(Nest::default());
    let b = &mut |ui: &mut Ui, s: &mut Nest| {
        if ui.button("Open", "Open", ButtonDesc::default()).clicked {
            s.outer = true;
        }
        let m = ui.modal("Outer", s.outer, &ModalDesc::default(), |ui| {
            ui.button("First", "First", ButtonDesc::default());
            if ui.button("More", "More", ButtonDesc::default()).clicked {
                s.inner = true;
            }
        });
        if m.dismissed {
            s.outer = false;
        }
        let m = ui.modal("Inner", s.inner, &ModalDesc::default(), |ui| {
            ui.button("Deep", "Deep", ButtonDesc::default());
        });
        if m.dismissed {
            s.inner = false;
        }
    };
    rig.idle(1, b);
    rig.key(UiKey::Tab, b);
    rig.key(UiKey::Enter, b);
    rig.idle(20, b);
    assert_eq!(rig.ui.focus_path(0), "Outer/Panel/First");
    rig.key(UiKey::Down, b);
    assert_eq!(rig.ui.focus_path(0), "Outer/Panel/More");
    rig.key(UiKey::Enter, b);
    rig.idle(20, b);
    assert_eq!(rig.ui.focus_path(0), "Inner/Panel/Deep");
    rig.key(UiKey::Escape, b);
    rig.idle(20, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Outer/Panel/More",
        "back to the row that opened it"
    );
    rig.key(UiKey::Escape, b);
    rig.idle(20, b);
    assert_eq!(rig.ui.focus_path(0), "Open", "and then to the first opener");
}

/// A HUD under a pause dialog, as og.ui.racingHud draws: it asks
/// `Ui::back` before the dialog is built this tick.
#[derive(Default)]
struct Paused {
    open: bool,
    locked: bool,
    hud_back: u32,
    dismissed: u32,
}

fn hud_under_pause(ui: &mut Ui, s: &mut Paused) {
    ui.set_navigation(false);
    if ui.back() {
        s.hud_back += 1;
        s.open = true;
    }
    let desc = ModalDesc {
        title: "Paused".into(),
        locked: s.locked,
        ..ModalDesc::default()
    };
    let m = ui.modal("Pause", s.open, &desc, |ui| {
        if ui.button("Resume", "Resume", ButtonDesc::default()).clicked {
            s.open = false;
        }
    });
    if m.dismissed {
        s.dismissed += 1;
        s.open = false;
    }
}

#[test]
fn a_hud_never_takes_back_while_a_dialog_is_open() {
    let mut rig = Rig::new(Paused::default());
    let b = &mut hud_under_pause;
    rig.idle(1, b);
    rig.key(UiKey::Escape, b);
    assert_eq!(
        rig.state.hud_back, 1,
        "no dialog: Back reaches the HUD (it pauses)"
    );
    assert!(rig.state.open);
    rig.idle(30, b);
    rig.key(UiKey::Escape, b);
    assert_eq!(
        rig.state.hud_back, 1,
        "the HUD asked first, but the open dialog keeps Back"
    );
    assert_eq!(rig.state.dismissed, 1, "Escape closed the pause dialog");
    assert!(!rig.state.open);
    rig.idle(30, b);
    rig.step(pad(0, GamepadFrame::B), b);
    assert_eq!(rig.state.hud_back, 2, "closed again: the pad's B pauses");
    rig.idle(30, b);
    rig.step(pad(0, GamepadFrame::B), b);
    assert_eq!(
        (rig.state.hud_back, rig.state.dismissed),
        (2, 2),
        "the pad's B closes it too"
    );
    // A locked dialog does not close on Back, so Back reaches the game.
    rig.state.locked = true;
    rig.state.open = true;
    rig.idle(30, b);
    rig.key(UiKey::Escape, b);
    assert_eq!(
        (rig.state.hud_back, rig.state.dismissed),
        (3, 2),
        "under a locked dialog Back is the game's"
    );
}

/// A 3×3 grid of buttons for local multiplayer: who pressed what.
#[derive(Default)]
struct Lobby {
    players: u32,
    presses: Vec<(String, Option<u32>)>,
    rings: Vec<(String, u32)>,
}

fn lobby(ui: &mut Ui, s: &mut Lobby) {
    ui.set_players(s.players);
    s.rings.clear();
    ui.column(
        "Lobby",
        Element {
            gap: 8.0,
            ..Element::column()
        },
        |ui| {
            for r in 0..3u32 {
                ui.row(
                    ui_idi("R", r),
                    Element {
                        gap: 8.0,
                        ..Element::default()
                    },
                    |ui| {
                        for c in 0..3u32 {
                            let name = format!("B{r}{c}");
                            let resp = ui.button(name.as_str(), &name, ButtonDesc::default());
                            if resp.clicked {
                                s.presses.push((name.clone(), resp.pad));
                            }
                            if resp.focused_by != 0 {
                                s.rings.push((name, resp.focused_by));
                            }
                        }
                    },
                );
            }
        },
    );
}

fn pad(n: usize, bit: u32) -> InputFrame {
    let mut f = InputFrame::default();
    if n == 0 {
        f.gamepad.pressed = bit;
    } else {
        f.extra_pads[n - 1].pressed = bit;
    }
    f
}

/// Pads beyond the first: with `set_players(3)`, pads 1 and 2 each move a
/// focus of their own and press with A (the response names the pad); pad 0
/// keeps the shared focus; `focus_path` answers each; their rings are
/// drawn; the state replays after a restore. Ways it could fail: an extra
/// pad moves the shared focus, presses the shared focus's control, is heard
/// while the screen takes one player, or loses its place after a restore.
#[test]
fn pads_beyond_the_first_move_their_own_focus_and_press() {
    let mut rig = Rig::new(Lobby {
        players: 1,
        ..Lobby::default()
    });
    let b = &mut lobby;
    rig.idle(2, b);
    // One player: pad 1 does nothing.
    rig.step(pad(1, GamepadFrame::RIGHT), b);
    rig.step(pad(1, GamepadFrame::A), b);
    rig.idle(1, b);
    assert!(rig.state.presses.is_empty(), "pad 1 is not a player yet");
    assert_eq!(rig.ui.focus_path(1), "");
    rig.state.players = 3;
    rig.idle(2, b);
    // Pad 1: its first move lands on the control nearest the surface's
    // centre (B22, the grid sits top left), then left to B21.
    rig.step(pad(1, GamepadFrame::RIGHT), b);
    rig.step(pad(1, GamepadFrame::LEFT), b);
    // Pad 2: onto B22 the same way, then up to B12.
    rig.step(pad(2, GamepadFrame::DOWN), b);
    rig.step(pad(2, GamepadFrame::UP), b);
    // Pad 0: the shared focus, its own place.
    rig.step(pad(0, GamepadFrame::DOWN), b);
    rig.idle(1, b);
    assert_eq!(rig.ui.focus_path(1), "Lobby/R#2/B21");
    assert_eq!(rig.ui.focus_path(2), "Lobby/R#1/B12");
    assert_eq!(rig.ui.focus_path(0), "Lobby/R#0/B00");
    assert_eq!(rig.ui.focus_path(3), "", "pad 3 has not moved");
    let saved = rig.ui.save_state();
    rig.step(pad(1, GamepadFrame::A), b);
    rig.step(pad(2, GamepadFrame::A), b);
    rig.step(pad(0, GamepadFrame::A), b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.presses,
        vec![
            ("B21".to_string(), Some(1)),
            ("B12".to_string(), Some(2)),
            ("B00".to_string(), Some(0)),
        ],
        "each pad pressed the control under its own focus"
    );
    let ring = |name: &str| rig.state.rings.iter().find(|r| r.0 == name).map(|r| r.1);
    assert_eq!(ring("B21"), Some(2), "B21 shows pad 1's ring");
    assert_eq!(ring("B12"), Some(4), "B12 shows pad 2's ring");
    let info = rig.ui.theme().palette.info.linear();
    assert!(
        rig.ui
            .frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Quad { color, .. } if *color == info)),
        "pad 1's ring is drawn in its player colour"
    );
    // A restore before the presses replays them identically.
    let before = rig.state.presses.clone();
    rig.state.presses.clear();
    rig.ui.restore_state(&saved).expect("restore");
    rig.step(pad(1, GamepadFrame::A), b);
    rig.step(pad(2, GamepadFrame::A), b);
    rig.step(pad(0, GamepadFrame::A), b);
    rig.idle(1, b);
    assert_eq!(rig.state.presses, before, "the pads' focus is UI state");
    // Back to one player: the extra focus is dropped.
    rig.state.players = 1;
    rig.idle(2, b);
    assert_eq!(rig.ui.focus_path(1), "");
}

/// What opens takes the focus (the og-ui skill §4: the topmost owns it): a
/// dialogue box that appears over a HUD whose button had the focus answers
/// Enter at once, and a dialog the game opens (not a press: a state change,
/// as a shop or the results open) answers Enter on its first control.
#[test]
fn a_dialogue_or_a_dialog_that_appears_takes_the_focus_so_enter_answers_it() {
    #[derive(Default)]
    struct Talk {
        talking: bool,
        open: bool,
        advanced: u32,
        paused: u32,
        bought: u32,
    }
    let mut rig = Rig::new(Talk::default());
    let b = &mut |ui: &mut Ui, s: &mut Talk| {
        if ui.button("Pause", "Pause", ButtonDesc::default()).clicked {
            s.paused += 1;
        }
        if s.talking {
            let r = ui.dialogue(
                "Talk",
                &DialogueDesc {
                    speaker: "Burrow".into(),
                    text: "Mind the brambles.".into(),
                    ..DialogueDesc::default()
                },
            );
            if r.advanced {
                s.advanced += 1;
            }
        }
        let m = ui.open_modal(
            "Shop",
            s.open,
            &ModalDesc {
                title: "Shop".into(),
                ..ModalDesc::default()
            },
        );
        if m.visible && ui.button("Buy", "Buy", ButtonDesc::default()).clicked {
            s.bought += 1;
        }
        ui.close_modal();
    };
    rig.idle(1, b);
    rig.key(UiKey::Tab, b);
    rig.idle(1, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Pause",
        "the HUD's button had the focus"
    );
    rig.state.talking = true;
    rig.idle(2, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Talk",
        "the dialogue took it as it appeared"
    );
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(
        (rig.state.advanced, rig.state.paused),
        (1, 0),
        "Enter finished the line, then advanced it; Pause was not pressed"
    );
    rig.state.talking = false;
    rig.state.open = true;
    rig.idle(30, b);
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(
        rig.state.bought, 1,
        "the dialog the game opened answered Enter"
    );
}

/// Kestrel-stair FINDINGS #14: a title menu is not a dialog, so its first
/// item takes the focus itself as it appears (`ButtonDesc::autofocus`), and
/// Enter answers it on the first press; a move queued while the focus is on
/// nothing lands on the first item, however many were queued before it did.
#[test]
fn a_menu_that_appears_takes_the_focus_and_a_move_from_nothing_lands_on_its_first_item() {
    #[derive(Default)]
    struct Title {
        shown: bool,
        autofocus: bool,
        chosen: Vec<&'static str>,
    }
    let b = &mut |ui: &mut Ui, s: &mut Title| {
        if !s.shown {
            return;
        }
        for (k, id) in ["Play", "Chapters", "Settings"].into_iter().enumerate() {
            let desc = ButtonDesc {
                autofocus: s.autofocus && k == 0,
                ..ButtonDesc::default()
            };
            if ui.button(id, id, desc).clicked {
                s.chosen.push(id);
            }
        }
    };
    let mut rig = Rig::new(Title {
        shown: true,
        autofocus: true,
        ..Title::default()
    });
    rig.idle(1, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Play",
        "the first item took the focus as the menu appeared"
    );
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(rig.state.chosen, ["Play"], "the first Enter chose it");
    // Moved on, the focus stays where the player put it: the item asks only as it appears.
    rig.key(UiKey::Down, b);
    rig.idle(3, b);
    assert_eq!(rig.ui.focus_path(0), "Chapters");
    // The menu leaves and comes back: it takes the focus again.
    rig.state.shown = false;
    rig.idle(2, b);
    assert_eq!(rig.ui.focus_path(0), "");
    rig.state.shown = true;
    rig.idle(1, b);
    assert_eq!(rig.ui.focus_path(0), "Play");

    // Without autofocus, a game asking each tick until the focus lands
    // queues several moves from nothing: all of them land on the first item.
    let mut rig = Rig::new(Title {
        shown: true,
        ..Title::default()
    });
    rig.idle(1, b);
    assert_eq!(rig.ui.focus_path(0), "");
    rig.ui.navigate(Nav::Down);
    rig.ui.navigate(Nav::Down);
    rig.idle(1, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Play",
        "two moves from nothing landed on the first item"
    );
    rig.ui.navigate(Nav::Prev);
    rig.idle(1, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Settings",
        "a move from a focused item moves as asked"
    );
    // A move from nothing whatever its direction, then a move queued after it landed.
    let mut rig = Rig::new(Title {
        shown: true,
        ..Title::default()
    });
    rig.idle(1, b);
    rig.ui.navigate(Nav::Prev);
    rig.idle(1, b);
    rig.ui.navigate(Nav::Down);
    rig.idle(1, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Chapters",
        "from nothing to the first item, then down one"
    );
}

#[derive(Default)]
struct Cards {
    clicks: [u32; 2],
}

fn cards_screen(ui: &mut Ui, s: &mut Cards) {
    ui.button("First", "First", ButtonDesc::default());
    for (k, focusable) in [true, false].into_iter().enumerate() {
        ui.open(
            ui_idi("Card", k as u32),
            Element {
                interactive: true,
                focusable,
                padding: Insets::all(8.0),
                ..Element::column()
            },
        );
        ui.label(
            "Name",
            if focusable { "Reach" } else { "Skip" },
            LabelDesc::default(),
        );
        if ui.close().clicked {
            s.clicks[k] += 1;
        }
    }
    ui.button("Last", "Last", ButtonDesc::default());
}

/// An interactive container marked focusable is a stop of Tab and the pad
/// and Enter presses it; one that is not stays a pointer target only.
///
/// Ways it could fail: Tab skips the focusable card (it is not a nav
/// target), Enter on it does not click, the plain interactive container
/// becomes a Tab stop, or a pointer click no longer reaches either.
#[test]
fn a_focusable_container_takes_keys_and_pads_as_a_button() {
    let mut rig = Rig::new(Cards::default());
    let b = &mut cards_screen;
    rig.idle(2, b);
    rig.key(UiKey::Tab, b);
    assert_eq!(rig.ui.focus_path(0), "First");
    rig.key(UiKey::Tab, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Card#0",
        "the focusable card is the next stop"
    );
    rig.key(UiKey::Enter, b);
    rig.idle(1, b);
    assert_eq!(rig.state.clicks, [1, 0], "Enter presses it");
    rig.key(UiKey::Tab, b);
    assert_eq!(
        rig.ui.focus_path(0),
        "Last",
        "the plain container is no stop"
    );
    rig.click("Card#1", b);
    rig.click("Card#0", b);
    rig.idle(1, b);
    assert_eq!(rig.state.clicks, [2, 1], "the pointer reaches both");
}

/// An adjustable focusable container takes Left and Right while it has
/// the focus (Response.adjust, the screen's directions) and keeps the
/// focus; Up and Down still move it, and a plain focusable one lets Left
/// and Right move the focus. Ways it could fail: Right moves the focus off
/// the row (a submenu could never open from the keyboard), the step is
/// reported to a row that is not focused, or the sign is the reading
/// order's.
#[test]
fn an_adjustable_container_takes_left_and_right_while_focused() {
    #[derive(Default)]
    struct Rows {
        adjust: Vec<i32>,
    }
    let mut rig = Rig::new(Rows::default());
    let b = &mut |ui: &mut Ui, s: &mut Rows| {
        s.adjust.clear();
        ui.button("Side", "Side", ButtonDesc::default());
        ui.column("Menu", Element::column(), |ui| {
            for k in 0..2u32 {
                ui.open(
                    ui_idi("Row", k),
                    Element {
                        interactive: true,
                        focusable: true,
                        adjustable: k == 0,
                        padding: Insets::all(8.0),
                        ..Element::row()
                    },
                );
                ui.label("Name", "Row", LabelDesc::default());
                s.adjust.push(ui.close().adjust);
            }
        });
    };
    rig.idle(2, b);
    rig.key(UiKey::Tab, b);
    rig.key(UiKey::Tab, b);
    assert_eq!(rig.ui.focus_path(0), "Menu/Row#0");
    rig.step(keys(&[UiKey::Right]), b);
    assert_eq!(rig.state.adjust, [1, 0], "Right steps it forward");
    rig.step(InputFrame::default(), b);
    assert_eq!(rig.state.adjust, [0, 0], "one tick");
    assert_eq!(rig.ui.focus_path(0), "Menu/Row#0", "and the focus stays");
    rig.step(keys(&[UiKey::Left]), b);
    assert_eq!(rig.state.adjust, [-1, 0]);
    rig.key(UiKey::Down, b);
    assert_eq!(rig.ui.focus_path(0), "Menu/Row#1", "Down still moves");
    rig.key(UiKey::Right, b);
    assert_ne!(
        rig.ui.focus_path(0),
        "Menu/Row#1",
        "a plain row lets Right move the focus"
    );
}

/// `last_path` names the element just built by its full path (what
/// `bounds` and `gesture` take), inside nested containers and after a
/// container closes; empty before anything is built.
#[test]
fn last_path_names_the_element_just_built() {
    let mut ui = Ui::new(UiDesc::default());
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    assert_eq!(ui.last_path(), "");
    let mut seen = Vec::new();
    ui.column("Shop", Element::column(), |ui| {
        ui.row("Row", Element::row(), |ui| {
            ui.button("Buy", "Buy", ButtonDesc::default());
            seen.push(ui.last_path());
        });
        seen.push(ui.last_path());
    });
    seen.push(ui.last_path());
    ui.end();
    assert_eq!(seen, ["Shop/Row/Buy", "Shop/Row", "Shop"]);
}
