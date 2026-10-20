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
//! 1000 2000); `--forced` puts every step on the pool
//! (`og_physics::set_grains(Some(og_physics::Grains::always()))`), to find where the pool pays.
//! `--bodies N`, `--pairs N` and `--rows N` set those grains (the others measured),
//! to find where splitting the sync and the readout by ranges pays.
//! `--phases` prints, for each scene, the mean of each phase of the step
//! over the timed ticks (`PhysicsSystem::stats`) and whether it split.

#[path = "../tests/support/mod.rs"]
mod support;

use og_ecs::World;
use og_physics::{Phase, PhysicsSystem};
use std::time::Instant;
use support::pile;

const WARM: u32 = 240;
const TIMED: u32 = 600;

/// Milliseconds per tick over `TIMED` ticks after `WARM`, the hash, and
/// each phase's mean ms and whether it split on any tick.
fn time((mut w, physics): (World, PhysicsSystem)) -> (f64, u64, Vec<(f64, bool)>) {
    for _ in 0..WARM {
        w.step().unwrap();
    }
    let mut phases = vec![(0.0, false); Phase::COUNT];
    let t = Instant::now();
    for _ in 0..TIMED {
        w.step().unwrap();
        for (sum, p) in phases.iter_mut().zip(physics.stats().phases) {
            sum.0 += p.ms / TIMED as f64;
            sum.1 |= p.split;
        }
    }
    (
        t.elapsed().as_secs_f64() * 1000.0 / TIMED as f64,
        w.world_hash(),
        phases,
    )
}

fn phase_row(scene: &str, phases: &[(f64, bool)]) {
    let cells: Vec<String> = phases
        .iter()
        .map(|(ms, split)| format!("{ms:.3}{}", if *split { "*" } else { "" }))
        .collect();
    println!("| {scene} | {} |", cells.join(" | "));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let forced = args.iter().any(|a| a == "--forced");
    let show_phases = args.iter().any(|a| a == "--phases");
    if forced {
        og_physics::set_grains(Some(og_physics::Grains::always()));
    }
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .map(|i| args[i + 1].parse::<usize>().expect("a grain"))
    };
    let (bodies, pairs, rows) = (value("--bodies"), value("--pairs"), value("--rows"));
    if bodies.is_some() || pairs.is_some() || rows.is_some() {
        let threads = og_lanes::threads();
        let measured = og_physics::Grains::measured(threads);
        og_physics::set_grains(Some(og_physics::Grains {
            bodies: bodies.unwrap_or(measured.bodies),
            pairs: pairs.unwrap_or(measured.pairs),
            rows: rows.unwrap_or(measured.rows),
            ..if forced {
                og_physics::Grains::always()
            } else {
                measured
            }
        }));
    }
    let sizes: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            !a.starts_with("--")
                && (*i == 0 || !matches!(args[i - 1].as_str(), "--bodies" | "--pairs" | "--rows"))
        })
        .map(|(_, a)| a.parse().expect("a pen size"))
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
    let from = og_physics::pool_bodies(lanes.threads);
    let from = if forced {
        "every step on the pool".to_string()
    } else if from == usize::MAX {
        "never on the pool".to_string()
    } else {
        format!("on the pool from {from} bodies")
    };
    println!("lanes {mode}; {from}");
    println!("| scene | bodies | ms per tick | world hash | physics calls |");
    println!("|---|---|---|---|---|");
    let calls = || og_lanes::report().sites[og_lanes::Site::Physics as usize].calls;
    let mut rows = Vec::new();
    let before = calls();
    let (ms, hash, phases) = time(pile::build(42));
    println!(
        "| the pile (paddle, sensor, churn) | {} | {ms:.3} | {hash:016x} | {} |",
        pile::BODIES,
        calls() - before
    );
    rows.push((format!("the pile ({})", pile::BODIES), phases));
    for n in sizes {
        let before = calls();
        let (ms, hash, phases) = time(pile::pen(n, 7));
        println!(
            "| a pen | {n} | {ms:.3} | {hash:016x} | {} |",
            calls() - before
        );
        rows.push((format!("a pen of {n}"), phases));
    }
    if show_phases {
        println!();
        println!("ms per tick by phase (`*`: split on the pool on some tick)");
        let names: Vec<&str> = Phase::ALL.iter().map(|p| p.name()).collect();
        println!("| scene | {} |", names.join(" | "));
        println!("|---|{}", "---|".repeat(names.len()));
        for (scene, phases) in rows {
            phase_row(&scene, &phases);
        }
    }
}
