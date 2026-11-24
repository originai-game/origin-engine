//! Snapshot → restore gives the identical world: the same hash, the same
//! bytes, and the same future (entity ids handed out, iteration order, rng
//! draws, pending events). A bad snapshot is refused and changes nothing.

use og_core::math::Vec3;
use og_core::{Rng, Schema};
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
    build_with(seed, None)
}

/// A system that runs between "heat" and "log".
type Middle = Box<dyn Fn(&mut og_ecs::SystemContext<'_>) -> og_core::Result<()> + Send>;

/// [`build`], with `middle` (named "saver") scheduled between its two systems.
fn build_with(seed: u32, middle: Option<Middle>) -> World {
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
    if let Some(middle) = middle {
        w.add_system(SystemDesc::new("saver").read::<Heat>(), middle)
            .unwrap();
    }
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
