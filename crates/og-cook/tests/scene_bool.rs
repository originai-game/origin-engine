//! A scene component with a bool or an enum field packs to og-core's layout:
//! one byte per bool and a u32 per enum, so each row is 4 + the schema's size
//! bytes. Writing them as four f32 bytes made `og cook` panic on the
//! engine's og.DirectionalLight (castShadow is a bool): "28 bytes is not a
//! whole number of 25-byte rows".

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use og_core::reflect::{FieldSchema, FieldType, Schema};

#[test]
fn scene_timing_payloads_cook_losslessly_and_reject_invalid_text() {
    let root = std::env::temp_dir().join(format!("og-cook-scene-ease-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    let path = root.join("assets/scenes/shot.ogt");
    let options = CookOptions {
        threads: 1,
        schemas: vec![Schema {
            name: "Shot".into(),
            version: 1,
            doc: String::new(),
            fields: vec![FieldSchema::new("ease", FieldType::Ease)],
        }],
        ..CookOptions::default()
    };
    use og_core::ease::{Ease, StepPosition};
    for (text, expected) in [
        (
            "cubic-bezier(.17,.85,.63,.08)",
            Some(Ease::CubicBezier([0.17, 0.85, 0.63, 0.08])),
        ),
        (
            "steps(4,jump-start)",
            Some(Ease::Steps {
                count: 4,
                position: StepPosition::JumpStart,
            }),
        ),
        ("steps(0)", None),
        ("cubic-bezier(2,0,0,1)", None),
    ] {
        std::fs::write(&path, format!("type scene\nversion 1\n---\nentity \"shot\" {{\n Shot {{\n ease \"{text}\"\n }}\n}}\n")).unwrap();
        let result = cook_project(&root, &options);
        if let Some(ease) = expected {
            result.unwrap();
            let scene = Bundle::open(&root.join("cooked"))
                .unwrap()
                .load_scene("scenes/shot.ogt")
                .unwrap();
            let row = scene
                .components
                .iter()
                .find(|c| c.name == "Shot")
                .unwrap()
                .get(0)
                .unwrap();
            assert_eq!(og_core::ease::Ease::from_bytes(row), Some(ease));
        } else {
            let report = result.unwrap();
            assert!(report.assets.iter().any(|asset| {
                asset
                    .error
                    .as_ref()
                    .is_some_and(|error| error.contains(text))
            }));
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn bool_and_enum_fields_pack_to_the_schema_layout() {
    let root = std::env::temp_dir().join(format!("og-cook-scene-bool-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(
        root.join("assets/scenes/lit.ogt"),
        "type scene\nversion 1\n---\nname \"Lit\"\nentity \"sun\" {\n  og.DirectionalLight {\n    color 1 0.5 0.25\n    intensity 3\n    castShadow false\n    shadowDistance 80\n    bus 2\n  }\n}\nentity \"lamp\" {\n  og.DirectionalLight {\n    castShadow true\n  }\n}\n",
    )
    .unwrap();
    let mut light = Schema {
        name: "og.DirectionalLight".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("color", FieldType::Vec3),
            FieldSchema::new("intensity", FieldType::F32),
            FieldSchema::new("castShadow", FieldType::Bool),
            FieldSchema::new("shadowDistance", FieldType::F32),
            FieldSchema::new("bus", FieldType::Enum),
        ],
    };
    light.fields[4].variants = vec!["master".into(), "music".into(), "sfx".into()];
    let options = CookOptions {
        threads: 1,
        schemas: vec![light],
        ..CookOptions::default()
    };
    cook_project(&root, &options).expect("the scene cooks");
    let scene = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene("scenes/lit.ogt")
        .unwrap();
    let t = scene
        .components
        .iter()
        .find(|c| c.name == "og.DirectionalLight")
        .unwrap();
    // color 12 + intensity 4 + castShadow 1 + shadowDistance 4 + bus 4.
    let sun = t.get(0).unwrap();
    assert_eq!(sun.len(), 25);
    assert_eq!(f32::from_le_bytes(sun[4..8].try_into().unwrap()), 0.5);
    assert_eq!(f32::from_le_bytes(sun[12..16].try_into().unwrap()), 3.0);
    assert_eq!(sun[16], 0);
    assert_eq!(f32::from_le_bytes(sun[17..21].try_into().unwrap()), 80.0);
    assert_eq!(u32::from_le_bytes(sun[21..25].try_into().unwrap()), 2);
    assert_eq!(t.get(1).unwrap()[16], 1);
    let _ = std::fs::remove_dir_all(&root);
}
