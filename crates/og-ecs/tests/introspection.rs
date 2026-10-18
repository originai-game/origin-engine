//! What the probe's `dump_world` and the host's view stamps read from the
//! world: `system_info` (declarations and matched counts), entity names (in
//! snapshots and the hash), and the structural-change counter.

use og_core::ErrorKind;
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
fn names_address_entities_and_survive_snapshots() {
    let mut w = world();
    let a = w.spawn((p(),)).unwrap();
    let b = w.spawn((p(),)).unwrap();
    let c = w.spawn(()).unwrap();
    let unnamed_hash = w.world_hash();
    w.set_name(a, "hero").unwrap();
    w.set_name(b, "crate_1").unwrap();
    assert_ne!(w.world_hash(), unnamed_hash, "names are in the hash");
    assert_eq!(w.find("hero"), Some(a));
    assert_eq!(w.name(b), Some("crate_1"));
    assert_eq!(w.name(c), None);

    // One entity per name, one name per entity.
    let err = w.set_name(c, "hero").unwrap_err();
    assert!(err.reason().contains("already belongs to"), "{err}");
    for bad in ["", "#3.1", "a b", "x:y", "k=v", "semi;colon"] {
        assert_eq!(
            w.set_name(c, bad).unwrap_err().kind(),
            ErrorKind::Invalid,
            "{bad:?}"
        );
    }
    w.set_name(b, "crate_2").unwrap();
    assert_eq!(w.find("crate_1"), None, "renaming frees the old name");
    assert_eq!(w.names(), [(a, "hero"), (b, "crate_2")]);

    let bytes = w.snapshot();
    let hash = w.world_hash();
    let mut r = world();
    r.restore(&bytes).unwrap();
    assert_eq!(r.find("hero"), Some(a));
    assert_eq!(r.name(b), Some("crate_2"));
    assert_eq!(r.world_hash(), hash);
    assert_eq!(r.snapshot(), bytes);

    // A despawned entity takes its name with it; a restore brings both back.
    r.despawn(a).unwrap();
    assert_eq!(r.find("hero"), None);
    r.set_name(c, "hero").unwrap();
    assert_eq!(r.find("hero"), Some(c));
    r.restore(&bytes).unwrap();
    assert_eq!((r.find("hero"), r.name(c)), (Some(a), None));
    assert_eq!(r.world_hash(), hash);

    // The same names given in another order are the same state.
    let mut x = world();
    let xa = x.spawn((p(),)).unwrap();
    let xb = x.spawn((p(),)).unwrap();
    x.spawn(()).unwrap();
    x.set_name(xb, "crate_2").unwrap();
    x.set_name(xa, "hero").unwrap();
    assert_eq!(x.world_hash(), hash);
    x.clear_name(xa).unwrap();
    assert_eq!(x.find("hero"), None);
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
