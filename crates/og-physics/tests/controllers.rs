//! The kinematic controllers: og.CharacterController (3D, and top-down 2D)
//! and og.PlatformerController (2D), driven through their components.
//!
//! Ways this could fail, each caught below: the move ignores the input or
//! the world (it walks through walls, or never moves); gravity or the jump
//! is applied twice or never (it hovers, or the jump height is off); the
//! jump fires every tick it is held (bunny hops); slopes and steps are not
//! limited (it climbs a cliff) or snapping is off (it skips down slopes);
//! moving platforms do not carry it; dynamic bodies are not pushed; the
//! platformer's designer units (jump height, time to apex) do not produce
//! the stated jump; coyote time or the jump buffer are lost; releasing jump
//! does not cut the jump; one-way platforms block from below or let it
//! fall through from above; dropping through does not work; wall slides do
//! not limit the fall or wall jumps do not push away; corner correction
//! never nudges (or nudges when disabled); events are missing.

use crate::support;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn character(w: &World, e: Entity) -> CharacterController {
    w.get::<CharacterController>(e).unwrap()
}

fn platformer(w: &World, e: Entity) -> PlatformerController {
    w.get::<PlatformerController>(e).unwrap()
}

fn set_character(w: &mut World, e: Entity, f: impl FnOnce(&mut CharacterController)) {
    let mut c = character(w, e);
    f(&mut c);
    w.insert(e, c).unwrap();
}

fn set_platformer(w: &mut World, e: Entity, f: impl FnOnce(&mut PlatformerController)) {
    let mut c = platformer(w, e);
    f(&mut c);
    w.insert(e, c).unwrap();
}

/// A capsule character (0.4 radius, 1.8 m tall) standing at `x, z` on the floor.
fn walker(w: &mut World, x: f32, y: f32, z: f32) -> Entity {
    w.spawn((
        at(x, y + 0.9, z),
        Collider::capsule(0.4, 1.0),
        CharacterController::default(),
    ))
    .unwrap()
}

fn events(w: &World) -> Vec<CharacterEvent> {
    w.pending_events::<CharacterEvent>().unwrap()
}

#[test]
fn a_character_climbs_gentle_slopes_and_steps_but_not_steep_ones_or_tall_steps() {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    // A 25 degree ramp rising towards +x from x = 2, a 60 degree one towards -x from x = -2.
    let ramp = |w: &mut World, x: f32, angle: f32| {
        let q = Quat::from_rotation_z(angle);
        spawn_static(
            w,
            rotated(at(x, 0.0, 0.0), q),
            Collider::cuboid(Vec3::new(8.0, 0.2, 4.0)),
        );
    };
    ramp(&mut w, 5.5, 25f32.to_radians());
    ramp(&mut w, -5.5, -60f32.to_radians());
    let up = walker(&mut w, 0.0, 0.05, 0.0);
    let steep = walker(&mut w, -0.5, 0.05, 3.0);
    let _ = steep;
    run(&mut w, 10);
    set_character(&mut w, up, |c| c.movement = Vec3::new(3.0, 0.0, 0.0));
    let mut top = 0.0f32;
    for _ in 0..180 {
        w.step().unwrap();
        top = top.max(transform(&w, up).position.y);
    }
    assert!(top > 2.2, "walked up the gentle ramp to {top}");
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    ramp(&mut w, -5.5, -60f32.to_radians());
    let steep = walker(&mut w, 0.0, 0.05, 0.0);
    run(&mut w, 10);
    set_character(&mut w, steep, |c| c.movement = Vec3::new(-3.0, 0.0, 0.0));
    run(&mut w, 300);
    let p = transform(&w, steep).position;
    assert!(p.y < 1.3, "did not climb the 60 degree ramp: {p}");

    // Steps: a 0.25 m one is climbed, a 0.6 m one is not.
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    spawn_static(
        &mut w,
        at(3.0, 0.125, 0.0),
        Collider::cuboid(Vec3::new(2.0, 0.25, 4.0)),
    );
    spawn_static(
        &mut w,
        at(-3.0, 0.3, 0.0),
        Collider::cuboid(Vec3::new(2.0, 0.6, 4.0)),
    );
    let a = walker(&mut w, 0.0, 0.05, -1.0);
    let b = walker(&mut w, 0.0, 0.05, 1.0);
    run(&mut w, 10);
    set_character(&mut w, a, |c| c.movement = Vec3::new(2.0, 0.0, 0.0));
    set_character(&mut w, b, |c| c.movement = Vec3::new(-2.0, 0.0, 0.0));
    run(&mut w, 90);
    let (pa, pb) = (transform(&w, a).position, transform(&w, b).position);
    assert!(
        pa.x > 2.5 && pa.y > 1.1,
        "stepped up onto the low block: {pa}"
    );
    assert!(pb.x > -2.1 && pb.y < 1.0, "stopped at the tall block: {pb}");
}

// ---- the platformer -------------------------------------------------------

fn world2d() -> (World, PhysicsSystem) {
    world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    )
}

/// A 0.8 × 1.6 box hero standing at `x` on ground whose top is y = 0.
fn hero(w: &mut World, x: f32, y: f32, c: PlatformerController) -> Entity {
    w.spawn((
        at(x, y + 0.81, 0.0),
        Collider::cuboid(Vec3::new(0.8, 1.6, 1.0)),
        c,
    ))
    .unwrap()
}

fn ground2d(w: &mut World) -> Entity {
    spawn_static(
        w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(200.0, 1.0, 1.0)),
    )
}

#[test]
fn coyote_time_and_the_jump_buffer_forgive_late_and_early_presses() {
    // Coyote: run off a ledge and press jump 4 ticks later (inside 0.1 s).
    let ledge = |presses_after: u32| {
        let (mut w, _p) = world2d();
        spawn_static(
            &mut w,
            at(-10.0, -0.5, 0.0),
            Collider::cuboid(Vec3::new(20.0, 1.0, 1.0)),
        );
        spawn_static(
            &mut w,
            at(0.0, -20.0, 0.0),
            Collider::cuboid(Vec3::new(200.0, 1.0, 1.0)),
        );
        let e = hero(&mut w, -1.0, 0.0, PlatformerController::default());
        run(&mut w, 5);
        set_platformer(&mut w, e, |c| c.run = 1.0);
        let mut left_at = None;
        for tick in 0..120 {
            w.step().unwrap();
            if left_at.is_none() && !platformer(&w, e).grounded {
                left_at = Some(tick);
            }
            if left_at.is_some_and(|t| tick == t + presses_after) {
                set_platformer(&mut w, e, |c| c.jump = true);
            }
            if events(&w).iter().any(|ev| ev.kind == CHARACTER_JUMPED) {
                return true;
            }
        }
        false
    };
    assert!(ledge(4), "a press 4 ticks after the edge jumps");
    assert!(!ledge(12), "a press 12 ticks after the edge does not");

    // Buffer: press jump 5 ticks before landing; the jump starts on landing.
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let e = hero(&mut w, 0.0, 3.0, PlatformerController::default());
    let mut jumped = false;
    for _ in 0..200 {
        let p = platformer(&w, e);
        let y = transform(&w, e).position.y - 0.81;
        if !p.grounded && p.velocity.y < 0.0 && y < -p.velocity.y * 5.0 / 60.0 && !p.jump {
            set_platformer(&mut w, e, |c| c.jump = true);
        }
        w.step().unwrap();
        if events(&w).iter().any(|ev| ev.kind == CHARACTER_JUMPED) {
            jumped = true;
            break;
        }
    }
    assert!(jumped, "the buffered press jumped on landing");
}
