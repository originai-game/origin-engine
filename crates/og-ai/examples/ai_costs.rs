//! The cost numbers of tests/evidence/E6/ai-costs.json: 200 guards running
//! a behaviour tree (flee, bite, chase, patrol) with 120° sight against a
//! wall ray, on a 64 × 64 grid through og.nav, chasing four moving heroes;
//! the tick with og.ai thinking every tick, every third tick, and with the
//! agents inactive (og.nav alone).
//!
//!     cargo run --release -p og-ai --example ai_costs > tests/evidence/E6/ai-costs.json

use og_ai::{AiAgent, AiDesc, Behaviour, Perceivable, Perception, SightRay, install};
use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World, WorldDesc};
use og_nav::system::Transform;
use og_nav::{Grid, Nav, NavAgent, NavDesc, Navs, Plane};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "Health")]
struct Health {
    hp: f32,
}

const GUARD: &str = r#"
type behaviour
version 1
---
name "guard"
route beat {
  point 4 4 0
  point 60 4 0
  point 60 60 0
  point 4 60 0
}
tree {
  selector {
    sequence flee {
      check Health.hp below 20
      flee {
        distance 6
      }
    }
    sequence bite {
      check sees
      check targetDistance below 1
      command bite {
        cooldown 0.5
      }
    }
    sequence chase {
      check sees
      chase {
        speed 3
      }
    }
    patrol beat {
      wait 0.5
      speed 2
    }
  }
}
"#;

fn world(think_every: u32, active: bool) -> (World, Vec<Entity>) {
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 3,
    })
    .unwrap();
    world.register::<Health>().unwrap();
    // A wall along x = 32 with a gap: sight rays crossing it are blocked.
    let ray: SightRay = Arc::new(|o: Vec3, d: Vec3, max: f32, _m: u32, _e: Entity| {
        if d.x.abs() < 1e-6 {
            return None;
        }
        let t = (32.0 - o.x) / d.x;
        let y = o.y + d.y * t;
        (t > 0.0 && t < max && !(28.0..36.0).contains(&y)).then_some((t, Entity::from_bits(1)))
    });
    let ai = install(
        &mut world,
        &["Health".to_string()],
        AiDesc {
            plane: Plane::Xy,
            ray: Some(ray),
            sink: Some(Arc::new(|_: &str, _: Vec<String>| {})),
            behaviours: None,
        },
    )
    .unwrap();
    let mut grid = Grid::new(64, 64, 1.0, Vec2::ZERO);
    for y in 0..64 {
        if !(28..36).contains(&y) {
            grid.set(32, y, og_nav::BLOCKED);
        }
    }
    let mut navs = Navs::default();
    let g = navs.insert(Nav::Grid(grid));
    og_nav::install(
        &mut world,
        Arc::new(Mutex::new(navs)),
        NavDesc {
            plane: Plane::Xy,
            raycast: None,
            plans_per_tick: og_nav::PLANS_PER_TICK,
        },
    )
    .unwrap();
    let b = ai
        .behaviours
        .lock()
        .unwrap()
        .insert(Behaviour::parse(GUARD).unwrap());
    for k in 0..200u32 {
        let p = Vec3::new(
            2.0 + (k % 20) as f32 * 3.0,
            2.0 + (k / 20) as f32 * 6.0,
            0.0,
        );
        world
            .spawn((
                Transform {
                    position: p,
                    ..Default::default()
                },
                NavAgent {
                    nav: g,
                    speed: 2.0,
                    acceleration: 30.0,
                    radius: 0.3,
                    ..Default::default()
                },
                AiAgent {
                    behaviour: b,
                    active,
                    think_every,
                    ..Default::default()
                },
                Perception {
                    sight: 8.0,
                    fov: 120.0,
                    team: 2,
                    ..Default::default()
                },
                Health { hp: 50.0 },
            ))
            .unwrap();
    }
    let heroes = (0..4)
        .map(|k| {
            world
                .spawn((
                    Transform {
                        position: Vec3::new(10.0 + k as f32 * 14.0, 30.0, 0.0),
                        ..Default::default()
                    },
                    Perceivable {
                        team: 1,
                        eye_height: 0.0,
                    },
                ))
                .unwrap()
        })
        .collect();
    (world, heroes)
}

/// Mean ms per tick over `ticks`, after `warm` ticks, the heroes circling.
// A cost example: its scene setup never runs in the tick.
#[allow(clippy::disallowed_methods)]
fn run(think_every: u32, active: bool, warm: u32, ticks: u32) -> f64 {
    let (mut w, heroes) = world(think_every, active);
    let mut total = 0.0;
    for t in 0..warm + ticks {
        for (k, &h) in heroes.iter().enumerate() {
            let a = t as f32 * 0.01 + k as f32 * 1.57;
            let p = Vec3::new(32.0 + 20.0 * a.cos(), 32.0 + 20.0 * a.sin(), 0.0);
            w.insert(
                h,
                Transform {
                    position: p,
                    ..Default::default()
                },
            )
            .unwrap();
        }
        let start = Instant::now();
        w.step().unwrap();
        if t >= warm {
            total += start.elapsed().as_secs_f64() * 1000.0;
        }
    }
    total / f64::from(ticks)
}

fn main() {
    let every = run(1, true, 60, 600);
    let third = run(3, true, 60, 600);
    let nav_only = run(1, false, 60, 600);
    println!(
        "{{\n  \"agents\": 200,\n  \"grid\": \"64x64, a wall with a gap\",\n  \"tickMs.thinkEvery1\": {:.3},\n  \"tickMs.thinkEvery3\": {:.3},\n  \"tickMs.navOnly\": {:.3},\n  \"aiMsPerAgentThink\": {:.5}\n}}",
        every,
        third,
        nav_only,
        (every - nav_only).max(0.0) / 200.0
    );
}
