//! The content identity is shared by serde, schema validation and author types.
use super::temp;
use og_cook::cook::source_schemas::{self, ContentRef};
use og_core::reflect::source::{InputSchema, typescript};
use serde_json::json;

#[test]
fn content_index_discovers_namespaces_and_types_without_evaluation() {
    use og_cook::cook::{source_index::Index, source_project::Project};
    use std::collections::BTreeMap;
    let dir = temp::TempDir::new("content-index");
    let write = |file: &str, text: &str| {
        let at = dir.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    let manifest = |root: &str| {
        format!(
            "import {{defineProject}} from '@origingame/engine/author'; export default defineProject({});",
            json!({"name":"World","id":"world","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts",
            "modules":[{"id":"north","root":root,"dependsOn":["base"]},{"id":"empty","root":"modules/empty"}]})
        )
    };
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    std::fs::create_dir_all(dir.join("modules/north")).unwrap();
    std::fs::create_dir_all(dir.join("modules/empty")).unwrap();
    write("project.ts", &manifest("modules/north"));
    let project = Project::open(&dir).unwrap();
    let empty = Index::open(&dir, &project).unwrap();
    assert!(empty.entries.is_empty());
    assert_eq!(empty.declarations().len(), 3);
    let wind = "import {file as data} from '@origingame/engine/author';\nimport missing from '../../sources/missing.ts';\nthrow new Error('CONTENT_EXECUTED');\nexport default data.field(missing());";
    write("content/not-a-kind/wind.ts", wind);
    let import = "import * as og from '@origingame/engine/author';\nthrow new Error('IMPORT_EXECUTED');\nexport default og.file.import({kind:'model',source:'assets/gate.glb'});";
    write("modules/north/content/model/gate'北.ts", import);
    write(
        "modules/north/content/scene/main.ts",
        "import * as og from '@origingame/engine/author'; export default og.file.scene({entities:[]});",
    );
    for skipped in [".og", "cooked", "og-out", "dist", "node_modules"] {
        write(&format!("content/{skipped}/invalid.ts"), "not source");
    }
    let index = Index::open(&dir, &project).unwrap();
    assert_eq!(index.entries.len(), 3);
    assert_eq!(index.entries["base/not-a-kind/wind"].kind, "field");
    let gate = &index.entries["north/model/gate'北"];
    assert_eq!((&*gate.factory, &*gate.kind), ("import", "model"));
    assert_eq!(index.files[&gate.file], gate.id);
    assert_eq!(index.entries["north/scene/main"].kind, "scene");
    let types = index.declarations();
    assert_eq!(types, Index::open(&dir, &project).unwrap().declarations());
    let candidate = BTreeMap::from([(gate.file.clone(), import.replace("'model'", "'texture'"))]);
    let edited = Index::open_with_overlay(&dir, &project, &candidate).unwrap();
    assert_eq!(edited.entries[&gate.id].kind, "texture");
    assert_ne!(edited.entries[&gate.id].digest, gate.digest);
    assert_eq!(edited.declarations()["base"], types["base"]);
    let value_edit = BTreeMap::from([(gate.file.clone(), import.replace("gate.glb", "other.glb"))]);
    let value_index = Index::open_with_overlay(&dir, &project, &value_edit).unwrap();
    assert_ne!(value_index.entries[&gate.id].digest, gate.digest);
    assert_eq!(value_index.declarations(), types);
    assert_eq!(
        std::fs::read_to_string(dir.join(&gate.file)).unwrap(),
        import
    );

    // The actual public author module is augmented, not a handwritten stand-in.
    let engine = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine")
        .canonicalize()
        .unwrap();
    for (module, text) in &types {
        write(&format!(".og/types/content/{module}.d.ts"), text);
    }
    write("tsconfig.json", &json!({
        "compilerOptions":{"strict":true,"noEmit":true,"target":"esnext","module":"esnext","moduleResolution":"bundler",
            "paths":{"@origingame/engine/author":[engine.join("src/author.d.ts").to_str().unwrap()]}},
        "include":["contracts.ts",".og/types/**/*.d.ts"]
    }).to_string());
    write(
        "contracts.ts",
        r#"
import type {ContentIndex} from '@origingame/engine/author';
const id: keyof ContentIndex = "north/model/gate'北";
const kind: ContentIndex[typeof id]['kind'] = 'model';
const factory: ContentIndex[typeof id]['factory'] = 'import';
const file: ContentIndex[typeof id]['file'] = "modules/north/content/model/gate'北.ts";
const wind: ContentIndex['base/not-a-kind/wind']['kind'] = 'field';
// @ts-expect-error A missing source is not in this snapshot.
const absent: keyof ContentIndex = 'north/model/missing';
// @ts-expect-error Primary kind is not the import factory's name or its folder.
const wrong: ContentIndex[typeof id]['kind'] = 'texture';
// @ts-expect-error Derived subresources are not guessed from factory spellings.
const derived: keyof ContentIndex = "north/model/gate'北#collider";
declare const entry: ContentIndex[typeof id];
// @ts-expect-error Generated identities and metadata are readonly.
entry.kind = 'model';
"#,
    );
    let check = std::process::Command::new("tsrs")
        .args(["--project", "tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs checker");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    // Module relocation preserves IDs and source byte identities, not file paths.
    std::fs::create_dir_all(dir.join("chapters")).unwrap();
    std::fs::rename(dir.join("modules/north"), dir.join("chapters/north")).unwrap();
    write("project.ts", &manifest("chapters/north"));
    let moved = Project::open(&dir).unwrap();
    let moved_index = Index::open(&dir, &moved).unwrap();
    assert_eq!(moved_index.entries[&gate.id].digest, gate.digest);
    assert_eq!(
        moved_index.entries[&gate.id].file,
        "chapters/north/content/model/gate'北.ts"
    );
    assert_eq!(moved_index.declarations()["base"], types["base"]);
    std::fs::remove_file(dir.join("chapters/north/content/model/gate'北.ts")).unwrap();
    assert!(
        !Index::open(&dir, &moved)
            .unwrap()
            .entries
            .contains_key(&gate.id)
    );

    let bad_file = "content/bad.ts";
    for value in [
        "{kind: 'not-a-kind'}",
        "{kind: 3}",
        "{}",
        "{kind: tag}",
        "{kind:'model', ...rest}",
        "{kind:'model',kind:'texture'}",
        "importedOptions",
    ] {
        write(
            bad_file,
            &format!(
                "import * as og from '@origingame/engine/author';\nexport default og.file.import({value});"
            ),
        );
        let error = Index::open(&dir, &moved).err().unwrap();
        assert_eq!((&*error.file, &*error.path), (bad_file, "/kind"));
        assert_eq!(error.location.line, 2);
        assert!(error.message.contains("direct string literal"), "{error}");
    }
    for value in ["file.unknown({})", "file[kind]({})", "({})"] {
        write(
            bad_file,
            &format!("import {{file}} from '@origingame/engine/author'; export default {value};"),
        );
        assert!(Index::open(&dir, &moved).is_err());
    }
    std::fs::remove_file(dir.join(bad_file)).unwrap();
    for file in [
        "content/old.ogt",
        "content/wind.d.ts",
        "content/wind#field.ts",
        "content/wind*.ts",
        "content/wind.ts ",
    ] {
        write(file, wind);
        assert!(Index::open(&dir, &moved).is_err(), "accepted {file}");
        std::fs::remove_file(dir.join(file)).unwrap();
    }
    write("content/not-a-kind/WIND.ts", wind);
    let collision = Index::open(&dir, &moved).err().unwrap();
    assert!(collision.message.contains("collides"), "{collision}");
    std::fs::remove_file(dir.join("content/not-a-kind/WIND.ts")).unwrap();
    for file in [
        "../outside.ts",
        "content/../src/main.ts",
        ".og/types/content/base.d.ts",
        "content/absent.ts",
    ] {
        assert!(
            Index::open_with_overlay(&dir, &moved, &BTreeMap::from([(file.into(), wind.into())]))
                .is_err()
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.join("src"), dir.join("content/linked")).unwrap();
        assert!(
            Index::open(&dir, &moved)
                .err()
                .unwrap()
                .message
                .contains("symlink")
        );
        std::fs::remove_file(dir.join("content/linked")).unwrap();
    }
}

#[test]
fn content_preparation_tracks_scoped_sources_and_captured_inputs() {
    use og_cook::cook::{generator::Program, source_index::Index, source_project::Project};
    use std::collections::BTreeMap;
    let dir = temp::TempDir::new("content-preparation");
    let write = |file: &str, text: &str| {
        let at = dir.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    let manifest = |north: &[&str], east: &[&str]| {
        write(
            "project.ts",
            &format!(
                "import {{defineProject}} from '@origingame/engine/author'; export default defineProject({});",
                json!({"name":"World","id":"world","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts",
                "modules":[{"id":"north","root":"modules/north","dependsOn":north},
                {"id":"east","root":"modules/east","dependsOn":east}]})
            ),
        );
    };
    write(
        "src/main.ts",
        "export type Token = string; throw new Error('GAME_EXECUTED');",
    );
    write(
        "modules/north/src/schema/labels.ts",
        "export const locale = 'en';",
    );
    write(
        "lib/shared.ts",
        "import {input} from '@origingame/engine/author'; export const name = input.text('sources/label.txt');",
    );
    write(
        "modules/east/lib/colors.ts",
        "export {name as east} from '../../../lib/shared.ts';",
    );
    write("sources/label.txt", "BASE, NOT NORTH");
    write("modules/east/sources/label.txt", "EAST, NOT NORTH");
    write("modules/north/sources/label.txt", "北门");
    write("assets/payload.bin", "wrong root");
    std::fs::create_dir_all(dir.join("modules/north/assets")).unwrap();
    std::fs::write(dir.join("modules/north/assets/payload.bin"), [7, 231]).unwrap();
    let file = "modules/north/content/main.ts";
    let text = r#"
import * as og from '@origingame/engine/author';
import type {Token} from '../../../src/main.ts';
import {name} from '../../../lib/shared.ts';
import {east} from '../../east/lib/colors.ts';
import {locale} from '../src/schema/labels.ts';
const label: Token = name;
export default og.file.strings({locale, entries:{label, east, pinned:'P', bytes:og.input.bytes('assets/payload.bin').join(',')}});
"#;
    write(file, text);
    manifest(&["base", "east"], &["base"]);
    let project = Project::open(&dir).unwrap();
    let index = Index::open(&dir, &project).unwrap();
    let entry = &index.entries["north/main"];
    let empty = BTreeMap::new();
    let prepared = Program::prepare_content(&dir, &project, entry, &empty).unwrap();
    assert_eq!(
        prepared.input_dependencies,
        [
            "modules/north/assets/payload.bin",
            "modules/north/sources/label.txt"
        ]
    );
    assert_eq!(prepared.imports.len(), 4);
    assert_eq!(
        prepared.imports["modules/east/lib/colors.ts"],
        ["lib/shared.ts"]
    );
    assert!(prepared.imports["lib/shared.ts"].is_empty());
    assert!(!prepared.dependencies.iter().any(|p| p == "src/main.ts"
        || p == "sources/label.txt"
        || p == "modules/east/sources/label.txt"));
    let expected = json!({"$ogFile":"strings","value":{"locale":"en","entries":{"label":"北门","east":"北门","pinned":"P","bytes":"7,231"}}});
    // Preparation captures bytes; neither cold evaluation nor a cache hit may
    // reread them. Shared helpers use the ENTRY module's raw-input authority.
    write("modules/north/sources/label.txt", "changed after capture");
    assert_eq!(prepared.run().unwrap(), expected);
    assert_eq!(prepared.run().unwrap(), expected);
    serde_json::from_value::<source_schemas::File>(expected.clone())
        .unwrap()
        .check(None)
        .unwrap();
    let changed = Program::prepare_content(&dir, &project, entry, &empty).unwrap();
    assert_ne!(changed.key, prepared.key);
    assert_eq!(
        changed.run().unwrap()["value"]["entries"]["east"],
        "changed after capture"
    );
    write("sources/label.txt", "unused raw input changed");
    assert_eq!(
        Program::prepare_content(&dir, &project, entry, &empty)
            .unwrap()
            .key,
        changed.key
    );
    write("modules/north/sources/label.txt", "北门");

    let candidate = BTreeMap::from([(file.into(), text.replace("pinned:'P'", "pinned:'Q'"))]);
    assert!(
        Program::prepare_content(&dir, &project, entry, &candidate)
            .err()
            .unwrap()
            .contains("stale")
    );
    let updated = Index::open_with_overlay(&dir, &project, &candidate).unwrap();
    let preview =
        Program::prepare_content(&dir, &project, &updated.entries["north/main"], &candidate)
            .unwrap();
    assert_eq!(preview.run().unwrap()["value"]["entries"]["pinned"], "Q");
    assert_eq!(std::fs::read_to_string(dir.join(file)).unwrap(), text);

    // Access is checked on each import edge, not once per visited helper or
    // against the entry's flattened/transitive dependency closure.
    manifest(&["east"], &["base"]);
    let missing_direct = Project::open(&dir).unwrap();
    assert!(
        Program::prepare_content(&dir, &missing_direct, entry, &empty)
            .err()
            .unwrap()
            .contains("module north to declare a direct dependency on base")
    );
    manifest(&["base", "east"], &[]);
    let missing_reuse = Project::open(&dir).unwrap();
    assert!(
        Program::prepare_content(&dir, &missing_reuse, entry, &empty)
            .err()
            .unwrap()
            .contains("module east to declare a direct dependency on base")
    );
    // Conversely, a legal east → base import does not require north → base
    // unless north imports base itself.
    let indirect = BTreeMap::from([(
        file.into(),
        text.replace(
            "import {name} from '../../../lib/shared.ts';",
            "const name = 'local';",
        ),
    )]);
    assert_eq!(
        Program::prepare_content(
            &dir,
            &missing_direct,
            &Index::open_with_overlay(&dir, &missing_direct, &indirect)
                .unwrap()
                .entries["north/main"],
            &indirect
        )
        .unwrap()
        .run()
        .unwrap()["value"]["entries"]["label"],
        "local"
    );

    for target in [
        "./other.ts",
        "../../../src/main.ts",
        "../../../project.ts",
        "../sources/not-code.ts",
        "../../../undeclared/lib/helper.ts",
        "../lib/types.d.ts",
        "../lib/.og/hidden.ts",
    ] {
        let overlay = BTreeMap::from([(file.into(), format!("import '{target}';\n{text}"))]);
        let candidate = Index::open_with_overlay(&dir, &project, &overlay).unwrap();
        let error =
            Program::prepare_content(&dir, &project, &candidate.entries["north/main"], &overlay)
                .err()
                .unwrap();
        assert!(error.contains("cannot import"), "{target}: {error}");
        assert!(!error.contains("GAME_EXECUTED"));
    }
    for input in [
        "../sources/label.txt",
        "sources/../sources/label.txt",
        "src/main.ts",
        "modules/east/sources/label.txt",
        "Sources/label.txt",
        "assets\\payload.bin",
    ] {
        let overlay = BTreeMap::from([(
            file.into(),
            text.replace("'assets/payload.bin'", &json!(input).to_string()),
        )]);
        let candidate = Index::open_with_overlay(&dir, &project, &overlay).unwrap();
        assert!(
            Program::prepare_content(&dir, &project, &candidate.entries["north/main"], &overlay)
                .is_err(),
            "accepted input {input}"
        );
    }
    write(file, &text.replace("pinned:'P'", "pinned:'disk changed'"));
    assert!(
        Program::prepare_content(&dir, &project, entry, &empty)
            .err()
            .unwrap()
            .contains("stale")
    );
}

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
fn schema_bound_references_ignore_json_and_preserve_container_constraints() {
    use source_schemas::references::{Reference, ReferenceSchema};
    let edge = |path: &str, id: &str, kinds: &[&str]| Reference {
        path: path.into(),
        id: id.into(),
        kinds: Some(kinds.iter().map(|s| (*s).to_owned()).collect()),
    };
    for kind in source_schemas::all() {
        ReferenceSchema::new((kind.schema)()).unwrap_or_else(|e| panic!("{}: {e}", kind.name));
    }
    let schema = source_schemas::world::pack::PackInput::schema();
    let references = ReferenceSchema::new(schema).unwrap();
    let value = json!({
        "about":{"$ogRef":"this is author JSON", "nested":[{"$ogRef":"also JSON"}]},
        "fonts":[{"$ogRef":"base/font/body"}],
        "icons":{"a~/界":{"$ogRef":"base/texture/icon"}},
        "images":{"plate":{"src":{"light":{"$ogRef":"base/texture/light"},"dark":{"$ogRef":"base/texture/dark"}}, "slice":2}},
        "sounds":{"press":{"clips":[{"$ogRef":"base/sfx/tap"}]}}
    });
    let expected = vec![
        edge("/fonts/0", "base/font/body", &["font"]),
        edge("/icons/a~0~1界", "base/texture/icon", &["texture"]),
        edge("/images/plate/src/dark", "base/texture/dark", &["texture"]),
        edge(
            "/images/plate/src/light",
            "base/texture/light",
            &["texture"],
        ),
        edge("/sounds/press/clips/0", "base/sfx/tap", &["sfx", "sound"]),
    ];
    assert_eq!(references.collect(&value).unwrap(), expected);
    assert_eq!(references.collect(&value).unwrap(), expected);
    let shared = json!({"images":{"plate":{"src":{"$ogRef":"base/texture/shared"},"slice":2}}});
    assert_eq!(
        references.collect(&shared).unwrap(),
        vec![edge(
            "/images/plate/src",
            "base/texture/shared",
            &["texture"]
        )]
    );
    let mut invalid = value;
    invalid["fonts"][0] = json!(7);
    assert_eq!(references.collect(&invalid).unwrap_err().path, "/fonts/0");

    // Reflected handles carry the same marker even when their schema is inline.
    let document = source_schemas::reflected::schema_for_record("og.PostEffects").unwrap();
    let reflected = InputSchema::<serde_json::Value>::from_document(document).unwrap();
    let edges = ReferenceSchema::new(&reflected)
        .unwrap()
        .collect(&json!({"lut":{"$ogRef":"base/lut/dusk"}}))
        .unwrap();
    assert_eq!(edges, vec![edge("/lut", "base/lut/dusk", &["lut"])]);
}

#[test]
fn reference_schemas_select_branches_and_follow_recursive_maps() {
    use source_schemas::references::ReferenceSchema;
    let schema = InputSchema::<serde_json::Value>::from_document(json!({
        "type":"object", "properties": {
            "tree":{"$ref":"#/$defs/a"},
            "choice":{"oneOf":[
                {"type":"object","required":["kind","value"],"properties":{"kind":{"const":"raw"},"value":true}},
                {"type":"object","required":["kind","value"],"properties":{"kind":{"const":"edge"},"value":{"$ref":"#/$defs/ref"}}}
            ]},
            "map":{"type":"object", "properties":{"fixed":true},
                "patternProperties":{"^edge/":{"$ref":"#/$defs/ref", "x-og-ref-kinds":["texture"]}},
                "additionalProperties":{"$ref":"#/$defs/ref", "x-og-ref-kinds":["font"]}}
        }, "$defs": {
            "ref": InputSchema::<ContentRef>::new().unwrap().document(),
            "a":{"type":"object","properties":{"b":{"$ref":"#/$defs/b"},"asset":{"$ref":"#/$defs/ref"}}},
            "b":{"type":"object","properties":{"a":{"$ref":"#/$defs/a"}}}
        }
    })).unwrap();
    let references = ReferenceSchema::new(&schema).unwrap();
    let mut value = json!({
        "tree":{"b":{"a":{"asset":{"$ogRef":"base/model/deep"}}}},
        "choice":{"kind":"raw","value":{"$ogRef":"arbitrary JSON"}},
        "map":{"fixed":{"$ogRef":"also JSON"},"edge/界":{"$ogRef":"base/texture/icon"},"other":{"$ogRef":"base/font/body"}}
    });
    let edges = references.collect(&value).unwrap();
    assert_eq!(
        edges
            .iter()
            .map(|e| (e.path.as_str(), e.id.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("/map/edge~1界", "base/texture/icon"),
            ("/map/other", "base/font/body"),
            ("/tree/b/a/asset", "base/model/deep")
        ]
    );
    assert_eq!(
        edges[0]
            .kinds
            .as_ref()
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["texture"]
    );
    assert_eq!(
        edges[1]
            .kinds
            .as_ref()
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["font"]
    );
    assert!(edges[2].kinds.is_none());
    value["choice"]["kind"] = json!("edge");
    assert_eq!(references.collect(&value).unwrap()[0].path, "/choice/value");

    let constrained = InputSchema::<serde_json::Value>::from_document(json!({
        "allOf":[
            InputSchema::<ContentRef>::new().unwrap().document(),
            {"x-og-ref-kinds":["font","texture"]},
            {"x-og-ref-kinds":["font"]}
        ]
    }))
    .unwrap();
    let edges = ReferenceSchema::new(&constrained)
        .unwrap()
        .collect(&json!({"$ogRef":"base/font/body"}))
        .unwrap();
    assert_eq!(edges[0].path, "");
    assert_eq!(edges[0].kinds, Some([String::from("font")].into()));
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
