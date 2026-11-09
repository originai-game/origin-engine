//! Retained style metadata and tick-driven widget motion. Counters and
//! hover springs replay after restoration; token aliases reject cycles.
//! Part painting and CSS state transitions are tested in dressing.rs.

use crate::support;

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
    assert_eq!(ui.element("Score").unwrap().text, "12,840");
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

/// The `Go` button's top edge over `n` ticks of `input` under `style`.
fn tops(ui: &mut Ui, input: &InputFrame, n: usize) -> Vec<f32> {
    (0..n)
        .map(|_| {
            step(ui, input.clone(), false);
            ui.element("Go").expect("the button").y
        })
        .collect()
}

#[test]
fn the_hover_lift_moves_on_springs_and_replays() {
    // Ways it could fail: the lift jumps to its end in a tick (no motion),
    // eases on a curve that ignores the style's spring (no overshoot at a
    // low damping), never settles exactly, settles somewhere else, or
    // differs after a restore.
    let springy = STYLE.replace("  pop 0.12\n", "  pop 0.12\n  hoverSpring 0.3 300\n");
    for (text, bounces) in [(STYLE.to_string(), false), (springy, true)] {
        let file = parse_style(&text).expect("the style parses");
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(file.theme(Some(atlas()), [0; 6]));
        for _ in 0..30 {
            step(&mut ui, InputFrame::default(), false);
        }
        let rest = ui.element("Go").unwrap().clone();
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
}

/// The test style with what `og style upgrade` must carry over besides:
/// a button state's own frame and shadow (version 1 shares a state's
/// shadow with its class), a tile over the panel, a measured tone, a
/// sound and a press spring.
fn v1_style() -> String {
    STYLE.replace(
        "soft 96 0 32 32 12\n",
        "soft 96 0 32 32 12\nslice buttonHover 32 0 32 32 6 tint\ntone buttonHover 0.5 1\nshadow buttonHover 0 2 0 0 \"#00000080\"\ntile 0 32 16 16 16 16 0.3 2 panel\nsound press \"audio/press.wav\"\n",
    )
    .replace("  pop 0.12\n", "  pop 0.12\n  pressSpring 0.55 120\n")
}

#[test]
fn metadata_tokens_support_aliases_and_reject_cycles() {
    let base = upgrade_style(&v1_style()).unwrap();
    let theme = parse_style(&base).unwrap().theme;
    let start = base.find("--radius-sm:").unwrap();
    let end = start + base[start..].find('\n').unwrap();
    let mut cycle = base.clone();
    cycle.replace_range(start..end, "--radius-sm: var(--radius-sm)");
    assert!(
        parse_style(&cycle)
            .unwrap_err()
            .contains("cyclic token --radius-sm")
    );
    let mut alias = base;
    alias.replace_range(start..end, "--radius-sm: var(--space-lg)");
    assert_eq!(
        parse_style(&alias).unwrap().theme.tokens.radius[tokens::RadiusStep::Sm.index()],
        theme.tokens.space[tokens::SpaceStep::Lg.index()]
    );
}
