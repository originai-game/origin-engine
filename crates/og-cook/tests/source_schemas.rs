//! The content identity is shared by serde, schema validation and author types.
use super::temp;
use og_cook::cook::source_schemas::{self, ContentRef};
use og_core::reflect::source::{InputSchema, typescript};
use serde_json::json;

#[test]
fn content_reference_contract_is_not_a_path_string_or_runtime_handle() {
    let schema = InputSchema::<ContentRef>::new().unwrap();
    let value = json!({"$ogRef": "north/effect-texture/smoke#emission#array"});
    let decoded = schema.decode(value.clone()).unwrap();
    assert_eq!(decoded.id, "north/effect-texture/smoke#emission#array");
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    for wrong in [
        json!("north/model/gate"),
        json!(42),
        json!({"$ogRef": ""}),
        json!({"$ogRef": "north/model/gate", "kind": "model"}),
        json!({"id": "north/model/gate"}),
    ] {
        assert!(schema.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }
    assert!(source_schemas::schema("not-a-kind").is_none());

    let dir = temp::TempDir::new("content-ref-schema");
    std::fs::write(
        dir.join("ref.ts"),
        format!(
            "{}\n{}",
            typescript(schema.document(), "ContentRef").unwrap(),
            r#"
const valid: ContentRef = { $ogRef: "north/model/gate" };
// @ts-expect-error Content references are tagged, not bare strings.
const path: ContentRef = "north/model/gate";
// @ts-expect-error A runtime numeric handle is not a content identity.
const handle: ContentRef = 42;
// @ts-expect-error The target kind comes from the index, not a second wire field.
const extra: ContentRef = { $ogRef: "north/model/gate", kind: "model" };
// @ts-expect-error Generated content identities are immutable.
valid.$ogRef = "north/model/other";
"#
        ),
    )
    .unwrap();
    let check = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "ref.ts"])
        .current_dir(&dir)
        .output()
        .expect("the pinned tsrs checker must be installed (docs/ci.md)");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}
