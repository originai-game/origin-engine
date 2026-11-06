//! The M6 field types in a world: a bool column is one byte per entity (a
//! `Uint8Array` on the web) and round-trips through snapshot and hash; a
//! mat4 column is sixteen f32 per entity; a string field is plain data in
//! events, components and resources alike, and migrates across capacities.

use og_core::math::Mat4;
use og_core::{Bool8, ErrorKind, FieldSchema, FieldType, Schema, Str64};
use og_ecs::{QueryDesc, Reflect, World, WorldDesc};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Lamp {
    on: bool,
    level: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Pose {
    world: Mat4,
}

/// An event with a string and an enum.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Said {
    text: Str64,
    #[reflect(variants = ["calm", "angry"])]
    mood: u32,
}

const FLAGS: &str = r#"{"name":"Flags","version":1,"fields":[
  {"name":"visible","type":"bool","default":[1]},
  {"name":"team","type":"u32"}]}"#;

fn world() -> World {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    w.register::<Lamp>().unwrap();
    w.register::<Pose>().unwrap();
    w.register::<Said>().unwrap();
    w.register_schema(Schema::from_json(FLAGS).unwrap())
        .unwrap();
    w
}

fn flags(visible: u8, team: u32) -> Vec<u8> {
    let mut b = vec![visible];
    b.extend(team.to_le_bytes());
    b
}

#[test]
fn a_bool_column_round_trips_through_snapshot_and_hash() {
    let mut a = world();
    let flags_id = a.id_by_name("Flags").unwrap();
    let mut lamps = Vec::new();
    for i in 0..5u32 {
        let e = a
            .spawn((Lamp {
                on: i % 2 == 0,
                level: i as f32,
            },))
            .unwrap();
        a.insert_bytes(e, flags_id, &flags(u8::from(i != 3), i))
            .unwrap();
        lamps.push(e);
    }

    // One byte per entity, viewable as Bool8 or as raw bytes.
    let on = a.field::<Lamp>("on").unwrap();
    let visible = a.field_by_id(flags_id, "visible").unwrap();
    let q = QueryDesc {
        read: vec!["Lamp".into(), "Flags".into()],
        ..QueryDesc::default()
    };
    let view = a.query(&q).unwrap().next().unwrap();
    assert_eq!(&*view.read_bytes(on).unwrap(), &[1, 0, 1, 0, 1]);
    assert_eq!(&*view.read_bytes(visible).unwrap(), &[1, 1, 1, 0, 1]);
    let typed: Vec<bool> = view
        .read::<Bool8>(on)
        .unwrap()
        .iter()
        .map(|b| b.get())
        .collect();
    assert_eq!(typed, [true, false, true, false, true]);
    assert_eq!(
        view.read::<u32>(on).unwrap_err().kind(),
        ErrorKind::Invalid,
        "a bool column is not a u32 column"
    );
    drop(view);

    let bytes = a.snapshot();
    let hash = a.world_hash();
    let mut b = world();
    b.restore(&bytes).unwrap();
    assert_eq!(b.world_hash(), hash);
    assert_eq!(b.snapshot(), bytes);
    assert_eq!(
        b.get::<Lamp>(lamps[2]).unwrap(),
        Lamp {
            on: true,
            level: 2.0
        }
    );
    assert_eq!(b.get_bytes(lamps[3], flags_id).unwrap(), flags(0, 3));

    // The hash sees one flipped bool, and flipping it back restores it.
    let flip = |w: &World| {
        let mut desc = QueryDesc::new();
        desc.write.push("Lamp".into());
        let view = w.query(&desc).unwrap().next().unwrap();
        let mut col = view.write::<Bool8>(on).unwrap();
        let v = !col[1].get();
        col[1].set(v);
    };
    flip(&b);
    assert_ne!(b.world_hash(), hash);
    assert!(b.get::<Lamp>(lamps[1]).unwrap().on);
    flip(&b);
    assert_eq!(b.world_hash(), hash);

    // The byte entry points take only 0 and 1.
    let err = b
        .insert_bytes(lamps[0], flags_id, &flags(2, 0))
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.reason().contains("Flags.visible: bool byte 2"), "{err}");
    assert_eq!(b.world_hash(), hash, "a refused write changes nothing");
}

/// A post chain: handles in a list, and a weight.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Chain {
    passes: og_core::List<u64, 4>,
    weight: f32,
}

fn chain(passes: &[u64], weight: f32) -> Chain {
    Chain {
        passes: og_core::List::from_slice(passes).unwrap(),
        weight,
    }
}

#[test]
fn a_list_column_round_trips_and_migrates_to_another_capacity() {
    let make = || {
        let mut w = World::new(&WorldDesc::default()).unwrap();
        w.register::<Chain>().unwrap();
        let empty = w.spawn((chain(&[], 1.0),)).unwrap();
        let zero = w.spawn((chain(&[0], 1.0),)).unwrap();
        let two = w.spawn((chain(&[7, 9], 0.5),)).unwrap();
        let full = w.spawn((chain(&[1, 2, 3, 4], 2.0),)).unwrap();
        (w, [empty, zero, two, full])
    };
    let ((a, [empty, zero, two, full]), (b, _)) = (make(), make());
    assert_eq!(a.world_hash(), b.world_hash());

    // `[]` and `[0]` are different values and hash differently.
    let (mut c, _) = make();
    c.insert(zero, chain(&[], 1.0)).unwrap();
    assert_ne!(c.world_hash(), a.world_hash(), "[0] is not []");

    // The same capacity: restored byte for byte.
    let snapshot = a.snapshot();
    let mut same = World::new(&WorldDesc::default()).unwrap();
    same.register::<Chain>().unwrap();
    same.restore(&snapshot).unwrap();
    assert_eq!(same.world_hash(), a.world_hash());
    assert_eq!(same.get::<Chain>(two).unwrap(), chain(&[7, 9], 0.5));
    assert_eq!(same.get::<Chain>(zero).unwrap(), chain(&[0], 1.0));

    // A narrower capacity keeps what fits and starts a longer list empty,
    // never truncated; the field beside it is untouched.
    let narrow = |capacity: usize| {
        let mut w = World::new(&WorldDesc::default()).unwrap();
        w.register_schema(Schema {
            name: "Chain".into(),
            version: 1,
            doc: String::new(),
            fields: vec![
                FieldSchema::new(
                    "passes",
                    FieldType::list(og_core::Scalar::U64, capacity).unwrap(),
                ),
                FieldSchema::new("weight", FieldType::F32),
            ],
        })
        .unwrap();
        w
    };
    let mut d = narrow(2);
    let report = d.restore_migrating(&snapshot).unwrap();
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("Chain.passes changed from u64[4] to u64[2]")),
        "{report:?}"
    );
    let id = d.id_by_name("Chain").unwrap();
    let packed = |passes: &[u64], capacity: usize, weight: f32| {
        let mut b = (passes.len() as u64).to_le_bytes().to_vec();
        for p in passes {
            b.extend(p.to_le_bytes());
        }
        b.resize(8 * (capacity + 1), 0);
        b.extend(weight.to_le_bytes());
        b
    };
    assert_eq!(d.get_bytes(empty, id).unwrap(), packed(&[], 2, 1.0));
    assert_eq!(d.get_bytes(zero, id).unwrap(), packed(&[0], 2, 1.0));
    assert_eq!(d.get_bytes(two, id).unwrap(), packed(&[7, 9], 2, 0.5));
    assert_eq!(
        d.get_bytes(full, id).unwrap(),
        packed(&[], 2, 2.0),
        "four passes do not fit two: the list starts empty"
    );

    // A wider one keeps every list, zero-padded, and restores back.
    let mut e = narrow(6);
    e.restore_migrating(&snapshot).unwrap();
    let id = e.id_by_name("Chain").unwrap();
    assert_eq!(
        e.get_bytes(full, id).unwrap(),
        packed(&[1, 2, 3, 4], 6, 2.0)
    );
    let mut back = World::new(&WorldDesc::default()).unwrap();
    back.register::<Chain>().unwrap();
    back.restore_migrating(&e.snapshot()).unwrap();
    assert_eq!(back.get::<Chain>(full).unwrap(), chain(&[1, 2, 3, 4], 2.0));
    assert_eq!(back.get::<Chain>(zero).unwrap(), chain(&[0], 1.0));
}
