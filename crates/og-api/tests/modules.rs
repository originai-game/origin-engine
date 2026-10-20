//! The engine modules' records are the IDL's: a world registers the IDL's
//! layout first and og-physics and og-audio adopt it only when the name, the
//! field order, the names, the storage types and an enum's variants agree
//! (og-ecs refuses the engine otherwise). And the UI's ThemeColor enum names
//! every colour of the design tokens og-ui is generated from.

use og_api::host::{RECORDS, RecordDecl};
use og_core::reflect::{FieldSchema, FieldType, Reflect, Schema};
use std::path::Path;

fn idl_schema(decl: &RecordDecl) -> Schema {
    Schema {
        name: decl.schema.to_string(),
        version: 1,
        doc: String::new(),
        fields: decl
            .fields
            .iter()
            .map(|f| {
                let ty = if !f.variants.is_empty() {
                    FieldType::Enum
                } else {
                    match f.column {
                        "f32" => FieldType::F32,
                        "f64" => FieldType::F64,
                        "u32" => FieldType::U32,
                        "i32" => FieldType::I32,
                        "bool" => FieldType::Bool,
                        "Vector2" => FieldType::Vec2,
                        "Vector3" | "Color" => FieldType::Vec3,
                        "Quaternion" => FieldType::Quat,
                        "Matrix4" => FieldType::Mat4,
                        "Entity" | "Geometry" | "Material" | "Asset" | "Sound"
                        | "SkeletonAsset" | "ClipAsset" | "GraphAsset" | "FlipbookSheet"
                        | "Tween" | "TilemapAsset" => FieldType::U64,
                        // Event text: og-anim's marker names are str64.
                        "String" => FieldType::Str(64),
                        other => panic!(
                            "{}.{}: column {other} has no storage type here",
                            decl.schema, f.name
                        ),
                    }
                };
                let mut s = FieldSchema::new(f.name, ty);
                s.variants = f.variants.iter().map(|v| v.to_string()).collect();
                s
            })
            .collect(),
    }
}

fn check<T: Reflect>(failures: &mut Vec<String>) {
    let module = T::schema();
    let Some(decl) = RECORDS.iter().find(|r| r.schema == module.name) else {
        failures.push(format!("{} is not declared in api.ogt", module.name));
        return;
    };
    let idl = idl_schema(decl);
    if idl.layout_hash() != module.layout_hash() {
        let show = |s: &Schema| {
            s.fields
                .iter()
                .map(|f| format!("{} {}{:?}", f.name, f.ty.name(), f.variants))
                .collect::<Vec<_>>()
                .join(", ")
        };
        failures.push(format!(
            "{}: api.ogt has [{}], the module registers [{}]",
            module.name,
            show(&idl),
            show(&module)
        ));
    }
}

#[test]
fn physics_audio_animation_and_input_records_have_the_idl_layouts() {
    let mut failures = Vec::new();
    check::<og_physics::Transform>(&mut failures);
    check::<og_physics::RigidBody>(&mut failures);
    check::<og_physics::Collider>(&mut failures);
    check::<og_physics::Velocity>(&mut failures);
    check::<og_physics::Physics>(&mut failures);
    check::<og_physics::CollisionStarted>(&mut failures);
    check::<og_physics::CollisionEnded>(&mut failures);
    check::<og_physics::SensorEntered>(&mut failures);
    check::<og_physics::SensorExited>(&mut failures);
    check::<og_physics::Contact>(&mut failures);
    check::<og_physics::Fluid3D>(&mut failures);
    check::<og_audio::AudioSource>(&mut failures);
    check::<og_audio::AudioListener>(&mut failures);
    check::<og_audio::Audio>(&mut failures);
    check::<og_audio::SoundStarted>(&mut failures);
    check::<og_audio::SoundEnded>(&mut failures);
    check::<og_anim::Transform>(&mut failures);
    check::<og_anim::Skeleton>(&mut failures);
    check::<og_anim::AnimationPlayer>(&mut failures);
    check::<og_anim::AnimationGraph>(&mut failures);
    check::<og_anim::Blendshapes>(&mut failures);
    check::<og_anim::IkChain>(&mut failures);
    check::<og_anim::AnimationEvent>(&mut failures);
    check::<og_anim::Flipbook>(&mut failures);
    check::<og_anim::FlipbookEvent>(&mut failures);
    check::<og_tween::TweenCompleted>(&mut failures);
    check::<og_input::Pointers>(&mut failures);
    check::<og_input::Gamepad>(&mut failures);
    check::<og_input::PointerDown>(&mut failures);
    check::<og_input::PointerUp>(&mut failures);
    check::<og_input::PointerMove>(&mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn theme_color_names_every_token_colour() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tokens: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("crates/og-ui/tokens/tokens.json")).unwrap(),
    )
    .unwrap();
    let camel = |k: &str| {
        let mut out = String::new();
        let mut up = false;
        for c in k.chars() {
            if c == '-' {
                up = true;
            } else if up {
                out.extend(c.to_uppercase());
                up = false;
            } else {
                out.push(c);
            }
        }
        out
    };
    let mut want: Vec<String> = tokens["color"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| !k.starts_with('_'))
        .map(|k| camel(k))
        .collect();
    want.sort();
    let api = og_idl::read(&root.join(og_idl::IDL_PATH)).unwrap();
    let mut have: Vec<String> = api
        .modules
        .iter()
        .flat_map(|m| m.items.iter())
        .find_map(|i| match i {
            og_idl::model::Item::Enum(e) if e.name == "ThemeColor" => Some(
                e.variants
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .expect("api.ogt declares ThemeColor");
    have.sort();
    assert_eq!(have, want, "ThemeColor and tokens.json `color` differ");
}
