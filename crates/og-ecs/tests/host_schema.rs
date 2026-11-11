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
    assert_eq!(names, ["og.Time", "og.Rng", "og.TimeScale", "Velocity"]);
    // Engine-owned names are dotted, the same as the TypeScript tokens.
    assert_eq!(w.id_by_name("og.Time").unwrap(), w.id::<Time>().unwrap());
    assert_eq!(w.id_by_name("og.Rng").unwrap(), w.id::<Rng>().unwrap());
    assert_eq!(
        w.id_by_name("Time").unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(types[3]["fields"][1]["tunable"], true);
    assert_eq!(types[3]["fields"][1]["default"][0], 0.1);
    assert_eq!(types[0]["fields"][1]["unit"], "s");
}

/// The Rust twin of the host layout.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Velocity {
    v: Vec3,
    drag: f32,
}

/// A component's columns together are its rows' packed values: row `i` of
/// each, in field order, is `get_bytes` of entity `i`.
#[test]
fn a_components_columns_hold_each_rows_packed_bytes_in_field_order() {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let vel = w
        .register_schema(Schema::from_json(VELOCITY).unwrap())
        .unwrap();
    let mass = w
        .register_schema(
            Schema::from_json(
                r#"{"name":"Mass","version":1,"fields":[{"name":"kg","type":"f32"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
    for (v, drag) in [([1.0, 2.0, 3.0], 0.5), ([4.0, 5.0, 6.0], 0.25)] {
        w.spawn_bytes(vec![
            (vel, packed(v, drag)),
            (mass, 7.0f32.to_le_bytes().to_vec()),
        ])
        .unwrap();
    }
    let mut q = QueryDesc::new();
    q.write.push("Velocity".into());
    let mut seen = 0;
    for view in w.query(&q).unwrap() {
        let columns = view.component_bytes(vel).unwrap();
        assert_eq!(columns.len(), 2, "one column a field");
        for (row, &e) in view.entities().iter().enumerate() {
            let mut bytes = Vec::new();
            for c in &columns {
                let size = c.len() / view.len();
                bytes.extend_from_slice(&c[row * size..(row + 1) * size]);
            }
            assert_eq!(bytes, w.get_bytes(e, vel).unwrap(), "row {row}");
            seen += 1;
        }
        drop(columns);
        // A component the query did not declare is refused, and so is one
        // a column of which is held for writing.
        assert_eq!(
            view.component_bytes(mass).unwrap_err().kind(),
            ErrorKind::Access
        );
        let held = view
            .write_bytes(w.field_by_id(vel, "drag").unwrap())
            .unwrap();
        let busy = view.component_bytes(vel).unwrap_err();
        assert!(busy.to_string().contains("still held"), "{busy}");
        drop(held);
        assert!(view.component_bytes(vel).is_ok());
    }
    assert_eq!(seen, 2);
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
