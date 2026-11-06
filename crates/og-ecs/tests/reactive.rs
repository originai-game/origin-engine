//! Reactivity: change filters across ticks and across a save and restore,
//! hook order, state transitions running enter and exit exactly once, the
//! spatial hash against brute force, and a world that uses none of it
//! saving and hashing as before.

use og_core::Rng;
use og_core::math::Vec3;
use og_ecs::{
    Entity, HookDesc, HookEvent, QueryDesc, Reflect, SpatialDesc, SpatialHash, StateDesc,
    SystemDesc, World, WorldDesc,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Pos {
    p: Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Hp {
    hp: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Tag {
    n: u32,
}

type Log = Arc<Mutex<Vec<String>>>;

fn log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

fn take(log: &Log) -> Vec<String> {
    std::mem::take(&mut *log.lock().unwrap())
}

fn base() -> World {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    w.register::<Pos>().unwrap();
    w.register::<Hp>().unwrap();
    w.register::<Tag>().unwrap();
    w
}

fn names(ctx: &og_ecs::SystemContext<'_>, q: usize) -> Vec<String> {
    let mut out = Vec::new();
    for a in ctx.query(q).unwrap() {
        let tags = a.read::<u32>(ctx_field(ctx)).unwrap();
        for r in a.rows() {
            out.push(format!("t{}", tags[r as usize]));
        }
    }
    out.sort();
    out
}

fn ctx_field(ctx: &og_ecs::SystemContext<'_>) -> og_ecs::FieldRef {
    let id = ctx.id::<Tag>().unwrap();
    og_ecs::FieldRef {
        component: id,
        field: 0,
        ty: og_core::reflect::FieldType::U32,
    }
}

/// A world whose `Damage` system hurts entity tag 1 on even ticks, sets an
/// unchanged value on tag 2 every tick (not a change), and from tick 3
/// spawns one tagged entity per tick and despawns tag 1 at tick 5;
/// `Watch` logs what changed, was added and was removed since it last ran.
fn changes(log: &Log) -> World {
    let mut w = base();
    w.add_system(
        SystemDesc::new("Damage")
            .write::<Hp>()
            .write::<Tag>()
            .query(QueryDesc::new().read::<Tag>().write::<Hp>()),
        |ctx| {
            let tick = ctx.time().tick;
            let tag = ctx_field(ctx);
            let hp = ctx.world_field::<Hp>("hp")?;
            let mut doomed = Vec::new();
            for a in ctx.query(0)? {
                let tags = a.read::<u32>(tag)?;
                let mut hps = a.write::<u32>(hp)?;
                for (i, t) in tags.iter().enumerate() {
                    if *t == 1 && tick % 2 == 0 {
                        hps[i] -= 1;
                    }
                    if *t == 2 {
                        hps[i] = std::hint::black_box(hps[i]); // same value: no change
                    }
                    if *t == 1 && tick == 5 {
                        doomed.push(a.entities()[i]);
                    }
                }
            }
            for e in doomed {
                ctx.despawn(e)?;
            }
            if tick >= 3 {
                ctx.spawn((
                    Tag {
                        n: 10 + tick as u32,
                    },
                    Hp { hp: 1 },
                ))?;
            }
            Ok(())
        },
    )
    .unwrap();
    let l = log.clone();
    w.add_system(
        SystemDesc::new("Watch")
            .read::<Tag>()
            .read::<Hp>()
            .query(QueryDesc {
                read: vec!["Tag".into()],
                changed: vec!["Hp".into()],
                ..Default::default()
            })
            .query(QueryDesc {
                read: vec!["Tag".into()],
                added: vec!["Hp".into()],
                ..Default::default()
            }),
        move |ctx| {
            let tick = ctx.time().tick;
            let hp = ctx.id::<Hp>()?;
            let removed = ctx.removed(hp)?.len();
            l.lock().unwrap().push(format!(
                "{tick} changed {:?} added {:?} removed {removed}",
                names(ctx, 0),
                names(ctx, 1)
            ));
            Ok(())
        },
    )
    .unwrap();
    for t in 1..=2 {
        w.spawn((Tag { n: t }, Hp { hp: 10 })).unwrap();
    }
    w
}

trait WorldField {
    fn world_field<T: Reflect>(&self, name: &str) -> og_core::Result<og_ecs::FieldRef>;
}

impl WorldField for og_ecs::SystemContext<'_> {
    fn world_field<T: Reflect>(&self, name: &str) -> og_core::Result<og_ecs::FieldRef> {
        let id = self.id::<T>()?;
        let index = T::schema()
            .fields
            .iter()
            .position(|f| f.name == name)
            .unwrap();
        Ok(og_ecs::FieldRef {
            component: id,
            field: index as u32,
            ty: T::schema().fields[index].ty,
        })
    }
}

#[test]
fn change_filters_survive_a_save_and_restore() {
    // Save at every tick of the run, between ticks with a write pending,
    // and check the restored world logs and hashes as the original.
    for at in 0..7 {
        let l = log();
        let mut w = changes(&l);
        for _ in 0..at {
            w.step().unwrap();
        }
        let first = w
            .query(&QueryDesc::new().read::<Hp>())
            .unwrap()
            .next()
            .unwrap()
            .entities()[0];
        w.insert(first, Hp { hp: 77 }).unwrap(); // pending until the next tick compares it
        let saved = w.snapshot();
        let hash = w.world_hash();
        let l2 = log();
        let mut r = changes(&l2);
        r.restore(&saved).unwrap();
        assert_eq!(r.world_hash(), hash, "restored at {at}");
        take(&l);
        take(&l2);
        for _ in 0..4 {
            w.step().unwrap();
            r.step().unwrap();
            assert_eq!(r.world_hash(), w.world_hash());
        }
        assert_eq!(take(&l2), take(&l), "restored at {at}");
    }
}

#[test]
fn hooks_run_in_order_and_nest() {
    let l = log();
    let mut w = base();
    let hook = |name: &str, comp: &str, on: HookEvent| HookDesc {
        name: name.into(),
        component: comp.into(),
        on,
        reads: vec!["Tag".into(), "Hp".into()],
        writes: vec!["Tag".into()],
    };
    let a = l.clone();
    w.add_hook(hook("A", "Hp", HookEvent::Add), move |ctx, e| {
        a.lock()
            .unwrap()
            .push(format!("A {}", ctx.get::<Hp>(e)?.hp));
        // An added Hp tags its entity: Tag's add hook runs after this one returns.
        if !ctx.components_of(e)?.contains(&ctx.id::<Tag>()?) {
            ctx.insert(e, Tag { n: 7 })?;
        }
        Ok(())
    })
    .unwrap();
    let b = l.clone();
    w.add_hook(hook("B", "Hp", HookEvent::Add), move |_, _| {
        b.lock().unwrap().push("B".into());
        Ok(())
    })
    .unwrap();
    let t = l.clone();
    w.add_hook(hook("T", "Tag", HookEvent::Add), move |ctx, e| {
        t.lock()
            .unwrap()
            .push(format!("T {}", ctx.get::<Tag>(e)?.n));
        Ok(())
    })
    .unwrap();
    let r = l.clone();
    w.add_hook(hook("R", "Hp", HookEvent::Remove), move |ctx, e| {
        // Before it goes: the hook reads it.
        r.lock()
            .unwrap()
            .push(format!("R {}", ctx.get::<Hp>(e)?.hp));
        Ok(())
    })
    .unwrap();
    let e = w.spawn((Hp { hp: 5 },)).unwrap();
    assert_eq!(take(&l), ["A 5", "T 7", "B"]);
    w.insert(e, Hp { hp: 6 }).unwrap(); // an overwrite adds nothing
    assert_eq!(take(&l), Vec::<String>::new());
    w.remove::<Hp>(e).unwrap();
    assert_eq!(take(&l), ["R 6"]);
    w.insert(e, Hp { hp: 8 }).unwrap();
    assert_eq!(take(&l), ["A 8", "B"]);
    // From a system's commands, as they apply, in command order.
    w.add_system(SystemDesc::new("Spawner").write::<Hp>(), |ctx| {
        if ctx.time().tick == 0 {
            ctx.spawn((Hp { hp: 1 },))?;
            ctx.spawn((Hp { hp: 2 },))?;
        }
        Ok(())
    })
    .unwrap();
    w.step().unwrap();
    assert_eq!(take(&l), ["A 1", "T 7", "B", "A 2", "T 7", "B"]);
    w.despawn(e).unwrap();
    assert_eq!(take(&l), ["R 8"]);
    // A restore runs none.
    let saved = w.snapshot();
    w.restore(&saved).unwrap();
    assert_eq!(take(&l), Vec::<String>::new());
}

/// A game with a menu and a play state: `Start` (in menu) asks for play at
/// tick 2, `Quit` (in play) asks for the menu at tick 5.
fn states(log: &Log) -> World {
    let mut w = base();
    w.add_state(StateDesc {
        name: "Mode".into(),
        values: vec!["menu".into(), "play".into()],
        initial: String::new(),
    })
    .unwrap();
    let push = |l: &Log, s: &str| {
        let (l, s) = (l.clone(), s.to_string());
        move |ctx: &mut og_ecs::SystemContext<'_>| {
            l.lock().unwrap().push(format!("{} {s}", ctx.time().tick));
            Ok(())
        }
    };
    w.add_system(
        SystemDesc::new("EnterMenu").on_enter("Mode", "menu"),
        push(log, "enter menu"),
    )
    .unwrap();
    w.add_system(
        SystemDesc::new("ExitMenu").on_exit("Mode", "menu"),
        push(log, "exit menu"),
    )
    .unwrap();
    w.add_system(
        SystemDesc::new("EnterPlay").on_enter("Mode", "play"),
        push(log, "enter play"),
    )
    .unwrap();
    w.add_system(
        SystemDesc::new("ExitPlay").on_exit("Mode", "play"),
        push(log, "exit play"),
    )
    .unwrap();
    let l = log.clone();
    w.add_system(
        SystemDesc::new("Start").in_state("Mode", "menu"),
        move |ctx| {
            l.lock().unwrap().push(format!("{} menu", ctx.time().tick));
            if ctx.time().tick == 2 {
                ctx.set_state("Mode", "play")?;
            }
            Ok(())
        },
    )
    .unwrap();
    let l = log.clone();
    w.add_system(
        SystemDesc::new("Quit").in_state("Mode", "play"),
        move |ctx| {
            l.lock().unwrap().push(format!("{} play", ctx.time().tick));
            if ctx.time().tick == 5 {
                ctx.set_state("Mode", "menu")?;
                ctx.set_state("Mode", "menu")?; // asked twice: one transition
            }
            Ok(())
        },
    )
    .unwrap();
    w
}

#[test]
fn a_restored_state_machine_goes_on_alike() {
    for at in 0..8 {
        let l = log();
        let mut w = states(&l);
        for _ in 0..at {
            w.step().unwrap();
        }
        let saved = w.snapshot();
        let l2 = log();
        let mut r = states(&l2);
        r.restore(&saved).unwrap();
        assert_eq!(r.world_hash(), w.world_hash());
        take(&l);
        for _ in 0..6 {
            w.step().unwrap();
            r.step().unwrap();
        }
        assert_eq!(take(&l2), take(&l), "saved after {at} ticks");
        assert_eq!(r.world_hash(), w.world_hash());
    }
}

fn brute(points: &[(Entity, [f32; 3])], c: [f32; 3]) -> Vec<(f64, Entity)> {
    let mut v: Vec<(f64, Entity)> = points
        .iter()
        .map(|(e, p)| {
            // Squared by multiplying, as the index does: `powi` is the host's
            // (MSVC's debug build rounds 480 squares in a million differently).
            let d: f64 = (0..3)
                .map(|k| {
                    let d = f64::from(p[k]) - f64::from(c[k]);
                    d * d
                })
                .sum();
            (d.sqrt(), *e)
        })
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    v
}

#[test]
fn spatial_queries_match_brute_force_on_random_worlds() {
    for seed in 0..40u32 {
        let mut rng = Rng::new(seed);
        let mut w = base();
        let n = 1 + rng.integer(0, 299) as usize;
        let spread = [1.0, 10.0, 200.0][seed as usize % 3];
        let flat = seed % 4 == 0;
        for i in 0..n {
            // Integer grid points in some worlds, so equal distances happen.
            let mut c = || {
                if seed % 2 == 0 {
                    (rng.integer(0, 8) as f32) - 4.0
                } else {
                    rng.range_f32(-spread, spread)
                }
            };
            let p = Vec3::new(c(), c(), if flat { 0.0 } else { c() });
            if i % 7 == 3 {
                w.spawn((Pos { p }, Tag { n: i as u32 })).unwrap();
            } else {
                w.spawn((Pos { p },)).unwrap();
            }
        }
        let field = w.field::<Pos>("p").unwrap();
        let all: Vec<(Entity, [f32; 3])> = w
            .entities()
            .into_iter()
            .map(|e| {
                let p = w.get::<Pos>(e).unwrap().p;
                (e, [p.x, p.y, p.z])
            })
            .collect();
        for cell in [0.0f32, 0.5, 3.0] {
            let index: SpatialHash = w
                .spatial(&SpatialDesc {
                    field,
                    cell,
                    with: Vec::new(),
                    without: Vec::new(),
                })
                .unwrap();
            assert_eq!(index.len(), n);
            for _ in 0..20 {
                let c = [
                    rng.range_f32(-spread, spread),
                    rng.range_f32(-spread, spread),
                    rng.range_f32(-spread, spread),
                ];
                let expect = brute(&all, c);
                let radius = rng.range_f32(0.0, spread);
                let got: Vec<(f64, Entity)> = index
                    .within(c, radius)
                    .into_iter()
                    .map(|n| (n.distance, n.entity))
                    .collect();
                let want: Vec<(f64, Entity)> = expect
                    .iter()
                    .copied()
                    .filter(|(d, _)| d * d <= f64::from(radius) * f64::from(radius) + 0.0)
                    .collect();
                assert_eq!(got.len(), want.len(), "seed {seed} cell {cell} r {radius}");
                assert_eq!(got, want, "seed {seed} cell {cell}");
                let k = rng.integer(1, 11) as usize;
                let got: Vec<Entity> = index.nearest(c, k).into_iter().map(|n| n.entity).collect();
                let want: Vec<Entity> = expect.iter().take(k).map(|(_, e)| *e).collect();
                assert_eq!(got, want, "seed {seed} cell {cell} k {k}");
            }
        }
        // `with` narrows to tagged entities.
        let tag = w.id::<Tag>().unwrap();
        let tagged = w
            .spatial(&SpatialDesc {
                field,
                cell: 0.0,
                with: vec![tag],
                without: Vec::new(),
            })
            .unwrap();
        assert_eq!(tagged.len(), (0..n).filter(|i| i % 7 == 3).count());
    }
}
