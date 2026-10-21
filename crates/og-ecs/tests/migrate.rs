//! A hot reload changes the game's types; `restore_migrating` carries the
//! world across by name: added fields start at their defaults, removed
//! fields and types are dropped, enum values follow their variant names, and
//! entity ids, iteration order, names, `Time` and the rng stay.

use og_core::{ErrorKind, Rng};
use og_ecs::{Entity, QueryDesc, Reflect, World, WorldDesc};

/// The game before the reload.
mod v1 {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Health")]
    pub struct Health {
        pub hp: f32,
        pub regen: f32,
        #[reflect(variants = ["ok", "poisoned", "burning"])]
        pub status: u32,
    }

    /// A type the reload deletes.
    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    pub struct Legacy {
        pub x: u32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Tuning")]
    pub struct Tuning {
        pub speed: f32,
        pub old_knob: u32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Damage")]
    pub struct Damage {
        pub amount: f32,
        pub crit: bool,
    }
}

/// The game after the reload: `Health.regen` removed, `Health.armor` added
/// (default 3), `Health.status` lost `poisoned` and moved `burning`;
/// `Legacy` deleted; `Tuning.old_knob` removed, `Tuning.gravity` added;
/// `Damage.crit` changed type.
mod v2 {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Health")]
    pub struct Health {
        #[reflect(default = 3)]
        pub armor: u32,
        pub hp: f32,
        #[reflect(variants = ["burning", "ok"], default = 1)]
        pub status: u32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Tuning")]
    pub struct Tuning {
        pub speed: f32,
        #[reflect(default = -9.8)]
        pub gravity: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Damage")]
    pub struct Damage {
        pub amount: f32,
        pub crit: f32,
    }

    /// A type only the new bundle has.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
    pub struct Fresh {
        pub n: u32,
    }
}

fn old_world() -> (World, Vec<Entity>) {
    let mut w = World::new(&WorldDesc {
        seed: 9,
        tick_rate: 60,
    })
    .unwrap();
    w.register::<v1::Health>().unwrap();
    w.register::<v1::Legacy>().unwrap();
    w.register::<v1::Tuning>().unwrap();
    w.register::<v1::Damage>().unwrap();
    w.insert_resource(v1::Tuning {
        speed: 4.0,
        old_knob: 1,
    })
    .unwrap();
    let mut es = Vec::new();
    for i in 0..6u32 {
        let h = v1::Health {
            hp: 10.0 * i as f32,
            regen: 0.5,
            status: i % 3,
        };
        // Even entities also carry Legacy: two archetypes the reload merges.
        let e = if i % 2 == 0 {
            w.spawn((h, v1::Legacy { x: i })).unwrap()
        } else {
            w.spawn((h,)).unwrap()
        };
        es.push(e);
    }
    w.despawn(es[1]).unwrap();
    w.set_name(es[2], "boss").unwrap();
    w.send(v1::Damage {
        amount: 2.5,
        crit: true,
    })
    .unwrap();
    for _ in 0..7 {
        w.step().unwrap();
    }
    w.resource_mut::<Rng>().unwrap().next_u32();
    (w, es)
}

fn new_world() -> World {
    let mut w = World::new(&WorldDesc {
        seed: 1,
        tick_rate: 60,
    })
    .unwrap();
    w.register::<v2::Fresh>().unwrap();
    w.register::<v2::Health>().unwrap();
    w.register::<v2::Tuning>().unwrap();
    w.register::<v2::Damage>().unwrap();
    w.insert_resource(v2::Fresh { n: 42 }).unwrap();
    w
}

#[test]
fn a_migrating_restore_maps_fields_by_name() {
    let (old, es) = old_world();
    let bytes = old.snapshot();

    let mut w = new_world();
    // The strict restore refuses the changed layout and changes nothing.
    let before = w.world_hash();
    let err = w.restore(&bytes).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Schema, "{err}");
    assert_eq!(w.world_hash(), before);

    let report = w.restore_migrating(&bytes).unwrap();
    assert_eq!(report.migrated, ["Health", "Tuning", "Damage"]);
    assert_eq!(
        report.added,
        ["Health.armor", "Tuning.gravity", "Damage.crit"]
    );
    assert_eq!(
        report.removed,
        ["Health.regen", "Tuning.old_knob", "Damage.crit"]
    );
    assert_eq!(report.dropped, ["Legacy"]);
    assert_eq!(report.warnings.len(), 2, "{:?}", report.warnings);
    assert!(report.warnings[0].contains("Health.status: variant(s) poisoned no longer exist"));
    assert!(report.warnings[1].contains("Damage.crit changed type from bool to f32"));
    assert!(!report.is_exact());

    // Entities keep their ids; values move by field name.
    assert_eq!(w.entities(), old.entities());
    let status = |i: usize| w.get::<v2::Health>(es[i]).unwrap().status;
    for i in [0, 2, 3, 4, 5] {
        let h = w.get::<v2::Health>(es[i]).unwrap();
        assert_eq!(h.hp, 10.0 * i as f32);
        assert_eq!(h.armor, 3, "a new field starts at its default");
    }
    // v1 status i % 3: ok(0) -> v2 ok(1); poisoned(1) -> gone, default ok(1);
    // burning(2) -> v2 burning(0).
    assert_eq!(
        [0, 2, 3, 4, 5].map(status),
        [1, 0, 1, 1, 0],
        "ok, burning, ok, poisoned->default, burning"
    );
    // The two archetypes Legacy kept apart are one now, in snapshot order.
    let order: Vec<Entity> = w
        .query(&QueryDesc::new().read::<v2::Health>())
        .unwrap()
        .flat_map(|a| a.entities().to_vec())
        .collect();
    let old_order: Vec<Entity> = old
        .query(&QueryDesc::new().read::<v1::Health>())
        .unwrap()
        .flat_map(|a| a.entities().to_vec())
        .collect();
    assert_eq!(old_order.len(), 5);
    assert_eq!(order, old_order, "rows keep the snapshot's storage order");
    assert_eq!(
        w.query(&QueryDesc::new().read::<v2::Health>())
            .unwrap()
            .count(),
        1,
        "one archetype"
    );

    // Resources and events.
    assert_eq!(
        *w.resource::<v2::Tuning>().unwrap(),
        v2::Tuning {
            speed: 4.0,
            gravity: -9.8
        }
    );
    assert_eq!(
        w.resource::<v2::Fresh>().unwrap().n,
        42,
        "unknown to the snapshot: kept"
    );
    assert!(w.pending_events::<v2::Damage>().unwrap().is_empty());
    assert_eq!(w.find("boss"), Some(es[2]));
    assert_eq!(w.time(), old.time());
    assert_eq!(
        *w.resource::<Rng>().unwrap(),
        *old.resource::<Rng>().unwrap()
    );

    // The new world goes on and spawns where the old one would have.
    let next_old = {
        let mut o = old_world().0;
        o.spawn(()).unwrap()
    };
    assert_eq!(w.spawn(()).unwrap(), next_old);
}

#[test]
fn a_snapshot_that_fits_migrates_to_the_same_world_as_restore() {
    let (old, _) = old_world();
    let bytes = old.snapshot();
    let mut strict = old_world().0;
    strict.step().unwrap();
    strict.restore(&bytes).unwrap();
    let mut migrating = old_world().0;
    migrating.step().unwrap();
    let report = migrating.restore_migrating(&bytes).unwrap();
    assert!(report.is_exact(), "{report:?}");
    assert_eq!(migrating.world_hash(), old.world_hash());
    assert_eq!(migrating.snapshot(), strict.snapshot());
}

#[test]
fn events_in_flight_are_migrated_too() {
    let mut old = World::new(&WorldDesc::default()).unwrap();
    old.register::<v1::Damage>().unwrap();
    old.send(v1::Damage {
        amount: 1.0,
        crit: true,
    })
    .unwrap();
    let mut w = World::new(&WorldDesc::default()).unwrap();
    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    #[reflect(name = "Damage")]
    struct Damage2 {
        amount: f32,
        crit: bool,
        #[reflect(default = 2)]
        stacks: u32,
    }
    w.register::<Damage2>().unwrap();
    let report = w.restore_migrating(&old.snapshot()).unwrap();
    assert_eq!(report.added, ["Damage.stacks"]);
    assert_eq!(
        w.pending_events::<Damage2>().unwrap(),
        [Damage2 {
            amount: 1.0,
            crit: true,
            stacks: 2
        }]
    );
}

#[test]
fn a_malformed_snapshot_is_refused_and_changes_nothing() {
    let (old, _) = old_world();
    let bytes = old.snapshot();
    let mut w = new_world();
    let before = w.world_hash();
    for cut in (0..bytes.len()).step_by(61) {
        let err = w.restore_migrating(&bytes[..cut]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Parse, "cut at {cut}: {err}");
        assert_eq!(w.world_hash(), before, "cut at {cut}");
    }
    assert_eq!(w.resource::<v2::Fresh>().unwrap().n, 42);
}

#[test]
fn check_snapshot_refuses_what_restore_would_naming_the_fields_and_changes_nothing() {
    let (old, _) = old_world();
    let bytes = old.snapshot();
    let w = new_world();
    let before = w.world_hash();
    let err = w.check_snapshot(&bytes, false).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Schema, "{err}");
    // The first type that differs, with what differs.
    assert!(
        err.reason().contains(
            "type 'Health' has a different layout in this world (Health.armor added; Health.regen removed; Health.status's variants changed)"
        ),
        "{err}"
    );
    // A migrating restore would take it.
    w.check_snapshot(&bytes, true).unwrap();
    assert_eq!(w.world_hash(), before);
    // A snapshot of its own world passes; garbage does not.
    w.check_snapshot(&w.snapshot(), false).unwrap();
    assert!(w.check_snapshot(b"OGWS", false).is_err());
    assert!(w.check_snapshot(b"nope", true).is_err());
}
