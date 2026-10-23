//! Styles, motion and screen transitions, driven tick by tick.
//!
//! Ways these could fail, each covered below:
//! - a glass part stays see-through when the page prefers reduced
//!   transparency (`a_glass_part_draws_the_blurred_frame_under_its_thinned_nine_slice`);
//! - a style's slices never reach the frame (the background stays a plain
//!   quad), or the nine pieces leave gaps or overlap the element's edges;
//! - a frame a few px taller than its corners squeezes its art's whole
//!   middle into its middle row, and the atlas's mips draw a light band
//!   across both ends (`a_frame_smaller_than_its_art_…`);
//! - the default theme starts emitting atlas pieces (it must stay the token
//!   look; the reference frames in tests/gfx.rs and tests/kit_gfx.rs prove
//!   the pixels);
//! - a style file with a typo, a region outside its atlas or a glow without
//!   a soft slice loads anyway instead of naming the line;
//! - a screen transition switches the screen before the cover closes,
//!   never switches, never ends, or draws movement under reduced motion;
//! - a count-up runs on the wall clock, jumps, or differs after a restore;
//! - a refused press does not shake, or UI events with a sound never reach
//!   the host's queue;
//! - a panel's staggered controls refuse presses until their own cascade
//!   ends, long after the panel has arrived (a title screen that ignores
//!   the first half second of taps), or take presses while the panel
//!   itself is still popping in.

mod support;

use og_core::glam::Vec2;
use og_render::{ColorSpace, Mips, RenderScene, TextureData, TextureHandle};
use og_ui::*;
use support::*;

const STYLE: &str = r##"type style
version 1
---
name "test"
scheme dark
atlas "styles/test/atlas.png" 128 64
palette {
  backdrop "#000000"
  well "#111113"
  panel "#203040"
  panelHover "#3a3a3c"
  panelRaised "#2c2c2e"
  line "#38383a"
  lineStrong "#545458"
  ink "#f0f0f0"
  inkMuted "#aeaeb2"
  inkFaint "#98989d"
  inkInverse "#18181a"
  control "#7878803d"
  controlStrong "#7878805c"
  primary "#ffb224"
  primaryHover "#ffc04d"
  primaryActive "#ffd38a"
  onPrimary "#1d1300"
  primarySoft "#ffb22429"
  highlight "#ffb224"
  focus "#ffb224"
  knob "#636366"
  hud "#1c1c1ec7"
  scrim "#0000006b"
  shadow "#000000"
  sheen "#ffffff52"
  success "#3dd68c"
  warn "#ff8a3d"
  danger "#ff8a82"
  info "#5cc8ff"
  health "#3dd68c"
  mana "#0a84ff"
  xp "#bf5af2"
  gold "#ffb224"
}
radius {
  small 4
  control 4
  bar 4
  panel 4
}
slice panel 0 0 32 32 8
slice card 0 0 32 32 8
slice button 32 0 32 32 6 tint
slice fill 64 0 16 16 4
slice track 80 0 16 16 4
soft 96 0 32 32 12
corner 0 32 16 16 16 16 primary 2
corners panel card
shadow card 0 6 10 0 "#00000080"
glow button 4 primary 0.5
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

fn styled_ui() -> (Ui, TextureHandle) {
    let file = parse_style(STYLE).expect("the test style parses");
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
fn a_styled_panel_is_nine_pieces_of_the_atlas_that_tile_its_rectangle() {
    let (mut ui, tex) = styled_ui();
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
fn the_default_theme_draws_no_atlas_pieces() {
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
fn a_style_file_names_the_line_that_is_wrong() {
    let typo = STYLE.replace("slice fill", "slise fill");
    let e = parse_style(&typo).unwrap_err();
    assert!(e.contains("unknown style setting 'slise'"), "{e}");
    let outside = STYLE.replace("slice track 80 0 16 16 4", "slice track 120 0 16 16 4");
    let e = parse_style(&outside).unwrap_err();
    assert!(e.contains("not inside the 128×64 atlas"), "{e}");
    let no_soft = STYLE.replace("soft 96 0 32 32 12\n", "");
    let e = parse_style(&no_soft).unwrap_err();
    assert!(e.contains("needs a `soft` line"), "{e}");
    let colour = STYLE.replace("\"#203040\"", "\"203040\"");
    let e = parse_style(&colour).unwrap_err();
    assert!(e.contains("#rrggbb"), "{e}");
    // A style is complete: no base to fall back on, every role and radius.
    let e = parse_style(&STYLE.replace("  gold \"#ffb224\"\n", "")).unwrap_err();
    assert!(e.contains("leaves out gold"), "{e}");
    let e = parse_style(&STYLE.replace("  bar 4\n", "")).unwrap_err();
    assert!(e.contains("leaves out bar"), "{e}");
    let e = parse_style(&STYLE.replace("type style", "type skin")).unwrap_err();
    assert!(e.contains("now `type style`"), "{e}");
    let e = parse_style(&STYLE.replace("scheme dark", "base dark")).unwrap_err();
    assert!(e.contains("`base` is gone"), "{e}");
    let file = parse_style(STYLE).unwrap();
    assert_eq!(file.theme.palette.panel, Srgba::rgb(0x20, 0x30, 0x40));
    assert_eq!(file.theme.radius.panel, 4.0);
    assert_eq!(file.references(), vec!["styles/test/atlas.png".to_string()]);
}

/// Ids in a style file resolve relative to the file first, then from
/// assets/ (lantern-vale FINDINGS #6: a copied pack named
/// `styles/fantasy/...` and had to be rewritten). Fails if a relative id is
/// read from the root, an absolute one is rewritten, `..` escapes the
/// assets, or the cook and the runtime could disagree (one function).
#[test]
fn a_style_resolves_its_ids_beside_itself_then_from_assets() {
    let assets = [
        "styles/mine/atlas.png",
        "styles/mine/Body.ttf",
        "styles/shared/rope.png",
        "fonts/Noto.ttf",
        "styles/fantasy/atlas.png",
    ];
    let exists = |id: &str| assets.contains(&id);
    let r = |style: &str, id: &str| og_ui::style_file::resolve_reference(style, id, exists);
    assert_eq!(
        r("styles/mine/style.ogt", "atlas.png"),
        "styles/mine/atlas.png"
    );
    assert_eq!(
        r("styles/mine/style.ogt", "./Body.ttf"),
        "styles/mine/Body.ttf"
    );
    assert_eq!(
        r("styles/mine/style.ogt", "../shared/rope.png"),
        "styles/shared/rope.png"
    );
    // Absolute ids still work, also from a copy whose own directory lacks them.
    assert_eq!(
        r("styles/mine/style.ogt", "fonts/Noto.ttf"),
        "fonts/Noto.ttf"
    );
    assert_eq!(
        r("styles/mine/style.ogt", "styles/fantasy/atlas.png"),
        "styles/fantasy/atlas.png"
    );
    // Nothing there: left as written, for the cook to name as missing.
    assert_eq!(r("styles/mine/style.ogt", "nope.png"), "nope.png");
    assert_eq!(
        r("styles/mine/style.ogt", "../../../x.png"),
        "../../../x.png"
    );
    let text = STYLE.replace("\"styles/test/atlas.png\"", "\"atlas.png\"");
    let mut file = parse_style(&text).unwrap();
    file.resolve("styles/test/style.ogt", |id| id == "styles/test/atlas.png");
    assert_eq!(file.references(), vec!["styles/test/atlas.png".to_string()]);
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
    let (mut ui, _) = styled_ui();
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
    let (mut ui, _) = styled_ui();
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
    let file = parse_style(STYLE).expect("the test style parses");
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

/// The `Go` button's top edge over `n` ticks of `input` under `style`.
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
    // eases on a curve that ignores the style's spring (no overshoot at a
    // low damping), never settles exactly, settles somewhere else, or
    // differs after a restore; the press squash does not spring back past
    // rest on release.
    let springy = STYLE.replace("  pop 0.12\n", "  pop 0.12\n  hoverSpring 0.3 300\n");
    for (text, bounces) in [(STYLE.to_string(), false), (springy, true)] {
        let file = parse_style(&text).expect("the style parses");
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
            "it lifts by the style's 2 px: {} to {end}",
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
    let file = parse_style(STYLE).unwrap();
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
    let (mut ui, _) = styled_ui();
    let rest = answer(&mut ui, None, false);
    let mut xs = vec![answer(&mut ui, Some(ShakeDesc::default()), false)];
    for _ in 0..40 {
        xs.push(answer(&mut ui, None, false));
    }
    let far = xs.iter().map(|x| (x - rest).abs()).fold(0.0, f32::max);
    assert!(
        far > 2.0 && far <= 6.01,
        "it swings by up to the style's 6 px with no further call: {xs:?}"
    );
    assert!(
        xs[xs.len() - 1] == rest && ui.take_sounds().is_empty(),
        "it comes to rest where it was, silently by default"
    );
    // A snapshot mid-shake replays the same swings.
    let mut a = styled_ui().0;
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
    let mut b = styled_ui().0;
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
    let mut c = styled_ui().0;
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

/// The atlas pieces a 100×60 panel draws in a style whose panel slice
/// fills its edges with `fill` (a 32×32 region with 8 px corners: a 16 px
/// middle), as (rect, uv), only those inside the panel.
fn panel_pieces(fill: &str) -> (Rect, Vec<(Rect, [f32; 4])>) {
    sliced_pieces(&format!("0 0 32 32 8 tint {fill}"), 100.0, 60.0)
}

/// The atlas pieces inside a `w`×`h` panel whose slice is `slice panel
/// <slice>` in a 128×64 atlas, as (rect, uv).
fn sliced_pieces(slice: &str, w: f32, h: f32) -> (Rect, Vec<(Rect, [f32; 4])>) {
    // The test style's palette and radius blocks.
    let values = &STYLE[STYLE.find("palette {").unwrap()..STYLE.find("slice ").unwrap()];
    let text = format!(
        "type style\nversion 1\n---\nname \"tiles\"\natlas \"a.png\" 128 64\n{values}slice panel {slice}\nslice card {slice}\n"
    );
    let file = parse_style(&text).expect("the style parses");
    let tex = atlas();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(Some(tex), [0; 6]));
    ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
    ui.panel(
        "Box",
        PanelDesc {
            width: Size::Fixed(w),
            height: Size::Fixed(h),
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
fn a_slice_tiles_its_edges_and_centre_when_the_style_says_repeat_or_round() {
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
        parse_style(
            "type style\nversion 1\n---\natlas \"a.png\" 128 64\nslice panel 0 0 16 16 8 repeat\n"
        )
        .unwrap_err()
        .contains("no middle"),
        "a slice that is all corners cannot tile"
    );
}

#[test]
fn a_frame_smaller_than_its_art_tiles_exactly_and_never_squeezes_its_middle_past_twice() {
    // A 48 texel region with 12 texel corners: a 24 texel middle.
    let texel = [1.0 / 128.0, 1.0 / 64.0];
    let slice = "0 0 48 48 12 tint";
    // Rumpus Isle's small Pause: 28 px tall, a 4 px middle row; then an
    // element smaller than its two corners both ways; then one taller
    // than its art, whose middle stretches as before.
    for (w, h) in [(64.0, 28.0), (16.0, 10.0), (64.0, 60.0)] {
        let (r, pieces) = sliced_pieces(slice, w, h);
        // Exact tiling: the areas sum to the element's and no two overlap.
        let area: f32 = pieces.iter().map(|(p, _)| p.width * p.height).sum();
        assert!(
            (area - r.width * r.height).abs() < 0.01,
            "{w}x{h}: the pieces cover the element once ({area} of {}): {pieces:?}",
            r.width * r.height
        );
        for (i, (a, _)) in pieces.iter().enumerate() {
            for (b, _) in &pieces[i + 1..] {
                let ox = a.right().min(b.right()) - a.x.max(b.x);
                let oy = a.bottom().min(b.bottom()) - a.y.max(b.y);
                assert!(ox <= 0.01 || oy <= 0.01, "{w}x{h}: {a:?} overlaps {b:?}");
            }
        }
        // Along the middle row and column (between the corners), no piece
        // shows more than twice its length of art: further, the atlas's
        // mips blur an edge's border into its fill (the band). Corners
        // scaled to fit squeeze theirs, as they must.
        for (p, uv) in &pieces {
            let mid_x = p.x > r.x + 0.01 && p.right() < r.right() - 0.01;
            let mid_y = p.y > r.y + 0.01 && p.bottom() < r.bottom() - 0.01;
            let sx = (uv[2] - uv[0]) / texel[0] / p.width;
            let sy = (uv[3] - uv[1]) / texel[1] / p.height;
            assert!(
                (!mid_x || sx <= 2.0 + 1e-3) && (!mid_y || sy <= 2.0 + 1e-3),
                "{w}x{h}: {p:?} squeezes its middle art {sx}x{sy}"
            );
        }
    }
    // The tall element's middle row still shows the whole 24 texel middle.
    let (r, pieces) = sliced_pieces(slice, 64.0, 60.0);
    let left = pieces
        .iter()
        .find(|(p, _)| (p.x - r.x).abs() < 0.01 && (p.y - r.y - 12.0).abs() < 0.01)
        .expect("the left edge");
    assert!(
        ((left.1[3] - left.1[1]) / texel[1] - 24.0).abs() < 1e-3,
        "{left:?}"
    );
}

#[test]
fn the_probe_names_each_elements_kind_and_a_disabled_control() {
    let (mut ui, _) = styled_ui();
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

/// A style's `glass <part> <blur> <fill> <tint>` line (here the card a
/// panel is dressed as): the part draws a
/// glass panel (og-render's blurred frame) of its own rectangle and
/// radius under its nine-slice, tinted by its fill colour, and its
/// nine-slice at `control` of its opacity; a style without it draws no glass.
///
/// Ways it could fail: the line is not read, the glass is drawn over the
/// panel's frame (a later painter level) or at another rectangle, or the
/// frame keeps its full opacity and hides the glass.
#[test]
fn a_glass_part_draws_the_blurred_frame_under_its_thinned_nine_slice() {
    let frame_of = |style: &str, reduced_transparency: bool| {
        let file = parse_style(style).expect("the style parses");
        let tex = atlas();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(file.theme(Some(tex), [0; 6]));
        let vp = Viewport {
            reduced_transparency,
            ..Viewport::new(800.0, 600.0)
        };
        for _ in 0..30 {
            ui.begin(InputFrame::default(), vp);
            screen(&mut ui, false);
            ui.end();
        }
        let menu = ui.frame().element("Menu").expect("the panel").clone();
        let glass: Vec<(i32, Rect, f32, f32, [f32; 4])> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Glass {
                    rect,
                    radius,
                    blur,
                    tint,
                    ..
                } => Some((i.level, *rect, *radius, *blur, *tint)),
                _ => None,
            })
            .collect();
        // The panel's centre piece: the atlas image filling most of it, inside
        // it (the shadow's soft centre lies 6 px lower).
        let centre = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Image {
                    rect,
                    texture: Some(t),
                    color,
                    ..
                } if *t == tex
                    && rect.width > menu.width * 0.5
                    && rect.height > menu.height * 0.5
                    && rect.x >= menu.x - 0.01
                    && rect.y >= menu.y - 0.01
                    && rect.right() <= menu.x + menu.width + 0.01
                    && rect.bottom() <= menu.y + menu.height + 0.01 =>
                {
                    Some((i.level, color[3]))
                }
                _ => None,
            })
            .next()
            .expect("the panel's centre piece");
        (menu, glass, centre, ui.theme().radius.panel)
    };
    let (_, none, (_, solid), _) = frame_of(STYLE, false);
    assert!(none.is_empty(), "no glass without the line");
    let glassy = STYLE.replace("glow button", "glass card 16 0.5 0.2\nglow button");
    let (menu, glass, (level, thin), card_radius) = frame_of(&glassy, false);
    assert_eq!(glass.len(), 1, "one glass panel: {glass:?}");
    let (glevel, rect, radius, blur, tint) = glass[0];
    assert!(
        (rect.x - menu.x).abs() < 0.01
            && (rect.y - menu.y).abs() < 0.01
            && (rect.width - menu.width).abs() < 0.01
            && (rect.height - menu.height).abs() < 0.01,
        "the glass is the panel's rectangle: {rect:?} against {menu:?}"
    );
    assert_eq!(
        (radius, blur),
        (card_radius, 16.0),
        "the card's radius and the line's blur"
    );
    assert!((tint[3] - 0.2).abs() < 1e-6, "the line's tint share");
    assert!(glevel < level, "the glass is under the panel's frame");
    assert!(
        (thin - solid * 0.5).abs() < 1e-3,
        "the frame is drawn at the line's fill: {thin} against {solid}"
    );
    // The page prefers reduced transparency: no blur, the frame whole (an
    // opaque surface), as without the glass line.
    let (_, opaque, (_, whole), _) = frame_of(&glassy, true);
    assert!(opaque.is_empty(), "no glass under reduced transparency");
    assert!(
        (whole - solid).abs() < 1e-6,
        "the frame at full opacity: {whole} against {solid}"
    );
}

/// One tick of a styled screen panel: a title and four buttons. Answers
/// whether `Last` was clicked.
fn menu_step(ui: &mut Ui, input: InputFrame) -> bool {
    menu_step_as(ui, input, false)
}

/// [`menu_step`] with the panel's `instant`.
fn menu_step_as(ui: &mut Ui, input: InputFrame, instant: bool) -> bool {
    ui.begin(input, Viewport::new(800.0, 600.0));
    let mut clicked = false;
    ui.anchor(
        "Menu",
        AnchorDesc {
            anchor: Anchor::Center,
            direction: Direction::Column,
            gap: 8.0,
            background: Some(Srgba::rgb(32, 48, 64)),
            instant,
            ..AnchorDesc::default()
        },
        |ui| {
            ui.label("Title", "Menu", LabelDesc::default());
            for id in ["First", "Second", "Third"] {
                ui.button(id, id, ButtonDesc::default());
            }
            clicked = ui.button("Last", "Last", ButtonDesc::default()).clicked;
        },
    );
    ui.end();
    clicked
}

fn rect_of(ui: &Ui, id: &str) -> (f32, f32, f32, f32) {
    let e = ui.frame().element(id).expect("the element");
    (e.x, e.y, e.width, e.height)
}

#[test]
fn a_staggered_panels_controls_take_presses_once_the_panel_has_arrived() {
    // Ways it could fail: the stagger holds each control inert until its
    // own place in the cascade has played (the last button of a title
    // screen ignores taps for twice the entrance), or no control is
    // inert even while the panel pops in.
    let (mut settled, _) = styled_ui();
    for _ in 0..120 {
        menu_step(&mut settled, InputFrame::default());
    }
    let (panel_end, last_end) = (rect_of(&settled, "Menu"), rect_of(&settled, "Last"));
    // The first tick the panel stands where it ends while `Last` is still
    // moving into place: the cascade outlasts the panel's own entrance.
    let (mut ui, _) = styled_ui();
    let mut arrived = None;
    for tick in 0..120 {
        menu_step(&mut ui, InputFrame::default());
        if rect_of(&ui, "Menu") == panel_end {
            arrived = Some((tick, rect_of(&ui, "Last")));
            break;
        }
    }
    let (tick, last_now) = arrived.expect("the panel arrives");
    assert!(
        last_now != last_end,
        "the cascade is still moving Last when the panel has arrived (tick {tick}): {last_now:?}"
    );
    // A press on Last there, released a tick later, is a click.
    let (x, y) = (last_now.0 + last_now.2 / 2.0, last_now.1 + last_now.3 / 2.0);
    let release = InputFrame {
        pointer: Vec2::new(x, y),
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    assert!(
        !menu_step(&mut ui, press_at(x, y)) && menu_step(&mut ui, release),
        "a press on Last right after the panel arrived (tick {tick}) clicks it"
    );
    // While the panel itself pops in, the same press is refused.
    let (mut early, _) = styled_ui();
    menu_step(&mut early, InputFrame::default());
    let r = rect_of(&early, "Last");
    let (x, y) = (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
    let pressed = menu_step(&mut early, press_at(x, y));
    let clicked = menu_step(
        &mut early,
        InputFrame {
            pointer: Vec2::new(x, y),
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        },
    );
    assert!(
        rect_of(&early, "Menu") != panel_end && !pressed && !clicked,
        "a press on Last while the panel pops in is not taken"
    );
}

#[test]
fn an_instant_panel_is_settled_and_answering_on_its_first_tick_and_enters_when_it_comes_back() {
    // Ways it could fail: `instant` is ignored (the opening title behind the
    // web's play gate stays part-way through the style's entrance and
    // stagger, examples/WEB-FINDINGS.md #1), it only skips the panel's own
    // entrance and leaves the cascade, or it sticks to the panel so a title
    // the player comes back to never enters again.
    let (mut settled, _) = styled_ui();
    for _ in 0..120 {
        menu_step(&mut settled, InputFrame::default());
    }
    let (panel_end, last_end) = (rect_of(&settled, "Menu"), rect_of(&settled, "Last"));
    let (mut ui, _) = styled_ui();
    menu_step_as(&mut ui, InputFrame::default(), true);
    assert_eq!(
        (rect_of(&ui, "Menu"), rect_of(&ui, "Last")),
        (panel_end, last_end),
        "the first tick of an instant panel is its settled frame"
    );
    let (x, y) = (last_end.0 + last_end.2 / 2.0, last_end.1 + last_end.3 / 2.0);
    let release = InputFrame {
        pointer: Vec2::new(x, y),
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    assert!(
        !menu_step_as(&mut ui, press_at(x, y), true) && menu_step_as(&mut ui, release, true),
        "Last clicks on the ticks right after it appeared"
    );
    // Gone for a tick (the player left the title), then back without
    // `instant`: the style's entrance plays again.
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.end();
    menu_step_as(&mut ui, InputFrame::default(), false);
    assert_ne!(
        rect_of(&ui, "Menu"),
        panel_end,
        "a panel that comes back enters with the style"
    );
    // Without `instant` the first tick is the entrance's first (the old behaviour).
    let (mut plain, _) = styled_ui();
    menu_step(&mut plain, InputFrame::default());
    assert_ne!(rect_of(&plain, "Last"), last_end, "a plain panel enters");
}
