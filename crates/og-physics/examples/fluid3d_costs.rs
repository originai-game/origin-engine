//! The CPU cost of the 3D fluid per 60 Hz tick (the phase 2 SPH round's
//! cost table, `tests/evidence/P2/sph/costs.md`): each scene poured over a
//! floor (a 3D box, or a terrain bowl), stepped 120 ticks to settle, then
//! timed over short windows of 30 ticks, the best of ten (the machine is
//! shared; the best window is the one least disturbed). Release builds
//! only give meaningful numbers:
//!
//!   cargo run --release -p og-physics --example fluid3d_costs

use og_core::glam::{Vec2, Vec3};
use og_ecs::{World, WorldDesc};
use og_physics::*;
use std::sync::Arc;
use std::time::Instant;

fn world(bowl: bool) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    let p = install(&mut w, Physics::default()).unwrap();
    if bowl {
        let (n, size) = (33u32, 12.0f32);
        let mut heights = Vec::new();
        for r in 0..n {
            for c in 0..n {
                let x = -size * 0.5 + c as f32 * size / (n - 1) as f32;
                let z = -size * 0.5 + r as f32 * size / (n - 1) as f32;
                heights.push(0.1 * (x * x + z * z));
            }
        }
        p.set_heightfield(
            1,
            HeightfieldDesc {
                columns: n,
                rows: n,
                size: Vec2::splat(size),
                heights: Arc::from(heights),
            },
        )
        .unwrap();
        w.spawn((
            Transform::default(),
            Collider {
                shape: SHAPE_HEIGHTFIELD,
                mesh: 1,
                ..Collider::default()
            },
        ))
        .unwrap();
    } else {
        w.spawn((
            Transform::at(Vec3::new(0.0, -0.5, 0.0)),
            Collider::cuboid(Vec3::new(60.0, 1.0, 60.0)),
        ))
        .unwrap();
    }
    (w, p)
}

/// Milliseconds per tick: the best of ten windows of 30 ticks after 120.
fn time(w: &mut World) -> f64 {
    for _ in 0..120 {
        w.step().unwrap();
    }
    let mut best = f64::MAX;
    for _ in 0..10 {
        let t = Instant::now();
        for _ in 0..30 {
            w.step().unwrap();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / 30.0);
    }
    best
}

/// A block of `count` particles at the default spacing, in bounds four
/// times its footprint.
fn block(count: u32, substeps: u32) -> Fluid3D {
    let side = (count as f32 / 2000.0).cbrt();
    Fluid3D {
        count,
        size: Vec3::new(2.0, 1.0, 2.0) * side,
        bounds: Vec3::new(4.0 * side, 0.0, 4.0 * side),
        substeps,
        max_count: count.max(8192),
        ..Fluid3D::default()
    }
}

fn main() {
    let at = |y: f32| Transform::at(Vec3::new(0.0, y, 0.0));
    let empty = time(&mut world(false).0);
    println!("empty world (a floor): {empty:.3} ms per tick");
    println!();
    println!("| Scene | particles | substeps | ms per tick |");
    println!("|---|---|---|---|");
    for count in [500u32, 1000, 2000, 4000, 8000] {
        for substeps in [1u32, 2] {
            let (mut w, p) = world(false);
            let e = w.spawn((at(1.0), block(count, substeps))).unwrap();
            let ms = time(&mut w) - empty;
            let n = p.fluid3d_particles(e).unwrap().len();
            println!("| one fluid on a floor | {n} | {substeps} | {ms:.3} |");
        }
    }
    {
        let (mut w, p) = world(true);
        let e = w
            .spawn((
                at(2.0),
                Fluid3D {
                    bounds: Vec3::ZERO,
                    ..block(2000, 2)
                },
            ))
            .unwrap();
        let ms = time(&mut w) - empty;
        let n = p.fluid3d_particles(e).unwrap().len();
        println!("| one fluid in a terrain bowl | {n} | 2 | {ms:.3} |");
    }
    {
        let (mut w, p) = world(false);
        let mut n = 0;
        let mut es = Vec::new();
        for (y, density, seed) in [(2.2, 3.0, 1), (1.0, 1.0, 2)] {
            es.push(
                w.spawn((
                    at(y),
                    Fluid3D {
                        group: 1,
                        density,
                        seed,
                        bounds: Vec3::new(2.5, 0.0, 2.5),
                        ..block(1000, 2)
                    },
                ))
                .unwrap(),
            );
        }
        let ms = time(&mut w) - empty;
        for e in es {
            n += p.fluid3d_particles(e).unwrap().len();
        }
        println!("| two of 1000 in one group, layering (mixing 0) | {n} | 2 | {ms:.3} |");
    }
    {
        let (mut w, p) = world(false);
        let e = w
            .spawn((
                at(1.0),
                Fluid3D {
                    count: 0,
                    max_count: 2000,
                    emit_rate: 600.0,
                    emit_offset: Vec3::new(0.0, 1.0, 0.0),
                    sink_offset: Vec3::new(1.5, -0.9, 0.0),
                    sink_size: Vec3::new(0.6, 0.4, 3.0),
                    bounds: Vec3::new(4.0, 0.0, 4.0),
                    ..block(0, 2)
                },
            ))
            .unwrap();
        let ms = time(&mut w) - empty;
        let n = p.fluid3d_particles(e).unwrap().len();
        println!("| a 600/s emitter into a sink (alive at the end) | {n} | 2 | {ms:.3} |");
    }
}
