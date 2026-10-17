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
            variants: Vec::new(),
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

/// The M6 field types: bool, mat4, enum and the event-only strings.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Said")]
struct Said {
    #[reflect(name = "isLoud", default = 1)]
    is_loud: bool,
    world: og_core::math::Mat4,
    /// The speaker's mood.
    #[reflect(variants = ["calm", "angry", "sad"], default = 2)]
    mood: u32,
    text: og_core::Str64,
    long: og_core::Str256,
    flag: og_core::Bool8,
}

#[test]
fn bool_mat4_enum_and_string_fields_have_their_layout() {
    let s = Said::schema();
    s.validate().unwrap();
    let types: Vec<_> = s.fields.iter().map(|f| (f.name.as_str(), f.ty)).collect();
    assert_eq!(
        types,
        [
            ("isLoud", FieldType::Bool),
            ("world", FieldType::Mat4),
            ("mood", FieldType::Enum),
            ("text", FieldType::Str64),
            ("long", FieldType::Str256),
            ("flag", FieldType::Bool),
        ]
    );
    assert_eq!(s.fields[2].variants, ["calm", "angry", "sad"]);
    assert!(s.has_strings());
    let layout = Layout::of(&s);
    assert_eq!(layout.size(), 1 + 64 + 4 + 64 + 256 + 1);

    let value = Said {
        is_loud: true,
        world: og_core::math::Mat4::from_cols_array(&std::array::from_fn(|i| i as f32)),
        mood: 1,
        text: og_core::Str64::new("héllo").unwrap(),
        long: og_core::Str256::new(&"x".repeat(256)).unwrap(),
        flag: og_core::Bool8::TRUE,
    };
    let bytes = to_bytes(&value).unwrap();
    assert_eq!(bytes[0], 1, "a bool is one byte, 1 for true");
    assert_eq!(
        &bytes[1 + 4 * 5..1 + 4 * 6],
        &5f32.to_le_bytes(),
        "mat4 is column-major f32"
    );
    assert_eq!(
        &bytes[65..69],
        &1u32.to_le_bytes(),
        "an enum is its u32 index"
    );
    assert_eq!(&bytes[69..75], "héllo".as_bytes());
    assert!(bytes[75..133].iter().all(|b| *b == 0), "zero-padded");
    assert_eq!(from_bytes::<Said>(&bytes).unwrap(), value);
    s.check_packed(&bytes).unwrap();
    assert_eq!(from_bytes::<Said>(&bytes).unwrap().text.as_str(), "héllo");

    // Byte checks name the field.
    let reject = |at: usize, v: u8, needle: &str| {
        let mut b = bytes.clone();
        b[at] = v;
        let err = s.check_packed(&b).unwrap_err();
        assert!(err.reason().contains(needle), "{}", err.reason());
    };
    reject(0, 2, "og.Said.isLoud: bool byte 2");
    reject(65, 3, "og.Said.mood: enum index 3");
    reject(
        76,
        b'z',
        "og.Said.text: a string is followed by non-zero bytes",
    );
    reject(69, 0xff, "og.Said.text: a string that is not UTF-8");

    // Defaults: bool 1, enum index 2, strings empty.
    let d = s.default_bytes();
    assert_eq!((d[0], &d[65..69]), (1, &2u32.to_le_bytes()[..]));
    assert!(og_core::Str64::new(&"y".repeat(65)).is_err());
    assert!(og_core::Str64::new("a\0b").is_err());

    // JSON: lower-case type names and the variants.
    let json: serde_json::Value = serde_json::from_str(&s.to_json()).unwrap();
    assert_eq!(json["fields"][0]["type"], "bool");
    assert_eq!(json["fields"][1]["type"], "mat4");
    assert_eq!(json["fields"][2]["type"], "enum");
    assert_eq!(json["fields"][2]["variants"][1], "angry");
    assert_eq!(json["fields"][3]["type"], "str64");
    assert_eq!(Schema::from_json(&s.to_json()).unwrap(), s);
}

#[test]
fn new_field_types_validate_and_hash_their_meaning() {
    let base = || Schema {
        name: "Thing".into(),
        version: 1,
        doc: String::new(),
        fields: vec![FieldSchema::enumeration("kind", &["a", "b"])],
    };
    let refuse = |s: Schema, needle: &str| {
        let err = s.validate().unwrap_err();
        assert!(err.reason().contains(needle), "{}", err.reason());
    };
    base().validate().unwrap();
    let mut s = base();
    s.fields[0].variants.clear();
    refuse(s, "declares no variants");
    let mut s = base();
    s.fields[0].variants[1] = "a".into();
    refuse(s, "declares variant 'a' twice");
    let mut s = base();
    s.fields[0].default = Some(vec![2.0]);
    refuse(s, "is not a valid enum value");
    let mut s = base();
    s.fields.push(FieldSchema::new("t", FieldType::Str64));
    s.fields[1].default = Some(vec![0.0]);
    refuse(s, "a string field starts empty");
    let mut s = base();
    s.fields.push(FieldSchema::new("on", FieldType::Bool));
    s.fields[1].default = Some(vec![0.5]);
    refuse(s, "is not a valid bool value");
    let mut s = base();
    s.fields.push(FieldSchema::new("n", FieldType::U32));
    s.fields[1].variants = vec!["x".into()];
    refuse(s, "only an enum field has them");

    // Reordering variants changes the meaning of the bytes, so the hash.
    let mut swapped = base();
    swapped.fields[0].variants.swap(0, 1);
    assert_ne!(swapped.layout_hash(), base().layout_hash());
}
