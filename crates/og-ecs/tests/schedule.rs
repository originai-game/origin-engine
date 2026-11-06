//! The schedule's contract: declarations are checked at registration and
//! enforced at run time; structural changes land when a system returns;
//! events sent in tick N are read in tick N+1; a failing system stops the
//! tick with the tick and system named.

use og_core::math::Vec3;
use og_core::{ErrorKind, Rng, Time};
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};

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
