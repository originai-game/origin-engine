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
