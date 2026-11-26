//! One evaluated-content path, through the real restricted VM and native schemas.
use super::temp;
use og_cook::cook::{
    source_content::{Evaluator, Graph, GraphBuilder},
    source_index::Index,
    source_project::Project,
    source_schemas::world::scene::SceneInput,
};
use og_core::reflect::source::AuthorSchema;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn project_records_bind_module_scoped_world_inputs() {
    use og_cook::cook::source_records::Records;
    use serde_json::json;
    let dir = temp::TempDir::new("project-records");
    let write = |file: &str, text: &str| {
        let at = dir.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    write(
        "project.ts",
        &format!(
            "import {{defineProject}} from '@origingame/engine/author'; export default defineProject({});",
            json!({"name":"Records","id":"records","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts",
        "schemas":["src/schema/base.ts","modules/north/src/schema/local.ts"],
        "modules":[{"id":"north","root":"modules/north","dependsOn":["base"]}]})
        ),
    );
    let base = "import * as s from '@origingame/engine/schema'; export const Health = s.defineComponent('game.Health', {z:s.u32(83,{min:1,max:100}), a:s.Texture(), label:s.str(8,'北'), pos:s.Vector3({x:1,y:2,z:3}), ammo:s.list(s.u32(),4)}); export const Settings=s.defineResource('game.Settings',{volume:s.f32(0.3)});";
    write("src/schema/base.ts", base);
    let local = "import {defineComponent,f32} from '@origingame/engine/schema'; export const Local=defineComponent('north.Local',{power:f32(7)});";
    write("modules/north/src/schema/local.ts", local);
    let project = Project::open(&dir).unwrap();
    let records = Records::open(&dir, &project).unwrap();
    assert_eq!(
        records.declarations["game.Health"]
            .record
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["z", "a", "label", "pos", "ammo"]
    );
    let north = &records.contexts["north"];
    for (kind, name) in [("scene", "Scene"), ("prefab", "Prefab"), ("table", "Table")] {
        og_core::reflect::source::typescript(north.schema(kind).unwrap().document(), name).unwrap();
    }
    let scene = json!({"entities":[{"name":"北😀","components":{"game.Health":{"z":11,"a":{"$ogRef":"base/texture"},"pos":[3,7,2],"ammo":[3,11]},"north.Local":{"power":13},"og.Transform":{"position":[1,3,7]}}}],"resources":{"game.Settings":{"volume":0.7}}});
    north.scene.check(&scene).unwrap();
    assert!(records.contexts["base"].scene.check(&scene).is_err());
    let table = json!({"record":"game.Health","rows":[{"id":"one","values":{"z":21,"a":{"$ogRef":"base/texture"}}}]});
    north.table.check(&table).unwrap();
    write(
        "content/texture.ts",
        "import {file} from '@origingame/engine/author'; export default file.import({kind:'texture',source:'assets/leaf.png'});",
    );
    write(
        "modules/north/content/table.ts",
        &format!(
            "import {{file}} from '@origingame/engine/author'; export default file.table({table});"
        ),
    );
    let index = Index::open(&dir, &project).unwrap();
    let mut evaluator = Evaluator::default();
    let mut graph = GraphBuilder::new(&project, &index);
    for entry in index.entries.values() {
        let context = &records.contexts[&entry.module];
        let content = evaluator
            .evaluate(
                &dir,
                &project,
                entry,
                &BTreeMap::new(),
                context.schema(&entry.factory),
            )
            .unwrap();
        graph.insert(&content, &BTreeMap::new()).unwrap();
    }
    let graph = graph.finish().unwrap();
    assert_eq!(
        graph.nodes["north/table"].references[0].target,
        "base/texture"
    );
    assert_eq!(graph.nodes["north/table"].references[0].kind, "texture");
    let mut wrong = table.clone();
    wrong["record"] = json!("north.Local");
    assert!(north.table.check(&wrong).is_err());
    for (path, bad) in [
        ("/entities/0/components/game.Health/z", json!(101)),
        ("/entities/0/components/game.Health/a", json!(0)),
        (
            "/entities/0/components/game.Health/ammo",
            json!([1, 2, 3, 4, 5]),
        ),
    ] {
        let mut wrong = scene.clone();
        *wrong.pointer_mut(path).unwrap() = bad;
        assert!(north.scene.check(&wrong).is_err(), "{path}");
    }
    assert_eq!(
        records.evaluations["src/schema/base.ts"].1,
        ["src/schema/base.ts"]
    );
    let overlay = BTreeMap::from([(
        "src/schema/base.ts".into(),
        base.replace("max:100", "max:10"),
    )]);
    let candidate = Records::open_with_overlay(&dir, &project, &overlay).unwrap();
    assert!(candidate.contexts["north"].table.check(&table).is_err());
    assert_ne!(
        candidate.evaluations["src/schema/base.ts"].0,
        records.evaluations["src/schema/base.ts"].0
    );
    for (file, text, message) in [
        (
            "src/schema/base.ts",
            base.replace("game.Health", "og.Transform"),
            "engine-owned",
        ),
        (
            "src/schema/base.ts",
            base.replace("game.Settings", "game.Health"),
            "duplicate record",
        ),
        (
            "src/schema/base.ts",
            format!("import '../../modules/north/src/schema/local.ts'; {base}"),
            "direct dependency",
        ),
        (
            "modules/north/src/schema/local.ts",
            format!("import '../../../..//src/main.ts'; {local}"),
            "cannot import",
        ),
        (
            "src/schema/base.ts",
            format!(
                "import {{input}} from '@origingame/engine/author'; input.text('assets/data'); {base}"
            ),
            "cannot read raw inputs",
        ),
        (
            "src/schema/base.ts",
            format!("Math.random(); {base}"),
            "rng(seed)",
        ),
        (
            "src/schema/base.ts",
            format!("export const helper = 3; {base}"),
            "only named record tokens",
        ),
    ] {
        let overlay = BTreeMap::from([(file.into(), text)]);
        let error = match Records::open_with_overlay(&dir, &project, &overlay) {
            Ok(_) => panic!("accepted {message}"),
            Err(error) => error,
        };
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn evaluated_content_resolves_snapshot_scoped_references() {
    let dir = temp::TempDir::new("evaluated-content");
    let write = |file: &str, text: &str| {
        let at = dir.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    let manifest = |north: &[&str], boot: &str| {
        format!(
            "import {{defineProject}} from '@origingame/engine/author'; export default defineProject({});",
            serde_json::json!({"name":"World","id":"world","engine":env!("CARGO_PKG_VERSION"),"entry":"src/main.ts",
                "inputs":["sources/*.txt"], "scenes":{"Boot":{"$ogRef":boot}},
                "modules":[{"id":"north","root":"modules/north","dependsOn":north},
                    {"id":"east","root":"modules/east","dependsOn":["base"]}],
                "packs":[{"id":"north","include":["north/**"],"optional":true,"roots":[{"$ogRef":"north/fx"}]}]})
        )
    };
    write("src/main.ts", "throw new Error('GAME_EXECUTED');");
    write("project.ts", &manifest(&["base", "east"], "north/main"));
    write("sources/title.txt", "北😀");
    write(
        "lib/title.ts",
        "import {input} from '@origingame/engine/author'; export const title = input.text('sources/title.txt');",
    );
    write(
        "content/look.ts",
        "import {file} from '@origingame/engine/author'; import {title} from '../lib/title.ts'; export default file.look({name:title});",
    );
    let model = "import {file} from '@origingame/engine/author'; export default file.import({kind:'model',source:'assets/gate.glb'});";
    write("content/model.ts", model);
    let scene_file = "modules/north/content/main.ts";
    let scene = "import * as og from '@origingame/engine/author';\r\nexport default og.file.scene({entities:[\r\n  {name:'北😀', model: og.ref('base/model')}\r\n]});";
    write(scene_file, scene);
    write(
        "modules/north/content/fx.ts",
        "import * as og from '@origingame/engine/author'; const texture = () => og.ref('base/model#tex0'); export default og.file.vfx({duration:1,elements:[{node:{kind:'decal',texture:texture(),normal:texture()}}]});",
    );
    write(
        "modules/east/content/unused.ts",
        "import {file} from '@origingame/engine/author'; export default file.look({});",
    );
    let project = Project::open(&dir).unwrap();
    let index = Index::open(&dir, &project).unwrap();
    let schema = SceneInput::schema_with_records(&[], &[]).unwrap();
    let empty = BTreeMap::new();
    let build = |project: &Project,
                 index: &Index,
                 overlay: &BTreeMap<String, String>,
                 kind: Option<&str>|
     -> Result<Graph, _> {
        let mut evaluator = Evaluator::default();
        let mut graph = GraphBuilder::new(project, index);
        // Consumers arrive before producers: scheduling order is not authority.
        for entry in index.entries.values().rev() {
            let contextual = (entry.factory == "scene").then_some(&schema as &dyn AuthorSchema);
            let content = evaluator.evaluate(&dir, project, entry, overlay, contextual)?;
            let derived = if entry.id == "base/model" {
                kind.map(|kind| BTreeMap::from([("tex0".into(), kind.into())]))
                    .unwrap_or_default()
            } else if entry.id == "base/program" {
                BTreeMap::from([("lod".into(), "model".into())])
            } else if entry.id == "base/texture" {
                BTreeMap::from([
                    ("emission".into(), "texture".into()),
                    ("array".into(), "texture-array".into()),
                ])
            } else {
                BTreeMap::new()
            };
            // These are producer-reported facts; this test does not import GLB.
            graph.insert(&content, &derived)?;
            // `content` (including source, IR and buffers) is dropped each turn.
        }
        graph.finish()
    };
    let graph = build(&project, &index, &empty, Some("texture")).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.outputs.len(), 6);
    assert_eq!(graph.outputs["base/model"].kind, "model");
    assert_eq!(graph.outputs["base/model#tex0"].kind, "texture");
    assert_eq!(
        graph.dependents["base/model"],
        BTreeSet::from(["north/main".into(), "north/fx".into()])
    );
    assert!(graph.dependents["east/unused"].is_empty());
    let fx = &graph.nodes["north/fx"].references;
    assert_eq!(fx.len(), 2);
    assert!(fx.iter().all(|edge| edge.target == "base/model#tex0"
        && edge.entry == "base/model"
        && edge.kind == "texture"));
    let fx_source = std::fs::read_to_string(dir.join("modules/north/content/fx.ts")).unwrap();
    for edge in fx {
        assert_eq!(
            &fx_source[edge.location.start as usize..edge.location.end as usize],
            "texture()"
        );
    }
    let main = &graph.nodes["north/main"].references[0];
    assert_eq!(
        (main.path.as_str(), main.kind.as_str()),
        ("/entities/0/model", "model")
    );
    assert_eq!(main.location.line, 3);
    assert_eq!(
        main.location.column,
        "  {name:'北😀', model: ".chars().count() + 1
    );
    assert_eq!(
        &scene[main.location.start as usize..main.location.end as usize],
        "og.ref('base/model')"
    );
    // Root/pack refs assemble declared modules without authorizing base source
    // to import north (which already directly depends on base).
    assert_eq!(
        graph
            .roots
            .iter()
            .map(|e| (e.path.as_str(), e.target.as_str()))
            .collect::<Vec<_>>(),
        [
            ("/packs/0/roots/0", "north/fx"),
            ("/scenes/Boot", "north/main")
        ]
    );
    let look = &graph.nodes["base/look"];
    assert_eq!(look.input_dependencies, ["sources/title.txt"]);
    assert_eq!(look.imports["content/look.ts"], ["lib/title.ts"]);
    assert!(look.imports["lib/title.ts"].is_empty());

    let mut evaluator = Evaluator::default();
    // A ref does not execute/read its target; the closed VM only evaluates the
    // consumer even while the target's current bytes would throw.
    write("content/model.ts", "throw new Error('TARGET_EXECUTED');");
    let content = evaluator
        .evaluate(
            &dir,
            &project,
            &index.entries["north/main"],
            &empty,
            Some(&schema),
        )
        .unwrap();
    write("content/model.ts", model);
    assert_eq!(content.entry().id, "north/main");
    assert!(content.source().field("/entities/0/model").is_err());
    assert_eq!(
        content.data().root().into_json()["value"]["entities"][0]["model"]["$ogRef"],
        "base/model"
    );
    assert!(
        evaluator
            .evaluate(&dir, &project, &index.entries["north/main"], &empty, None)
            .err()
            .unwrap()
            .message
            .contains("record schemas")
    );
    let mut partial = GraphBuilder::new(&project, &index);
    partial.insert(&content, &empty).unwrap();
    assert!(
        partial
            .insert(&content, &empty)
            .unwrap_err()
            .message
            .contains("already added")
    );
    assert!(
        partial
            .finish()
            .unwrap_err()
            .message
            .contains("incomplete content graph")
    );

    for (target, message) in [
        ("base/absent", "unknown content output"),
        ("base/look", "has kind look"),
        ("base/model#invented", "unknown content output"),
    ] {
        let overlay = BTreeMap::from([(scene_file.into(), scene.replace("base/model", target))]);
        let candidate = Index::open_with_overlay(&dir, &project, &overlay).unwrap();
        assert!(
            GraphBuilder::new(&project, &candidate)
                .insert(&content, &empty)
                .unwrap_err()
                .message
                .contains("different project/index snapshot")
        );
        let error = build(&project, &candidate, &overlay, Some("texture")).unwrap_err();
        assert_eq!(
            (
                error.file.as_str(),
                error.path.as_str(),
                error.location.line
            ),
            (scene_file, "/entities/0/model", 3)
        );
        assert!(error.message.contains(message), "{error}");
    }
    // Never treat an unreported suffix as the primary output or infer its kind.
    assert!(
        build(&project, &index, &empty, None)
            .unwrap_err()
            .message
            .contains("base/model#tex0")
    );
    for wrong in ["model", "effect-texture"] {
        assert!(
            build(&project, &index, &empty, Some(wrong))
                .unwrap_err()
                .message
                .contains(&format!("has kind {wrong}"))
        );
    }
    let producer = evaluator
        .evaluate(&dir, &project, &index.entries["base/model"], &empty, None)
        .unwrap();
    let mut outputs = GraphBuilder::new(&project, &index);
    for bad in [
        BTreeMap::from([
            ("tex0".into(), "texture".into()),
            ("TEX0".into(), "texture".into()),
        ]),
        BTreeMap::from([("emission##array".into(), "texture-array".into())]),
    ] {
        assert!(outputs.insert(&producer, &bad).is_err());
    }
    outputs.insert(&producer, &empty).unwrap();

    let invalid = BTreeMap::from([(
        scene_file.into(),
        scene.replace("og.ref('base/model')", "7"),
    )]);
    let invalid_index = Index::open_with_overlay(&dir, &project, &invalid).unwrap();
    let error = build(&project, &invalid_index, &invalid, Some("texture")).unwrap_err();
    assert_eq!(
        (
            error.file.as_str(),
            error.path.as_str(),
            error.location.line
        ),
        (scene_file, "/entities/0/model", 3)
    );

    write("project.ts", &manifest(&["east"], "north/main"));
    let indirect = Project::open(&dir).unwrap();
    assert!(
        GraphBuilder::new(&indirect, &index)
            .insert(&content, &empty)
            .unwrap_err()
            .message
            .contains("different project/index snapshot")
    );
    let error = build(&indirect, &index, &empty, Some("texture")).unwrap_err();
    assert!(
        error
            .message
            .contains("module north to declare a direct dependency on base"),
        "{error}"
    );
    for (boot, message) in [
        ("base/model", "has kind model"),
        ("north/missing", "unknown content output"),
    ] {
        write("project.ts", &manifest(&["base"], boot));
        let invalid = Project::open(&dir).unwrap();
        let error = build(&invalid, &index, &empty, Some("texture")).unwrap_err();
        assert_eq!(
            (error.file.as_str(), error.path.as_str()),
            ("project.ts", "/scenes/Boot")
        );
        assert!(error.message.contains(message), "{error}");
    }
    write("project.ts", &manifest(&["base", "east"], "north/main"));
    write("sources/title.txt", "changed");
    let changed = build(&project, &index, &empty, Some("texture")).unwrap();
    assert_ne!(
        changed.nodes["base/look"].evaluation_key,
        look.evaluation_key
    );
    assert_eq!(
        changed.nodes["north/main"].evaluation_key,
        graph.nodes["north/main"].evaluation_key
    );
    assert_eq!(changed.nodes["base/look"].schema_key, look.schema_key);
    // Changing a target changes its local key, not its consumers' evaluation
    // keys. Its reverse edges identify affected domain jobs in the later stage.
    write(
        "content/model.ts",
        &model.replace("source:", "raycast:true,source:"),
    );
    let changed_index = Index::open(&dir, &project).unwrap();
    let changed = build(&project, &changed_index, &empty, Some("texture")).unwrap();
    assert_ne!(
        changed.nodes["base/model"].evaluation_key,
        graph.nodes["base/model"].evaluation_key
    );
    assert_eq!(
        changed.nodes["north/main"].evaluation_key,
        graph.nodes["north/main"].evaluation_key
    );

    // Identity closure does not impose a build DAG: relationship-specific cycle
    // rejection (e.g. scene expansion) is the later domain cooker's job.
    write(
        "modules/north/content/a.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file.scene({entities:[{name:'a',scene:ref('north/b')}]});",
    );
    write(
        "modules/north/content/b.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file.scene({entities:[{name:'b',scene:ref('north/a')}]});",
    );
    let cyclic = Index::open(&dir, &project).unwrap();
    let cyclic = build(&project, &cyclic, &empty, Some("texture")).unwrap();
    assert_eq!(
        cyclic.dependents["north/a"],
        BTreeSet::from(["north/b".into()])
    );
    assert_eq!(
        cyclic.dependents["north/b"],
        BTreeSet::from(["north/a".into()])
    );

    // Factories describe how to author a declaration; kind describes what a
    // consumer gets. Source-only uses must still reject other same-kind entries.
    write(
        "content/program.ts",
        "import {file} from '@origingame/engine/author'; export default file['model-program']({ops:[]});",
    );
    let variant = "import {file,ref} from '@origingame/engine/author'; export default file.model({kind:'program',desc:{program:ref('base/program')}});";
    write("content/variant.ts", variant);
    let texture = serde_json::json!({
        "mode":"noise","size":16,"frames":3,"columns":2,"grid":8,"steps":2,"dt":0.025,
        "pressure":7,"seed":17,"frequency":2.5,"range":1.5,"turbulence":0.2,
        "source":[0.1,0.35,0.8],"detail":0.25,"vorticity":0.4,"radius":0.15,
        "inject":0.2,"density":0.7,"heat":0.8,"buoyancy":-0.3,"expansion":0.4,
        "decay":0.6,"cooling":0.9,"absorption":1.5,"color":[0.1,0.5,0.8],
        "emission":[0.9,0.4,0.2],"light":[-1,2,3],"ambient":0.3,"direct":0.7
    });
    write(
        "content/texture.ts",
        &format!(
            "import {{file}} from '@origingame/engine/author'; export default file['effect-texture']({texture});"
        ),
    );
    write(scene_file, &scene.replace("base/model", "base/program"));
    let look = "import {file,ref} from '@origingame/engine/author'; export default file.look({MaterialStyle:{gradientMap:ref('base/texture#emission')},ui:ref('base/child-theme')});";
    write("content/look.ts", look);
    write(
        "content/sound.ts",
        "import {file} from '@origingame/engine/author'; export default file.sfx({seed:13,layers:[{freq:110}]});",
    );
    write(
        "content/pixel.ts",
        "import {file} from '@origingame/engine/author'; export default file.vfx2d({recipe:{layers:[{type:'burst'}]}});",
    );
    write(
        "content/theme.ts",
        "import {file} from '@origingame/engine/author'; export default file.theme({});",
    );
    let theme = "import {file,ref} from '@origingame/engine/author'; export default file.theme({extends:ref('base/theme')});";
    write("content/child-theme.ts", theme);
    write(
        "content/pack.ts",
        "import {file} from '@origingame/engine/author'; export default file.pack({});",
    );
    write(
        "content/font.ts",
        "import {file} from '@origingame/engine/author'; export default file.import({kind:'font',source:'assets/body.ttf'});",
    );
    write(
        "content/fonts.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file['font-extension']({kind:'font',name:'Body',faces:[ref('base/font')]});",
    );
    write(
        "modules/north/content/fx.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file.vfx({duration:1,elements:[{node:{kind:'decal',texture:ref('base/texture')}},{node:{kind:'sound',asset:ref('base/sound')}},{node:{kind:'flipbook',asset:ref('base/pixel')}}]});",
    );
    let index = Index::open(&dir, &project).unwrap();
    let graph = build(&project, &index, &empty, Some("texture")).unwrap();
    for (id, factory, kind) in [
        ("base/program", "model-program", "model"),
        ("base/texture", "effect-texture", "texture"),
        ("base/sound", "sfx", "sound"),
        ("base/pixel", "vfx2d", "flipbook"),
        ("base/theme", "theme", "pack"),
        ("base/fonts", "font-extension", "font-pack"),
    ] {
        assert_eq!(index.entries[id].factory, factory);
        assert_eq!(index.entries[id].kind, kind);
        assert_eq!(graph.outputs[id].kind, kind);
    }
    assert_eq!(graph.nodes["north/main"].references[0].kind, "model");
    assert_eq!(graph.nodes["base/look"].references[0].kind, "texture");
    for (file, source, from, to, message) in [
        (
            "content/variant.ts",
            variant,
            "base/program",
            "base/model",
            "factory",
        ),
        (
            "content/variant.ts",
            variant,
            "base/program",
            "base/program#lod",
            "factory",
        ),
        (
            "content/child-theme.ts",
            theme,
            "base/theme",
            "base/pack",
            "factory",
        ),
        (
            "content/look.ts",
            look,
            "base/texture#emission",
            "base/texture#array",
            "has kind texture-array",
        ),
    ] {
        let overlay = BTreeMap::from([(file.into(), source.replace(from, to))]);
        let candidate = Index::open_with_overlay(&dir, &project, &overlay).unwrap();
        let error = build(&project, &candidate, &overlay, Some("texture")).unwrap_err();
        assert_eq!(error.file, file);
        assert!(error.message.contains(message), "{error}");
    }
}
