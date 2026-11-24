//! The engine modules' records are the IDL's: a world registers the IDL's
//! layout first and og-physics and og-audio adopt it only when the name, the
//! field order, the names, the storage types and an enum's variants agree
//! (og-ecs refuses the engine otherwise).

use og_api::host::{RECORDS, RecordDecl};
use og_core::reflect::{FieldSchema, FieldType, Reflect, Schema};

/// The IDL's fields, compared at the module's version. Engine saves are
/// strict layouts, not a history of record-version migrations.
fn idl_schema(decl: &RecordDecl, version: u32) -> Schema {
    Schema {
        name: decl.schema.to_string(),
        version,
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
                        | "Tween" | "TilemapAsset" | "Rig2D" => FieldType::U64,
                        // Event text: og-anim's marker names are str64.
                        "String" => FieldType::Str(64),
                        other => panic!(
                            "{}.{}: column {other} has no storage type here",
                            decl.schema, f.name
                        ),
                    }
                };
                // A list column's `column` is its element's kind.
                let ty = if f.list > 0 {
                    FieldType::list(ty.scalar(), f.list as usize).expect("a list of numbers")
                } else {
                    ty
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
        failures.push(format!(
            "{} is not declared in api/<module>.ogt",
            module.name
        ));
        return;
    };
    let version = module.version;
    let idl = idl_schema(decl, version);
    if idl.layout_hash() != module.layout_hash() {
        let show = |s: &Schema| {
            s.fields
                .iter()
                .map(|f| format!("{} {}{:?}", f.name, f.ty.name(), f.variants))
                .collect::<Vec<_>>()
                .join(", ")
        };
        failures.push(format!(
            "{}: api/<module>.ogt has version {version} [{}], the module registers version {} [{}]",
            module.name,
            show(&idl),
            module.version,
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
    check::<og_physics::BodySlept>(&mut failures);
    check::<og_physics::BodyWoke>(&mut failures);
    check::<og_physics::Joint>(&mut failures);
    check::<og_physics::JointBroken>(&mut failures);
    check::<og_physics::ContactHook>(&mut failures);
    check::<og_physics::Vehicle>(&mut failures);
    check::<og_physics::VehicleWheel>(&mut failures);
    check::<og_physics::SoftBody>(&mut failures);
    check::<og_physics::Garment>(&mut failures);
    check::<og_physics::Rope>(&mut failures);
    check::<og_physics::Cloth>(&mut failures);
    check::<og_physics::Buoyancy>(&mut failures);
    check::<og_physics::Fluid2D>(&mut failures);
    check::<og_physics::Fluid3D>(&mut failures);
    check::<og_audio::AudioSource>(&mut failures);
    check::<og_audio::AudioListener>(&mut failures);
    check::<og_audio::Audio>(&mut failures);
    check::<og_audio::SoundStarted>(&mut failures);
    check::<og_audio::SoundEnded>(&mut failures);
    check::<og_audio::AudioBusFx>(&mut failures);
    check::<og_audio::AudioBusNode>(&mut failures);
    check::<og_audio::AudioSnapshot>(&mut failures);
    check::<og_audio::AudioEvent>(&mut failures);
    check::<og_anim::Transform>(&mut failures);
    check::<og_anim::Skeleton>(&mut failures);
    check::<og_anim::AnimationPlayer>(&mut failures);
    check::<og_anim::AnimationGraph>(&mut failures);
    check::<og_anim::Blendshapes>(&mut failures);
    check::<og_anim::IkChain>(&mut failures);
    check::<og_anim::LookAt>(&mut failures);
    check::<og_anim::JointLimit>(&mut failures);
    check::<og_anim::FootLock>(&mut failures);
    check::<og_anim::Hang>(&mut failures);
    check::<og_anim::SpringBone>(&mut failures);
    check::<og_anim::SpringCollider>(&mut failures);
    check::<og_anim::AnimationEvent>(&mut failures);
    check::<og_anim::Flipbook>(&mut failures);
    check::<og_anim::FlipbookEvent>(&mut failures);
    check::<og_anim::Skeleton2D>(&mut failures);
    check::<og_anim::Skeleton2DEvent>(&mut failures);
    check::<og_anim::IkTarget2D>(&mut failures);
    check::<og_tween::TweenCompleted>(&mut failures);
    check::<og_input::Pointers>(&mut failures);
    check::<og_input::Gamepad>(&mut failures);
    check::<og_input::PointerDown>(&mut failures);
    check::<og_input::PointerUp>(&mut failures);
    check::<og_input::PointerMove>(&mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
