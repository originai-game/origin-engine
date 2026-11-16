use super::*;

/// A 3×3 grid of buttons for local multiplayer.
const LOBBY: &str = r#"<meta name="component" content="t">
<div id="Root">
<column id="Lobby">
  <row id="R0"><button id="B00">B00</button><button id="B01">B01</button><button id="B02">B02</button></row>
  <row id="R1"><button id="B10">B10</button><button id="B11">B11</button><button id="B12">B12</button></row>
  <row id="R2"><button id="B20">B20</button><button id="B21">B21</button><button id="B22">B22</button></row>
</column>
</div>
<style>
#Lobby, #R0, #R1, #R2 { gap: 8px; }
</style>"#;

/// One tick with `players` players: a game declares them every tick.
fn step(d: &mut Doc, players: u32, input: InputFrame) {
    d.ui.begin(input, d.viewport);
    d.ui.set_players(players);
    d.ui.build_retained_documents(&bind::NoEnv).expect("builds");
    d.ui.end();
    d.events.extend(d.ui.take_doc_events());
}

fn idle_with(d: &mut Doc, players: u32, n: u32) {
    for _ in 0..n {
        step(d, players, InputFrame::default());
    }
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
/// focus of their own and press with A; pad 0 keeps the shared focus;
/// `focus_path` answers each; their rings are drawn; the state replays
/// after a restore. Ways it could fail: an extra pad moves the shared
/// focus, presses the shared focus's control, is heard while the screen
/// takes one player, or loses its place after a restore.
#[test]
fn pads_beyond_the_first_move_their_own_focus_and_press() {
    let mut d = mount(LOBBY, &[]);
    let mut players = 1;
    idle_with(&mut d, players, 2);
    // One player: pad 1 does nothing.
    step(&mut d, players, pad(1, GamepadFrame::RIGHT));
    step(&mut d, players, pad(1, GamepadFrame::A));
    idle_with(&mut d, players, 1);
    assert!(clicks(&mut d).is_empty(), "pad 1 is not a player yet");
    assert_eq!(focus(&d, 1), "");
    players = 3;
    idle_with(&mut d, players, 2);
    // Pad 1: its first move lands on the control nearest the surface's
    // centre (B22, the grid sits top left), then left to B21.
    step(&mut d, players, pad(1, GamepadFrame::RIGHT));
    step(&mut d, players, pad(1, GamepadFrame::LEFT));
    // Pad 2: onto B22 the same way, then up to B12.
    step(&mut d, players, pad(2, GamepadFrame::DOWN));
    step(&mut d, players, pad(2, GamepadFrame::UP));
    // Pad 0: the shared focus, its own place.
    step(&mut d, players, pad(0, GamepadFrame::DOWN));
    idle_with(&mut d, players, 1);
    assert_eq!(focus(&d, 1), "Lobby/R2/B21");
    assert_eq!(focus(&d, 2), "Lobby/R1/B12");
    assert_eq!(focus(&d, 0), "Lobby/R0/B00");
    assert_eq!(focus(&d, 3), "", "pad 3 has not moved");
    let saved = d.ui.save_state();
    clicks(&mut d);
    // Each pad's A presses the control under its own focus.
    let presses = |d: &mut Doc| {
        let mut presses = Vec::new();
        for n in [1, 2, 0] {
            step(d, players, pad(n, GamepadFrame::A));
            idle_with(d, players, 1);
            presses.push((n, clicks(d)));
        }
        presses
    };
    let pressed = presses(&mut d);
    let want = |name: &str| vec![name.to_string()];
    assert_eq!(
        pressed,
        vec![(1, want("B21")), (2, want("B12")), (0, want("B00"))],
        "each pad pressed the control under its own focus"
    );
    let info = d.ui.theme().palette.info.linear();
    assert!(
        d.ui.frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. }
                if matches!(shape.mode, og_draw::UiMode::Stroke(_))
                    && shape.paint == og_draw::UiPaint::Solid(info))),
        "pad 1's ring is drawn in its player colour"
    );
    // A restore before the presses replays them identically.
    d.ui.restore_state(&saved).expect("restore");
    assert_eq!(presses(&mut d), pressed, "the pads' focus is UI state");
    // Back to one player: the extra focus is dropped.
    players = 1;
    idle_with(&mut d, players, 2);
    assert_eq!(focus(&d, 1), "");
}

/// A split screen: two `player` regions side by side and a shared button
/// outside both. Pads 1 and 2 each move only inside their own region
/// (their first move lands at its centre) and press there; pad 0 (no
/// region) reaches only the shared button. Ways it could fail: a pad's
/// focus crosses into the other player's view, or the shared controls are
/// lost.
#[test]
fn split_screen_regions_confine_each_pads_focus() {
    let mut d = mount(
        r#"<meta name="component" content="t">
<div id="Root">
<row id="Screen">
  <column id="View1" player="1"><button id="P1B0">P1B0</button><button id="P1B1">P1B1</button><button id="P1B2">P1B2</button></column>
  <column id="View2" player="2"><button id="P2B0">P2B0</button><button id="P2B1">P2B1</button><button id="P2B2">P2B2</button></column>
  <button id="Quit">Quit</button>
</row>
</div>
<style>
#View1, #View2 { gap: 8px; }
</style>"#,
        &[],
    );
    let players = 3;
    idle_with(&mut d, players, 2);
    // Pad 1 lands on its region's centre, then runs right and down: it
    // stays in view 1.
    step(&mut d, players, pad(1, GamepadFrame::DOWN));
    assert_eq!(focus(&d, 1), "Screen/View1/P1B1");
    for _ in 0..3 {
        step(&mut d, players, pad(1, GamepadFrame::RIGHT));
        step(&mut d, players, pad(1, GamepadFrame::DOWN));
    }
    assert!(
        focus(&d, 1).starts_with("Screen/View1/"),
        "{}",
        focus(&d, 1)
    );
    // Pad 2 lands in view 2 and runs left: it stays there.
    step(&mut d, players, pad(2, GamepadFrame::UP));
    assert_eq!(focus(&d, 2), "Screen/View2/P2B1");
    for _ in 0..3 {
        step(&mut d, players, pad(2, GamepadFrame::LEFT));
    }
    assert!(
        focus(&d, 2).starts_with("Screen/View2/"),
        "{}",
        focus(&d, 2)
    );
    // Pad 0 has no region: only the shared button.
    step(&mut d, players, pad(0, GamepadFrame::DOWN));
    step(&mut d, players, pad(0, GamepadFrame::LEFT));
    assert_eq!(focus(&d, 0), "Screen/Quit");
    clicks(&mut d);
    let mut who = Vec::new();
    for n in [1, 2, 0] {
        step(&mut d, players, pad(n, GamepadFrame::A));
        idle_with(&mut d, players, 1);
        who.push(clicks(&mut d));
    }
    assert_eq!(who.len(), 3);
    assert!(
        who[0].len() == 1 && who[0][0].starts_with("P1"),
        "pad 1 pressed in view 1: {who:?}"
    );
    assert!(
        who[1].len() == 1 && who[1][0].starts_with("P2"),
        "pad 2 pressed in view 2: {who:?}"
    );
    assert_eq!(who[2], ["Quit"], "pad 0 the shared button");
}

/// A UI state saved at version 12 (before each hit carried its focusable
/// owner) upgrades to this version with the owners unset, restores, and the
/// next frame fills them: a press on the item focuses its grid again.
#[test]
fn a_version_12_state_upgrades_and_restores() {
    const GRID: &str = r#"<meta name="component" content="t">
<div id="Root"><row id="Grid" interactive focusable><div id="A" interactive></div><div id="B" interactive></div></row></div>
<style>
#Grid { padding: 8px; }
#A, #B { width: 40px; height: 40px; }
</style>"#;
    let mut d = mount(GRID, &[]);
    let now = d.ui.save_state();
    // This version's layout: each hit's owner (8 bytes) after its key, rect,
    // full rect, flags, scope and scroller, and its focus group and
    // recogniser (16) after its maps and rects; version 12 had neither, nor
    // the groups, moves, recognisers and press (61 bytes here, all empty)
    // before the layer stack.
    let u32_at =
        |b: &[u8], at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    assert_eq!(u32_at(&now, 0), 25);
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
    // letters), version 18's (4: no Rive players), version 20's (69:
    // nothing selected), version 24's (8: no touch press) and version
    // 25's (1: no drag axis).
    v12.extend_from_slice(&now[at..docs - 61 - 122 - 4 - 12 - 4 - 69 - 8 - 1]);
    v12.extend_from_slice(&now[docs..]);
    assert_eq!(owners, 2, "both items had the grid as their owner");

    assert!(
        d.ui.restore_state(&v12).is_err(),
        "a v12 block is not read as this version"
    );
    let upgraded = Ui::upgrade_state(&v12).expect("version 12 upgrades");
    assert_eq!(upgraded, zeroed, "the same state, its owners unset");
    let mut other = mount(GRID, &[]);
    other
        .ui
        .restore_state(&upgraded)
        .expect("the upgraded state restores");
    idle(&mut other, 1);
    tap(&mut other, "Grid/B");
    assert_eq!(focus(&other, 0), "Grid");
}
