//! The content identity is shared by serde, schema validation and author types.
use super::temp;
use og_cook::cook::source_schemas::{self, ContentRef};
use og_core::reflect::source::{InputSchema, typescript};
use serde_json::json;

#[test]
fn author_factories_use_the_registered_input_contracts() {
    use og_cook::cook::{generator::Program, source_schemas::File};
    let dir = temp::TempDir::new("author-factories");
    std::fs::write(
        dir.join("content.ts"),
        r#"
import * as og from '@origingame/engine/author';
const id = og.ref('north/model/gate#collider');
export default {
  content: og.file.strings({locale:'zh-CN',entries:{gate:'北门'}}),
  id,
  factories: Object.keys(og.file).sort(),
  other: og.file['texture-array']({}),
};
"#,
    )
    .unwrap();
    let program = Program::prepare_value(&dir, "content.ts").unwrap();
    let value = program.run().unwrap();
    let file: File = serde_json::from_value(value["content"].clone()).unwrap();
    assert_eq!(file.kind, "strings");
    assert_eq!(file.value["entries"]["gate"], "北门");
    file.check(None).unwrap();
    assert_eq!(value["id"], json!({"$ogRef":"north/model/gate#collider"}));
    assert_eq!(value["other"]["$ogFile"], "texture-array");
    let mut kinds: Vec<_> = source_schemas::all().map(|k| k.name).collect();
    kinds.extend(["scene", "prefab", "table"]);
    kinds.sort_unstable();
    assert_eq!(value["factories"], json!(kinds));
    for (wrong, path) in [
        (
            json!({"$ogFile":"strings","value":{"locale":"en","entries":{"gate":42}}}),
            "/entries/gate",
        ),
        (json!({"$ogFile":"unknown","value":{}}), ""),
        (json!({"$ogFile":"scene","value":{"entities":[]}}), ""),
    ] {
        let file: File = serde_json::from_value(wrong).unwrap();
        assert_eq!(file.check(None).unwrap_err().path, path);
    }
    assert!(
        serde_json::from_value::<File>(json!({
            "$ogFile":"strings","value":{},"extra":true
        }))
        .is_err()
    );
    let schema = source_schemas::world::scene::SceneInput::schema_with_records(&[], &[]).unwrap();
    let scene: File = serde_json::from_value(json!({
        "$ogFile":"scene","value":{"entities":[]}
    }))
    .unwrap();
    scene.check(Some(&schema)).unwrap();

    // Provenance is relative to the factory's argument, not the descriptor
    // envelope. A multi-file candidate is evaluated without modifying disk.
    use og_cook::cook::generator::source::{Edit, Source};
    use std::collections::BTreeMap;
    let text = "import {file as files} from '@origingame/engine/author';\nimport {label} from './helper.ts';\nexport default (files.strings({\n  locale: 'en',\n  entries: {gate: 'Gate', computed: label}, // keep\n}) satisfies unknown);\n";
    std::fs::write(dir.join("edit.ts"), text).unwrap();
    std::fs::write(dir.join("helper.ts"), "export const label = 'Helper';").unwrap();
    std::fs::write(dir.join("unused.ts"), "export const unused = 1;").unwrap();
    let original = Program::prepare_value(&dir, "edit.ts").unwrap();
    assert_eq!(original.source.content_kind().unwrap(), "strings");
    let gate = original.source.field("/entries/gate").unwrap();
    assert_eq!(gate.value, Some(json!("Gate")));
    assert_eq!(gate.location.line, 5);
    assert_eq!(
        &text[gate.location.start as usize..gate.location.end as usize],
        "'Gate'"
    );
    assert_eq!(
        original.source.field("/entries/computed").unwrap_err().code,
        "computed-source"
    );
    let patched = original
        .source
        .patch(
            text,
            &[Edit {
                path: "/entries/gate".into(),
                expected: Some(json!("Gate")),
                value: json!("North gate"),
            }],
        )
        .unwrap();
    assert!(patched.contains("gate: 'North gate', computed: label}, // keep"));
    let mut overlay = BTreeMap::from([
        ("edit.ts".into(), patched),
        (
            "helper.ts".into(),
            "export const label = 'Candidate helper';".into(),
        ),
    ]);
    let candidate = Program::prepare_value_with_overlay(&dir, "edit.ts", &overlay).unwrap();
    assert_eq!(original.run().unwrap()["value"]["entries"]["gate"], "Gate");
    let evaluated = candidate.run().unwrap();
    assert_eq!(
        evaluated["value"]["entries"],
        json!({"gate":"North gate","computed":"Candidate helper"})
    );
    assert_ne!(candidate.key, original.key);
    assert_ne!(candidate.source.digest(), original.source.digest());
    let checked: File = serde_json::from_value(evaluated.clone()).unwrap();
    checked.check(None).unwrap();
    std::fs::write(
        dir.join("captured.ts"),
        "import {input} from '@origingame/engine/author'; export default input.text('helper.ts');",
    )
    .unwrap();
    assert_eq!(
        Program::prepare_value_with_overlay(&dir, "captured.ts", &overlay)
            .unwrap()
            .run()
            .unwrap(),
        json!("export const label = 'Candidate helper';")
    );
    overlay.insert("unused.ts".into(), "export const unused = 99;".into());
    let warm = Program::prepare_value_with_overlay(&dir, "edit.ts", &overlay).unwrap();
    assert_eq!(warm.key, candidate.key);
    assert_eq!(warm.run().unwrap(), evaluated);
    assert_eq!(std::fs::read_to_string(dir.join("edit.ts")).unwrap(), text);
    assert_eq!(
        std::fs::read_to_string(dir.join("helper.ts")).unwrap(),
        "export const label = 'Helper';"
    );
    for path in [
        "../escape.ts",
        "./edit.ts",
        "edit/../edit.ts",
        "helper.json",
        "missing.ts",
    ] {
        let invalid = BTreeMap::from([(path.into(), "export default {};".into())]);
        assert!(
            Program::prepare_value_with_overlay(&dir, "edit.ts", &invalid).is_err(),
            "{path}"
        );
    }
    for (imports, call, kind) in [
        (
            "import * as og from '@origingame/engine/author';",
            "og.file['texture-array']({})",
            Some("texture-array"),
        ),
        (
            "import {file} from '@origingame/engine/author';",
            "file.strings({})",
            Some("strings"),
        ),
        (
            "const file = {strings: (v: unknown) => v};",
            "file.strings({})",
            None,
        ),
        (
            "import {file} from '@origingame/engine/author'; const make = file.strings;",
            "make({})",
            None,
        ),
        (
            "import {file} from '@origingame/engine/author'; const kind = 'strings';",
            "file[kind]({})",
            None,
        ),
        (
            "import type {file} from '@origingame/engine/author';",
            "file.strings({})",
            None,
        ),
        (
            "import {file} from '@origingame/engine/author';",
            "file.strings(...[{}])",
            None,
        ),
    ] {
        let source = Source::new("source.ts", &format!("{imports}\nexport default ({call});"));
        assert_eq!(source.content_kind().ok().as_deref(), kind, "{call}");
    }
    for expr in ["og.ref(42)", "og.ref('')", "og.file.unknown({})"] {
        std::fs::write(
            dir.join("bad.ts"),
            format!("import * as og from '@origingame/engine/author'; export default {expr};"),
        )
        .unwrap();
        assert!(
            Program::prepare_value(&dir, "bad.ts")
                .unwrap()
                .run()
                .is_err()
        );
    }

    // Check the actual public declarations against freshly generated shapes,
    // rather than a handwritten approximation of any domain input.
    let engine = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine")
        .canonicalize()
        .unwrap();
    let declarations = include_str!("../../../packages/engine/src/author.d.ts")
        .replace(
            "../types/index.d.ts",
            &engine.join("types/index.d.ts").to_string_lossy(),
        )
        .replace("../types/author-inputs.d.ts", "./inputs.d.ts");
    std::fs::write(dir.join("author.d.ts"), declarations).unwrap();
    std::fs::write(
        dir.join("inputs.d.ts"),
        source_schemas::declarations().unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("contracts.ts"),
        r#"
import * as og from './author';
const data = og.file.strings({locale:'en',entries:{gate:'Gate'}});
const kind: 'strings' = data.$ogFile;
const id = og.ref('north/model/gate#collider');
const exact: 'north/model/gate#collider' = id.$ogRef;
// @ts-expect-error A misspelled kind has no factory.
og.file.string({});
// @ts-expect-error Invalid nested fields do not pass the schema-derived input.
og.file.strings({locale:'en',entries:{gate:42}});
// @ts-expect-error Contextual records must be bound by project generation first.
og.file.scene({entities:[]});
// @ts-expect-error Runtime handles are not resource IDs.
og.ref(42);
// @ts-expect-error Descriptors are immutable.
data.value.entries.gate = 'Other';
// @ts-expect-error Ref identity is immutable.
id.$ogRef = 'north/model/other';
"#,
    )
    .unwrap();
    let check = std::process::Command::new("tsrs")
        .args([
            "--ignoreConfig",
            "--strict",
            "--noEmit",
            "--target",
            "esnext",
            "--moduleResolution",
            "bundler",
            "--module",
            "esnext",
            "contracts.ts",
        ])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs checker");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

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
