use super::*;

#[test]
fn typed_project_cook_refreshes_helpers_and_rejects_catalogue_collisions() {
    let root = TempDir::new("og-typed-cook");
    for file in [
        "og-project.json",
        "project.ts",
        "src/main.ts",
        "lib/timing.ts",
        "lib/depth.ts",
        "content/fields/depth.ts",
        "modules/island/content/fields/slope.ts",
        "assets/scenes/harbor.ogt",
        "assets/scenes/inland.ogt",
    ] {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::copy(repo().join("tests/fixtures/ts-startup").join(file), at).unwrap();
    }
    let metadata = std::fs::read(root.join("og-project.json")).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    for samples in [[3.0, 11.0, -2.0, 17.0], [19.0, -7.0, 5.0, 23.0]] {
        std::fs::write(
            root.join("lib/depth.ts"),
            format!("export const samples = {samples:?};"),
        )
        .unwrap();
        let result = cook();
        assert!(result.status.success(), "{}", text(&result));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        assert_eq!(
            bundle.load_field("base/fields/depth").unwrap().values,
            samples
        );
        assert_eq!(
            bundle.load_field("island/fields/slope").unwrap().values,
            [5.0, 19.0]
        );
        assert!(bundle.read("scenes/harbor.ogt").is_ok());
        assert!(
            bundle.read("base/scenes/harbor").is_err(),
            "no second output or lookup alias"
        );
        assert_eq!(
            std::fs::read(root.join("og-project.json")).unwrap(),
            metadata
        );
        // A no-change cook must retain the same observable data.
        assert!(cook().status.success());
        assert_eq!(
            og_cook::runtime::Bundle::open(&root.join("cooked"))
                .unwrap()
                .load_field("base/fields/depth")
                .unwrap()
                .values,
            samples
        );
    }
    std::fs::create_dir_all(root.join("content/scenes")).unwrap();
    std::fs::write(
        root.join("content/scenes/harbor.ts"),
        "import {file} from '@origingame/engine/author'; export default file.scene({entities:[]});",
    )
    .unwrap();
    let collision = cook();
    assert!(
        !collision.status.success(),
        "catalogue silently preferred a source"
    );
    assert!(
        text(&collision).contains("base/scenes/harbor"),
        "{}",
        text(&collision)
    );
    assert!(
        text(&collision).contains("duplicate content identity"),
        "{}",
        text(&collision)
    );
}

#[test]
fn checked_project_startup_is_fresh_and_metadata_is_read_only() {
    use serde_json::json;
    let root = TempDir::new("og-project-startup");
    let write = |file: &str, text: &str| {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    let metadata = "{\n  \"version\": 1, \"name\": \"港口\", \"engine\": \"dev.20261126.3\",\n  \"look\": {\"$ogRef\":\"base/looks/harbor\"}\n}\n";
    write("og-project.json", metadata);
    write("project.ogt", "THIS MUST NOT BE PARSED");
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    write("lib/timing.ts", "export const timing = {tick:30, seed:83};");
    write("assets/scenes/harbor.ogt", "type scene\nversion 1\n---\n");
    write("assets/scenes/inland.ogt", "type scene\nversion 1\n---\n");
    write("assets/looks/harbor.ogt", "type look\nversion 1\n---\n");
    write(
        "assets/dialogue/arrival.yarn",
        "title: Start\n---\nWelcome\n===\n",
    );
    let source = "import {defineProject,ref} from '@origingame/engine/author';\nimport {timing} from './lib/timing.ts';\nexport default defineProject({id:'startup',entry:'src/main.ts',...timing,saveVersion:4,scene:'harbor',scenes:{harbor:ref('base/scenes/harbor'),inland:ref('base/scenes/inland')},create:{form:'2.5d'},publish:{out:'export',fit:'letterbox',aspect:'4:3',dprCap:1.5,loading:'game'},flow:{source:ref('base/dialogue/arrival'),screen:'game'}});\n";
    write("project.ts", source);
    let snapshot = || {
        og().args(["project", "src", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let first = snapshot();
    assert!(first.status.success(), "{}", text(&first));
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["name"], "港口");
    assert_eq!(value["engine"], "dev.20261126.3");
    assert_eq!(value["id"], "startup");
    assert_eq!(
        value["startup"],
        json!({
            "tick":30,"seed":83,"saveVersion":4,"scene":"harbor",
            "scenes":{"harbor":"scenes/harbor.ogt","inland":"scenes/inland.ogt"},
            "profile":"3d","loading":"game","look":"looks/harbor.ogt",
            "flow":{"dialogue":"dialogue/arrival.yarn","screen":"game"}
        })
    );
    assert_eq!(
        value["publish"],
        json!({"fit":"letterbox","aspect":"4:3","dprCap":1.5,"loading":"game"})
    );
    let dependencies = value["dependencies"].as_array().unwrap();
    for file in ["og-project.json", "project.ts", "lib/timing.ts"] {
        assert!(dependencies.contains(&json!(file)), "{value}");
    }
    assert!(value["input"].get("name").is_none());
    assert!(value["input"].get("engine").is_none());
    assert!(value["input"].get("look").is_none());
    assert_eq!(
        std::fs::read_to_string(root.join("og-project.json")).unwrap(),
        metadata
    );
    write("lib/timing.ts", "export const timing = {tick:24, seed:0};");
    let changed = snapshot();
    assert!(changed.status.success(), "{}", text(&changed));
    let changed: serde_json::Value = serde_json::from_slice(&changed.stdout).unwrap();
    assert_eq!(changed["startup"]["tick"], 24);
    assert_eq!(changed["startup"]["seed"], 0);
    write("assets/scenes/harbor.ogt", "type prefab\nversion 1\n---\n");
    let wrong_kind = snapshot();
    assert!(!wrong_kind.status.success());
    assert!(
        text(&wrong_kind).contains("expected scene, got prefab"),
        "{}",
        text(&wrong_kind)
    );
    std::fs::remove_file(root.join("assets/scenes/harbor.ogt")).unwrap();
    let missing = snapshot();
    assert!(!missing.status.success());
    assert!(
        text(&missing).contains("no scanned scene output"),
        "{}",
        text(&missing)
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'startup',entry:'src/main.ts'});",
    );
    write(
        "og-project.json",
        r#"{"version":1,"name":"Minimal","engine":"0.9.0"}"#,
    );
    let minimal = snapshot();
    assert!(minimal.status.success(), "{}", text(&minimal));
    let minimal: serde_json::Value = serde_json::from_slice(&minimal.stdout).unwrap();
    assert_eq!(minimal["startup"], json!({"profile":"3d"}));
    write(
        "project.ts",
        &source.replace("entry:'src/main.ts'", "entry:'src/main.ts',engine:'0.9.0'"),
    );
    let duplicate = snapshot();
    assert!(!duplicate.status.success());
    assert!(text(&duplicate).contains("engine"), "{}", text(&duplicate));
    write("og-project.json", "{invalid");
    let malformed = snapshot();
    assert!(!malformed.status.success());
    assert!(
        text(&malformed).contains("project_metadata_invalid"),
        "{}",
        text(&malformed)
    );
}

#[test]
fn project_types_refresh_without_merging_module_roles() {
    let root = TempDir::new("og-project-types");
    let write = |file: &str, text: &str| {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Types","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        &format!(
            "import {{defineProject}} from '@origingame/engine/author'; export default defineProject({});",
            serde_json::json!({"id":"types","entry":"src/main.ts",
        "schemas":["src/schema/base.ts","modules/north/src/schema/local.ts"],
        "modules":[{"id":"north","root":"modules/north","dependsOn":["base"]}]})
        ),
    );
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    write(
        "src/schema/base.ts",
        "import {defineComponent,u32} from '@origingame/engine/schema'; export const Base=defineComponent('game.Base',{hp:u32(83)});",
    );
    let local = "import {defineComponent,f32} from '@origingame/engine/schema'; export const Local=defineComponent('north.Local',{power:f32(7)});";
    write("modules/north/src/schema/local.ts", local);
    let base = "import {file} from '@origingame/engine/author'; throw new Error('CONTENT_EXECUTED'); export default file.scene({entities:[{name:'base',components:{'game.Base':{hp:11}}}]});";
    write("content/main.ts", base);
    let north = "import {file} from '@origingame/engine/author'; export default file.scene({entities:[{name:'north',components:{'game.Base':{hp:3},'north.Local':{power:13}}}]});";
    write("modules/north/content/main.ts", north);
    // A deliberately broad user include must not combine both generated
    // augmentations. Each generated module config isolates its declarations.
    write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"noEmit":true,"allowImportingTsExtensions":true},"include":["**/*.ts",".og/types/**/*.d.ts"]}"#,
    );
    let check = || {
        og().args(["check", "--json"])
            .current_dir(&root)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    let positive = check();
    assert!(positive.status.success(), "{}", text(&positive));
    write(
        "tools/archive.ts",
        "const separateTool: number = 'not game code';",
    );
    write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"noEmit":true,"allowImportingTsExtensions":true},"include":["src/**/*.ts",".og/types/**/*.d.ts"]}"#,
    );
    let scoped = check();
    assert!(scoped.status.success(), "{}", text(&scoped));
    for module in ["base", "north"] {
        assert!(
            root.join(format!(".og/types/author/{module}/index.d.ts"))
                .is_file()
        );
        assert!(
            root.join(format!(".og/types/author/{module}/tsconfig.json"))
                .is_file()
        );
    }
    write(
        "content/main.ts",
        &base.replace("'game.Base':{hp:11}", "'north.Local':{power:11}"),
    );
    let leaked = check();
    assert!(!leaked.status.success(), "base acquired north's records");
    let report: serde_json::Value = serde_json::from_slice(&leaked.stdout).unwrap();
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"].as_str().unwrap().contains("north.Local")),
        "{}",
        text(&leaked)
    );
    write("content/main.ts", base);
    write(
        "modules/north/src/schema/local.ts",
        &local.replace("power:", "strength:"),
    );
    let stale = check();
    assert!(
        !stale.status.success(),
        "a schema edit left stale author types"
    );
    assert!(text(&stale).contains("power"), "{}", text(&stale));
    write(
        "modules/north/content/main.ts",
        &north.replace("power:", "strength:"),
    );
    let refreshed = check();
    assert!(refreshed.status.success(), "{}", text(&refreshed));
}

/// Source-bound edits must change evaluated data without erasing code or
/// following aliases. This is the source-edit boundary used by author tooling.
#[test]
fn author_literal_edits_preserve_source_identity() {
    use og_cook::cook::generator::{
        Program,
        source::{Edit, Source},
    };
    use serde_json::json;
    let root = TempDir::new("og-author-source");
    let original = "// 原始源码\r\nconst shared = {weight: 13};\r\nexport default ({\r\n  // keep the label\r\n  label: '北', // keep trailing comment\r\n  samples: [3, -7, 11] as const,\r\n  'a/b~c': {enabled: false},\r\n  empty: {/* keep */},\r\n  shared,\r\n  computed: 5 * 2,\r\n} satisfies Record<string, unknown>);\r\n";
    std::fs::write(root.join("source.ts"), original).unwrap();
    let program = Program::prepare_value(&root, "source.ts").unwrap();
    let before = program.run().unwrap();
    let source = &program.source;
    let label = source.field("/label").unwrap();
    assert_eq!(label.value, Some(json!("北")));
    assert_eq!((label.location.line, label.location.column), (5, 10));
    assert_eq!(
        &original[label.location.start as usize..label.location.end as usize],
        "'北'"
    );
    assert!(source.field("/empty/optional").unwrap().value.is_none());
    let edits = [
        Edit {
            path: "/label".into(),
            expected: Some(before["label"].clone()),
            value: json!("南'\\\"\n"),
        },
        Edit {
            path: "/samples/1".into(),
            expected: Some(json!(-7)),
            value: json!(19),
        },
        Edit {
            path: "/a~1b~0c/enabled".into(),
            expected: Some(json!(false)),
            value: json!(true),
        },
        Edit {
            path: "/empty/optional".into(),
            expected: None,
            value: json!(null),
        },
        Edit {
            path: "/empty/other".into(),
            expected: None,
            value: json!([2, 17]),
        },
        Edit {
            path: "/new".into(),
            expected: None,
            value: json!(23),
        },
    ];
    let patched = source.patch(original, &edits).unwrap();
    assert!(patched.contains("// keep the label\r\n  label: '南"));
    assert!(patched.contains(", // keep trailing comment\r\n"));
    assert!(patched.contains("[3, 19, 11] as const"));
    assert!(patched.contains("{/* keep */"));
    assert!(patched.contains("  shared,\r\n  computed: 5 * 2,"));
    assert!(patched.contains("\r\n} satisfies Record<string, unknown>);\r\n"));
    assert!(!patched.replace("\r\n", "").contains('\n'));
    // A prepared entry retains its original bytes even after the file changes.
    std::fs::write(root.join("source.ts"), &patched).unwrap();
    assert_eq!(source.field("/label").unwrap().value, Some(json!("北")));
    assert_eq!(
        source.patch(&patched, &edits).unwrap_err().code,
        "source-conflict"
    );
    let next = Program::prepare_value(&root, "source.ts").unwrap();
    assert_ne!(source.digest(), next.source.digest());
    assert_ne!(program.key, next.key);
    let after = next.run().unwrap();
    assert_eq!(
        after,
        json!({"label": "南'\\\"\n", "samples": [3,19,11], "a/b~c": {"enabled": true}, "empty": {"optional": null,"other": [2,17]}, "shared": {"weight":13}, "computed":10, "new":23})
    );
    assert_eq!(program.run().unwrap(), before);

    let valid = Edit {
        path: "/samples/0".into(),
        expected: Some(json!(3)),
        value: json!(29),
    };
    for (bad, expected, code) in [
        ("/computed", Some(json!(10)), "computed-source"),
        ("/shared/weight", Some(json!(13)), "computed-source"),
        ("/empty", Some(json!({})), "commented-source"),
        ("/label", Some(json!("wrong")), "value-conflict"),
        ("/empty/optional", Some(json!(null)), "value-conflict"),
        ("/samples/01", Some(json!(-7)), "missing-source"),
        ("/samples/-", None, "missing-source"),
        ("/a~2b", None, "invalid-pointer"),
        ("label", None, "invalid-pointer"),
        ("/samples/0", Some(json!(3)), "overlapping-edits"),
        ("/samples", Some(json!([3, -7, 11])), "overlapping-edits"),
    ] {
        let error = source
            .patch(
                original,
                &[
                    valid.clone(),
                    Edit {
                        path: bad.into(),
                        expected,
                        value: json!(31),
                    },
                ],
            )
            .unwrap_err();
        assert_eq!(error.code, code, "{bad}: {error}");
        assert_eq!(error.file, "source.ts");
        assert_eq!(error.path, bad);
    }
    // An edit of a whole computed container must not erase its expressions.
    assert_eq!(
        source
            .patch(
                original,
                &[Edit {
                    path: "".into(),
                    expected: Some(before),
                    value: json!({})
                }]
            )
            .unwrap_err()
            .code,
        "computed-source"
    );
    for expression in [
        "{x: 1, ...{x: 2}}",
        "{['x']: 1}",
        "{x: 1, x: 2}",
        "{get x() {return 1}}",
        "{__proto__: {x: 1}}",
        "{0: 1}",
        "{x: [1, ...[2]]}",
        "{x: [1,,3]}",
    ] {
        let text = format!("export default {expression};");
        let source = Source::new("ambiguous.ts", &text);
        let path = if expression.contains('[') && expression.starts_with("{x:") {
            "/x/0"
        } else {
            "/x"
        };
        assert_eq!(
            source
                .patch(
                    &text,
                    &[Edit {
                        path: path.into(),
                        expected: Some(json!(1)),
                        value: json!(37)
                    }]
                )
                .unwrap_err()
                .code,
            "computed-source",
            "{expression}"
        );
    }
    for value in [
        json!(9007199254740993_u64),
        json!({"nested": {"__proto__": {"x": 1}}}),
    ] {
        assert_eq!(
            source
                .patch(
                    original,
                    &[Edit {
                        path: "/samples/0".into(),
                        expected: Some(json!(3)),
                        value,
                    }]
                )
                .unwrap_err()
                .code,
            "invalid-value"
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("source.ts")).unwrap(),
        patched
    );
    // Only a real trailing comma counts; commas in comments are trivia.
    for text in [
        "export default {};",
        "export default {x: 1 /* , */};",
        "export default {x: 1 // ,\n};",
        "export default {x: 1, // ,\n};",
    ] {
        std::fs::write(root.join("insert.ts"), text).unwrap();
        let program = Program::prepare_value(&root, "insert.ts").unwrap();
        let patched = program
            .source
            .patch(
                text,
                &[
                    Edit {
                        path: "/a".into(),
                        expected: None,
                        value: json!(17),
                    },
                    Edit {
                        path: "/b".into(),
                        expected: None,
                        value: json!(false),
                    },
                ],
            )
            .unwrap();
        std::fs::write(root.join("insert.ts"), patched).unwrap();
        let after = Program::prepare_value(&root, "insert.ts")
            .unwrap()
            .run()
            .unwrap();
        let expected = if text == "export default {};" {
            json!({"a":17,"b":false})
        } else {
            json!({"x":1,"a":17,"b":false})
        };
        assert_eq!(after, expected);
    }
    let text = "export default {x: [0x10, -2, 3.5], nested: {a: 0b11}};";
    let patched = Source::new("containers.ts", text)
        .patch(
            text,
            &[
                Edit {
                    path: "/x".into(),
                    expected: Some(json!([16, -2, 3.5])),
                    value: json!([7, 19]),
                },
                Edit {
                    path: "/nested".into(),
                    expected: Some(json!({"a":3})),
                    value: json!({"b":13}),
                },
            ],
        )
        .unwrap();
    std::fs::write(root.join("containers.ts"), patched).unwrap();
    assert_eq!(
        Program::prepare_value(&root, "containers.ts")
            .unwrap()
            .run()
            .unwrap(),
        json!({"x":[7,19],"nested":{"b":13}})
    );
}

/// Generated effects use the public cook/load contracts, including virtual
/// array links; uncached deterministic volume data and cache hits agree.
#[test]
fn generated_effect_sources_cook_and_link() {
    let root = TempDir::new("og-generated-effects");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Effects","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject,ref} from '@origingame/engine/author'; export default defineProject({id:'effects',entry:'src/main.ts',roots:[ref('base/volume#array'),ref('base/volume#emission#array#layer2')]});").unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    let volume = r#"import {file} from '@origingame/engine/author';
export default file['effect-texture']({
    mode:'explosion', size:8, frames:3, columns:2, grid:8, steps:1,
    dt:0.04, pressure:4, seed:17, frequency:3, turbulence:0.03,
    source:[0.43,0.38,0.5], radius:0.3, inject:0, density:12, heat:1,
    buoyancy:0.1, expansion:0.4, decay:0.5, cooling:2, absorption:8,
    color:[0.4,0.45,0.5], emission:[1,0.2,0.01], range:1, detail:0.8,
    vorticity:2, light:[-1,2,-1], ambient:0.2, direct:1,
});"#;
    std::fs::write(root.join("content/volume.ts"), volume).unwrap();
    for (name, ty, body) in [
        (
            "surface",
            "effect",
            "main \"base/volume#emission\"\nblend additive\n",
        ),
        (
            "emitter",
            "particles",
            "maxParticles 1\nrate 0\nburst 0 1\nrender sprite\narray \"base/volume#array\" 3 life 1\neffect \"surface.ogt\"\n",
        ),
        (
            "composed",
            "vfx",
            "duration 0.75\nemitter \"emitter.ogt\" {\nat 0\n}\n",
        ),
    ] {
        std::fs::write(
            assets.join(format!("{name}.ogt")),
            format!(
                "type {ty}\nversion {}\n---\ngenerate {{\nscript \"{name}.ts\"\n}}\n",
                if ty == "vfx" { 2 } else { 1 }
            ),
        )
        .unwrap();
        std::fs::write(
            assets.join(format!("{name}.ts")),
            format!(
                "export default () => ({{text: {}}});",
                serde_json::to_string(body).unwrap()
            ),
        )
        .unwrap();
    }
    let cook = |cache: &str| {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook("off");
    assert!(out.status.success(), "{}", text(&out));
    let load = || og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let bundle = load();
    let sheet = bundle.load_texture("base/volume").unwrap();
    assert_eq!((sheet.meta.width, sheet.meta.height), (16, 16));
    let array = bundle.load_texture_array("base/volume#array").unwrap();
    assert_eq!(
        (array.meta.width, array.meta.height, array.layers.len()),
        (8, 8, 3)
    );
    assert_eq!(array.layers[2], "base/volume#array#layer2");
    assert!(bundle.load_texture("base/volume#emission").is_ok());
    let bytes = bundle.read("composed.ogt").unwrap();
    let vfx = og_cook::vfx::parse_vfx(&bundle.vfx_text("composed.ogt", &bytes).unwrap()).unwrap();
    assert_eq!(vfx.duration, 0.75);
    assert_eq!(vfx.references().emitters, ["emitter.ogt"]);
    let first = bundle.read("base/volume#array#layer1").unwrap().to_vec();
    let pixels = bundle
        .load_texture("base/volume#array#layer1")
        .unwrap()
        .ktx2;
    // Remove only this test's disposable cooked output, forcing a cold
    // generation. Then exercise population and reuse of its private cache.
    for cache in [
        "off".to_string(),
        root.join("cache").to_string_lossy().into_owned(),
        root.join("cache").to_string_lossy().into_owned(),
    ] {
        std::fs::remove_dir_all(root.join("cooked")).unwrap();
        let out = cook(&cache);
        assert!(out.status.success(), "{}", text(&out));
        assert_eq!(&*load().read("base/volume#array#layer1").unwrap(), first);
    }
    std::fs::write(
        root.join("content/volume.ts"),
        volume.replace("seed:17", "seed:29"),
    )
    .unwrap();
    assert!(cook("off").status.success());
    assert_ne!(
        load()
            .load_texture("base/volume#array#layer1")
            .unwrap()
            .ktx2,
        pixels
    );
    // Only the producer's real outputs may be referenced, not guessed suffixes.
    std::fs::write(
        root.join("content/volume.ts"),
        volume.replace("frames:3", "frames:2"),
    )
    .unwrap();
    let out = cook("off");
    assert!(!out.status.success(), "nonexistent derived layer accepted");
    assert!(
        text(&out).contains("base/volume#emission#array#layer2"),
        "{}",
        text(&out)
    );
    std::fs::write(root.join("content/volume.ts"), volume).unwrap();
    std::fs::write(
        assets.join("volume.ogt"),
        "type effect-texture\nversion 1\n---\n",
    )
    .unwrap();
    let out = cook("off");
    assert!(!out.status.success(), "removed OGT author entry accepted");
    assert!(
        text(&out).contains("OGT authoring is removed"),
        "{}",
        text(&out)
    );
    std::fs::remove_file(assets.join("volume.ogt")).unwrap();
    // The same ordinary target parser must reject malformed generated text.
    std::fs::write(
        assets.join("surface.ts"),
        "export default () => ({text:'notAnEffectField 3'});",
    )
    .unwrap();
    let out = cook("off");
    assert!(!out.status.success(), "invalid generated material accepted");
}

/// A graph asset resolves clip names at cook, keeps bindings in its
/// typed blob, and revalidates them when the source changes.
#[test]
fn animation_graph_cooks_with_tracked_clip_bindings() {
    use og_anim::cooked::BundleAnim;
    let root = TempDir::new("og-generated-graph");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Graph","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'graph',entry:'src/main.ts'});").unwrap();
    let model =
        std::fs::read(repo().join("tests/fixtures/anim-markers/assets/models/walker.glb")).unwrap();
    std::fs::write(assets.join("walker.glb"), &model).unwrap();
    std::fs::write(
        assets.join("walker.glb.ogt"),
        "type import\nversion 1\n---\nconform off\n",
    )
    .unwrap();
    let graph = "type animgraph\nversion 1\n---\nbind \"stride\" \"walker.glb\" \"walk\"\nclip moving \"stride\"\nlayer base {\nstate active moving\nentry active\n}\n";
    std::fs::write(assets.join("graph.ogt"), graph).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let source = bundle
        .animgraph("graph.ogt", &bundle.read("graph.ogt").unwrap())
        .unwrap();
    assert!(!source.sprite);
    assert_eq!(
        (
            source.bindings[0].name.as_str(),
            source.bindings[0].clip.as_str()
        ),
        ("stride", "walk")
    );
    let authored = "type import\nversion 1\n---\nconform off\nanimation {\nrate 4\ncompress true\nclip \"designed\" {\nchannel \"hips\" translation {\ninterpolation cubicSpline\nkey 0 0 0 0 2 3 5 8 0 0\nkey 1 0 0 0 6 7 9 0 0 0\n}\nchannel \"body\" visibility {\ninterpolation step\nkey 0 1\nkey 0.5 0\nkey 1 1\n}\n}\nevent \"designed\" \"half\" 0.5\n}\n";
    std::fs::write(assets.join("walker.glb.ogt"), authored).unwrap();
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("\"walk\"", "\"designed\""),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let animation = bundle.load_animation("walker.glb").unwrap();
    let clip = animation.clip("designed").unwrap();
    let position = &clip.channels[0];
    // Hermite at t=0.5: midpoint plus 1/8 of the first out-tangent.
    for (lane, expected) in [5.0, 5.0, 7.0].into_iter().enumerate() {
        assert!((position.lane(2, lane) - expected).abs() < 0.001);
    }
    assert_eq!(clip.channels[1].lane(1, 0), 1.0);
    assert_eq!(clip.channels[1].lane(2, 0), 0.0);
    assert_eq!(
        (clip.events[0].name.as_str(), clip.events[0].time),
        ("half", 0.5)
    );
    let cold = bundle.read("walker.glb#anim").unwrap();
    assert!(cook().status.success());
    assert_eq!(
        &*og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .read("walker.glb#anim")
            .unwrap(),
        &*cold
    );
    // Reject unknown nodes, then prove changing only authored keys invalidates output.
    std::fs::write(
        assets.join("walker.glb.ogt"),
        authored.replace("\"hips\"", "\"absent\""),
    )
    .unwrap();
    assert!(!cook().status.success());
    std::fs::write(
        assets.join("walker.glb.ogt"),
        authored.replace("6 7 9", "10 7 9"),
    )
    .unwrap();
    assert!(cook().status.success());
    let changed = og_cook::runtime::Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_animation("walker.glb")
        .unwrap();
    assert!((changed.clip("designed").unwrap().channels[0].lane(2, 0) - 7.0).abs() < 0.001);
    std::fs::write(
        assets.join("walker.glb.ogt"),
        "type import\nversion 1\n---\nconform off\n",
    )
    .unwrap();
    std::fs::write(assets.join("graph.ogt"), graph).unwrap();
    let mut renamed = model;
    for i in 0..renamed.len() - 3 {
        if &renamed[i..i + 4] == b"walk" {
            renamed[i..i + 4].copy_from_slice(b"hike");
        }
    }
    std::fs::write(assets.join("walker.glb"), renamed).unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(text(&out).contains("has no clip 'walk'"), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("\"walk\"", "\"hike\""),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("bind \"stride\" \"walker.glb\" \"walk\"", "sprite true"),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("state active moving", "state active absent"),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success(), "invalid graph accepted");
}

/// Public lattice cook/load and emitter consumption retain channel order,
/// float precision and tracked generator dependencies.
#[test]
fn generated_fields_load_losslessly_and_feed_particles() {
    let root = TempDir::new("og-generated-fields");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Field","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'field',entry:'src/main.ts'});").unwrap();
    std::fs::write(
        assets.join("source.ogt"),
        "type field\nversion 1\n---\ngenerate {\nscript \"field.ts\"\n}\n",
    )
    .unwrap();
    std::fs::write(assets.join("field.ts"), "import {input} from '@origingame/engine/author'; export default () => {const amplitude = input.bytes('input.bin')[0]; return {dimensions: [2, 2, 2], channels: 3, origin: [-3, 2, 7], spacing: [0.5, 2, 3], values: Float32Array.from({length: 24}, (_, i) => amplitude + i / 32)};};").unwrap();
    std::fs::write(assets.join("input.bin"), [7]).unwrap();
    std::fs::write(
        assets.join("particles.ogt"),
        "type particles\nversion 1\n---\nfield \"source.ogt\" 0.25 9\n",
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for amplitude in [7, 11] {
        std::fs::write(assets.join("input.bin"), [amplitude]).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let field = bundle.load_field("source.ogt").unwrap();
        assert_eq!(field.dimensions, [2, 2, 2]);
        assert_eq!(field.channels, 3);
        assert_eq!(field.origin, [-3.0, 2.0, 7.0]);
        assert_eq!(field.spacing, [0.5, 2.0, 3.0]);
        let expected: Vec<f32> = (0..24)
            .map(|i| f32::from(amplitude) + i as f32 / 32.0)
            .collect();
        assert_eq!(field.values, expected);
        let bytes = bundle.read("particles.ogt").unwrap();
        let text = bundle.emitter_text("particles.ogt", &bytes).unwrap();
        let emitter = og_cook::fx::parse_emitter(&text, &mut |_| Ok(None)).unwrap();
        let og_render_data::VectorField::Grid {
            strength,
            size,
            cells,
            vectors,
        } = emitter.field.unwrap()
        else {
            panic!("not a grid")
        };
        assert_eq!((strength, size, cells), (0.25, 9.0, 2));
        assert_eq!(
            vectors
                .into_iter()
                .flat_map(|v| v.to_array())
                .collect::<Vec<_>>(),
            expected
        );
    }
    // A rectangular scalar lattice is valid by itself, not a particle grid.
    std::fs::remove_file(assets.join("particles.ogt")).unwrap();
    std::fs::write(assets.join("field.ts"), "export default () => ({dimensions: [3, 2, 1], channels: 1, values: [117, -2, 0.125, 4, 9, 0]});").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.load_field("source.ogt").unwrap().values,
        [117.0, -2.0, 0.125, 4.0, 9.0, 0.0]
    );
    for invalid in [
        "({dimensions: [3, 2, 1], channels: 1, values: [1, 2]})",
        "({dimensions: [0, 2, 1], channels: 1, values: []})",
        "({dimensions: [1, 1, 1], channels: 1, spacing: [1, 0, 1], values: [1]})",
        "({dimensions: [1, 1, 1], channels: 5, values: [1, 2, 3, 4, 5]})",
    ] {
        std::fs::write(
            assets.join("field.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}

/// The public offline executor: dependencies and parameters invalidate a
/// content key, moving a project does not, and ambient authority is absent.
#[test]
fn script_tracks_its_closed_inputs_and_rejects_ambient_authority() {
    let root = TempDir::new("og-script");
    let run = |name: &str, params: &str| {
        og().args([
            "script",
            name,
            "--params",
            params,
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap()
    };
    let value = |out: &Output| -> serde_json::Value {
        assert!(out.status.success(), "{}", text(out));
        serde_json::from_slice(&out.stdout).unwrap()
    };
    std::fs::write(root.join("input.bin"), [7, 11]).unwrap();
    std::fs::write(root.join("shape.ts"), "export const offset: number = 3;\n").unwrap();
    std::fs::write(root.join("main.ts"), r#"
        import {offset} from './shape.ts';
        export default ({params, inputs}: {params: {scale: number}; inputs: Record<string, number[]>}) =>
            new Float32Array([offset * params.scale, inputs.bytes[1], Math.sin(0.7), 2 ** 0.3]);
    "#).unwrap();
    let first = value(&run("main.ts", "{\"scale\":2}"));
    assert_eq!(first["data"][0], 6.0);
    assert_eq!(first["data"][1], 11.0);
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2}")));
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2.0}")));
    assert_ne!(first["key"], value(&run("main.ts", "{\"scale\":4}"))["key"]);
    std::fs::write(root.join("shape.ts"), "export const offset: number = 5;\n").unwrap();
    let changed = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(first["key"], changed["key"]);
    assert_eq!(changed["data"][0], 10.0);
    std::fs::write(root.join("input.bin"), [7, 13]).unwrap();
    let input = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(changed["key"], input["key"]);
    assert_eq!(input["data"][1], 13.0);
    let moved = TempDir::new("og-script-moved");
    for file in ["main.ts", "shape.ts", "input.bin"] {
        std::fs::copy(root.join(file), moved.join(file)).unwrap();
    }
    let output = og()
        .args([
            "script",
            "main.ts",
            "--params",
            "{\"scale\":2}",
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&moved)
        .output()
        .unwrap();
    assert_eq!(input, value(&output));

    // Type declarations have no executable dependency in the cook realm.
    std::fs::write(
        root.join("types.d.ts"),
        "export interface Point { x: number }\n",
    )
    .unwrap();
    std::fs::write(root.join("data.ts"), r#"
        import type {Point} from './types.d.ts';
        export default () => {
            const point: Point = {x: 7};
            const shared = {x: point.x};
            const record = Object.assign(Object.create(null), {z: null, a: false});
            return {second: shared, first: shared, record, samples: new Float32Array([3, -7, 11]).subarray(1)};
        };
    "#).unwrap();
    let data = value(&run("data.ts", "{}"));
    assert_eq!(
        data["data"],
        serde_json::json!({
            "first": {"x": 7}, "second": {"x": 7},
            "record": {"a": false, "z": null}, "samples": [-7, 11]
        })
    );
    std::fs::remove_file(root.join("types.d.ts")).unwrap();
    assert_eq!(data, value(&run("data.ts", "{}")));

    std::fs::write(root.join("mechanisms.ts"), r#"
        import {procgen} from '@origingame/engine/generate';
        export default () => {
            const h = procgen.heightmap({columns: 3, rows: 2, size: {x: 11, y: 7}, heights: new Float32Array([117, 119, 123, 127, 131, 137])});
            const w = procgen.wfc({width: 3, height: 1, tiles: [
                {weight: 1, up: 0, right: 1, down: 0, left: 2, rotations: false},
                {weight: 1, up: 0, right: 2, down: 0, left: 1, rotations: false},
            ], fixed: [{x: 0, y: 0, tile: 0, quarterTurns: 0}]});
            if (!h.ok || !w.ok) throw new Error('mechanism refused');
            if (!(h.value.heights instanceof Float32Array) || !(w.value.tiles instanceof Uint32Array)) throw new Error('lost typed buffers');
            return {heights: h.value.heights.subarray(1, 5), tiles: w.value.tiles};
        };
    "#).unwrap();
    let mechanisms = value(&run("mechanisms.ts", "{}"));
    assert_eq!(
        mechanisms["data"]["heights"],
        serde_json::json!([119, 123, 127, 131])
    );
    assert_eq!(mechanisms["data"]["tiles"], serde_json::json!([0, 1, 0]));

    std::fs::write(root.join("channels.ts"), r#"
        import {procgen} from '@origingame/engine/generate';
        export default () => {
            const heights = Float32Array.from({length:15}, (_, i) => 100+i);
            const preserve = new Float32Array(15);
            preserve[7] = 0.5; preserve[8] = 1;
            const desc = {columns:5, rows:3, size:{x:8,y:4}, heights, preserve, preserveBorder:1,
                channels:[{points:[{x:-4,y:80,z:0},{x:4,y:72,z:0}],width:4,depth:2,river:true}]};
            const result = procgen.heightmap(desc);
            if (!result.ok) throw new Error(result.reason);
            const rising = procgen.heightmap({...desc, channels:[{...desc.channels[0],points:[{x:0,y:0,z:0},{x:1,y:1,z:0}]}]});
            const malformed = procgen.heightmap({...desc, preserve:new Float32Array(14)});
            return {heights:result.value.heights, length:result.value.riverLength, rising:rising.ok, malformed:malformed.ok};
        };
    "#).unwrap();
    let channels = value(&run("channels.ts", "{}"));
    assert_eq!(
        channels["data"]["heights"],
        serde_json::json!([
            100, 101, 102, 103, 104, 105, 76, 90.5, 108, 109, 110, 111, 112, 113, 114
        ])
    );
    assert_eq!(channels["data"]["length"], 8);
    assert_eq!(channels["data"]["rising"], false);
    assert_eq!(channels["data"]["malformed"], false);

    std::fs::write(root.join("constraints.ts"), r#"
        import {procgen} from '@origingame/engine/generate';
        export default () => {
            const require = (v, message) => {if (!v) throw new Error(message)};
            const tiles = [0,1].map(() => ({weight:1,up:0,right:0,down:0,left:0,rotations:false}));
            const w = procgen.wfc({width:2,height:2,tiles,allowed:[[1],[0],[0,1],[1]]});
            require(w.ok && w.value.tiles[0] === 1 && w.value.tiles[1] === 0 && w.value.tiles[3] === 1, 'allowed WFC domains');
            require(!procgen.wfc({width:2,height:2,tiles,allowed:[[1],[0],[0,1],[1]],fixed:[{x:0,y:0,tile:0}]}).ok, 'conflicting fixed cell must fail');
            require(!procgen.wfc({width:1,height:1,tiles,allowed:[[]]}).ok, 'empty domain must fail');
            const polygon = (x0,y0,x1,y1) => [{x:x0,y:y0},{x:x1,y:y0},{x:x1,y:y1},{x:x0,y:y1}];
            const retained = [{x:.5,y:.5},{x:6,y:.5}];
            const p = procgen.poissonDisk({seed:13,width:8,height:6,radius:.7,region:polygon(.25,.25,7.75,5.75),holes:[polygon(.5,2,1.5,3)],points:retained,columns:4,rows:1,density:new Float32Array([1,0,1,1]),radii:new Float32Array([.7,1,1.4,2]),maxPoints:100});
            require(p.ok && p.value.length > 5, 'constrained Poisson must place points');
            require(p.value.slice(0,2).every((p,i) => p.x === retained[i].x && p.y === retained[i].y), 'retained point ordering');
            const radius = p => [.7,1,1.4,2][Math.min(3,Math.floor(p.x/2))];
            for (let i=0;i<p.value.length;i++) {
                const a = p.value[i];
                require(a.x>=.25 && a.x<7.75 && a.y>=.25 && a.y<5.75 && !(a.x>=2 && a.x<4) && !(a.x>=.5 && a.x<1.5 && a.y>=2 && a.y<3), 'region, hole and density must constrain samples');
                for(let j=0;j<i;j++) {const b=p.value[j]; require(Math.hypot(a.x-b.x,a.y-b.y)+1e-6>=Math.max(radius(a),radius(b)), 'symmetric variable separation');}
            }
            require(!procgen.poissonDisk({width:8,height:6,radius:1,points:[{x:1,y:1},{x:1.1,y:1}]}).ok, 'overlapping retained points must fail');
            const mask = Uint8Array.from({length:14*9}, (_,i) => i%14===7 && Math.floor(i/14)!==5 ? 0 : 1);
            const rooms = [{x:1,y:1,width:2,height:2},{x:10,y:1,width:2,height:3}];
            const b = procgen.bsp({seed:5,width:14,height:9,minRoom:2,maxRoom:3,depth:2,padding:1,corridor:1,mask,rooms});
            require(b.ok && rooms.every((r,i) => Object.entries(r).every(([k,v]) => b.value.rooms[i][k]===v)), 'BSP retained rooms');
            require(b.value.cells.every((v,i) => !v || mask[i]), 'BSP must not carve exclusions');
            const floor = Array.from(b.value.cells).findIndex(v=>v!==0), seen=new Set([floor]), queue=[floor];
            for(let k=0;k<queue.length;k++) {const i=queue[k]; for(const j of [i-14,i+14,...(i%14 ? [i-1]:[]),...(i%14<13 ? [i+1]:[])]) if(j>=0 && j<126 && b.value.cells[j] && !seen.has(j)){seen.add(j);queue.push(j);}}
            require(seen.size===b.value.cells.reduce((n,v)=>n+Number(v!==0),0), 'every BSP floor must remain connected');
            mask[5*14+7]=0;
            require(!procgen.bsp({width:14,height:9,minRoom:2,maxRoom:3,depth:2,padding:1,corridor:1,mask,rooms}).ok, 'disconnected mask must fail');
            const f = procgen.fracture({positions:new Float32Array([0,0,0,2,0,0,0,2,0,0,0,2]),indices:new Uint32Array([0,2,1,0,1,3,0,3,2,1,2,3]),sites:[{x:.2,y:.2,z:.2},{x:1,y:.2,z:.2}]});
            require(f.ok && f.value.pieces.length===2 && Math.abs(f.value.pieces.reduce((n,p)=>n+p.volume,0)-4/3)<1e-5, 'authored fracture sites must partition the original volume');
            const sites = [{x:3,y:0,z:0},{x:0,y:4,z:3}];
            const flat = procgen.worley(0,0,0,0,false,sites), solid=procgen.worley(0,0,0,0,true,sites);
            require(flat.x===3 && flat.y===4 && solid.x===3 && solid.y===5, 'authored Worley distances must use declared dimensions');
            const path=[{x:-2,y:0,z:1},{x:-1,y:2,z:1},{x:1,y:3,z:-1}];
            const plant=procgen.plant({levels:0,path,levelsSpec:[{segments:4}],leaves:{count:1,start:.5,end:.5,sizeVariance:0},leafMesh:{positions:new Float32Array([0,0,0,1,0,0,0,1,0]),uvs:new Float32Array([0,0,1,0,0,1]),colors:new Float32Array([1,1,1,.25,1,1,1,.5,1,1,1,.75])}});
            require(plant.ok && plant.value.leafCount===1 && plant.value.leaves.indices.length===3, 'custom foliage must replace the primitive');
            for(let ring=0;ring<3;ring++) for(let axis=0;axis<3;axis++) {
                let centre=0;for(let v=0;v<4;v++)centre+=plant.value.wood.positions[(ring*5+v)*3+axis]/4;
                require(Math.abs(centre-path[ring][['x','y','z'][axis]])<1e-5,'authored trunk ring centre');
            }
            require(plant.value.leaves.colors[3]===.25 && plant.value.leaves.colors[7]===.5 && plant.value.leaves.colors[11]===.75, 'custom foliage alpha must survive');
            return {verified:true};
        };
    "#).unwrap();
    assert_eq!(
        value(&run("constraints.ts", "{}"))["data"]["verified"],
        true
    );

    let exported = og()
        .args([
            "script",
            "mechanisms.ts",
            "--out",
            "out/nested/result.json",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(value(&exported)["data"], serde_json::Value::Null);
    let published = std::fs::read(root.join("out/nested/result.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&published).unwrap(),
        mechanisms["data"]
    );

    for code in [
        "export default () => Date.now()",
        "export default () => Math.random()",
        "export default () => fetch('https://example.invalid')",
        "export default () => process.env.HOME",
        "import fs from 'node:fs'; export default () => fs.readFileSync('/etc/passwd')",
        "export default () => (()=>{}).constructor('return 1')()",
        "export default () => eval('2 ** 0.3')",
        "export default async () => [1]",
        "export default () => [Infinity]",
        "export default () => ({nested: {missing: undefined}})",
        "export default () => [undefined]",
        "export default () => ({nested: () => 1})",
        "export default () => ({nested: Symbol('lost')})",
        "export default () => ({nested: 1n})",
        "export default () => ({nested: new Map([['lost', 7]])})",
        "export default () => ({nested: new Set([7])})",
        "export default () => ({nested: new (class {x = 7})()})",
        "export default () => ({nested: Promise.resolve(7)})",
        "export default () => ({nested: new DataView(new ArrayBuffer(8))})",
        "export default () => new (class extends Float32Array {})([7])",
        "export default () => Object.assign(new Float32Array([7]), {lost: 11})",
        "export default () => ({nested: {[Symbol('lost')]: 7}})",
        "export default () => ({nested: {get x() {return 7}}})",
        "export default () => ({nested: {toJSON() {return 7}}})",
        "export default () => ({nested: Object.defineProperty({}, 'lost', {value: 7})})",
        "export default () => {const a = {}; a.self = a; return a}",
        "export default () => Array(2)",
        "export default () => Object.assign([7], {lost: 11})",
        "export default () => {if (false) import('./shape.ts'); return 7}",
        "await Promise.resolve(); export default () => 7",
        "for await (const x of []) {} export default () => 7",
        "export default () => { while (true) {} }",
    ] {
        std::fs::write(root.join("denied.ts"), code).unwrap();
        let out = run("denied.ts", "{}");
        assert!(
            !out.status.success(),
            "accepted forbidden generator {code}: {}",
            text(&out)
        );
        assert!(out.stdout.is_empty(), "partial result for {code}");
    }
    std::fs::write(
        root.join("denied.ts"),
        "export default () => ({nested: {missing: undefined}})",
    )
    .unwrap();
    assert!(text(&run("denied.ts", "{}")).contains("denied.ts: Error: $[\"nested\"][\"missing\"]"));
    std::fs::write(
        root.join("denied.ts"),
        "// source position\nexport default () => import('./shape.ts')",
    )
    .unwrap();
    assert!(text(&run("denied.ts", "{}")).contains("denied.ts:2:22: dynamic import"));
    let refused = og()
        .args([
            "script",
            "denied.ts",
            "--out",
            "out/nested/result.json",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert_eq!(
        std::fs::read(root.join("out/nested/result.json")).unwrap(),
        published
    );
    let out = run("../outside.ts", "{}");
    assert!(!out.status.success());
    assert!(text(&out).contains("leaves its input root"));

    // A descriptor is evaluated, never called, in the same closed realm.
    use og_cook::cook::generator::{Program, Spec};
    let authored = "import {offset} from './shape.ts'; globalThis.count = (globalThis.count ?? 0) + 1; export default {offset, count: globalThis.count, samples: new Float32Array([3, -7])};";
    std::fs::write(root.join("value.ts"), authored).unwrap();
    let first = Program::prepare_value(&root, "value.ts").unwrap();
    let expected = serde_json::json!({"offset": 5, "count": 1, "samples": [3, -7]});
    assert_eq!(first.run().unwrap(), expected);
    assert_eq!(first.run().unwrap(), expected);
    std::fs::write(root.join("other.ts"), authored).unwrap();
    assert_eq!(
        Program::prepare_value(&root, "other.ts")
            .unwrap()
            .run()
            .unwrap(),
        expected
    );
    std::fs::copy(root.join("value.ts"), moved.join("value.ts")).unwrap();
    assert_eq!(
        first.key,
        Program::prepare_value(&moved, "value.ts").unwrap().key
    );
    std::fs::write(root.join("shape.ts"), "export const offset = 19;").unwrap();
    let changed = Program::prepare_value(&root, "value.ts").unwrap();
    assert_ne!(first.key, changed.key);
    assert_eq!(changed.run().unwrap()["offset"], 19);

    // Warming the callable cache must not let a function pass as source data.
    std::fs::write(root.join("call.ts"), "export default () => ({x: 7});").unwrap();
    let callable = Program::prepare(
        &root,
        "",
        &Spec {
            script: "call.ts".into(),
            params: serde_json::json!({}),
            seed: 0,
            inputs: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(callable.run().unwrap(), serde_json::json!({"x": 7}));
    let data = Program::prepare_value(&root, "call.ts").unwrap();
    assert_ne!(callable.key, data.key);
    assert!(data.run().is_err());
}

/// Author streams use the engine sequence without shared draw state, including
/// forks, helper imports, separate realms and warm action-cache results.
#[test]
fn author_rng_is_explicit_and_independent() {
    use og_cook::cook::generator::Program;
    use serde_json::json;
    let root = TempDir::new("og-author-rng");
    std::fs::write(root.join("helper.ts"), "import {rng as create} from '@origingame/engine/author'; export const first = (seed: number) => create(seed).next();").unwrap();
    let source = r#"
        import * as og from '@origingame/engine/author';
        import {first} from './helper.ts';
        const parent = og.rng(42);
        const before = parent.fork('区域🦊');
        const one = parent.next();
        const childOne = before.next();
        const after = parent.fork('区域🦊');
        export default {
            seed: parent.seed,
            draws: [one, parent.next(), parent.next()],
            fork: [before.seed, childOne, before.next(), after.next(), after.next()],
            other: parent.fork('other').next(),
            independent: first(42),
            zero: first(0),
            max: first(4294967295),
            maxSeed: og.rng(4294967295).seed,
        };
    "#;
    std::fs::write(root.join("first.ts"), source).unwrap();
    std::fs::write(root.join("second.ts"), source).unwrap();
    let first = Program::prepare_value(&root, "first.ts").unwrap();
    let second = Program::prepare_value(&root, "second.ts").unwrap();
    // Main stream values are the existing Web substrate vectors; fork values
    // were independently computed with UTF-16 units, not UTF-8 or codepoints.
    let expected = json!({
        "seed": 42,
        "draws": [0.09396462352015078, 0.5757808552589267, 0.19534794357605278],
        "fork": [2981131337_u32, 0.7657625777646899, 0.4455386148765683,
            0.7657625777646899, 0.4455386148765683],
        "other": 0.7984659534413368,
        "independent": 0.09396462352015078,
        "zero": 0.479614513926208,
        "max": 0.3764090109616518,
        "maxSeed": 4294967295_u32,
    });
    std::thread::scope(|scope| {
        let a = scope.spawn(|| first.run().unwrap());
        let b = scope.spawn(|| second.run().unwrap());
        assert_eq!(a.join().unwrap(), expected);
        assert_eq!(b.join().unwrap(), expected);
    });
    assert_eq!(first.run().unwrap(), expected);

    for expression in [
        "rng()",
        "rng(-1)",
        "rng(4294967296)",
        "rng(0.25)",
        "rng(NaN)",
        "rng(Infinity)",
        "rng('42')",
        "rng(null)",
        "rng(true)",
        "rng(42).fork(7)",
        "rng(42)",
        "Math.random()",
    ] {
        std::fs::write(
            root.join("invalid.ts"),
            format!(
                "import {{rng}} from '@origingame/engine/author'; export default {expression};"
            ),
        )
        .unwrap();
        assert!(
            Program::prepare_value(&root, "invalid.ts")
                .unwrap()
                .run()
                .is_err(),
            "accepted {expression}"
        );
    }
    // A TS assertion cannot bypass the numeric check above. The declaration
    // separately catches missing/non-number seeds, wrong labels and mutation.
    std::fs::write(
        root.join("contract.ts"),
        format!(
            r#"
        import {{rng, type Rng}} from {};
        const stream: Rng = rng(42).fork('区域🦊');
        const value: number = stream.next();
        const seed: number = stream.seed;
        // @ts-expect-error A seed is mandatory.
        rng();
        // @ts-expect-error No string coercion.
        rng('42');
        // @ts-expect-error Labels are strings.
        stream.fork(7);
        // @ts-expect-error The seed is immutable.
        stream.seed = 7;
        // @ts-expect-error The draw operation is immutable.
        stream.next = () => 0;
    "#,
            json!(repo().join("packages/engine/src/author.js"))
        ),
    )
    .unwrap();
    let check = Command::new("tsrs")
        .args([
            "--ignoreConfig",
            "--strict",
            "--skipLibCheck",
            "--noEmit",
            "--target",
            "esnext",
            "--module",
            "esnext",
            "--moduleResolution",
            "bundler",
            "contract.ts",
        ])
        .current_dir(&root)
        .output()
        .expect("the pinned tsrs checker must be installed");
    assert!(check.status.success(), "{}", text(&check));
}

/// The author input capability snapshots only literal, tracked dependencies;
/// helpers, cache hits and separate realms observe the same closed inputs.
#[test]
fn author_inputs_are_captured_before_evaluation() {
    use og_cook::cook::generator::Program;
    let root = TempDir::new("og-author-inputs");
    let moved = TempDir::new("og-author-inputs-moved");
    let helper = "import {input as source} from '@origingame/engine/author'; export {procgen as pure} from '@origingame/engine/author'; export const lines = () => source.text('lines.csv'); export const local = (input: {bytes(p: string): number}) => input.bytes('not-a-file');";
    let entry = "import * as og from '@origingame/engine/author'; import {lines, local} from './lib/helper.ts'; const bytes = og.input.bytes('./samples.bin'); bytes[0] = 99; export default {bytes: og.input.bytes('samples.bin'), lines: lines(), local: local({bytes: () => 37}), distance: og.procgen.worley(0, 0, 0, 0, false, [{x:3,y:4,z:0},{x:8,y:15,z:0}]).x};";
    let expected = serde_json::json!({"bytes": [5, 191, 7, 255], "lines": "\u{feff}北,17\r\n", "local": 37, "distance": 5});
    for dir in [&root, &moved] {
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::write(dir.join("lib/helper.ts"), helper).unwrap();
        std::fs::write(dir.join("value.ts"), entry).unwrap();
        std::fs::write(dir.join("samples.bin"), [5, 191, 7, 255]).unwrap();
        std::fs::write(dir.join("lines.csv"), "\u{feff}北,17\r\n").unwrap();
    }
    let first = Program::prepare_value(&root, "value.ts").unwrap();
    assert_eq!(
        first.dependencies,
        ["lib/helper.ts", "lines.csv", "samples.bin", "value.ts"]
    );
    // Even removing the files cannot make an already prepared realm do I/O.
    std::fs::remove_file(root.join("samples.bin")).unwrap();
    assert_eq!(first.run().unwrap(), expected);
    assert_eq!(first.run().unwrap(), expected);
    let relocated = Program::prepare_value(&moved, "value.ts").unwrap();
    assert_eq!(first.key, relocated.key);
    assert_eq!(relocated.run().unwrap(), expected);
    std::fs::write(moved.join("unrelated.bin"), [81]).unwrap();
    assert_eq!(
        relocated.key,
        Program::prepare_value(&moved, "value.ts").unwrap().key
    );
    std::fs::write(moved.join("samples.bin"), [13, 191, 7, 255]).unwrap();
    let changed = Program::prepare_value(&moved, "value.ts").unwrap();
    assert_ne!(changed.key, first.key);
    assert_eq!(
        changed.run().unwrap()["bytes"],
        serde_json::json!([13, 191, 7, 255])
    );
    std::fs::write(moved.join("lines.csv"), "南,29\n").unwrap();
    let changed_text = Program::prepare_value(&moved, "value.ts").unwrap();
    assert_ne!(changed_text.key, changed.key);
    assert_eq!(changed_text.run().unwrap()["lines"], "南,29\n");

    for expression in [
        "input.bytes('samples' + '.bin')",
        "input.text(`lines.csv`)",
        "input.bytes('samples.bin', 1)",
        "input.bytes?.('samples.bin')",
        "input['bytes']('samples.bin')",
        "(() => {const read = input.bytes; return read('samples.bin')})()",
        "(() => {const alias = input; return alias.text('lines.csv')})()",
    ] {
        std::fs::write(
            root.join("denied.ts"),
            format!(
                "import {{input}} from '@origingame/engine/author';\nexport default {expression};"
            ),
        )
        .unwrap();
        let error = Program::prepare_value(&root, "denied.ts")
            .err()
            .expect(expression);
        assert!(error.contains("denied.ts:2:"), "{expression}: {error}");
    }
    for code in [
        "import * as og from '@origingame/engine/author'; const alias = og; export default alias.input.text('lines.csv');",
        "import * as og from '@origingame/engine/author'; export default og['input'].text('lines.csv');",
        "export {input} from '@origingame/engine/author';",
        "export * from '@origingame/engine/author';",
        "import {input} from '@origingame/engine/author'; export {input};",
    ] {
        std::fs::write(root.join("denied.ts"), code).unwrap();
        assert!(
            Program::prepare_value(&root, "denied.ts").is_err(),
            "{code}"
        );
    }
    for (path, reason) in [
        ("../outside.bin", "leaves its input root"),
        ("absent.bin", "absent.bin"),
        ("lines.csv", "requires UTF-8"),
    ] {
        std::fs::write(root.join("lines.csv"), [0xff]).unwrap();
        std::fs::write(root.join("denied.ts"), format!("import {{input}} from '@origingame/engine/author'; export default input.text('{path}');")).unwrap();
        let error = Program::prepare_value(&root, "denied.ts")
            .err()
            .expect(path);
        assert!(error.contains(reason), "{path}: {error}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("lines.csv"), root.join("linked.csv")).unwrap();
        std::fs::write(root.join("denied.ts"), "import {input} from '@origingame/engine/author'; export default input.bytes('linked.csv');").unwrap();
        let error = Program::prepare_value(&root, "denied.ts").err().unwrap();
        assert!(error.contains("cannot follow symlinks"), "{error}");
    }
}

/// Generated cells pass through the real map compiler, preserve authored
/// metadata, and change when tracked script input changes.
#[test]
fn generated_tilemap_cooks_loads_and_rejects_invalid_grids() {
    let root = TempDir::new("og-generated-map");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Map","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'map',entry:'src/main.ts'});").unwrap();
    std::fs::write(
        assets.join("tiles.ogt"),
        "type tileset\nversion 1\n---\ntexture \"\"\ngrid 8 8 3 1\n",
    )
    .unwrap();
    let map = "type tilemap\nversion 1\n---\ntile 8 16\nunit 3\ntileset \"tiles.ogt\"\ngenerate {\nscript \"grid.ts\"\ninput \"source\" \"cell.bin\"\n}\nlayer \"cells\" {\nz 4\nparallax 0.6 0.8\n}\nlayer \"manual\" {\nrow 2 -1 0\nrow 1 2 -1\n}\n";
    std::fs::write(assets.join("map.ogt"), map).unwrap();
    std::fs::write(assets.join("cell.bin"), [0]).unwrap();
    std::fs::write(assets.join("grid.ts"), "export default ({inputs}) => ({width: 3, height: 2, layers: [{name: 'cells', tiles: new Int32Array([inputs.source[0], -1, 2, 1, 0, 2])}]});").unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let loaded = || {
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_tilemap("map.ogt")
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let first = loaded();
    assert_eq!((first.width, first.height), (3, 2));
    assert_eq!(first.tile_size().to_array(), [3.0, 6.0]);
    // Stored cells are tile id + 1, zero for empty. This distinguishes
    // a real tile zero from an empty cell and catches transposed rows.
    assert_eq!(first.layers[0].cells, [1, 0, 3, 2, 1, 3]);
    assert_eq!(first.layers[1].cells, [3, 0, 1, 2, 3, 0]);
    assert_eq!(first.layers[0].z, 4);
    assert_eq!(first.layers[0].parallax.to_array(), [0.6, 0.8]);
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded(), first);
    std::fs::write(assets.join("cell.bin"), [2]).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded().layers[0].cells, [3, 0, 3, 2, 1, 3]);

    for source in [
        "({width: 3, height: 2, layers: [{name: 'cells', tiles: [0, 1, 2]}]})",
        "({width: 3, height: 2, layers: [{name: 'absent', tiles: [0, 1, 2, 0, 1, 2]}]})",
        "({width: 3, height: 2, layers: [{name: 'cells', tiles: [0, 1, 2, 0, -2, 2]}]})",
        "({width: 0, height: 2, layers: []})",
    ] {
        std::fs::write(
            assets.join("grid.ts"),
            format!("export default () => {source};"),
        )
        .unwrap();
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted invalid map {source}: {}",
            text(&out)
        );
    }
}

/// Float source samples follow the same clamping, channel order and sound
/// encoding as an authored WAV, rather than a parallel generator codec.
#[test]
fn generated_pcm_cooks_as_an_authored_sound() {
    let root = TempDir::new("og-generated-sound");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"PCM","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'pcm',entry:'src/main.ts'});").unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(root.join("content/made.ts"), "import {file} from '@origingame/engine/author'; export default file.sound({rate: 22050, channels: 2, samples: Float32Array.from({length: 600}, (_, i) => [-2, 0.25, 0.5, -0.75, 2, -0.5][i % 6])});").unwrap();
    let mut writer = hound::WavWriter::create(
        assets.join("authored.wav"),
        hound::WavSpec {
            channels: 2,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for _ in 0..100 {
        for sample in [-32767i16, 8192, 16384, -24575, 32767, -16384] {
            writer.write_sample(sample).unwrap();
        }
    }
    writer.finalize().unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let made = bundle.load_sound("base/made").unwrap().decode().unwrap();
    let authored = bundle.load_sound("authored.wav").unwrap().decode().unwrap();
    assert_eq!(
        (made.sample_rate, made.channels, made.samples.len()),
        (22050, 2, 600)
    );
    assert_eq!(made.samples, authored.samples);
    assert!(cook().status.success());
    assert_eq!(
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_sound("base/made")
            .unwrap()
            .decode()
            .unwrap()
            .samples,
        authored.samples,
    );
    // Cross the automatic QOA streaming boundary with a compact typed buffer.
    // The graph and planner must agree about the real last chunk.
    std::fs::write(root.join("content/long.ts"), "import {file} from '@origingame/engine/author'; export default file.sound({rate:192000,channels:1,samples:new Float32Array(2600000).fill(0.25)});").unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject,ref} from '@origingame/engine/author'; export default defineProject({id:'pcm',entry:'src/main.ts',roots:[ref('base/long#chunk3')]});").unwrap();
    let streamed = cook();
    assert!(streamed.status.success(), "{}", text(&streamed));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert!(bundle.read("base/long#chunk3").is_ok());
    assert!(bundle.read("base/long#chunk4").is_err());
    std::fs::remove_file(root.join("content/long.ts")).unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'pcm',entry:'src/main.ts'});").unwrap();
    for source in [
        "({rate: 22050, channels: 2, samples: [0, 0.5, 1]})",
        "({rate: 7999, channels: 1, samples: [0]})",
        "({rate: 22050, channels: 3, samples: [0, 0, 0]})",
        "({rate: 22050, channels: 1, samples: []})",
    ] {
        std::fs::write(
            root.join("content/made.ts"),
            format!("import {{file}} from '@origingame/engine/author'; export default file.sound({source});"),
        )
        .unwrap();
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted invalid PCM {source}: {}",
            text(&out)
        );
    }
    std::fs::remove_file(root.join("content/made.ts")).unwrap();
    std::fs::write(assets.join("made.ogt"), "type sound\nversion 1\n---\n").unwrap();
    let old = cook();
    assert!(!old.status.success());
    assert!(
        text(&old).contains("OGT authoring is removed"),
        "{}",
        text(&old)
    );
    std::fs::remove_file(assets.join("made.ogt")).unwrap();
    std::fs::write(assets.join("sampled.ogt"), "type sfx\nversion 1\n---\nlayer {\nsample {\nsource \"authored.wav\"\nloop true\n}\nattack 0\nsustain 0.01\ndecay 0\ngain 1\n}\n").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let decoded = || {
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_sound("sampled.ogt")
            .unwrap()
            .decode()
            .unwrap()
            .samples
    };
    let first = decoded();
    assert!(first.iter().any(|&v| v != 0));
    // Changing only the recording must invalidate the sample-based synth,
    // including an existing shared-cache entry for the unchanged SFX text.
    let mut writer = hound::WavWriter::create(
        assets.join("authored.wav"),
        hound::WavSpec {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for _ in 0..8 {
        writer.write_sample(0i16).unwrap();
    }
    writer.finalize().unwrap();
    std::fs::write(assets.join("zero.ogt"), "type sfx\nversion 1\n---\nlayer {\nsample {\nsamples 0\nrate 8000\nloop true\n}\nattack 0\nsustain 0.01\ndecay 0\ngain 1\n}\n").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_ne!(first, decoded());
    let silence = og_cook::runtime::Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_sound("zero.ogt")
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(decoded(), silence.samples);
}

/// Typed source attributes cross the public cook/load boundary, including
/// a mirrored merge's tangent handedness and vertex colours without a map.
#[test]
fn generated_mesh_cooks_with_attributes_and_placed_frames() {
    let root = TempDir::new("og-generated-mesh");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Mesh","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'mesh',entry:'src/main.ts'});").unwrap();
    std::fs::write(
        assets.join("mesh.ogt"),
        r#"type model
version 1
---
conform off
part "source" {
  bufferGeometry {
    generate {
      script "mesh.ts"
    }
  }
}
part "placed" {
  mergeGeometries {
    part {
      bufferGeometry {
        generate {
          script "mesh.ts"
        }
      }
      scale 2 1 3
      mirror x
    }
  }
}
part "mixed" {
  mergeGeometries {
    part {
      bufferGeometry {
        generate {
          script "mesh.ts"
        }
      }
    }
    part {
      bufferGeometry {
        positions 10 0 0 12 0 0 10 1 0
        uvs 0 1 1 1 0 0
      }
    }
  }
}
"#,
    )
    .unwrap();
    let source = "({positions: [-2, 0, 0, 3, 0, 0, 1, 4, 0], colors: [1,0,0, 0,1,0, 0,0,1], uvs: [0,1, 1,1, 0.6,0], tangents: [1,0,0,-1, 1,0,0,-1, 1,0,0,-1], indices: [0,1,2]})";
    std::fs::write(
        assets.join("mesh.ts"),
        format!("export default () => {source};"),
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("mesh.ogt").unwrap();
    assert_eq!(model.primitives.len(), 3);
    for (i, p) in model.primitives.iter().take(2).enumerate() {
        assert_eq!(p.lods[0].count, 3);
        assert_eq!(p.vertices.len(), 3);
        assert_eq!(p.tangents.len(), 3);
        let (x, w) = if i == 0 { (1.0, -1.0) } else { (-1.0, 1.0) };
        for (v, t) in p.vertices.iter().zip(&p.tangents) {
            assert!((v.normal[2] - 1.0).abs() < 0.001);
            assert!((t[0] - x).abs() < 0.001 && t[1].abs() < 0.001 && t[2].abs() < 0.001);
            assert_eq!(t[3], w);
            let c = if v.position[1] > 3.0 {
                assert!(v.uv[1].abs() < 0.001);
                [0.0, 0.0, 1.0, 1.0]
            } else if v.position[0] * x < 0.0 {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 1.0, 0.0, 1.0]
            };
            assert_eq!(v.color, c);
        }
        assert_eq!(p.min.x, if i == 0 { -2.0 } else { -6.0 });
        assert_eq!(p.max.x, if i == 0 { 3.0 } else { 4.0 });
    }
    let mixed = &model.primitives[2];
    assert_eq!(mixed.lods[0].count, 6);
    assert_eq!(mixed.tangents.len(), 6);
    for (v, t) in mixed.vertices.iter().zip(&mixed.tangents) {
        assert!((t[0] - 1.0).abs() < 0.001 && t[1].abs() < 0.001 && t[2].abs() < 0.001);
        // The supplied frame stays -1; the other triangle's decreasing V
        // gives an image-up bitangent along +Y, hence +1 for N=Z, T=X.
        assert_eq!(t[3], if v.position[0] > 9.0 { 1.0 } else { -1.0 });
    }
    // An explicit image/camera recipe must retain its asymmetric source
    // placement, and changing only the image must invalidate the cooked map.
    std::fs::write(assets.join("projection.ogt"), "type model\nversion 1\n---\nconform off\npart \"panel\" {\nplaneGeometry {\nwidth 2\nheight 2\n}\n}\nprocess {\nproject {\nsize 32\npadding 2\nview \"view.svg\" {\nposition 0 0 4\ntarget 0 0 0\northo 1\n}\n}\n}\n").unwrap();
    let image = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#e34a2c"/><rect x="24" width="8" height="32" fill="#2c69df"/></svg>"##;
    std::fs::write(assets.join("view.svg"), image).unwrap();
    // The wider variant sorts before its processed base. Transfer retains
    // metric distances rather than stretching the base image over the variant.
    std::fs::write(assets.join("a-variant.ogt"), "type model\nversion 1\n---\nconform off\npart \"panel\" {\nplaneGeometry {\nwidth 3\nheight 2\n}\n}\nprocess {\ntransfer {\nsource \"projection.ogt\"\nsize 32\npadding 2\n}\n}\n").unwrap();
    let cooked_map = |id: &str| {
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let model = bundle.load_model(id).unwrap();
        let index = model
            .materials
            .iter()
            .find_map(|m| m.base_color_texture)
            .unwrap();
        bundle
            .load_texture(&model.textures[index].id)
            .unwrap()
            .to_texture_data()
            .unwrap()
    };
    let first = cooked_map("projection.ogt");
    assert_eq!((first.width, first.height), (32, 32));
    let pixel = |rgba: &[u8], x: usize| rgba[(16 * 32 + x) * 4..(16 * 32 + x) * 4 + 3].to_vec();
    let left = pixel(&first.rgba, 8);
    let right = pixel(&first.rgba, 28);
    assert!(left[0] > 180 && left[2] < 90, "{left:?}");
    assert!(right[2] > 180 && right[0] < 90, "{right:?}");
    assert_eq!(cooked_map("projection.ogt").rgba, first.rgba);
    let variant = cooked_map("a-variant.ogt");
    let middle = pixel(&variant.rgba, 20);
    assert!(
        middle[2] > 180 && middle[0] < 90,
        "metric transfer must not stretch the red region: {middle:?}"
    );
    assert_eq!(pixel(&variant.rgba, 8), pixel(&first.rgba, 8));
    assert_eq!(cooked_map("a-variant.ogt").rgba, variant.rgba);
    std::fs::write(assets.join("view.svg"), image.replace("#e34a2c", "#2ce34a")).unwrap();
    let changed = cooked_map("projection.ogt");
    let left = pixel(&changed.rgba, 8);
    assert!(left[1] > 180 && left[0] < 90, "{left:?}");
    assert_eq!(pixel(&changed.rgba, 28), right);
    let variant = cooked_map("a-variant.ogt");
    assert_eq!(pixel(&variant.rgba, 8), left);
    let middle = pixel(&variant.rgba, 20);
    assert!(middle[2] > 180 && middle[0] < 90, "{middle:?}");
    // The consumer sorts before its source. Import order is a dependency,
    // not filename order, and a selected node keeps its rest placement.
    let operand = "type model\nversion 1\n---\nconform off\npart \"copy\" {\nmergeGeometries {\npart {\nmodelGeometry {\nsource \"mesh.ogt\"\nnode \"placed\"\n}\nposition 0 2 0\n}\n}\n}\n";
    std::fs::write(assets.join("a-copy.ogt"), operand).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let copy = bundle.load_model("a-copy.ogt").unwrap();
    let p = &copy.primitives[0];
    assert_eq!((p.min.x, p.max.x, p.min.y, p.max.y), (-6.0, 4.0, 2.0, 6.0));
    for (v, t) in p.vertices.iter().zip(&p.tangents) {
        assert!((t[0] + 1.0).abs() < 0.001);
        assert_eq!(t[3], 1.0);
        assert_eq!(v.color.iter().sum::<f32>(), 2.0);
    }
    std::fs::write(
        assets.join("mesh.ts"),
        format!(
            "export default () => {};",
            source.replace("-2, 0, 0", "-4, 0, 0")
        ),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.load_model("a-copy.ogt").unwrap().primitives[0].max.x,
        8.0
    );
    std::fs::write(
        assets.join("mesh.ogt"),
        operand
            .replace("mesh.ogt", "a-copy.ogt")
            .replace("node \"placed\"", ""),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(text(&out).contains("dependency cycle"), "{}", text(&out));
    std::fs::remove_file(assets.join("a-copy.ogt")).unwrap();
    std::fs::write(assets.join("mesh.ogt"), "type model\nversion 1\n---\nconform off\npart \"source\" {\nbufferGeometry {\ngenerate {\nscript \"mesh.ts\"\n}\n}\n}\n").unwrap();
    for invalid in [
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,3]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], colors: [1,0,0]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], tangents: [1,0,0,-1]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,1.5]})",
    ] {
        std::fs::write(
            assets.join("mesh.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}

/// Generated declarations use the same scene/prefab/reference compiler as
/// authored ones, including input invalidation and sub-scene cycle refusal.
#[test]
fn generated_scene_cooks_as_authored_declarations() {
    let root = TempDir::new("og-generated-scene");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Scene","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'scene',entry:'src/main.ts'});").unwrap();
    std::fs::write(
        assets.join("sub.ogt"),
        "type scene\nversion 1\n---\nentity \"nested\" {\n}\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("model.ogt"),
        "type model\nversion 1\n---\nconform off\npart \"part\" {\nboxGeometry {\n}\n}\n",
    )
    .unwrap();
    std::fs::write(assets.join("base.ogt"), "type prefab\nversion 1\n---\nentity \"base\" {\nStats {\nweight 2\n}\nentity \"prefab-child\" {\nStats {\nweight 4\n}\n}\n}\n").unwrap();
    let prefix = "type scene\nversion 1\n---\nname \"Generated\"\nentity \"manual\" {\nStats {\nweight 1\n}\n}\n";
    std::fs::write(
        assets.join("made.ogt"),
        format!("{prefix}generate {{\nscript \"scene.ts\"\ninput \"source\" \"weight.bin\"\n}}\n"),
    )
    .unwrap();
    std::fs::write(assets.join("scene.ts"), r#"export default ({inputs}) => ({
        entities: [{name: "generated", prefab: "base.ogt", components: {Stats: {weight: inputs.source[0]}}, children: [
            {name: "instance", model: "model.ogt", static: true},
            {name: "cell", scene: "sub.ogt"}
        ]}], resources: {Run: {count: 3, enabled: true, weights: [2, 7, 3]}}
    });"#).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for weight in [7, 9] {
        std::fs::write(assets.join("weight.bin"), [weight]).unwrap();
        std::fs::write(assets.join("authored.ogt"), format!("{prefix}entity \"generated\" {{\nprefab \"base.ogt\"\nStats {{\nweight {weight}\n}}\nentity \"instance\" {{\nmodel \"model.ogt\"\nstatic true\n}}\nentity \"cell\" {{\nscene \"sub.ogt\"\n}}\n}}\nresource Run {{\ncount 3\nenabled true\nweights 2 7 3\n}}\n")).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let made = bundle.load_scene("made.ogt").unwrap();
        let authored = bundle.load_scene("authored.ogt").unwrap();
        assert_eq!(made.entities.len(), 5);
        for (mut a, mut b) in made.entities.into_iter().zip(authored.entities) {
            a.line = None;
            b.line = None;
            assert_eq!(a, b);
        }
        assert_eq!(made.components, authored.components);
        assert_eq!(made.resources, authored.resources);
        assert_eq!(made.prefabs, authored.prefabs);
    }
    for invalid in [
        "({entities: [{name: 'bad', model: 'missing.glb'}]})",
        "({entities: [{name: 'bad', scene: 'made.ogt'}]})",
        "({entities: [{name: 'bad', components: {Stats: {weight: {nested: 1}}}}]})",
        "({entities: [{name: 'bad', components: {entity: {}}}]})",
        "({entities: [{name: 'bad', unexpected: true}]})",
    ] {
        std::fs::write(
            assets.join("scene.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}
