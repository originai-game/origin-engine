//! The M6 field types in a world: a bool column is one byte per entity (a
//! `Uint8Array` on the web) and round-trips through snapshot and hash; a
//! mat4 column is sixteen f32 per entity; string fields ride on events only.

use og_core::math::Mat4;
use og_core::{Bool8, ErrorKind, Schema, Str64};
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};

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

#[test]
fn a_mat4_column_is_sixteen_floats_per_entity() {
    let mut w = world();
    let m = |k: f32| Mat4::from_cols_array(&std::array::from_fn(|i| k * 100.0 + i as f32));
    let a = w.spawn((Pose { world: m(1.0) },)).unwrap();
    w.spawn((Pose { world: m(2.0) },)).unwrap();
    let field = w.field::<Pose>("world").unwrap();
    let view = w
        .query(&QueryDesc::new().read::<Pose>())
        .unwrap()
        .next()
        .unwrap();
    let floats: Vec<f32> = view
        .read_bytes(field)
        .unwrap()
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    assert_eq!(floats.len(), 32);
    assert_eq!(
        &floats[14..18],
        &[114.0, 115.0, 200.0, 201.0],
        "stride 16, column-major"
    );
    assert_eq!(view.read::<Mat4>(field).unwrap()[1], m(2.0));
    drop(view);
    let bytes = w.snapshot();
    let mut b = world();
    b.restore(&bytes).unwrap();
    assert_eq!(b.get::<Pose>(a).unwrap().world, m(1.0));
}

#[test]
fn strings_ride_on_events_only() {
    let mut w = world();
    let said = w.id::<Said>().unwrap();
    let e = w.spawn(()).unwrap();
    let hello = Said {
        text: Str64::new("hello").unwrap(),
        mood: 1,
    };
    for err in [
        w.spawn((hello,)).unwrap_err(),
        w.insert(e, hello).unwrap_err(),
        w.insert_resource(hello).unwrap_err(),
    ] {
        assert_eq!(err.kind(), ErrorKind::Schema);
        assert!(err.reason().contains("only events carry"), "{err}");
    }

    w.send(hello).unwrap();
    let mut raw = vec![0u8; 68];
    raw[..3].copy_from_slice(b"hey");
    w.send_bytes(said, &raw).unwrap();
    let mut bad_enum = raw.clone();
    bad_enum[64] = 2;
    assert!(w
        .send_bytes(said, &bad_enum)
        .unwrap_err()
        .reason()
        .contains("enum index 2"));
    let mut bad_text = raw.clone();
    bad_text[5] = b'!';
    assert!(w
        .send_bytes(said, &bad_text)
        .unwrap_err()
        .reason()
        .contains("followed by non-zero bytes"));

    // Events are copied, so they round-trip through a snapshot and are read
    // the next tick.
    let mut b = world();
    b.restore(&w.snapshot()).unwrap();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = seen.clone();
    b.add_system(SystemDesc::new("listen").read::<Said>(), move |ctx| {
        for s in ctx.events::<Said>()? {
            sink.lock()
                .unwrap()
                .push((s.text.as_str().to_owned(), s.mood));
        }
        Ok(())
    })
    .unwrap();
    b.step().unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        [("hello".to_owned(), 1), ("hey".to_owned(), 0)]
    );
}
