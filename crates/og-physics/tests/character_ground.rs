//! og.CharacterController (3D) on the ground, the 3D side of row 72 (the
//! cause of examples/hollowbell's #15 and #16, measured here too before
//! the move was replaced: on rapier's character controller a run dropped
//! to 0 to 0.4 m/s for a tick every second or so on one box, on adjacent
//! boxes and on a triangle mesh, a capsule put on the floor sank 9 cm
//! into it and crept up 0.1 mm a tick, and 0.2 m stairs with 0.3 m treads
//! stopped it at the first step). A run holds its speed tick after tick on
//! one box, on adjacent boxes, on a triangle mesh (a glTF floor) and on a
//! height field (terrain), up and down a 30° slope; stairs are climbed up
//! to stepHeight and not past it; a slope steeper than maxSlope stops it;
//! a capsule put exactly on a floor stands a skin over it and runs.
//!
//! Ways this could fail, each caught below: the move meets the floor it
//! grazes and loses its step (a tick near 0 m/s); a seam between boxes or
//! triangles stops it; a shape put at the floor's height sinks and
//! sticks; the gap to the floor drifts; a step is not taken, or one higher
//! than stepHeight is; it walks up a slope steeper than maxSlope; it
//! leaves the ground running down a slope or stairs within snapDistance.

mod support;

use og_core::glam::{Quat, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use std::sync::Arc;
use support::*;

/// The capsule's centre over its feet (radius 0.3, a 1 m segment).
const HALF: f32 = 0.8;
/// The default skin.
const SKIN: f32 = 0.02;

/// Lantern Vale's hero: a capsule of radius 0.3 and a 1 m segment (1.6 m
/// tall), its controller's settings, its feet at `top` at (x, z).
fn hero(w: &mut World, x: f32, z: f32, top: f32) -> Entity {
    w.spawn((
        at(x, top + HALF, z),
        Collider::capsule(0.3, 1.0),
        CharacterController {
            max_slope: 40.0,
            step_height: 0.35,
            snap_distance: 0.3,
            ..CharacterController::default()
        },
    ))
    .unwrap()
}

/// One tick with `movement`: the horizontal speed it moved at, where it
/// ended and whether it stands on the ground.
struct Tick {
    speed: f32,
    at: Vec3,
    grounded: bool,
}

fn walk(w: &mut World, e: Entity, movement: Vec3, ticks: u32) -> Vec<Tick> {
    (0..ticks)
        .map(|_| {
            let mut c = w.get::<CharacterController>(e).unwrap();
            c.movement = movement;
            w.insert(e, c).unwrap();
            let before = transform(w, e).position;
            w.step().unwrap();
            let c = w.get::<CharacterController>(e).unwrap();
            let at = transform(w, e).position;
            let d = (at - before) * 60.0;
            Tick {
                speed: Vec2::new(d.x, d.z).length(),
                at,
                grounded: c.grounded,
            }
        })
        .collect()
}

fn speeds(ticks: &[Tick]) -> Vec<f32> {
    ticks.iter().map(|t| t.speed).collect()
}

/// Two seconds of `movement` from standing at (x, z) on ground whose top
/// is `top`: every tick at its speed, on the ground, the feet a skin over
/// it.
fn holds_its_speed(w: &mut World, x: f32, z: f32, top: f32, movement: Vec3, what: &str) {
    let e = hero(w, x, z, top);
    walk(w, e, Vec3::ZERO, 10);
    let want = movement.length();
    let ticks = walk(w, e, movement, 120);
    for (i, t) in ticks.iter().enumerate() {
        assert!(t.grounded, "{what}: tick {i} left the ground at {}", t.at);
        assert!(
            t.speed >= 0.99 * want,
            "{what}: tick {i} ran {} m/s ({:?})",
            t.speed,
            speeds(&ticks)
        );
        assert!(
            (t.at.y - top - HALF - SKIN).abs() < 1e-3,
            "{what}: tick {i} the centre is at {}",
            t.at.y
        );
    }
    let ran = transform(w, e).position - Vec3::new(x, top + HALF + SKIN, z);
    assert!(
        (Vec2::new(ran.x, ran.z).length() - 2.0 * want).abs() < 0.05,
        "{what}: two seconds ran {ran}"
    );
}

/// Twelve boxes of uneven widths and depths, their tops at 16.
fn adjacent_boxes(w: &mut World) {
    let mut x0 = -50.0f32;
    for (i, width) in [
        3.0f32, 1.0, 5.0, 2.0, 7.0, 1.0, 4.0, 3.0, 6.0, 2.0, 5.0, 3.0, 9.0, 1.3, 2.7, 30.0,
    ]
    .iter()
    .enumerate()
    {
        let depth = 1.0 + (i % 3) as f32;
        spawn_static(
            w,
            at(x0 + width * 0.5, 16.0 - depth * 0.5, 0.0),
            Collider::cuboid(Vec3::new(*width, depth, 6.0)),
        );
        x0 += width;
    }
}

/// A flat grid of `n` × `n` cells of `cell` m, centred on the origin, as a
/// glTF floor comes: two triangles a cell.
fn grid_mesh(n: u32, cell: f32) -> MeshDesc {
    let half = n as f32 * cell * 0.5;
    let mut positions = Vec::new();
    for r in 0..=n {
        for c in 0..=n {
            positions.push(Vec3::new(
                c as f32 * cell - half,
                0.0,
                r as f32 * cell - half,
            ));
        }
    }
    let mut indices = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let i = r * (n + 1) + c;
            indices.push([i, i + n + 1, i + 1]);
            indices.push([i + 1, i + n + 1, i + n + 2]);
        }
    }
    MeshDesc { positions, indices }
}

fn mesh_floor(w: &mut World, p: &PhysicsSystem) {
    p.set_mesh(3, grid_mesh(80, 1.0)).unwrap();
    spawn_static(w, at(0.0, 0.0, 0.0), Collider::trimesh(3));
}

/// A flat 80 × 80 m height field of 1 m cells at y 0.
fn terrain(w: &mut World, p: &PhysicsSystem) {
    p.set_heightfield(
        4,
        HeightfieldDesc {
            columns: 81,
            rows: 81,
            size: Vec2::splat(80.0),
            heights: Arc::from(vec![0.0f32; 81 * 81]),
        },
    )
    .unwrap();
    spawn_static(
        w,
        at(0.0, 0.0, 0.0),
        Collider {
            shape: SHAPE_HEIGHTFIELD,
            mesh: 4,
            ..Collider::default()
        },
    );
}

/// A 200 × 1 × 200 floor whose top is y 0.
fn big_floor(w: &mut World) {
    spawn_static(
        w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(200.0, 1.0, 200.0)),
    );
}

#[test]
fn a_run_on_one_box_and_on_adjacent_boxes_holds_its_speed() {
    let (mut w, _p) = world(Physics::default(), 1);
    big_floor(&mut w);
    holds_its_speed(&mut w, -40.0, 0.3, 0.0, Vec3::new(6.0, 0.0, 0.0), "one box");
    let (mut w, _p) = world(Physics::default(), 1);
    big_floor(&mut w);
    holds_its_speed(
        &mut w,
        -40.0,
        -20.0,
        0.0,
        Vec3::new(4.8, 0.0, 3.6),
        "one box, diagonally",
    );
    let (mut w, _p) = world(Physics::default(), 1);
    adjacent_boxes(&mut w);
    holds_its_speed(
        &mut w,
        -48.5,
        0.3,
        16.0,
        Vec3::new(6.0, 0.0, 0.0),
        "adjacent boxes",
    );
}

#[test]
fn a_run_on_a_triangle_mesh_and_a_height_field_holds_its_speed() {
    let (mut w, p) = world(Physics::default(), 1);
    mesh_floor(&mut w, &p);
    holds_its_speed(
        &mut w,
        -30.0,
        0.3,
        0.0,
        Vec3::new(5.0, 0.0, 3.0),
        "a triangle mesh",
    );
    let (mut w, p) = world(Physics::default(), 1);
    mesh_floor(&mut w, &p);
    holds_its_speed(
        &mut w,
        -30.0,
        0.5,
        0.0,
        Vec3::new(6.0, 0.0, 0.0),
        "a triangle mesh, along its rows",
    );
    let (mut w, p) = world(Physics::default(), 1);
    terrain(&mut w, &p);
    holds_its_speed(
        &mut w,
        -30.0,
        0.3,
        0.0,
        Vec3::new(5.0, 0.0, 3.0),
        "a height field",
    );
}

#[test]
fn a_controller_put_exactly_on_the_floor_stands_a_skin_over_it_and_runs() {
    // 190 places on the adjacent boxes, the mesh and the field.
    let (mut boxes, _pb) = world(Physics::default(), 1);
    adjacent_boxes(&mut boxes);
    let (mut mesh, pm) = world(Physics::default(), 1);
    mesh_floor(&mut mesh, &pm);
    let (mut field, pf) = world(Physics::default(), 1);
    terrain(&mut field, &pf);
    for (w, top, x0, what) in [
        (&mut boxes, 16.0, -48.0, "boxes"),
        (&mut mesh, 0.0, -35.0, "mesh"),
        (&mut field, 0.0, -35.0, "field"),
    ] {
        let e = hero(w, x0, 0.0, top);
        for i in 0..190 {
            let x = x0 + i as f32 * 0.37;
            let z = (i % 7) as f32 * 0.29 - 0.9;
            // A warp: the transform written to the floor's height.
            w.insert(e, at(x, top + HALF, z)).unwrap();
            let settled = walk(w, e, Vec3::ZERO, 2);
            assert!(
                (settled[1].at.y - top - HALF - SKIN).abs() < 1e-3 && settled[1].grounded,
                "{what}: put at ({x}, {z}) the centre settled at {}",
                settled[1].at.y
            );
            let ticks = walk(w, e, Vec3::new(0.0, 0.0, 6.0), 12);
            let ran = transform(w, e).position.z - z;
            assert!(
                ran > 1.15 && ticks.iter().all(|t| t.grounded),
                "{what}: put at ({x}, {z}), 12 ticks ran {ran} m ({:?})",
                speeds(&ticks)
            );
        }
    }
}

#[test]
fn a_run_back_and_forth_keeps_the_feet_a_skin_over_the_floor() {
    let (mut w, p) = world(Physics::default(), 1);
    mesh_floor(&mut w, &p);
    let e = hero(&mut w, 0.0, 0.0, 0.0);
    for i in 0..12 {
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        for t in walk(&mut w, e, Vec3::new(6.0 * dir, 0.0, 1.0), 45) {
            assert!(
                (t.at.y - HALF - SKIN).abs() < 1e-3 && t.grounded,
                "the centre at {} running at {}",
                t.at.y,
                t.speed
            );
        }
    }
}

#[test]
fn it_runs_up_and_down_a_30_degree_slope_at_its_speed_and_not_up_a_50_degree_one() {
    let (mut w, _p) = world(Physics::default(), 1);
    big_floor(&mut w);
    // A plank 16 m long turned `deg` about z, its middle at x on the floor.
    let ramp = |w: &mut World, x: f32, deg: f32| {
        spawn_static(
            w,
            rotated(at(x, 0.0, 0.0), Quat::from_rotation_z(deg.to_radians())),
            Collider::cuboid(Vec3::new(16.0, 0.2, 6.0)),
        );
    };
    // A 30° hill from x 3 peaking 4 m up at x 16.93, down to the floor at
    // x 30, then a 50° wall from x 45.
    ramp(&mut w, 10.0, 30.0);
    ramp(&mut w, 23.86, -30.0);
    ramp(&mut w, 45.0, 50.0);
    let e = hero(&mut w, 0.0, 0.0, 0.0);
    walk(&mut w, e, Vec3::ZERO, 5);
    let ticks = walk(&mut w, e, Vec3::new(6.0, 0.0, 0.0), 480);
    let mut top = 0.0f32;
    for (i, t) in ticks.iter().enumerate() {
        top = top.max(t.at.y);
        if t.at.x < 43.0 {
            assert!(t.grounded, "tick {i} left the ground at {}", t.at);
            assert!(
                t.speed >= 0.99 * 6.0,
                "tick {i} at {} ran {} m/s",
                t.at,
                t.speed
            );
        }
    }
    assert!(top > 4.0, "it climbed to {top}");
    let p = transform(&w, e).position;
    assert!(
        p.x > 43.0 && p.x < 45.0 && p.y < HALF + 0.3,
        "the 50 degree slope stopped it at {p}"
    );
}

/// A flight of `n` steps of `rise` m and 0.3 m treads from x 3 up along +x,
/// then a landing.
fn stairs(w: &mut World, n: u32, rise: f32) -> f32 {
    for i in 0..n {
        let h = rise * (i + 1) as f32;
        spawn_static(
            w,
            at(3.0 + 0.3 * i as f32 + 0.15, h * 0.5, 0.0),
            Collider::cuboid(Vec3::new(0.3, h, 3.0)),
        );
    }
    let top = rise * n as f32;
    let start = 3.0 + 0.3 * n as f32;
    spawn_static(
        w,
        at(start + 5.0, top * 0.5, 0.0),
        Collider::cuboid(Vec3::new(10.0, top, 3.0)),
    );
    top
}

#[test]
fn stairs_of_0_2_m_are_climbed_and_descended_at_walking_speed() {
    let (mut w, _p) = world(Physics::default(), 1);
    big_floor(&mut w);
    let top = stairs(&mut w, 10, 0.2);
    let e = hero(&mut w, 0.0, 0.0, 0.0);
    walk(&mut w, e, Vec3::ZERO, 5);
    let up = walk(&mut w, e, Vec3::new(3.0, 0.0, 0.0), 150);
    for (i, t) in up.iter().enumerate() {
        assert!(t.grounded, "up: tick {i} left the ground at {}", t.at);
        assert!(
            t.speed >= 0.99 * 3.0,
            "up: tick {i} at {} ran {} m/s ({:?})",
            t.at,
            t.speed,
            speeds(&up)
        );
    }
    let p = transform(&w, e).position;
    assert!(
        (p.y - top - HALF - SKIN).abs() < 1e-3,
        "up the stairs to {p}"
    );
    let down = walk(&mut w, e, Vec3::new(-3.0, 0.0, 0.0), 150);
    for (i, t) in down.iter().enumerate() {
        assert!(t.grounded, "down: tick {i} left the ground at {}", t.at);
        assert!(
            t.speed >= 0.99 * 3.0,
            "down: tick {i} at {} ran {} m/s",
            t.at,
            t.speed
        );
    }
    let p = transform(&w, e).position;
    assert!((p.y - HALF - SKIN).abs() < 1e-3, "down the stairs to {p}");
}

#[test]
fn a_step_of_step_height_is_climbed_and_a_higher_one_is_not() {
    let (mut w, _p) = world(Physics::default(), 1);
    big_floor(&mut w);
    // Blocks at x 2 to 4, one per lane along z, each `h` high.
    let heights = [0.1f32, 0.2, 0.3, 0.345, 0.36, 0.4, 0.6];
    let mut walkers = Vec::new();
    for (i, h) in heights.iter().enumerate() {
        let z = i as f32 * 3.0;
        spawn_static(
            &mut w,
            at(3.0, h * 0.5, z),
            Collider::cuboid(Vec3::new(2.0, *h, 2.0)),
        );
        walkers.push(hero(&mut w, 0.0, z, 0.0));
    }
    run(&mut w, 5);
    for _ in 0..60 {
        for &e in &walkers {
            let mut c = w.get::<CharacterController>(e).unwrap();
            c.movement = Vec3::new(3.0, 0.0, 0.0);
            w.insert(e, c).unwrap();
        }
        w.step().unwrap();
    }
    for (h, e) in heights.iter().zip(walkers) {
        let p = transform(&w, e).position;
        if *h <= 0.35 {
            assert!(
                p.x > 2.5 && (p.y - h - HALF - SKIN).abs() < 1e-3,
                "the {h} m step was climbed: {p}"
            );
        } else {
            assert!(
                p.x < 2.0 && (p.y - HALF - SKIN).abs() < 1e-3,
                "the {h} m step stopped it: {p}"
            );
        }
    }
}
