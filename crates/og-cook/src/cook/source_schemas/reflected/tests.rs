//! The IDL-backed public patch contract, including direct serde and real TS.
use super::{RecordMarker, Reflected, schema_for_record};
use crate::cook::test_dir::TempDir;
use og_core::reflect::source::{InputSchema, typescript};
use serde_json::json;

#[derive(Debug)]
struct Transform;
impl RecordMarker for Transform {
    const NAME: &'static str = "Transform";
    const EXCLUDED: &'static [&'static str] = &["scale"];
    const REQUIRED: &'static [&'static str] = &["position"];
}

struct Interactor;
impl RecordMarker for Interactor {
    const NAME: &'static str = "Interactor";
}

struct PostEffects;
impl RecordMarker for PostEffects {
    const NAME: &'static str = "PostEffects";
}

struct Material;
impl RecordMarker for Material {
    const NAME: &'static str = "StandardMaterial";
    fn layout() -> Option<og_core::Schema> {
        Some(<og_render_data::StandardMaterial as og_core::Reflect>::schema())
    }
}

#[test]
fn reflected_patches_validate_without_a_factory_and_generate_typescript() {
    for record in og_records::RECORDS {
        schema_for_record(record.name).unwrap_or_else(|e| panic!("{}: {e}", record.name));
    }
    let schema = InputSchema::<Reflected<Transform>>::new().unwrap();
    let valid = json!({"position": [-2.5, 3, 7], "quaternion": [0, 0, 0, 1]});
    let decoded: Reflected<Transform> = serde_json::from_value(valid.clone()).unwrap();
    assert_eq!(decoded.fields()["position"], json!([-2.5, 3, 7]));
    assert_eq!(decoded.into_fields()["quaternion"], json!([0, 0, 0, 1]));
    schema.decode(valid).unwrap();
    for invalid in [
        json!({}),
        json!({"position": [2, 3]}),
        json!({"position": [2, 3, 7, 9]}),
        json!({"position": [2, "three", 7]}),
        json!({"position": [2, 3, 7], "scale": [1, 1, 1]}),
        json!({"position": [2, 3, 7], "typo": 1}),
        json!({"position": [2, 3, 1e39]}),
    ] {
        assert!(
            serde_json::from_value::<Reflected<Transform>>(invalid.clone()).is_err(),
            "serde accepted {invalid}"
        );
        assert!(
            schema.decode(invalid.clone()).is_err(),
            "schema accepted {invalid}"
        );
    }
    assert!(schema_for_record("BoxGeometryDesc").is_err());
    // Resource patches share the same IDL bounds and enum spellings.
    let render = InputSchema::<serde_json::Value>::from_document(
        schema_for_record("RenderSettings").unwrap(),
    )
    .unwrap();
    render
        .validate(&json!({"exposure": 1.25, "toneMapping": "agx"}))
        .unwrap();
    assert!(
        render
            .validate(&json!({"toneMapping": "imaginary"}))
            .is_err()
    );
    let model = schema_for_record("Model").unwrap();
    assert_eq!(model["properties"]["asset"]["x-og-handle-column"], "Asset");
    assert!(model["properties"]["asset"].get("x-og-ref-kinds").is_none());
    let text: Reflected<Interactor> =
        serde_json::from_value(json!({"action": "界".repeat(10)})).unwrap();
    assert_eq!(text.fields()["action"].as_str().unwrap().len(), 30);
    assert!(
        serde_json::from_value::<Reflected<Interactor>>(json!({"action": "界".repeat(11)}))
            .is_err()
    );
    assert!(serde_json::from_value::<Reflected<Interactor>>(json!({"action": "a\0b"})).is_err());
    let content: Reflected<PostEffects> =
        serde_json::from_value(json!({"lut": {"$ogRef": "base/lut/dusk"}})).unwrap();
    assert_eq!(content.fields()["lut"]["$ogRef"], "base/lut/dusk");
    for wrong in [
        json!(42),
        json!("base/lut/dusk"),
        json!({"$ogRef": ""}),
        json!({"$ogRef": "base/lut/dusk", "kind": "lut"}),
    ] {
        assert!(serde_json::from_value::<Reflected<PostEffects>>(json!({"lut": wrong})).is_err());
    }

    let material = InputSchema::<Reflected<Material>>::new().unwrap();
    material
        .decode(json!({"color": [0.2, 0.5, 0.8], "roughness": 0.3, "side": 2}))
        .unwrap();
    for wrong in [
        json!({"roughness": 1.01}),
        json!({"side": -1}),
        json!({"color": [0.2, 0.5]}),
        json!({"color": [-0.1, 0.5, 0.8]}),
        json!({"roughnes": 0.3}),
    ] {
        assert!(material.decode(wrong.clone()).is_err(), "accepted {wrong}");
        assert!(serde_json::from_value::<Reflected<Material>>(wrong).is_err());
    }
    let dir = TempDir::new("reflected-contract");
    std::fs::write(
        dir.join("patch.ts"),
        format!(
            "{}\n{}\n{}",
            typescript(schema.document(), "Patch").unwrap(),
            typescript(material.document(), "MaterialPatch").unwrap(),
            r#"
const valid: Patch = { position: [-2.5, 3, 7], quaternion: [0, 0, 0, 1] };
const material: MaterialPatch = { color: [0.2, 0.5, 0.8], roughness: 0.3, side: 2 };
// @ts-expect-error Packed layout vectors retain their exact length.
const materialVector: MaterialPatch = { color: [0.2, 0.5] };
// @ts-expect-error Packed layout fields retain their scalar type.
const materialScalar: MaterialPatch = { roughness: "smooth" };
// @ts-expect-error Layout patches reject invented properties.
const materialUnknown: MaterialPatch = { roughnes: 0.3 };
// @ts-expect-error The domain requires position even though ordinary patches do not.
const missing: Patch = {};
// @ts-expect-error A vector is not an arbitrary array.
const short: Patch = { position: [2, 3] };
// @ts-expect-error A vector has no trailing elements.
const long: Patch = { position: [2, 3, 7, 9] };
// @ts-expect-error Excluded fields are not writable in this domain.
const excluded: Patch = { position: [2, 3, 7], scale: [1, 1, 1] };
// @ts-expect-error Reflection is closed against unknown fields.
const unknown: Patch = { position: [2, 3, 7], typo: 1 };
// @ts-expect-error A field's array is readonly, not just its containing property.
valid.position[0] = 8;
"#
        ),
    )
    .unwrap();
    let check = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "patch.ts"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs is installed");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}
