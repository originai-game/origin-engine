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
    assert!(
        build(&project, &index, &empty, Some("model"))
            .unwrap_err()
            .message
            .contains("has kind model")
    );
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
}
