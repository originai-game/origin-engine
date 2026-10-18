//! og-ai through its public API: behaviours parsed from text, then ticked
//! in a world with og-nav, the way the runtime mounts them (og.ai before the
//! game's systems, og.nav after).
//!
//! Ways it could fail, each asserted below: a malformed behaviour accepted
//! (an unknown node, route, state, operator, or a built-in key set) or an
//! error without its line; a selector that keeps running a lower branch when
//! a higher one becomes true; a condition on a component or resource field
//! that reads the wrong bytes; a command fired every tick despite its
//! cooldown; sight through a wall, behind the agent or beyond its range;
//! hearing beyond its radius; a state machine that ignores its timers or its
//! enter/exit actions; two runs or a restored snapshot that decide
//! differently.

use og_ai::{
    AI_RUNNING, AiAgent, AiDesc, Behaviour, Noise, Perceivable, Perception, SightRay, Value,
    install,
};
use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World, WorldDesc};
use og_nav::system::Transform;
use og_nav::{Grid, Nav, NavAgent, NavDesc, Navs, Plane};
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
  selector {
    sequence flee {
      check Health.hp below 20
      flee {
        distance 6
      }
    }
    sequence attack {
      check sees
      check targetDistance below 1.2
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
    sequence alarm {
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

fn errors(text: &str) -> String {
    Behaviour::parse(text).expect_err("the behaviour should be refused")
}

#[test]
fn malformed_behaviours_are_refused_with_their_line_and_the_fix() {
    let head = "type behaviour\nversion 1\n---\n";
    let e = errors(&format!(
        "{head}tree {{\n  sequence {{\n    jump\n  }}\n}}\n"
    ));
    assert!(
        e.contains("line 6") && e.contains("'jump' is not a node"),
        "{e}"
    );
    let e = errors(&format!("{head}tree {{\n  patrol nowhere\n}}\n"));
    assert!(
        e.contains("line 5") && e.contains("no route 'nowhere'"),
        "{e}"
    );
    let e = errors(&format!("{head}tree {{\n  check hp under 3\n}}\n"));
    assert!(e.contains("'under' is not an operator"), "{e}");
    let e = errors(&format!("{head}tree {{\n  selector\n}}\n"));
    assert!(e.contains("selector needs children"), "{e}");
    let e = errors(&format!("{head}tree {{\n  set sees true\n}}\n"));
    assert!(e.contains("written by og.ai"), "{e}");
    let e = errors(&format!(
        "{head}machine {{\n  state a {{\n    to b\n  }}\n}}\n"
    ));
    assert!(e.contains("no state 'b'"), "{e}");
    let e = errors(&format!(
        "{head}tree {{\n  wait 1\n}}\nmachine {{\n  state a\n}}\n"
    ));
    assert!(e.contains("not both"), "{e}");
    let e = errors(&format!("{head}tree {{\n  command Bite\n}}\n"));
    assert!(e.contains("snake_case"), "{e}");
    let b = Behaviour::parse(GUARD).expect("the guard parses");
    assert_eq!(b.name, "guard");
    assert_eq!(
        b.field_operands(),
        vec!["Health.hp".to_string(), "Game.alarm".to_string()]
    );
}

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
    let ray: SightRay = Arc::new(move |o: Vec3, d: Vec3, max: f32, _mask: u32, _ex: Entity| {
        if !wall || d.x.abs() < 1e-6 {
            return None;
        }
        let t = (10.0 - o.x) / d.x;
        (t > 0.0 && t < max).then_some((t, Entity::from_bits(0xdead)))
    });
    let mut records = vec!["Health".to_string(), "Game".to_string()];
    records.sort();
    let ai = install(
        &mut world,
        &records,
        AiDesc {
            plane: Plane::Xy,
            ray: Some(ray),
            sink: None,
            behaviours: None,
        },
    )
    .unwrap();
    let mut navs = Navs::default();
    let grid = navs.insert(Nav::Grid(Grid::new(20, 12, 1.0, Vec2::ZERO)));
    og_nav::install(
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

fn path(s: &Scene) -> String {
    let b =
        s.ai.behaviours
            .lock()
            .unwrap()
            .get(s.world.get::<AiAgent>(s.guard).unwrap().behaviour)
            .unwrap();
    s.ai.describe(s.guard).unwrap().path(&b)
}

fn pos(s: &Scene, e: Entity) -> Vec3 {
    s.world.get::<Transform>(e).unwrap().position
}

fn steps(s: &mut Scene, n: u32) -> Vec<(String, Vec<String>)> {
    let mut cmds = Vec::new();
    for _ in 0..n {
        s.world.step().unwrap();
        cmds.extend(s.ai.take_commands());
    }
    cmds
}

#[test]
fn a_guard_patrols_chases_on_sight_bites_on_a_cooldown_and_flees_when_hurt() {
    let mut s = scene(GUARD, false);
    steps(&mut s, 120);
    assert_eq!(
        path(&s),
        "selector > patrol beat",
        "it patrols while it sees nothing"
    );
    assert!(
        pos(&s, s.guard).x > 3.0,
        "it walked the beat: {}",
        pos(&s, s.guard)
    );
    assert_eq!(s.world.get::<AiAgent>(s.guard).unwrap().status, AI_RUNNING);
    // The hero steps into view ahead of it (it walks +x towards (8, 2)).
    let g = pos(&s, s.guard);
    s.world
        .insert(
            s.hero,
            Transform {
                position: g + Vec3::new(3.0, 1.0, 0.0),
                ..Default::default()
            },
        )
        .unwrap();
    steps(&mut s, 2);
    let p = s.world.get::<Perception>(s.guard).unwrap();
    assert!(
        p.sees && p.target == s.hero.to_bits(),
        "it sees the hero: {p:?}"
    );
    assert_eq!(
        path(&s),
        "selector > sequence chase > chase",
        "seeing the hero, it chases"
    );
    assert_eq!(
        s.world.get::<NavAgent>(s.guard).unwrap().speed,
        3.0,
        "chase's speed"
    );
    // It closes in and bites, at most once per 0.5 s (30 ticks).
    let cmds = steps(&mut s, 120);
    let bites: Vec<_> = cmds.iter().filter(|c| c.0 == "bite").collect();
    assert!(
        bites.len() >= 2 && bites.len() <= 4,
        "{} bites in 2 s with a 0.5 s cooldown",
        bites.len()
    );
    assert_eq!(
        bites[0].1,
        vec![s.guard.to_bits().to_string()],
        "the command gets the agent's handle"
    );
    // Hurt below 20 hp (a component field), it flees whatever it sees.
    s.world.insert(s.guard, Health { hp: 12.0 }).unwrap();
    let before = pos(&s, s.guard).distance(pos(&s, s.hero));
    steps(&mut s, 90);
    assert_eq!(path(&s), "selector > sequence flee > flee");
    assert!(
        pos(&s, s.guard).distance(pos(&s, s.hero)) > before + 2.0,
        "it ran away"
    );
}

#[test]
fn a_resource_fact_redirects_the_patrol_and_the_selector_is_reactive() {
    let mut s = scene(GUARD, false);
    steps(&mut s, 30);
    s.world.resource_mut::<Game>().unwrap().alarm = true;
    steps(&mut s, 2);
    assert_eq!(
        path(&s),
        "selector > sequence alarm > move_to",
        "the alarm (Game.alarm) takes over from the patrol"
    );
    assert_eq!(
        s.world.get::<NavAgent>(s.guard).unwrap().target,
        Vec3::new(5.0, 9.0, 0.0)
    );
    s.world.resource_mut::<Game>().unwrap().alarm = false;
    steps(&mut s, 2);
    assert_eq!(
        path(&s),
        "selector > patrol beat",
        "back to the patrol when it clears"
    );
}

#[test]
fn walls_the_view_cone_and_range_hide_the_target_and_noises_are_heard_within_hearing() {
    // Behind a wall at x = 10: in range and in the cone, but hidden.
    let mut s = scene(GUARD, true);
    s.world
        .insert(
            s.guard,
            Transform {
                position: Vec3::new(8.0, 5.0, 0.0),
                ..Default::default()
            },
        )
        .unwrap();
    s.world
        .insert(
            s.guard,
            Perception {
                sight: 6.0,
                fov: 120.0,
                team: 2,
                forward: Vec3::X,
                ..Default::default()
            },
        )
        .unwrap();
    s.world
        .insert(
            s.hero,
            Transform {
                position: Vec3::new(12.0, 5.0, 0.0),
                ..Default::default()
            },
        )
        .unwrap();
    steps(&mut s, 2);
    assert!(
        !s.world.get::<Perception>(s.guard).unwrap().sees,
        "the wall hides the hero"
    );
    // No wall: seen; behind its back: not; out of range: not.
    let mut s = scene(GUARD, false);
    let look = |s: &mut Scene, hero: Vec3| -> bool {
        s.world
            .insert(
                s.guard,
                Transform {
                    position: Vec3::new(8.0, 5.0, 0.0),
                    ..Default::default()
                },
            )
            .unwrap();
        s.world
            .insert(
                s.guard,
                Perception {
                    sight: 6.0,
                    fov: 120.0,
                    team: 2,
                    forward: Vec3::X,
                    ..Default::default()
                },
            )
            .unwrap();
        s.world
            .insert(
                s.hero,
                Transform {
                    position: hero,
                    ..Default::default()
                },
            )
            .unwrap();
        s.world.step().unwrap();
        s.world.get::<Perception>(s.guard).unwrap().sees
    };
    assert!(
        look(&mut s, Vec3::new(12.0, 5.0, 0.0)),
        "in the open, it sees ahead"
    );
    assert!(
        !look(&mut s, Vec3::new(4.0, 5.0, 0.0)),
        "not behind it (fov 120)"
    );
    assert!(
        !look(&mut s, Vec3::new(14.5, 5.0, 0.0)),
        "not past its sight of 6 m"
    );
    // Hearing: a noise 5 m away is heard (hearing 6), one 9 m away is not.
    s.ai.noise(Noise {
        position: Vec3::new(8.0, 10.0, 0.0),
        loudness: 1.0,
    });
    s.world.step().unwrap();
    let p = s.world.get::<Perception>(s.guard).unwrap();
    assert!(p.heard && p.heard_at == Vec3::new(8.0, 10.0, 0.0), "{p:?}");
    s.ai.noise(Noise {
        position: Vec3::new(17.0, 5.0, 0.0),
        loudness: 1.0,
    });
    s.world.step().unwrap();
    assert!(
        !s.world.get::<Perception>(s.guard).unwrap().heard,
        "too far to hear"
    );
}

const MACHINE: &str = r#"
type behaviour
version 1
---
name "sentry"
key mood 0
route post {
  point 3 3 0
}
machine {
  initial idle
  state idle {
    enter {
      set mood 1
    }
    patrol post
    to alert {
      check heard
    }
  }
  state alert {
    enter {
      set mood 2
      command shout
    }
    exit {
      set mood 3
    }
    move_to heardAt
    to idle {
      after 1.5
    }
  }
}
"#;

#[test]
fn a_state_machine_moves_on_facts_and_timers_and_runs_enter_and_exit() {
    let mut s = scene(MACHINE, false);
    steps(&mut s, 3);
    let state = |s: &Scene| {
        let b =
            s.ai.behaviours
                .lock()
                .unwrap()
                .get(s.world.get::<AiAgent>(s.guard).unwrap().behaviour)
                .unwrap();
        s.ai.describe(s.guard).unwrap().state_name(&b)
    };
    assert_eq!(state(&s), "idle");
    assert_eq!(
        s.ai.get(s.guard, "mood"),
        Some(Value::Num(1.0)),
        "idle's enter ran"
    );
    s.ai.noise(Noise {
        position: Vec3::new(6.0, 6.0, 0.0),
        loudness: 2.0,
    });
    let cmds = steps(&mut s, 1);
    assert_eq!(state(&s), "alert");
    assert_eq!(
        s.ai.get(s.guard, "mood"),
        Some(Value::Num(2.0)),
        "alert's enter ran"
    );
    assert_eq!(
        cmds.iter().filter(|c| c.0 == "shout").count(),
        1,
        "enter's command ran once"
    );
    assert_eq!(
        s.world.get::<NavAgent>(s.guard).unwrap().target,
        Vec3::new(6.0, 6.0, 0.0),
        "it goes where it heard the noise"
    );
    steps(&mut s, 88);
    assert_eq!(state(&s), "alert", "1.5 s are not up after 89 ticks");
    steps(&mut s, 2);
    assert_eq!(state(&s), "idle", "back to idle after 1.5 s in alert");
    assert_eq!(
        s.ai.get(s.guard, "mood"),
        Some(Value::Num(1.0)),
        "alert's exit (3) ran, then idle's enter (1)"
    );
    assert!(s.world.get::<AiAgent>(s.guard).unwrap().changes >= 2);
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
