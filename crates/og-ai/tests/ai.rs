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
//! hearing beyond its radius, or its own sounds (and its children's); a
//! state machine that ignores its timers or its enter/exit actions; a
//! restart that leaves a machine in its state or a tree mid-way; a check
//! against a tuning field that reads it once, or not at all; two runs or a
//! restored snapshot that decide differently.

use og_ai::{
    AI_RUNNING, AiAgent, AiDesc, Behaviour, Kind, Noise, Parent, Perceivable, Perception, SightRay,
    Value, install,
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
    let e = errors(&format!("{head}tree {{\n  check hp below 1 2\n}}\n"));
    assert!(
        e.contains("line 5") && e.contains("a key or field holding it"),
        "{e}"
    );
    // A threshold in a tuning resource: compared with, read each time.
    let b = Behaviour::parse(&format!(
        "{head}tree {{\n  check Health.hp below Tuning.flee\n}}\n"
    ))
    .expect("a check against a field parses");
    let Kind::Check(c) = &b.nodes[0].kind else {
        panic!("a check")
    };
    assert_eq!(c.against.as_deref(), Some("Tuning.flee"));
    assert_eq!(b.nodes[0].name, "check Health.hp below Tuning.flee");
    assert_eq!(
        b.field_operands(),
        vec!["Health.hp".to_string(), "Tuning.flee".to_string()]
    );
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
        ..Default::default()
    });
    s.world.step().unwrap();
    let p = s.world.get::<Perception>(s.guard).unwrap();
    assert!(p.heard && p.heard_at == Vec3::new(8.0, 10.0, 0.0), "{p:?}");
    s.ai.noise(Noise {
        position: Vec3::new(17.0, 5.0, 0.0),
        loudness: 1.0,
        ..Default::default()
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
        ..Default::default()
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

#[test]
fn an_agent_does_not_hear_its_own_sounds_nor_its_children_s() {
    use og_audio::AudioSource;
    use og_audio::schema::SoundStarted;
    // Kestrel Stair FINDINGS #8: an imp heard its own chitter.
    let mut s = scene(GUARD, false);
    let spatial = AudioSource {
        spatial: true,
        volume: 1.0,
        ..Default::default()
    };
    let at = |p: Vec3| Transform {
        position: p,
        ..Default::default()
    };
    s.world.insert(s.guard, spatial).unwrap();
    let guard = s.guard.to_bits();
    let bell = s
        .world
        .spawn((
            at(Vec3::new(1.0, 0.0, 0.0)),
            Parent { entity: guard },
            spatial,
        ))
        .unwrap();
    // A cart 3 m off with a bell 1 m further on it: heard where the bell is.
    let cart = s.world.spawn((at(Vec3::new(5.0, 2.0, 0.0)),)).unwrap();
    let cart_bell = s
        .world
        .spawn((
            at(Vec3::new(1.0, 0.0, 0.0)),
            Parent {
                entity: cart.to_bits(),
            },
            spatial,
        ))
        .unwrap();
    let hears = |s: &mut Scene, from: Entity| {
        s.world
            .send(SoundStarted {
                entity: from.to_bits(),
                ..Default::default()
            })
            .unwrap();
        s.world.step().unwrap();
        s.world.get::<Perception>(s.guard).unwrap()
    };
    let me = s.guard;
    assert!(!hears(&mut s, me).heard, "its own sound");
    assert!(!hears(&mut s, bell).heard, "its child's sound");
    let p = hears(&mut s, cart_bell);
    assert!(p.heard, "another's sound");
    assert_eq!(p.heard_at, Vec3::new(6.0, 2.0, 0.0), "where the bell is");
    // A game noise made by the agent itself is not heard either.
    s.ai.noise(Noise {
        position: Vec3::new(3.0, 2.0, 0.0),
        loudness: 1.0,
        source: guard,
    });
    s.world.step().unwrap();
    assert!(!s.world.get::<Perception>(s.guard).unwrap().heard);
}

#[test]
fn a_restart_puts_a_machine_in_its_initial_state_and_a_tree_at_its_root() {
    // Kestrel Stair FINDINGS #5: a new run needed a second handle.
    let mut s = scene(MACHINE, false);
    steps(&mut s, 3);
    s.ai.noise(Noise {
        position: Vec3::new(6.0, 6.0, 0.0),
        loudness: 2.0,
        ..Default::default()
    });
    steps(&mut s, 2);
    assert_eq!(s.world.get::<AiAgent>(s.guard).unwrap().state, 1, "alert");
    assert!(s.ai.restart(s.guard));
    steps(&mut s, 1);
    assert_eq!(
        s.world.get::<AiAgent>(s.guard).unwrap().state,
        0,
        "idle again"
    );
    assert_eq!(
        s.ai.get(s.guard, "mood"),
        Some(Value::Num(1.0)),
        "idle's enter ran again"
    );
    assert_eq!(
        s.world.get::<Perception>(s.guard).unwrap().since_heard,
        og_ai::NEVER,
        "what it heard is forgotten"
    );
    // A tree: `steps` resumes at its running child, a restart at the first.
    let tree = r#"
type behaviour
version 1
---
name "drill"
tree {
  steps {
    command one
    wait 10
  }
}
"#;
    let mut s = scene(tree, false);
    let ones = |c: &[(String, Vec<String>)]| c.iter().filter(|c| c.0 == "one").count();
    assert_eq!(ones(&steps(&mut s, 30)), 1, "once, then it waits");
    assert!(s.ai.restart(s.guard));
    assert_eq!(ones(&steps(&mut s, 30)), 1, "again from the root");
    // An agent that has not thought: nothing to restart.
    let mut s = scene(tree, false);
    assert!(!s.ai.restart(s.guard));
    assert_eq!(ones(&steps(&mut s, 2)), 1);
}

#[test]
fn a_check_against_a_tuning_field_reads_it_every_time() {
    let tree = r#"
type behaviour
version 1
---
name "skittish"
tree {
  selector {
    sequence {
      check Health.hp below Tuning.flee
      command low
    }
    sequence {
      check Health.hp atLeast Tuning.missing
      command never
    }
    wait 100
  }
}
"#;
    let mut s = scene(tree, false);
    let lows = |c: &[(String, Vec<String>)]| c.iter().filter(|c| c.0 == "low").count();
    assert_eq!(lows(&steps(&mut s, 5)), 0, "50 hp is not below 20");
    s.world.resource_mut::<Tuning>().unwrap().flee = 60.0;
    let cmds = steps(&mut s, 5);
    assert_eq!(
        lows(&cmds),
        5,
        "50 hp is below 60, read the tick it changed"
    );
    s.world.insert(s.guard, Health { hp: 70.0 }).unwrap();
    let cmds = steps(&mut s, 5);
    assert_eq!(lows(&cmds), 0, "70 hp is not below 60");
    assert!(cmds.iter().all(|c| c.0 != "never"), "a missing field fails");
    let w = s.ai.take_warnings();
    assert!(
        w.iter().any(|w| w.contains("'Tuning.missing'")),
        "the missing field is named: {w:?}"
    );
}

/// The template's yard as og.nav and og.ai run it (templates/game: the
/// guard's machine on a navmesh round a wall), with what G2-4 must leave
/// alone beside it: a crowd that separates without og.Crowd, a box obstacle
/// set at run time, and a grid agent over weighted cells. Its world hash
/// and its 300-tick trajectory were recorded on main 67a07d57, before
/// G2-4: a world that uses none of G2-4 hashes and moves bit for bit as
/// before, and a restore at tick 150 replays it.
#[test]
fn a_yard_without_g2_4_features_hashes_and_moves_as_before_it() {
    use og_nav::{Heightfield, NavMesh, NavParams, Obstacle, Steering};
    const GUARD_MACHINE: &str = include_str!("../../../templates/game/assets/ai/guard.ogt");
    struct Yard {
        world: World,
        ai: og_ai::AiSystem,
        navs: og_nav::SharedNavs,
        mesh: u64,
        agents: Vec<Entity>,
        trickster: Entity,
    }
    let yard = || {
        let mut world = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 1,
        })
        .unwrap();
        let ai = install(
            &mut world,
            &[],
            AiDesc {
                plane: Plane::Xz,
                ray: None,
                sink: None,
                behaviours: None,
            },
        )
        .unwrap();
        let params = NavParams {
            cell_size: 0.25,
            tile_size: 32,
            ..Default::default()
        };
        // 24 m square from (-12, 0, -12); the template's wall, 0.6 × 6 m at (-3, -2).
        let field =
            Heightfield::flat(params, Vec3::new(-12.0, 0.0, -12.0), 96, 96).without(|x, z| {
                let (wx, wz) = (x as f32 * 0.25 - 12.0, z as f32 * 0.25 - 12.0);
                (-3.3..-2.7).contains(&wx) && (-5.0..1.0).contains(&wz)
            });
        let mut navs = Navs::default();
        let mesh = navs.insert(Nav::Mesh(Box::new(NavMesh::build(field))));
        let mut grid = Grid::new(16, 16, 1.0, Vec2::new(-8.0, -8.0));
        for y in 0..14 {
            grid.set(8, y, 40);
        }
        let grid = navs.insert(Nav::Grid(grid));
        let navs = Arc::new(Mutex::new(navs));
        og_nav::install(
            &mut world,
            navs.clone(),
            NavDesc {
                plane: Plane::Xz,
                raycast: None,
                plans_per_tick: og_nav::PLANS_PER_TICK,
            },
        )
        .unwrap();
        let b = ai
            .behaviours
            .lock()
            .unwrap()
            .insert(Behaviour::parse(GUARD_MACHINE).unwrap());
        let mut agents = vec![
            world
                .spawn((
                    Transform {
                        position: Vec3::new(-6.0, 0.0, -3.0),
                        ..Default::default()
                    },
                    NavAgent {
                        nav: mesh,
                        speed: 1.5,
                        acceleration: 12.0,
                        face_movement: true,
                        turn_speed: 8.0,
                        ..Default::default()
                    },
                    AiAgent {
                        behaviour: b,
                        ..Default::default()
                    },
                    Perception {
                        sight: 8.0,
                        fov: 120.0,
                        team: 2,
                        ..Default::default()
                    },
                ))
                .unwrap(),
        ];
        // A crowd crossing the wall's line both ways, separating.
        for i in 0..8 {
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let z = -4.0 + i as f32;
            agents.push(
                world
                    .spawn((
                        Transform {
                            position: Vec3::new(-8.0 * side, 0.0, z),
                            ..Default::default()
                        },
                        NavAgent {
                            nav: mesh,
                            target: Vec3::new(6.0 * side, 0.0, -z),
                            seeking: true,
                            speed: 2.5 + 0.1 * i as f32,
                            ..Default::default()
                        },
                        Steering {
                            separation: 1.5,
                            separation_radius: 1.2,
                            wander: 0.2,
                            ..Default::default()
                        },
                    ))
                    .unwrap(),
            );
        }
        agents.push(
            world
                .spawn((
                    Transform {
                        position: Vec3::new(-6.5, 0.0, -6.5),
                        ..Default::default()
                    },
                    NavAgent {
                        nav: grid,
                        target: Vec3::new(6.5, 0.0, -6.5),
                        seeking: true,
                        speed: 4.0,
                        ..Default::default()
                    },
                ))
                .unwrap(),
        );
        let trickster = world
            .spawn((
                Transform {
                    position: Vec3::new(4.0, 0.0, 6.0),
                    ..Default::default()
                },
                Perceivable {
                    team: 1,
                    eye_height: 1.0,
                },
            ))
            .unwrap();
        Yard {
            world,
            ai,
            navs,
            mesh,
            agents,
            trickster,
        }
    };
    // What the game does at tick t: the trickster walks past the guard's post and away; a crate lands in the crowd's way.
    let act = |y: &mut Yard, t: u32| {
        let s = t as f32 / 60.0;
        let at = if s < 2.5 {
            Vec3::new(4.0 - 3.6 * s, 0.0, 6.0 - 2.4 * s)
        } else {
            Vec3::new(-5.0 + 4.0 * (s - 2.5), 0.0, 0.0 + 3.0 * (s - 2.5))
        };
        y.world
            .insert(
                y.trickster,
                Transform {
                    position: at,
                    ..Default::default()
                },
            )
            .unwrap();
        if t == 100
            && let Some(Nav::Mesh(m)) = y.navs.lock().unwrap().get_mut(y.mesh)
        {
            m.set_obstacle(
                7,
                Some(Obstacle {
                    min: Vec3::new(0.5, 0.0, -1.0),
                    max: Vec3::new(1.5, 1.0, 1.0),
                }),
            );
        }
    };
    let fnv = |h: &mut u64, bits: u32| {
        for b in bits.to_le_bytes() {
            *h ^= b as u64;
            *h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    let mut y = yard();
    let mut trail = 0xcbf2_9ce4_8422_2325u64;
    let mut commands = Vec::new();
    let mut snap = None;
    for t in 0..300u32 {
        if t == 150 {
            snap = Some((y.world.snapshot(), trail));
        }
        act(&mut y, t);
        y.world.step().unwrap();
        commands.extend(y.ai.take_commands().into_iter().map(|c| (t, c.0)));
        for &e in &y.agents {
            let p = y.world.get::<Transform>(e).unwrap().position;
            for c in [p.x, p.y, p.z] {
                fnv(&mut trail, c.to_bits());
            }
        }
    }
    let hash = format!("{:016x}", y.world.world_hash());
    let guard = y.world.get::<Transform>(y.agents[0]).unwrap().position;
    assert!(
        commands.iter().any(|(_, c)| c == "alarm"),
        "the guard saw the trickster and raised the alarm: {commands:?}"
    );
    assert!(
        guard.distance(Vec3::new(-6.0, 0.0, -3.0)) > 1.0,
        "the guard moved"
    );
    assert_eq!(
        (hash.as_str(), format!("{trail:016x}").as_str()),
        (YARD_HASH, YARD_TRAIL),
        "a yard using no G2-4 feature hashes or moves otherwise than on main 67a07d57"
    );
    // Restored at tick 150 into a fresh yard, it goes on as the original.
    let (bytes, mut trail2) = snap.unwrap();
    let mut z = yard();
    z.world.restore(&bytes).unwrap();
    if let Some(Nav::Mesh(m)) = z.navs.lock().unwrap().get_mut(z.mesh) {
        assert_eq!(
            m.obstacles().count(),
            1,
            "the crate came back with the save"
        );
    }
    for t in 150..300u32 {
        act(&mut z, t);
        z.world.step().unwrap();
        for &e in &z.agents {
            let p = z.world.get::<Transform>(e).unwrap().position;
            for c in [p.x, p.y, p.z] {
                fnv(&mut trail2, c.to_bits());
            }
        }
    }
    assert_eq!(format!("{:016x}", z.world.world_hash()), hash);
    assert_eq!(trail2, trail, "the restored yard moved otherwise");
}

/// [`a_yard_without_g2_4_features_hashes_and_moves_as_before_it`]'s world
/// hash and trajectory digest on main 67a07d57 (before G2-4).
const YARD_HASH: &str = "10c3d2daa0248e7e";
const YARD_TRAIL: &str = "944e29cc07af861e";

// ── G2-4: game actions, subtrees, utility ───────────────────────────────────

const WORKER: &str = r#"
type behaviour
version 1
---
name "worker"
tree {
  selector {
    sequence alarm {
      check Game.alarm
      action shout loud 2
    }
    sequence work {
      action reload 2
      command reloaded
    }
  }
}
"#;

/// The game-action events og.ai sent, as `started reload 2` and
/// `stopped reload 2` (the result code), in the order the watcher read them.
fn watch(s: &mut Scene) -> Arc<Mutex<Vec<String>>> {
    use og_ai::{AiActionStarted, AiActionStopped};
    og_ai::enable_actions(&mut s.world).unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    s.world
        .add_system(
            og_ecs::SystemDesc::new("watch")
                .read::<AiActionStarted>()
                .read::<AiActionStopped>(),
            move |ctx| {
                let mut l = l.lock().unwrap();
                for e in ctx.events::<AiActionStopped>()? {
                    l.push(format!("stopped {} {}", e.name.as_str(), e.result));
                }
                for e in ctx.events::<AiActionStarted>()? {
                    l.push(format!("started {} {}", e.name.as_str(), e.args.as_str()));
                }
                Ok(())
            },
        )
        .unwrap();
    log
}

fn drain(log: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    std::mem::take(&mut *log.lock().unwrap())
}

#[test]
fn a_game_action_runs_until_finished_is_interrupted_by_a_higher_branch_and_its_result_survives_a_save()
 {
    let mut s = scene(WORKER, false);
    let log = watch(&mut s);
    steps(&mut s, 3);
    assert_eq!(drain(&log), ["started reload 2"]);
    assert!(path(&s).ends_with("action reload"), "{}", path(&s));
    // It stays running however long the game takes.
    assert!(steps(&mut s, 60).is_empty());
    assert!(drain(&log).is_empty());
    assert!(!s.ai.finish(s.guard, true, Some("shout")), "not running");
    assert!(s.ai.finish(s.guard, true, Some("reload")));
    let cmds = steps(&mut s, 3);
    assert_eq!(cmds.iter().filter(|c| c.0 == "reloaded").count(), 1);
    // Success, then the tree starts it again at its next think.
    assert_eq!(drain(&log), ["stopped reload 2", "started reload 2"]);
    // The alarm branch wins: reload is interrupted, shout starts.
    s.world.resource_mut::<Game>().unwrap().alarm = true;
    steps(&mut s, 2);
    assert_eq!(
        drain(&log),
        [
            format!("stopped reload {}", og_ai::AI_INTERRUPTED),
            "started shout loud 2".to_string()
        ]
    );
    // Finished but not yet answered: a save holds the pending result, and a
    // world restored from it answers the same at its next think.
    assert!(s.ai.finish(s.guard, false, None));
    let snap = s.world.snapshot();
    // The shout failed: the alarm branch fails, so the selector starts
    // reload in the same think; the next think the alarm wins again.
    let after_failure = [
        "stopped shout 3".to_string(),
        "started reload 2".to_string(),
        format!("stopped reload {}", og_ai::AI_INTERRUPTED),
        "started shout loud 2".to_string(),
    ];
    steps(&mut s, 3);
    assert_eq!(drain(&log), after_failure);
    let mut fresh = scene(WORKER, false);
    let flog = watch(&mut fresh);
    fresh.world.restore(&snap).unwrap();
    steps(&mut fresh, 3);
    assert_eq!(drain(&flog), after_failure);
    assert_eq!(fresh.world.world_hash(), s.world.world_hash());
    // A restart interrupts what was running.
    assert!(s.ai.restart(s.guard));
    steps(&mut s, 2);
    assert_eq!(
        drain(&log)[0],
        format!("stopped shout {}", og_ai::AI_INTERRUPTED)
    );
}

#[test]
fn an_action_is_refused_when_its_event_cannot_carry_it() {
    let head = "type behaviour\nversion 1\n---\ntree {\n  action ";
    let long = "x".repeat(64);
    assert!(errors(&format!("{head}{long}\n}}\n")).contains("under 64 bytes"));
    // 2 × 128 bytes and a space: 257, over og.AiActionStarted's 255.
    let arg = "a".repeat(128);
    let e = errors(&format!("{head}say {arg} {arg}\n}}\n"));
    assert!(
        e.contains("line 5") && e.contains("at most 255 bytes"),
        "{e}"
    );
    let ok = "a".repeat(127);
    Behaviour::parse(&format!(
        "type behaviour\nversion 1\n---\ntree {{\n  action say {ok} {ok}\n}}\n"
    ))
    .unwrap();
}

#[test]
fn without_the_action_events_registered_a_game_action_still_runs() {
    let mut s = scene(WORKER, false);
    steps(&mut s, 3);
    assert!(s.ai.finish(s.guard, true, None));
    assert_eq!(
        steps(&mut s, 2)
            .iter()
            .filter(|c| c.0 == "reloaded")
            .count(),
        1
    );
}

fn files(map: &[(&str, &str)]) -> impl FnMut(&str) -> Result<String, String> + use<> {
    let map: Vec<(String, String)> = map
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    move |p: &str| {
        map.iter()
            .find(|(k, _)| k == p)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| format!("no behaviour asset '{p}'"))
    }
}

const BEAT: &str = r#"
type behaviour
version 1
---
name "beat"
key laps 0
route beat {
  point 3 2 0
  point 8 2 0
}
tree {
  sequence {
    use "ai/look.ogt"
    patrol beat
  }
}
"#;

const LOOK: &str = r#"
type behaviour
version 1
---
name "look"
key looked 0
tree {
  set looked 1
}
"#;

#[test]
fn subtrees_inline_their_nodes_keys_and_routes_and_a_cycle_is_refused_naming_the_chain() {
    let main = r#"
type behaviour
version 1
---
name "sentry"
route home {
  point 2 2 0
}
tree {
  selector {
    sequence {
      check Game.alarm
      patrol home
    }
    use "ai/beat.ogt"
  }
}
"#;
    let mut r = files(&[("ai/beat.ogt", BEAT), ("ai/look.ogt", LOOK)]);
    let b = Behaviour::parse_with(main, "ai/sentry.ogt", &mut r).unwrap();
    assert_eq!(b.subtrees, ["ai/beat.ogt", "ai/look.ogt"]);
    let keys: Vec<&str> = b.keys.iter().map(|k| k.0.as_str()).collect();
    assert_eq!(keys, ["laps", "looked"]);
    let routes: Vec<&str> = b.routes.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(routes, ["home", "beat"]);
    // The subtree's patrol names its own route, now at index 1.
    assert!(b.nodes.iter().any(|n| matches!(
        &n.kind,
        Kind::Action(og_ai::Action::Patrol { route: 1, .. })
    )));
    // Run it: the beat's patrol, under its `use` node, and look's key set.
    let mut s = scene(GUARD, false);
    let h = s.ai.behaviours.lock().unwrap().insert(b);
    let mut a = s.world.get::<AiAgent>(s.guard).unwrap();
    a.behaviour = h;
    s.world.insert(s.guard, a).unwrap();
    steps(&mut s, 120);
    assert_eq!(
        path(&s),
        "selector > use ai/beat.ogt > sequence > patrol beat"
    );
    assert_eq!(s.ai.get(s.guard, "looked"), Some(Value::Num(1.0)));
    assert!(pos(&s, s.guard).x > 3.5, "it walks the beat");

    // A cycle through two files, refused naming every one.
    let a = "type behaviour\nversion 1\n---\ntree {\n  use \"ai/b.ogt\"\n}\n";
    let bb = "type behaviour\nversion 1\n---\ntree {\n  use \"ai/a.ogt\"\n}\n";
    let mut r = files(&[("ai/a.ogt", a), ("ai/b.ogt", bb)]);
    let e = Behaviour::parse_with(a, "ai/a.ogt", &mut r).unwrap_err();
    assert!(
        e.contains("use makes a cycle: ai/a.ogt → ai/b.ogt → ai/a.ogt"),
        "{e}"
    );
    // A file using itself.
    let mut r = files(&[(
        "ai/a.ogt",
        "type behaviour\nversion 1\n---\ntree {\n  use \"ai/a.ogt\"\n}\n",
    )]);
    let e = Behaviour::parse_with(
        "type behaviour\nversion 1\n---\ntree {\n  use \"ai/a.ogt\"\n}\n",
        "ai/a.ogt",
        &mut r,
    )
    .unwrap_err();
    assert!(e.contains("ai/a.ogt → ai/a.ogt"), "{e}");
    // A machine, a missing file, a clashing route, and text without a resolver.
    let machine = "type behaviour\nversion 1\n---\nmachine {\n  state idle {\n    wait 1\n  }\n}\n";
    let clash = "type behaviour\nversion 1\n---\nroute beat {\n  point 0 0 0\n}\ntree {\n  sequence {\n    patrol beat\n    use \"ai/beat.ogt\"\n  }\n}\n";
    let mut r = files(&[
        ("ai/m.ogt", machine),
        ("ai/beat.ogt", BEAT),
        ("ai/look.ogt", LOOK),
    ]);
    for (text, want) in [
        (
            "type behaviour\nversion 1\n---\ntree {\n  use \"ai/m.ogt\"\n}\n",
            "ai/m.ogt is a machine",
        ),
        (
            "type behaviour\nversion 1\n---\ntree {\n  use \"ai/none.ogt\"\n}\n",
            "no behaviour asset 'ai/none.ogt'",
        ),
        (clash, "route 'beat' differs"),
    ] {
        let e = Behaviour::parse_with(text, "ai/x.ogt", &mut r).unwrap_err();
        assert!(e.contains(want), "{e}");
    }
    let e = Behaviour::parse(main).unwrap_err();
    assert!(e.contains("use reads another behaviour asset"), "{e}");
}

const CHORES: &str = r#"
type behaviour
version 1
---
name "chores"
key hunger 0
tree {
  utility {
    inertia 0.2
    option eat {
      score hunger linear 0 10
      command eat {
        cooldown 5
      }
      wait 1
    }
    option idle {
      score 0.3
      wait 1
    }
  }
}
"#;

#[test]
fn a_utility_runs_its_best_option_keeps_the_running_one_by_its_inertia_and_ties_go_to_the_first() {
    let mut s = scene(CHORES, false);
    steps(&mut s, 2);
    assert_eq!(path(&s), "utility > option idle > wait");
    // Eat scores 0.4, idle 0.3 + 0.2 while it runs: idle keeps going.
    assert!(s.ai.set(s.guard, "hunger", Value::Num(4.0)));
    let eats = |c: &[(String, Vec<String>)]| c.iter().filter(|c| c.0 == "eat").count();
    assert_eq!(eats(&steps(&mut s, 20)), 0);
    assert_eq!(path(&s), "utility > option idle > wait");
    // Its wait over, idle has no inertia: eat wins.
    assert_eq!(eats(&steps(&mut s, 60)), 1);
    assert!(path(&s).starts_with("utility > option eat"), "{}", path(&s));
    // A tie (0.3 against 0.3) goes to the first option, eat.
    let mut s = scene(CHORES, false);
    assert!(
        !s.ai.set(s.guard, "hunger", Value::Num(3.0)),
        "not thought yet"
    );
    steps(&mut s, 1);
    assert!(s.ai.set(s.guard, "hunger", Value::Num(3.0)));
    // Idle runs out its second (its inertia keeps it), then the tie is decided.
    steps(&mut s, 120);
    assert!(path(&s).starts_with("utility > option eat"), "{}", path(&s));
    // Curves place the input between its ends.
    use og_ai::Curve;
    assert_eq!(Curve::Inverse.score(2.5, 0.0, 10.0), 0.75);
    assert_eq!(Curve::Quadratic.score(5.0, 0.0, 10.0), 0.25);
    assert_eq!(Curve::Smooth.score(5.0, 0.0, 10.0), 0.5);
    assert_eq!(Curve::Step.score(9.9, 0.0, 10.0), 0.0);
    assert_eq!(Curve::Linear.score(20.0, 0.0, 10.0), 1.0);
    for (bad, want) in [
        (
            "utility {\n    option a {\n      wait 1\n    }\n  }",
            "needs a score line",
        ),
        (
            "utility {\n    option a {\n      score x wobbly 0 1\n      wait 1\n    }\n  }",
            "'wobbly' is not a curve",
        ),
        ("utility {\n    inertia 0.1\n  }", "at least one option"),
        ("action", "action takes a name"),
    ] {
        let e = errors(&format!(
            "type behaviour\nversion 1\n---\ntree {{\n  {bad}\n}}\n"
        ));
        assert!(e.contains(want), "{e}");
    }
}
