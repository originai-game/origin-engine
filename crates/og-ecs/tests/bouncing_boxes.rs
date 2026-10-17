//! A Rust-only "bouncing boxes" world: boxes fall, bounce off the walls of
//! an arena with random jitter, emit events, retire after enough bounces and
//! are replaced by newly spawned ones. Advanced 3600 ticks (one minute at
//! 60 Hz) twice from the same seed, it ends with the same world hash; and a
//! run snapshotted halfway and restored into a fresh world ends there too.

use og_core::math::{sin, Vec3};
use og_core::Rng;
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Position {
    #[reflect(unit = "m")]
    p: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Velocity {
    #[reflect(unit = "m/s")]
    v: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "BoxShape")]
struct Shape {
    #[reflect(unit = "m", min = 0.05, max = 2.0)]
    half: f32,
    bounces: u32,
}

/// Sent when a box hits a wall.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Bounce {
    entity: u64,
    speed: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Stats {
    bounces: u64,
    spawned: u64,
    retired: u64,
    energy: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Arena {
    #[reflect(unit = "m", min = 1.0, max = 100.0, tunable)]
    extent: f32,
    #[reflect(unit = "m/s2", tunable)]
    gravity: f32,
    #[reflect(min = 0.0, max = 1.0, tunable)]
    restitution: f32,
}

fn random_box(rng: &mut Rng, extent: f32) -> (Position, Velocity, Shape) {
    let half = rng.range_f32(0.1, 0.6);
    let room = extent - half;
    (
        Position {
            p: Vec3::new(
                rng.range_f32(-room, room),
                rng.range_f32(0.0, room),
                rng.range_f32(-room, room),
            ),
        },
        Velocity {
            v: Vec3::new(
                rng.range_f32(-8.0, 8.0),
                rng.range_f32(-2.0, 12.0),
                rng.range_f32(-8.0, 8.0),
            ),
        },
        Shape { half, bounces: 0 },
    )
}

fn build(seed: u32) -> World {
    let mut w = World::new(&WorldDesc {
        seed,
        ..WorldDesc::default()
    })
    .unwrap();
    for r in [
        w.register::<Position>(),
        w.register::<Velocity>(),
        w.register::<Shape>(),
        w.register::<Bounce>(),
        w.register::<Stats>(),
        w.register::<Arena>(),
    ] {
        r.unwrap();
    }
    let arena = Arena {
        extent: 10.0,
        gravity: 9.81,
        restitution: 0.9,
    };
    w.insert_resource(arena).unwrap();
    w.insert_resource(Stats::default()).unwrap();
    let mut rng = *w.resource::<Rng>().unwrap();
    for _ in 0..200 {
        w.spawn(random_box(&mut rng, arena.extent)).unwrap();
    }
    *w.resource_mut::<Rng>().unwrap() = rng;

    let pos = w.field::<Position>("p").unwrap();
    let vel = w.field::<Velocity>("v").unwrap();
    let half = w.field::<Shape>("half").unwrap();
    let bounces = w.field::<Shape>("bounces").unwrap();

    w.add_system(
        SystemDesc::new("spawn_boxes")
            .read::<Arena>()
            .write::<Position>()
            .write::<Velocity>()
            .write::<Shape>()
            .write::<Rng>()
            .write::<Stats>(),
        |ctx| {
            if ctx.time().tick % 30 != 0 {
                return Ok(());
            }
            let extent = ctx.resource::<Arena>()?.extent;
            let n = ctx.resource_mut::<Rng>()?.integer(1, 3);
            for _ in 0..n {
                let b = random_box(&mut *ctx.resource_mut::<Rng>()?, extent);
                ctx.spawn(b)?;
            }
            ctx.resource_mut::<Stats>()?.spawned += n as u64;
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("gravity")
            .read::<Arena>()
            .write::<Velocity>()
            .query(QueryDesc::new().write::<Velocity>()),
        move |ctx| {
            let dv = ctx.resource::<Arena>()?.gravity * ctx.time().dt;
            for a in ctx.query(0)? {
                for v in a.write::<Vec3>(vel)?.iter_mut() {
                    v.y -= dv;
                }
            }
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("integrate")
            .read::<Velocity>()
            .write::<Position>()
            .query(QueryDesc::new().read::<Velocity>().write::<Position>()),
        move |ctx| {
            let dt = ctx.time().dt;
            for a in ctx.query(0)? {
                let v = a.read::<Vec3>(vel)?;
                let mut p = a.write::<Vec3>(pos)?;
                for (p, v) in p.iter_mut().zip(v.iter()) {
                    *p += *v * dt;
                }
            }
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("collide")
            .read::<Arena>()
            .write::<Position>()
            .write::<Velocity>()
            .write::<Shape>()
            .write::<Bounce>()
            .write::<Rng>()
            .query(
                QueryDesc::new()
                    .write::<Position>()
                    .write::<Velocity>()
                    .write::<Shape>(),
            ),
        move |ctx| {
            let arena = *ctx.resource::<Arena>()?;
            let wobble = sin(ctx.time().elapsed as f32 * 1.7) * 0.05;
            let mut rng = ctx.resource_mut::<Rng>()?;
            for a in ctx.query(0)? {
                let mut p = a.write::<Vec3>(pos)?;
                let mut v = a.write::<Vec3>(vel)?;
                let h = a.read::<f32>(half)?;
                let mut n = a.write::<u32>(bounces)?;
                for row in 0..a.len() {
                    let limit = arena.extent - h[row];
                    let mut hit = false;
                    for axis in 0..3 {
                        let (lo, hi) = if axis == 1 {
                            (h[row] - arena.extent, limit)
                        } else {
                            (-limit, limit)
                        };
                        if p[row][axis] < lo || p[row][axis] > hi {
                            p[row][axis] = p[row][axis].clamp(lo, hi);
                            v[row][axis] = -v[row][axis] * arena.restitution
                                + rng.range_f32(-0.2, 0.2)
                                + wobble;
                            hit = true;
                        }
                    }
                    if hit {
                        n[row] += 1;
                        ctx.send(Bounce {
                            entity: a.entities()[row].to_bits(),
                            speed: v[row].length(),
                        })?;
                    }
                }
            }
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("retire")
            .read::<Shape>()
            .write::<Stats>()
            .query(QueryDesc::new().read::<Shape>()),
        move |ctx| {
            let mut retire = Vec::new();
            for a in ctx.query(0)? {
                let n = a.read::<u32>(bounces)?;
                retire.extend(
                    a.entities()
                        .iter()
                        .zip(n.iter())
                        .filter(|(_, n)| **n > 12)
                        .map(|(e, _)| *e),
                );
            }
            ctx.resource_mut::<Stats>()?.retired += retire.len() as u64;
            for e in retire {
                ctx.despawn(e)?;
            }
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("count")
            .read::<Bounce>()
            .read::<Velocity>()
            .write::<Stats>()
            .query(QueryDesc::new().read::<Velocity>()),
        move |ctx| {
            let delivered = ctx.events::<Bounce>()?;
            let mut energy = 0.0f64;
            for a in ctx.query(0)? {
                for v in a.read::<Vec3>(vel)?.iter() {
                    energy += 0.5 * f64::from(v.length_squared());
                }
            }
            let mut stats = ctx.resource_mut::<Stats>()?;
            stats.bounces += delivered.len() as u64;
            stats.energy = energy;
            Ok(())
        },
    )
    .unwrap();
    w
}

fn run(w: &mut World, ticks: u64) {
    for _ in 0..ticks {
        w.step().unwrap();
    }
}

#[test]
fn the_same_seed_gives_the_same_world_after_3600_ticks() {
    let mut a = build(42);
    let mut b = build(42);
    assert_eq!(a.world_hash(), b.world_hash());
    run(&mut a, 3600);
    run(&mut b, 3600);
    assert_eq!(a.time().tick, 3600);
    assert_eq!(a.world_hash(), b.world_hash());
    assert_eq!(a.snapshot(), b.snapshot());

    // The minute did something: boxes bounced, retired and were replaced.
    let stats = *a.resource::<Stats>().unwrap();
    assert!(stats.bounces > 1_000, "{stats:?}");
    assert!(stats.spawned > 100 && stats.retired > 100, "{stats:?}");
    assert_eq!(a.entity_count() as u64, 200 + stats.spawned - stats.retired);
    assert!(stats.energy > 0.0);

    // A different seed is a different world.
    let mut c = build(43);
    run(&mut c, 3600);
    assert_ne!(c.world_hash(), a.world_hash());
}

#[test]
fn a_run_snapshotted_halfway_and_restored_ends_in_the_same_world() {
    let mut whole = build(7);
    run(&mut whole, 3600);

    let mut first = build(7);
    run(&mut first, 1800);
    let half = first.snapshot();
    let mut second = build(99); // a different seed: the snapshot carries the rng
    second.restore(&half).unwrap();
    assert_eq!(second.world_hash(), first.world_hash());
    run(&mut second, 1800);
    run(&mut first, 1800);
    assert_eq!(second.time().tick, 3600);
    assert_eq!(second.world_hash(), whole.world_hash());
    assert_eq!(first.world_hash(), whole.world_hash());
}
