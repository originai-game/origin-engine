use super::*;

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
            .any(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. }
                if matches!(shape.mode, og_render::UiMode::Stroke(_))
                    && shape.paint == og_render::UiPaint::Solid(info))),
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

/// A split screen: two `Props::player` regions side by side and a shared
/// button outside both. Pads 1 and 2 each move only inside their own region
/// (their first move lands at its centre) and a press says whose region it
/// was in; pad 0 (no region) reaches only the shared button. Ways it could
/// fail: a pad's focus crosses into the other player's view, the shared
/// controls are lost, or the response names the wrong player.
#[test]
fn split_screen_regions_confine_each_pads_focus() {
    #[derive(Default)]
    struct Split {
        presses: Vec<(String, Option<u32>)>,
    }
    let build = &mut |ui: &mut Ui, s: &mut Split| {
        ui.set_players(3);
        ui.open("Screen", Element::row());
        for k in 1..3u8 {
            ui.open_with(
                ui_idi("View", u32::from(k)),
                Element {
                    gap: 8.0,
                    ..Element::column()
                },
                Props {
                    player: Some(k),
                    ..Props::default()
                },
            );
            for b in 0..3 {
                let name = format!("P{k}B{b}");
                let r = ui.button(name.as_str(), &name, ButtonDesc::default());
                if r.clicked {
                    s.presses.push((name, r.player));
                }
            }
            ui.close();
        }
        let r = ui.button("Quit", "Quit", ButtonDesc::default());
        if r.clicked {
            s.presses.push(("Quit".into(), r.player));
        }
        ui.close();
    };
    let mut rig = Rig::new(Split::default());
    rig.idle(2, build);
    // Pad 1 lands on its region's centre, then runs right and down: it
    // stays in view 1.
    rig.step(pad(1, GamepadFrame::DOWN), build);
    assert_eq!(rig.ui.focus_path(1), "Screen/View#1/P1B1");
    for _ in 0..3 {
        rig.step(pad(1, GamepadFrame::RIGHT), build);
        rig.step(pad(1, GamepadFrame::DOWN), build);
    }
    assert!(
        rig.ui.focus_path(1).starts_with("Screen/View#1/"),
        "{}",
        rig.ui.focus_path(1)
    );
    // Pad 2 lands in view 2 and runs left: it stays there.
    rig.step(pad(2, GamepadFrame::UP), build);
    assert_eq!(rig.ui.focus_path(2), "Screen/View#2/P2B1");
    for _ in 0..3 {
        rig.step(pad(2, GamepadFrame::LEFT), build);
    }
    assert!(
        rig.ui.focus_path(2).starts_with("Screen/View#2/"),
        "{}",
        rig.ui.focus_path(2)
    );
    // Pad 0 has no region: only the shared button.
    rig.step(pad(0, GamepadFrame::DOWN), build);
    rig.step(pad(0, GamepadFrame::LEFT), build);
    assert_eq!(rig.ui.focus_path(0), "Screen/Quit");
    rig.step(pad(1, GamepadFrame::A), build);
    rig.step(pad(2, GamepadFrame::A), build);
    rig.step(pad(0, GamepadFrame::A), build);
    rig.idle(1, build);
    let who: Vec<_> = rig.state.presses.iter().map(|p| p.1).collect();
    assert_eq!(who, vec![Some(1), Some(2), None], "{:?}", rig.state.presses);
    assert!(rig.state.presses[0].0.starts_with("P1"));
    assert!(rig.state.presses[1].0.starts_with("P2"));
}

/// A UI state saved at version 12 (before each hit carried its focusable
/// owner) upgrades to this version with the owners unset, restores, and the
/// next frame fills them: a press on the item focuses its grid again.
#[test]
fn a_version_12_state_upgrades_and_restores() {
    let build = &mut |ui: &mut Ui, _: &mut ()| {
        ui.open(
            "Grid",
            Element {
                interactive: true,
                focusable: true,
                padding: Insets::all(8.0),
                ..Element::row()
            },
        );
        for id in ["A", "B"] {
            ui.open(
                id,
                Element {
                    interactive: true,
                    width: Size::Fixed(40.0),
                    height: Size::Fixed(40.0),
                    ..Element::default()
                },
            );
            ui.close();
        }
        ui.close();
    };
    let mut rig = Rig::new(());
    rig.idle(2, build);
    let now = rig.ui.save_state();
    // This version's layout: each hit's owner (8 bytes) after its key, rect,
    // full rect, flags, scope and scroller, and its focus group and
    // recogniser (16) after its maps and rects; version 12 had neither, nor
    // the groups, moves, recognisers and press (61 bytes here, all empty)
    // before the layer stack.
    let u32_at =
        |b: &[u8], at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    assert_eq!(u32_at(&now, 0), 23);
    let scrolls = u32_at(&now, 44);
    let hits_at = 48 + scrolls * 12;
    let hits = u32_at(&now, hits_at);
    assert!(hits >= 3, "the grid and its two items are hit");
    let (mut v12, mut zeroed) = (now[..hits_at + 4].to_vec(), now[..hits_at + 4].to_vec());
    v12[..4].copy_from_slice(&12u32.to_le_bytes());
    let mut at = hits_at + 4;
    let mut owners = 0;
    for _ in 0..hits {
        let head = 8 + 16 + 16 + 2 + 8 + 8;
        v12.extend_from_slice(&now[at..at + head]);
        zeroed.extend_from_slice(&now[at..at + head]);
        owners += usize::from(now[at + head..at + head + 8] != [0; 8]);
        zeroed.extend_from_slice(&[0; 8]);
        // Its map (24, and version 22's projective row, 12), local rect
        // and clip.
        let map = at + head + 8;
        v12.extend_from_slice(&now[map..map + 24]);
        v12.extend_from_slice(&now[map + 36..map + 36 + 32]);
        // Version 14's group and recogniser (16) and version 21's split
        // screen region (1) are kept in the upgrade, empty.
        zeroed.extend_from_slice(&now[map..map + 36 + 32 + 16 + 1]);
        at = map + 36 + 32 + 16 + 1;
    }
    zeroed.extend_from_slice(&now[at..]);
    // The layer stack is the last string: its length, then its JSON.
    let docs = (0..now.len() - 4)
        .rev()
        .find(|&k| u32_at(&now, k) + k + 4 == now.len() && now[k + 4] == b'{')
        .expect("the layer stack");
    // Without version 14's empty block (61 bytes), version 15's (122: no
    // virtual cursor, no HUD layouts or targets), version 16's (4: no
    // touch control holding a finger), version 17's (12: no typed
    // letters), version 18's (4: no Rive players) and version 20's (69:
    // nothing selected).
    v12.extend_from_slice(&now[at..docs - 61 - 122 - 4 - 12 - 4 - 69]);
    v12.extend_from_slice(&now[docs..]);
    assert_eq!(owners, 2, "both items had the grid as their owner");

    assert!(
        rig.ui.restore_state(&v12).is_err(),
        "a v12 block is not read as this version"
    );
    let upgraded = Ui::upgrade_state(&v12).expect("version 12 upgrades");
    assert_eq!(upgraded, zeroed, "the same state, its owners unset");
    let mut other = Rig::new(());
    other.idle(1, build);
    other
        .ui
        .restore_state(&upgraded)
        .expect("the upgraded state restores");
    other.idle(1, build);
    other.click("Grid/B", build);
    assert_eq!(other.ui.focus_path(0), "Grid");
}
