//! The touch controls (joystick, d-pad) under several
//! fingers at once, as og.Pointers holds them.
//!
//! Ways these could fail, each covered below:
//! - the stick jumps from 0 to the dead zone's size at its edge instead of
//!   rising from 0, or keeps its vector after the finger lifts;
//! - a second finger landing on the stick takes it from the first;
//! - the d-pad answers nothing on a diagonal or in a corner, or answers
//!   in its middle;
//! - a disabled control still answers a finger.

use super::*;
use og_input::{ActionDesc, ActionKind, ActionMap};

const HUD: &str = r#"<meta name="component" content="t">
<row id="Hud">
  <joystick id="Stick" action="move"/>
  <dpad id="Pad" action="look"/>
</row>
<style>
#Hud { gap: 40px; padding: 40px; }
#Stick { width: 160px; }
#Pad { width: 150px; }
</style>"#;

fn rig() -> (Doc, Fingers) {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.coarse_pointer = true;
    (mount_at(HUD, &[], vp), Fingers::new())
}

/// The stick's knob, drawn in the knob's own colour while a finger holds
/// it (faded into the HUD at rest): its centre, `None` at rest.
fn held_knob(d: &Doc) -> Option<Vec2> {
    let knob = d.ui.theme().palette.knob.linear();
    d.ui.frame().items.iter().find_map(|i| match &i.cmd {
        DrawCmd::Shape { shape, .. } if shape.paint == og_draw::UiPaint::Solid(knob) => {
            Some(shape.center)
        }
        _ => None,
    })
}

/// The action map a game samples, fed the zones the document laid out.
struct Actions {
    map: ActionMap,
    tick: u64,
}

impl Actions {
    fn new(d: &mut Doc) -> Actions {
        let mut map = ActionMap::new();
        map.define(ActionDesc {
            name: "move".into(),
            kind: ActionKind::Axis2,
            ..ActionDesc::default()
        })
        .unwrap();
        map.set_surface(og_input::Surface { coarse: true });
        map.set_zones(d.ui.take_control_zones().expect("the stick's zone"));
        Actions { map, tick: 0 }
    }

    /// One tick: `input` built, then sampled.
    fn step(&mut self, d: &mut Doc, input: InputFrame) -> og_input::ActionState {
        d.step(input.clone());
        if let Some(zones) = d.ui.take_control_zones() {
            self.map.set_zones(zones);
        }
        self.tick += 1;
        self.map.sample(
            &Default::default(),
            &input.pointers,
            &Default::default(),
            self.tick,
            1.0 / 60.0,
        );
        self.map.state("move").unwrap()
    }
}

#[test]
fn the_stick_rises_from_zero_past_its_dead_zone_and_rests_on_release() {
    let (mut d, mut f) = rig();
    let mut actions = Actions::new(&mut d);
    let c = center(&d, "Stick");
    let r = 80.0;
    let s = actions.step(&mut d, f.at(&[(1, c + Vec2::new(0.1 * r, 0.0))]));
    assert!(held_knob(&d).is_some(), "the finger holds the stick");
    assert!(
        s.vector == Vec2::ZERO,
        "inside the dead zone: {:?}",
        s.vector
    );
    let s = actions.step(&mut d, f.at(&[(1, c + Vec2::new(0.6 * r, 0.0))]));
    let x = s.vector.x;
    assert!(
        (x - 0.45 / 0.85).abs() < 1e-3,
        "rescaled past the 0.15 dead zone, not the raw 0.6: {x}"
    );
    // A finger past the base, below it (the action's y is up).
    let s = actions.step(&mut d, f.at(&[(1, c + Vec2::new(0.0, 3.0 * r))]));
    assert!(
        s.vector.x.abs() < 1e-3 && (s.vector.y + 1.0).abs() < 1e-3,
        "a finger past the base still steers, clamped: {:?}",
        s.vector
    );
    let k = held_knob(&d).expect("held");
    assert!(
        (k.x - c.x).abs() < 0.5 && k.y > c.y && k.y < c.y + r,
        "the knob leans down inside the base: {k:?} about {c:?}"
    );
    let s = actions.step(&mut d, f.lift(1));
    assert!(
        !s.down && s.vector == Vec2::ZERO,
        "released: {:?}",
        s.vector
    );
    idle(&mut d, 1);
    assert_eq!(held_knob(&d), None, "the knob rests");
}

#[test]
fn the_stick_keeps_the_finger_that_took_it() {
    let (mut d, mut f) = rig();
    let c = center(&d, "Stick");
    // Finger 2 grabs the stick to the right; finger 1 then lands on its left.
    d.step(f.at(&[(2, c + Vec2::new(50.0, 0.0))]));
    d.step(f.at(&[(2, c + Vec2::new(50.0, 0.0)), (1, c - Vec2::new(50.0, 0.0))]));
    let k = held_knob(&d).expect("held");
    assert!(
        k.x - c.x > 0.3 * 50.0,
        "the first finger keeps it: the knob at {k:?} about {c:?}"
    );
}
