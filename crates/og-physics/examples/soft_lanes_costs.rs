//! The soft bodies on the lanes (tests/evidence/P4/physics-lanes-2/): ropes,
//! cloths, 3D and 2D fluids and ragdolls, each scene stepped 120 ticks to
//! settle and timed over 300 (the mean per tick of `World::step`), with
//! the grains as measured (`Grains::measured`) or forced to split one kind
//! of pass, the body step left where the measured grains put it:
//!
//!   OG_LANES=off cargo run --release -p og-physics --example soft_lanes_costs
//!   OG_LANES=4   cargo run --release -p og-physics --example soft_lanes_costs -- --split
//!   OG_LANES=4   cargo run --release -p og-physics --example soft_lanes_costs -- --particles 512
//!
//! `--split` splits ropes, cloths and fluid sets side by side from two of
//! them and the fluids' per-particle passes from two grains of
//! `--particles` (default the measured grain); without it, the measured
//! grains decide. Each scene's world hash must be the same in every mode.

#[path = "../tests/support/mod.rs"]
mod support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::time::Instant;
use support::{at, beyond, floor, spawn_static};

const WARM: u32 = 120;
const TIMED: u32 = 300;

fn time(mut w: World) -> (f64, u64) {
    for _ in 0..WARM {
        w.step().unwrap();
    }
    let t = Instant::now();
    for _ in 0..TIMED {
        w.step().unwrap();
    }
    (
        t.elapsed().as_secs_f64() * 1000.0 / TIMED as f64,
        w.world_hash(),
    )
}

fn world3() -> World {
    let (mut w, _p) = beyond::new_world(5, PROFILE_3D);
    floor(&mut w);
    spawn_static(
        &mut w,
        at(0.0, 1.0, 0.0),
        Collider::cuboid(Vec3::new(30.0, 0.4, 0.6)),
    );
    w
}

fn ropes(n: usize) -> World {
    let mut w = world3();
    for i in 0..n {
        let (x, z) = (-15.0 + (i % 30) as f32, -3.0 + (i / 30) as f32);
        w.spawn((
            at(x, 4.0, z),
            Rope {
                wind: Vec3::new(0.6, 0.0, 0.3),
                ..Rope::default()
            },
        ))
        .unwrap();
    }
    w
}

fn cloths(n: usize) -> World {
    let mut w = world3();
    for i in 0..n {
        let (x, z) = (-15.0 + 2.0 * (i % 15) as f32, -3.0 + 2.0 * (i / 15) as f32);
        w.spawn((
            at(x, 2.5, z),
            Cloth {
                wind: Vec3::new(0.0, 0.0, 1.5),
                gust: 0.5,
                ..Cloth::default()
            },
        ))
        .unwrap();
    }
    w
}

fn fluids3(n: usize, count: u32) -> World {
    let mut w = world3();
    for i in 0..n {
        let s = 1.0 + og_core::math::pow(count as f32 / 2000.0, 1.0 / 3.0);
        w.spawn((
            at(
                -12.0 + 6.0 * (i % 5) as f32,
                1.5 + s * 0.5,
                -8.0 - 6.0 * (i / 5) as f32,
            ),
            Fluid3D {
                count,
                size: Vec3::new(s, s * 0.5, s),
                bounds: Vec3::new(s * 1.5, 8.0, s * 1.5),
                ..Fluid3D::default()
            },
        ))
        .unwrap();
    }
    w
}

fn fluids2(n: usize, count: u32) -> World {
    let (mut w, _p) = beyond::new_world(6, PROFILE_2D);
    floor(&mut w);
    for i in 0..n {
        let s = 2.0 * (count as f32 / 1000.0).sqrt();
        w.spawn((
            at(-30.0 + (s * 2.0 + 1.0) * i as f32, 1.0 + s * 0.5, 0.0),
            Fluid2D {
                count,
                size: Vec2::new(s, s * 0.75),
                bounds: Vec2::new(s * 1.5, 12.0),
                ..Fluid2D::default()
            },
        ))
        .unwrap();
    }
    w
}

fn ragdolls(n: usize) -> World {
    let (mut w, physics) = beyond::new_world(8, PROFILE_3D);
    floor(&mut w);
    for i in 0..n {
        let (x, z) = (-20.0 + 2.0 * (i % 20) as f32, -20.0 + 2.0 * (i / 20) as f32);
        let e = w
            .spawn((
                at(x, 0.0, z),
                Ragdoll {
                    weight: if i % 2 == 0 { 0.0 } else { 1.0 },
                    ..Ragdoll::default()
                },
            ))
            .unwrap();
        physics.set_ragdoll_rig(e, beyond::rig()).unwrap();
    }
    w
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let split = args.iter().any(|a| a == "--split");
    let lanes = og_lanes::report();
    let mut grains = Grains::measured(lanes.threads);
    if let Some(k) = args.iter().position(|a| a == "--particles") {
        let n = args[k + 1].parse().expect("--particles N");
        grains.particles_2d = n;
        grains.particles_3d = n;
    }
    if split {
        grains.rope_work = 1;
        grains.cloth_work = 1;
        grains.fluid_work = 1;
    }
    set_grains(Some(grains));
    let mode = if lanes.on {
        format!("on, {} threads", lanes.threads)
    } else {
        format!("off ({})", lanes.why)
    };
    println!(
        "lanes {mode}; {} (particles 2D {}, 3D {})",
        if split {
            "soft bodies split from two"
        } else {
            "the measured grains"
        },
        grains.particles_2d,
        grains.particles_3d
    );
    println!("| scene | work | ms per tick | world hash | physics calls |");
    println!("|---|---|---|---|---|");
    let calls = || og_lanes::report().sites[og_lanes::Site::Physics as usize].calls;
    let row = |label: String, work: usize, w: World| {
        let before = calls();
        let (ms, hash) = time(w);
        println!(
            "| {label} | {work} | {ms:.3} | {hash:016x} | {} |",
            calls() - before
        );
    };
    for n in [8, 16, 32, 64, 128] {
        row(
            format!("{n} ropes (17 points, 12 iterations)"),
            n * 17 * 12,
            ropes(n),
        );
    }
    for n in [2, 4, 8, 16, 32] {
        row(
            format!("{n} cloths (12 × 12, 8 iterations)"),
            n * 144 * 8,
            cloths(n),
        );
    }
    for (n, count) in [
        (2, 250),
        (2, 500),
        (4, 500),
        (2, 1000),
        (4, 1000),
        (8, 1000),
    ] {
        row(
            format!("{n} 3D fluids of {count} (2 substeps)"),
            n * count * 2,
            fluids3(n, count as u32),
        );
    }
    for count in [1000, 2000, 4000, 8000] {
        row(
            format!("a 3D fluid of {count} (2 substeps)"),
            count * 2,
            fluids3(1, count as u32),
        );
    }
    for (n, count) in [(2, 250), (2, 500), (4, 500), (4, 1000)] {
        row(
            format!("{n} 2D fluids of {count} (2 substeps)"),
            n * count * 2,
            fluids2(n, count as u32),
        );
    }
    for count in [1000, 2000, 4000, 8000] {
        row(
            format!("a 2D fluid of {count} (2 substeps)"),
            count * 2,
            fluids2(1, count as u32),
        );
    }
    for n in [16, 64, 256] {
        row(
            format!("{n} ragdolls (3 bones, half limp)"),
            n * 3,
            ragdolls(n),
        );
    }
}
