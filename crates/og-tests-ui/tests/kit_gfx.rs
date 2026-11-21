//! The game widget set rendered headless and compared with
//! `references/<tier>/widget-set.png` (og-tests' tolerance, as `tests/gfx.rs`;
//! `OG_UI_CAPTURE=1` records). Covers what only drawing shows: checkbox
//! ticks and chevrons as rotated bars; badges; focus rings with their gap;
//! the d-pad and stick; an open select menu over the panel.

use crate::support;

use og_core::glam::Vec2;
use og_render::RenderScene;
use og_ui::*;
use support::*;

#[test]
#[ignore = "requires GPU"]
fn the_widget_set_matches_its_reference_frame() {
    let shot = with_test_font(widget_set_shot);
    let frame = shot.ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert!(
        shot.ui.element("Difficulty.list/List/Option#2").is_some(),
        "the menu is open"
    );

    let rendered = render_frame(&shot.scene, &shot.desc);
    let stats = rendered.stats;
    println!(
        "sprites {} glyphs {} sprite draws {} levels {} missing glyphs {}",
        stats.sprites,
        stats.glyphs,
        stats.sprite_draws,
        frame.levels(),
        stats.missing_glyphs
    );
    assert_eq!(stats.missing_glyphs, 0);

    matches_reference(
        "widget-set",
        &rendered.image,
        &rendered.info,
        Against::OwnTier,
    );
}

/// A stick centred on the surface, the input source of the action `move`.
const STICK_DOC: &str = r#"<meta name="component" content="stick">
<row id="Pad"><joystick id="Stick" action="move"/></row>
<style>#Pad { width: stretch; height: stretch; justify-content: center; align-items: center; }</style>"#;

/// What the stick answers: the action's value (y up, as og-input reads a
/// stick) and whether a press holds it.
#[derive(Debug)]
struct Stick {
    x: f32,
    y: f32,
    active: bool,
}

/// The on-screen stick at rest and held by a finger pushed past its rim up
/// and to the right, 240×240 at 2×, with its answer: its zone read by
/// og-input as the action map reads it, from the press that started on it.
fn stick_shot(font: &FontFiles, push: Option<Vec2>) -> (Shot, Stick) {
    let vp = Viewport {
        width: 240.0,
        height: 240.0,
        dpr: 2.0,
        coarse_pointer: true,
        ..Viewport::default()
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    mount_screen_document(&mut ui, "stick", STICK_DOC, &[]);
    step_screen(&mut ui, InputFrame::default(), vp);
    let mut pointers = og_input::Pointers::default();
    if let Some(push) = push {
        let (x, y) = ui.element("Stick").expect("the stick").center();
        let at = |p: Vec2, pressed: bool| InputFrame {
            pointer: p,
            pointer_inside: true,
            pressed,
            down: true,
            ..InputFrame::default()
        };
        step_screen(&mut ui, at(Vec2::new(x, y), true), vp);
        step_screen(&mut ui, at(Vec2::new(x, y) + push, false), vp);
        step_screen(&mut ui, at(Vec2::new(x, y) + push, false), vp);
        pointers.held = 1;
        pointers.start0 = Vec2::new(x, y);
        pointers.position0 = Vec2::new(x, y) + push;
    }
    let zones = ui.take_control_zones().expect("the stick's zone");
    let [zone] = &zones[..] else {
        panic!("one zone: {zones:?}")
    };
    assert_eq!(zone.action, "move");
    let reading = og_input::control::Reader {
        input: &og_input::Input::default(),
        pointers: &pointers,
        pads: &og_input::Gamepad::default(),
        deadzones: &[og_input::PadDeadzone::default(); og_input::MAX_GAMEPADS],
        surface: og_input::Surface {
            width: vp.width,
            height: vp.height,
            coarse: true,
        },
    }
    .read_zone(zone)
    .0;
    let out = Stick {
        x: reading.value.x,
        y: reading.value.y,
        active: reading.device.is_some(),
    };
    (shot(ui, scene, vp), out)
}

/// A held stick draws its knob pushed toward the finger and held inside
/// the base however far the finger goes, with the rim lit toward the push,
/// against `references/<tier>/stick-held.png`; the frame at rest has its
/// knob centred.
#[test]
#[ignore = "requires GPU"]
fn a_held_stick_draws_its_knob_pushed_inside_its_base() {
    let render = |push: Option<Vec2>| {
        let (shot, out) = with_test_font(|f| stick_shot(f, push));
        let f = shot.ui.frame();
        assert!(f.warnings.is_empty(), "{:?}", f.warnings);
        let base = shot.ui.element("Stick").expect("the stick").clone();
        // The knob's rim disc (the widest round quad but the base) and the arcs.
        // A disc's corner radius is drawn a little under half its width (27.4
        // for a 55 px knob), so round is within half a pixel of it.
        let knob = f
            .items
            .iter()
            .filter_map(|i| match round_fill(&i.cmd) {
                Some((rect, radius))
                    if radius >= rect.width / 2.0 - 0.5 && rect.width < base.width - 1.0 =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .max_by(|a, b| a.width.total_cmp(&b.width))
            .expect("the knob");
        let arcs: Vec<(f32, f32)> = f
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Shape { shape, .. } => match shape.geometry {
                    og_render::UiGeometry::Arc { start, sweep, .. } => Some((start, sweep)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let held = render_frame(&shot.scene, &shot.desc);
        (held, out, base, knob, arcs)
    };
    let (_, still, base, knob, arcs) = render(None);
    let (bx, by) = base.center();
    let off = |k: &og_ui::Rect| Vec2::new(k.x + k.width / 2.0 - bx, k.y + k.height / 2.0 - by);
    assert!(!still.active && still.x == 0.0 && still.y == 0.0);
    assert!(
        off(&knob).length() < 0.01,
        "at rest the knob is centred: {knob:?}"
    );
    assert!(arcs.is_empty(), "at rest no rim is lit");

    // Pushed 100 CSS px up and right: past the 64 px rim, so full.
    let (held, out, _, knob, arcs) = render(Some(Vec2::new(100.0, -100.0)));
    let full = (out.x * out.x + out.y * out.y).sqrt();
    // Up is +y as og-input reads a stick (the widget's own answer was y down).
    assert!(out.active && out.x > 0.6 && out.y > 0.6, "{out:?}");
    assert!((full - 1.0).abs() < 1e-3, "clamped to the rim: {full}");
    let moved = off(&knob);
    let radius = base.width / 2.0;
    assert!(
        moved.x > 20.0 && (moved.x + moved.y).abs() < 0.5,
        "the knob moved {moved} along the push (45°)"
    );
    // The knob leans on the base's 2 px rim and never crosses it.
    let reach = moved.length() + knob.width / 2.0;
    assert!(
        reach <= radius - 2.0 && reach > radius - 4.0,
        "the knob reaches {reach} of the base's {radius}: inside its rim, against it"
    );
    // One arc of the rim, centred on the push.
    let [(start, sweep)] = arcs[..] else {
        panic!("one lit arc: {arcs:?}")
    };
    let mid = start + sweep / 2.0;
    assert!(
        (mid + std::f32::consts::FRAC_PI_4).abs() < 1e-3 && sweep > 0.5,
        "the rim lit toward the push: {start} + {sweep}"
    );
    matches_reference("stick-held", &held.image, &held.info, Against::OwnTier);
}
