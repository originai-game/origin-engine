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

use crate::support;

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

/// A dressing's draw lines (docs/ui.md §5, §12.3): stops without an offset
/// share the space between their neighbours, colours are tokens or
/// literals, an atlas region is cut in the atlas's texels, a later block of
/// one selector replaces the fills it sets and keeps the rest, and a line
/// the grammar refuses names its own line.
#[test]
fn a_dressing_draws_with_the_ui_pass_types() {
    use og_ui::dress::{DressMaterial, OrnamentKind};
    use og_ui::paint::*;
    let base = upgrade_style(&v1_style()).unwrap();
    let extra = concat!(
        "dress \"Meter.fill[rarity=epic]\" {\n",
        "  fill primary\n",
        "  fill conic 90 primary \"#ff000080\" 0.2 danger lineStrong reflect\n",
        "  stroke 2 linear 180 primary danger inside dash 6 3 offset 1 round\n",
        "  effect dropShadow 0 4 8 2 \"#000000\"\n",
        "  effect backdrop 12 saturation 1.4 tint panel glass thick elevation lift edge 1 lineStrong\n",
        "  material hologram 0.5 2\n",
        "  blend screen\n",
        "  mask radial \"#ffffffff\" \"#ffffff00\"\n",
        "  ornament corner flourish\n",
        "  sound press \"audio/tick.wav\"\n",
        "}\n",
        "dress \"Meter.fill[rarity=epic]\" {\n",
        "  fill image 8 16 32 32 cover focal 0.25 0.75\n",
        "}\n",
    );
    let file = parse_style(&(base.clone() + extra)).unwrap_or_else(|e| panic!("{e}"));
    let d = file
        .dressings
        .iter()
        .find(|d| d.selector.part == "fill" && !d.selector.attrs.is_empty())
        .expect("the epic meter's dressing");
    // The second block's fill replaced both of the first's.
    let [Paint::Atlas(a)] = &d.draw.fills[..] else {
        panic!("one atlas fill: {:?}", d.draw.fills);
    };
    assert_eq!(a.fit, Fit::Cover);
    assert_eq!(a.focal, [0.25, 0.75]);
    // The first block's other lines stay.
    let [s] = &d.draw.strokes[..] else {
        panic!("one stroke");
    };
    assert_eq!(
        (s.width, s.align, s.dash, s.dash_offset, s.dash_cap),
        (2.0, StrokeAlign::Inside, [6.0, 3.0], 1.0, Cap::Round)
    );
    let Paint::Gradient(g) = &s.paint else {
        panic!("a gradient stroke");
    };
    // 180°: top to bottom.
    assert_eq!((g.from, g.to), ([0.5, 0.0], [0.5, 1.0]));
    assert_eq!(g.stops[1].color, ColorRef::token("danger"));
    assert_eq!(d.draw.effects.len(), 2);
    assert!(matches!(
        &d.draw.effects[1],
        Effect::Backdrop { blur, saturation, brightness, tint, glass }
            if *blur == 12.0 && *saturation == 1.4 && *brightness == 1.0
                && *tint == ColorRef::token("panel")
                && *glass == GlassParts {
                    weight: Some(og_render::UiGlassWeight::Thick),
                    elevation: Some(og_render::UiElevation::Lift),
                    edge: 1.0,
                    edge_color: Some(ColorRef::token("lineStrong")),
                    own_tint: true,
                    ..GlassParts::default()
                }
    ));
    let DressMaterial {
        layer,
        asset,
        params,
    } = d.draw.material.clone().unwrap();
    assert_eq!(layer.material, Material::Builtin(Builtin::Hologram));
    assert_eq!(
        (layer.params[0], asset, params),
        ([0.5, 2.0, 0.0, 0.0], None, 2)
    );
    assert_eq!(d.draw.blend, Some(Blend::Screen));
    assert!(matches!(&d.draw.mask, Some(Mask::Gradient(g)) if g.kind == GradientKind::Radial));
    assert_eq!(
        d.draw.ornaments,
        [OrnamentKind::Corner, OrnamentKind::Flourish]
    );
    assert_eq!(d.draw.sounds.len(), 1);

    // Unset offsets between set ones: 0, 0.2, then 0.6 and 1 evenly.
    let only_conic = concat!(
        "dress \"Meter.fill[rarity=epic]\" {\n",
        "  fill conic 90 primary alpha 0.25 \"#ff000080\" alpha 0.5 0.2 danger lineStrong reflect\n",
        "}\n",
    );
    let file = parse_style(&(base.clone() + only_conic)).unwrap();
    let d = file
        .dressings
        .iter()
        .find(|d| !d.selector.attrs.is_empty())
        .unwrap();
    let [Paint::Gradient(g)] = &d.draw.fills[..] else {
        panic!("a conic fill");
    };
    let offsets: Vec<f32> = g.stops.iter().map(|s| s.offset).collect();
    assert_eq!(offsets.len(), 4);
    for (got, want) in offsets.iter().zip([0.0, 0.2, 0.6, 1.0]) {
        assert!((got - want).abs() < 1e-6, "{offsets:?}");
    }
    // CSS 90° (right) is the paint's angle 0 from +x.
    assert!(g.to[0].abs() < 1e-6, "{:?}", g.to);
    assert_eq!((g.kind, g.spread), (GradientKind::Conic, Spread::Reflect));
    assert_eq!(
        g.stops[0].color,
        ColorRef::Token {
            name: "primary".into(),
            alpha: 0.25
        }
    );
    let ColorRef::Literal(red) = g.stops[1].color else {
        panic!("literal stop")
    };
    assert!((red.alpha - 128.0 / 255.0 * 0.5).abs() < 1e-6);

    // Refused lines report themselves.
    let lines = base.lines().count();
    for (bad, why) in [
        ("fill linear 90 primary", "2 to 8 gradient stops"),
        ("fill notAColour", "not a paint"),
        ("stroke 2 linear 0 primary nope", "not a palette colour"),
        ("stroke 2 primary sideways", "'sideways'"),
        ("effect sparkle 2", "not an effect"),
        ("blend lighten", "not a blend mode"),
        ("fill linear 90 primary 0.8 danger 0.2", "out of order"),
        ("fill primary alpha 1.1", "alpha from 0 to 1"),
    ] {
        let text = format!("{base}dress \"Meter.fill\" {{\n  {bad}\n}}\n");
        let err = parse_style(&text).unwrap_err();
        assert!(err.contains(why), "{bad}: {err}");
        assert!(err.contains(&format!("line {}", lines + 2)), "{bad}: {err}");
    }
}
