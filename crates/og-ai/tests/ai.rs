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

use og_ai::{AiAgent, AiDesc, Behaviour, Kind, Perceivable, Perception, SightRay, Value, install};
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
