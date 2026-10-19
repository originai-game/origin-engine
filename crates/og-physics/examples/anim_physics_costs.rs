//! The CPU cost of the phase 2 animation and physics round's features, per
//! 60 Hz tick (`tests/evidence/P2/anim-physics/costs.md`): cloth with and
//! without self-collision by particle count, fluids alone and in groups,
//! and ragdolls in 2D and 3D. Each scene is built fresh three times, run 30
//! ticks and then timed over 150 (the best of the three), so a cloth is
//! timed while it folds onto the floor and a fluid while it settles.
//! Release builds only give meaningful numbers:
//!
//!   cargo run --release -p og-physics --example anim_physics_costs

use og_core::glam::{Quat, Vec2, Vec3};
use og_ecs::{World, WorldDesc};
use og_physics::*;
use std::time::Instant;

fn world(profile: u32) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    let p = install(
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
    (w, p)
}

/// Milliseconds per tick: the best of three fresh builds, each run 30 ticks
/// and timed over the next 150; `each` runs before every tick.
fn time(
    build: &dyn Fn() -> (World, PhysicsSystem),
    each: &dyn Fn(&mut World, &PhysicsSystem, u64),
) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..3 {
        let (mut w, p) = build();
        for tick in 0..30 {
            each(&mut w, &p, tick);
            w.step().unwrap();
        }
        let t = Instant::now();
        for tick in 30..180 {
            each(&mut w, &p, tick);
            w.step().unwrap();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / 150.0);
    }
    best
}

fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::at(Vec3::new(x, y, z))
}

fn none(_: &mut World, _: &PhysicsSystem, _: u64) {}

fn main() {
    let base3 = time(&|| world(PROFILE_3D), &none);
    let base2 = time(&|| world(PROFILE_2D), &none);
    println!(
        "Empty worlds (a floor): 3D {base3:.3} ms, 2D {base2:.3} ms per tick; the rows below are over them.\n"
    );

    println!(
        "| Cloth, lowered onto the floor | particles | ms per tick, no self-collision | ms per tick, self-collision (thickness = spacing, 2 substeps) |"
    );
    println!("|---|---|---|---|");
    for n in [12u32, 24, 32, 48, 64] {
        let spacing = 1.0 / n as f32;
        let cloth = move |thickness: f32, substeps: u32| {
            move || {
                let (mut w, p) = world(PROFILE_3D);
                w.spawn((
                    at(0.0, 1.2, 0.0),
                    Cloth {
                        columns: n,
                        rows: n,
                        spacing,
                        gust: 0.0,
                        radius: spacing * 0.5,
                        thickness,
                        substeps,
                        friction: 0.9,
                        iterations: 6,
                        ..Cloth::default()
                    },
                ))
                .unwrap();
                (w, p)
            }
        };
        let lower = |w: &mut World, _: &PhysicsSystem, tick: u64| {
            let s = (tick as f32 / 150.0).min(1.0);
            let e = w.entities()[1];
            w.insert(e, at(0.0, 1.2 - 0.8 * s, 0.0)).unwrap();
        };
        let plain = time(&cloth(0.0, 1), &lower) - base3;
        let selfc = time(&cloth(spacing, 2), &lower) - base3;
        println!("| {n}x{n} | {} | {plain:.3} | {selfc:.3} |", n * n);
    }

    println!("\n| Fluids (2D, a 6 m basin) | particles | ms per tick |");
    println!("|---|---|---|");
    let basin = |fluids: Vec<Fluid2D>| {
        move || {
            let (mut w, p) = world(PROFILE_2D);
            for (x, y, sx, sy) in [(-3.2, 3.0, 0.4, 6.0), (3.2, 3.0, 0.4, 6.0)] {
                w.spawn((at(x, y, 0.0), Collider::cuboid(Vec3::new(sx, sy, 1.0))))
                    .unwrap();
            }
            for (i, f) in fluids.iter().enumerate() {
                w.spawn((
                    at(
                        -2.2 + (i % 4) as f32 * 1.45,
                        1.0 + (i / 4) as f32 * 1.3,
                        0.0,
                    ),
                    *f,
                ))
                .unwrap();
            }
            (w, p)
        }
    };
    let fluid = |count: u32, group: u32, density: f32, mixing: f32, seed: u32| Fluid2D {
        count,
        size: Vec2::new(1.3, 1.1),
        group,
        density,
        mixing,
        seed,
        ..Fluid2D::default()
    };
    let rows: Vec<(&str, Vec<Fluid2D>)> = vec![
        (
            "one fluid alone",
            vec![Fluid2D {
                size: Vec2::new(5.5, 2.0),
                ..fluid(1000, 0, 1.0, 0.0, 1)
            }],
        ),
        (
            "two of 500 in one group, layering (mixing 0)",
            vec![fluid(500, 1, 3.0, 0.0, 1), fluid(500, 1, 1.0, 0.0, 2)],
        ),
        (
            "two of 500 in one group, mixing (1)",
            vec![fluid(500, 1, 3.0, 1.0, 1), fluid(500, 1, 1.0, 1.0, 2)],
        ),
        (
            "four of 250 in one group",
            (0..4)
                .map(|i| fluid(250, 1, 1.0 + i as f32, 0.5, i))
                .collect(),
        ),
        (
            "eight of 125 alone",
            (0..8).map(|i| fluid(125, 0, 1.0, 0.0, i)).collect(),
        ),
        (
            "eight of 125 in two groups of four",
            (0..8)
                .map(|i| fluid(125, 1 + i / 4, 1.0 + (i % 4) as f32, 0.3, i))
                .collect(),
        ),
    ];
    for (label, fluids) in rows {
        let total: u32 = fluids.iter().map(|f| f.count).sum();
        let ms = time(&basin(fluids), &none) - base2;
        println!("| {label} | {total} | {ms:.3} |");
    }

    println!("\n| Ragdolls, knocked over on the floor | bones | ms per tick |");
    println!("|---|---|---|");
    let figure = |profile: u32| {
        let down = if profile == PROFILE_2D {
            Quat::from_xyzw(0.0, 0.0, 1.0, 0.0)
        } else {
            Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)
        };
        let bone = |parent: Option<u32>, x: f32, y: f32, rotation: Quat, length: f32| RagdollBone {
            parent,
            rest: (Vec3::new(x, y, 0.0), rotation),
            length,
            axis: Vec3::Y,
        };
        RagdollRig {
            bones: vec![
                bone(None, 0.0, 1.0, Quat::IDENTITY, 0.3),
                bone(Some(0), 0.0, 1.3, Quat::IDENTITY, 0.5),
                bone(Some(1), 0.0, 1.8, Quat::IDENTITY, 0.3),
                bone(Some(1), 0.25, 1.75, down, 0.3),
                bone(Some(3), 0.25, 1.45, down, 0.3),
                bone(Some(1), -0.25, 1.75, down, 0.3),
                bone(Some(5), -0.25, 1.45, down, 0.3),
                bone(Some(0), 0.1, 1.0, down, 0.45),
                bone(Some(7), 0.1, 0.55, down, 0.45),
                bone(Some(0), -0.1, 1.0, down, 0.45),
                bone(Some(9), -0.1, 0.55, down, 0.45),
            ],
        }
    };
    for (profile, count) in [
        (PROFILE_2D, 1u32),
        (PROFILE_2D, 8),
        (PROFILE_3D, 1),
        (PROFILE_3D, 8),
    ] {
        let build = move || {
            let (mut w, p) = world(profile);
            for i in 0..count {
                let e = w
                    .spawn((
                        at(
                            i as f32 * 3.0,
                            0.0,
                            if profile == PROFILE_2D { 0.0 } else { i as f32 },
                        ),
                        Ragdoll {
                            weight: 1.0,
                            impulse: Vec3::new(20.0, 5.0, 0.0),
                            ..Ragdoll::default()
                        },
                    ))
                    .unwrap();
                p.set_ragdoll_rig(e, figure(profile)).unwrap();
            }
            (w, p)
        };
        let base = if profile == PROFILE_2D { base2 } else { base3 };
        let ms = time(&build, &none) - base;
        println!(
            "| {count} in {} | {} | {ms:.3} |",
            if profile == PROFILE_2D { "2D" } else { "3D" },
            count * 11
        );
    }
}
