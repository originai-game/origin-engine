//! What a game's saves need beyond a restore: a snapshot opened for editing
//! (a save migration renames, splits and adds before the world takes it)
//! and selections (the tagged entities and chosen resources, restored into
//! a running world without replacing the rest).
//!
//! Ways it could go wrong, and the test that catches each:
//! - an edit that changed nothing does not write the snapshot it read
//!   (a migration of one block would then perturb the rest) →
//!   `an_edit_that_changes_nothing_writes_the_snapshot_it_read`
//! - a renamed type's rows land in another archetype or order, so the
//!   migrated world is not the one a new build makes →
//!   `a_type_renamed_in_an_edit_restores_as_the_world_built_with_its_new_name`
//! - splitting a type loses values, or moves the other entities →
//!   `a_type_split_in_an_edit_keeps_every_value_and_the_other_entities`
//! - an entity added to a save takes an id a spawn would not →
//!   `an_entity_added_to_an_edit_takes_the_id_a_spawn_would`
//! - a selection replaces untagged entities, keeps stale tagged ones, loses
//!   names, leaves references at the old ids, or a refused one changes the
//!   world → `a_selection_replaces_only_the_tagged_entities_and_its_resources`

use og_core::ErrorKind;
use og_ecs::{Entity, QueryDesc, Reflect, TypeValues, World, WorldDesc};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Pos")]
struct Pos {
    x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Gold")]
struct Gold {
    amount: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Coins")]
struct Coins {
    amount: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Stats")]
struct Stats {
    hp: f32,
    mp: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Health")]
struct Health {
    hp: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Mana")]
struct Mana {
    mp: f32,
}

/// A tag: the entities a game keeps across sessions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "Keep")]
struct Keep {
    /// Another entity's id (its `to_bits`), or 0.
    owner: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Progress")]
struct Progress {
    level: u32,
}

fn world() -> World {
    World::new(&WorldDesc {
        seed: 3,
        tick_rate: 60,
    })
    .unwrap()
}

/// Pos, then `G` (Gold or Coins, the same place in the type table), with
/// entities spawned, one despawned, one named, an event and some ticks.
fn wallet_world<G: og_core::Reflect + Copy>(gold: impl Fn(u32) -> G) -> (World, Vec<Entity>) {
    let mut w = world();
    w.register::<Pos>().unwrap();
    w.register::<G>().unwrap();
    w.register::<Stats>().unwrap();
    let mut es = Vec::new();
    for i in 0..5u32 {
        let e = w.spawn((Pos { x: i as f32 },)).unwrap();
        if i % 2 == 0 {
            w.insert(e, gold(i * 10)).unwrap();
        }
        es.push(e);
    }
    w.despawn(es[3]).unwrap();
    w.set_name(es[4], "hero").unwrap();
    w.send(Pos { x: 9.0 }).unwrap();
    for _ in 0..3 {
        w.step().unwrap();
    }
    (w, es)
}

#[test]
fn a_type_renamed_in_an_edit_restores_as_the_world_built_with_its_new_name() {
    let (old, es) = wallet_world(|amount| Gold { amount });
    let (built, _) = wallet_world(|amount| Coins { amount });
    let mut edit = old.edit_snapshot(&old.snapshot()).unwrap();
    let values = TypeValues {
        schema: Coins::schema(),
        components: edit.components("Gold"),
        resource: None,
    };
    edit.replace_types(&["Gold"], vec![values]).unwrap();

    let mut w = world();
    w.register::<Pos>().unwrap();
    w.register::<Coins>().unwrap();
    w.register::<Stats>().unwrap();
    // The save as written is refused: it names Gold, which this build lacks.
    assert_eq!(
        w.restore(&old.snapshot()).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    w.restore(&edit.to_bytes()).unwrap();
    assert_eq!(w.get::<Coins>(es[2]).unwrap().amount, 20);
    assert_eq!(w.find("hero"), Some(es[4]));
    assert_eq!(w.world_hash(), built.world_hash());
    // And the two go on alike.
    let mut built = built;
    w.step().unwrap();
    built.step().unwrap();
    assert_eq!(w.world_hash(), built.world_hash());
}

#[test]
fn a_type_split_in_an_edit_keeps_every_value_and_the_other_entities() {
    let mut old = world();
    old.register::<Pos>().unwrap();
    old.register::<Stats>().unwrap();
    let a = old
        .spawn((Pos { x: 1.0 }, Stats { hp: 7.0, mp: 2.0 }))
        .unwrap();
    let b = old.spawn((Pos { x: 2.0 },)).unwrap();
    let c = old
        .spawn((Pos { x: 3.0 }, Stats { hp: 5.0, mp: 9.0 }))
        .unwrap();
    old.set_name(c, "mage").unwrap();
    let mut edit = old.edit_snapshot(&old.snapshot()).unwrap();
    let stats = edit.components("Stats");
    let half = |at: usize| -> Vec<(Entity, Vec<u8>)> {
        stats
            .iter()
            .map(|(e, v)| (*e, v[at..at + 4].to_vec()))
            .collect()
    };
    let (health, mana) = (half(0), half(4));
    edit.replace_types(
        &["Stats"],
        vec![
            TypeValues {
                schema: Health::schema(),
                components: health,
                resource: None,
            },
            TypeValues {
                schema: Mana::schema(),
                components: mana,
                resource: None,
            },
        ],
    )
    .unwrap();
    let mut w = world();
    w.register::<Pos>().unwrap();
    w.register::<Health>().unwrap();
    w.register::<Mana>().unwrap();
    w.restore(&edit.to_bytes()).unwrap();
    assert_eq!(w.get::<Health>(a).unwrap().hp, 7.0);
    assert_eq!(w.get::<Mana>(c).unwrap().mp, 9.0);
    assert!(!w.has::<Health>(b).unwrap());
    assert_eq!(w.get::<Pos>(b).unwrap().x, 2.0);
    assert_eq!(w.find("mage"), Some(c));
    let order: Vec<Entity> = w
        .query(&QueryDesc::new().read::<Health>())
        .unwrap()
        .flat_map(|q| q.entities().to_vec())
        .collect();
    assert_eq!(order, [a, c]);
    // A component of an entity the save does not hold is refused, and the
    // edit is left as it was.
    let before = edit.to_bytes();
    let err = edit
        .replace_types(
            &[],
            vec![TypeValues {
                schema: Gold::schema(),
                components: vec![(Entity::from_bits(99 | 1 << 32), vec![0; 4])],
                resource: None,
            }],
        )
        .unwrap_err();
    assert!(err.reason().contains("does not hold"), "{err}");
    assert_eq!(edit.to_bytes(), before);
}

#[test]
fn a_selection_replaces_only_the_tagged_entities_and_its_resources() {
    let mut w = world();
    w.register::<Pos>().unwrap();
    w.register::<Keep>().unwrap();
    w.register::<Progress>().unwrap();
    w.insert_resource(Progress { level: 4 }).unwrap();
    let scenery = w.spawn((Pos { x: 100.0 },)).unwrap();
    let hero = w.spawn((Pos { x: 1.0 }, Keep::default())).unwrap();
    let pet = w
        .spawn((
            Pos { x: 2.0 },
            Keep {
                owner: hero.to_bits(),
            },
        ))
        .unwrap();
    w.set_name(hero, "hero").unwrap();
    let keep = w.id::<Keep>().unwrap();
    let progress = w.id::<Progress>().unwrap();
    let save = w.snapshot_selection(&[keep], &[progress]).unwrap();
    // Smaller than the whole world: only the types it uses.
    assert!(save.len() < w.snapshot().len());
    let edit = w.edit_snapshot(&save).unwrap();
    assert!(edit.is_selection());
    assert_eq!(edit.filter(), ["Keep"]);
    assert_eq!(edit.to_bytes(), save);

    // The session goes on: the hero moves, a third kept entity appears,
    // the level changes, the scenery moves.
    w.insert(hero, Pos { x: 50.0 }).unwrap();
    w.spawn((Pos { x: 3.0 }, Keep::default())).unwrap();
    w.insert_resource(Progress { level: 9 }).unwrap();
    w.insert(scenery, Pos { x: 101.0 }).unwrap();

    // A whole restore of a selection is refused.
    assert!(w.restore(&save).is_err());
    let no_fields = |_: &str| Vec::new();
    let owner = |t: &str| if t == "Keep" { vec![0] } else { vec![] };
    let report = w
        .restore_selection(&save, false, &|_| false, &|_| false, &owner)
        .unwrap();
    assert_eq!(report.despawned, 3);
    assert_eq!(report.spawned.len(), 2);
    assert_eq!(report.resources, ["Progress"]);
    let kept: Vec<Entity> = w
        .query(&QueryDesc::new().read::<Keep>())
        .unwrap()
        .flat_map(|q| q.entities().to_vec())
        .collect();
    assert_eq!(kept.len(), 2);
    let new_hero = w.find("hero").unwrap();
    assert_ne!(new_hero, hero, "spawned anew");
    assert_eq!(w.get::<Pos>(new_hero).unwrap().x, 1.0);
    let new_pet = report.spawned[1].1;
    assert_eq!(report.spawned[1].0, pet);
    assert_eq!(
        w.get::<Keep>(new_pet).unwrap().owner,
        new_hero.to_bits(),
        "a reference to a saved entity follows it"
    );
    assert_eq!(w.resource::<Progress>().unwrap().level, 4);
    assert_eq!(
        w.get::<Pos>(scenery).unwrap().x,
        101.0,
        "untagged: untouched"
    );

    // A name an entity it does not replace holds refuses it, unchanged.
    let other = w.spawn((Pos { x: 0.0 },)).unwrap();
    w.clear_name(new_hero).unwrap();
    w.set_name(other, "hero").unwrap();
    let before = w.world_hash();
    let err = w
        .restore_selection(&save, false, &|_| false, &|_| false, &no_fields)
        .unwrap_err();
    assert!(err.reason().contains("'hero'"), "{err}");
    assert_eq!(w.world_hash(), before);
}
