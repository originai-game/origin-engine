//! The engine modules' records are the IDL's: a world registers the IDL's
//! layout first and og-physics and og-audio adopt it only when the name, the
//! field order, the names, the storage types and an enum's variants agree
//! (og-ecs refuses the engine otherwise). And the UI's ThemeColor enum names
//! every colour role of og-ui's palette, in its order.

use og_api::host::{RECORDS, RecordDecl};
use og_core::reflect::{FieldSchema, FieldType, Reflect, Schema};
use std::path::Path;

/// The IDL's layout of `decl` at `version`: the IDL declares the fields,
/// [`VERSIONS`] the version they are at.
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

/// Each record's version and the hash of its IDL fields at that version
/// (`idl_schema(decl, 0).layout_hash()`), written down here rather than read
/// from the module, so a module whose version is not the IDL's fails, and a
/// layout change (a field added, removed, renamed or retyped) without a
/// version bump fails too: a save restores a record by its version, so
/// changing one's fields means a new version (og.Gamepad 2: eight pads).
const VERSIONS: &[(&str, u32, u64)] = &[
    ("og.AnimationEvent", 1, 0x28aaa9078ce00980),
    ("og.AnimationGraph", 1, 0x14f7a654994eb4b2),
    ("og.AnimationPlayer", 1, 0x2bec5d121d54fe87),
    ("og.Audio", 1, 0xc68086afae021f9b),
    ("og.AudioBusFx", 2, 0x55dfec49f19793bb),
    ("og.AudioBusNode", 1, 0xd5fe02e154cb1de2),
    ("og.AudioEvent", 1, 0x40a0bb866463cafa),
    ("og.AudioListener", 1, 0x51f48da421d73dc2),
    ("og.AudioSnapshot", 1, 0x515c0d8ddf345f37),
    ("og.AudioSource", 2, 0x35081d41554c04b2),
    ("og.Blendshapes", 2, 0x4f86806f3bbd31a0),
    ("og.BodySlept", 1, 0x316a96ff6521a2ed),
    ("og.BodyWoke", 1, 0x268b8a476bcb2864),
    ("og.Collider", 1, 0xe83c74a0520778bc),
    ("og.CollisionEnded", 1, 0x4e62d2a73639aeb5),
    ("og.CollisionStarted", 1, 0x6c9e735a7abf1f6a),
    ("og.Contact", 1, 0xe9a7082085c77412),
    ("og.ContactHook", 1, 0x5f7ff8a7ff78c1bb),
    ("og.Flipbook", 1, 0xbbede7fc327207f1),
    ("og.FlipbookEvent", 1, 0x7d1245df738a14b1),
    ("og.Fluid3D", 1, 0xf7ae93e48ea458df),
    ("og.FootLock", 1, 0x335fb3e07795add3),
    ("og.Gamepad", 2, 0xc97a34bf75360270),
    ("og.IkChain", 1, 0xbbe02154306d4e60),
    ("og.IkTarget2D", 1, 0x4e454eb44cd38470),
    ("og.Joint", 2, 0x263a678c33ed5248),
    ("og.JointBroken", 1, 0xb0cc4d0a3a3cb8c3),
    ("og.JointLimit", 1, 0x4b0c9cec8d17a70f),
    ("og.LookAt", 1, 0x219ef1857429f798),
    ("og.Physics", 2, 0xc6d013db66fc764a),
    ("og.PointerDown", 1, 0xa0091884ee01db37),
    ("og.PointerMove", 1, 0x9328be6142683eea),
    ("og.PointerUp", 1, 0x2f71597da11eef1a),
    ("og.Pointers", 1, 0xe0a9491012ab78ea),
    ("og.RigidBody", 2, 0xd0c1d5054f7c1d2e),
    ("og.SensorEntered", 1, 0xa33b45062725b7f1),
    ("og.SensorExited", 1, 0xcaa743ce70b55c7a),
    ("og.Skeleton", 1, 0xdf1d8ad824fb1c2a),
    ("og.Skeleton2D", 1, 0x0987b2b676dc1ad4),
    ("og.Skeleton2DEvent", 1, 0x4ade934299b37c48),
    ("og.SoundEnded", 1, 0x90355457f6c84261),
    ("og.SoundStarted", 1, 0xb828d371c76e923c),
    ("og.SpringBone", 1, 0xeb6a65842c7e9d8b),
    ("og.SpringCollider", 1, 0x4078f34f3ce7d3b3),
    ("og.Transform", 1, 0x1f6309afb81c85f3),
    ("og.TweenCompleted", 1, 0xe394841eee68bb0a),
    ("og.Vehicle", 2, 0x45a35b63ff56d1d2),
    ("og.VehicleWheel", 1, 0xc3aa8551ae5e84c4),
    ("og.SoftBody", 1, 0x3bcfc31a2cb8077b),
    ("og.Velocity", 1, 0x70aff545497a9c4d),
];

fn check<T: Reflect>(failures: &mut Vec<String>) {
    let module = T::schema();
    let Some(decl) = RECORDS.iter().find(|r| r.schema == module.name) else {
        failures.push(format!(
            "{} is not declared in api/<module>.ogt",
            module.name
        ));
        return;
    };
    let fields = idl_schema(decl, 0).layout_hash();
    let Some(&(_, version, pinned)) = VERSIONS.iter().find(|(n, _, _)| *n == module.name) else {
        failures.push(format!(
            "{} has no entry in VERSIONS: add (\"{}\", {}, {fields:#018x})",
            module.name, module.name, module.version
        ));
        return;
    };
    if fields != pinned {
        failures.push(format!(
            "{}'s fields changed since version {version}: bump its version (the module's #[reflect(version)] and VERSIONS) and pin {fields:#018x}",
            module.name
        ));
    }
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

#[test]
fn theme_color_names_every_palette_role() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let api = og_idl::read(&root.join(og_idl::IDL_PATH)).unwrap();
    let have: Vec<String> = api
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
        .expect("api/<module>.ogt declares ThemeColor");
    assert_eq!(
        have,
        og_ui::Palette::NAMES,
        "ThemeColor and og-ui's palette roles differ (names or order)"
    );
}
