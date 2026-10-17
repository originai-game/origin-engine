//! `#[derive(Reflect)]` produces the schema the inspector, Tune, the probe
//! and the wiki read, and moves values through packed bytes.

use og_core::math::{Quat, Vec3};
use og_core::reflect::{from_bytes, schemas_to_json, to_bytes, Layout};
use og_core::{ErrorKind, FieldSchema, FieldType, Reflect, Schema};

/// Hit points of anything that can be destroyed.
///
/// Second paragraph.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Health", version = 2)]
struct Life {
    /// Current hit points.
    #[reflect(
        min = 0,
        max = 1000.0,
        label = "Hit points",
        unit = "hp",
        tunable,
        default = 100
    )]
    hp: f32,
    /// Regeneration per second.
    #[reflect(min = -5.5, max = 5.5, unit = "hp/s")]
    regen: f64,
    team: u32,
    seed: u64,
    offset: i32,
    big: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Transform {
    #[reflect(default = [0.0, 1.0, -2.5])]
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

/// A tag.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Player;

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Empty {}

#[test]
fn derive_produces_the_schema_with_metadata() {
    let s = Life::schema();
    assert_eq!(Life::NAME, "Health");
    assert_eq!(s.name, "Health");
    assert_eq!(s.version, 2);
    assert_eq!(
        s.doc,
        "Hit points of anything that can be destroyed.\n\nSecond paragraph."
    );
    let hp = &s.fields[0];
    assert_eq!(
        hp,
        &FieldSchema {
            name: "hp".into(),
            ty: FieldType::F32,
            doc: "Current hit points.".into(),
            label: Some("Hit points".into()),
            unit: Some("hp".into()),
            min: Some(0.0),
            max: Some(1000.0),
            tunable: true,
            default: Some(vec![100.0]),
        }
    );
    assert_eq!((s.fields[1].min, s.fields[1].max), (Some(-5.5), Some(5.5)));
    let types: Vec<_> = s.fields.iter().map(|f| f.ty).collect();
    assert_eq!(
        types,
        [
            FieldType::F32,
            FieldType::F64,
            FieldType::U32,
            FieldType::U64,
            FieldType::I32,
            FieldType::I64
        ]
    );
    assert!(!s.fields[2].tunable && s.fields[2].doc.is_empty());
    s.validate().unwrap();

    let t = Transform::schema();
    assert_eq!(Transform::NAME, "Transform");
    assert_eq!(t.version, 1);
    assert_eq!(t.fields[0].default, Some(vec![0.0, 1.0, -2.5]));
    assert_eq!(t.fields[1].ty, FieldType::Quat);
    assert_eq!(Player::schema().fields.len(), 0);
    assert_eq!(Player::schema().doc, "A tag.");
    assert_eq!(Empty::schema().fields.len(), 0);
}

#[test]
fn schema_json_is_exported_and_reads_back() {
    let json = schemas_to_json(&[Life::schema(), Transform::schema(), Player::schema()]);
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(doc["format"], "og-schema");
    assert_eq!(doc["types"][0]["name"], "Health");
    assert_eq!(doc["types"][0]["fields"][0]["type"], "f32");
    assert_eq!(doc["types"][0]["fields"][0]["tunable"], true);
    assert_eq!(doc["types"][0]["fields"][0]["unit"], "hp");
    assert_eq!(doc["types"][1]["fields"][1]["type"], "quat");
    assert!(doc["types"][0]["fields"][2].get("min").is_none());

    let one = Life::schema().to_json();
    assert_eq!(Schema::from_json(&one).unwrap(), Life::schema());
    // A host declares a layout by JSON with only the required keys.
    let host = Schema::from_json(
        r#"{"name":"Velocity","version":1,"fields":[{"name":"v","type":"vec3"}]}"#,
    )
    .unwrap();
    assert_eq!(host.fields[0], FieldSchema::new("v", FieldType::Vec3));
}

#[test]
fn invalid_schemas_are_refused_with_a_reason() {
    let base = || Schema {
        name: "Thing".into(),
        version: 1,
        doc: String::new(),
        fields: vec![FieldSchema::new("a", FieldType::F32)],
    };
    let refuse = |s: Schema, needle: &str| {
        let err = s.validate().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Schema);
        assert!(err.reason().contains(needle), "{}", err.reason());
    };
    let mut s = base();
    s.name = "9lives".into();
    refuse(s, "not an identifier");
    let mut s = base();
    s.version = 0;
    refuse(s, "version 0");
    let mut s = base();
    s.fields.push(FieldSchema::new("a", FieldType::U32));
    refuse(s, "declared twice");
    let mut s = base();
    s.fields[0].min = Some(2.0);
    s.fields[0].max = Some(1.0);
    refuse(s, "above max");
    let mut s = base();
    s.fields[0].ty = FieldType::Vec3;
    s.fields[0].default = Some(vec![1.0]);
    refuse(s, "needs 3 default");
    let mut s = base();
    s.fields[0].max = Some(1.0);
    s.fields[0].default = Some(vec![2.0]);
    refuse(s, "outside its range");
    assert_eq!(Schema::from_json("{").unwrap_err().kind(), ErrorKind::Parse);
    let mut namespaced = base();
    namespaced.name = "og.Transform".into();
    namespaced.validate().unwrap();
}

#[test]
fn layout_hash_tracks_bytes_not_metadata() {
    let a = Life::schema();
    let mut b = a.clone();
    b.fields[0].doc = "changed".into();
    b.fields[0].max = Some(5.0);
    b.fields[0].tunable = false;
    assert_eq!(a.layout_hash(), b.layout_hash());
    let mut c = a.clone();
    c.fields[0].ty = FieldType::U32;
    assert_ne!(a.layout_hash(), c.layout_hash());
    let mut d = a.clone();
    d.version = 3;
    assert_ne!(a.layout_hash(), d.layout_hash());
    let mut e = a.clone();
    e.fields.swap(0, 1);
    assert_ne!(a.layout_hash(), e.layout_hash());
}

#[test]
fn values_round_trip_through_packed_bytes() {
    let life = Life {
        hp: 42.5,
        regen: -1.25,
        team: 7,
        seed: u64::MAX - 3,
        offset: -9,
        big: i64::MIN + 1,
    };
    let bytes = to_bytes(&life).unwrap();
    assert_eq!(bytes.len(), 4 + 8 + 4 + 8 + 4 + 8);
    assert_eq!(&bytes[0..4], &42.5f32.to_le_bytes());
    assert_eq!(&bytes[4..12], &(-1.25f64).to_le_bytes());
    assert_eq!(from_bytes::<Life>(&bytes).unwrap(), life);

    let t = Transform {
        position: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_xyzw(0.0, 0.6, 0.0, 0.8),
        scale: Vec3::ONE,
    };
    let layout = Layout::of(&Transform::schema());
    assert_eq!(layout.size(), 12 + 16 + 12);
    assert_eq!(layout.range(1), 12..28);
    let bytes = layout.encode(&t).unwrap();
    assert_eq!(layout.decode::<Transform>(&bytes).unwrap(), t);
    assert_eq!(
        layout.decode::<Transform>(&bytes[1..]).unwrap_err().kind(),
        ErrorKind::Invalid
    );
    assert!(to_bytes(&Player).unwrap().is_empty());
}
