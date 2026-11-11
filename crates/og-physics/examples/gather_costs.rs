//! What reading the physics components out of the world costs each tick
//! (the step's `gather` phase, with `sync` beside it, which diffs what was
//! read against rapier's world) on levels that mostly stand still: `n`
//! static colliders (boxes, spheres and capsules on a grid, three in four
//! on entities of one archetype, the rest on entities with a marker beside
//! them) under 64 of the pile's bodies dropped on a floor, and a pen of `n`
//! dynamic bodies (to 5000) and no more statics than its walls. Each is stepped 240
//! ticks to settle, then timed over 600 (the means per tick of
//! `PhysicsSystem::stats`); the world hash at the end is what a change to
//! the gather must not move. Release builds only give meaningful numbers:
//!
//!   OG_LANES=off cargo run --release -p og-physics --example gather_costs
//!
//! Arguments, when given, are the sizes (default 1000 5000 20000).
//! `--moving` also moves one static collider a tick (a platform game code
//! carries): its og.Transform is written every tick, its archetype's
//! og.Collider columns never.

#[path = "../tests/support/mod.rs"]
mod support;

use og_core::Reflect;
use og_core::glam::Vec3;
use og_ecs::{Entity, QueryDesc, SystemDesc, World};
use og_physics::{Collider, Phase, PhysicsSystem, Transform};
use std::time::Instant;
use support::{at, pile, spawn_static};

const WARM: u32 = 240;
const TIMED: u32 = 600;
const BODIES: usize = 64;
const PEN_MAX: usize = 5000;

/// Beside a quarter of the statics, so they are a second archetype.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Prop {}

/// The pen of [`BODIES`] bodies with `n` static colliders on a grid 3 m
/// apart around it, below the floor's top so nothing rests on them.
fn level(n: usize, moving: bool) -> (World, PhysicsSystem) {
    let (mut w, physics) = pile::pen(BODIES, 7);
    w.register::<Prop>().unwrap();
    let side = (n as f32).sqrt().ceil() as usize;
    let mut first: Option<Entity> = None;
    for i in 0..n {
        let (x, z) = ((i % side) as f32, (i / side) as f32);
        let t = at(
            (x - side as f32 * 0.5) * 3.0,
            -3.0,
            (z - side as f32 * 0.5) * 3.0,
        );
        let collider = match i % 3 {
            0 => Collider::cuboid(Vec3::new(1.0, 1.0, 1.0)),
            1 => Collider::sphere(0.5),
            _ => Collider::capsule(0.3, 0.5),
        };
        if i % 4 == 3 {
            w.spawn((t, collider, Prop {})).unwrap();
        } else {
            let e = spawn_static(&mut w, t, collider);
            first.get_or_insert(e);
        }
    }
    if let (true, Some(platform)) = (moving, first) {
        w.add_system(
            SystemDesc::new("platform")
                .write::<Transform>()
                .query(QueryDesc::new().write::<Transform>()),
            move |ctx| {
                let mut t = ctx.get::<Transform>(platform)?;
                t.position.y = -3.0 - (ctx.time().tick % 120) as f32 * 0.01;
                ctx.set(platform, t)
            },
        )
        .unwrap();
    }
    (w, physics)
}

/// The mean ms per tick of the gather, the sync and the whole physics
/// system over `TIMED` ticks after `WARM`, the mean of `World::step`, and
/// the world hash at the end.
fn time((mut w, physics): (World, PhysicsSystem)) -> (f64, f64, f64, f64, u64) {
    for _ in 0..WARM {
        w.step().unwrap();
    }
    let (mut gather, mut sync, mut system) = (0.0, 0.0, 0.0);
    let t = Instant::now();
    for _ in 0..TIMED {
        w.step().unwrap();
        let stats = physics.stats();
        gather += stats.phase(Phase::Gather).ms;
        sync += stats.phase(Phase::Sync).ms;
        system += stats.ms;
    }
    let n = f64::from(TIMED);
    (
        gather / n,
        sync / n,
        system / n,
        t.elapsed().as_secs_f64() * 1000.0 / n,
        w.world_hash(),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let moving = args.iter().any(|a| a == "--moving");
    let sizes: Vec<usize> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(|a| a.parse().expect("a size"))
        .collect();
    let sizes = if sizes.is_empty() {
        vec![1000, 5000, 20000]
    } else {
        sizes
    };
    let lanes = og_lanes::report();
    println!(
        "lanes {}{}",
        if lanes.on {
            format!("on, {} threads", lanes.threads)
        } else {
            format!("off ({})", lanes.why)
        },
        if moving {
            "; one static collider moves every tick"
        } else {
            ""
        }
    );
    println!("| scene | gather ms | sync ms | og.physics ms | tick ms | world hash |");
    println!("|---|---|---|---|---|---|");
    let row = |scene: String, (gather, sync, system, tick, hash): (f64, f64, f64, f64, u64)| {
        println!("| {scene} | {gather:.4} | {sync:.4} | {system:.4} | {tick:.4} | {hash:016x} |");
    };
    for &n in &sizes {
        row(
            format!("{n} statics, {BODIES} bodies"),
            time(level(n, moving)),
        );
    }
    // Larger pens only time rapier's step.
    for &n in sizes.iter().filter(|n| **n <= PEN_MAX) {
        row(format!("a pen of {n} bodies"), time(pile::pen(n, 7)));
    }
}
