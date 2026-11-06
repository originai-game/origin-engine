//! What the probe's `dump_world` and the host's view stamps read from the
//! world: `system_info` (declarations and matched counts), entity names (in
//! snapshots and the hash), and the structural-change counter.

use og_core::math::Vec3;
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Pos {
    p: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Vel {
    v: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Sleeping;

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Spawner {
    every: u32,
}

fn world() -> World {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    w.register::<Pos>().unwrap();
    w.register::<Vel>().unwrap();
    w.register::<Sleeping>().unwrap();
    w.register::<Spawner>().unwrap();
    w.insert_resource(Spawner { every: 1 }).unwrap();
    w
}

fn p() -> Pos {
    Pos { p: Vec3::ZERO }
}

fn v() -> Vel {
    Vel { v: Vec3::X }
}

#[test]
fn system_info_reports_declarations_and_what_each_query_matched() {
    let mut w = world();
    for _ in 0..3 {
        w.spawn((p(), v())).unwrap();
    }
    for _ in 0..2 {
        w.spawn((p(), v(), Sleeping)).unwrap();
    }
    for _ in 0..4 {
        w.spawn((p(),)).unwrap();
    }
    w.add_system(
        SystemDesc::new("spawn")
            .read::<Spawner>()
            .write::<Pos>()
            .write::<Vel>()
            .query(QueryDesc::new().read::<Vel>()),
        |ctx| {
            ctx.spawn((p(), v()))?;
            Ok(())
        },
    )
    .unwrap();
    w.add_system(
        SystemDesc::new("move")
            .read::<Vel>()
            .read::<Sleeping>()
            .write::<Pos>()
            .query(
                QueryDesc::new()
                    .read::<Vel>()
                    .write::<Pos>()
                    .without::<Sleeping>(),
            )
            .query(QueryDesc::new().read::<Pos>()),
        |_| Ok(()),
    )
    .unwrap();

    let info = w.system_info();
    assert_eq!(info.len(), 2);
    assert_eq!(info[1].name, "move");
    assert_eq!(
        info[1].reads,
        ["Vel", "Sleeping"],
        "id order, writes excluded"
    );
    assert_eq!(info[1].writes, ["Pos"]);
    assert_eq!(info[1].queries.len(), 2);
    assert_eq!(info[1].queries[0].without, ["Sleeping"]);
    assert_eq!(
        (info[1].matched, info[1].last_tick),
        (0, None),
        "not run yet"
    );

    w.step().unwrap();
    let info = w.system_info();
    assert_eq!(info[0].reads, ["Spawner"]);
    assert_eq!(info[0].writes, ["Pos", "Vel"]);
    assert_eq!(
        (info[0].matched, info[0].query_matched.clone()),
        (5, vec![5])
    );
    // "move" runs after "spawn"'s command landed: 3 + 1 awake movers, 10 with Pos.
    assert_eq!(info[1].query_matched, [4, 10]);
    assert_eq!(info[1].matched, 14);
    assert_eq!(info[1].last_tick, Some(0));

    w.step().unwrap();
    let info = w.system_info();
    assert_eq!(info[0].query_matched, [6]);
    assert_eq!(info[1].query_matched, [5, 11]);
    assert_eq!(info[1].last_tick, Some(1));
}

#[test]
fn the_structural_counter_moves_exactly_on_structural_operations() {
    let mut w = world();
    let vel = w.field::<Vel>("v").unwrap();
    let pos_id = w.id::<Pos>().unwrap();
    let mut last = w.structural_changes();
    let mut expect = |w: &World, moved: bool, what: &str| {
        let now = w.structural_changes();
        assert_eq!(now != last, moved, "{what}: {last} -> {now}");
        last = now;
    };

    let e = w.spawn((p(),)).unwrap();
    expect(&w, true, "spawn");
    let f = w.spawn(()).unwrap();
    expect(&w, true, "spawn of an empty entity");
    w.insert(e, v()).unwrap();
    expect(&w, true, "insert of a new component");
    w.insert(e, Vel { v: Vec3::Y }).unwrap();
    expect(&w, false, "insert overwriting a component");
    w.insert_bytes(e, pos_id, &[0; 12]).unwrap();
    expect(&w, false, "insert_bytes overwriting a component");
    {
        let view = w
            .query(&QueryDesc::new().write::<Vel>())
            .unwrap()
            .next()
            .unwrap();
        view.write::<Vec3>(vel).unwrap()[0] = Vec3::Z;
    }
    expect(&w, false, "a write through a view");
    w.resource_mut::<Spawner>().unwrap().every = 3;
    w.insert_resource(Spawner { every: 4 }).unwrap();
    expect(&w, false, "resource writes");
    w.set_name(e, "e").unwrap();
    expect(&w, false, "a name");
    assert!(w.spawn((p(), p())).is_err());
    expect(&w, false, "a refused spawn");
    assert!(w.remove::<Sleeping>(e).is_err());
    expect(&w, false, "a refused remove");
    w.step().unwrap();
    expect(&w, false, "a tick without systems");
    let snap = w.snapshot();
    let hash = w.world_hash();
    expect(&w, false, "snapshot and hash");

    w.remove::<Vel>(e).unwrap();
    expect(&w, true, "remove");
    w.despawn(f).unwrap();
    expect(&w, true, "despawn");
    w.restore(&snap).unwrap();
    expect(&w, true, "restore");
    assert_eq!(w.world_hash(), hash);

    // Systems: commands count as their operations when they are applied;
    // `set` and resource writes inside a system do not count.
    w.add_system(
        SystemDesc::new("tick")
            .write::<Pos>()
            .write::<Vel>()
            .write::<Spawner>(),
        move |ctx| {
            ctx.set(e, Pos { p: Vec3::ONE })?;
            let every = ctx.resource::<Spawner>()?.every;
            if every == 0 {
                return Ok(());
            }
            ctx.resource_mut::<Spawner>()?.every = 0;
            let n = ctx.spawn((p(),))?;
            ctx.insert(n, v())?;
            Ok(())
        },
    )
    .unwrap();
    let before = w.structural_changes();
    w.step().unwrap();
    assert_eq!(w.structural_changes() - before, 2, "a spawn and an insert");
    let before = w.structural_changes();
    w.step().unwrap();
    assert_eq!(
        w.structural_changes(),
        before,
        "a tick that only sets values"
    );
}
