//! Snapshot → restore gives the identical world: the same hash, the same
//! bytes, and the same future (entity ids handed out, iteration order, rng
//! draws, pending events). A bad snapshot is refused and changes nothing.

use og_core::math::Vec3;
use og_core::{ErrorKind, Rng, Schema};
use og_ecs::{Entity, QueryDesc, Reflect, SystemDesc, World, WorldDesc};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Pos {
    p: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Heat {
    t: f64,
    level: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Frozen;

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Ping {
    from: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Log {
    pings: u64,
    last: u64,
}

fn build(seed: u32) -> World {
    let mut w = World::new(&WorldDesc {
        seed,
        tick_rate: 30,
    })
    .unwrap();
    w.register::<Pos>().unwrap();
    w.register::<Heat>().unwrap();
    w.register::<Frozen>().unwrap();
    w.register::<Ping>().unwrap();
    w.register::<Log>().unwrap();
    // A host-declared layout with a raw resource and raw components.
    let wind = w
        .register_schema(Schema::from_json(r#"{"name":"Wind","version":1,"fields":[{"name":"dir","type":"vec3"},{"name":"gust","type":"u32"}]}"#).unwrap())
        .unwrap();
    let mut wind_bytes = Vec::new();
    for f in [0.5f32, 0.0, -0.5] {
        wind_bytes.extend(f.to_le_bytes());
    }
    wind_bytes.extend(7u32.to_le_bytes());
    w.insert_resource_bytes(wind, &wind_bytes).unwrap();
    w.insert_resource(Log::default()).unwrap();
    let heat_t = w.field::<Heat>("t").unwrap();
    w.add_system(
        SystemDesc::new("heat")
            .write::<Heat>()
            .write::<Rng>()
            .write::<Ping>()
            .write::<Frozen>()
            .query(QueryDesc::new().write::<Heat>().without::<Frozen>()),
        move |ctx| {
            let mut freeze = Vec::new();
            {
                let mut rng = ctx.resource_mut::<Rng>()?;
                for a in ctx.query(0)? {
                    let mut t = a.write::<f64>(heat_t)?;
                    for (row, e) in a.entities().iter().enumerate() {
                        t[row] += rng.next_f64() - 0.45;
                        if t[row] > 3.0 {
                            freeze.push(*e);
                            ctx.send(Ping { from: e.to_bits() })?;
                        }
                    }
                }
            }
            for e in freeze {
                ctx.insert(e, Frozen)?;
            }
            Ok(())
        },
    )
    .unwrap();
    w.add_system(
        SystemDesc::new("log").read::<Ping>().write::<Log>(),
        |ctx| {
            let pings = ctx.events::<Ping>()?;
            let mut log = ctx.resource_mut::<Log>()?;
            log.pings += pings.len() as u64;
            if let Some(p) = pings.last() {
                log.last = p.from;
            }
            Ok(())
        },
    )
    .unwrap();
    w
}

/// Deterministic host-side churn mixed with ticks.
fn churn(w: &mut World, rng: &mut Rng, rounds: usize) -> Vec<Entity> {
    let mut spawned = Vec::new();
    for _ in 0..rounds {
        for _ in 0..20 {
            let e = match rng.integer(0, 2) {
                0 => w
                    .spawn((Pos {
                        p: Vec3::splat(rng.range_f32(-1.0, 1.0)),
                    },))
                    .unwrap(),
                1 => w
                    .spawn((Pos { p: Vec3::X }, Heat { t: 0.0, level: 1 }))
                    .unwrap(),
                _ => w
                    .spawn((Heat {
                        t: rng.next_f64(),
                        level: 2,
                    },))
                    .unwrap(),
            };
            spawned.push(e);
        }
        let live = w.entities();
        for _ in 0..12 {
            let e = live[rng.integer(0, live.len() as i64 - 1) as usize];
            if w.is_alive(e) {
                w.despawn(e).unwrap();
            }
        }
        w.send(Ping { from: 1 }).unwrap();
        w.step().unwrap();
    }
    spawned
}

#[test]
fn restore_gives_the_identical_world_and_the_same_future() {
    let mut a = build(5);
    let mut rng = Rng::new(1);
    churn(&mut a, &mut rng, 40);
    w_has_pending_events(&mut a);
    let bytes = a.snapshot();
    let hash = a.world_hash();

    let mut b = build(5);
    b.restore(&bytes).unwrap();
    assert_eq!(b.world_hash(), hash);
    assert_eq!(b.snapshot(), bytes);
    assert_eq!(b.entities(), a.entities());
    assert_eq!(b.time(), a.time());
    assert_eq!(*b.resource::<Rng>().unwrap(), *a.resource::<Rng>().unwrap());
    assert_eq!(
        b.pending_events::<Ping>().unwrap(),
        a.pending_events::<Ping>().unwrap()
    );
    let wind = b.id_by_name("Wind").unwrap();
    assert_eq!(
        b.resource_bytes(wind).unwrap(),
        a.resource_bytes(wind).unwrap()
    );

    // The same future: the same ids are handed out, the same draws are made.
    let (mut ra, mut rb) = (rng, rng);
    let ea = churn(&mut a, &mut ra, 30);
    let eb = churn(&mut b, &mut rb, 30);
    assert_eq!(ea, eb);
    assert_eq!(a.world_hash(), b.world_hash());
    assert!(a.resource::<Log>().unwrap().pings > 30);

    // Restoring the old snapshot brings the old world back.
    a.restore(&bytes).unwrap();
    assert_eq!(a.world_hash(), hash);
}

fn w_has_pending_events(w: &mut World) {
    w.send(Ping { from: 99 }).unwrap();
    assert!(!w.pending_events::<Ping>().unwrap().is_empty());
}

#[test]
fn the_hash_sees_every_part_of_the_state() {
    let base = || {
        let mut w = build(3);
        churn(&mut w, &mut Rng::new(2), 5);
        w
    };
    let h = base().world_hash();
    assert_eq!(base().world_hash(), h);
    let mut w = base();
    let e = w.entities()[3];
    let p = w.get::<Pos>(e).ok();
    match p {
        Some(mut p) => {
            p.p.x += 1e-6;
            w.insert(e, p).unwrap();
        }
        None => w.insert(e, Pos { p: Vec3::ZERO }).unwrap(),
    }
    assert_ne!(w.world_hash(), h, "a component value");
    let mut w = base();
    w.resource_mut::<Rng>().unwrap().next_u32();
    assert_ne!(w.world_hash(), h, "the rng state");
    let mut w = base();
    w.send(Ping { from: 5 }).unwrap();
    assert_ne!(w.world_hash(), h, "a pending event");
    let mut w = base();
    let e = w.spawn(()).unwrap();
    w.despawn(e).unwrap();
    assert_ne!(w.world_hash(), h, "the allocator's free list");
}

#[test]
fn a_bad_snapshot_is_refused_and_changes_nothing() {
    let mut a = build(5);
    churn(&mut a, &mut Rng::new(4), 10);
    let bytes = a.snapshot();

    let mut target = build(5);
    churn(&mut target, &mut Rng::new(8), 3);
    let before = target.world_hash();
    let refuse = |target: &mut World, data: &[u8]| {
        let err = target.restore(data).unwrap_err();
        assert_eq!(
            target.world_hash(),
            before,
            "a refused restore changed the world: {err}"
        );
        err
    };

    assert_eq!(refuse(&mut target, b"nope").kind(), ErrorKind::Parse);
    for cut in (0..bytes.len()).step_by(97).chain([bytes.len() - 1]) {
        assert_eq!(
            refuse(&mut target, &bytes[..cut]).kind(),
            ErrorKind::Parse,
            "cut at {cut}"
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(refuse(&mut target, &trailing).reason().contains("trailing"));
    // Random corruption never panics; whatever is refused leaves no trace.
    let mut rng = Rng::new(77);
    for _ in 0..300 {
        let mut bad = bytes.clone();
        let i = rng.integer(0, bad.len() as i64 - 1) as usize;
        bad[i] ^= 1 << rng.integer(0, 7);
        let mut scratch = build(5);
        let h = scratch.world_hash();
        if scratch.restore(&bad).is_err() {
            assert_eq!(scratch.world_hash(), h);
        }
    }

    // A world without one of the snapshot's types, or with another layout.
    let mut missing = World::new(&WorldDesc::default()).unwrap();
    missing.register::<Pos>().unwrap();
    let err = missing.restore(&bytes).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert!(err.reason().contains("'Heat'"), "{}", err.reason());

    #[derive(Clone, Copy, Reflect)]
    #[reflect(name = "Heat")]
    struct OtherHeat {
        t: f32,
    }
    let mut other = World::new(&WorldDesc::default()).unwrap();
    other.register::<Pos>().unwrap();
    other.register::<OtherHeat>().unwrap();
    let err = other.restore(&bytes).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Schema);
    assert!(
        err.reason().contains("different layout"),
        "{}",
        err.reason()
    );
}

#[test]
fn restore_maps_types_by_name_across_registration_order() {
    let mut a = build(5);
    churn(&mut a, &mut Rng::new(4), 10);
    let bytes = a.snapshot();
    // The same types registered in another order (plus an extra one).
    let mut b = World::new(&WorldDesc::default()).unwrap();
    #[derive(Clone, Copy, Reflect)]
    struct Extra {
        x: f32,
    }
    b.register::<Extra>().unwrap();
    b.register::<Log>().unwrap();
    b.register::<Frozen>().unwrap();
    b.register::<Heat>().unwrap();
    b.register::<Ping>().unwrap();
    b.register::<Pos>().unwrap();
    b.register_schema(Schema::from_json(r#"{"name":"Wind","version":1,"fields":[{"name":"dir","type":"vec3"},{"name":"gust","type":"u32"}]}"#).unwrap())
        .unwrap();
    b.restore(&bytes).unwrap();
    assert_eq!(b.entities(), a.entities());
    for e in a.entities() {
        assert_eq!(b.get::<Pos>(e).ok(), a.get::<Pos>(e).ok());
        assert_eq!(b.get::<Heat>(e).ok(), a.get::<Heat>(e).ok());
        assert_eq!(b.has::<Frozen>(e).unwrap(), a.has::<Frozen>(e).unwrap());
    }
    assert_eq!(*b.resource::<Log>().unwrap(), *a.resource::<Log>().unwrap());
    // Iteration order inside an archetype survives.
    let order = |w: &World| {
        w.query(&QueryDesc::new().read::<Heat>())
            .unwrap()
            .flat_map(|v| v.entities().to_vec())
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&b), order(&a));
}

#[test]
fn a_retired_entity_slot_stays_retired_across_restore() {
    use og_core::handle::MAX_GENERATION;
    let mut w = build(5);
    let mut e = w.spawn((Frozen,)).unwrap();
    while e.generation() < MAX_GENERATION {
        w.despawn(e).unwrap();
        e = w.spawn((Frozen,)).unwrap();
        assert_eq!(e.index(), 0);
    }
    // The last id slot 0 gives out is an exact JavaScript number.
    assert!(e.to_bits() < 1 << 53);
    assert_eq!(Entity::from_bits(e.to_bits() as f64 as u64), e);
    w.despawn(e).unwrap();

    let mut back = build(5);
    back.restore(&w.snapshot()).unwrap();
    assert_eq!(back.world_hash(), w.world_hash());
    for world in [&mut w, &mut back] {
        let next = world.spawn((Frozen,)).unwrap();
        assert_eq!((next.index(), next.generation()), (1, 1), "slot 0 retired");
        assert_eq!(world.despawn(e).unwrap_err().kind(), ErrorKind::DeadHandle);
    }
    assert_eq!(back.world_hash(), w.world_hash());
}

/// A system can snapshot the world while it holds its own columns (a host
/// system's typed views stay borrowed while it runs): the snapshot is the
/// world as that system sees it, its queued structural changes not in it.
#[test]
fn a_system_snapshots_the_world_while_it_holds_its_columns() {
    use std::sync::{Arc, Mutex};
    let mut w = build(5);
    churn(&mut w, &mut Rng::new(4), 3);
    let taken: Arc<Mutex<Option<Vec<u8>>>> = Arc::default();
    let out = taken.clone();
    let heat_t = w.field::<Heat>("t").unwrap();
    w.add_system(
        SystemDesc::new("saver")
            .write::<Heat>()
            .write::<Pos>()
            .query(QueryDesc::new().write::<Heat>()),
        move |ctx| {
            if out.lock().unwrap().is_some() {
                return Ok(());
            }
            let mut columns = Vec::new();
            for a in ctx.query(0)? {
                let mut t = a.write::<f64>(heat_t)?;
                t[0] = 42.0;
                columns.push(t);
            }
            ctx.spawn((Pos { p: Vec3::Y },))?;
            *out.lock().unwrap() = Some(ctx.snapshot());
            drop(columns);
            Ok(())
        },
    )
    .unwrap();
    let live = w.entities().len();
    w.step().unwrap();
    let bytes = taken.lock().unwrap().take().unwrap();
    // The spawn the system queued applied when it returned: one more now than in the snapshot.
    assert_eq!(w.entities().len(), live + 1);
    let mut b = build(5);
    b.restore(&bytes).unwrap();
    assert_eq!(b.entities().len(), live);
    // Its own write through the held column is in it.
    let heats: Vec<f64> = b
        .query(&QueryDesc::new().read::<Heat>())
        .unwrap()
        .filter(|a| !a.is_empty())
        .map(|a| a.read::<f64>(heat_t).unwrap()[0])
        .collect();
    assert!(heats.iter().all(|&t| t == 42.0), "{heats:?}");
}
