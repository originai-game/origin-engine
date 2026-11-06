//! The touch controls (joystick, d-pad, action button) under several
//! fingers at once, as og.Pointers holds them.
//!
//! Ways these could fail, each covered below:
//! - the stick jumps from 0 to the dead zone's size at its edge instead of
//!   rising from 0, or keeps its vector after the finger lifts;
//! - a second finger landing on the stick takes it from the first;
//! - a thumb steering the stick fires a button by sliding over it;
//! - the d-pad answers nothing on a diagonal or in a corner, or answers
//!   in its middle;
//! - a disabled control still answers a finger.

use super::*;

#[derive(Default)]
struct Pad {
    stick: JoystickResponse,
    dpad: DpadResponse,
    fire: Response,
    disabled: bool,
}

fn controls(ui: &mut Ui, s: &mut Pad) {
    ui.row(
        "Hud",
        Element {
            gap: 40.0,
            padding: Insets::all(40.0),
            ..Element::row()
        },
        |ui| {
            s.stick = ui.joystick(
                "Stick",
                JoystickDesc {
                    size: 160.0,
                    disabled: s.disabled,
                    ..JoystickDesc::default()
                },
            );
            s.dpad = ui.dpad(
                "Pad",
                DpadDesc {
                    size: 150.0,
                    disabled: s.disabled,
                    ..DpadDesc::default()
                },
            );
            s.fire = ui.action_button(
                "Fire",
                "Fire",
                &ActionButtonDesc {
                    size: 80.0,
                    disabled: s.disabled,
                    ..ActionButtonDesc::default()
                },
            );
        },
    );
}

fn rig() -> (Rig<Pad>, Fingers) {
    let mut rig = Rig::new(Pad::default());
    rig.vp.coarse_pointer = true;
    rig.idle(2, &mut controls);
    (rig, Fingers::new())
}

#[test]
fn the_stick_rises_from_zero_past_its_dead_zone_and_rests_on_release() {
    let (mut rig, mut f) = rig();
    let c = rig.center("Hud/Stick");
    let r = 80.0;
    rig.step(f.at(&[(1, c + Vec2::new(0.1 * r, 0.0))]), &mut controls);
    let s = rig.state.stick;
    assert!(s.active && s.x == 0.0, "inside the dead zone: {s:?}");
    rig.step(f.at(&[(1, c + Vec2::new(0.6 * r, 0.0))]), &mut controls);
    let x = rig.state.stick.x;
    assert!(
        (x - 0.45 / 0.85).abs() < 1e-3,
        "rescaled past the 0.15 dead zone, not the raw 0.6: {x}"
    );
    rig.step(f.at(&[(1, c + Vec2::new(0.0, 3.0 * r))]), &mut controls);
    let s = rig.state.stick;
    assert!(
        s.x.abs() < 1e-3 && (s.y - 1.0).abs() < 1e-3,
        "a finger past the base still steers, clamped: {s:?}"
    );
    rig.step(f.lift(1), &mut controls);
    assert_eq!(rig.state.stick, JoystickResponse::default(), "released");
}

#[test]
fn the_stick_keeps_its_finger_and_a_sliding_thumb_never_fires() {
    let (mut rig, mut f) = rig();
    let (c, fire) = (rig.center("Hud/Stick"), rig.center("Hud/Fire"));
    // Finger 2 grabs the stick to the right; finger 1 then lands on its left.
    rig.step(f.at(&[(2, c + Vec2::new(50.0, 0.0))]), &mut controls);
    rig.step(
        f.at(&[(2, c + Vec2::new(50.0, 0.0)), (1, c - Vec2::new(50.0, 0.0))]),
        &mut controls,
    );
    assert!(
        rig.state.stick.x > 0.3,
        "the first finger keeps it: {:?}",
        rig.state.stick
    );
    // Finger 1 slides from the stick onto Fire and lifts there.
    rig.step(
        f.at(&[(2, c + Vec2::new(50.0, 0.0)), (1, fire)]),
        &mut controls,
    );
    assert!(
        !rig.state.fire.pressed,
        "a thumb sliding over does not hold it"
    );
    rig.step(f.lift(1), &mut controls);
    assert!(!rig.state.fire.clicked, "nor click it as it lifts");
    // A finger that lands on Fire holds it beside the stick, and taps it.
    rig.step(
        f.at(&[(2, c + Vec2::new(50.0, 0.0)), (3, fire)]),
        &mut controls,
    );
    assert!(rig.state.fire.pressed && rig.state.stick.x > 0.3);
    rig.step(f.lift(3), &mut controls);
    assert!(rig.state.fire.clicked, "lifting over it clicks it");
}
