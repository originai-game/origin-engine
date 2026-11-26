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
const north = og.defineModule({id:'north',root:'modules/north',dependsOn:['base']});
const project = og.defineProject({name:'World',id:'world',engine:'0.9.0',entry:'src/main.ts',
  modules:[north],scenes:{main:og.ref('base/scene/main')},
  packs:[{id:'north',include:['north/**'],dependsOn:['base'],optional:true}],
  publish:{fit:'letterbox',aspect:'16:9',dprCap:2,loadingLabels:{fetch:'Loading'}},
});
// @ts-expect-error Project fields are derived, not an arbitrary option bag.
og.defineProject({name:'World',id:'world',engine:'0.9.0',entry:'src/main.ts',typo:1});
// @ts-expect-error Project scene paths use the shared reference wire.
og.defineProject({name:'World',id:'world',engine:'0.9.0',entry:'src/main.ts',scenes:{main:'scene/main'}});
// @ts-expect-error Bootstrap output remains readonly.
project.modules = [];
// @ts-expect-error Modules cannot declare delivery options.
og.defineModule({id:'north',root:'modules/north',optional:true});
// @ts-expect-error A required project pin may not be omitted.
og.defineProject({name:'World',id:'world',entry:'src/main.ts'});
// @ts-expect-error The settings vocabulary is closed and typed.
og.defineProject({name:'World',id:'world',engine:'0.9.0',entry:'src/main.ts',publish:{fit:'stretch'}});
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
fn project_bootstrap_checks_namespaces_without_running_game_or_content() {
    use og_cook::cook::{
        generator::source::{Edit, Source},
        source_project::Project,
    };
    use std::collections::BTreeMap;
    let dir = temp::TempDir::new("project-bootstrap");
    let write = |file: &str, text: &str| {
        let at = dir.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    let manifest = |value: &serde_json::Value| {
        format!(
            "import {{defineProject}} from '@origingame/engine/author';\nexport default defineProject({value});"
        )
    };
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    write(
        "src/schema/records.ts",
        "throw new Error('SCHEMA_EXECUTED');",
    );
    write(
        "content/field/wind.ts",
        "throw new Error('CONTENT_EXECUTED');",
    );
    let small =
        json!({"name":"Room","id":"room","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts"});
    write("project.ts", &manifest(&small));
    let minimal = Project::open(&dir).unwrap();
    assert_eq!(minimal.module_order, ["base"]);
    assert_eq!(minimal.pack_order, ["base"]);
    assert_eq!(minimal.module_for("content/field/wind.ts").id, "base");
    assert_eq!(minimal.dependencies, ["project.ts"]);
    assert_eq!(minimal.data, Project::open(&dir).unwrap().data);

    write(
        "modules/north/module.ts",
        "import {defineModule} from '@origingame/engine/author'; export default defineModule({id:'north',root:'modules/north',dependsOn:['base'],inputs:['sources/**/*.csv']});",
    );
    write(
        "modules/north/extra/lib/constants.ts",
        "export const value = 1;",
    );
    write("modules/NORTH/lib/constants.ts", "export const value = 2;");
    write(
        "modules/north/src/schema/records.ts",
        "throw new Error('MODULE_SCHEMA_EXECUTED');",
    );
    write("lib/title.ts", "export const title = 'World';");
    let large = json!({"name":"World","id":"world","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts",
        "schemas":["src/schema/records.ts","modules/north/src/schema/records.ts"],
        "modules":[{"id":"nested","root":"modules/north/extra","dependsOn":["north"]},
                   {"id":"north","root":"modules/north","dependsOn":["base"],"inputs":["sources/**/*.csv"]}],
        "packs":[{"id":"north","include":["north/**"],"dependsOn":["shared"]},
                 {"id":"base","dependsOn":["shared"]},
                 {"id":"shared","include":["base/model/**"]},
                 {"id":"dlc","optional":true,"dependsOn":["north"]}],
        "scene":"menu","scenes":{"menu":{"$ogRef":"base/scene/menu"}},"tick":37,"seed":4294967295u32,
        "perf":{"render":0.25},"publish":{"fit":"letterbox","aspect":"16:9"}
    });
    let imported = manifest(&large)
        .replace("export default", "import north from './modules/north/module.ts'; import {title} from './lib/title.ts';\nexport default")
        .replace(&large["modules"][1].to_string(), "north")
        .replace("\"name\":\"World\"", "\"name\":title");
    write("project.ts", &imported);
    let world = Project::open(&dir).unwrap();
    assert_eq!(world.module_order, ["base", "north", "nested"]);
    assert_eq!(world.pack_order, ["shared", "base", "north", "dlc"]);
    assert_eq!(
        world.module_for("modules/north/content/field/wind.ts").id,
        "north"
    );
    assert_eq!(
        world
            .module_for("modules/north/extra/content/field/wind.ts")
            .id,
        "nested"
    );
    assert_eq!(
        world
            .module_for("modules/northern/content/field/wind.ts")
            .id,
        "base"
    );
    assert_eq!(world.input.tick, Some(37));
    assert_eq!(world.input.seed, Some(u32::MAX));
    assert_eq!(
        world.dependencies,
        ["lib/title.ts", "modules/north/module.ts", "project.ts"]
    );
    let overlay = BTreeMap::from([(
        "lib/title.ts".into(),
        "export const title = 'Candidate';".into(),
    )]);
    let candidate = Project::open_with_overlay(&dir, &overlay).unwrap();
    assert_eq!(candidate.input.name, "Candidate");
    assert_ne!(candidate.evaluation_key, world.evaluation_key);
    assert_eq!(
        std::fs::read_to_string(dir.join("lib/title.ts")).unwrap(),
        "export const title = 'World';"
    );

    // Each rejected alternative differs at a boundary relevant to discovery,
    // namespacing or delivery. Success on the small project is not sufficient.
    for (key, value, path, message) in [
        (
            "engine",
            json!("not-installed"),
            "/engine",
            "does not match",
        ),
        ("entry", json!("../src/main.ts"), "/entry", "canonical"),
        (
            "entry",
            json!("content/field/wind.ts"),
            "/entry",
            "game code",
        ),
        (
            "schemas",
            json!(["src/main.ts"]),
            "/schemas/0",
            "src/schema",
        ),
        (
            "modules",
            json!([{"id":"base","root":"modules/north"}]),
            "/modules/0/id",
            "reserved",
        ),
        (
            "modules",
            json!([{"id":"north","root":"modules/north","dependsOn":["missing"]}]),
            "/modules/0/dependsOn/0",
            "unknown",
        ),
        (
            "modules",
            json!([{"id":"north","root":"modules/north","dependsOn":["base","base"]}]),
            "/modules/0/dependsOn/1",
            "duplicate",
        ),
        (
            "modules",
            json!([{"id":"north","root":"modules/north","dependsOn":["nested"]},{"id":"nested","root":"modules/north/extra","dependsOn":["north"]}]),
            "/modules/1/dependsOn",
            "cycle",
        ),
        (
            "modules",
            json!([{"id":"north","root":"modules/north"},{"id":"east","root":"modules/NORTH"}]),
            "/modules/1/root",
            "case folding",
        ),
        (
            "modules",
            json!([{"id":"north","root":"src"}]),
            "/modules/0/root",
            "reserved",
        ),
        (
            "packs",
            json!([{"id":"north"},{"id":"north"}]),
            "/packs/1/id",
            "duplicate",
        ),
        (
            "packs",
            json!([{"id":"base","optional":true}]),
            "/packs/0/optional",
            "boot pack",
        ),
        (
            "packs",
            json!([{"id":"north","dependsOn":["missing"]}]),
            "/packs/0/dependsOn/0",
            "unknown",
        ),
        (
            "packs",
            json!([{"id":"base","dependsOn":["dlc"]},{"id":"dlc","optional":true}]),
            "/packs/0/dependsOn/0",
            "optional",
        ),
        ("scene", json!("absent"), "/scene", "absent"),
        ("perf", json!({"render":-1}), "/perf/render", "minimum"),
        // The shared schema deliberately reports a nullable object's union root.
        ("publish", json!({"dprCap":4.01}), "/publish", "anyOf"),
    ] {
        let mut wrong = small.clone();
        wrong[key] = value;
        write("project.ts", &manifest(&wrong));
        let error = Project::open(&dir)
            .err()
            .unwrap_or_else(|| panic!("accepted {wrong}"));
        assert_eq!(error.path, path, "{error}");
        assert!(error.message.contains(message), "{error}");
    }
    for file in [
        "content/field/wind.ts",
        "src/main.ts",
        "src/schema/../main.ts",
    ] {
        write(
            "project.ts",
            &format!("import './{file}';\n{}", manifest(&small)),
        );
        let error = Project::open(&dir)
            .err()
            .expect("forbidden bootstrap import");
        assert!(error.message.contains("bootstrap cannot read"), "{error}");
        assert!(!error.message.contains("EXECUTED"), "{error}");
    }
    write(
        "project.ts",
        &format!("import '@origingame/engine';\n{}", manifest(&small)),
    );
    assert!(
        Project::open(&dir)
            .err()
            .unwrap()
            .message
            .contains("imports must be relative")
    );

    // Source diagnostics and edits unwrap only the real imported helpers.
    let invalid = format!(
        "import {{defineProject as project}} from '@origingame/engine/author';\nexport default project({{\nname:'Room',id:'room',engine:'{}',entry:'src/main.ts',\ntick:241\n}});",
        env!("CARGO_PKG_VERSION")
    );
    write("project.ts", &invalid);
    let error = Project::open(&dir).err().unwrap();
    assert_eq!(error.path, "/tick");
    assert_eq!(error.location.line, 4);
    assert_eq!(error.location.column, 6);
    let source = Source::new("project.ts", &invalid);
    let patched = source
        .patch(
            &invalid,
            &[Edit {
                path: "/tick".into(),
                expected: Some(json!(241)),
                value: json!(37),
            }],
        )
        .unwrap();
    let overlay = BTreeMap::from([("project.ts".into(), patched)]);
    assert_eq!(
        Project::open_with_overlay(&dir, &overlay)
            .unwrap()
            .input
            .tick,
        Some(37)
    );
    for (imports, call, writable) in [
        (
            "import * as og from '@origingame/engine/author';",
            "og['defineModule']({id:'north'})",
            true,
        ),
        (
            "import {defineModule as module} from '@origingame/engine/author';",
            "module({id:'north'})",
            true,
        ),
        (
            "const defineModule = (v:unknown) => v;",
            "defineModule({id:'north'})",
            false,
        ),
        (
            "import type {defineModule} from '@origingame/engine/author';",
            "defineModule({id:'north'})",
            false,
        ),
        (
            "import {defineModule} from '@origingame/engine/author'; const alias=defineModule;",
            "alias({id:'north'})",
            false,
        ),
    ] {
        let source = Source::new("module.ts", &format!("{imports}\nexport default {call};"));
        assert_eq!(source.field("/id").is_ok(), writable, "{call}");
        assert!(source.content_kind().is_err(), "a manifest is not content");
    }
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
