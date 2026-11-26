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
    let canonical = program.run_ir().unwrap();
    assert_eq!(canonical.root().into_json(), value);
    // Numeric storage survives the realm and action cache, including view
    // offsets, signedness, non-ASCII strings and a writer buffer growth.
    use og_cook::cook::generator::value::{Data, Element, Value};
    std::fs::write(
        dir.join("buffers.ts"),
        r#"
export default {
  a: new Int8Array([-97, 13]), b: new Uint8Array([251, 17]),
  c: new Uint8ClampedArray([-7, 301]), d: new Int16Array([-30001, 513]),
  e: new Uint16Array([65001, 1027]), f: new Int32Array([-2147483647, 66051]),
  g: new Uint32Array([4294967295, 16909060]),
  h: new Float32Array([91, -7.25, 0.125, 103]).subarray(1, 3),
  i: new Float64Array([-1.5e100, 0.0625]), empty: new Uint16Array(0),
  large: Float32Array.from({length: 32769}, (_, i) => (i % 7 - 3) / 8),
  label: '北门🙂', '\u{10000}': true, '\ue000': false,
};
// Private encoding must use captured intrinsics, not user-editable hooks.
Object.defineProperty(Uint8Array, Symbol.species, {get() {throw new Error('species hook');}});
Object.defineProperty(Uint8Array.prototype, 'length', {get() {throw new Error('length hook');}});
"#,
    )
    .unwrap();
    let buffers = Program::prepare_value(&dir, "buffers.ts").unwrap();
    let data = buffers.run_ir().unwrap();
    assert_eq!(buffers.run_ir().unwrap(), data);
    assert_eq!(Data::from_bytes(data.as_bytes().to_vec()).unwrap(), data);
    let Value::Object(values) = data.root() else {
        panic!("an authored object must remain structural data");
    };
    for (key, element, expected) in [
        ("a", Element::I8, vec![-97.0, 13.0]),
        ("b", Element::U8, vec![251.0, 17.0]),
        ("c", Element::U8Clamped, vec![0.0, 255.0]),
        ("d", Element::I16, vec![-30001.0, 513.0]),
        ("e", Element::U16, vec![65001.0, 1027.0]),
        ("f", Element::I32, vec![-2147483647.0, 66051.0]),
        ("g", Element::U32, vec![4294967295.0, 16909060.0]),
        ("h", Element::F32, vec![-7.25, 0.125]),
        ("i", Element::F64, vec![-1.5e100, 0.0625]),
        ("empty", Element::U16, vec![]),
    ] {
        let Value::Buffer(buffer) = &values[key] else {
            panic!("{key}: lost typed storage");
        };
        assert_eq!(buffer.element, element, "{key}");
        assert_eq!(buffer.len(), expected.len(), "{key}");
        assert_eq!(buffer.numbers().collect::<Vec<_>>(), expected, "{key}");
    }
    let Value::Buffer(samples) = &values["large"] else {
        panic!("large samples expanded into a tree");
    };
    assert_eq!(samples.len(), 32769);
    assert_eq!(samples.as_bytes().len(), 32769 * 4);
    assert_eq!(samples.numbers().next(), Some(-0.375));
    assert_eq!(samples.numbers().last(), Some(-0.25));
    assert_eq!(values["label"], Value::String("北门🙂".into()));
    assert_eq!(values["\u{10000}"], Value::Bool(true));
    assert_eq!(values["\u{e000}"], Value::Bool(false));
    let Value::Buffer(wide) = &values["g"] else {
        unreachable!();
    };
    assert_eq!(wide.as_bytes(), &[255, 255, 255, 255, 4, 3, 2, 1]);
    assert_eq!(buffers.run().unwrap()["h"], json!([-7.25, 0.125]));
    std::fs::write(
        dir.join("reordered.ts"),
        "import value from './buffers.ts'; export default Object.fromEntries(Object.entries(value).reverse());",
    )
    .unwrap();
    assert_eq!(
        Program::prepare_value(&dir, "reordered.ts")
            .unwrap()
            .run_ir()
            .unwrap(),
        data,
        "object insertion order and entry path cannot change canonical data"
    );

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
    for expr in [
        "og.ref(42)",
        "og.ref('')",
        "og.file.unknown({})",
        "Object.setPrototypeOf(new Float32Array([1.25]), Int32Array.prototype)",
        "Object.setPrototypeOf(new Float32Array([NaN]), Int32Array.prototype)",
    ] {
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
