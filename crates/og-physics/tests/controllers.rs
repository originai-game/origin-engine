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

use og_core::glam::{Quat, Vec2, Vec3};
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
fn a_character_walks_at_its_speed_stays_on_the_ground_and_jumps_once_per_press() {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let c = walker(&mut w, 0.0, 0.05, 0.0);
    run(&mut w, 20);
    assert!(character(&w, c).grounded, "{:?}", character(&w, c));
    let y0 = transform(&w, c).position.y;
    assert!((y0 - 0.92).abs() < 0.05, "standing at {y0}");
    set_character(&mut w, c, |c| c.movement = Vec3::new(3.0, 0.0, 0.0));
    run(&mut w, 60);
    let p = transform(&w, c).position;
    assert!((p.x - 3.0).abs() < 0.1, "walked to {p}");
    assert!(character(&w, c).grounded);
    assert!((character(&w, c).velocity.x - 3.0).abs() < 0.05);
    // Jump: held for a second, it jumps once.
    set_character(&mut w, c, |c| {
        c.movement = Vec3::ZERO;
        c.jump = true;
    });
    w.step().unwrap();
    assert!(!character(&w, c).grounded);
    let mut top = 0.0f32;
    let mut jumps = 0;
    for _ in 0..90 {
        w.step().unwrap();
        top = top.max(transform(&w, c).position.y);
        jumps += events(&w)
            .iter()
            .filter(|e| e.kind == CHARACTER_JUMPED)
            .count();
    }
    // v² / 2g = 36 / 19.62 = 1.83 m above standing height.
    assert!(
        (top - y0 - 1.83).abs() < 0.1,
        "the jump peaked {} up",
        top - y0
    );
    assert_eq!(jumps, 0, "holding jump does not jump again");
    assert!(character(&w, c).grounded, "it landed");
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

#[test]
fn a_character_rides_a_moving_platform_and_pushes_crates() {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let platform = spawn_body(
        &mut w,
        at(0.0, 1.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::new(3.0, 0.2, 3.0)),
    );
    let c = walker(&mut w, 0.0, 1.15, 0.0);
    run(&mut w, 20);
    assert!(character(&w, c).grounded);
    assert_eq!(character(&w, c).ground, platform.to_bits());
    // The platform moves 2 m along +x over a second (by its transform).
    let start = transform(&w, c).position;
    for i in 1..=60 {
        w.insert(platform, at(i as f32 / 30.0, 1.0, 0.0)).unwrap();
        w.step().unwrap();
    }
    let moved = transform(&w, c).position - start;
    assert!((moved.x - 2.0).abs() < 0.1, "carried {moved}");
    assert!(moved.y.abs() < 0.05, "{moved}");

    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let crate_ = spawn_body(
        &mut w,
        at(2.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let c = walker(&mut w, 0.0, 0.05, 0.0);
    run(&mut w, 10);
    set_character(&mut w, c, |c| c.movement = Vec3::new(2.0, 0.0, 0.0));
    run(&mut w, 90);
    assert!(
        transform(&w, crate_).position.x > 3.0,
        "the crate was pushed to {}",
        transform(&w, crate_).position
    );
}

#[test]
fn a_top_down_character_moves_in_the_plane_and_slides_along_walls() {
    let (mut w, _p) = world(
        Physics {
            profile: PROFILE_2D,
            gravity: Vec3::ZERO,
            ..Physics::default()
        },
        1,
    );
    spawn_static(
        &mut w,
        at(2.0, 0.0, 0.0),
        Collider::cuboid(Vec3::new(0.5, 10.0, 1.0)),
    );
    let c = w
        .spawn((
            at(0.0, 0.0, 0.0),
            Collider::sphere(0.4),
            CharacterController {
                gravity_scale: 0.0,
                movement: Vec3::new(2.0, 1.0, 0.0),
                ..CharacterController::default()
            },
        ))
        .unwrap();
    run(&mut w, 120);
    let p = transform(&w, c).position;
    assert!(p.x < 1.4 && p.x > 1.2, "held by the wall at {p}");
    assert!(p.y > 1.8, "slid along it to {p}");
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
fn a_platformer_jumps_to_its_jump_height_in_its_time_to_apex_and_a_tap_jumps_lower() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let full = hero(&mut w, 0.0, 0.0, PlatformerController::default());
    let tap = hero(&mut w, 5.0, 0.0, PlatformerController::default());
    run(&mut w, 10);
    assert!(platformer(&w, full).grounded);
    let y0 = transform(&w, full).position.y;
    for e in [full, tap] {
        set_platformer(&mut w, e, |c| c.jump = true);
    }
    let (mut top_full, mut top_tap, mut apex_tick) = (0.0f32, 0.0f32, 0);
    for tick in 1..=90 {
        if tick == 6 {
            set_platformer(&mut w, tap, |c| c.jump = false);
        }
        w.step().unwrap();
        let y = transform(&w, full).position.y;
        if y > top_full {
            top_full = y;
            apex_tick = tick;
        }
        top_tap = top_tap.max(transform(&w, tap).position.y);
    }
    let h = top_full - y0;
    // jumpHeight 3.2 with apex hang (0.6 gravity near the top) lands a bit above it.
    assert!(h > 3.1 && h < 3.6, "a full jump rose {h}");
    let t = apex_tick as f32 / 60.0;
    assert!(t > 0.36 && t < 0.5, "the apex came at {t} s");
    let low = top_tap - y0;
    assert!(low < h * 0.6, "a tap rose {low} against {h}");
    assert!(platformer(&w, full).grounded && platformer(&w, tap).grounded);
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
        set_platformer(&mut w, e, |c| c.move_x = 1.0);
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

#[test]
fn one_way_platforms_are_solid_from_above_and_down_drops_through() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    // A one-way platform (layer 2) 2 m up, right above the hero.
    let shelf = spawn_static(
        &mut w,
        at(0.0, 2.0, 0.0),
        Collider::cuboid(Vec3::new(4.0, 0.2, 1.0)).on_layers(2, u32::MAX),
    );
    let e = hero(
        &mut w,
        0.0,
        0.0,
        PlatformerController {
            one_way_layers: 2,
            ..PlatformerController::default()
        },
    );
    run(&mut w, 5);
    set_platformer(&mut w, e, |c| c.jump = true);
    run(&mut w, 80);
    let p = platformer(&w, e);
    let y = transform(&w, e).position.y;
    assert!(
        p.grounded && (y - 2.91).abs() < 0.05,
        "jumped up through and stands on it at {y}"
    );
    assert_eq!(p.ground, shelf.to_bits());
    set_platformer(&mut w, e, |c| {
        c.jump = false;
        c.down = true;
    });
    run(&mut w, 60);
    let y = transform(&w, e).position.y;
    assert!(y < 1.0, "dropped through to {y}");
}

#[test]
fn a_wall_slows_the_fall_and_a_wall_jump_pushes_away() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    spawn_static(
        &mut w,
        at(1.0, 10.0, 0.0),
        Collider::cuboid(Vec3::new(1.0, 20.0, 1.0)),
    );
    let e = hero(&mut w, 0.0, 8.0, PlatformerController::default());
    set_platformer(&mut w, e, |c| c.move_x = 1.0);
    let mut sliding = false;
    let mut touched = false;
    for _ in 0..40 {
        w.step().unwrap();
        let p = platformer(&w, e);
        // The slide limit holds from the tick after it touched the wall.
        if p.on_wall == 1 && touched {
            sliding = true;
            assert!(p.velocity.y >= -3.01, "sliding at {}", p.velocity.y);
        }
        touched = p.on_wall == 1;
    }
    assert!(sliding, "it reached the wall");
    let x0 = transform(&w, e).position.x;
    set_platformer(&mut w, e, |c| c.jump = true);
    w.step().unwrap();
    assert!(
        events(&w).iter().any(|ev| ev.kind == CHARACTER_WALL_JUMPED),
        "{:?}",
        events(&w)
    );
    run(&mut w, 8);
    let p = transform(&w, e).position;
    assert!(p.x < x0 - 0.6, "pushed off the wall to {p}");
    assert!(platformer(&w, e).velocity.y > 0.0);
}

#[test]
fn corner_correction_slides_a_jump_past_a_ceiling_corner() {
    let run_jump = |correction: f32| {
        let (mut w, _p) = world2d();
        ground2d(&mut w);
        // A ceiling block whose left edge is 0.1 m right of the hero's right side.
        spawn_static(
            &mut w,
            at(0.3 + 1.0, 3.0, 0.0),
            Collider::cuboid(Vec3::new(2.0, 0.5, 1.0)),
        );
        let e = hero(
            &mut w,
            0.0,
            0.0,
            PlatformerController {
                corner_correction: correction,
                ..PlatformerController::default()
            },
        );
        run(&mut w, 5);
        set_platformer(&mut w, e, |c| c.jump = true);
        let mut top = 0.0f32;
        for _ in 0..40 {
            w.step().unwrap();
            top = top.max(transform(&w, e).position.y);
            if std::env::var_os("OG_DEBUG_CORNER").is_some() {
                eprintln!(
                    "{} {:?}",
                    transform(&w, e).position,
                    platformer(&w, e).velocity
                );
            }
        }
        top
    };
    let bumped = run_jump(0.0);
    let corrected = run_jump(0.15);
    assert!(
        bumped < 2.0,
        "without correction it hit the ceiling at {bumped}"
    );
    assert!(
        corrected > 3.2,
        "with correction it slid past, up to {corrected}"
    );
}

#[test]
fn a_platformer_rides_a_moving_platform() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let platform = spawn_body(
        &mut w,
        at(0.0, 2.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::new(3.0, 0.2, 1.0)),
    );
    let e = hero(&mut w, 0.0, 2.15, PlatformerController::default());
    run(&mut w, 10);
    assert_eq!(platformer(&w, e).ground, platform.to_bits());
    let x0 = transform(&w, e).position.x;
    for i in 1..=60 {
        w.insert(platform, at(i as f32 / 40.0, 2.0 + i as f32 / 60.0, 0.0))
            .unwrap();
        w.step().unwrap();
    }
    let p = transform(&w, e).position;
    assert!((p.x - x0 - 1.5).abs() < 0.08, "carried to {p}");
    assert!((p.y - 3.91).abs() < 0.08, "lifted to {p}");
    assert!(platformer(&w, e).grounded);
}

/// A platformer running right at 6.4 m/s into a 1 m crate of `mass` kg two
/// metres ahead for a second: how far the crate went, and where the hero
/// stopped.
fn push_run(mass: f32, push_bodies: bool) -> (f32, f32) {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let crate_ = spawn_body(
        &mut w,
        at(2.0, 0.5, 0.0),
        RigidBody {
            mass,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::ONE),
    );
    let e = hero(
        &mut w,
        0.0,
        0.0,
        PlatformerController {
            max_speed: 6.4,
            push_bodies,
            ..PlatformerController::default()
        },
    );
    run(&mut w, 10);
    let x0 = transform(&w, crate_).position.x;
    set_platformer(&mut w, e, |c| c.move_x = 1.0);
    run(&mut w, 60);
    (
        transform(&w, crate_).position.x - x0,
        transform(&w, e).position.x,
    )
}

#[test]
fn a_platformer_shoves_a_light_crate_and_not_a_heavy_one_nor_any_without_push_bodies() {
    // 2 kg: shoved ahead at about the run speed (0.3 s to reach the crate).
    let (light, _) = push_run(2.0, true);
    assert!(light > 2.5, "a 2 kg crate went {light} m");
    // A tonne: friction holds more than pushStrength's 1000 N.
    let (heavy, hero_x) = push_run(1000.0, true);
    assert!(heavy.abs() < 0.05, "a 1000 kg crate went {heavy} m");
    assert!(
        hero_x < 1.2,
        "the hero stopped at the heavy crate, at {hero_x}"
    );
    // 100 kg: 1000 N beats its 490 N of friction, slowly.
    let (middle, _) = push_run(100.0, true);
    assert!(
        middle > 0.05 && middle < light * 0.5,
        "a 100 kg crate went {middle} m (2 kg: {light})"
    );
    // pushBodies false: even the light one stays.
    let (none, hero_x) = push_run(2.0, false);
    assert!(none.abs() < 0.01, "pushBodies false moved it {none} m");
    assert!(hero_x < 1.2, "the hero stopped at the crate, at {hero_x}");
}

#[test]
fn an_impulse_knocks_a_platformer_back_once() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let e = hero(&mut w, 0.0, 0.0, PlatformerController::default());
    run(&mut w, 10);
    assert!(platformer(&w, e).grounded);
    // A hit from the right: back and up, taken on the next step.
    set_platformer(&mut w, e, |c| c.impulse = Vec2::new(-6.0, 7.0));
    w.step().unwrap();
    let c = platformer(&w, e);
    assert_eq!(c.impulse, Vec2::ZERO, "consumed");
    assert!(!c.grounded && c.velocity.y > 6.0, "{c:?}");
    // Air deceleration (25 m/s²) from 6 m/s: the first tick keeps 5.58.
    assert!((c.velocity.x + 6.0 - 25.0 / 60.0).abs() < 1e-3, "{c:?}");
    assert!(
        events(&w).iter().any(|ev| ev.kind == CHARACTER_LEFT_GROUND),
        "{:?}",
        events(&w)
    );
    let mut landed = None;
    for tick in 0..120 {
        w.step().unwrap();
        if platformer(&w, e).grounded {
            landed = Some(tick);
            break;
        }
    }
    let x = transform(&w, e).position.x;
    assert!(
        landed.is_some() && x < -0.5 && x > -3.0,
        "landed {landed:?} at x {x}"
    );
    // Taken once: standing still afterwards.
    run(&mut w, 30);
    assert!(platformer(&w, e).velocity.x.abs() < 1e-6);
}

#[test]
fn a_platformer_rides_a_platform_moving_by_its_velocity() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let platform = w
        .spawn((
            at(0.0, 2.0, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(3.0, 0.2, 1.0)),
            Velocity {
                linear: Vec3::new(2.0, 0.0, 0.0),
                angular: Vec3::ZERO,
            },
        ))
        .unwrap();
    let e = hero(&mut w, 0.0, 2.15, PlatformerController::default());
    run(&mut w, 10);
    assert_eq!(platformer(&w, e).ground, platform.to_bits());
    let (x0, p0) = (
        transform(&w, e).position.x,
        transform(&w, platform).position.x,
    );
    run(&mut w, 60);
    let moved = transform(&w, platform).position.x - p0;
    assert!((moved - 2.0).abs() < 1e-3, "the platform moved {moved}");
    let carried = transform(&w, e).position.x - x0;
    assert!((carried - 2.0).abs() < 0.05, "carried {carried}");
    assert!(platformer(&w, e).grounded);
}

// ---- collision layers -----------------------------------------------------

/// Walls across x 3 that a controller on layer 1 with `mask` walks into
/// from x 0: one on layer 2 colliding with everything, one on layer 4
/// whose mask leaves out layer 1, one on layer 1 colliding with everything.
fn layered_walls(w: &mut World, z: f32, size: Vec3) -> [Entity; 3] {
    [(2u32, u32::MAX), (4, !1), (1, u32::MAX)].map(|(layers, mask)| {
        spawn_static(
            w,
            at(3.0 + layers as f32 * 2.0, size.y * 0.5, z),
            Collider::cuboid(size).on_layers(layers, mask),
        )
    })
}

#[test]
fn a_character_walks_through_colliders_its_layers_do_not_meet() {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    // Walls at x 7, 11 and 5 (by layer): masked out by the character (2),
    // masking the character out (4), and solid to it (1).
    layered_walls(&mut w, 0.0, Vec3::new(0.4, 3.0, 4.0));
    let c = w
        .spawn((
            at(0.0, 0.95, 0.0),
            Collider::capsule(0.4, 1.0).on_layers(1, !2),
            CharacterController::default(),
        ))
        .unwrap();
    run(&mut w, 10);
    set_character(&mut w, c, |c| c.movement = Vec3::new(3.0, 0.0, 0.0));
    run(&mut w, 120);
    // Through the wall at 7? Not yet: the one at 5 (layer 1) stops it.
    let p = transform(&w, c).position;
    assert!(p.x > 4.3 && p.x < 4.6, "stopped by the layer-1 wall: {p}");
    // Without the layer-1 wall it walks through the other two.
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let [two, four, one] = layered_walls(&mut w, 0.0, Vec3::new(0.4, 3.0, 4.0));
    let _ = (two, four);
    w.despawn(one).unwrap();
    let c = w
        .spawn((
            at(0.0, 0.95, 0.0),
            Collider::capsule(0.4, 1.0).on_layers(1, !2),
            CharacterController::default(),
        ))
        .unwrap();
    run(&mut w, 10);
    set_character(&mut w, c, |c| c.movement = Vec3::new(3.0, 0.0, 0.0));
    run(&mut w, 300);
    let p = transform(&w, c).position;
    assert!(p.x > 14.0, "walked through the walls it does not meet: {p}");
    assert!(character(&w, c).grounded);
}

#[test]
fn a_platformer_runs_through_colliders_its_layers_do_not_meet() {
    let (mut w, _p) = world2d();
    ground2d(&mut w);
    let walls = layered_walls(&mut w, 0.0, Vec3::new(0.4, 3.0, 1.0));
    let solid = |w: &mut World| {
        w.spawn((
            at(0.0, 0.0, 0.0),
            Collider {
                offset: Vec3::new(0.0, 0.625, 0.0),
                ..Collider::cuboid(Vec3::new(0.7, 1.25, 1.0))
            }
            .on_layers(1, !2),
            PlatformerController::default(),
        ))
        .unwrap()
    };
    let e = solid(&mut w);
    run(&mut w, 10);
    set_platformer(&mut w, e, |c| c.move_x = 1.0);
    run(&mut w, 120);
    let p = transform(&w, e).position;
    assert!(p.x > 4.4 && p.x < 4.6, "stopped by the layer-1 wall: {p}");
    w.despawn(walls[2]).unwrap();
    run(&mut w, 120);
    let p = transform(&w, e).position;
    assert!(p.x > 14.0, "ran through the walls it does not meet: {p}");
    assert!(platformer(&w, e).grounded);
}

/// A walker pressing at 2 m/s into the gap between two 40 kg balls (it
/// slides off one into the other, so both are hit each tick): the tick
/// both are first pushed, and the state 40 ticks on, bit for bit.
fn wedge_run() -> ([Vec3; 2], Vec<u32>) {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let ball = |w: &mut World, x: f32| {
        let body = RigidBody {
            mass: 40.0,
            ..RigidBody::dynamic()
        };
        let e = spawn_body(w, at(x, 0.5, 1.0), body, Collider::sphere(0.5));
        w.insert(e, Velocity::default()).unwrap();
        e
    };
    let balls = [ball(&mut w, -0.55), ball(&mut w, 0.55)];
    let c = walker(&mut w, 0.0, 0.05, 0.0);
    run(&mut w, 10);
    set_character(&mut w, c, |c| c.movement = Vec3::new(0.0, 0.0, 2.0));
    let mut first = None;
    for _ in 0..40 {
        w.step().unwrap();
        let v = balls.map(|b| velocity(&w, b).linear);
        if first.is_none() && v.iter().any(|v| v.length() > 1e-4) {
            first = Some(v);
        }
    }
    let mut bits = Vec::new();
    for e in [balls[0], balls[1], c] {
        let t = transform(&w, e);
        bits.extend(t.position.to_array().map(f32::to_bits));
        bits.extend(t.quaternion.to_array().map(f32::to_bits));
    }
    for b in balls {
        bits.extend(velocity(&w, b).linear.to_array().map(f32::to_bits));
    }
    (first.expect("the walker reached the balls"), bits)
}

#[test]
fn two_bodies_walked_into_at_once_share_the_push() {
    let ([a, b], bits) = wedge_run();
    // Each asks 0.8 × 40 kg × its closing speed, well past the tick's
    // 1000 N × 1/60 s, so each gets half: before, the first hit took all of
    // it and the other none (a 0.335 m/s, b 0).
    assert!(
        a.length() > 0.05 && b.length() > 0.05,
        "both balls are pushed the first tick: {a} and {b}"
    );
    let ratio = a.length() / b.length();
    assert!(
        (0.9..1.1).contains(&ratio),
        "the two balls share alike: {a} and {b}"
    );
    let taken = 40.0 * (a.length() + b.length());
    assert!(
        taken <= 1000.0 / 60.0 * 1.001,
        "the tick's pushes sum to at most pushStrength × dt: {taken} N·s"
    );
    // Determinism: the same scenario again, bit for bit.
    assert_eq!(wedge_run().1, bits, "a second run ends elsewhere");
}
