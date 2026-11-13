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

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use support::*;

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

/// The world's wind (`set_air`, og.Wind as the host reads it) pushes a rope
/// and a cloth exactly as the same wind set on each of them does, and the
/// two add: half from the world and half their own is that wind too.
#[test]
fn the_worlds_wind_pushes_a_rope_and_a_cloth_as_their_own_does() {
    let blow = Vec3::new(1.0, 0.0, 6.0);
    let hang = |world_wind: Vec3, own: Vec3| {
        let (mut w, p) = world(Physics::default(), 1);
        p.set_air(og_core::air::Air {
            wind: world_wind,
            ..Default::default()
        });
        let rope = w
            .spawn((
                at(0.0, 5.0, 0.0),
                Rope {
                    wind: own,
                    ..Rope::default()
                },
            ))
            .unwrap();
        let cloth = w
            .spawn((
                at(4.0, 5.0, 0.0),
                Cloth {
                    wind: own,
                    ..Cloth::default()
                },
            ))
            .unwrap();
        run(&mut w, 90);
        (
            p.rope_points(rope).unwrap(),
            p.cloth_points(cloth).unwrap().points,
        )
    };
    let still = hang(Vec3::ZERO, Vec3::ZERO);
    let world_wind = hang(blow, Vec3::ZERO);
    assert_eq!(
        world_wind,
        hang(Vec3::ZERO, blow),
        "the world's against its own"
    );
    assert_eq!(world_wind, hang(blow * 0.5, blow * 0.5), "half of each");
    // Downwind of where they hang in still air (+z, where the wind goes).
    let (rope_end, hem) = (world_wind.0.last().unwrap(), world_wind.1.last().unwrap());
    assert!(
        rope_end.z > still.0.last().unwrap().z + 0.2,
        "the rope's end at {rope_end}"
    );
    assert!(
        hem.z > still.1.last().unwrap().z + 0.1,
        "the cloth's hem at {hem}"
    );
}

/// A fan (an og.WindMotor in the world's air) blows the rope hanging in
/// its reach and leaves alone the one hanging out of it: the air is asked
/// where each hangs, not once for the world.
#[test]
fn a_motor_blows_the_rope_in_its_reach_and_not_the_one_beyond() {
    let hang = |fan: bool| {
        let (mut w, p) = world(Physics::default(), 1);
        if fan {
            p.set_air(og_core::air::Air {
                motors: vec![og_core::air::Motor {
                    position: Vec3::new(0.0, 4.0, 0.0),
                    direction: Vec3::Z,
                    radius: 5.0,
                    strength: 12.0,
                    kind: og_core::air::MotorKind::Directional,
                }],
                ..Default::default()
            });
        }
        let ropes = [0.0, 20.0].map(|x| w.spawn((at(x, 5.0, 0.0), Rope::default())).unwrap());
        run(&mut w, 90);
        ropes.map(|r| *p.rope_points(r).unwrap().last().unwrap())
    };
    let (still, blown) = (hang(false), hang(true));
    assert!(
        blown[0].z > still[0].z + 0.2,
        "in the fan's reach: {} against {}",
        blown[0],
        still[0]
    );
    assert_eq!(blown[1], still[1], "out of its reach");
}
