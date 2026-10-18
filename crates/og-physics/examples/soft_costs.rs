//! The CPU cost of the physics beyond rigid bodies, per 60 Hz tick (the E5
//! round's cost table, `tests/evidence/E5/feature-costs.md`): each feature
//! alone in a world with a floor, stepped 120 ticks to settle, then timed
//! over 300 ticks (best of three). Release builds only give meaningful
//! numbers:
//!
//!   cargo run --release -p og-physics --example soft_costs

use og_core::glam::{Vec2, Vec3};
use og_ecs::{World, WorldDesc};
use og_physics::*;
use std::time::Instant;

fn world(profile: u32) -> World {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    install(
        &mut w,
        Physics {
            profile,
            ..Physics::default()
        },
    )
    .unwrap();
    w.spawn((
        Transform::at(Vec3::new(0.0, -0.5, 0.0)),
        Collider::cuboid(Vec3::new(60.0, 1.0, 60.0)),
    ))
    .unwrap();
    w
}

/// Milliseconds per tick: the best of three runs of 300 ticks after 120.
fn time(mut w: World) -> f64 {
    for _ in 0..120 {
        w.step().unwrap();
    }
    let mut best = f64::MAX;
    for _ in 0..3 {
        let t = Instant::now();
        for _ in 0..300 {
            w.step().unwrap();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / 300.0);
    }
    best
}

fn main() {
    let at = |x: f32, y: f32, z: f32| Transform::at(Vec3::new(x, y, z));
    let base3 = time(world(PROFILE_3D));
    let base2 = time(world(PROFILE_2D));
    println!("| Feature | profile | ms per tick | over the empty world |");
    println!("|---|---|---|---|");
    println!("| empty world (a floor) | 3D | {base3:.3} | |");
    println!("| empty world (a floor) | 2D | {base2:.3} | |");
    let row = |label: &str, profile: u32, w: World| {
        let ms = time(w);
        let base = if profile == PROFILE_3D { base3 } else { base2 };
        println!(
            "| {label} | {} | {ms:.3} | {:+.3} |",
            if profile == PROFILE_3D { "3D" } else { "2D" },
            ms - base
        );
    };
    for count in [1000u32, 2000, 4000] {
        let mut w = world(PROFILE_2D);
        // A basin the particles settle in.
        for (x, y, sx, sy) in [(-3.2, 3.0, 0.4, 6.0), (3.2, 3.0, 0.4, 6.0)] {
            w.spawn((at(x, y, 0.0), Collider::cuboid(Vec3::new(sx, sy, 1.0))))
                .unwrap();
        }
        w.spawn((
            at(0.0, 2.0, 0.0),
            Fluid2D {
                count,
                size: Vec2::new(6.0, 4.0),
                ..Fluid2D::default()
            },
        ))
        .unwrap();
        row(
            &format!("og.Fluid2D, {count} particles (2 substeps)"),
            PROFILE_2D,
            w,
        );
    }
    for resolution in [64u32, 128] {
        let mut w = world(PROFILE_3D);
        w.spawn((
            at(0.0, 1.0, 0.0),
            WaterSurface {
                resolution,
                size: 16.0,
                ..WaterSurface::default()
            },
        ))
        .unwrap();
        for i in 0..8 {
            w.spawn((
                at(-3.0 + i as f32, 3.0 + i as f32 * 0.5, 0.0),
                RigidBody::default(),
                Collider::cuboid(Vec3::splat(0.6)),
            ))
            .unwrap();
        }
        row(
            &format!("og.WaterSurface {resolution}x{resolution}, 8 floating boxes"),
            PROFILE_3D,
            w,
        );
    }
    {
        let mut w = world(PROFILE_2D);
        w.spawn((at(0.0, 1.0, 0.0), WaterLine::default())).unwrap();
        row("og.WaterLine, 96 columns", PROFILE_2D, w);
    }
    {
        let mut w = world(PROFILE_3D);
        for i in 0..10 {
            w.spawn((at(i as f32, 6.0, 0.0), Rope::default())).unwrap();
        }
        row("10 og.Rope, 16 segments", PROFILE_3D, w);
    }
    {
        let mut w = world(PROFILE_3D);
        for i in 0..4 {
            w.spawn((
                at(i as f32 * 3.0, 4.0, 0.0),
                Cloth {
                    pins: PIN_TOP,
                    ..Cloth::default()
                },
            ))
            .unwrap();
        }
        row("4 og.Cloth, 12x12", PROFILE_3D, w);
    }
    {
        let mut w = world(PROFILE_3D);
        for i in 0..8 {
            w.spawn((
                at(i as f32 * 2.0, 1.0, 0.0),
                Collider::capsule(0.3, 1.0),
                CharacterController {
                    movement: Vec3::new(1.0, 0.0, 0.5),
                    ..CharacterController::default()
                },
            ))
            .unwrap();
        }
        row("8 og.CharacterController", PROFILE_3D, w);
    }
    {
        let mut w = world(PROFILE_2D);
        for i in 0..8 {
            w.spawn((
                at(i as f32 * 2.0, 1.0, 0.0),
                Collider::cuboid(Vec3::new(0.6, 1.0, 1.0)),
                PlatformerController {
                    move_x: 1.0,
                    ..PlatformerController::default()
                },
            ))
            .unwrap();
        }
        row("8 og.PlatformerController", PROFILE_2D, w);
    }
    {
        let mut w = world(PROFILE_3D);
        for i in 0..4 {
            w.spawn((
                at(i as f32 * 4.0, 1.0, 0.0),
                RigidBody {
                    mass: 800.0,
                    ..RigidBody::default()
                },
                Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
                Vehicle {
                    throttle: 0.5,
                    ..Vehicle::default()
                },
            ))
            .unwrap();
        }
        row("4 og.Vehicle (raycast cars)", PROFILE_3D, w);
    }
}
