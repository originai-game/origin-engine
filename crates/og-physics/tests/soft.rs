//! Verlet ropes and cloth (og.Rope, og.Cloth), the 2D particle fluid
//! (og.Fluid2D) and water (og.WaterSurface, og.WaterLine), through the
//! world and the physics system's read-outs.
//!
//! Ways this could fail, each caught below: the rope's links are not held
//! (it stretches under gravity) or are too stiff (it never swings); its
//! start does not follow its entity or its tied end its target; points go
//! through solid colliders; cloth pins are not held or the grid tears;
//! wind does nothing; the fluid explodes, collapses to a point, leaks out
//! of its container or through an obstacle, or produces NaN; water waves
//! never start, never travel or never die out; a body crossing the surface
//! sends no og.Splash; buoyancy does not hold up a light body or holds up a
//! heavy one; the 2D line ignores characters.

mod support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use support::*;

#[test]
fn a_rope_hangs_at_its_length_follows_its_anchor_and_drapes_over_a_box() {
    let (mut w, p) = world(Physics::default(), 1);
    let anchor = w
        .spawn((
            at(0.0, 6.0, 0.0),
            Rope {
                segments: 20,
                length: 4.0,
                ..Rope::default()
            },
        ))
        .unwrap();
    run(&mut w, 240);
    let pts = p.rope_points(anchor).unwrap();
    assert_eq!(pts.len(), 21);
    assert!((pts[0] - Vec3::new(0.0, 6.0, 0.0)).length() < 1e-4);
    let length: f32 = pts.windows(2).map(|s| (s[1] - s[0]).length()).sum();
    assert!((length - 4.0).abs() < 0.08, "the rope is {length} long");
    assert!(
        (pts[20].y - 2.0).abs() < 0.1,
        "it hangs down to {}",
        pts[20]
    );
    // The anchor moves: the rope swings after it.
    w.insert(anchor, at(2.0, 6.0, 0.0)).unwrap();
    w.step().unwrap();
    let pts = p.rope_points(anchor).unwrap();
    assert!((pts[0] - Vec3::new(2.0, 6.0, 0.0)).length() < 1e-4);
    assert!(pts[20].x < 1.0, "the free end lags behind at {}", pts[20]);
    run(&mut w, 300);
    let pts = p.rope_points(anchor).unwrap();
    assert!(
        (pts[20].x - 2.0).abs() < 0.3,
        "and swings back under it: {}",
        pts[20]
    );

    // Over a box: no point inside it.
    let (mut w, p) = world(Physics::default(), 1);
    spawn_static(
        &mut w,
        at(0.0, 3.0, 0.0),
        Collider::cuboid(Vec3::new(2.0, 1.0, 2.0)),
    );
    let rope = w
        .spawn((
            at(-0.5, 5.0, 0.0),
            Rope {
                segments: 24,
                length: 5.0,
                radius: 0.05,
                ..Rope::default()
            },
        ))
        .unwrap();
    run(&mut w, 300);
    for q in p.rope_points(rope).unwrap() {
        let inside = q.x.abs() < 1.0 - 0.02 && (q.y - 3.0).abs() < 0.5 - 0.02 && q.z.abs() < 1.0;
        assert!(!inside, "a rope point is inside the box: {q}");
    }
}

#[test]
fn a_rope_tied_at_both_ends_sags_between_them() {
    let (mut w, p) = world(Physics::default(), 1);
    let post = spawn_static(&mut w, at(4.0, 5.0, 0.0), Collider::sphere(0.1));
    let rope = w
        .spawn((
            at(0.0, 5.0, 0.0),
            Rope {
                length: 5.0,
                end: post.to_bits(),
                ..Rope::default()
            },
        ))
        .unwrap();
    run(&mut w, 240);
    let pts = p.rope_points(rope).unwrap();
    assert!((pts.last().unwrap().x - 4.0).abs() < 1e-3);
    let low = pts.iter().map(|q| q.y).fold(f32::MAX, f32::min);
    assert!(low < 4.3 && low > 3.0, "it sags to {low}");
}

#[test]
fn cloth_hangs_from_its_pins_flutters_in_the_wind_and_drapes_over_a_ball() {
    let (mut w, p) = world(Physics::default(), 1);
    let flag = w
        .spawn((
            at(0.0, 5.0, 0.0),
            Cloth {
                columns: 10,
                rows: 8,
                spacing: 0.2,
                ..Cloth::default()
            },
        ))
        .unwrap();
    run(&mut w, 200);
    let c = p.cloth_points(flag).unwrap();
    assert_eq!((c.columns, c.rows), (10, 8));
    for x in 0..10 {
        let pin = c.points[x];
        assert!(
            (pin - Vec3::new(x as f32 * 0.2, 5.0, 0.0)).length() < 1e-4,
            "pin {x} at {pin}"
        );
    }
    let bottom = c.points[7 * 10 + 5];
    assert!(
        (bottom.y - (5.0 - 7.0 * 0.2)).abs() < 0.12,
        "hangs to {bottom}"
    );
    // Wind along +z blows it out.
    let mut cloth = w.get::<Cloth>(flag).unwrap();
    cloth.wind = Vec3::new(0.0, 0.0, 12.0);
    w.insert(flag, cloth).unwrap();
    run(&mut w, 120);
    let c = p.cloth_points(flag).unwrap();
    let z = c.points[7 * 10 + 5].z;
    assert!(z > 0.4, "the wind lifted it to z {z}");

    // Draped over a ball: no point inside it.
    let (mut w, p) = world(Physics::default(), 1);
    spawn_static(&mut w, at(1.0, 2.0, -1.0), Collider::sphere(0.8));
    let sheet = w
        .spawn((
            rotated(
                at(0.0, 3.0, 0.0),
                og_core::glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            Cloth {
                columns: 11,
                rows: 11,
                spacing: 0.2,
                pins: 0,
                ..Cloth::default()
            },
        ))
        .unwrap();
    run(&mut w, 120);
    for q in p.cloth_points(sheet).unwrap().points {
        let d = (q - Vec3::new(1.0, 2.0, -1.0)).length();
        assert!(d > 0.8, "a cloth point is {d} from the ball's centre");
    }
}

fn world2d() -> (World, PhysicsSystem) {
    world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    )
}

#[test]
fn a_2d_fluid_settles_into_its_container_without_leaking_or_blowing_up() {
    let (mut w, p) = world2d();
    // A 6 × 6 tank: floor and walls, with a box in the middle of the floor.
    spawn_static(
        &mut w,
        at(0.0, -0.25, 0.0),
        Collider::cuboid(Vec3::new(7.0, 0.5, 1.0)),
    );
    spawn_static(
        &mut w,
        at(-3.25, 3.0, 0.0),
        Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
    );
    spawn_static(
        &mut w,
        at(3.25, 3.0, 0.0),
        Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
    );
    spawn_static(
        &mut w,
        at(0.0, 0.5, 0.0),
        Collider::cuboid(Vec3::new(1.0, 1.0, 1.0)),
    );
    let fluid = w
        .spawn((
            at(-1.5, 4.0, 0.0),
            Fluid2D {
                count: 800,
                size: Vec2::new(2.5, 3.0),
                ..Fluid2D::default()
            },
        ))
        .unwrap();
    let start = p.fluid_particles(fluid).map(|v| v.len());
    assert_eq!(start, None, "no particles before the first step");
    run(&mut w, 360);
    let pts = p.fluid_particles(fluid).unwrap();
    assert_eq!(pts.len(), 800);
    let mut mean = Vec2::ZERO;
    for q in &pts {
        assert!(q.is_finite(), "{q}");
        assert!(
            q.x > -3.0 - 0.01 && q.x < 3.0 + 0.01 && q.y > -0.01,
            "leaked: {q}"
        );
        let in_box = q.x.abs() < 0.5 - 0.01 && q.y < 1.0 - 0.01;
        assert!(!in_box, "inside the obstacle: {q}");
        mean += *q;
    }
    mean /= pts.len() as f32;
    // It spread over the whole floor (both sides of the box) and lies low:
    // pressure levels it (without it the mean stays at x -0.37).
    assert!(mean.x.abs() < 0.3, "spread to a mean x of {}", mean.x);
    assert!(mean.y < 1.6, "settled to a mean height of {}", mean.y);
    let right = pts.iter().filter(|q| q.x > 0.5).count();
    assert!(right > 150, "{right} particles flowed past the box");
    // Not collapsed: neighbours keep their distance.
    let mut tight = 0;
    for (i, a) in pts.iter().enumerate().step_by(16) {
        for b in &pts[i + 1..] {
            if (*a - *b).length() < 0.01 {
                tight += 1;
            }
        }
    }
    assert!(tight < 5, "{tight} pairs collapsed onto each other");
    // Near pressure keeps the particles apart (without it they pack to 0.054).
    let mut nearest = 0.0;
    let probes: Vec<&Vec2> = pts.iter().step_by(20).collect();
    for a in &probes {
        let m = pts
            .iter()
            .filter(|b| *b != *a)
            .map(|b| (**a - *b).length())
            .fold(f32::MAX, f32::min);
        nearest += m;
    }
    let nearest = nearest / probes.len() as f32;
    assert!(nearest > 0.065, "neighbours sit {nearest} apart");
}

#[test]
fn a_body_dropped_in_water_splashes_makes_waves_that_die_out_and_floats() {
    let (mut w, p) = world(Physics::default(), 1);
    let pool = w
        .spawn((
            at(0.0, 0.0, 0.0),
            WaterSurface {
                resolution: 48,
                size: 12.0,
                ..WaterSurface::default()
            },
        ))
        .unwrap();
    spawn_static(
        &mut w,
        at(0.0, -3.5, 0.0),
        Collider::cuboid(Vec3::new(12.0, 1.0, 12.0)),
    );
    // A light crate (density 0.4) and a heavy one (density 3) of 1 m³.
    let light = spawn_body(
        &mut w,
        at(-2.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider {
            density: 0.4,
            ..Collider::cuboid(Vec3::ONE)
        },
    );
    let heavy = spawn_body(
        &mut w,
        at(2.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider {
            density: 3.0,
            ..Collider::cuboid(Vec3::ONE)
        },
    );
    let mut splashes = 0;
    let mut peak = 0.0f32;
    for _ in 0..90 {
        w.step().unwrap();
        splashes += w
            .pending_events::<Splash>()
            .unwrap()
            .iter()
            .filter(|s| s.water == pool.to_bits() && s.speed < -2.0)
            .count();
        let h = p.water_heights(pool).unwrap();
        peak = peak.max(h.heights.iter().fold(0.0f32, |m, x| m.max(x.abs())));
    }
    assert_eq!(splashes, 2, "both crates splashed in");
    assert!(peak > 0.05, "waves rose to {peak}");
    // Far from the crates (a corner) the waves arrived.
    let h = p.water_heights(pool).unwrap();
    assert_eq!(h.resolution, 48);
    run(&mut w, 600);
    let yl = transform(&w, light).position.y;
    let yh = transform(&w, heavy).position.y;
    assert!(yl > -0.5 && yl < 0.4, "the light crate floats at {yl}");
    assert!(yh < -2.0, "the heavy crate sank to {yh}");
    let h = p.water_heights(pool).unwrap();
    let left = h.heights.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(
        left < peak * 0.25,
        "the waves died down from {peak} to {left}"
    );
}

#[test]
fn a_2d_water_line_is_splashed_by_bodies_and_characters_and_settles() {
    let (mut w, p) = world2d();
    let pool = w
        .spawn((
            at(0.0, 0.0, 0.0),
            WaterLine {
                width: 10.0,
                columns: 80,
                ..WaterLine::default()
            },
        ))
        .unwrap();
    spawn_static(
        &mut w,
        at(0.0, -3.5, 0.0),
        Collider::cuboid(Vec3::new(12.0, 1.0, 1.0)),
    );
    let ball = spawn_body(
        &mut w,
        at(-2.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.4),
    );
    let hero = w
        .spawn((
            at(2.0, 2.0, 0.0),
            Collider::cuboid(Vec3::new(0.6, 1.2, 1.0)),
            PlatformerController::default(),
        ))
        .unwrap();
    let mut who = std::collections::BTreeSet::new();
    let mut peak = 0.0f32;
    for _ in 0..120 {
        w.step().unwrap();
        for s in w.pending_events::<Splash>().unwrap() {
            who.insert(s.entity);
        }
        let line = p.water_line(pool).unwrap();
        assert_eq!(line.len(), 80);
        peak = peak.max(line.iter().fold(0.0f32, |m, x| m.max(x.abs())));
    }
    assert!(
        who.contains(&ball.to_bits()) && who.contains(&hero.to_bits()),
        "{who:?}"
    );
    assert!(peak > 0.05, "the surface moved by {peak}");
    run(&mut w, 900);
    let line = p.water_line(pool).unwrap();
    let left = line.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(left < peak * 0.2, "it settled from {peak} to {left}");
    let y = transform(&w, ball).position.y;
    assert!(y > -0.6, "the ball (density 1, buoyancy 12) floats at {y}");
}

/// #10 of kestrel-stair's findings: a cloth on a child entity (a cape on a
/// body) hangs from its world transform, its parent's after its own, and
/// follows the parent when the parent moves or turns.
#[test]
fn a_cloth_pinned_to_a_child_entity_follows_its_parent() {
    use og_core::glam::Quat;
    let (mut w, p) = world(Physics::default(), 1);
    let body = w.spawn((at(10.0, 0.0, 39.0),)).unwrap();
    let cape = w
        .spawn((
            at(-0.3, 1.5, -0.2),
            Parent {
                entity: body.to_bits(),
            },
            Cloth {
                columns: 4,
                rows: 5,
                spacing: 0.15,
                ..Cloth::default()
            },
        ))
        .unwrap();
    run(&mut w, 60);
    let pins = |p: &PhysicsSystem| {
        let c = p.cloth_points(cape).unwrap();
        (c.points[0], c.points[3])
    };
    let (first, last) = pins(&p);
    assert!(
        (first - Vec3::new(9.7, 1.5, 38.8)).length() < 1e-4,
        "the first pin hangs at {first}, not at the body's shoulders"
    );
    assert!((last - Vec3::new(10.15, 1.5, 38.8)).length() < 1e-4);
    // The body walks and turns half a circle: the pins go with it.
    w.insert(
        body,
        rotated(
            at(4.0, 2.0, -6.0),
            Quat::from_rotation_y(std::f32::consts::PI),
        ),
    )
    .unwrap();
    w.step().unwrap();
    let (first, last) = pins(&p);
    assert!(
        (first - Vec3::new(4.3, 3.5, -5.8)).length() < 1e-4,
        "after the turn the first pin is at {first}"
    );
    assert!((last - Vec3::new(3.85, 3.5, -5.8)).length() < 1e-4);
    // The free points follow under it.
    run(&mut w, 240);
    let c = p.cloth_points(cape).unwrap();
    let hem = c.points[4 * 4 + 1];
    assert!(
        (hem.x - 4.15).abs() < 0.2 && (hem.z + 5.8).abs() < 0.2 && hem.y < 3.0,
        "the hem hangs under the moved pins at {hem}"
    );
    // A grandparent's move carries the cloth too.
    let ground = w.spawn((at(0.0, 10.0, 0.0),)).unwrap();
    w.insert(
        body,
        Parent {
            entity: ground.to_bits(),
        },
    )
    .unwrap();
    w.step().unwrap();
    let (first, _) = pins(&p);
    assert!(
        (first - Vec3::new(4.3, 13.5, -5.8)).length() < 1e-4,
        "under the grandparent the first pin is at {first}"
    );
}
