//! The physics step with lanes off and on (tests/evidence/P4/physics-lanes/):
//! the 500-body pile of the determinism test (`tests/support/pile.rs`:
//! a kinematic paddle, a sensor, one body churned every 20 ticks) and pens
//! of `n` dynamic bodies of the same mixed shapes dropped on a floor, each
//! stepped 240 ticks to fall, then timed over 600 (the mean per tick of
//! `World::step`, physics and its ECS around it). The lanes are what
//! `OG_LANES` gives the process, read once, so each mode is its own run;
//! the world hash at the end must be the same in every mode. Release builds
//! only give meaningful numbers:
//!
//!   OG_LANES=off cargo run --release -p og-physics --example lanes_costs
//!   OG_LANES=4   cargo run --release -p og-physics --example lanes_costs
//!   OG_LANES=8   cargo run --release -p og-physics --example lanes_costs
//!
//! Arguments, when given, are the pen sizes (default 16 64 128 256 512
//! 1000 2000).

#[path = "../tests/support/mod.rs"]
mod support;

use og_ecs::World;
use std::time::Instant;
use support::pile;

const WARM: u32 = 240;
const TIMED: u32 = 600;

/// Milliseconds per tick over `TIMED` ticks after `WARM`, and the hash.
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

fn main() {
    let sizes: Vec<usize> = std::env::args()
        .skip(1)
        .map(|a| a.parse().expect("a pen size"))
        .collect();
    let sizes = if sizes.is_empty() {
        vec![16, 64, 128, 256, 512, 1000, 2000]
    } else {
        sizes
    };
    let lanes = og_lanes::report();
    let mode = if lanes.on {
        format!("on, {} threads", lanes.threads)
    } else {
        format!("off ({})", lanes.why)
    };
    println!("lanes {mode}");
    println!("| scene | bodies | ms per tick | world hash | physics calls |");
    println!("|---|---|---|---|---|");
    let calls = || og_lanes::report().sites[og_lanes::Site::Physics as usize].calls;
    let before = calls();
    let (ms, hash) = time(pile::build(42).0);
    println!(
        "| the pile (paddle, sensor, churn) | {} | {ms:.3} | {hash:016x} | {} |",
        pile::BODIES,
        calls() - before
    );
    for n in sizes {
        let before = calls();
        let (ms, hash) = time(pile::pen(n, 7).0);
        println!(
            "| a pen | {n} | {ms:.3} | {hash:016x} | {} |",
            calls() - before
        );
    }
}
