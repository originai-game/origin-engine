//! The cook end to end on the project template
//! (`tests/fixtures/cook-assets/assets`): its determinism, and what it
//! writes loading back as authored.
//!
//! Ways this could fail, and the test that catches each:
//! - output bytes depend on thread scheduling, map order, timestamps or
//!   uninitialised padding → `every_thread_count_cooks_the_same_bytes`
//!   (cooks of a small template on 1, 2 and 8 threads, three at a time);
//! - QOA, font and scene compilation produce data the loaders cannot read
//!   or that means something else → `sounds_fonts_and_scenes_load_as_authored`.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The public behaviour cook/load contract: canonical transitive subtrees,
/// native machine indices, route relocation and captured helper invalidation.
#[test]
fn typed_behaviours_inline_native_trees_and_refresh_machine_consumers() {
    let root = crate::temp::TempDir::new("typed-behaviours");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'ai',entry:'src/main.ts',dependsOn:['north'],modules:[{id:'north',root:'modules/north'}]});",
    );
    let machine = "import {file,ref} from '@origingame/engine/author';export default file.behaviour({name:'guard😀',keys:{rank:7},routes:{a:[[1,2,3]]},root:{kind:'machine',initial:'active',states:[{name:'idle',transitions:[{to:'active',after:0.25}]},{name:'active',enter:[{kind:'set',key:'rank',value:11}],exit:[{kind:'stop'}],body:{kind:'subtree',source:ref('north/patrol')},transitions:[{to:'idle',checks:[{operand:'Health.hp',op:'below',against:'Tuning.flee'}]}]}]}});";
    let patrol = "import {file,ref} from '@origingame/engine/author';export default file.behaviour({name:'patrol',keys:{rank:1,point:[2,3,7]},routes:{z:[[-3,5,8],[9,-2,4]]},root:{kind:'tree',node:{kind:'sequence',name:'route then pause',children:[{kind:'action',action:{kind:'patrol',route:'z',wait:0.125,speed:'Tuning.speed',loops:false}},{kind:'subtree',source:ref('north/leaf')}]}}});";
    let leaf = "import {file} from '@origingame/engine/author';import {hold} from '../lib/options.ts';export default file.behaviour({name:'leaf',root:{kind:'tree',node:{kind:'action',action:{kind:'wait',seconds:hold}}}});";
    write("content/machine.ts", machine);
    write("modules/north/content/patrol.ts", patrol);
    write("modules/north/content/leaf.ts", leaf);
    write("modules/north/lib/options.ts", "export const hold=0.375;");
    let mut first = Vec::new();
    for threads in [1, 4] {
        let report = cook_project(
            &root,
            &CookOptions {
                threads,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let blob = bundle.read("base/machine").unwrap();
        let data = bundle.behaviour_data("base/machine", &blob).unwrap();
        let b = og_ai::Behaviour::from_bytes(&data).unwrap();
        assert_eq!(b.name, "guard😀");
        assert_eq!(b.subtrees, ["north/patrol", "north/leaf"]);
        assert_eq!(
            b.keys.iter().find(|k| k.0 == "rank").unwrap().1,
            og_ai::Value::Num(7.0)
        );
        assert_eq!(b.routes[1].0, "z");
        assert_eq!(b.routes[1].1[1].to_array(), [9.0, -2.0, 4.0]);
        assert!(b.nodes.iter().any(|n| matches!(&n.kind, og_ai::Kind::Action(og_ai::Action::Patrol {route:1,loops:false,speed:Some(og_ai::Speed::Read(s)),..}) if s=="Tuning.speed")));
        let og_ai::Root::Machine { states, initial } = &b.root else {
            panic!("expected a machine")
        };
        assert_eq!(*initial, 1);
        assert_eq!(states[0].transitions[0].to, 1);
        assert_eq!(states[1].transitions[0].to, 0);
        assert_eq!(
            states[1].transitions[0].checks[0].against.as_deref(),
            Some("Tuning.flee")
        );
        assert!(matches!(
            states[1].enter[0],
            og_ai::Action::Set {
                value: og_ai::Value::Num(11.0),
                ..
            }
        ));
        assert!(b.nodes.iter().any(|n| matches!(
            n.kind,
            og_ai::Kind::Action(og_ai::Action::Wait { seconds: 0.375 })
        )));
        if first.is_empty() {
            first = blob.to_vec();
        } else {
            assert_eq!(&*blob, first);
        }
        let mut trailing = data;
        trailing.push(0);
        assert!(og_ai::Behaviour::from_bytes(&trailing).is_err());
    }
    write("modules/north/lib/options.ts", "export const hold=0.875;");
    let changed = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(changed.ok(), "{:?}", changed.errors());
    assert!(
        changed
            .cooked_outputs()
            .iter()
            .any(|o| o.id == "base/machine")
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let blob = bundle.read("base/machine").unwrap();
    assert_ne!(&*blob, first);
    let b = og_ai::Behaviour::from_bytes(&bundle.behaviour_data("base/machine", &blob).unwrap())
        .unwrap();
    assert!(b.nodes.iter().any(|n| matches!(
        n.kind,
        og_ai::Kind::Action(og_ai::Action::Wait { seconds: 0.875 })
    )));
    assert!(
        cook_project(&root, &CookOptions::default())
            .unwrap()
            .cooked_outputs()
            .is_empty()
    );
    for (path, invalid, expected) in [
        (
            "modules/north/content/patrol.ts",
            patrol.replace("route:'z'", "route:'absent'"),
            "absent",
        ),
        (
            "modules/north/content/patrol.ts",
            patrol.replace("kind:'sequence'", "kind:'unknown'"),
            "kind",
        ),
        (
            "modules/north/content/leaf.ts",
            leaf.replace(
                "{kind:'action',action:{kind:'wait',seconds:hold}}",
                "{kind:'subtree',source:{\u{0024}ogRef:'north/patrol'}}",
            ),
            "cycle",
        ),
        (
            "content/machine.ts",
            machine.replace("to:'idle'", "to:'missing'"),
            "missing",
        ),
        (
            "content/machine.ts",
            machine.replace("against:'Tuning.flee'", "against:'Tuning.flee',value:4"),
            "exactly one",
        ),
        (
            "content/machine.ts",
            machine.replace("{kind:'stop'}", "{kind:'wait',seconds:1}"),
            "actions",
        ),
        (
            "content/machine.ts",
            machine.replace("routes:{a:[[1,2,3]]}", "routes:{z:[[1,2,3]]}"),
            "different points",
        ),
    ] {
        write(path, &invalid);
        let error = match cook_project(&root, &CookOptions::default()) {
            Ok(report) => {
                assert!(!report.ok());
                format!("{:?}", report.errors())
            }
            Err(error) => error.to_string(),
        };
        assert!(error.contains(expected), "{error}");
        write("content/machine.ts", machine);
        write("modules/north/content/patrol.ts", patrol);
        write("modules/north/content/leaf.ts", leaf);
    }
    write(
        "assets/old.ogt",
        "type behaviour\nversion 1\n---\ntree { stop }\n",
    );
    assert!(!cook_project(&root, &CookOptions::default()).unwrap().ok());
    std::fs::remove_dir_all(root.join("content")).unwrap();
    std::fs::remove_dir_all(root.join("modules")).unwrap();
    // Loading an already cooked machine needs none of its author/subtree files.
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let blob = bundle.read("base/machine").unwrap();
    let b = og_ai::Behaviour::from_bytes(&bundle.behaviour_data("base/machine", &blob).unwrap())
        .unwrap();
    assert_eq!(b.subtrees, ["north/patrol", "north/leaf"]);
}

/// Both author branches pack native stories, bind resources, and resume by
/// stable identity. Explicit text imports invalidate warm cooks; loose Yarn
/// is not an author entry.
#[test]
fn typed_stories_and_explicit_yarn_cook_native_and_resume() {
    use og_story::{Library, Story, asset::Asset};
    use std::sync::Arc;
    let root = crate::temp::TempDir::new("native-stories");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    let project = "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'stories',entry:'src/main.ts',storyFunctions:{price:{returns:'number',parameters:[{name:'cost',kind:'number'},{name:'discount',kind:'number'}],body:{kind:'binary',op:'sub',left:{kind:'variable',name:'cost'},right:{kind:'binary',op:'mul',left:{kind:'variable',name:'discount'},right:{kind:'literal',value:2}}}}}});";
    write("project.ts", project);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(2, 2, image::Rgba([17, 89, 231, 255]))
        .save(root.join("assets/portrait.png"))
        .unwrap();
    crate::declare_import(&root, "portrait", "texture", "assets/portrait.png");
    let typed = r#"import {file,ref} from '@origingame/engine/author';
export default file.dialogue({name:'typed',start:'Start',vars:[{name:'gold',value:7,global:true}],
speakers:[{id:'Smith',portrait:ref('base/portrait')}],strings:{hello:'Welcome',buy:'Buy'},
nodes:[{id:'Start',steps:[{kind:'line',id:'greeting',speaker:'Smith',text:{key:'hello'}}],
choices:[{id:'buy',text:{key:'buy'},steps:[{kind:'set',name:'gold',value:{kind:'call',name:'price',args:[{kind:'literal',value:7},{kind:'literal',value:2}]}},
{kind:'command',name:'bought',args:[{kind:'variable',name:'gold'}]},{kind:'end'}]}]}]});"#;
    write("content/typed.ts", typed);
    let yarn = "#og-profile:yarn-spinner-3.2.2\n#og-name:imported\ntitle: Start\nog-speaker: Smith||portrait.png|left\n---\n<<declare $gold = 7>>\nSmith: Welcome #line:hello\n-> Buy #line:buy\n    <<set $gold = price(7, 2)>>\n    <<bought {$gold}>>\n    <<stop>>\n===\n";
    write("sources/story.yarn", yarn);
    write("assets/ignored.yarn", "not valid Yarn and never scanned");
    write(
        "content/imported.ts",
        "import {file,input,ref} from '@origingame/engine/author';export default file.dialogue({yarn:input.text('sources/story.yarn'),assets:{'portrait.png':ref('base/portrait')}});",
    );
    let quest = "import {file,ref} from '@origingame/engine/author';export default file.quest({id:'rescue',title:'quest.rescue',globals:{rescued:false},rewards:[{name:'reward',image:ref('base/portrait')}],stages:[{id:'find',objectives:[{id:'friend',text:{key:'quest.find'},when:{kind:'variable',name:'rescued'}}]}]});";
    write("content/quest.ts", quest);
    let mut previous = None;
    for (iteration, text, expected_gold) in [
        (0, "Welcome", 3.0),
        (1, "Welcome", 3.0),
        (2, "Changed", 3.0),
        (3, "Changed", 11.0),
    ] {
        write(
            "project.ts",
            &if iteration == 3 {
                project.replace("op:'sub'", "op:'add'")
            } else {
                project.into()
            },
        );
        write("sources/story.yarn", &yarn.replace("Welcome", text));
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let load = |id| {
            let bytes = bundle.read(id).unwrap();
            Asset::from_bytes(&bundle.story_data(id, &bytes).unwrap()).unwrap()
        };
        let Asset::Quest(quest) = load("base/quest") else {
            panic!("quest")
        };
        assert_eq!(quest.rewards[0].image, "base/portrait");
        assert!(quest.vars[0].global);
        let Asset::Dialogue(imported) = load("base/imported") else {
            panic!("dialogue")
        };
        assert_eq!(imported.strings["hello"], format!("Smith: {text}"));
        let packed = bundle.read("base/imported").unwrap().to_vec();
        if let Some(before) = &previous {
            assert_eq!(before == &packed, iteration == 1);
        }
        previous = Some(packed);
        let Asset::Dialogue(typed) = load("base/typed") else {
            panic!("dialogue")
        };
        for dialogue in [typed, imported] {
            assert_eq!(dialogue.speakers[0].portrait, "base/portrait");
            let name = dialogue.name.clone();
            let lib = Library {
                dialogues: BTreeMap::from([(name.clone(), Arc::new(dialogue))]),
                ..Default::default()
            };
            let mut story = Story::new(og_core::Rng::new(19).fork("story"));
            let id = story
                .start(&lib, &name, BTreeMap::new(), BTreeMap::new(), None)
                .unwrap();
            let mut bytes = Vec::new();
            story.save(&mut bytes);
            let mut restored = Story::load(&bytes).unwrap();
            for state in [&mut story, &mut restored] {
                let shown = &state.convs[&id].history[0];
                assert_eq!(
                    shown.line_id,
                    if name == "typed" { "greeting" } else { "hello" }
                );
                assert_eq!(shown.key, "hello");
                state.choose(&lib, id, 0).unwrap();
                assert_eq!(state.globals["gold"], og_story::Val::Num(expected_gold));
                assert!(state.events.iter().any(|e| matches!(e, og_story::Event::Emit {name, arg, ..} if name == "bought" && arg == &expected_gold.to_string())));
                assert_eq!(state.convs[&id].wait, og_story::Wait::Ended);
            }
            let mut a = Vec::new();
            let mut b = Vec::new();
            story.save(&mut a);
            restored.save(&mut b);
            assert_eq!(a, b);
        }
    }
    for (from, to, expected) in [
        ("ref('base/portrait')", "ref('base/absent')", "base/absent"),
        ("id:'Start',steps", "id:'Missing',steps", "start"),
        ("value:7}", "value:true}", "expected"),
    ] {
        write("content/typed.ts", &typed.replace(from, to));
        let error = cook_project(&root, &CookOptions::default())
            .err()
            .unwrap()
            .to_string();
        assert!(
            error.contains("content/typed.ts:") && error.contains(expected),
            "{error}"
        );
    }
    write("content/typed.ts", typed);
    write(
        "content/quest.ts",
        &quest.replace("kind:'variable',name:'rescued'", "kind:'literal',value:2"),
    );
    let error = cook_project(&root, &CookOptions::default())
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("content/quest.ts:") && error.contains("expected"),
        "{error}"
    );
    write("content/quest.ts", quest);
    write("project.ts", &project.replace("value:2", "value:true"));
    let error = cook_project(&root, &CookOptions::default())
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("price"), "{error}");
}

/// Modules, delivery and residency are independent: selecting a package must
/// neither rebuild another region nor turn a build-only input into a download.
#[test]
fn delivery_packs_partition_loadable_assets_and_keep_partial_cooks_scoped() {
    let root = crate::temp::TempDir::new("delivery-packs");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    let project = "import {defineProject,ref} from '@origingame/engine/author';export default defineProject({id:'packs',entry:'src/main.ts',scenes:{main:ref('base/boot')},scene:'main',packs:[{id:'base',dependsOn:['shared']},{id:'shared',include:['base/shared/**'],roots:[ref('base/shared/room')]},{id:'north',include:['base/north/**'],dependsOn:['shared'],roots:[ref('base/north/room')]},{id:'expansion',include:['base/expansion/**'],dependsOn:['base'],optional:true,roots:[ref('base/expansion/room')]}]});";
    write("project.ts", project);
    let scene = "import {file} from '@origingame/engine/author';export default file.scene({name:'shared',entities:[{name:'shared-marker'}]});";
    write("content/shared/room.ts", scene);
    write(
        "content/expansion/room.ts",
        &scene.replace("shared-marker", "expansion-marker"),
    );
    let referring = "import {file,ref} from '@origingame/engine/author';export default file.scene({entities:[{name:'placed-room',scene:ref('base/shared/room')}]});";
    write("content/boot.ts", referring);
    write("content/north/room.ts", referring);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    assert_eq!(report.delivery.len(), 4);
    let base = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        base.manifest()
            .outputs()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        ["base/boot"]
    );
    let required = &base.manifest().requires;
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].file, "delivery/shared/.cook.ogt");
    let bytes = std::fs::read(root.join("cooked").join(&required[0].file)).unwrap();
    assert_eq!(required[0].integrity, og_cook::cook::integrity(&bytes));
    let north_path = root.join("cooked/delivery/north");
    let north = Bundle::open(&north_path).unwrap();
    assert_eq!(north.manifest().requires[0].file, "../shared/.cook.ogt");
    assert!(north.read("base/north/room").is_ok());
    assert!(north.output("base/shared/room").is_err());
    let north_bytes = north.read("base/north/room").unwrap();
    let pack = |id: &str| report.delivery.iter().find(|p| p.id == id).unwrap();
    assert!(pack("base").boot && pack("shared").boot);
    assert!(!pack("north").boot && !pack("expansion").boot);
    assert_eq!(
        pack("north").cross_pack_refs["base/north/room"]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["base/shared/room"]
    );
    write(
        "content/north/room.ts",
        &referring.replace("placed-room", "changed-room"),
    );
    write(
        "content/shared/room.ts",
        &scene.replace("shared-marker", "updated-shared"),
    );
    let selected = CookOptions {
        pack: Some("shared".into()),
        ..Default::default()
    };
    let report = cook_project(&root, &selected).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    assert_eq!(report.pack.as_deref(), Some("shared"));
    assert_eq!(
        report
            .delivery
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        ["shared"]
    );
    assert!(
        report
            .cooked_outputs()
            .iter()
            .all(|o| o.id.starts_with("base/shared/"))
    );
    assert_eq!(
        Bundle::open(&north_path)
            .unwrap()
            .read("base/north/room")
            .unwrap()
            .to_vec(),
        north_bytes.to_vec()
    );
    let north = Bundle::open(&north_path).unwrap();
    assert_eq!(
        north.manifest().requires[0].integrity,
        og_cook::cook::integrity(
            &std::fs::read(root.join("cooked/delivery/shared/.cook.ogt")).unwrap()
        )
    );
    let warm = cook_project(&root, &selected).unwrap();
    assert!(warm.ok() && warm.cooked_outputs().is_empty());
    let full = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(full.ok(), "{:?}", full.errors());
    assert_ne!(
        Bundle::open(&north_path)
            .unwrap()
            .read("base/north/room")
            .unwrap()
            .to_vec(),
        north_bytes.to_vec()
    );
    // Identical local stream names remain independent delivery units.
    for region in ["north", "expansion"] {
        write(
            &format!("content/{region}/cell.ts"),
            "import {file} from '@origingame/engine/author';export default file.sfx({preset:'click',seed:17});",
        );
        write(
            &format!("content/{region}/room.ts"),
            &format!(
                "import {{file,ref}} from '@origingame/engine/author';export default file.scene({{streams:{{cells:{{assets:[ref('base/{region}/cell')]}}}}}});"
            ),
        );
    }
    let streamed = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(streamed.ok(), "{:?}", streamed.errors());
    for region in ["north", "expansion"] {
        let dir = root.join(format!("cooked/delivery/{region}"));
        let bundle = Bundle::open(&dir).unwrap();
        assert_eq!(bundle.manifest().streams[0].name, format!("{region}/cells"));
        let report = streamed.delivery.iter().find(|p| p.id == region).unwrap();
        assert!(report.stream_bytes > 0);
        assert_eq!(
            report.bytes,
            report
                .files
                .iter()
                .map(|f| std::fs::metadata(root.join("cooked").join(f))
                    .unwrap()
                    .len())
                .sum::<u64>()
        );
    }
    // Restore the cross-pack reference for the dependency rejection below.
    write("content/north/room.ts", referring);
    for (invalid, expected) in [
        (
            project.replace(
                "id:'north',include:['base/north/**'],dependsOn:['shared']",
                "id:'north',include:['base/north/**'],dependsOn:[]",
            ),
            "dependsOn edge",
        ),
        (
            project.replace("base/north/**", "base/shared/**"),
            "must not overlap",
        ),
        (
            project
                .replace(
                    "id:'base',dependsOn:['shared']",
                    "id:'base',dependsOn:['expansion']",
                )
                .replace(
                    "id:'expansion',include:['base/expansion/**'],dependsOn:['base']",
                    "id:'expansion',include:['base/expansion/**'],dependsOn:[]",
                ),
            "optional",
        ),
    ] {
        write("project.ts", &invalid);
        let error = cook_project(&root, &CookOptions::default())
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}

/// Explicit imports own identity and controls; external model inputs invalidate
/// the same native asset without depending on filename aliases or sidecars.
#[test]
fn typed_imports_capture_external_inputs_and_keep_canonical_identity() {
    let root = crate::temp::TempDir::new("typed-imports");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "content/props/gate.ts",
        "import {file} from '@origingame/engine/author';import {lods} from '../../lib/options.ts';export default file.import({kind:'model',source:'sources/raw/mesh.gltf',lods,conform:'off'});",
    );
    write("lib/options.ts", "export const lods=[1];");
    write(
        "sources/raw/mesh.gltf",
        r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],"buffers":[{"uri":"mesh.bin","byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-2,0,0],"max":[3,4,0]}]}"#,
    );
    let mut prior = None;
    for (threads, top) in [(1, 4.0_f32), (3, 4.0), (2, 7.0)] {
        let positions = [[-2.0_f32, 0.0, 0.0], [3.0, 0.0, 0.0], [1.0, top, 0.0]];
        std::fs::write(
            root.join("sources/raw/mesh.bin"),
            bytemuck::cast_slice(&positions),
        )
        .unwrap();
        let options = CookOptions {
            threads,
            ..Default::default()
        };
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let b = Bundle::open(&root.join("cooked")).unwrap();
        let model = b.load_model("base/props/gate").unwrap();
        assert_eq!(model.primitives[0].max.y, top);
        assert!(b.load_model("raw/mesh.gltf").is_err());
        let bytes = b.read("base/props/gate").unwrap().to_vec();
        if let Some(previous) = &prior {
            if top == 4.0 {
                assert_eq!(&bytes, previous);
            } else {
                assert_ne!(&bytes, previous);
            }
        }
        prior = Some(bytes);
    }
    write("lib/options.ts", "export const lods=[0.5,1];");
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!report.ok(), "changed imported settings were not checked");
    write("lib/options.ts", "export const lods=[1];");
    write(
        "assets/import.ogt",
        "type import\nversion 1\n---\nmatch \"*.png\" {texture {kind pixel}}\n",
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!report.ok(), "removed directory defaults were accepted");
    std::fs::remove_file(root.join("assets/import.ogt")).unwrap();
    write(
        "sources/raw/mesh.gltf",
        r#"{"asset":{"version":"2.0"},"buffers":[{"uri":"../../../outside.bin","byteLength":36}]}"#,
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!report.ok(), "external buffer escaped its owning module");
}

/// Table data is packed, not author text; cyclic row identities, defaults and
/// localized strings survive both cold and warm cooks and schema edits.
#[test]
fn typed_tables_pack_rows_and_validate_cross_table_identities() {
    let root = crate::temp::TempDir::new("typed-tables");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'tables',entry:'src/main.ts',schemas:['src/schema/items.ts']});",
    );
    let schema = "import * as s from '@origingame/engine/schema';export const Item=s.defineComponent('Item',{power:s.u32(17,{min:1,max:90}),label:s.str(24,'北'),icon:s.str(64),image:s.Texture(),next:s.TableRow(),links:s.list(s.TableRow(),4)});";
    write("src/schema/items.ts", schema);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(2, 2, image::Rgba([17, 89, 231, 255]))
        .save(root.join("assets/icon.png"))
        .unwrap();
    crate::declare_import(&root, "icon", "texture", "assets/icon.png");
    let table = "import {file,ref} from '@origingame/engine/author';\nexport default file.table({record:'Item',rows:[\n{id:'bow',values:{label:{text:{key:'items.bow',base:'Bow'}},icon:{asset:ref('base/icon')},image:ref('base/icon'),next:{table:ref('base/materials'),row:'wood'},links:[{row:'arrow'}]}},\n{id:'arrow',values:{power:3}}]});";
    write("content/items.ts", table);
    write(
        "content/materials.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.table({record:'Item',rows:[{id:'wood',values:{next:{table:ref('base/items'),row:'bow'}}}]});",
    );
    let mut previous = None;
    for (threads, default) in [(1, 17), (3, 17), (2, 29)] {
        if default == 29 {
            write("src/schema/items.ts", &schema.replace("u32(17", "u32(29"));
        }
        let report = cook_project(
            &root,
            &CookOptions {
                threads,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/items").unwrap();
        let native = bundle.table("base/items", &bytes).unwrap();
        assert_eq!(
            native
                .rows
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            ["bow", "arrow"]
        );
        let layout = og_core::Layout::of(&native.schema);
        let field = |i: usize, name| {
            &native.rows[i].bytes[layout.range(native.schema.field_index(name).unwrap())]
        };
        assert_eq!(
            u32::from_le_bytes(field(0, "power").try_into().unwrap()),
            default
        );
        assert_eq!(u32::from_le_bytes(field(1, "power").try_into().unwrap()), 3);
        assert_eq!(og_core::unpack_str(field(1, "label")), "北");
        assert_eq!(og_core::unpack_str(field(0, "label")), "items.bow");
        assert_eq!(native.strings["items.bow"], "Bow");
        assert_eq!(og_core::unpack_str(field(0, "icon")), "base/icon");
        assert_eq!(native.bindings[&0]["image"]["$ogRef"], "base/icon");
        assert_eq!(
            u64::from_le_bytes(field(0, "next").try_into().unwrap()),
            og_core::table::row_handle("base/materials", "wood")
        );
        let links = &native.schema.fields[native.schema.field_index("links").unwrap()];
        assert_eq!(
            og_core::list_elements(links.ty, field(0, "links"))
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
                .collect::<Vec<_>>(),
            [og_core::table::row_handle("base/items", "arrow")]
        );
        if let Some(was) = &previous {
            if default == 17 {
                assert_eq!(was, &bytes.to_vec());
            } else {
                assert_ne!(was, &bytes.to_vec());
            }
        }
        previous = Some(bytes.to_vec());
    }
    for (from, to, message, line) in [
        ("row:'wood'", "row:'absent'", "has no row 'absent'", 3),
        ("id:'arrow'", "id:'bow'", "duplicate row id bow", 2),
        ("power:3", "power:91", "90", 4),
    ] {
        write("content/items.ts", &table.replace(from, to));
        let error = cook_project(&root, &CookOptions::default())
            .err()
            .unwrap()
            .to_string();
        assert!(
            error.contains(message) && error.contains(&format!("content/items.ts:{line}:")),
            "{error}"
        );
    }
}

/// Checked static and animated declarations share native frame, timing and
/// subresource contracts, including captured exchange inputs on warm cooks.
#[test]
fn typed_flipbooks_cook_native_sheets_and_arrays() {
    use og_anim::{FlipMode, cooked::BundleAnim};
    let root = crate::temp::TempDir::new("typed-flipbooks");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject,ref} from '@origingame/engine/author';export default defineProject({id:'flipbooks',entry:'src/main.ts',roots:[ref('base/grid#array#layer5'),ref('base/animated#array')]});",
    );
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::create_dir_all(root.join("sources")).unwrap();
    image::RgbaImage::from_pixel(13, 9, image::Rgba([17, 89, 231, 255]))
        .save(root.join("assets/sheet.png"))
        .unwrap();
    let animation = |delays: [u32; 2]| {
        let file = std::fs::File::create(root.join("sources/idle.gif")).unwrap();
        let mut encoder = image::codecs::gif::GifEncoder::new(file);
        for ms in delays {
            encoder
                .encode_frame(image::Frame::from_parts(
                    image::RgbaImage::from_pixel(13, 9, image::Rgba([193, 51, 7, 255])),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(ms, 1),
                ))
                .unwrap();
        }
    };
    animation([100, 300]);
    write(
        "content/sheet.ts",
        "import {file} from '@origingame/engine/author';export default file.import({kind:'texture',source:'assets/sheet.png',conform:{kind:'off'},usage:'pixel'});",
    );
    let grid = "import {file,ref} from '@origingame/engine/author';export default file.flipbook({texture:ref('base/sheet'),conform:{kind:'off'},array:true,grid:{size:[3,2],columns:3,rows:2},margin:1,spacing:1,pivot:[0.25,0.75],clips:[{name:'stride',frames:['4','1','5'],durations:[0.125,0.25,0.5],mode:'pingPong',events:[[1,'step']]}]});";
    write("content/grid.ts", grid);
    write(
        "content/atlas.ts",
        "import {file,ref,input} from '@origingame/engine/author';export default file.flipbook({texture:ref('base/sheet'),conform:{kind:'off'},atlas:input.text('sources/atlas.json')});",
    );
    let atlas = r#"{"frames":{"run2":{"frame":{"x":4,"y":1,"w":3,"h":2},"duration":125,"pivot":{"x":0.25,"y":0.75}},"run1":{"frame":{"x":0,"y":0,"w":3,"h":2},"duration":250}},"meta":{"frameTags":[{"name":"run","from":0,"to":1,"direction":"reverse"}]}}"#;
    write("sources/atlas.json", atlas);
    write(
        "content/animated.ts",
        "import {file} from '@origingame/engine/author';export default file.flipbook({sources:[{source:'sources/idle.gif',clip:'pose',events:[[0,'ready']]}],conform:{kind:'off'},array:true,padding:0,align:'none',pivot:[0.2,0.8]});",
    );
    // This ref must bind the producer's actual #sheet output, not a guessed file.
    write(
        "content/derived.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.flipbook({texture:ref('base/animated#sheet'),conform:{kind:'off'},grid:{size:[13,9]},clips:[{name:'pose',frames:['0'],fps:4}]});",
    );
    let opts = CookOptions::default();
    let mut previous = None;
    for warm in [false, true] {
        let report = cook_project(&root, &opts).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let b = Bundle::open(&root.join("cooked")).unwrap();
        let sheet = b.load_flipbook("base/grid").unwrap();
        assert_eq!(sheet.texture, "base/sheet");
        assert_eq!(sheet.size, [13, 9]);
        assert_eq!(sheet.frames[4].rect, [5, 4, 3, 2]);
        assert_eq!(sheet.frames[4].pivot, [0.25, 0.75]);
        assert_eq!(sheet.clips[0].frames, [4, 1, 5]);
        assert_eq!(sheet.clips[0].durations, [0.125, 0.25, 0.5]);
        assert_eq!(sheet.clips[0].mode, FlipMode::PingPong);
        assert_eq!(sheet.clips[0].events, [(1, "step".into())]);
        let imported = b.load_flipbook("base/atlas").unwrap();
        assert_eq!(imported.frames[1].name, "run2");
        assert_eq!(imported.frames[1].pivot, [0.25, 0.25]);
        assert_eq!(imported.clips[0].frames, [1, 0]);
        assert_eq!(
            imported.clips[0].durations,
            if warm {
                vec![0.375, 0.25]
            } else {
                vec![0.125, 0.25]
            }
        );
        let animated = b.load_flipbook("base/animated").unwrap();
        assert_eq!(animated.texture, "base/animated#sheet");
        assert_eq!(
            animated.clips[0].durations,
            if warm { [0.2, 0.4] } else { [0.1, 0.3] }
        );
        assert_eq!(animated.clips[0].events, [(0, "ready".into())]);
        assert_eq!(animated.frames[0].pivot, [0.2, 0.8]);
        assert_eq!(b.load_flipbook("base/derived").unwrap().size, [13, 9]);
        for id in [
            "base/grid#array",
            "base/grid#array#layer5",
            "base/animated#array",
            "base/animated#array#layer0",
        ] {
            assert!(b.read(id).is_ok(), "missing {id}");
        }
        let bytes = b.read("base/atlas").unwrap().to_vec();
        if warm {
            assert_ne!(previous.as_ref().unwrap(), &bytes);
        }
        previous = Some(bytes);
        write("sources/atlas.json", &atlas.replace("125", "375"));
        // Same packed image, different file delays: the native timing must recook.
        animation([200, 400]);
    }
    for invalid in [
        grid.replace("'4','1','5'", "'missing'"),
        grid.replace("columns:3", "columns:4"),
        grid.replace("spacing:1", "spacing:4294967295"),
    ] {
        write("content/grid.ts", &invalid);
        let error = match cook_project(&root, &opts) {
            Ok(report) => {
                assert!(!report.ok());
                format!("{:?}", report.errors())
            }
            Err(error) => error.to_string(),
        };
        assert!(error.contains("grid"), "{error}");
    }
    write("content/grid.ts", grid);
    write(
        "assets/old.ogt",
        "type flipbook\nversion 1\n---\ntexture sheet.png\ngrid 3 2\n",
    );
    assert!(!cook_project(&root, &opts).unwrap().ok());
}

/// Recipes enter through the restricted VM and load as native sheets, with
/// exact timing, captured helper edits, actual texture dependencies and no OGT.
#[test]
fn typed_pixel_effects_cook_native_frames_and_events() {
    use og_anim::{FlipMode, SheetFilter, cooked::BundleAnim};
    let root = crate::temp::TempDir::new("typed-pixel-effects");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    let declaration = "import {file} from '@origingame/engine/author';\nimport {width} from '../lib/shape.ts';\nexport default file.vfx2d({duration:0.75,loop:true,impact:0.2,events:[{time:0.5,name:'尾😀'}],recipe:{w:width,h:12,frames:5,fps:19,pivot:[0.25,0.75],supersample:2,scale:0.75,line:1.5,innerLine:0.5,layers:[{type:'burst',r0:0.1,r1:0.6}]}});";
    write("content/hit.ts", declaration);
    write("lib/shape.ts", "export const width=8;");
    let opts = CookOptions {
        threads: 2,
        ..Default::default()
    };
    let mut previous = None;
    for warm in [false, true, true] {
        let report = cook_project(&root, &opts).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let b = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = b.read("base/hit").unwrap().to_vec();
        if warm {
            assert_eq!(previous.as_ref().unwrap(), &bytes);
        }
        let sheet = b.load_flipbook("base/hit").unwrap();
        assert_eq!(sheet.texture, "base/hit#sheet");
        assert!(b.read(&sheet.texture).is_ok());
        assert_eq!((sheet.size, sheet.filter), ([40, 12], SheetFilter::Linear));
        assert_eq!(sheet.frames[3].rect, [24, 0, 8, 12]);
        assert_eq!(sheet.frames[3].pivot, [0.25, 0.25]);
        let clip = &sheet.clips[0];
        assert_eq!(clip.name, "hit");
        assert_eq!(clip.mode, FlipMode::Loop);
        assert_eq!(clip.frames, [0, 1, 2, 3, 4]);
        assert_eq!(clip.durations, [0.15; 5]);
        assert_eq!(clip.events, [(2, "impact".into()), (4, "尾😀".into())]);
        if !warm {
            let mut native = Vec::new();
            sheet.encode(&mut native);
            for end in 0..native.len() {
                assert!(og_anim::FlipbookSheet::decode(&native[..end]).is_err());
            }
            native.push(0);
            assert!(og_anim::FlipbookSheet::decode(&native).is_err());
        }
        previous = Some(bytes);
        if warm {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
    }
    write("lib/shape.ts", "export const width=13;");
    let report = cook_project(&root, &opts).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(b.load_flipbook("base/hit").unwrap().size, [65, 12]);
    assert_ne!(previous.unwrap(), b.read("base/hit").unwrap().to_vec());
    for invalid in [
        declaration.replace("impact:0.2", "impact:0.76"),
        declaration.replace("duration:0.75", "duration:0"),
        declaration.replace("r0:0.1", "typo:0.1"),
    ] {
        write("content/hit.ts", &invalid);
        let error = match cook_project(&root, &opts) {
            Ok(report) => {
                assert!(!report.ok(), "accepted invalid recipe");
                format!("{:?}", report.errors())
            }
            Err(error) => error.to_string(),
        };
        assert!(error.contains("content/hit.ts"), "{error}");
    }
    std::fs::remove_file(root.join("content/hit.ts")).unwrap();
    write(
        "assets/old.ogt",
        "type vfx2d\nversion 1\n---\n{\"layers\":[{\"type\":\"burst\"}]}\n",
    );
    assert!(!cook_project(&root, &opts).unwrap().ok());
}

/// Grammars cross the author/native boundary once, including unreferenced
/// validation, imported parameters and warm-cache district dependencies.
#[test]
fn typed_grammars_build_districts_and_reject_invalid_rules() {
    let root = crate::temp::TempDir::new("typed-grammar");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'grammar',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write(
        "content/materials.ts",
        "import {file} from '@origingame/engine/author';export default file.materials({materials:[{name:'stone',labels:['stone','road','pavement','ground','kerb'],surface:{color:[0.2,0.4,0.7],map:'surface.png'}}]});",
    );
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(2, 3, image::Rgba([17, 89, 231, 255]))
        .save(root.join("assets/surface.png"))
        .unwrap();
    let district = "import {file,ref} from '@origingame/engine/author';export default file.district({seed:7,region:{kind:'along',street:{points:[[0,0],[0,24]],width:4,sidewalk:1},depth:10},lots:{width:[6,6],depth:[8,8]},buildings:{density:1,rules:[{grammar:ref('north/building')}]},materials:ref('base/materials')});";
    write("modules/north/content/town.ts", district);
    write(
        "assets/gate.gltf",
        r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"Root"}]}"#,
    );
    crate::declare_import(&root, "gate", "model", "assets/gate.gltf");
    let source = "import {file,ref} from '@origingame/engine/author';\nimport {height} from '../lib/height.ts';\nexport default file.grammar({params:{height},start:'Lot',rules:{\nLot:[{op:'detail',level:0},{op:'extrude',depth:'height',faces:{sides:'Wall',top:'Gate'}}],\nWall:[{op:'label',label:'stone'}],\nGate:[{op:'insert',model:ref('base/gate')}],\nUnused:[{op:'insert',model:ref('base/gate')}]}});";
    write("modules/north/content/building.ts", source);
    let mut options = CookOptions {
        schemas: og_cook::cook::engine_schemas(),
        ..Default::default()
    };
    let mut previous = None;
    for height in [7, 11] {
        write(
            "modules/north/lib/height.ts",
            &format!("export const height={height};"),
        );
        for threads in [1, 3] {
            options.threads = threads;
            let report = cook_project(&root, &options).unwrap();
            assert!(report.ok(), "{:?}", report.errors());
            let bundle = Bundle::open(&root.join("cooked")).unwrap();
            let scene = bundle.load_scene("north/town").unwrap();
            assert!(
                scene
                    .entities
                    .iter()
                    .any(|e| e.instance.as_deref() == Some("base/gate")),
                "grammar insert did not bind the actual imported model output"
            );
            let id = scene
                .entities
                .iter()
                .filter(|e| e.instance.as_deref() != Some("gate.gltf"))
                .find_map(|e| e.instance.as_ref())
                .expect("generated district model");
            let model = bundle.load_model(id).unwrap();
            assert_eq!(model.min.y, 0.0);
            assert_eq!(
                model.max.y, height as f32,
                "imported grammar parameter was stale"
            );
            assert!(!model.primitives.is_empty());
            assert!(model.primitives.iter().all(|p| !p.indices.is_empty()));
            let bytes = bundle.read(id).unwrap().to_vec();
            if threads == 3 {
                assert_eq!(
                    previous.as_ref().unwrap(),
                    &bytes,
                    "warm or parallel cook changed geometry"
                );
            } else if let Some(old) = &previous {
                assert_ne!(old, &bytes, "changed parameter reused old model bytes");
            }
            previous = Some(bytes);
        }
    }
    // Streamed outputs must be discovered before authored scenes bind their
    // references. A module root is not the assets root of its material maps.
    for merge in [true, false] {
        write(
            "modules/north/content/town.ts",
            &district.replace(
                "seed:7",
                &format!("seed:7,cells:{{size:16,load:37}},merge:{merge}"),
            ),
        );
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let scene = bundle.load_scene("north/town").unwrap();
        let cells: Vec<_> = scene
            .entities
            .iter()
            .filter_map(|e| e.subscene.as_deref())
            .collect();
        assert!(cells.len() > 1, "district did not stream across cells");
        let mut generated = 0;
        for id in cells {
            assert!(id.starts_with("north/town#cell-"), "{id}");
            let cell = bundle.load_scene(id).unwrap();
            let model_names: Vec<_> = cell
                .entities
                .iter()
                .filter(|e| {
                    e.instance
                        .as_deref()
                        .is_some_and(|m| m.starts_with("north/town#"))
                })
                .map(|e| e.name.as_str())
                .collect();
            if merge {
                assert_eq!(model_names, [format!("{id}/model")]);
            } else {
                assert!(model_names.iter().all(|name| {
                    *name == format!("{id}/model")
                        || (name.starts_with(&format!("{id}.")) && name.ends_with("/model"))
                }));
            }
            for model in cell
                .entities
                .iter()
                .filter_map(|e| e.instance.as_deref())
                .filter(|id| id.starts_with("north/town#"))
            {
                assert!(bundle.load_model(model).is_ok(), "{model}");
                generated += 1;
            }
        }
        assert!(generated > 1);
    }
    write("modules/north/content/town.ts", district);
    for (from, to, expected) in [
        ("width:[6,6]", "width:[8,4]", "width"),
        (
            "density:1,rules:[{grammar:ref('north/building')}]",
            "density:1",
            "grammar rule",
        ),
        (
            "ref('base/materials')",
            "ref('base/gate')",
            "material library",
        ),
    ] {
        write("modules/north/content/town.ts", &district.replace(from, to));
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(r) => format!("{:?}", r.errors()),
        };
        assert!(
            error.contains(expected) && error.contains("content/town.ts:"),
            "{error}"
        );
    }
    write("modules/north/content/town.ts", district);
    // Unreachable rules still validate; otherwise a broken helper could hide
    // indefinitely behind a seed or a district's rule selection.
    for (from, to, expected) in [
        ("depth:'height'", "depth:'missing'", "param `missing`"),
        ("sides:'Wall'", "sides:'Absent'", "rule `Absent`"),
        ("op:'detail',level:0", "op:'drop'", "exactly one"),
        (
            "Unused:[{op:'insert',model:ref('base/gate')}]",
            "Unused:[{op:'recess',depth:1}]",
            "exactly one",
        ),
        ("ref('base/gate')", "ref('base/absent')", "base/absent"),
    ] {
        write(
            "modules/north/content/building.ts",
            &source.replace(from, to),
        );
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/building.ts:"), "{error}");
    }
    write("modules/north/content/building.ts", source);
    for old in [
        "type grammar\nversion 1\n---\nstart A\nrule A {\n drop\n}\n",
        "type district\nversion 1\n---\n",
    ] {
        write("assets/old.ogt", old);
        assert!(
            !cook_project(&root, &options).unwrap().ok(),
            "old author input was accepted"
        );
    }
}

/// Checked tileset data, cross-module atlas references and embedded imported
/// tilesets retain their meaning through bundle loading and world snapshots.
#[test]
fn typed_tilesets_preserve_shapes_animation_terrain_and_import_bindings() {
    use og_tilemap::{Collision, Filter, TilemapAsset, Tileset, TilesetRef};
    let root = crate::temp::TempDir::new("typed-tileset");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'tiles',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(53, 31, image::Rgba([17, 83, 149, 255]))
        .save(root.join("assets/atlas.png"))
        .unwrap();
    crate::declare_import(&root, "atlas", "texture", "assets/atlas.png");
    write("modules/north/lib/timing.ts", "export const hold=0.375;");
    let declaration = "import {file,ref} from '@origingame/engine/author';\nimport {hold} from '../lib/timing.ts';\nexport default file.tileset({texture:ref('base/atlas'),tileWidth:7,tileHeight:11,columns:3,rows:2,margin:2,spacing:1,filter:'linear',conform:{kind:'off'},tiles:{0:{collision:{kind:'rect',x:-1,y:0,w:9,h:11}},2:{collision:{kind:'rect',x:1,y:3,w:4,h:5},polygons:[[[-2,1],[3,2],[1,9]]],animation:{frames:[4,1],durations:[0.125,hold]}}},terrains:[{name:'泥😀',mode:'blob47',default:5,edgeFilled:false,rules:{0:1,7:4}},{name:'edges',mode:'bitmask16',default:2,rules:{9:3}}]});";
    write("modules/north/content/tiles.ts", declaration);
    write(
        "modules/north/assets/map.tmj",
        r#"{"type":"map","orientation":"orthogonal","width":2,"height":1,"tilewidth":7,"tileheight":11,"tilesets":[{"firstgid":1,"source":"external.tsx"},{"firstgid":10,"name":"内蔵","image":"atlas.png","tilewidth":7,"tileheight":11,"columns":3,"tilecount":6,"margin":2,"spacing":1,"tiles":[{"id":1,"properties":[{"name":"solid","type":"bool","value":true}],"animation":[{"tileid":4,"duration":125},{"tileid":2,"duration":375}]}]}],"layers":[{"type":"tilelayer","name":"outside","width":2,"height":1,"data":[1,3]},{"type":"tilelayer","name":"inside","width":2,"height":1,"data":[11,10]}]}"#,
    );
    write(
        "modules/north/content/map.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.import({kind:'tilemap',source:'assets/map.tmj',tilesets:{'external.tsx':ref('north/tiles')},textures:{'atlas.png':ref('base/atlas')}});",
    );
    let options = CookOptions::default();
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let b = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = b.read("north/tiles").unwrap();
        if let Some(old) = previous {
            assert_eq!(
                old,
                bytes.to_vec(),
                "a warm cook changed native tileset bytes"
            );
        }
        previous = Some(bytes.to_vec());
        let t = b.tileset("north/tiles", &bytes).unwrap();
        assert_eq!(
            (t.texture.as_str(), t.filter),
            ("base/atlas", Filter::Linear)
        );
        assert_eq!(
            (
                t.tile_width,
                t.tile_height,
                t.columns,
                t.rows,
                t.margin,
                t.spacing
            ),
            (7, 11, 3, 2, 2, 1)
        );
        assert_eq!(t.collision(0), Some(Collision::Full));
        assert_eq!(
            t.collision(2),
            Some(Collision::Rect {
                x: 1.0,
                y: 3.0,
                w: 4.0,
                h: 5.0
            })
        );
        assert_eq!(
            t.tiles[&2].polygons,
            [[[-2.0, 1.0], [3.0, 2.0], [1.0, 9.0]]]
        );
        let anim = t.tiles[&2].anim.as_ref().unwrap();
        assert_eq!(anim.frames, [4, 1]);
        assert_eq!(anim.durations, [0.125, 0.375]);
        assert_eq!(
            (
                t.terrains[0].name.as_str(),
                t.terrains[0].default,
                t.terrains[0].edge_filled
            ),
            ("泥😀", 5, false)
        );
        assert_eq!(t.terrains[0].rules, BTreeMap::from([(0, 1), (7, 4)]));
        assert_eq!(t.terrains[1].rules, BTreeMap::from([(9, 3)]));
        let mut native = Vec::new();
        t.encode(&mut native);
        assert_eq!(Tileset::decode(&native).unwrap(), t);
        for end in 0..native.len() {
            assert!(
                Tileset::decode(&native[..end]).is_err(),
                "accepted truncated tileset at {end}"
            );
        }
        native.push(0);
        assert!(Tileset::decode(&native).is_err());
        let map = b.load_tilemap("north/map").unwrap();
        assert!(matches!(&map.tilesets[0], TilesetRef::Asset(id) if id=="north/tiles"));
        let TilesetRef::Embedded(embedded) = &map.tilesets[1] else {
            panic!("missing embedded tileset")
        };
        assert_eq!(
            (embedded.texture.as_str(), embedded.columns, embedded.rows),
            ("base/atlas", 3, 2)
        );
        assert_eq!(embedded.collision(1), Some(Collision::Full));
        assert_eq!(embedded.tiles[&1].anim.as_ref().unwrap().frames, [4, 2]);
        assert_eq!(
            embedded.tiles[&1].anim.as_ref().unwrap().durations,
            [0.125, 0.375]
        );
        assert_eq!(map.layers[0].cells, [1, 3]);
        assert_eq!(map.layers[1].cells, [2, 1]);
        let mut snapshot = Vec::new();
        map.encode(&mut snapshot);
        assert_eq!(TilemapAsset::decode(&snapshot).unwrap(), map);
    }
    write("modules/north/lib/timing.ts", "export const hold=0.625;");
    assert!(cook_project(&root, &options).unwrap().ok());
    assert_eq!(
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_tileset("north/tiles")
            .unwrap()
            .tiles[&2]
            .anim
            .as_ref()
            .unwrap()
            .durations,
        [0.125, 0.625]
    );
    for (from, to, expected) in [
        ("frames:[4,1]", "frames:[6,1]", "names tile 6"),
        ("durations:[0.125,hold]", "durations:[hold]", "duration"),
        ("0:1,7:4", "0:1,2:4", "mask"),
        ("base/atlas", "base/missing", "base/missing"),
    ] {
        write(
            "modules/north/content/tiles.ts",
            &declaration.replace(from, to),
        );
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/tiles.ts:3"), "{error}");
    }
    write("modules/north/content/tiles.ts", declaration);
    crate::declare_import(
        &root.join("modules/north"),
        "map",
        "tilemap",
        "assets/map.tmj",
    );
    let error = cook_project(&root, &options).err().unwrap().to_string();
    assert!(error.contains("external.tsx"), "{error}");
    std::fs::remove_file(root.join("modules/north/content/map.ts")).unwrap();
    std::fs::remove_file(root.join("modules/north/assets/map.tmj")).unwrap();
    write("assets/old.ogt", "type tileset\nversion 1\n---\n");
    assert!(
        !cook_project(&root, &options).unwrap().ok(),
        "old tileset author parser accepted input"
    );
    std::fs::remove_file(root.join("assets/old.ogt")).unwrap();
    write(
        "og-project.json",
        r#"{"version":1,"name":"Tiles","engine":"0.9.0","look":{"$ogRef":"base/look"}}"#,
    );
    write(
        "content/look.ts",
        "import {file} from '@origingame/engine/author'; export default file.look({ArtPixels:{unit:2}, ArtPalette:{maxColors:64,rampSteps:6}});",
    );
    for (conform, expected) in [("kind:'off'", (7, 11)), ("kind:'size',size:4", (3, 4))] {
        let source = declaration.replace("kind:'off'", conform);
        write("modules/north/content/tiles.ts", &source);
        for _ in 0..2 {
            let report = cook_project(&root, &options).unwrap();
            assert!(report.ok(), "{:?}", report.errors());
            let t = Bundle::open(&root.join("cooked"))
                .unwrap()
                .load_tileset("north/tiles")
                .unwrap();
            assert_eq!((t.tile_width, t.tile_height), expected);
            assert_eq!(t.collision(0), Some(Collision::Full));
            assert_eq!(t.tiles[&2].anim.as_ref().unwrap().durations, [0.125, 0.625]);
            if expected == (3, 4) {
                assert!(
                    Bundle::open(&root.join("cooked"))
                        .unwrap()
                        .read("base/atlas#index")
                        .is_ok(),
                    "palette conformance must retain the derived index texture"
                );
            }
            assert_eq!(
                std::fs::read_to_string(root.join("modules/north/content/tiles.ts")).unwrap(),
                source,
                "conformance changed author source"
            );
        }
    }
}

/// Lighting's public cook/load contract retains scene associations, full-width
/// hashes, remaps, multiple layers and volumes across module and warm-cache use.
#[test]
fn typed_lightmaps_preserve_bindings_and_capture_page_dependencies() {
    use og_cook::lightmap::{f32_to_f16, pages_to_ktx2};
    let root = crate::temp::TempDir::new("typed-lightmap");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'lighting',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write(
        "content/room.ts",
        "import {file} from '@origingame/engine/author';export default file.scene({entities:[{name:'Room'}]});",
    );
    write(
        "assets/mesh.gltf",
        r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"Root"}]}"#,
    );
    crate::declare_import(&root, "mesh", "model", "assets/mesh.gltf");
    write(
        "modules/north/lib/irradiance.ts",
        "export const ambient=0.375;",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nimport {ambient} from '../lib/irradiance.ts';\nexport default file.lightmap({scene:ref('base/room'),sceneHash:'fedcba9876543210',pages:'assets/pages.ktx2',samples:83,seconds:1.25,
      models:[{entity:'北😀',asset:ref('base/mesh'),hash:'ffffffffffffffff',parts:[
        {vertices:5,xref:[4,1,3],indices:[2,0,1],uv:new Float32Array([0.125,0.25,1,0.5,0.75,0,1,0,1])},
        {vertices:2,xref:[1],uv:[0.25,0.5,0]}]},
        {entity:'',asset:{hlodProxy:7},hash:'123456789abcdef0',parts:[]},
        {entity:'',asset:{hlodProxy:2},hash:'76543210',parts:[]}],
      volumes:[{entity:'red',min:[-2,0,-1],max:[3,4,2],fade:0.625,counts:[2,1,1],probes:new Float32Array([1,2,3,ambient,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24])},
        {entity:'blue',min:[0,1,2],max:[0,2,3],counts:[1,1,1],probes:[25,26,27,28,29,30,31,32,33,34,35,36]}]});";
    write("modules/north/content/lighting.ts", declaration);
    let options = CookOptions::default();
    let mut previous = None;
    for (step, intensity) in [0.25, 0.25, 0.75].into_iter().enumerate() {
        let pages = vec![vec![f32_to_f16(intensity); 8], vec![f32_to_f16(1.5); 8]];
        let ktx2 = pages_to_ktx2(2, 1, &pages);
        let path = root.join("modules/north/assets/pages.ktx2");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, &ktx2).unwrap();
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let b = Bundle::open(&root.join("cooked")).unwrap();
        assert_eq!(
            b.manifest().lightmaps.get("base/room").map(String::as_str),
            Some("north/lighting")
        );
        let bytes = b.read("north/lighting").unwrap();
        let lm = b.lightmap("north/lighting", &bytes).unwrap();
        assert_eq!(
            (lm.file.scene.as_str(), lm.file.scene_hash),
            ("base/room", 0xfedcba9876543210)
        );
        assert_eq!((lm.file.samples, lm.file.seconds), (83, 1.25));
        assert_eq!((lm.width, lm.height), (2, 1));
        assert_eq!(lm.pages, pages);
        assert_eq!(lm.file.models[0].entity, "北😀");
        assert_eq!(
            (lm.file.models[0].asset.as_str(), lm.file.models[0].hash),
            ("base/mesh", u64::MAX)
        );
        assert_eq!(lm.file.models[0].parts[0].xref, [4, 1, 3]);
        assert_eq!(lm.file.models[0].parts[0].indices, [2, 0, 1]);
        assert_eq!(
            lm.file.models[0].parts[0].uv,
            [[0.125, 0.25, 1.0], [0.5, 0.75, 0.0], [1.0, 0.0, 1.0]]
        );
        assert_eq!(lm.file.models[0].parts[1].xref, [1]);
        assert_eq!(lm.file.models[1].asset, "base/room#hlod-7");
        assert_eq!(lm.file.models[2].asset, "base/room#hlod-2");
        assert_eq!(lm.file.volumes[0].probes[1][2], [21.0, 22.0, 23.0, 24.0]);
        assert_eq!(lm.file.volumes[1].probes[0][0], [25.0, 26.0, 27.0, 28.0]);
        if let Some(old) = previous {
            assert_eq!(
                old == bytes.to_vec(),
                step == 1,
                "page-only changes must invalidate a warm cook"
            );
        }
        previous = Some(bytes.to_vec());
    }
    write(
        "modules/north/lib/irradiance.ts",
        "export const ambient=0.875;",
    );
    assert!(cook_project(&root, &options).unwrap().ok());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        b.load_lightmap("north/lighting").unwrap().file.volumes[0].probes[0][0][3],
        0.875
    );
    for (from, to, expected) in [
        ("xref:[4,1,3]", "xref:[5,1,3]", "invalid vertex remapping"),
        ("0.125,0.25,1", "0.125,0.25,2", "invalid lightmap UV"),
        (
            "uv:[0.25,0.5,0]",
            "uv:[0.25,0.5]",
            "complete u/v/page triples",
        ),
        ("33,34,35,36]", "33,34,35]", "twelve RGB coefficients"),
        ("base/mesh", "base/missing", "base/missing"),
        ("base/mesh", "base/room", "model"),
        ("assets/pages.ktx2", "../pages.ktx2", "canonical"),
    ] {
        write(
            "modules/north/content/lighting.ts",
            &declaration.replace(from, to),
        );
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        let line = if from == "base/mesh" { 4 } else { 3 };
        assert!(
            error.contains(&format!("content/lighting.ts:{line}")),
            "{error}"
        );
    }
    write("modules/north/content/lighting.ts", declaration);
    write("modules/north/content/duplicate.ts", declaration);
    let error = cook_project(&root, &options).err().unwrap().to_string();
    assert!(error.contains("more than one lightmap"), "{error}");
    std::fs::remove_file(root.join("modules/north/content/duplicate.ts")).unwrap();
    write("assets/old.ogt", "type lightmap\nversion 1\n---\n");
    assert!(
        !cook_project(&root, &options).unwrap().ok(),
        "old lightmap author entry accepted"
    );
}

/// Font-pack TS refs, cmaps and conditional dictionary bytes reach the native
/// reader without changing another font, including warm-cache invalidation.
#[test]
fn typed_font_packs_preserve_membership_and_dictionary_dependencies() {
    use og_i18n::BundleFontPacks;
    let root = crate::temp::TempDir::new("typed-font-pack");
    crate::init_project(&root);
    std::fs::create_dir_all(root.join("assets/fonts")).unwrap();
    std::fs::create_dir_all(root.join("content/fonts")).unwrap();
    for name in ["first", "second", "unrelated"] {
        std::fs::copy(
            template_assets().join("fonts/NotoSansSC-Regular-subset.ttf"),
            root.join(format!("assets/fonts/{name}.ttf")),
        )
        .unwrap();
        crate::declare_import(
            &root,
            &format!("fonts/{name}"),
            "font",
            &format!("assets/fonts/{name}.ttf"),
        );
    }
    let mut options = CookOptions::default();
    assert!(cook_project(&root, &options).unwrap().ok());
    let unrelated = Bundle::open(&root.join("cooked"))
        .unwrap()
        .read("base/fonts/unrelated")
        .unwrap()
        .to_vec();
    std::fs::write(
        root.join("assets/fonts/en.hyph"),
        b"English dictionary snapshot",
    )
    .unwrap();
    std::fs::write(
        root.join("assets/fonts/fr.hyph"),
        b"French dictionary snapshot",
    )
    .unwrap();
    let declaration = "import {file,ref} from '@origingame/engine/author';\nexport default file['font-extension']({kind:'font',name:'Body',languages:['en_us'],scripts:['Latn'],faces:[ref('base/fonts/second'),ref('base/fonts/first')],hyphenation:['assets/fonts/en.hyph','assets/fonts/fr.hyph']});";
    std::fs::write(root.join("content/fonts/body.ts"), declaration).unwrap();
    options.code.hyphens = true;
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/fonts/body").unwrap();
        let pack = bundle.font_pack("base/fonts/body", &bytes).unwrap();
        assert_eq!(pack.name, "Body");
        assert_eq!(pack.languages, ["en-US"]);
        assert_eq!(pack.faces, ["base/fonts/second", "base/fonts/first"]);
        assert!(pack.covers('A') && !pack.covers('😀'));
        let blob = bundle
            .blob::<og_cook::format::FontPack>("base/fonts/body", &bytes)
            .unwrap();
        assert_eq!(
            blob.raw("hyphenation:en").unwrap(),
            b"English dictionary snapshot"
        );
        assert!(blob.raw("hyphenation:fr").is_err());
        assert_eq!(
            bundle.read("base/fonts/unrelated").unwrap().as_ref(),
            unrelated
        );
        if let Some(previous) = &previous {
            assert_eq!(previous, &bytes.to_vec());
        }
        previous = Some(bytes.to_vec());
    }
    std::fs::write(
        root.join("assets/fonts/en.hyph"),
        b"Changed English dictionary",
    )
    .unwrap();
    assert!(cook_project(&root, &options).unwrap().ok());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("base/fonts/body").unwrap();
    assert_ne!(bytes.to_vec(), previous.unwrap());
    assert_eq!(
        bundle
            .blob::<og_cook::format::FontPack>("base/fonts/body", &bytes)
            .unwrap()
            .raw("hyphenation:en")
            .unwrap(),
        b"Changed English dictionary"
    );
    options.code.hyphens = false;
    assert!(cook_project(&root, &options).unwrap().ok());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("base/fonts/body").unwrap();
    assert!(
        bundle
            .blob::<og_cook::format::FontPack>("base/fonts/body", &bytes)
            .unwrap()
            .raw("hyphenation:en")
            .is_err()
    );
    std::fs::write(
        root.join("content/fonts/body.ts"),
        declaration.replace("base/fonts/second", "base/missing"),
    )
    .unwrap();
    let error = cook_project(&root, &options).err().unwrap().to_string();
    assert!(
        error.contains("base/missing") && error.contains("content/fonts/body.ts:2"),
        "{error}"
    );
}

/// Typed locale input reaches the native reader with ICU, metadata and ordered
/// physical font IDs intact, including warm-helper invalidation and refusals.
#[test]
fn typed_locales_preserve_messages_and_resolve_font_chains() {
    use og_i18n::{Arg, BundleStrings, Direction};
    let root = crate::temp::TempDir::new("typed-locales");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    std::fs::create_dir_all(root.join("assets/fonts")).unwrap();
    for name in ["first", "second"] {
        std::fs::copy(
            template_assets().join("fonts/NotoSansSC-Regular-subset.ttf"),
            root.join(format!("assets/fonts/{name}.ttf")),
        )
        .unwrap();
        crate::declare_import(
            &root,
            &format!("fonts/{name}"),
            "font",
            &format!("assets/fonts/{name}.ttf"),
        );
    }
    write(
        "lib/messages.ts",
        "export const greeting = '北の{name} — مرحبًا';",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nimport {greeting} from '../../lib/messages.ts';\nexport default file.strings({locale:'ar_eg',fallback:'en_us',base:true,direction:'rtl',fonts:[ref('base/fonts/second'),ref('base/fonts/first')],entries:{hello:greeting,coins:'{n, plural, =0 {None} other {# coins}}'}});";
    write("content/strings/labels.ts", declaration);
    let options = CookOptions::default();
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/strings/labels").unwrap();
        let table = bundle.strings("base/strings/labels", &bytes).unwrap();
        assert_eq!(
            (
                table.locale.as_str(),
                table.fallback.as_deref(),
                table.base,
                table.direction
            ),
            ("ar-EG", Some("en-US"), true, Direction::Rtl)
        );
        assert_eq!(table.fonts, ["base/fonts/second", "base/fonts/first"]);
        assert_eq!(table.entries["hello"].0, "北の{name} — مرحبًا");
        assert_eq!(
            table.entries["coins"]
                .1
                .format("en", &BTreeMap::from([("n".into(), Arg::Num(3.0))])),
            "3 coins"
        );
        if let Some(previous) = &previous {
            assert_eq!(previous, &bytes.to_vec());
        }
        previous = Some(bytes.to_vec());
    }
    write("lib/messages.ts", "export const greeting = 'New {name}';");
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("base/strings/labels").unwrap();
    assert_ne!(previous.unwrap(), bytes.to_vec());
    assert_eq!(
        bundle
            .strings("base/strings/labels", &bytes)
            .unwrap()
            .entries["hello"]
            .0,
        "New {name}"
    );
    for (from, to, expected) in [
        ("base/fonts/second", "base/missing", "base/missing"),
        ("base/fonts/second", "base/strings/labels", "font"),
        ("ar_eg", "bad tag", "locale"),
        ("# coins", "{bad", "coins"),
    ] {
        write("content/strings/labels.ts", &declaration.replace(from, to));
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(r) => r.errors().join("\n"),
        };
        assert!(
            error.contains(expected) && error.contains("content/strings/labels.ts:3"),
            "{error}"
        );
    }
    write("content/strings/labels.ts", declaration);
    write(
        "assets/old.ogt",
        "type strings\nversion 1\n---\nlocale \"en\"\nstring \"hello\" \"Old\"\n",
    );
    assert!(!cook_project(&root, &options).unwrap().ok());
}

/// Checked maps survive native loading and feed the actual retarget/skin
/// pipeline, including imported helpers and invalidation of its warm outputs.
#[test]
fn typed_rigs_load_native_maps_and_refresh_model_consumers() {
    use og_anim::cooked::BundleAnim;
    let root = crate::temp::TempDir::new("typed-rig");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'rigs',entry:'src/main.ts',dependsOn:['cast'],modules:[{id:'cast',root:'modules/cast'}]});",
    );
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/motion/assets");
    std::fs::create_dir_all(root.join("assets/models")).unwrap();
    std::fs::create_dir_all(root.join("assets/anims")).unwrap();
    for path in ["models/strider.glb", "anims/humanoid-clips.glb"] {
        std::fs::copy(fixture.join(path), root.join("assets").join(path)).unwrap();
    }
    crate::declare_import(
        &root,
        "anims/humanoid-clips",
        "model",
        "assets/anims/humanoid-clips.glb",
    );
    write(
        "content/models/strider.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.import({kind:'model',source:'assets/models/strider.glb',relaxSkin:true,animation:{rig:ref('cast/rigs/hero'),libraries:[ref('base/anims/humanoid-clips')]}});",
    );
    write(
        "modules/cast/lib/feet.ts",
        "export const heights = [0.0625,0.1875];",
    );
    let declaration = "import {file} from '@origingame/engine/author';import {heights} from '../../lib/feet.ts';\nexport default file.rig({scheme:'mixamo',footHeight:heights,bones:{hips:'mixamorig:Hips',leftToes:null}});";
    write("modules/cast/content/rigs/hero.ts", declaration);
    write(
        "content/auto.ts",
        "import {file} from '@origingame/engine/author';export default file.rig({});",
    );
    write(
        "content/unicode.ts",
        "import {file} from '@origingame/engine/author';export default file.rig({bones:{hips:'骨盤'}});",
    );
    let options = CookOptions::default();
    let mut previous = None;
    let mut original_clips = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("cast/rigs/hero").unwrap();
        let rig = bundle.rig("cast/rigs/hero", &bytes).unwrap();
        assert_eq!(rig.scheme.as_deref(), Some("mixamo"));
        assert_eq!(rig.foot_height, Some([0.0625, 0.1875]));
        assert_eq!(rig.bones.len(), 2);
        let animation = bundle.load_animation("base/models/strider").unwrap();
        let map = rig.map(&animation.skeleton).unwrap();
        assert_eq!(
            map.nodes[og_anim::human_bone("hips").unwrap()],
            animation.skeleton.node("mixamorig:Hips").unwrap()
        );
        assert_eq!(
            map.nodes[og_anim::human_bone("leftToes").unwrap()],
            og_anim::NONE
        );
        assert!(!animation.clips.is_empty(), "library was not retargeted");
        original_clips = Some(animation.clips);
        let auto = bundle
            .rig("base/auto", &bundle.read("base/auto").unwrap())
            .unwrap();
        assert!(auto.scheme.is_none() && auto.foot_height.is_none() && auto.bones.is_empty());
        let unicode = bundle
            .rig("base/unicode", &bundle.read("base/unicode").unwrap())
            .unwrap();
        assert_eq!(unicode.bones[0].1.as_deref(), Some("骨盤"));
        let outputs = [
            bytes.to_vec(),
            bundle.read("base/models/strider").unwrap().to_vec(),
            bundle.read("base/models/strider#anim").unwrap().to_vec(),
        ];
        if let Some(previous) = &previous {
            assert_eq!(previous, &outputs);
        }
        previous = Some(outputs);
    }
    write(
        "modules/cast/lib/feet.ts",
        "export const heights = [0.3125,0.4375];",
    );
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let changed = bundle
        .rig("cast/rigs/hero", &bundle.read("cast/rigs/hero").unwrap())
        .unwrap();
    assert_eq!(changed.foot_height, Some([0.3125, 0.4375]));
    assert_ne!(
        original_clips.unwrap(),
        bundle.load_animation("base/models/strider").unwrap().clips,
        "retargeting reused the old map"
    );
    assert_ne!(
        previous.unwrap()[1],
        bundle.read("base/models/strider").unwrap().to_vec(),
        "skin preparation did not include its rig dependency"
    );
    for (from, to, expected) in [
        ("'mixamo'", "'unknown'", "scheme"),
        ("leftToes:null", "tail:null", "tail"),
        ("footHeight:heights", "footHeight:[0,10.125]", "footHeight"),
        ("mixamorig:Hips", "missing-骨盤", "missing-骨盤"),
    ] {
        write(
            "modules/cast/content/rigs/hero.ts",
            &declaration.replace(from, to),
        );
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(
            error.contains(expected) && error.contains("content/rigs/hero.ts:2"),
            "{error}"
        );
    }
    write("modules/cast/content/rigs/hero.ts", declaration);
    write("assets/old.ogt", "type rig\nversion 1\n---\nscheme auto\n");
    assert!(
        !cook_project(&root, &options).unwrap().ok(),
        "old rig author entry was accepted"
    );
}

/// Public TS cook/load contract: canonical refs, native fields and rejection
/// stay the same on a warm cook, including an imported source's physical ID.
#[test]
fn typed_motion_actions_resolve_refs_and_load_native_fields() {
    let root = crate::temp::TempDir::new("typed-reaction");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "assets/brace.gltf",
        r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"Root"}]}"#,
    );
    crate::declare_import(&root, "brace", "model", "assets/brace.gltf");
    write(
        "content/human.ts",
        "import {file} from '@origingame/engine/author';export default file.rig({scheme:'auto'});",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nexport default file['motion-action']({source:ref('base/brace'),clip:'brace-右',rig:ref('base/human'),holdStart:0.375,holdEnd:1.625,hand:'right',handTarget:[0.125,-0.25,0.75]});";
    write("content/actions/brace.ts", declaration);
    write(
        "content/actions/plain.ts",
        "import {file,ref} from '@origingame/engine/author';export default file['motion-action']({source:ref('base/brace'),clip:'idle',holdEnd:0.75});",
    );
    let options = CookOptions::default();
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/actions/brace").unwrap();
        let action = bundle.motion_action("base/actions/brace", &bytes).unwrap();
        assert_eq!(
            (
                action.source.as_str(),
                action.rig.as_str(),
                action.clip.as_str()
            ),
            ("base/brace", "base/human", "brace-右")
        );
        assert_eq!((action.hold_start, action.hold_end), (0.375, 1.625));
        assert_eq!(action.hand, "right");
        assert_eq!(action.hand_target, [0.125, -0.25, 0.75]);
        let plain = bundle.read("base/actions/plain").unwrap();
        let plain = bundle.motion_action("base/actions/plain", &plain).unwrap();
        assert_eq!((plain.hold_start, plain.hold_end), (0.0, 0.75));
        assert!(plain.rig.is_empty() && plain.hand.is_empty());
        assert_eq!(plain.hand_target, [0.0; 3]);
        if let Some(previous) = &previous {
            assert_eq!(previous, &bytes.to_vec());
        }
        previous = Some(bytes.to_vec());
    }
    for (from, to, expected) in [
        ("holdEnd:1.625", "holdEnd:0.375", "holdStart < holdEnd"),
        ("base/brace", "base/missing", "base/missing"),
        ("base/brace", "base/human", "model"),
    ] {
        write("content/actions/brace.ts", &declaration.replace(from, to));
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/actions/brace.ts:2"), "{error}");
    }
    write("content/actions/brace.ts", declaration);
    write(
        "assets/old.ogt",
        "type motion-action\nversion 1\n---\nsource \"brace.gltf\"\nclip \"brace\"\nholdEnd 1\n",
    );
    let report = cook_project(&root, &options).unwrap();
    assert!(!report.ok(), "old motion-action author entry was accepted");
}

/// A public navigation bake roundtrip, including imported buffer changes,
/// module refs, profile selection, ordered areas and full-width link IDs.
#[test]
fn typed_navigation_bakes_captured_geometry_and_preserves_profiles_and_links() {
    use og_ai::nav::cooked::BundleNav;
    use og_core::glam::Vec3;
    let root = crate::temp::TempDir::new("typed-nav");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'nav',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("assets/platform.gltf", &serde_json::json!({
        "asset":{"version":"2.0"}, "scene":0, "scenes":[{"nodes":[0]}],
        "nodes":[{"mesh":0}], "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
        "buffers":[{"uri":"platform.bin","byteLength":72}],
        "bufferViews":[{"buffer":0,"byteLength":72}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":6,"type":"VEC3","min":[-5,0,-3],"max":[5,3,3]}]
    }).to_string());
    crate::declare_import(&root, "platform", "model", "assets/platform.gltf");
    write(
        "content/scene.ts",
        "import {file} from '@origingame/engine/author';export default file.scene({entities:[{name:'east',components:{'og.Transform':{position:[20,1,0]},'og.Collider':{size:[6,2,6]}}}]});",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nexport default file.navmesh({
        scene:ref('base/scene'),models:[{model:ref('base/platform'),transform:{position:[2,0,0],scale:[2,1,1]}}],
        boxes:[{center:[-20,0,0],size:[6,2,6]}],
        agents:[{name:'large',radius:0.75,height:2.5},{radius:0,height:1}],
        cellSize:0.5,cellHeight:0.25,tileSize:16,
        bounds:{min:[-26,-2,-8],max:[26,12,8]},
        areas:[{id:3,shape:{kind:'box',center:[0,0,0],size:[10,10,6],yaw:0.2}},
               {id:5,shape:{kind:'cylinder',center:[0,0,0],radius:2,height:8}},
               {id:7,shape:{kind:'convex',points:[[-1,-1],[1,-1],[1,1],[-1,1]],bottom:0,top:8}}],
        links:[{id:16777217,start:[-2,1,0],end:[2,1,0],kind:'door',bidirectional:false,cost:2.75,duration:0.625,open:false,traverse:'game'}]
    });";
    write("modules/north/content/navigation.ts", declaration);
    let options = CookOptions {
        threads: 2,
        ..Default::default()
    };
    let mut previous = None;
    for height in [1.0f32, 3.0] {
        let positions = [
            [-5., height, -3.],
            [-5., height, 3.],
            [5., height, 3.],
            [-5., height, -3.],
            [5., height, 3.],
            [5., height, -3.],
        ];
        std::fs::write(
            root.join("assets/platform.bin"),
            positions
                .into_iter()
                .flatten()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        for warm in [false, true] {
            let report = cook_project(&root, &options).unwrap();
            assert!(report.ok(), "{:?}", report.errors());
            let b = Bundle::open(&root.join("cooked")).unwrap();
            let bytes = b.read("north/navigation").unwrap();
            assert_eq!(
                b.navmesh_profiles("north/navigation", &bytes).unwrap(),
                ["", "large"]
            );
            let (field, tiles) = b.navmesh("north/navigation", &bytes).unwrap();
            let sample = |x: f32| {
                let column = ((x - field.origin.x) / field.params.cell_size) as i32;
                let row = ((0.0 - field.origin.z) / field.params.cell_size) as i32;
                let spans: Vec<_> = field
                    .column(column, row)
                    .map(|i| {
                        field.origin.y + field.spans[i].floor as f32 * field.params.cell_height
                    })
                    .collect();
                assert_eq!(spans.len(), 1, "x={x}: {spans:?}");
                spans[0]
            };
            assert!((sample(0.0) - height).abs() <= 0.25);
            assert!((sample(-7.0) - height).abs() <= 0.25, "model scale missing");
            assert!(
                (sample(11.0) - height).abs() <= 0.25,
                "model position missing"
            );
            assert!((sample(20.0) - 2.0).abs() <= 0.25, "scene geometry missing");
            assert!((sample(-20.0) - 1.0).abs() <= 0.25, "explicit box missing");
            let areas = b.navmesh_areas("north/navigation", &bytes).unwrap();
            assert_eq!(
                areas.iter().map(|(_, id)| *id).collect::<Vec<_>>(),
                [3, 5, 7]
            );
            let mesh = og_ai::nav::NavMesh::from_tiles_with_areas(field, tiles, areas).unwrap();
            let (poly, _) = mesh
                .nearest(Vec3::new(0.0, height, 0.0), Vec3::splat(0.5))
                .expect("walkable model surface");
            assert_eq!(
                mesh.poly(poly).unwrap().1.area,
                7,
                "last overlapping area wins"
            );
            let links = b.navmesh_links("north/navigation", &bytes).unwrap();
            let (id, link) = &links[0];
            assert_eq!(
                (*id, link.bidirectional, link.game, link.open),
                (16777217, false, true, false)
            );
            assert_eq!((link.cost, link.duration), (2.75, 0.625));
            assert!(matches!(link.kind, og_ai::nav::LinkKind::Door));
            let profile = b
                .navmesh_profile("north/navigation", &bytes, "large")
                .unwrap()
                .unwrap();
            let (large, _) = b.navmesh("north/navigation", profile).unwrap();
            assert_eq!(
                (large.params.agent_radius, large.params.agent_height),
                (0.75, 2.5)
            );
            if warm {
                assert_eq!(previous.as_ref().unwrap(), &bytes.to_vec());
            } else if let Some(previous) = &previous {
                assert_ne!(previous, &bytes.to_vec(), "stale external buffer");
            }
            previous = Some(bytes.to_vec());
        }
    }
    for (suffix, expected) in [
        ("agents:[{},{}]", "agent profile '' is used twice"),
        ("boxes:[{quaternion:[0,0,0,0]}]", "zero length"),
        (
            "links:[{id:1,start:[0,0,0],end:[1,0,0]},{id:1,start:[0,0,0],end:[2,0,0]}]",
            "link id 1 is used twice",
        ),
    ] {
        write(
            "modules/north/content/navigation.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author';\nexport default file.navmesh({{{suffix}}});"
            ),
        );
        let result = cook_project(&root, &options);
        let error = match result {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/navigation.ts:2"), "{error}");
    }
    write("modules/north/content/navigation.ts", declaration);
    write("assets/old.ogt", "type navmesh\nversion 1\n---\n");
    let report = cook_project(&root, &options).unwrap();
    assert!(!report.ok(), "old navigation author entry was accepted");
}

/// `tests/fixtures/cook-assets/assets`: what the template ships.
fn template_assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cook-assets/assets")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).unwrap();
        }
    }
}

fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                walk(base, &e.path(), out);
            } else {
                let rel = e
                    .path()
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(e.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// What differs between two `cooked/` trees, one line per difference: a
/// file on one side only, a blob's bytes (its size and first differing
/// offset), each differing line of the manifest, and the cook times with
/// the times masked (the same assets, whatever their milliseconds; README
/// §Cook times: the one file two cooks write differently). Empty: the
/// same bytes.
fn differences(a: &BTreeMap<String, Vec<u8>>, b: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut out = Vec::new();
    for k in a.keys().filter(|k| !b.contains_key(*k)) {
        out.push(format!("cooked/{k}: only in the first cook"));
    }
    for k in b.keys().filter(|k| !a.contains_key(*k)) {
        out.push(format!("cooked/{k}: only in the second cook"));
    }
    for (k, x) in a {
        let Some(y) = b.get(k) else { continue };
        if k == og_cook::COOK_TIMES {
            let masked = |t: &[u8]| {
                og_cook::CookTimes::parse(&String::from_utf8_lossy(t))
                    .map(|c| c.assets.into_keys().collect::<Vec<_>>())
            };
            let (mx, my) = (masked(x), masked(y));
            if mx != my {
                out.push(format!(
                    "cooked/{k}: the assets differ (times masked): {mx:?} against {my:?}"
                ));
            }
        } else if x != y {
            let at = x.iter().zip(y).position(|(p, q)| p != q);
            out.push(format!(
                "cooked/{k}: {} against {} bytes, first difference at {}",
                x.len(),
                y.len(),
                at.map_or("the end of the shorter".to_string(), |i| format!(
                    "byte {i}"
                ))
            ));
            if k == og_cook::runtime::MANIFEST {
                let (tx, ty) = (String::from_utf8_lossy(x), String::from_utf8_lossy(y));
                let (lx, ly): (Vec<&str>, Vec<&str>) = (tx.lines().collect(), ty.lines().collect());
                for i in 0..lx.len().max(ly.len()) {
                    let (p, q) = (lx.get(i), ly.get(i));
                    if p != q {
                        out.push(format!("  line {}: {p:?} against {q:?}", i + 1));
                    }
                }
            }
        }
    }
    out
}

/// The template made small enough to cook thirty times: as the template
/// ships (without the fixture's trickster.glb, whose three 1024² textures
/// are most of a cook; its scene entity is the walker), the two standalone
/// textures cut to 128². Every kind of output and both texture encoders
/// remain.
fn small_template(root: &Path) {
    copy_dir(&template_assets(), &root.join("assets"));
    copy_dir(
        &template_assets().with_file_name("content"),
        &root.join("content"),
    );
    let assets = root.join("assets");
    for t in ["textures/stone.png", "textures/stone_n.png"] {
        let img = image::open(assets.join(t)).unwrap();
        img.resize_exact(128, 128, image::imageops::FilterType::Triangle)
            .save(assets.join(t))
            .unwrap();
    }
}

/// Cooks of one project on one, two and eight threads, the three series
/// at once (so every cook shares the machine with two others and its
/// pool's schedule moves from run to run): every file each writes is the
/// first one-thread cook's bytes, the cook times masked (`differences`).
/// One run a thread count in the quick and functional lanes; ten under
/// `--all` (`OG_GATE_ALL`), whose repeats hunt a race that
/// one schedule may miss (the owner, 2026-11-06: 31 cooks were the gate's
/// slowest test).
#[test]
fn every_thread_count_cooks_the_same_bytes() {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/threads");
    let _ = std::fs::remove_dir_all(&base);
    let source = base.join("source");
    small_template(&source);
    let cook = |dir: &Path, threads: usize| {
        let _ = std::fs::remove_dir_all(dir);
        crate::init_project(dir);
        copy_dir(&source.join("assets"), &dir.join("assets"));
        copy_dir(&source.join("content"), &dir.join("content"));
        let options = CookOptions {
            threads,
            ..Default::default()
        };
        let report = cook_project(dir, &options).unwrap();
        assert!(report.ok(), "{threads} threads: {:?}", report.errors());
        files(&dir.join("cooked"))
    };
    let reference = cook(&base.join("reference"), 1);
    assert!(!reference.is_empty());
    let runs = if std::env::var_os("OG_GATE_ALL").is_some() {
        10
    } else {
        1
    };
    let failures: Vec<String> = std::thread::scope(|s| {
        let series: Vec<_> = [1usize, 2, 8]
            .into_iter()
            .map(|threads| {
                let (base, reference, cook, runs) = (&base, &reference, &cook, runs);
                s.spawn(move || {
                    let mut out = Vec::new();
                    for run in 0..runs {
                        let dir = base.join(format!("t{threads}-{run}"));
                        let got = cook(&dir, threads);
                        for d in differences(reference, &got) {
                            out.push(format!("{threads} threads, run {run}: {d}"));
                        }
                        let _ = std::fs::remove_dir_all(&dir);
                    }
                    out
                })
            })
            .collect();
        series.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    assert!(
        failures.is_empty(),
        "cooks differ from the first one-thread cook:\n{}",
        failures.join("\n")
    );
}

#[test]
fn sounds_fonts_and_scenes_load_as_authored() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/template");
    let _ = std::fs::remove_dir_all(&root);
    crate::init_project(&root);
    copy_dir(&template_assets(), &root.join("assets"));
    copy_dir(
        &template_assets().with_file_name("content"),
        &root.join("content"),
    );
    crate::declare_import(&root, "audio/recording", "sound", "assets/audio/blip.wav");
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).expect("the manifest loads");
    // Sound: QOA decodes to the WAV's length and stays close to it.
    let s = b.load_sound("base/audio/recording").unwrap();
    let pcm = s.decode().unwrap();
    let mut wav = hound::WavReader::open(template_assets().join("audio/blip.wav")).unwrap();
    let src: Vec<i16> = wav.samples::<i16>().map(Result::unwrap).collect();
    assert_eq!((pcm.channels, pcm.sample_rate), (1, 44100));
    assert_eq!(pcm.samples.len(), src.len());
    let (mut sig, mut err) = (0.0f64, 0.0f64);
    for (a, b) in src.iter().zip(&pcm.samples) {
        sig += (*a as f64).powi(2);
        err += (*a as f64 - *b as f64).powi(2);
    }
    let snr = 10.0 * (sig / err).log10();
    assert!(snr > 20.0, "QOA SNR {snr:.1} dB");

    // Font: the file, whole, with its metrics; no atlas.
    let f = b.load_font("base/fonts/NotoSansSC-Regular-subset").unwrap();
    let source =
        std::fs::read(template_assets().join("fonts/NotoSansSC-Regular-subset.ttf")).unwrap();
    assert_eq!(f.file.as_deref(), Some(source.as_slice()));
    assert!(f.pages.is_empty() && f.glyphs.is_empty() && f.kerning.is_empty());
    assert!(f.meta.ascender > 0.5 && f.meta.descender < 0.0);

    // Locale data and compiled ICU arguments survive the native blob boundary.
    use og_i18n::BundleStrings;
    let bytes = b.read("base/strings/en").unwrap();
    let strings = b.strings("base/strings/en", &bytes).unwrap();
    assert_eq!(strings.locale, "en");
    assert!(strings.base && strings.fallback.is_none());
    assert_eq!(strings.direction, og_i18n::Direction::Ltr);
    assert_eq!(strings.entries["menu.title"].0, "Origin Engine");
    assert_eq!(strings.entries["menu.spinning"].1.arguments(), ["speed"]);

    // Scene: entities, nesting, model instances and the Transform table.
    let scene = b.load_scene("base/scenes/main").unwrap();
    assert_eq!(scene.name, "Main");
    let names: Vec<&str> = scene.entities.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["ground", "dropship", "trickster", "marker"]);
    assert_eq!(scene.entities[3].parent, Some(2));
    assert_eq!(
        scene.entities[1].instance.as_deref(),
        Some("base/models/dropship")
    );
    let t = scene
        .components
        .iter()
        .find(|c| c.name == "og.Transform")
        .unwrap();
    assert!(!t.inferred);
    let fields: Vec<&str> = t.schema.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(fields, ["position", "quaternion", "scale"]);
    let marker = t.get(3).unwrap();
    let f32s: Vec<f32> = marker
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect();
    assert_eq!(f32s, [0.0, 2.2, 0.0, 0.0, 0.0, 0.0, 1.0, 0.25, 0.25, 0.25]);
}

/// A discontinuous weight at a seam stretches two coincident triangles;
/// a disconnected, healthy triangle must retain its authored weights.
/// Exercise import through cooked animation data, not the relaxation helper.
#[test]
fn skin_relax_is_explicit_local_and_deterministic() {
    use serde_json::json;
    let root = crate::temp::TempDir::new("skin-relax");
    crate::init_project(&root);
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    let positions: [[f32; 3]; 9] = [
        [0.0, 1.0, 0.0],
        [0.03, 1.0, 0.0],
        [0.0, 1.03, 0.0],
        [0.0, 1.0, 0.0],
        [0.03, 1.0, 0.0],
        [0.0, 1.03, 0.0],
        [3.0, 0.02, 0.0],
        [3.001, 0.02, 0.0],
        [3.0, 0.021, 0.0],
    ];
    let joints: [[u16; 4]; 9] = std::array::from_fn(|i| [u16::from(i % 3 == 0), 0, 0, 0]);
    let weights = [[1.0f32, 0.0, 0.0, 0.0]; 9];
    let mut bytes = Vec::new();
    let mut views = Vec::new();
    for data in [
        bytemuck::cast_slice(&positions),
        bytemuck::cast_slice(&joints),
        bytemuck::cast_slice(&weights),
    ] {
        views.push(json!({"buffer":0,"byteOffset":bytes.len(),"byteLength":data.len()}));
        bytes.extend_from_slice(data);
    }
    let mut source = json!({
        "asset":{"version":"2.0"}, "buffers":[{"uri":"mesh.bin","byteLength":bytes.len()}],
        "bufferViews":views,
        "accessors":[
            {"bufferView":0,"componentType":5126,"count":9,"type":"VEC3","min":[0,0.02,0],"max":[3.001,1.03,0]},
            {"bufferView":1,"componentType":5123,"count":9,"type":"VEC4"},
            {"bufferView":2,"componentType":5126,"count":9,"type":"VEC4"}
        ],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"JOINTS_0":1,"WEIGHTS_0":2}}]}],
        "nodes":[{"children":[1,4]}, {"children":[2]}, {"children":[3]}, {},
            {"children":[5]}, {"children":[6]}, {}, {"mesh":0,"skin":0}, {"mesh":0,"skin":1}],
        "skins":[{"joints":[0,1,2,3,4,5,6]}, {"joints":[0,1]}],
        "scenes":[{"nodes":[0,7,8]}],"scene":0
    });
    std::fs::write(assets.join("mesh.bin"), &bytes).unwrap();
    std::fs::write(assets.join("mesh.gltf"), source.to_string()).unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    let cook = |setting: &str, threads| {
        std::fs::write(
            root.join("content/mesh.ts"),
            format!("import {{file}} from '@origingame/engine/author';export default file.import({{kind:'model',source:'assets/mesh.gltf',lods:[1],{setting}}});"),
        )
        .unwrap();
        let report = cook_project(
            &root,
            &CookOptions {
                threads,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let output = report
            .assets
            .iter()
            .flat_map(|a| &a.outputs)
            .find(|o| o.id == "base/mesh#anim")
            .unwrap();
        let bytes = std::fs::read(root.join("cooked").join(&output.file)).unwrap();
        let animation =
            og_anim::cooked::CookedAnimation::from_bytes("base/mesh#anim", &bytes).unwrap();
        (report, animation.skin_streams, bytes)
    };
    let (off, authored, _) = cook("", 1);
    assert!(
        off.warnings
            .iter()
            .any(|w| w.contains("skin stretch") && w.contains("relaxSkin"))
    );
    let (_, explicit_off, _) = cook("relaxSkin:false", 1);
    assert_eq!(authored, explicit_off);
    let (on, repaired, first) = cook("relaxSkin:true", 1);
    assert!(!on.warnings.iter().any(|w| w.contains("skin stretch")));
    assert!(
        on.assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("6 of 9 vertices reweighted; 0 remaining"))
    );
    assert_ne!(authored, repaired);
    let stream = &repaired[0];
    // The second binding has no branch probes and only two joints. Even
    // zero-weight slots must remain valid for that binding of the shared mesh.
    assert!(stream.joints.iter().flatten().all(|&j| j < 2));
    assert_eq!(
        stream
            .weights
            .iter()
            .filter(|&&w| w == [1.0, 0.0, 0.0, 0.0])
            .count(),
        3
    );
    for ws in &stream.weights {
        assert!((ws.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }
    std::fs::remove_dir_all(root.join("cooked")).unwrap();
    let (again, _, second) = cook("relaxSkin:true", 8);
    assert_eq!(first, second, "fresh parallel cook must be byte-identical");
    assert!(!again.warnings.iter().any(|w| w.contains("skin stretch")));
    let (cached, _, _) = cook("relaxSkin:true", 8);
    assert!(
        cached
            .assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("6 of 9 vertices reweighted; 0 remaining"))
    );
    assert_eq!(std::fs::read(assets.join("mesh.bin")).unwrap(), bytes);

    // The same mesh in quantised-accessor units, with dequantisation in its
    // inverse binds as gltfpack writes it. The tiny disconnected triangle is
    // below 8 cm in model space even though its accessor-space edges are huge.
    let scaled = positions.map(|p| p.map(|v| v * 32768.0));
    bytes[..std::mem::size_of_val(&scaled)].copy_from_slice(bytemuck::cast_slice(&scaled));
    let s = 1.0f32 / 32768.0;
    let inverse = [
        s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    source["bufferViews"].as_array_mut().unwrap().push(json!({
        "buffer":0,"byteOffset":bytes.len(),"byteLength":7 * 64
    }));
    bytes.extend_from_slice(bytemuck::cast_slice(&[inverse; 7]));
    source["buffers"][0]["byteLength"] = json!(bytes.len());
    source["accessors"][0]["min"] = json!([0.0, 0.02 * 32768.0, 0.0]);
    source["accessors"][0]["max"] = json!([3.001 * 32768.0, 1.03 * 32768.0, 0.0]);
    for (skin, count) in [7, 2].into_iter().enumerate() {
        source["skins"][skin]["inverseBindMatrices"] = json!(3 + skin);
        source["accessors"].as_array_mut().unwrap().push(json!({
            "bufferView":3,"componentType":5126,"count":count,"type":"MAT4"
        }));
    }
    std::fs::write(assets.join("mesh.bin"), &bytes).unwrap();
    std::fs::write(assets.join("mesh.gltf"), source.to_string()).unwrap();
    let (_, scaled_authored, _) = cook("", 1);
    assert_eq!(authored, scaled_authored);
    let (scaled_on, scaled_repaired, _) = cook("relaxSkin:true", 1);
    assert!(
        !scaled_on
            .warnings
            .iter()
            .any(|w| w.contains("skin stretch"))
    );
    assert_eq!(
        repaired, scaled_repaired,
        "skin weights must not depend on accessor units"
    );
}
