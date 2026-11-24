//! og-ai through its public API: a guard's behaviour parsed from text and
//! ticked in a world with og.nav, the way the runtime mounts them.
//!
//! Ways it could fail: two runs or a restored snapshot decide differently
//! (state kept outside the world, an iteration order that is not fixed).

use og_ai::nav::system::Transform;
use og_ai::nav::{Grid, Nav, NavAgent, NavDesc, Navs, Plane};
use og_ai::{AiAgent, AiDesc, Behaviour, Perceivable, Perception, SightRay, install};
use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World, WorldDesc};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "Health")]
struct Health {
    hp: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "Game")]
struct Game {
    alarm: bool,
    level: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "Tuning")]
struct Tuning {
    flee: f32,
}

const GUARD: &str = r#"
type behaviour
version 1
---
name "guard"
key bites 0
route beat {
  point 2 2 0
  point 8 2 0
}
tree {
  ReactiveFallback {
    ReactiveSequence flee {
      check Health.hp below 20
      flee {
        distance 6
      }
    }
    ReactiveSequence attack {
      check sees
      check targetDistance below 1.2
      command bite {
        cooldown 0.5
      }
    }
    ReactiveSequence chase {
      check sees
      chase {
        speed 3
      }
    }
    ReactiveSequence alarm {
      check Game.alarm
      move_to 5 9 0
    }
    patrol beat {
      wait 0.5
      speed 2
    }
  }
}
"#;

struct Scene {
    world: World,
    ai: og_ai::AiSystem,
    guard: Entity,
    hero: Entity,
}

/// A 2D room 20 × 12 with a guard and a hero; `wall_at_x` hides anything across x = 10 from the guard's rays.
fn scene(behaviour: &str, wall: bool) -> Scene {
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 9,
    })
    .unwrap();
    world.register::<Health>().unwrap();
    world.register::<Game>().unwrap();
    world.insert_resource(Game::default()).unwrap();
    world.register::<Tuning>().unwrap();
    world.insert_resource(Tuning { flee: 20.0 }).unwrap();
    let ray: SightRay = Arc::new(move |o: Vec3, d: Vec3, max: f32, _mask: u32, _ex: Entity| {
        if !wall || d.x.abs() < 1e-6 {
            return None;
        }
        let t = (10.0 - o.x) / d.x;
        (t > 0.0 && t < max).then_some((t, Entity::from_bits(0xdead)))
    });
    let mut records = vec![
        "Health".to_string(),
        "Game".to_string(),
        "Tuning".to_string(),
    ];
    records.sort();
    let ai = install(
        &mut world,
        &records,
        AiDesc {
            plane: Plane::Xy,
            ray: Some(ray),
            sink: None,
            behaviours: None,
            hearing: None,
        },
    )
    .unwrap();
    let mut navs = Navs::default();
    let grid = navs.insert(Nav::Grid(Grid::new(20, 12, 1.0, Vec2::ZERO)));
    og_ai::nav::install(
        &mut world,
        Arc::new(Mutex::new(navs)),
        NavDesc {
            plane: Plane::Xy,
            raycast: None,
            plans_per_tick: 0,
        },
    )
    .unwrap();
    let b = ai
        .behaviours
        .lock()
        .unwrap()
        .insert(Behaviour::parse(behaviour).unwrap());
    let guard = world
        .spawn((
            Transform {
                position: Vec3::new(2.0, 2.0, 0.0),
                ..Default::default()
            },
            NavAgent {
                nav: grid,
                speed: 2.0,
                acceleration: 60.0,
                stop_distance: 0.1,
                ..Default::default()
            },
            AiAgent {
                behaviour: b,
                ..Default::default()
            },
            Perception {
                sight: 6.0,
                fov: 120.0,
                team: 2,
                ..Default::default()
            },
            Health { hp: 50.0 },
        ))
        .unwrap();
    let hero = world
        .spawn((
            Transform {
                position: Vec3::new(18.0, 10.0, 0.0),
                ..Default::default()
            },
            Perceivable {
                team: 1,
                eye_height: 0.0,
            },
        ))
        .unwrap();
    Scene {
        world,
        ai,
        guard,
        hero,
    }
}

#[test]
fn two_runs_and_a_restored_snapshot_decide_the_same() {
    let run = |snap_at: Option<u32>| {
        let mut s = scene(GUARD, true);
        let mut snap = None;
        let mut cmds = Vec::new();
        for t in 0..600u32 {
            if t == 200 {
                s.world
                    .insert(
                        s.hero,
                        Transform {
                            position: Vec3::new(9.0, 3.0, 0.0),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            if t == 400 {
                s.world.insert(s.guard, Health { hp: 5.0 }).unwrap();
            }
            if Some(t) == snap_at {
                snap = Some(s.world.snapshot());
            }
            s.world.step().unwrap();
            cmds.extend(s.ai.take_commands().into_iter().map(|c| (t, c.0)));
        }
        (s.world.world_hash(), cmds, snap)
    };
    let (h1, c1, snap) = run(Some(300));
    let (h2, c2, _) = run(None);
    assert_eq!(h1, h2, "two runs end in different worlds");
    assert_eq!(c1, c2, "two runs queued different commands");
    assert!(!c1.is_empty(), "the guard bit at least once");
    // A fresh scene restored at tick 300 ends where the first did.
    let mut s = scene(GUARD, true);
    s.world.restore(&snap.unwrap()).unwrap();
    for t in 300..600u32 {
        if t == 400 {
            s.world.insert(s.guard, Health { hp: 5.0 }).unwrap();
        }
        s.world.step().unwrap();
    }
    assert_eq!(
        s.world.world_hash(),
        h1,
        "the restored world decided differently"
    );
}
