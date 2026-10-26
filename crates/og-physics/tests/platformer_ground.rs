//! og.PlatformerController on the ground (examples/hollowbell's findings
//! #15 and #16): a run on flat ground holds maxSpeed tick after tick, on
//! one box, on adjacent boxes and on a tilemap's collision rectangles, and
//! a controller put exactly on a floor stands on it and runs.
//!
//! Ways this could fail, each caught below: the move meets the floor it
//! grazes and loses its step (a stutter: a tick near 0 m/s, then the
//! acceleration again); a seam between two boxes stops it; a shape put at
//! the floor's height sinks into it and sticks; it drifts off the floor
//! (the gap grows or shrinks run after run).

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use og_tilemap::{Layer, MapState, TilemapAsset, Tileset, TilesetRef, collision_rects};
use std::sync::Arc;
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

/// Hollowbell's hero: a 0.7 × 1.25 box offset so the transform is at its
/// feet, standing at `x` on ground whose top is `top`, with every default
/// but `maxSpeed`.
fn hero(w: &mut World, x: f32, top: f32, max_speed: f32) -> Entity {
    w.spawn((
        at(x, top, 0.0),
        Collider {
            offset: Vec3::new(0.0, 0.625, 0.0),
            ..Collider::cuboid(Vec3::new(0.7, 1.25, 1.0))
        },
        PlatformerController {
            max_speed,
            ..PlatformerController::default()
        },
    ))
    .unwrap()
}

/// Runs with `move_x` for `ticks`: each tick's x velocity and feet height.
fn run_for(w: &mut World, e: Entity, move_x: f32, ticks: u32) -> Vec<(f32, f32)> {
    (0..ticks)
        .map(|_| {
            let mut c = w.get::<PlatformerController>(e).unwrap();
            c.move_x = move_x;
            w.insert(e, c).unwrap();
            w.step().unwrap();
            let c = w.get::<PlatformerController>(e).unwrap();
            assert!(
                c.grounded,
                "it left the ground at {}",
                transform(w, e).position
            );
            (c.velocity.x, transform(w, e).position.y)
        })
        .collect()
}

/// 2.3 s of Right from standing at `x`: once at maxSpeed, no tick under
/// 95 % of it, the distance what the speed gives, the feet a skin over `top`.
fn holds_its_speed(w: &mut World, x: f32, top: f32, max_speed: f32, what: &str) {
    let e = hero(w, x, top, max_speed);
    run(w, 10);
    let ticks = run_for(w, e, 1.0, 138);
    let reached = ticks
        .iter()
        .position(|t| t.0 >= max_speed)
        .unwrap_or_else(|| panic!("{what}: it never reached {max_speed} m/s"));
    for (i, (vx, y)) in ticks.iter().enumerate().skip(reached) {
        assert!(
            *vx >= 0.95 * max_speed,
            "{what}: tick {i} ran {vx} m/s ({:?})",
            ticks.iter().map(|t| t.0).collect::<Vec<_>>()
        );
        assert!(
            (y - top - 0.01).abs() < 1e-3,
            "{what}: tick {i} the feet are at {y}"
        );
    }
    // Accelerating at 70 m/s² costs max_speed² / 140 m against running at it.
    let ran = transform(w, e).position.x - x;
    let want = max_speed * 2.3 - max_speed * max_speed / 140.0;
    assert!(
        (ran - want).abs() < 0.1,
        "{what}: 2.3 s ran {ran} m, not {want}"
    );
}

#[test]
fn a_run_on_one_flat_box_holds_max_speed() {
    let (mut w, _p) = world2d();
    spawn_static(
        &mut w,
        at(37.0, -1.5, 0.0),
        Collider::cuboid(Vec3::new(74.0, 3.0, 1.0)),
    );
    holds_its_speed(&mut w, 2.0, 0.0, 8.0, "one box");
    // Hollowbell's tuning, where its floor is.
    let (mut w, _p) = world2d();
    spawn_static(
        &mut w,
        at(45.0, 14.5, 0.0),
        Collider::cuboid(Vec3::new(74.0, 3.0, 1.0)),
    );
    holds_its_speed(&mut w, 9.0, 16.0, 6.5, "one box at y 16");
}

#[test]
fn a_run_over_adjacent_boxes_holds_max_speed() {
    let (mut w, _p) = world2d();
    // Twelve boxes of uneven widths and depths, their tops at 16 (each
    // centre and size in f32, as a tilemap's are).
    let mut x0 = 0.0f32;
    for (i, width) in [
        3.0f32, 1.0, 5.0, 2.0, 7.0, 1.0, 4.0, 3.0, 6.0, 2.0, 5.0, 3.0,
    ]
    .iter()
    .enumerate()
    {
        let depth = 1.0 + (i % 3) as f32;
        spawn_static(
            &mut w,
            at(x0 + width * 0.5, 16.0 - depth * 0.5, 0.0),
            Collider::cuboid(Vec3::new(*width, depth, 1.0)),
        );
        x0 += width;
    }
    holds_its_speed(&mut w, 1.0, 16.0, 6.5, "adjacent boxes");
}

/// A Hollowbell-like rock layer (96 × 48 cells of 1 m, its top-left corner
/// at y 48): an upper floor three rows deep with its top at y 16 from x 8 to
/// 82, cave walls, a ledge above it and uneven rock below, as og-runtime
/// spawns it: a static box per collision rectangle.
fn tilemap(w: &mut World) {
    let tileset = Tileset::parse(
        "type tileset\nversion 1\n---\ntexture \"t.png\"\ngrid 16 16 8 8\nsolid 0\n",
    )
    .unwrap();
    let (cols, rows) = (96u32, 48u32);
    let mut rock = Layer::new("rock", cols, rows, 0);
    rock.collision = true;
    let asset = TilemapAsset {
        width: cols,
        height: rows,
        tile_width_px: 16,
        tile_height_px: 16,
        unit: 1.0,
        tilesets: vec![TilesetRef::Asset("tiles/t.ogt".into())],
        layers: vec![rock],
        objects: Vec::new(),
    };
    let mut state = MapState::new(&asset, &[Arc::new(tileset)]).unwrap();
    let mut fill = |x: std::ops::Range<u32>, y: std::ops::Range<u32>| {
        for cy in y {
            for cx in x.clone() {
                state.set(0, cx, cy, 0).unwrap();
            }
        }
    };
    fill(0..8, 0..48); // the left wall
    fill(8..82, 32..35); // the upper floor, top at y 16
    fill(86..96, 32..35);
    fill(30..41, 22..25); // a ledge above it
    fill(8..40, 35..39); // uneven rock under it
    fill(50..82, 35..41);
    fill(8..96, 46..48);
    let origin = Vec3::new(0.0, 48.0, 0.0);
    let rects = collision_rects(&state, 0);
    assert!(rects.len() > 5, "{rects:?}");
    for r in rects {
        let c = (r.min + r.max) * 0.5;
        let size = r.max - r.min;
        spawn_static(
            w,
            at(origin.x + c.x, origin.y + c.y, origin.z),
            Collider::cuboid(Vec3::new(size.x, size.y, 1.0)),
        );
    }
}

#[test]
fn a_run_on_a_tilemaps_collision_layer_holds_max_speed() {
    let (mut w, _p) = world2d();
    tilemap(&mut w);
    holds_its_speed(&mut w, 9.0, 16.0, 6.5, "the tilemap's floor");
}

#[test]
fn a_controller_put_exactly_on_the_floor_stands_and_runs() {
    // Every 0.37 m of the floor (x 26 and 40 among the places it sank).
    let xs: Vec<f32> = (0..190)
        .map(|i| 9.0 + i as f32 * 0.37)
        .filter(|x| *x < 80.0)
        .collect();
    let (mut w, _p) = world2d();
    tilemap(&mut w);
    let e = hero(&mut w, 9.0, 16.0, 6.5);
    for x in xs.into_iter().chain([20.0, 26.0, 40.0, 60.0]) {
        // A warp: the transform written to the floor's height.
        w.insert(e, at(x, 16.0, 0.0)).unwrap();
        let settled = run_for(&mut w, e, 0.0, 2);
        let y = settled[1].1;
        assert!(
            (y - 16.01).abs() < 1e-3,
            "put at x {x}: the feet settled at {y}"
        );
        let ticks = run_for(&mut w, e, 1.0, 12);
        let ran = transform(&w, e).position.x - x;
        assert!(
            ran > 0.6,
            "put at x {x}: 12 ticks of Right ran {ran} m ({ticks:?})"
        );
        run_for(&mut w, e, 0.0, 10);
    }
}

#[test]
fn a_run_left_and_right_keeps_the_feet_a_skin_over_the_floor() {
    let (mut w, _p) = world2d();
    tilemap(&mut w);
    let e = hero(&mut w, 40.0, 16.0, 8.0);
    for i in 0..12 {
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        for (vx, y) in run_for(&mut w, e, dir, 45) {
            assert!((y - 16.01).abs() < 1e-3, "the feet at {y} running at {vx}");
        }
    }
}

#[test]
fn it_runs_up_and_down_a_walkable_slope_at_its_speed_and_not_up_a_steep_one() {
    let (mut w, _p) = world2d();
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(200.0, 1.0, 1.0)),
    );
    // A plank 8 m long turned `deg` about z, its middle at `x` on the floor.
    let ramp = |w: &mut World, x: f32, deg: f32| {
        spawn_static(
            w,
            rotated(
                at(x, 0.0, 0.0),
                og_core::glam::Quat::from_rotation_z(deg.to_radians()),
            ),
            Collider::cuboid(Vec3::new(8.0, 0.1, 1.0)),
        );
    };
    // A 30° hill peaking 2 m up at x 15.46, then a 60° wall about x 40.
    ramp(&mut w, 12.0, 30.0);
    ramp(&mut w, 18.93, -30.0);
    ramp(&mut w, 40.0, 60.0);
    let e = hero(&mut w, 2.0, 0.0, 8.0);
    run(&mut w, 5);
    let mut top = 0.0f32;
    for _ in 0..360 {
        let mut c = w.get::<PlatformerController>(e).unwrap();
        c.move_x = 1.0;
        w.insert(e, c).unwrap();
        w.step().unwrap();
        let c = w.get::<PlatformerController>(e).unwrap();
        let p = transform(&w, e).position;
        top = top.max(p.y);
        if p.x > 12.5 && p.x < 22.0 {
            assert!(c.grounded, "on the hill at {p}");
            assert!(
                c.velocity.x >= 0.95 * 8.0,
                "on the hill at {p}: {} m/s",
                c.velocity.x
            );
        }
    }
    assert!(top > 1.6, "it climbed to {top}");
    let p = transform(&w, e).position;
    assert!(
        p.x > 36.0 && p.x < 40.0 && p.y < 0.5,
        "the steep slope stopped it at {p}"
    );
}
