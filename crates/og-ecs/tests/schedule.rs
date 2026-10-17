//! The schedule's contract: declarations are checked at registration and
//! enforced at run time; structural changes land when a system returns;
//! events sent in tick N are read in tick N+1; a failing system stops the
//! tick with the tick and system named.

use og_core::math::Vec3;
use og_core::{ErrorKind, Rng, Time};
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Position {
    p: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Velocity {
    v: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Hit {
    n: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Score {
    points: u64,
}

fn world() -> World {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    w.register::<Position>().unwrap();
    w.register::<Velocity>().unwrap();
    w.register::<Hit>().unwrap();
    w.register::<Score>().unwrap();
    w.insert_resource(Score::default()).unwrap();
    w
}

fn refused(w: &mut World, desc: SystemDesc) -> og_core::Error {
    w.add_system(desc, |_| Ok(())).unwrap_err()
}

#[test]
fn registration_refuses_undeclared_access_and_names_it() {
    let mut w = world();
    let err = refused(
        &mut w,
        SystemDesc::new("move")
            .read::<Velocity>()
            .query(QueryDesc::new().read::<Velocity>().write::<Position>()),
    );
    assert_eq!(err.kind(), ErrorKind::Access);
    assert!(
        err.reason()
            .contains("system 'move' query 0 writes 'Position'"),
        "{}",
        err.reason()
    );

    let err = refused(
        &mut w,
        SystemDesc::new("peek").query(QueryDesc::new().read::<Velocity>()),
    );
    assert_eq!(err.kind(), ErrorKind::Access);
    assert!(
        err.reason().contains("reads 'Velocity'"),
        "{}",
        err.reason()
    );

    let mut typo = SystemDesc::new("typo");
    typo.writes.push("Positoin".into());
    let err = refused(&mut w, typo);
    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert!(err.reason().contains("'Positoin'"), "{}", err.reason());

    let err = refused(&mut w, SystemDesc::new("clock").write::<Time>());
    assert_eq!(err.kind(), ErrorKind::Access);

    let err = refused(
        &mut w,
        SystemDesc::new("never")
            .read::<Position>()
            .query(QueryDesc::new().read::<Position>().without::<Position>()),
    );
    assert_eq!(err.kind(), ErrorKind::Invalid);

    w.add_system(SystemDesc::new("ok").write::<Position>(), |_| Ok(()))
        .unwrap();
    assert_eq!(
        refused(&mut w, SystemDesc::new("ok")).kind(),
        ErrorKind::Invalid
    );
    assert_eq!(
        refused(&mut w, SystemDesc::new("")).kind(),
        ErrorKind::Invalid
    );
    assert_eq!(w.systems(), ["ok"]);
}

type Body = fn(&mut og_ecs::SystemContext<'_>) -> og_core::Result<()>;

/// Runs one system once and returns what it returned.
fn run_once(
    desc: SystemDesc,
    body: impl Fn(&mut og_ecs::SystemContext<'_>) -> og_core::Result<()> + Send + 'static,
) -> og_core::Result<()> {
    let mut w = world();
    w.spawn((Position { p: Vec3::ZERO }, Velocity { v: Vec3::X }))
        .unwrap();
    w.add_system(desc, body).unwrap();
    w.step()
}

#[test]
fn run_time_access_outside_the_declaration_is_refused() {
    // Writing a column the query only reads; the field is named.
    let err = run_once(
        SystemDesc::new("sneak")
            .read::<Position>()
            .query(QueryDesc::new().read::<Position>()),
        |ctx| {
            let p = ctx.id::<Position>()?;
            let field = og_ecs::FieldRef {
                component: p,
                field: 0,
                ty: og_core::FieldType::Vec3,
            };
            for a in ctx.query(0)? {
                a.write::<Vec3>(field)?;
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Access);
    assert!(
        err.reason().contains("tick 0, system 'sneak'"),
        "{}",
        err.reason()
    );
    assert!(err.reason().contains("Position.p"), "{}", err.reason());

    let cases: Vec<(&str, SystemDesc, Body)> = vec![
        (
            "resource_mut",
            SystemDesc::new("s").read::<Score>(),
            |ctx| ctx.resource_mut::<Score>().map(drop),
        ),
        ("resource", SystemDesc::new("s"), |ctx| {
            ctx.resource::<Score>().map(drop)
        }),
        ("rng", SystemDesc::new("s"), |ctx| {
            ctx.resource_mut::<Rng>().map(drop)
        }),
        ("send", SystemDesc::new("s").read::<Hit>(), |ctx| {
            ctx.send(Hit { n: 1 })
        }),
        ("events", SystemDesc::new("s"), |ctx| {
            ctx.events::<Hit>().map(drop)
        }),
        ("spawn", SystemDesc::new("s").write::<Position>(), |ctx| {
            ctx.spawn((Position { p: Vec3::ZERO }, Hit { n: 0 }))
                .map(drop)
        }),
        ("get", SystemDesc::new("s"), |ctx| {
            let e = og_ecs::Entity::from_bits(1 << 32);
            ctx.get::<Position>(e).map(drop)
        }),
        ("query index", SystemDesc::new("s"), |ctx| {
            ctx.query(0).map(drop)
        }),
    ];
    for (what, desc, body) in cases {
        let err = run_once(desc, body).unwrap_err();
        let kind = if what == "query index" {
            ErrorKind::NotFound
        } else {
            ErrorKind::Access
        };
        assert_eq!(err.kind(), kind, "{what}: {}", err.reason());
    }
    // Time is readable by every system without a declaration.
    run_once(SystemDesc::new("clock"), |ctx| {
        ctx.resource::<Time>().map(drop)
    })
    .unwrap();
}

#[test]
fn structural_changes_land_when_the_system_returns() {
    let mut w = world();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    w.add_system(
        SystemDesc::new("spawner")
            .write::<Position>()
            .query(QueryDesc::new().read::<Position>()),
        move |ctx| {
            let before = ctx.query(0)?.entity_count();
            let e = ctx.spawn((Position { p: Vec3::Y },))?;
            // Not placed yet: invisible to this system's queries and reads.
            assert_eq!(ctx.query(0)?.entity_count(), before);
            assert_eq!(
                ctx.get::<Position>(e).unwrap_err().kind(),
                ErrorKind::NotFound
            );
            log.lock().unwrap().push(("spawner", before));
            Ok(())
        },
    )
    .unwrap();
    let log = seen.clone();
    w.add_system(
        SystemDesc::new("observer")
            .read::<Position>()
            .query(QueryDesc::new().read::<Position>()),
        move |ctx| {
            log.lock()
                .unwrap()
                .push(("observer", ctx.query(0)?.entity_count()));
            Ok(())
        },
    )
    .unwrap();
    w.step().unwrap();
    w.step().unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        [
            ("spawner", 0),
            ("observer", 1),
            ("spawner", 1),
            ("observer", 2)
        ]
    );
    assert_eq!(w.entity_count(), 2);
}

#[test]
fn events_sent_in_one_tick_are_read_in_the_next() {
    let mut w = world();
    w.add_system(SystemDesc::new("emit").write::<Hit>(), |ctx| {
        ctx.send(Hit {
            n: ctx.time().tick as u32,
        })
    })
    .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    w.add_system(SystemDesc::new("read").read::<Hit>(), move |ctx| {
        let tick = ctx.time().tick;
        log.lock().unwrap().push((tick, ctx.events::<Hit>()?));
        Ok(())
    })
    .unwrap();
    w.send(Hit { n: 100 }).unwrap(); // from the host, before tick 0
    for _ in 0..3 {
        w.step().unwrap();
    }
    assert_eq!(
        *seen.lock().unwrap(),
        [
            (0, vec![Hit { n: 100 }]),
            (1, vec![Hit { n: 0 }]),
            (2, vec![Hit { n: 1 }])
        ]
    );
    assert_eq!(w.pending_events::<Hit>().unwrap(), [Hit { n: 2 }]);
}

#[test]
fn a_failing_system_stops_the_tick_and_says_where() {
    let mut w = world();
    w.add_system(SystemDesc::new("score").write::<Score>(), |ctx| {
        ctx.resource_mut::<Score>()?.points += 1;
        Ok(())
    })
    .unwrap();
    w.add_system(
        SystemDesc::new("spawn_then_fail").write::<Position>(),
        |ctx| {
            ctx.spawn((Position { p: Vec3::ZERO },))?;
            if ctx.time().tick == 2 {
                return Err(og_core::Error::new(
                    ErrorKind::Invalid,
                    "the third tick is cursed",
                ));
            }
            Ok(())
        },
    )
    .unwrap();
    w.step().unwrap();
    w.step().unwrap();
    let err = w.step().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert_eq!(
        err.reason(),
        "tick 2, system 'spawn_then_fail': the third tick is cursed"
    );
    // Time did not advance; the failing system's spawn was dropped; the
    // earlier system's effect stays.
    assert_eq!(w.time().tick, 2);
    assert_eq!(w.entity_count(), 2);
    assert_eq!(w.resource::<Score>().unwrap().points, 3);
}

#[test]
fn time_advances_by_ticks_without_drift() {
    let mut w = World::new(&WorldDesc {
        tick_rate: 50,
        seed: 0,
    })
    .unwrap();
    for _ in 0..1000 {
        w.step().unwrap();
    }
    let t = w.time();
    assert_eq!(t.tick, 1000);
    assert_eq!(t.dt, 0.02f32);
    assert_eq!(t.elapsed, 1000.0 * f64::from(0.02f32));
    assert_eq!(
        World::new(&WorldDesc {
            tick_rate: 0,
            seed: 0
        })
        .err()
        .unwrap()
        .kind(),
        ErrorKind::Invalid
    );
}

#[test]
fn commands_after_a_despawn_in_the_same_batch() {
    // Despawning twice in one batch is a no-op the second time.
    let mut w = world();
    let e = w.spawn((Position { p: Vec3::ZERO },)).unwrap();
    w.add_system(SystemDesc::new("twice").write::<Position>(), move |ctx| {
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
            ctx.despawn(e)?;
        }
        Ok(())
    })
    .unwrap();
    w.step().unwrap();
    assert!(!w.is_alive(e));
    assert_eq!(w.time().tick, 1);

    // An insert (or remove) queued after a despawn of the same entity stops
    // the tick: the commands before it are applied, it and the rest dropped.
    for remove in [false, true] {
        let mut w = world();
        let e = w.spawn((Position { p: Vec3::ZERO },)).unwrap();
        w.add_system(
            SystemDesc::new("late")
                .write::<Position>()
                .write::<Velocity>(),
            move |ctx| {
                ctx.despawn(e)?;
                if remove {
                    ctx.remove::<Position>(e)?;
                } else {
                    ctx.insert(e, Velocity { v: Vec3::X })?;
                }
                ctx.spawn((Position { p: Vec3::Y },))?;
                Ok(())
            },
        )
        .unwrap();
        let err = w.step().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DeadHandle, "{}", err.reason());
        assert!(
            err.reason().starts_with("tick 0, system 'late': "),
            "{}",
            err.reason()
        );
        assert!(!w.is_alive(e));
        assert_eq!(
            w.entity_count(),
            0,
            "the spawn after the failure is dropped"
        );
        assert_eq!(w.time().tick, 0);
    }
}
