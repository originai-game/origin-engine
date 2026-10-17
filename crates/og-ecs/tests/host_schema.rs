//! A host (the TypeScript `defineComponent`) registers a layout by schema,
//! by name, and moves values as packed bytes; Rust code over the same
//! layout sees the same columns.

use og_core::math::Vec3;
use og_core::{ErrorKind, Rng, Schema, Time};
use og_ecs::{QueryDesc, Reflect, SystemDesc, World, WorldDesc};

const VELOCITY: &str = r#"{
  "name": "Velocity", "version": 1, "doc": "Linear velocity.",
  "fields": [
    {"name": "v", "type": "vec3", "unit": "m/s"},
    {"name": "drag", "type": "f32", "min": 0, "max": 1, "tunable": true, "default": [0.1]}
  ]
}"#;

fn packed(v: [f32; 3], drag: f32) -> Vec<u8> {
    v.iter()
        .chain([drag].iter())
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

#[test]
fn a_host_layout_is_registered_by_name_and_used_as_bytes() {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let vel = w
        .register_schema(Schema::from_json(VELOCITY).unwrap())
        .unwrap();
    assert_eq!(w.id_by_name("Velocity").unwrap(), vel);
    // Registering the same layout again is a no-op; another layout is refused.
    assert_eq!(
        w.register_schema(Schema::from_json(VELOCITY).unwrap())
            .unwrap(),
        vel
    );
    let other = Schema::from_json(
        r#"{"name":"Velocity","version":1,"fields":[{"name":"v","type":"vec2"}]}"#,
    )
    .unwrap();
    assert_eq!(
        w.register_schema(other).unwrap_err().kind(),
        ErrorKind::Schema
    );

    let a = w
        .spawn_bytes(vec![(vel, packed([1.0, 2.0, 3.0], 0.5))])
        .unwrap();
    let b = w
        .spawn_bytes(vec![(vel, packed([4.0, 5.0, 6.0], 0.25))])
        .unwrap();
    assert_eq!(w.get_bytes(b, vel).unwrap(), packed([4.0, 5.0, 6.0], 0.25));
    assert_eq!(
        w.spawn_bytes(vec![(vel, vec![0; 3])]).unwrap_err().kind(),
        ErrorKind::Invalid
    );

    // Column views over the host layout, by field name.
    let v = w.field_by_id(vel, "v").unwrap();
    let drag = w.field_by_id(vel, "drag").unwrap();
    let mut q = QueryDesc::new();
    q.write.push("Velocity".into());
    for view in w.query(&q).unwrap() {
        assert_eq!(view.entities(), [a, b]);
        assert_eq!(
            *view.read::<Vec3>(v).unwrap(),
            [Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]
        );
        // The same column as raw scalars, as a Float32Array view would see it.
        let raw = view.read_bytes(v).unwrap();
        let floats: &[f32] = bytemuck::cast_slice(&raw);
        assert_eq!(floats, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        drop(raw);
        view.write::<f32>(drag).unwrap()[1] = 0.75;
        assert_eq!(view.read::<f32>(v).unwrap_err().kind(), ErrorKind::Invalid);
    }
    assert_eq!(w.get_bytes(b, vel).unwrap(), packed([4.0, 5.0, 6.0], 0.75));

    // The schema JSON export carries the host's metadata.
    let json: serde_json::Value = serde_json::from_str(&w.schemas_json()).unwrap();
    let types = json["types"].as_array().unwrap();
    let names: Vec<_> = types.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["og.Time", "og.Rng", "Velocity"]);
    // Engine-owned names are dotted, the same as the TypeScript tokens.
    assert_eq!(w.id_by_name("og.Time").unwrap(), w.id::<Time>().unwrap());
    assert_eq!(w.id_by_name("og.Rng").unwrap(), w.id::<Rng>().unwrap());
    assert_eq!(
        w.id_by_name("Time").unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(types[2]["fields"][1]["tunable"], true);
    assert_eq!(types[2]["fields"][1]["default"][0], 0.1);
    assert_eq!(types[0]["fields"][1]["unit"], "s");
}

/// The Rust twin of the host layout.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Velocity {
    v: Vec3,
    drag: f32,
}

#[test]
fn a_rust_type_adopts_a_host_layout_of_the_same_shape() {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let host = w
        .register_schema(Schema::from_json(VELOCITY).unwrap())
        .unwrap();
    let e = w
        .spawn_bytes(vec![(host, packed([1.0, 0.0, 0.0], 0.5))])
        .unwrap();
    w.insert_resource_bytes(host, &packed([0.0, 9.0, 0.0], 0.0))
        .unwrap();
    assert_eq!(w.register::<Velocity>().unwrap(), host);
    assert_eq!(
        w.get::<Velocity>(e).unwrap(),
        Velocity {
            v: Vec3::X,
            drag: 0.5
        }
    );
    // The raw resource became a typed one.
    assert_eq!(
        w.resource::<Velocity>().unwrap().v,
        Vec3::new(0.0, 9.0, 0.0)
    );
    let v = w.field::<Velocity>("v").unwrap();
    w.add_system(
        SystemDesc::new("push")
            .write::<Velocity>()
            .query(QueryDesc::new().write::<Velocity>()),
        move |ctx| {
            for a in ctx.query(0)? {
                for x in a.write::<Vec3>(v)?.iter_mut() {
                    x.x += 1.0;
                }
            }
            Ok(())
        },
    )
    .unwrap();
    w.step().unwrap();
    assert_eq!(w.get_bytes(e, host).unwrap(), packed([2.0, 0.0, 0.0], 0.5));
}

#[test]
fn a_world_can_move_to_the_engine_thread() {
    fn send<T: Send>(_: &T) {}
    let w = World::new(&WorldDesc::default()).unwrap();
    send(&w);
    let h = std::thread::spawn(move || w.world_hash()).join().unwrap();
    assert_eq!(h, World::new(&WorldDesc::default()).unwrap().world_hash());
}
