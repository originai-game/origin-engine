//! What the probe's `dump_world` and the host's view stamps read from the
//! world: `system_info` (declarations and matched counts), entity names (in
//! snapshots and the hash), the structural-change counter, and a
//! component's write stamp in an archetype.

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

#[test]
fn a_components_write_stamp_moves_on_every_write_of_it_and_on_no_other() {
    let mut w = world();
    let a = w.spawn((p(), v())).unwrap();
    let b = w.spawn((p(), v())).unwrap();
    let tagged = w.spawn((p(), Sleeping)).unwrap();
    let (pos, vel, sleeping) = (
        w.id::<Pos>().unwrap(),
        w.id::<Vel>().unwrap(),
        w.id::<Sleeping>().unwrap(),
    );
    let moving = QueryDesc::new().read::<Pos>().write::<Vel>();
    // The stamps of Pos and Vel in the one archetype that has both.
    let stamps = |w: &World| {
        let views: Vec<_> = w.query(&moving).unwrap().collect();
        assert_eq!(views.len(), 1);
        let (p, v) = (views[0].written(pos), views[0].written(vel));
        (p.unwrap(), v.unwrap())
    };

    // Reads move neither.
    let s0 = stamps(&w);
    assert_eq!(w.get::<Pos>(a).unwrap(), p());
    for view in w.query(&moving).unwrap() {
        assert_eq!(
            view.read::<Vec3>(w.field::<Vel>("v").unwrap())
                .unwrap()
                .len(),
            2
        );
    }
    assert_eq!(stamps(&w), s0);

    // A component set in place moves its own stamp alone.
    w.insert(a, Pos { p: Vec3::Y }).unwrap();
    let s1 = stamps(&w);
    assert!(s1.0 > s0.0 && s1.1 == s0.1, "{s0:?} then {s1:?}");

    // So does a column taken to write, written or not; while it is held,
    // nothing can be said of the component.
    for view in w.query(&moving).unwrap() {
        let held = view.write::<Vec3>(w.field::<Vel>("v").unwrap()).unwrap();
        assert_eq!(view.written(vel), None);
        assert_eq!(view.written(pos), Some(s1.0));
        drop(held);
    }
    let s2 = stamps(&w);
    assert!(s2.0 == s1.0 && s2.1 > s1.1, "{s1:?} then {s2:?}");

    // A write elsewhere (another archetype's Pos) moves neither.
    w.insert(tagged, Pos { p: Vec3::X }).unwrap();
    assert_eq!(stamps(&w), s2);

    // A row added or removed moves every component's: one out and one in
    // leave the row count where it was, and the stamps say they changed.
    w.despawn(b).unwrap();
    let s3 = stamps(&w);
    assert!(s3.0 > s2.0 && s3.1 > s2.1, "{s2:?} then {s3:?}");
    w.spawn((p(), v())).unwrap();
    let s4 = stamps(&w);
    assert!(s4.0 > s3.0 && s4.1 > s3.1, "{s3:?} then {s4:?}");

    // A restore fills every column again.
    let snap = w.snapshot();
    w.restore(&snap).unwrap();
    let s5 = stamps(&w);
    assert!(s5.0 > s4.0 && s5.1 > s4.1, "{s4:?} then {s5:?}");

    // No stamp for a component the archetype has not, or one with no field.
    let all = QueryDesc::new().read::<Pos>();
    for view in w.query(&all).unwrap() {
        assert_eq!(view.written(vel).is_some(), view.has(vel));
        assert_eq!(view.written(sleeping), None);
    }
}
