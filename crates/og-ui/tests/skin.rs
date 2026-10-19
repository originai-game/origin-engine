//! Skins, motion and screen transitions, driven tick by tick.
//!
//! Ways these could fail, each covered below:
//! - a skin's slices never reach the frame (the background stays a plain
//!   quad), or the nine pieces leave gaps or overlap the element's edges;
//! - the default theme starts emitting skin pieces (it must stay the token
//!   look; the reference frames in tests/gfx.rs and tests/kit_gfx.rs prove
//!   the pixels);
//! - a skin file with a typo, a region outside its atlas or a glow without
//!   a soft slice loads anyway instead of naming the line;
//! - a screen transition switches the screen before the cover closes,
//!   never switches, never ends, or draws movement under reduced motion;
//! - a count-up runs on the wall clock, jumps, or differs after a restore;
//! - a refused press does not shake, or UI events with a sound never reach
//!   the host's queue.

mod support;

use og_core::glam::Vec2;
use og_render::{ColorSpace, Mips, RenderScene, TextureData, TextureHandle};
use og_ui::*;
use support::*;

const SKIN: &str = r##"type skin
version 1
---
name "test"
base dark
atlas "skins/test/atlas.png" 128 64
palette {
  surface "#203040"
  fg "#f0f0f0"
}
radius {
  panel 4
  button 4
}
slice panel 0 0 32 32 8
slice card 0 0 32 32 8
slice button 32 0 32 32 6 tint
slice fill 64 0 16 16 4
slice track 80 0 16 16 4
soft 96 0 32 32 12
corner 0 32 16 16 16 16 accent 2
corners panel card
shadow card 0 6 10 0 "#00000080"
glow button 4 accent 0.5
press 1.04 0.92 2
hover -2 1.02
motion {
  enter pop
  stagger 40
  progress 300
  shake 6
  pop 0.12
}
"##;

fn atlas() -> TextureHandle {
    RenderScene::new().add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

fn skinned_ui() -> (Ui, TextureHandle) {
    let file = parse_skin(SKIN).expect("the test skin parses");
    let tex = atlas();
    let mut ui = ui_with_font(test_font_metrics());
    let mut sounds = [0u64; 6];
    sounds[UiSound::Press as usize] = 7;
    sounds[UiSound::Error as usize] = 9;
    ui.set_theme(file.theme(Some(tex), sounds));
    (ui, tex)
}

fn screen(ui: &mut Ui, disabled: bool) {
    ui.panel("Menu", PanelDesc::default(), |ui| {
        ui.label("Title", "Menu", LabelDesc::default());
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                disabled,
                ..ButtonDesc::default()
            },
        );
    });
}

fn step(ui: &mut Ui, input: InputFrame, disabled: bool) {
    ui.begin(input, Viewport::new(800.0, 600.0));
    screen(ui, disabled);
    ui.end();
}

fn images(ui: &Ui, tex: TextureHandle) -> Vec<Rect> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Image {
                rect,
                texture: Some(t),
                ..
            } if *t == tex => Some(*rect),
            _ => None,
        })
        .collect()
}

#[test]
fn a_skinned_panel_is_nine_pieces_of_the_atlas_that_tile_its_rectangle() {
    let (mut ui, tex) = skinned_ui();
    for _ in 0..30 {
        step(&mut ui, InputFrame::default(), false);
    }
    let menu = ui.frame().element("Menu").expect("the panel").clone();
    let pieces: Vec<Rect> = images(&ui, tex)
        .into_iter()
        .filter(|r| {
            r.x >= menu.x - 0.01
                && r.y >= menu.y - 0.01
                && r.right() <= menu.x + menu.width + 0.01
                && r.bottom() <= menu.y + menu.height + 0.01
                && r.width < menu.width + 0.01
        })
        .collect();
    // The panel's nine pieces (the button's frame and corners are inside too).
    let area: f32 = pieces.iter().map(|r| r.width * r.height).sum();
    assert!(
        area >= menu.width * menu.height - 1.0,
        "the pieces cover the panel ({area} of {})",
        menu.width * menu.height
    );
    let corners: Vec<&Rect> = pieces
        .iter()
        .filter(|r| (r.width - 8.0).abs() < 0.01 && (r.height - 8.0).abs() < 0.01)
        .collect();
    assert!(
        corners.len() >= 4,
        "four 8 px corner pieces keep their size: {pieces:?}"
    );
    // The shadow (soft slice, offset 6 down) and the corner ornaments reach
    // past the panel.
    let outside = images(&ui, tex)
        .into_iter()
        .filter(|r| r.bottom() > menu.y + menu.height + 1.0)
        .count();
    assert!(
        outside > 0,
        "the drop shadow and corners draw below the panel"
    );
}

#[test]
fn the_default_theme_draws_no_skin_pieces() {
    let tex = atlas();
    let mut ui = ui_with_font(test_font_metrics());
    for _ in 0..30 {
        step(&mut ui, InputFrame::default(), false);
    }
    assert!(images(&ui, tex).is_empty());
    assert!(
        ui.frame().items.iter().all(|i| !matches!(
            i.cmd,
            DrawCmd::Image {
                texture: Some(_),
                ..
            }
        )),
        "only token quads and text"
    );
}

#[test]
fn a_skin_file_names_the_line_that_is_wrong() {
    let typo = SKIN.replace("slice fill", "slise fill");
    let e = parse_skin(&typo).unwrap_err();
    assert!(e.contains("unknown skin setting 'slise'"), "{e}");
    let outside = SKIN.replace("slice track 80 0 16 16 4", "slice track 120 0 16 16 4");
    let e = parse_skin(&outside).unwrap_err();
    assert!(e.contains("not inside the 128×64 atlas"), "{e}");
    let no_soft = SKIN.replace("soft 96 0 32 32 12\n", "");
    let e = parse_skin(&no_soft).unwrap_err();
    assert!(e.contains("needs a `soft` line"), "{e}");
    let colour = SKIN.replace("\"#203040\"", "\"203040\"");
    let e = parse_skin(&colour).unwrap_err();
    assert!(e.contains("#rrggbb"), "{e}");
    let file = parse_skin(SKIN).unwrap();
    assert_eq!(file.theme.palette.surface, Srgba::rgb(0x20, 0x30, 0x40));
    assert_eq!(file.theme.radius.panel, 4.0);
    assert_eq!(file.references(), vec!["skins/test/atlas.png".to_string()]);
}

/// Builds the screen the transition answers; returns (shown, cover pieces).
fn transition_step(ui: &mut Ui, wanted: &str, desc: &ScreenTransitionDesc) -> (String, usize) {
    ui.begin(InputFrame::default(), Viewport::new(320.0, 200.0));
    let t = ui.screen_transition("Screens", wanted, desc);
    ui.label("Screen", &t.shown, LabelDesc::default());
    ui.end();
    let full = ui
        .frame()
        .items
        .iter()
        .filter(|i| matches!(i.cmd, DrawCmd::Quad { .. }))
        .count();
    (t.shown, full)
}

#[test]
fn a_screen_transition_covers_switches_at_full_cover_and_reveals() {
    for kind in [
        ScreenWipe::Fade,
        ScreenWipe::Wipe,
        ScreenWipe::Iris,
        ScreenWipe::Dissolve,
        ScreenWipe::Slide,
    ] {
        let mut ui = ui_with_font(test_font_metrics());
        let desc = ScreenTransitionDesc {
            kind,
            duration_ms: 400.0,
            ..ScreenTransitionDesc::default()
        };
        assert_eq!(transition_step(&mut ui, "menu", &desc), ("menu".into(), 0));
        // 400 ms at 60 ticks/s: 24 ticks, 12 covering and 12 revealing.
        let mut shown = Vec::new();
        for _ in 0..30 {
            shown.push(transition_step(&mut ui, "game", &desc));
        }
        let switch = shown
            .iter()
            .position(|(s, _)| s == "game")
            .expect("it switches");
        assert_eq!(
            switch, 11,
            "{kind:?}: the old screen stays while the cover closes"
        );
        assert!(
            shown[..23].iter().all(|(_, n)| *n > 0),
            "{kind:?}: the cover draws through the transition"
        );
        assert_eq!(shown[24].1, 0, "{kind:?}: and is gone after it");
        assert!(shown[24..].iter().all(|(s, _)| s == "game"));
        if kind == ScreenWipe::Iris {
            assert!(shown[11].1 > 50, "an iris is drawn in bands");
        }
    }
    // Reduced motion: an iris is a fade, one full-surface piece per tick.
    let mut ui = ui_with_font(test_font_metrics());
    let mut t = *ui.theme();
    t.reduced_motion = true;
    ui.set_theme(t);
    let desc = ScreenTransitionDesc {
        kind: ScreenWipe::Iris,
        ..ScreenTransitionDesc::default()
    };
    transition_step(&mut ui, "a", &desc);
    for _ in 0..10 {
        let (_, n) = transition_step(&mut ui, "b", &desc);
        assert_eq!(n, 1);
    }
}

fn count_step(ui: &mut Ui, value: f64) -> f64 {
    ui.begin(InputFrame::default(), Viewport::new(320.0, 200.0));
    let v = ui.counter(
        "Score",
        value,
        &CounterDesc {
            from: Some(0.0),
            ..CounterDesc::default()
        },
    );
    ui.end();
    v
}

#[test]
fn a_counter_counts_by_the_tick_and_replays_after_a_restore() {
    let mut ui = ui_with_font(test_font_metrics());
    let shown: Vec<f64> = (0..60).map(|_| count_step(&mut ui, 12840.0)).collect();
    assert_eq!(shown[0], 0.0, "it starts at `from`");
    assert!(shown.windows(2).all(|w| w[1] >= w[0]), "it never goes back");
    assert_eq!(shown[54], 12840.0, "it lands at 900 ms (54 ticks)");
    assert!(
        shown[10] > 4000.0 && shown[10] < 12840.0,
        "ease-out: fast first"
    );
    assert_eq!(ui.frame().element("Score").unwrap().text, "12,840");
    // A new value counts on from where it is.
    let mut ui = ui_with_font(test_font_metrics());
    for _ in 0..20 {
        count_step(&mut ui, 1000.0);
    }
    let saved = ui.save_state();
    let a: Vec<f64> = (0..20).map(|_| count_step(&mut ui, 1000.0)).collect();
    ui.restore_state(&saved).unwrap();
    let b: Vec<f64> = (0..20).map(|_| count_step(&mut ui, 1000.0)).collect();
    assert_eq!(a, b, "a restored count replays exactly");
    assert_eq!(format_count(1234567.891, 2), "1,234,567.89");
}

#[test]
fn a_refused_press_shakes_the_button_and_queues_the_error_sound() {
    let (mut ui, _) = skinned_ui();
    for _ in 0..30 {
        step(&mut ui, InputFrame::default(), true);
    }
    let go = ui.frame().element("Go").expect("the button").clone();
    let (x, y) = go.center();
    let p = Vec2::new(x, y);
    let press = InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    };
    let release = InputFrame {
        pointer: p,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    step(&mut ui, press, true);
    step(&mut ui, release, true);
    let sounds: Vec<UiSound> = ui.take_sounds().into_iter().map(|(e, _)| e).collect();
    assert_eq!(
        sounds,
        vec![UiSound::Error],
        "a disabled button is no press, only a refusal"
    );
    let mut xs = Vec::new();
    for _ in 0..30 {
        step(&mut ui, InputFrame::hover(0.0, 0.0), true);
        xs.push(ui.frame().element("Go").unwrap().x);
    }
    assert!(
        xs.iter().any(|&v| (v - go.x).abs() > 1.0),
        "it shakes: {xs:?}"
    );
    assert!(
        (xs[29] - go.x).abs() < 0.01,
        "and comes to rest where it was"
    );
    // An enabled press sounds once.
    let (mut ui, _) = skinned_ui();
    for _ in 0..30 {
        step(&mut ui, InputFrame::default(), false);
    }
    step(&mut ui, InputFrame::hover(x, y), false);
    step(&mut ui, press_at(x, y), false);
    let sounds: Vec<(UiSound, u64)> = ui.take_sounds();
    assert_eq!(sounds, vec![(UiSound::Press, 7)]);
}

#[test]
fn keyboard_and_gamepad_focus_moves_sound_as_the_pointer_does() {
    // Ways it could fail: a focus move by key or pad is silent (only the
    // pointer reaching a control sounded), activation by Enter or A makes
    // no press sound, or a move that lands where focus already was sounds.
    let file = parse_skin(SKIN).expect("the test skin parses");
    let mut ui = ui_with_font(test_font_metrics());
    let mut sounds = [0u64; 6];
    sounds[UiSound::Hover as usize] = 5;
    sounds[UiSound::Press as usize] = 7;
    ui.set_theme(file.theme(Some(atlas()), sounds));
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.column("Menu", Element::column(), |ui| {
            ui.button("A", "A", ButtonDesc::default());
            ui.button("B", "B", ButtonDesc::default());
        });
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui, InputFrame::default());
    }
    assert!(
        ui.take_sounds().is_empty(),
        "nothing sounds before an input"
    );
    let key = |k: UiKey| InputFrame {
        keys: vec![k],
        ..InputFrame::default()
    };
    let heard =
        |ui: &mut Ui| -> Vec<UiSound> { ui.take_sounds().into_iter().map(|(e, _)| e).collect() };
    build(&mut ui, key(UiKey::Tab));
    assert_eq!(heard(&mut ui), vec![UiSound::Hover], "Tab onto A sounds");
    build(&mut ui, key(UiKey::Down));
    assert_eq!(heard(&mut ui), vec![UiSound::Hover], "Down onto B sounds");
    let pad = |bit: u32| InputFrame {
        gamepad: GamepadFrame {
            pressed: bit,
            ..GamepadFrame::default()
        },
        ..InputFrame::default()
    };
    build(&mut ui, pad(GamepadFrame::UP));
    assert_eq!(
        heard(&mut ui),
        vec![UiSound::Hover],
        "the d-pad onto A sounds"
    );
    build(&mut ui, key(UiKey::Enter));
    assert_eq!(
        heard(&mut ui),
        vec![UiSound::Press],
        "Enter presses A and sounds"
    );
    build(&mut ui, pad(GamepadFrame::A));
    assert_eq!(
        heard(&mut ui),
        vec![UiSound::Press],
        "the pad's A presses and sounds"
    );
    // One focusable control: a move that lands where focus already is stays silent.
    let lone = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.button("Only", "Only", ButtonDesc::default());
        ui.end();
    };
    for _ in 0..2 {
        lone(&mut ui, InputFrame::default());
    }
    lone(&mut ui, key(UiKey::Tab));
    heard(&mut ui);
    lone(&mut ui, key(UiKey::Tab));
    assert!(
        heard(&mut ui).is_empty(),
        "Tab on the only control moves nothing and is silent"
    );
}

/// The `Go` button's top edge over `n` ticks of `input` under `skin`.
fn tops(ui: &mut Ui, input: &InputFrame, n: usize) -> Vec<f32> {
    (0..n)
        .map(|_| {
            step(ui, input.clone(), false);
            ui.frame().element("Go").expect("the button").y
        })
        .collect()
}

#[test]
fn the_hover_lift_and_the_press_move_on_springs() {
    // Ways it could fail: the lift jumps to its end in a tick (no motion),
    // eases on a curve that ignores the skin's spring (no overshoot at a
    // low damping), never settles exactly, settles somewhere else, or
    // differs after a restore; the press squash does not spring back past
    // rest on release.
    let springy = SKIN.replace("  pop 0.12\n", "  pop 0.12\n  hoverSpring 0.3 300\n");
    for (text, bounces) in [(SKIN.to_string(), false), (springy, true)] {
        let file = parse_skin(&text).expect("the skin parses");
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(file.theme(Some(atlas()), [0; 6]));
        for _ in 0..30 {
            step(&mut ui, InputFrame::default(), false);
        }
        let rest = ui.frame().element("Go").unwrap().clone();
        let (x, y) = rest.center();
        let over = InputFrame::hover(x, y);
        let ys = tops(&mut ui, &over, 150);
        let end = ys[149];
        assert!(
            end < rest.y - 1.5,
            "it lifts by the skin's 2 px: {} to {end}",
            rest.y
        );
        assert_eq!(ys[148], end, "and settles exactly: {ys:?}");
        assert!(
            ys[1] > end + 0.5 && ys[1] < rest.y,
            "moving over several ticks, not at once: {ys:?}"
        );
        let lowest = ys.iter().copied().fold(f32::INFINITY, f32::min);
        if bounces {
            assert!(
                lowest < end - 0.3,
                "a damping of 0.3 overshoots the lift: {ys:?}"
            );
        } else {
            assert!(lowest >= end - 1e-4, "damping 1 never overshoots: {ys:?}");
        }
        // A restore mid-motion replays the same positions.
        step(&mut ui, InputFrame::hover(0.0, 0.0), false);
        let saved = ui.save_state();
        let a = tops(&mut ui, &over, 12);
        ui.restore_state(&saved).unwrap();
        let b = tops(&mut ui, &over, 12);
        assert_eq!(a, b, "the spring is UI state");
    }
    // The press: held, the frame squashes towards sy 0.92; released, it
    // springs back past its rest height before settling.
    let file = parse_skin(SKIN).unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    let tex = atlas();
    ui.set_theme(file.theme(Some(tex), [0; 6]));
    for _ in 0..30 {
        step(&mut ui, InputFrame::default(), false);
    }
    let (x, y) = ui.frame().element("Go").unwrap().center();
    // The button's frame: the nine pieces cut from the atlas's button
    // region (u 0.25 to 0.5 of the test atlas); their extent is its height.
    let frame_h = |ui: &Ui| {
        let (mut top, mut bottom) = (f32::INFINITY, f32::NEG_INFINITY);
        for item in &ui.frame().items {
            if let DrawCmd::Image {
                rect,
                texture: Some(t),
                uv,
                ..
            } = &item.cmd
                && *t == tex
                && uv[0] >= 0.25
                && uv[2] <= 0.5
            {
                top = top.min(rect.y);
                bottom = bottom.max(rect.bottom());
            }
        }
        bottom - top
    };
    // Hovered until the lift has settled: the rest height under the pointer.
    for _ in 0..120 {
        step(&mut ui, InputFrame::hover(x, y), false);
    }
    let rest_h = frame_h(&ui);
    step(&mut ui, press_at(x, y), false);
    let first = frame_h(&ui);
    let held = InputFrame {
        pointer: Vec2::new(x, y),
        pointer_inside: true,
        down: true,
        ..InputFrame::default()
    };
    let mut hs = vec![first];
    for _ in 0..20 {
        step(&mut ui, held.clone(), false);
        hs.push(frame_h(&ui));
    }
    assert!(
        first < rest_h && first > rest_h * 0.92 + 0.1,
        "the press starts squashing at once: {rest_h} then {hs:?}"
    );
    let squashed = *hs.last().unwrap();
    assert!(
        (squashed - rest_h * 0.92).abs() < 0.2,
        "and reaches sy 0.92 while held: {hs:?}"
    );
    let release = InputFrame {
        pointer: Vec2::new(x, y),
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    step(&mut ui, release, false);
    let mut back = vec![frame_h(&ui)];
    for _ in 0..30 {
        step(&mut ui, InputFrame::hover(x, y), false);
        back.push(frame_h(&ui));
    }
    let tallest = back.iter().copied().fold(0.0, f32::max);
    assert!(
        tallest > rest_h + 0.1,
        "released, it springs past its rest height: {rest_h} then {back:?}"
    );
    assert!(
        (back[30] - rest_h).abs() < 0.01,
        "and settles at rest: {back:?}"
    );
}

fn press_at(x: f32, y: f32) -> InputFrame {
    InputFrame {
        pointer: Vec2::new(x, y),
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    }
}

/// One tick of a screen whose `Answer` label is shaken when `shake` says so.
fn answer(ui: &mut Ui, shake: Option<ShakeDesc>, reduced: bool) -> f32 {
    let mut t = *ui.theme();
    t.reduced_motion = reduced;
    ui.set_theme(t);
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.label("Answer", "42", LabelDesc::default());
    if let Some(d) = shake {
        ui.shake(d);
    }
    ui.end();
    ui.frame().element("Answer").expect("the label").x
}

#[test]
fn game_code_shakes_an_element_once_and_it_comes_to_rest_by_itself() {
    let (mut ui, _) = skinned_ui();
    let rest = answer(&mut ui, None, false);
    let mut xs = vec![answer(&mut ui, Some(ShakeDesc::default()), false)];
    for _ in 0..40 {
        xs.push(answer(&mut ui, None, false));
    }
    let far = xs.iter().map(|x| (x - rest).abs()).fold(0.0, f32::max);
    assert!(
        far > 2.0 && far <= 6.01,
        "it swings by up to the skin's 6 px with no further call: {xs:?}"
    );
    assert!(
        xs[xs.len() - 1] == rest && ui.take_sounds().is_empty(),
        "it comes to rest where it was, silently by default"
    );
    // A snapshot mid-shake replays the same swings.
    let mut a = skinned_ui().0;
    answer(&mut a, None, false);
    answer(
        &mut a,
        Some(ShakeDesc {
            amount: 10.0,
            sound: true,
            ..ShakeDesc::default()
        }),
        false,
    );
    answer(&mut a, None, false);
    let saved = a.save_state();
    let tail: Vec<f32> = (0..10).map(|_| answer(&mut a, None, false)).collect();
    let mut b = skinned_ui().0;
    b.restore_state(&saved).unwrap();
    let again: Vec<f32> = (0..10).map(|_| answer(&mut b, None, false)).collect();
    assert_eq!(tail, again, "the shake is UI state");
    assert!(
        tail.iter().any(|x| (x - rest).abs() > 6.5),
        "amount 10: {tail:?}"
    );
    assert_eq!(
        a.take_sounds()
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>(),
        vec![UiSound::Error],
        "sound: true queues the error sound"
    );
    // Reduced motion keeps it still; no element before it warns.
    let mut c = skinned_ui().0;
    answer(&mut c, None, true);
    let still: Vec<f32> = std::iter::once(answer(&mut c, Some(ShakeDesc::default()), true))
        .chain((0..10).map(|_| answer(&mut c, None, true)))
        .collect();
    assert!(still.iter().all(|&x| x == rest), "{still:?}");
    c.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    c.shake(ShakeDesc::default());
    let warnings = c.end().warnings.clone();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("shake with no element before it"))
    );
}

/// The atlas pieces a 100×60 panel draws in a skin whose panel slice
/// fills its edges with `fill` (a 32×32 region with 8 px corners: a 16 px
/// middle), as (rect, uv), only those inside the panel.
fn panel_pieces(fill: &str) -> (Rect, Vec<(Rect, [f32; 4])>) {
    let text = format!(
        "type skin\nversion 1\n---\nname \"tiles\"\nbase dark\natlas \"a.png\" 128 64\nslice panel 0 0 32 32 8 tint {fill}\nslice card 0 0 32 32 8 {fill}\n"
    );
    let file = parse_skin(&text).expect("the skin parses");
    let tex = atlas();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(Some(tex), [0; 6]));
    ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
    ui.panel(
        "Box",
        PanelDesc {
            width: Size::Fixed(100.0),
            height: Size::Fixed(60.0),
            ..PanelDesc::default()
        },
        |_| {},
    );
    ui.end();
    let b = ui.frame().element("Box").expect("the panel").clone();
    let r = Rect::new(b.x, b.y, b.width, b.height);
    let pieces = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Image {
                rect,
                uv,
                texture: Some(t),
                ..
            } if *t == tex => Some((*rect, *uv)),
            _ => None,
        })
        .filter(|(p, _)| {
            p.x >= r.x - 0.01
                && p.y >= r.y - 0.01
                && p.right() <= r.right() + 0.01
                && p.bottom() <= r.bottom() + 0.01
        })
        .collect();
    (r, pieces)
}

#[test]
fn a_slice_tiles_its_edges_and_centre_when_the_skin_says_repeat_or_round() {
    let texel = 1.0 / 128.0;
    for fill in ["stretch", "repeat", "round"] {
        let (r, pieces) = panel_pieces(fill);
        let area: f32 = pieces.iter().map(|(p, _)| p.width * p.height).sum();
        assert!(
            (area - r.width * r.height).abs() < 0.5,
            "{fill}: the pieces cover the panel once ({area} of {})",
            r.width * r.height
        );
        // The top edge: between the corners, 8 px tall at the top.
        let mut top: Vec<&(Rect, [f32; 4])> = pieces
            .iter()
            .filter(|(p, _)| {
                (p.y - r.y).abs() < 0.01 && p.x > r.x + 7.99 && p.right() < r.right() - 7.99
            })
            .collect();
        top.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
        let widths: Vec<f32> = top.iter().map(|(p, _)| p.width).collect();
        let uvw: Vec<f32> = top.iter().map(|(_, uv)| (uv[2] - uv[0]) / texel).collect();
        match fill {
            "stretch" => assert_eq!((pieces.len(), widths.clone()), (9, vec![84.0]), "{fill}"),
            "repeat" => {
                // 84 px of edge in 16 px tiles: five whole, then 4 px showing a quarter of the art.
                assert_eq!(widths, vec![16.0, 16.0, 16.0, 16.0, 16.0, 4.0], "{fill}");
                assert!(
                    uvw.iter().zip(&widths).all(|(u, w)| (u - w).abs() < 1e-3),
                    "{fill}: each tile shows the art at its own size: {uvw:?}"
                );
            }
            _ => {
                // round(84 / 16) = 5 whole tiles of 16.8 px.
                assert_eq!(widths.len(), 5, "{fill}: {widths:?}");
                assert!(
                    widths.iter().all(|w| (w - 16.8).abs() < 1e-3)
                        && uvw.iter().all(|u| (u - 16.0).abs() < 1e-3),
                    "{fill}: {widths:?} {uvw:?}"
                );
            }
        }
    }
    assert!(
        parse_skin(
            "type skin\nversion 1\n---\natlas \"a.png\" 128 64\nslice panel 0 0 16 16 8 repeat\n"
        )
        .unwrap_err()
        .contains("no middle"),
        "a slice that is all corners cannot tile"
    );
}

#[test]
fn the_probe_names_each_elements_kind_and_a_disabled_control() {
    let (mut ui, _) = skinned_ui();
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    screen(&mut ui, true);
    ui.icon("Heart", "heart", IconDesc::default());
    ui.button("On", "On", ButtonDesc::default());
    ui.end();
    let f = ui.frame();
    let go = f.element("Menu/Go").expect("the disabled button");
    assert_eq!((go.kind.as_str(), go.disabled), ("button", true));
    let on = f.element("On").expect("an enabled button");
    assert_eq!((on.kind.as_str(), on.disabled), ("button", false));
    assert_eq!(f.element("Menu/Title").unwrap().kind, "label");
    assert_eq!(f.element("Heart").unwrap().kind, "icon");
}
