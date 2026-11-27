use super::*;

/// Material libraries use one checked declaration for cook and the author tools,
/// including module-owned images, actual derived IDs and warm invalidation.
#[test]
fn typed_material_libraries_generate_maps_and_drive_author_tools() {
    let root = TempDir::new("og-ts-materials");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Materials","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'materials',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "");
    write(
        "modules/north/lib/tile.ts",
        "export const tile=[2,1] as const;",
    );
    let library = "import {file} from '@origingame/engine/author';import {tile} from '../lib/tile.ts';export default file.materials({materials:[{name:'stone',labels:['wall'],tile,brief:'stone',generate:{source:'source.svg'},relief:0.02,roughnessRange:[0.8,0.3],occlusion:{strength:0.5,radius:3},surface:{map:'stone.png',normalMap:'stone-n.png',metalnessMap:'stone-mr.png',aoMap:'stone-ao.png'}}]});";
    write("modules/north/content/library.ts", library);
    write(
        "modules/north/content/panel.ts",
        "import {file} from '@origingame/engine/author';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'stone',input:{kind:'material',desc:{map:'assets/stone.png'}}},{name:'panel',input:{kind:'part',desc:{geometry:{kind:'planeGeometry',desc:{width:2,height:1}},material:'stone'}}}]}});",
    );
    write(
        "modules/north/content/tiles.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.tileset({texture:ref('north/library#map-0'),tileWidth:4,tileHeight:4,columns:4,rows:2});",
    );
    // A same-named base image must not be read for the north module.
    let image = r##"<svg xmlns="http://www.w3.org/2000/svg" width="19" height="11"><rect width="19" height="11" fill="#c84020"/></svg>"##;
    write("assets/source.svg", &image.replace("#c84020", "#2040c8"));
    write("modules/north/assets/source.svg", image);
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let mut previous = None;
    for (source, channel) in [
        (image.to_string(), 0),
        (image.replace("#c84020", "#20c840"), 1),
    ] {
        write("modules/north/assets/source.svg", &source);
        let mut cold = None;
        for _ in 0..2 {
            let out = cook();
            assert!(out.status.success(), "{}", text(&out));
            let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
            let map = bundle
                .load_texture("north/library#map-0")
                .unwrap()
                .to_texture_data()
                .unwrap();
            assert_eq!((map.width, map.height), (16, 8));
            assert!(
                map.rgba[channel] > 180 && map.rgba[2] < 80,
                "wrong module image: {:?}",
                &map.rgba[..4]
            );
            if let Some(bytes) = &cold {
                assert_eq!(bytes, &map.rgba);
            }
            cold = Some(map.rgba.clone());
            for i in 1..4 {
                assert!(
                    bundle
                        .load_texture(&format!("north/library#map-{i}"))
                        .is_ok()
                );
            }
            let model = bundle.load_model("north/panel").unwrap();
            let index = model
                .materials
                .iter()
                .find_map(|m| m.base_color_texture)
                .unwrap();
            let embedded = bundle
                .load_texture(&model.textures[index].id)
                .unwrap()
                .to_texture_data()
                .unwrap();
            assert_eq!(
                embedded.rgba, map.rgba,
                "model did not capture the generated map"
            );
            assert!(!root.join("modules/north/assets/stone.png").exists());
        }
        if let Some(bytes) = previous {
            assert_ne!(
                bytes,
                cold.as_ref().unwrap().clone(),
                "source change reused stale maps"
            );
        }
        previous = cold;
    }
    write(
        "modules/north/lib/tile.ts",
        "export const tile=[4,1] as const;",
    );
    // Briefs consume the last successful checked look report. Cook it before
    // make writes author images that deliberately collide with derived maps.
    write(
        "og-project.json",
        r#"{"version":1,"name":"Materials","engine":"0.9.0","look":"pixel"}"#,
    );
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let run = || {
        og().args([
            "materials",
            "make",
            "modules/north/content/library.ts",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap()
    };
    let out = run();
    assert!(out.status.success(), "{}", text(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["made"][0]["density"], serde_json::json!([4.0, 8.0]));
    assert_eq!(report["made"][0]["written"].as_array().unwrap().len(), 4);
    assert!(root.join("modules/north/assets/stone.png").exists());
    assert!(!root.join("assets/stone.png").exists());
    let out = run();
    assert!(out.status.success(), "{}", text(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["made"][0]["cached"], true);
    let out = og()
        .args([
            "materials",
            "brief",
            "modules/north/content/library.ts",
            "--json",
        ])
        .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
        .env("OG_BIN", env!("CARGO_BIN_EXE_og"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["requests"][0]["source"], "source.svg");
    assert_eq!(report["requests"][0]["tile"], serde_json::json!([4.0, 1.0]));
    assert!(
        report["requests"][0]["brief"]["prompt"]
            .as_str()
            .unwrap()
            .contains("stone")
    );
    #[cfg(unix)]
    {
        let assets = root.join("modules/north/assets");
        let saved = root.join("modules/north/saved-assets");
        std::fs::rename(&assets, &saved).unwrap();
        std::os::unix::fs::symlink(root.join("assets"), &assets).unwrap();
        let out = run();
        assert_eq!(
            out.status.code(),
            Some(2),
            "asset-root symlink followed: {}",
            text(&out)
        );
        assert!(text(&out).contains("symlink"), "{}", text(&out));
        let out = cook();
        assert!(!out.status.success(), "cook followed an asset-root symlink");
        assert!(text(&out).contains("symlink"), "{}", text(&out));
        std::fs::remove_file(&assets).unwrap();
        std::fs::rename(saved, assets).unwrap();
    }
    let out = cook();
    assert!(
        !out.status.success(),
        "generated map overwrote an author output"
    );
    assert!(text(&out).contains("collides"), "{}", text(&out));
    for name in ["stone.png", "stone-n.png", "stone-mr.png", "stone-ao.png"] {
        std::fs::remove_file(root.join("modules/north/assets").join(name)).unwrap();
    }
    write(
        "modules/north/content/library.ts",
        &library.replace("tile,brief", "tile,unknown:1,brief"),
    );
    let out = run();
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out).contains("unknown"), "{}", text(&out));
    write("modules/north/content/library.ts", library);
    write("assets/old.ogt", "type materials\nversion 1\n---\n");
    let out = cook();
    assert!(
        !out.status.success(),
        "OGT material libraries were accepted"
    );
    let out = og()
        .args(["materials", "make", "assets/old.ogt", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

/// All three rig inputs share the native loader, canonical graph and cache.
#[test]
fn rig2d_cooks_native_fields_imports_and_captured_parts() {
    use og_anim::cooked::BundleAnim;
    use og_cook::runtime::Bundle;
    use serde_json::json;
    let root = TempDir::new("og-ts-rig2d");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Rig","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'rig',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "");
    write("lib/unit.ts", "export const unit=7;");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(13, 9, image::Rgba([83, 119, 211, 255]))
        .save(root.join("assets/atlas.png"))
        .unwrap();
    write(
        "content/atlas.ts",
        "import {file} from '@origingame/engine/author';export default file.import({kind:'texture',source:'assets/atlas.png'});",
    );
    let body = json!({
        "bones":[{"name":"root","at":[3,7]}, {"name":"tip","parent":"root","at":[5,2],"length":4}],
        "slots":[{"name":"back","bone":"tip","attachment":"z-region"},{"name":"front","bone":"root","attachment":"a-box"}],
        "attachments":{
            "z-region":{"kind":"region","slot":"back","rect":[1,2,3,5]},
            "a-box":{"kind":"box","slot":"front","polygon":[[0,0],[2,0],[0,3]]},
            "mesh":{"kind":"mesh","slot":"back","vertices":[
                {"uv":[1,2],"influences":[{"bone":"root","at":[0,0],"weight":0.25},{"bone":"tip","at":[2,3],"weight":0.75}]},
                {"uv":[4,2],"influences":[{"bone":"tip","at":[2,0],"weight":1}]},
                {"uv":[1,7],"influences":[{"bone":"tip","at":[0,3],"weight":1}]}],"triangles":[0,1,2]},
            "child":{"kind":"child","slot":"front","armature":"flame","clip":"burn"}
        },
        "iks":[{"name":"aim","bones":["tip"],"target":"root","weight":0.25}],
        "transforms":[{"name":"follow","bones":["tip"],"target":"root","mix":[0,0,0,0]}],
        "skins":{"armor":{"z-region":"mesh","a-box":null}},
        "clips":[{"name":"walk","duration":1,"looping":false,
            "bones":[{"bone":"tip","translate":[{"time":0,"value":[2,7],"ease":{"kind":"step"}}],
                "rotate":[{"time":0,"value":{"degrees":23,"turn":{"kind":"clockwise","turns":2}}}]}],
            "slots":[{"slot":"front","attachment":[{"time":0.25,"value":"child"}]}],
            "deforms":{"mesh":[{"time":0,"value":[0,1,2,3,4,5,6,7]}]},
            "order":[{"time":0.5,"value":["front","back"]}],
            "iks":{"aim":[{"time":0,"value":{"weight":0.75,"counterclockwise":true}}]},
            "events":[{"time":0.75,"value":"late"},{"time":0.125,"value":"早"}]}]
    });
    let child = json!({"bones":[{"name":"flame-root"}],"clips":[{"name":"burn","duration":0.5}]});
    let authored = format!(
        "import {{file,ref}} from '@origingame/engine/author';\nimport {{unit}} from '../lib/unit.ts';\nexport default file.rig2d({{texture:ref('base/atlas'),filter:'nearest',unit,rig:{body},armatures:{{flame:{child}}}}});"
    );
    write("content/hero.ts", &authored);
    write(
        "sources/skeleton.json",
        r#"{"version":"5.5","frameRate":24,"armature":[{"name":"hero","bone":[{"name":"hip","transform":{"x":5,"y":9}}]}]}"#,
    );
    write(
        "sources/atlas.json",
        r#"{"imagePath":"DO-NOT-GUESS.png","width":13,"height":9,"SubTexture":[]}"#,
    );
    write(
        "content/imported.ts",
        "import {file,input,ref} from '@origingame/engine/author';export default file.rig2d({dragonbones:input.text('sources/skeleton.json'),atlas:input.text('sources/atlas.json'),texture:ref('base/atlas'),unit:11});",
    );
    for module in ["", "modules/north/"] {
        std::fs::create_dir_all(root.join(format!("{module}sources/parts"))).unwrap();
        image::RgbaImage::from_pixel(9, 13, image::Rgba([127, 83, 211, 255]))
            .save(root.join(format!("{module}sources/parts/body.png")))
            .unwrap();
        write(
            &format!("{module}content/creature.ts"),
            "import {file} from '@origingame/engine/author';export default file.rig2d({parts:'sources/parts',template:'creature',pivots:{body:[3,5]},unit:13,filter:'nearest'});",
        );
    }
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let hero = bundle.load_rig2d("base/hero").unwrap();
    assert_eq!(
        (&hero.texture, hero.size, hero.unit),
        (&"base/atlas".to_string(), [13, 9], 7.0)
    );
    assert_eq!(hero.bones[1].parent, Some(0));
    assert_eq!((hero.bones[0].x, hero.bones[0].y), (3.0, 7.0));
    assert_eq!(
        hero.attachments[hero.slots[0].attachment.unwrap()].name,
        "z-region"
    );
    assert_eq!(
        hero.clips[0].events,
        [(0.125, "早".into()), (0.75, "late".into())]
    );
    assert_eq!(hero.clips[0].order[0].1, [1, 0]);
    assert_eq!(
        hero.clips[0].deforms[0].keys[0].value,
        [0., 1., 2., 3., 4., 5., 6., 7.]
    );
    assert_eq!(hero.skins[0].name, "armor");
    assert_eq!(hero.armatures[0].clips[0].name, "burn");
    assert_eq!(hero.clips[0].iks[0].keys[0].value.weight, 0.75);
    assert!(matches!(
        hero.clips[0].bones[0].rotate[0].value.turn,
        og_anim::Turn::Clockwise(2)
    ));
    let imported = bundle.load_rig2d("base/imported").unwrap();
    assert_eq!(
        (imported.bones[0].x, imported.bones[0].y, imported.unit),
        (5.0, -9.0, 11.0)
    );
    let creature = bundle.load_rig2d("base/creature").unwrap();
    assert_eq!(creature.texture, "base/creature#atlas");
    assert!(!creature.clips.is_empty());
    assert_eq!(
        bundle.load_texture(&creature.texture).unwrap().meta.width,
        creature.size[0]
    );
    let north = bundle.read("north/creature").unwrap().to_vec();
    let original = bundle.read("base/hero").unwrap().to_vec();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::remove_dir_all(root.join("cooked")).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .read("base/hero")
            .unwrap()
            .to_vec(),
        original
    );
    write("lib/unit.ts", "export const unit=17;");
    write(
        "sources/skeleton.json",
        r#"{"version":"5.5","frameRate":24,"armature":[{"name":"hero","bone":[{"name":"hip","transform":{"x":5,"y":12}}]}]}"#,
    );
    image::RgbaImage::from_pixel(11, 13, image::Rgba([127, 83, 211, 255]))
        .save(root.join("sources/parts/body.png"))
        .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let changed = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(changed.load_rig2d("base/hero").unwrap().unit, 17.0);
    assert_eq!(
        changed.load_rig2d("base/imported").unwrap().bones[0].y,
        -12.0
    );
    assert_eq!(changed.read("north/creature").unwrap().to_vec(), north);
    assert_ne!(changed.load_rig2d("base/creature").unwrap(), creature);
    for invalid in [
        authored.replace("\"parent\":\"root\"", "\"parent\":\"tip\""),
        authored.replace("base/atlas", "base/hero"),
        authored.replace("base/atlas", "base/missing"),
        authored.replace("\"weight\":0.25", "\"weight\":0.5"),
        authored.replace("\"value\":[0,1,2,3,4,5,6,7]", "\"value\":[0,1]"),
        authored.replace("\"slot\":\"back\"", "\"slot\":\"missing\""),
    ] {
        assert_ne!(invalid, authored);
        write("content/hero.ts", &invalid);
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}");
        assert!(text(&out).contains("content/hero.ts:"), "{}", text(&out));
    }
    write("content/hero.ts", &authored);
    write(
        "assets/old.ogt",
        "type rig2d\nversion 1\n---\ntexture \"atlas.png\"\n",
    );
    let out = cook();
    assert!(!out.status.success(), "{}", text(&out));
}

/// The bake command produces cookable TS plus captured dense buffers, and
/// rebakes the existing declaration even when a module owns it elsewhere.
#[test]
fn lightmap_bakes_roundtrip_typed_sources_and_module_ownership() {
    use og_cook::runtime::Bundle;
    let root = TempDir::new("og-lightmap-bake");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Lighting","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'lighting',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "");
    write("modules/north/lib/empty.ts", "");
    write(
        "content/box.ts",
        "import {file} from '@origingame/engine/author'; export default file.model({kind:'geometry',desc:{items:[{name:'box',input:{kind:'part',desc:{geometry:{kind:'boxGeometry',desc:{width:1,height:1,depth:1}}}}}]}});",
    );
    write(
        "content/room.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.scene({hlod:{size:16,error:0.25,texels:64},entities:[{name:'north',model:ref('base/box'),static:true,components:{'og.Transform':{position:[1,0,1]}}},{name:'south',model:ref('base/box'),static:true,components:{'og.Transform':{position:[3,0,1]}}},{name:'probe',components:{'og.Volume':{size:[2,1,1]},'og.LightVolume':{spacing:1}}}]});",
    );
    let run = |args: &[&str]| {
        og().args(args)
            .current_dir(&root)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    let bake = || {
        let out = run(&[
            "bake",
            "lightmaps",
            ".",
            "--samples",
            "1",
            "--bounces",
            "1",
            "--texels-per-metre",
            "1",
            "--max-size",
            "32",
            "--threads",
            "2",
        ]);
        assert!(out.status.success(), "{}", text(&out));
    };
    bake();
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let lighting = bundle.load_lightmap("base/lightmaps/room").unwrap();
    assert_eq!(
        lighting
            .file
            .models
            .iter()
            .filter(|m| !m.entity.is_empty())
            .count(),
        2
    );
    assert!(lighting.file.models.iter().any(|m| m.entity.is_empty()));
    assert_eq!(lighting.file.volumes.len(), 1);
    let expected_uv = lighting.file.models[0].parts[0].uv.clone();
    let source = std::fs::read_to_string(root.join("content/lightmaps/room.ts")).unwrap();
    let out = run(&["check", "."]);
    assert!(out.status.success(), "{}", text(&out));
    write("modules/north/content/custom/lighting.ts", &source);
    std::fs::create_dir_all(root.join("modules/north/assets/lightmaps")).unwrap();
    for suffix in ["pages.ktx2", "bindings.bin"] {
        std::fs::rename(
            root.join(format!("assets/lightmaps/room.{suffix}")),
            root.join(format!("modules/north/assets/lightmaps/room.{suffix}")),
        )
        .unwrap();
    }
    std::fs::remove_file(root.join("content/lightmaps/room.ts")).unwrap();
    bake();
    assert!(!root.join("content/lightmaps/room.ts").exists());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle
            .manifest()
            .lightmaps
            .get("base/room")
            .map(String::as_str),
        Some("north/custom/lighting")
    );
    let lighting = bundle.load_lightmap("north/custom/lighting").unwrap();
    assert_eq!(lighting.file.models[0].parts[0].uv, expected_uv);
    let path = root.join("modules/north/assets/lightmaps/base/room.bindings.bin");
    let mut bytes = std::fs::read(&path).unwrap();
    // First part's xref + indices precede its interleaved f32 UVs.
    let part = &lighting.file.models[0].parts[0];
    let at = (part.xref.len() + part.indices.len()) * 4;
    bytes[at..at + 4].copy_from_slice(&0.375f32.to_le_bytes());
    std::fs::write(path, bytes).unwrap();
    let out = run(&["cook", "."]);
    assert!(out.status.success(), "{}", text(&out));
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle
            .load_lightmap("north/custom/lighting")
            .unwrap()
            .file
            .models[0]
            .parts[0]
            .uv[0][0],
        0.375
    );
    let out = run(&["check", "."]);
    assert!(out.status.success(), "{}", text(&out));
}

/// Extraction and UI eject author typed literals through the public tools,
/// preserving source bytes outside edits and refusing computed/colliding data.
#[test]
fn typed_locale_tools_extract_and_eject_without_overwriting_sources() {
    let root = TempDir::new("og-locale-tools");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Locales","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'locales',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write(
        "src/main.ts",
        "import {Confirm} from '@origingame/engine/ui/screens/confirm';export {Confirm};",
    );
    let declaration = "import {file} from '@origingame/engine/author';\n// keep this comment and spacing\nexport default file.strings({locale:'en',base:true,entries:{keep:'Untouched'}});\n";
    write("modules/north/content/labels.ts", declaration);
    write(
        "content/story/hello.ts",
        "import {file,input} from '@origingame/engine/author';export default file.dialogue({yarn:input.text('sources/story/hello.yarn')});",
    );
    write(
        "sources/story/hello.yarn",
        "#og-profile:yarn-spinner-3.2.2\n#og-name:hello\ntitle: Start\n---\nHello, 北! #line:hello\n===\n",
    );
    let run = |args: &[&str]| {
        og().args(args)
            .current_dir(&root)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    let out = run(&["i18n", "extract", "--write", "--json"]);
    assert!(out.status.success(), "{}", text(&out));
    let extracted: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(extracted["errors"], serde_json::json!([]), "{}", text(&out));
    let saved = std::fs::read_to_string(root.join("modules/north/content/labels.ts")).unwrap();
    assert!(saved.contains("// keep this comment and spacing"));
    assert!(saved.contains("keep:'Untouched'"));
    let out = run(&["i18n", "tables", "--json"]);
    assert!(out.status.success(), "{}", text(&out));
    let data: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(data["tables"][0]["id"], "north/labels");
    assert_eq!(data["tables"][0]["entries"]["hello"], "Hello, 北!");
    assert!(!root.join("content/strings/en.ts").exists());
    write(
        "sources/story/hello.yarn",
        "#og-profile:yarn-spinner-3.2.2\n#og-name:hello\ntitle: Start\n---\nMore #line:more\n===\n",
    );
    write(
        "modules/north/lib/messages.ts",
        "export const entries={keep:'Shared'};",
    );
    let computed = "import {file} from '@origingame/engine/author';import {entries} from '../lib/messages.ts';export default file.strings({locale:'en',base:true,entries});";
    write("modules/north/content/labels.ts", computed);
    let out = run(&["i18n", "extract", "--write", "--json"]);
    assert!(
        !out.status.success() && text(&out).contains("computed-source"),
        "{}",
        text(&out)
    );
    assert_eq!(
        std::fs::read_to_string(root.join("modules/north/content/labels.ts")).unwrap(),
        computed
    );
    write(
        "modules/north/lib/messages.ts",
        "throw new Error('index must not execute content');export const entries={keep:'Shared'};",
    );
    let out = run(&["i18n", "tables", "--index", "--json"]);
    assert!(out.status.success(), "{}", text(&out));
    let data: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        data["tables"],
        serde_json::json!([{"id":"north/labels","path":"modules/north/content/labels.ts"}])
    );
    write("modules/north/content/labels.ts", declaration);
    let out = run(&["ui", "eject", "screens/confirm"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(root.join("src/ui/screens/confirm.tsx").is_file());
    assert!(
        root.join("content/ui/screens/confirm.en.strings.ts")
            .is_file()
    );
    assert!(
        root.join("content/ui/screens/confirm.zh-CN.strings.ts")
            .is_file()
    );
    let out = run(&["i18n", "tables", "--json"]);
    assert!(out.status.success(), "{}", text(&out));
    let data: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let table = data["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "base/ui/screens/confirm.en.strings")
        .unwrap();
    assert_eq!(
        table["entries"]["game.screen.confirm.undone"],
        "Cannot be undone"
    );
    let before = std::fs::read(root.join("src/ui/screens/confirm.tsx")).unwrap();
    let out = run(&["ui", "eject", "screens/confirm"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert_eq!(
        std::fs::read(root.join("src/ui/screens/confirm.tsx")).unwrap(),
        before
    );
    assert!(!root.join("assets/ui").exists());
}

/// Typed terrain sources preserve raw-image ownership, ordered native edits,
/// material bundle references and per-input cache invalidation.
#[test]
fn typed_terrains_load_module_images_noise_and_ordered_model_stamps() {
    let root = TempDir::new("og-typed-terrain");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    for prefix in ["", "materials/"] {
        write(
            &format!("{prefix}og-project.json"),
            r#"{"version":1,"name":"Terrain","engine":"0.9.0"}"#,
        );
        write(&format!("{prefix}src/main.ts"), "export default {};");
    }
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'terrain',entry:'src/main.ts',cook:{uses:['materials/cooked']},modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write(
        "materials/project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'materials',entry:'src/main.ts'});",
    );
    write(
        "materials/content/materials.ts",
        "import {file} from '@origingame/engine/author'; export default file['texture-array']({kind:'pixel',layers:['assets/layer.png']});",
    );
    std::fs::create_dir_all(root.join("materials/assets")).unwrap();
    image::RgbaImage::from_pixel(2, 2, image::Rgba([73, 109, 193, 251]))
        .save(root.join("materials/assets/layer.png"))
        .unwrap();
    let cook = |dir: &Path| {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", root.join("cache"))
            .current_dir(dir)
            .output()
            .unwrap()
    };
    let out = cook(&root.join("materials"));
    assert!(out.status.success(), "{}", text(&out));
    let declaration = "import {file,ref} from '@origingame/engine/author'; export default file.terrain({source:{kind:'image',source:'assets/height.png',height:[-5,15]},size:[8,4],tile:16,control:['assets/control.png'],holes:'assets/holes.png',albedoArray:ref('base/materials'),layers:[{albedoLayer:0,normalStrength:0.75}],castShadows:false,collision:false,lod:{spacing:2,ring:32}});";
    write("modules/north/content/surface.ts", declaration);
    std::fs::create_dir_all(root.join("modules/north/assets")).unwrap();
    image::RgbaImage::from_pixel(1, 1, image::Rgba([73, 109, 193, 251]))
        .save(root.join("modules/north/assets/control.png"))
        .unwrap();
    image::GrayImage::from_raw(2, 1, vec![0, 255])
        .unwrap()
        .save(root.join("modules/north/assets/holes.png"))
        .unwrap();
    write(
        "content/edited.ts",
        "import {file,ref} from '@origingame/engine/author';export default file.terrain({source:{kind:'heightfield',field:{columns:5,rows:3,heights:Float32Array.from({length:15},()=>20)}},size:[8,4],tile:16,edits:[{kind:'brush',mode:'raise',center:[-2,0,0],radius:0.5,strength:3},{kind:'stamp',geometry:{kind:'modelGeometry',desc:{source:ref('base/stamp')}},transform:{position:[0,8,0]}},{kind:'brush',mode:'raise',center:[0,0,0],radius:0.5,strength:2}]});",
    );
    let mut previous_noise = None;
    let mut previous_image = None;
    for run in 0..3 {
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        let changed = run == 2;
        let pixels: Vec<u16> = if changed {
            vec![65535, 52428, 39321, 26214, 13107, 0]
        } else {
            vec![0, 13107, 26214, 39321, 52428, 65535]
        };
        image::ImageBuffer::<image::Luma<u16>, _>::from_raw(3, 2, pixels)
            .unwrap()
            .save(root.join("modules/north/assets/height.png"))
            .unwrap();
        write(
            "content/stamp.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author';export default file.model({{kind:'geometry',desc:{{conform:'off',items:[{{name:'stamp',input:{{kind:'part',desc:{{geometry:{{kind:'boxGeometry',desc:{{width:2,height:{},depth:2}}}}}}}}}}]}}}});",
                if changed { 4 } else { 2 }
            ),
        );
        write(
            "content/noise.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.terrain({{source:{{kind:'noise',columns:5,rows:3,height:[2,6],noise:{{octaves:2,frequency:1.7}}}},seed:{},tile:16}});",
                if changed { 11 } else { 7 }
            ),
        );
        let out = cook(&root);
        assert!(out.status.success(), "{}", text(&out));
        let b = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let terrain = b.load_terrain("north/surface").unwrap();
        assert_eq!(terrain.albedo_array.as_deref(), Some("base/materials"));
        assert_eq!(terrain.layers[0].layer.albedo_layer, Some(0));
        assert_eq!(terrain.layers[0].layer.normal_strength, 0.75);
        assert!(!terrain.collision && !terrain.cast_shadows);
        let field = terrain.field().unwrap();
        assert_eq!(
            (field.columns, field.rows, field.size.to_array()),
            (3, 2, [8.0, 4.0])
        );
        let expected = if changed {
            [15.0, 11.0, 7.0, 3.0, -1.0, -5.0]
        } else {
            [-5.0, -1.0, 3.0, 7.0, 11.0, 15.0]
        };
        for (actual, expected) in field.heights.iter().zip(expected) {
            assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
        }
        assert_eq!(
            field
                .holes
                .unwrap()
                .iter()
                .map(|&v| v != 0)
                .collect::<Vec<_>>(),
            [true, false]
        );
        assert!(
            field
                .control
                .unwrap()
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [73, 109, 193, 251])
        );
        assert!(
            b.read("base/materials").is_err(),
            "used material bundle was repacked"
        );
        let edited = b.load_terrain("base/edited").unwrap().field().unwrap();
        assert!((edited.heights[6] - 23.0).abs() < 0.001);
        assert!((edited.heights[7] - if changed { 8.0 } else { 9.0 }).abs() < 0.001);
        assert!((edited.heights[0] - 20.0).abs() < 0.001);
        let noise = b.load_terrain("base/noise").unwrap().field().unwrap();
        assert_eq!((noise.columns, noise.rows), (5, 3));
        assert!(noise.heights.iter().all(|&v| (2.0..=6.0).contains(&v)));
        assert!(
            noise
                .heights
                .iter()
                .any(|&v| (v - noise.heights[0]).abs() > 0.01)
        );
        let bytes = b.read("north/surface").unwrap().to_vec();
        if let Some(previous) = previous_image.replace(bytes.clone()) {
            assert_eq!(previous == bytes, !changed);
        }
        if let Some(previous) = previous_noise.replace(noise.heights.clone()) {
            assert_eq!(previous == noise.heights, !changed);
        }
    }
    write(
        "modules/north/content/surface.ts",
        &declaration.replace("ref('base/materials')", "ref('base/materials#layer0')"),
    );
    let out = cook(&root);
    assert!(
        !out.status.success(),
        "wrong external material kind accepted"
    );
    assert!(text(&out).contains("has kind texture"), "{}", text(&out));
    assert!(
        text(&out).contains("modules/north/content/surface.ts:"),
        "{}",
        text(&out)
    );
    write("modules/north/content/surface.ts", declaration);
    write(
        "assets/removed.ogt",
        "type terrain\nversion 1\n---\nheightmap \"height.png\"\n",
    );
    let out = cook(&root);
    assert!(!out.status.success(), "terrain OGT authoring accepted");
    assert!(
        text(&out).contains("terrain OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

/// Typed timeline tracks survive the public cook/load boundary; captured
/// captions and namespace bindings invalidate the same cache on later cooks.
#[test]
fn typed_timelines_load_tracks_captions_and_module_references() {
    use og_timeline::{FieldValue, Track};
    let root = TempDir::new("og-typed-timeline");
    let write = |file: &str, contents: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Timeline","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'timeline',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "export default {};");
    write(
        "content/scenes/room.ts",
        "import {file} from '@origingame/engine/author';export default file.scene({name:'Room'});",
    );
    write(
        "content/audio/tone.ts",
        "import {file} from '@origingame/engine/author';export default file.sound({rate:8000,channels:1,samples:[0,0.25,-0.5]});",
    );
    write(
        "content/timelines/child.ts",
        "import {file} from '@origingame/engine/author';export default file.timeline({duration:2,tracks:[{kind:'marker',keys:[{time:1,value:{name:'child'}}]}]});",
    );
    let declaration = r#"import {file,ref} from '@origingame/engine/author'; export default file.timeline({
        name:'Crossing', duration:8, paths:{rail:{kind:'linear',points:[[1,2,3],[5,3,-1]]}}, tracks:[
          {kind:'caption',imports:[{source:'sources/speech.vtt',offset:0.25}],locales:{ja:{keys:[{time:1,value:{text:'明示',end:2}}]}}},
          {kind:'camera',rail:'rail',blendIn:0.3,keys:[{time:3,ease:'power2.inOut',value:{at:1,target:[-2,1,4]}},{time:0,value:{at:0,fov:53}}]},
          {kind:'transform',entity:'crate',keys:[{time:0,value:{position:[2,-1,4],rotation:[0,90,0],scale:[1,2,3]}}]},
          {kind:'animation',entity:'actor',keys:[{time:0,value:{clip:'walk',blendIn:0.35,loop:false,speed:1.5}}]},
          {kind:'audio',keys:[{time:0,value:{sound:ref('base/audio/tone'),playbackRate:1.25,bus:'ui'}}]},
          {kind:'music',keys:[{time:0,value:{sound:ref('base/audio/tone'),end:7,fadeIn:0.2,fadeOut:0.7,loop:false}}]},
          {kind:'event',keys:[{time:4,value:{action:'spawn_wave:2',retroactive:true}}]},
          {kind:'transition',keys:[{time:0,value:{kind:'wipe',mode:'in',duration:0.7,from:'right',color:[0.1,0.2,0.4]}}]},
          {kind:'marker',keys:[{time:2,value:{name:'first'}},{time:2,value:{name:'second'}}]},
          {kind:'property',entity:'light',field:'og.PointLight.intensity',keys:[{time:0,value:3.5}]},
          {kind:'light',entity:'light',keys:[{time:0,value:{intensity:2.5,color:'#804020'}}]},
          {kind:'post',keys:[{time:0,value:{contrast:0.4}}]},
          {kind:'visibility',entity:'crate',keys:[{time:0,value:false}]},
          {kind:'blendshape',entity:'actor',keys:[{time:0,value:{w1:0.7}}]},
          {kind:'material',entity:'crate',index:2,keys:[{time:0,value:{standard:{roughness:0.3},mapOffset:[0.2,0.7],params:{pulse:[1,2,3,4]}}}]},
          {kind:'timeline',keys:[{time:1,value:{play:ref('base/timelines/child'),speed:1.5,loop:true}}]},
          {kind:'shot',keys:[{time:1,ease:'steps(3,end)',value:{camera:'closeup',blend:0.2}}]},
          {kind:'hotspot',keys:[{time:1,value:{id:'door',rect:[0.2,0.3,0.4,0.1],shape:'circle',kind:'inspect',label:'Door'}},{time:4,value:{id:'door',off:true}}]},
          {kind:'scene',keys:[{time:0,value:{scene:ref('base/scenes/room')}},{time:5,value:{scene:null}}]}
        ]});"#;
    write("content/timelines/intro.ts", declaration);
    write(
        "modules/north/content/timelines/intro.ts",
        "import {file} from '@origingame/engine/author';export default file.timeline({tracks:[{kind:'caption',imports:[{source:'sources/speech.vtt'}]}]});",
    );
    write(
        "modules/north/sources/speech.vtt",
        "WEBVTT\n\n00:00.100 --> 00:01.200\nNorth\n",
    );
    write(
        "sources/speech.ja.vtt",
        "WEBVTT\n\n00:00.100 --> 00:01.200\nIgnored\n",
    );
    let cook = || {
        og().args(["cook", "."])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for run in 0..3 {
        write(
            "sources/speech.vtt",
            if run == 2 {
                "WEBVTT\n\n00:00.500 --> 00:02.000\n<v Ada><b>Changed</b>\n"
            } else {
                "WEBVTT\n\n00:00.500 --> 00:02.000\n<v Ada><b>Base</b>\n"
            },
        );
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        if run == 2 {
            write(
                "sources/speech.fr.vtt",
                "WEBVTT\n\n00:00.200 --> 00:01.500\nFrançais\n",
            );
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let b = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let t = b
            .timeline(
                "base/timelines/intro",
                &b.read("base/timelines/intro").unwrap(),
            )
            .unwrap();
        assert_eq!(
            (t.name.as_str(), t.duration, t.tracks.len()),
            ("Crossing", 8.0, 19)
        );
        assert_eq!(t.paths[0].1.points[1].to_array(), [5.0, 3.0, -1.0]);
        let Track::Caption(c) = &t.tracks[0] else {
            panic!("caption")
        };
        assert_eq!(
            (c.keys[0].time, c.keys[0].end, c.keys[0].speaker.as_str()),
            (0.75, 2.25, "Ada")
        );
        assert_eq!(c.keys[0].text, if run == 2 { "Changed" } else { "Base" });
        assert_eq!(
            (c.keys[0].spans[0].position, c.keys[0].spans[0].length),
            (0, c.keys[0].text.chars().count() as u32)
        );
        assert_eq!(c.keys_in("ja")[0].text, "明示");
        if run == 2 {
            assert_eq!(c.keys_in("fr")[0].text, "Français");
        }
        let Track::Camera(c) = &t.tracks[1] else {
            panic!("camera")
        };
        assert_eq!(
            (c.keys[0].time, c.keys[1].time, c.blend_in),
            (0.0, 3.0, 0.3)
        );
        assert_eq!(c.keys[1].ease, og_core::ease::Ease::CubicInOut);
        let Track::Transform(tr) = &t.tracks[2] else {
            panic!("transform")
        };
        let turned = tr.keys[0].quaternion.unwrap() * og_core::glam::Vec3::Z;
        assert!((turned - og_core::glam::Vec3::X).length() < 0.0001);
        let Track::Audio(a) = &t.tracks[4] else {
            panic!("audio")
        };
        assert_eq!(
            (a[0].sound.as_str(), a[0].pitch, a[0].bus),
            ("base/audio/tone", 1.25, 3)
        );
        let Track::Marker(m) = &t.tracks[8] else {
            panic!("marker")
        };
        assert_eq!(
            (m[0].name.as_str(), m[1].name.as_str()),
            ("first", "second")
        );
        let Track::Fields(light) = &t.tracks[10] else {
            panic!("light")
        };
        let (_, FieldValue::Numbers(color)) = light.keys[0]
            .values
            .iter()
            .find(|(name, _)| name == "color")
            .unwrap()
        else {
            panic!("linear light color")
        };
        assert!((color[0] - 0.2158605).abs() < 0.000001);
        assert!((color[1] - 0.05126946).abs() < 0.000001);
        let Track::Fields(m) = &t.tracks[14] else {
            panic!("material")
        };
        assert_eq!(m.index, 2);
        assert!(m.keys[0].values.contains(&(
            "pulse".into(),
            FieldValue::Numbers(vec![1.0, 2.0, 3.0, 4.0])
        )));
        let Track::Scene(s) = &t.tracks[18] else {
            panic!("scene")
        };
        assert_eq!(
            (s[0].scene.as_str(), s[1].scene.as_str()),
            ("base/scenes/room", "")
        );
        let n = b
            .timeline(
                "north/timelines/intro",
                &b.read("north/timelines/intro").unwrap(),
            )
            .unwrap();
        let Track::Caption(nc) = &n.tracks[0] else {
            panic!("north caption")
        };
        assert_eq!((n.duration, nc.keys[0].text.as_str()), (1.2, "North"));
    }
    for invalid in [
        "{duration:1,tracks:[{kind:'transform',entity:'x',keys:[{time:0,value:{rotation:[0,0,0],quaternion:[0,0,0,1]}}]}]}",
        "{duration:1,tracks:[{kind:'camera',rail:'p',follow:'x',keys:[]}]}",
        "{duration:1,tracks:[{kind:'hotspot',keys:[{time:0,value:{id:'x',rect:[0.9,0,0.2,1]}}]}]}",
        "{duration:1,tracks:[{kind:'caption',imports:[{source:'sources/../speech.vtt'}]}]}",
        "{duration:1,tracks:[{kind:'marker',keys:[{time:2,value:{name:'late'}}]}]}",
        "{duration:1,tracks:[{kind:'timeline',keys:[{time:0,value:{play:ref('base/timelines/intro')}}]}]}",
    ] {
        write(
            "content/timelines/intro.ts",
            &format!(
                "import {{file,ref}} from '@origingame/engine/author';export default file.timeline({invalid});"
            ),
        );
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}");
    }
    write("content/timelines/intro.ts", declaration);
    write(
        "assets/removed.ogt",
        "type timeline\nversion 1\n---\nduration 1\n",
    );
    let out = cook();
    assert!(!out.status.success(), "accepted author OGT");
}

/// Video declarations, source tools and incremental cook use the same typed
/// input and module ownership; actual frame outputs bound references.
#[test]
fn typed_videos_cook_module_media_and_localized_audio() {
    let root = TempDir::new("og-typed-video");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Video","engine":"0.9.0"}"#,
    );
    let project = "import {defineProject,ref} from '@origingame/engine/author'; export default defineProject({id:'video',entry:'src/main.ts',roots:[ref('base/clips/intro#frames#layer1')],modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});";
    write("project.ts", project);
    write("src/main.ts", "export default {};");
    write("lib/clip.ts", "export const limit=0.5;");
    let declaration = "import {file} from '@origingame/engine/author'; import {limit} from '../../lib/clip.ts'; export default file.video({source:'sources/intro.mp4',fps:4,maxSeconds:limit,kind:'pixel',aiGenerated:true,placeholder:true,audioLocale:{fr:'sources/fr.wav'}});";
    write("content/clips/intro.ts", declaration);
    write(
        "modules/north/content/clips/intro.ts",
        "import {file} from '@origingame/engine/author'; export default file.video({source:'sources/intro.mp4',fps:4,maxSeconds:0.5,kind:'pixel'});",
    );
    let media = |file: &str, size: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let out = Command::new(og_cook::cook::video::ffmpeg_bin())
            .args([
                "-v",
                "error",
                "-nostdin",
                "-y",
                "-f",
                "lavfi",
                "-i",
                &format!("testsrc2=size={size}:rate=4:duration=1"),
                "-an",
                "-c:v",
                "libx264",
                "-threads",
                "1",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
    };
    let audio = |sample: i16| {
        let mut wav = hound::WavWriter::create(
            root.join("sources/fr.wav"),
            hound::WavSpec {
                channels: 1,
                sample_rate: 8000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        for n in 0..8000 {
            wav.write_sample(if n % 17 < 6 { sample } else { -sample })
                .unwrap();
        }
        wav.finalize().unwrap();
    };
    media("modules/north/sources/intro.mp4", "80x64");
    let cache = root.join("cache");
    let cook = || {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let mut previous = None;
    let mut north = None;
    for (run, size) in ["96x64", "96x64", "128x72", "128x72"]
        .into_iter()
        .enumerate()
    {
        if run == 3 {
            audio(19000);
        } else if run != 1 {
            media("sources/intro.mp4", size);
            audio(if run == 0 { 7000 } else { 13000 });
        } else {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let id = "base/clips/intro";
        let meta = bundle.video(id, &bundle.read(id).unwrap()).unwrap();
        assert_eq!(
            (
                meta.width,
                meta.height,
                meta.frames,
                meta.fps,
                meta.streamed,
                meta.audio
            ),
            (
                if run >= 2 { 128 } else { 96 },
                if run >= 2 { 72 } else { 64 },
                2,
                4,
                0,
                0
            )
        );
        let frames = bundle.load_texture_array(&format!("{id}#frames")).unwrap();
        assert_eq!(
            frames.layers,
            [format!("{id}#frames#layer0"), format!("{id}#frames#layer1")]
        );
        let layer = bundle.load_texture(&frames.layers[1]).unwrap();
        let localized = bundle.load_sound(&format!("{id}#audio.fr")).unwrap();
        assert_eq!(localized.meta.channels, 2);
        assert_eq!(localized.meta.sample_rate, 48000);
        let other = bundle
            .load_texture_array("north/clips/intro#frames")
            .unwrap();
        assert_eq!((other.meta.width, other.meta.height), (80, 64));
        if let Some((before, sound)) = previous {
            assert_eq!(
                layer == before,
                run == 1 || run == 3,
                "source refresh/cache mismatch"
            );
            assert_eq!(
                localized == sound,
                run == 1,
                "localized audio refresh/cache mismatch"
            );
            assert_eq!(Some(&other), north.as_ref());
        }
        previous = Some((layer, localized));
        north = Some(other);
    }
    // The source tools execute the declaration, including imported expressions,
    // instead of looking for textual source/maxSeconds lines.
    let edges = og()
        .args(["video", "edges", "content/clips/intro.ts", "--out", "edges"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(edges.status.success(), "{}", text(&edges));
    assert!(root.join("edges/intro.first.png").is_file());
    assert!(root.join("edges/intro.last.png").is_file());
    write(
        "content/segments/intro.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file.timeline({duration:0.5,tracks:[{kind:'video',keys:[{time:0,value:{video:ref('base/clips/intro')}}]}]});",
    );
    write(
        "content/story/intro.ts",
        "import {file,input,ref} from '@origingame/engine/author';export default file.dialogue({yarn:input.text('sources/story/intro.yarn'),assets:{'base/segments/intro':ref('base/segments/intro')}});",
    );
    write(
        "sources/story/intro.yarn",
        "#og-profile:yarn-spinner-3.2.2\n#og-name:intro\ntitle: intro\nog-flow: base/segments/intro\n---\n===\n",
    );
    let out = og()
        .args(["flow", "list", ".", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("sources/intro.mp4"), "{}", text(&out));
    assert!(text(&out).contains("placeholder"), "{}", text(&out));
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    for invalid in [
        "{source:'sources/intro.mp4',keyframe:1}",
        "{source:'sources/intro.mp4',fps:0}",
        "{source:'sources/intro.mp4',maxSeconds:0}",
        "{source:'sources/intro.mp4',posterSize:1,stream:true}",
        "{source:'sources/intro.mp4',audioLocale:{fr:'sources/fr.wav',FR:'sources/fr.wav'}}",
        "{source:'sources/intro.mp4',audioLocale:{'fr/CA':'sources/fr.wav'}}",
        "{source:'../sources/intro.mp4'}",
        "{source:'sources/../sources/intro.mp4'}",
        "{source:'intro.mp4'}",
        "{source:'sources/intro.png'}",
        "{source:'sources/missing.mov'}",
    ] {
        write(
            "content/clips/intro.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.video({invalid});"
            ),
        );
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}");
        assert!(
            text(&out).contains("content/clips/intro.ts"),
            "{}",
            text(&out)
        );
    }
    write("content/clips/intro.ts", declaration);
    write("project.ts", &project.replace("#layer1", "#layer2"));
    assert!(
        !cook().status.success(),
        "accepted nonexistent frame reference"
    );
    write("project.ts", project);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("intro.mp4", root.join("sources/link.mp4")).unwrap();
        write(
            "content/clips/intro.ts",
            &declaration.replace("intro.mp4", "link.mp4"),
        );
        assert!(!cook().status.success(), "accepted symlink media");
        std::fs::remove_file(root.join("sources/link.mp4")).unwrap();
        write("content/clips/intro.ts", declaration);
    }
    write(
        "assets/old.ogt",
        "type video\nversion 1\n---\nsource \"intro.mp4\"\n",
    );
    let out = cook();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("video OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

/// The public icon author/cook/load path preserves aliases and module ownership,
/// refreshes external SVG inputs, and ships the default set without an OGT path.
#[test]
fn typed_icons_load_module_svg_aliases_and_builtin_styles() {
    let root = TempDir::new("og-typed-icons");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Icons","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'icons',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "export default {};");
    let declaration = "import {file} from '@origingame/engine/author'; export default file.icons({source:'sources/icons',size:24,range:5,join:'round',sticker:[0.6,-1,2],icons:[['gate/界','nested/gate'],['alias','nested/gate']]});";
    write("content/icons/ui.ts", declaration);
    write("modules/north/content/icons/ui.ts", declaration);
    let svg = |edge: u32| {
        format!(
            r#"<svg viewBox="0 0 24 24"><path d="M2 3 H{edge} V19 H2 Z" fill="var(--color-icon-gold)"/><path d="M4 6 L13 17" fill="none" stroke="currentColor" stroke-width="2"/></svg>"#
        )
    };
    write("modules/north/sources/icons/nested/gate.svg", &svg(9));
    let cache = root.join("cache");
    let cook = || {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let mut previous = None;
    let mut north = None;
    for (run, edge) in [16, 16, 21].into_iter().enumerate() {
        write("sources/icons/nested/gate.svg", &svg(edge));
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let font = bundle.load_font("base/icons/ui").unwrap();
        assert_eq!(font.names, ["gate/界", "alias"]);
        assert!(font.file.is_none());
        assert!(font.tones.contains("icon-gold"));
        assert!(font.glyph('\u{e000}').is_some());
        assert!(font.glyph('\u{e001}').is_some());
        let other = bundle.load_font("north/icons/ui").unwrap();
        assert_ne!(font.pages, other.pages, "module SVGs were mixed");
        if let Some(before) = previous {
            assert_eq!(
                font == before,
                run == 1,
                "SVG change did not refresh the font"
            );
            assert_eq!(
                Some(&other),
                north.as_ref(),
                "base SVG invalidated another module"
            );
        }
        previous = Some(font);
        north = Some(other);
    }
    for invalid in [
        "{source:'sources/icons',icons:[['a','nested/gate'],['a','nested/gate']]}",
        r#"{source:'sources/icons',icons:[['bad\nname','nested/gate']]}"#,
        "{source:'sources/icons',icons:[['a','../gate']]}",
        "{source:'sources/icons/../icons',icons:[['a','nested/gate']]}",
        "{source:'../sources/icons',icons:[['a','nested/gate']]}",
        "{source:'sources/icons',icons:[['a','nested/gate']],sticker:[0,5,0]}",
        "{source:'sources/icons',icons:[]}",
    ] {
        write(
            "content/icons/ui.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.icons({invalid});"
            ),
        );
        let out = cook();
        assert!(
            !out.status.success(),
            "invalid icon declaration accepted: {invalid}"
        );
    }
    write("content/icons/ui.ts", declaration);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("nested", root.join("sources/icons/link")).unwrap();
        write(
            "content/icons/ui.ts",
            &declaration.replace("nested/gate", "link/gate"),
        );
        assert!(!cook().status.success(), "symlink SVG parent accepted");
        std::fs::remove_file(root.join("sources/icons/link")).unwrap();
        write("content/icons/ui.ts", declaration);
    }
    write(
        "assets/old.ogt",
        "type icons\nversion 1\n---\nicon \"gate\"\n",
    );
    let out = cook();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("icons OGT authoring is removed"),
        "{}",
        text(&out)
    );
    std::fs::remove_file(root.join("assets/old.ogt")).unwrap();
    // A style pack pulls in a typed built-in declaration and its imported names,
    // using the same VM and native baker as the game's own content.
    write(
        "content/styles/test.ts",
        "import {file,ref} from '@origingame/engine/author'; export default file.pack({iconSet:ref('@styles/shared/icons/game.ts')});",
    );
    for _ in 0..2 {
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let font = bundle.load_font("@styles/shared/icons/game.ts").unwrap();
        assert_eq!(font.names.first().map(String::as_str), Some("coin"));
        assert_eq!(font.names.last().map(String::as_str), Some("charm"));
        assert!(font.names.iter().any(|n| n == "heart"));
        assert!(font.tones.contains("icon-gold"));
        assert!(font.file.is_none());
        assert!(bundle.read("@styles/shared/icons/game.ogt").is_err());
    }
    // Installed styles can live in an arbitrarily named directory outside the
    // project. Both imported TS and native SVG inputs invalidate its checkpoint.
    let styles = TempDir::new("og-external-icon-styles");
    let icons = styles.join("shared/icons");
    std::fs::create_dir_all(&icons).unwrap();
    std::fs::write(
        icons.join("game.ts"),
        "import {file} from '@origingame/engine/author'; import {icons} from './names.ts'; export default file.icons({source:'shared/icons',size:24,icons});",
    )
    .unwrap();
    std::fs::write(
        icons.join("names.ts"),
        "export const icons = [['external','gate']] as const;",
    )
    .unwrap();
    let mut previous = None;
    for run in 0..4 {
        match run {
            0 => std::fs::write(icons.join("gate.svg"), svg(7)).unwrap(),
            2 => std::fs::write(icons.join("gate.svg"), svg(23)).unwrap(),
            3 => std::fs::write(
                icons.join("names.ts"),
                "export const icons = [['renamed','gate']] as const;",
            )
            .unwrap(),
            _ => {}
        }
        let out = og()
            .args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .env("OG_STYLES_DIR", styles.as_path())
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let font = bundle.load_font("@styles/shared/icons/game.ts").unwrap();
        assert_eq!(font.names, [if run == 3 { "renamed" } else { "external" }]);
        if let Some(before) = previous {
            assert_eq!(font == before, run == 1, "external change was missed");
        }
        previous = Some(font);
    }
}

/// Typed environment inputs retain native bake semantics, module ownership and
/// dependency invalidation across the public CLI and bundle loader.
#[test]
fn typed_environments_cook_module_images_and_refresh_cached_maps() {
    use og_render_data::bake::{BakeDesc, Equirect, bake};
    let root = TempDir::new("og-typed-environments");
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Environments","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'environments',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("src/main.ts", "export default {};");
    let source = "import {file} from '@origingame/engine/author'; export default file.environment({source:{equirect:'assets/sky.hdr'},size:16,intensity:1.3,rotation:37,samples:16});";
    write("content/env/sky.ts", source);
    write("modules/north/content/env/sky.ts", source);
    let hdr = |file: &str, gain: f32| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let pixels: Vec<_> = (0..128)
            .map(|i| {
                image::Rgb([
                    (1 + i % 16) as f32 * gain / 8.0,
                    (1 + i / 16) as f32 / 7.0,
                    (1 + i % 5) as f32 / 3.0,
                ])
            })
            .collect();
        image::codecs::hdr::HdrEncoder::new(std::fs::File::create(&path).unwrap())
            .encode(&pixels, 16, 8)
            .unwrap();
        let image = image::load_from_memory(&std::fs::read(path).unwrap())
            .unwrap()
            .to_rgb32f();
        bake(
            &Equirect {
                width: 16,
                height: 8,
                rgb: image.as_raw(),
            },
            &BakeDesc {
                size: 16,
                intensity: 1.3,
                rotation_degrees: 37.0,
                samples: 16,
                sun: false,
            },
        )
        .unwrap()
    };
    let north = hdr("modules/north/assets/sky.hdr", 0.35);
    let cache = root.join("cache");
    let cook = || {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let mut previous = None;
    for (run, gain) in [1.0, 1.0, 2.3].into_iter().enumerate() {
        let expected = hdr("assets/sky.hdr", gain);
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let actual = bundle.load_environment("base/env/sky").unwrap();
        assert_eq!(actual, expected);
        assert_eq!(bundle.load_environment("north/env/sky").unwrap(), north);
        assert!(
            bundle.read("sky.hdr").is_err(),
            "owned image became a standalone texture"
        );
        if let Some(before) = previous {
            assert_eq!(actual == before, run == 1);
        }
        previous = Some(actual);
    }
    // LDR retains a separately authored background, including upper-hemisphere
    // fill. The source is outside assets/, so only its declared owner reads it.
    std::fs::create_dir_all(root.join("sources")).unwrap();
    image::RgbImage::from_fn(16, 5, |x, y| {
        image::Rgb([(x * 11) as u8, (y * 37) as u8, 83])
    })
    .save(root.join("sources/upper.png"))
    .unwrap();
    write(
        "content/ldr.ts",
        "import {file} from '@origingame/engine/author'; export default file.environment({source:{equirect:'sources/upper.png'},size:16,samples:16,hemisphere:'upper',ground:[0.12,0.1,0.08],sun:'keep'});",
    );
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let ldr = bundle.load_environment("base/ldr").unwrap();
    assert!(!ldr.background.is_empty());
    assert!(ldr.background_size > 0);
    assert_eq!(ldr.sun.intensity, 0.0);
    for (invalid, message) in [
        (
            "{source:{equirect:'assets/sky.hdr'},size:24}",
            "power of two",
        ),
        (
            "{source:{equirect:'assets/sky.hdr'},prefiltered:true}",
            "KTX2",
        ),
        (
            "{source:{ktx2:'sources/map.ktx2'},prefiltered:true,samples:16}",
            "samples",
        ),
        (
            "{source:{ktx2:'sources/map.ktx2'},prefiltered:true,sun:'extract'}",
            "sun extract",
        ),
        (
            "{source:{equirect:'assets/sky.hdr'},ground:[0.1,0.2,0.3]}",
            "hemisphere",
        ),
        (
            "{source:{faces:['a.hdr','b.png','c.hdr','d.hdr','e.hdr','f.hdr']}}",
            "all be HDR",
        ),
        ("{source:{equirect:'assets/../sky.hdr'}}", "canonical"),
        ("{source:{equirect:'sky.hdr'}}", "owning module"),
        (
            "{source:{equirect:'assets/sky.hdr',ktx2:'sources/map.ktx2'}}",
            "/source",
        ),
    ] {
        write(
            "content/bad.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.environment({invalid});"
            ),
        );
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}");
        assert!(text(&out).contains(message), "{}", text(&out));
    }
    std::fs::remove_file(root.join("content/bad.ts")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("assets/sky.hdr"), root.join("sources/link.hdr"))
            .unwrap();
        write(
            "content/env/sky.ts",
            &source.replace("assets/sky.hdr", "sources/link.hdr"),
        );
        let out = cook();
        assert!(!out.status.success(), "symlink source accepted");
        assert!(text(&out).contains("symlink"), "{}", text(&out));
        std::fs::remove_file(root.join("sources/link.hdr")).unwrap();
        write("content/env/sky.ts", source);
    }
    write(
        "assets/old.ogt",
        "type environment\nversion 1\n---\nsource \"sky.hdr\"\n",
    );
    let out = cook();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("environment OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

#[test]
fn typed_vectors_cook_explicit_svg_inputs_and_inline_geometry() {
    let root = TempDir::new("og-typed-vectors");
    let write = |file: &str, text: &str| {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Vectors","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'vectors',entry:'src/main.ts',modules:[{id:'island',root:'modules/island',dependsOn:['base']}]});",
    );
    write("src/main.ts", "export default {};\n");
    let file = "import {file} from '@origingame/engine/author'; export default file.vector({source:'assets/emblem.svg',tolerance:0.05});";
    let svg = "<svg viewBox='-3 2 17 11'><path d='M -1 3 L 12 3 L 4 10 Z' fill='#c11153'/><circle cx='7' cy='6' r='2' fill='#17b53b'/></svg>";
    write("content/nested/from-file.ts", file);
    write("modules/island/content/nested/from-file.ts", file);
    write(
        "modules/island/assets/emblem.svg",
        &svg.replace("17 11", "23 7"),
    );
    let cache = root.join("cache");
    let cook = || {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let mut previous = None;
    for (run, width) in [17, 17, 31].into_iter().enumerate() {
        let svg = svg.replace("17 11", &format!("{width} 11"));
        write("assets/emblem.svg", &svg);
        write(
            "content/inline.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.vector({{svg:{},tolerance:0.05}});",
                serde_json::to_string(&svg).unwrap()
            ),
        );
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let read = |id| bundle.vector(id, &bundle.read(id).unwrap()).unwrap();
        let external = read("base/nested/from-file");
        assert_eq!(external, read("base/inline"));
        assert_eq!(external.view_box, [-3.0, 2.0, width as f32, 11.0]);
        assert!(!external.indices.is_empty());
        let island = read("island/nested/from-file");
        assert_eq!(island.view_box, [-3.0, 2.0, 23.0, 7.0]);
        if let Some((base, other)) = previous {
            assert_eq!(external == base, run == 1);
            assert_eq!(island, other);
        }
        previous = Some((external, island));
        assert!(bundle.read("nested/from-file.ogt").is_err());
    }
    for invalid in [
        "{source:'assets/emblem.svg',svg:'<svg/>'}",
        "{source:'assets/emblem.svg',tolerance:0}",
        "{}",
    ] {
        write(
            "content/nested/from-file.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author'; export default file.vector({invalid});"
            ),
        );
        let out = cook();
        assert!(
            !out.status.success(),
            "invalid TS vector accepted: {invalid}"
        );
        assert!(
            text(&out).contains("content/nested/from-file.ts:"),
            "{}",
            text(&out)
        );
    }
    for path in ["../emblem.svg", "emblem.svg", "assets/missing.svg"] {
        write(
            "content/nested/from-file.ts",
            &file.replace("assets/emblem.svg", path),
        );
        assert!(
            !cook().status.success(),
            "invalid SVG input accepted: {path}"
        );
    }
    write("content/nested/from-file.ts", file);
    write(
        "assets/emblem.svg",
        "<svg viewBox='0 0 17 11'><text>unsupported</text></svg>",
    );
    let out = cook();
    assert!(!out.status.success(), "unsupported SVG accepted");
    assert!(text(&out).contains("<text>"), "{}", text(&out));
    write("assets/emblem.svg", svg);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("assets/emblem.svg"), root.join("assets/link.svg"))
            .unwrap();
        write(
            "content/nested/from-file.ts",
            &file.replace("emblem.svg", "link.svg"),
        );
        assert!(!cook().status.success(), "symlink SVG accepted");
        std::fs::remove_file(root.join("assets/link.svg")).unwrap();
        write("content/nested/from-file.ts", file);
    }
    write(
        "assets/old.ogt",
        "type vector\nversion 1\n---\nsource \"emblem.svg\"\n",
    );
    let out = cook();
    assert!(!out.status.success(), "removed OGT vector accepted");
    assert!(
        text(&out).contains("OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

/// The public array path retains layer order, module-local file inputs and
/// warm-cache invalidation, and rejects removed or nonexistent author inputs.
#[test]
fn typed_texture_arrays_cook_module_inputs_and_real_layers() {
    let root = TempDir::new("og-typed-arrays");
    let write = |file: &str, text: &str| {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    };
    write(
        "og-project.json",
        r#"{"version":1,"name":"Arrays","engine":"0.9.0"}"#,
    );
    let project = "import {defineProject,ref} from '@origingame/engine/author'; export default defineProject({id:'arrays',entry:'src/main.ts',modules:[{id:'island',root:'modules/island',dependsOn:['base']}],roots:[ref('base/nested/tiles#layer1'),ref('island/nested/tiles#layer0')]});";
    write("project.ts", project);
    write("src/main.ts", "export default {};\n");
    let source = "import {file} from '@origingame/engine/author'; export default file['texture-array']({kind:'color',layers:['assets/a.png','assets/b.png']});";
    write("content/nested/tiles.ts", source);
    write(
        "modules/island/content/nested/tiles.ts",
        &source.replace(",'assets/b.png'", ""),
    );
    let image = |file: &str, size: (u32, u32), color: [u8; 4]| {
        let at = root.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        image::save_buffer(
            at,
            &color.repeat((size.0 * size.1) as usize),
            size.0,
            size.1,
            image::ColorType::Rgba8,
        )
        .unwrap();
    };
    image("assets/a.png", (4, 2), [193, 17, 83, 255]);
    image("assets/b.png", (4, 2), [23, 181, 59, 255]);
    image("modules/island/assets/a.png", (3, 5), [7, 41, 211, 255]);
    for name in ["a", "b"] {
        write(
            &format!("content/{name}.ts"),
            &format!(
                "import {{file}} from '@origingame/engine/author';export default file.import({{kind:'texture',source:'assets/{name}.png'}});"
            ),
        );
    }
    let cache = root.join("cache");
    let cook = || {
        og().args(["cook", ".", "--json"])
            .env("OG_COOK_CACHE", &cache)
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let load = || og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let mut previous = None;
    for (run, changed) in [false, false, true].into_iter().enumerate() {
        if run == 1 {
            std::fs::remove_dir_all(root.join("cooked")).unwrap();
        }
        if changed {
            image("assets/b.png", (4, 2), [213, 73, 11, 255]);
        }
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = load();
        let array = bundle.load_texture_array("base/nested/tiles").unwrap();
        assert_eq!(
            (array.meta.width, array.meta.height, array.meta.layers),
            (4, 2, 2)
        );
        assert_eq!(
            array.layers,
            ["base/nested/tiles#layer0", "base/nested/tiles#layer1"]
        );
        for (n, id) in ["base/a", "base/b"].into_iter().enumerate() {
            assert_eq!(
                bundle.load_texture(&array.layers[n]).unwrap().ktx2,
                bundle.load_texture(id).unwrap().ktx2
            );
        }
        let island = bundle.load_texture_array("island/nested/tiles").unwrap();
        assert_eq!(
            (island.meta.width, island.meta.height, island.meta.layers),
            (3, 5, 1)
        );
        let current = (
            bundle
                .load_texture("base/nested/tiles#layer1")
                .unwrap()
                .ktx2,
            bundle
                .load_texture("island/nested/tiles#layer0")
                .unwrap()
                .ktx2,
        );
        if let Some((base, island)) = previous {
            assert_eq!(current.1, island);
            assert_eq!(current.0 == base, !changed);
        }
        previous = Some(current);
        assert!(bundle.load_texture("base/nested/tiles#layer2").is_err());
        assert!(bundle.read("nested/tiles.ogt").is_err());
    }
    image("assets/b.png", (2, 4), [23, 181, 59, 255]);
    let out = cook();
    assert!(
        !out.status.success(),
        "mismatched layer dimensions accepted"
    );
    assert!(text(&out).contains("one size"), "{}", text(&out));
    image("assets/b.png", (4, 2), [23, 181, 59, 255]);
    write(
        "project.ts",
        &project.replace("tiles#layer1", "tiles#layer2"),
    );
    let out = cook();
    assert!(!out.status.success(), "invented array layer accepted");
    assert!(
        text(&out).contains("base/nested/tiles#layer2"),
        "{}",
        text(&out)
    );
    write("project.ts", project);
    for path in [
        "../a.png",
        "/tmp/a.png",
        "assets/missing.png",
        "a.png",
        "assets\\a.png",
    ] {
        write(
            "content/nested/tiles.ts",
            &source.replace("'assets/a.png'", &serde_json::to_string(path).unwrap()),
        );
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted layer {path}: {}",
            text(&out)
        );
    }
    write("content/nested/tiles.ts", source);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("assets/a.png"), root.join("assets/link.png"))
            .unwrap();
        write(
            "content/nested/tiles.ts",
            &source.replace("assets/a.png", "assets/link.png"),
        );
        assert!(!cook().status.success(), "symlink image accepted");
        std::fs::remove_file(root.join("assets/link.png")).unwrap();
        write("content/nested/tiles.ts", source);
    }
    write(
        "assets/old.ogt",
        "type texture-array\nversion 1\n---\nlayer \"a.png\"\n",
    );
    let out = cook();
    assert!(!out.status.success(), "removed OGT author input accepted");
    assert!(
        text(&out).contains("OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

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
        "content/scenes/harbor.ts",
        "content/scenes/inland.ts",
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
        assert!(bundle.read("base/scenes/harbor").is_ok());
        assert!(
            bundle.read("scenes/harbor.ogt").is_err(),
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
        root.join("content/scenes/Harbor.ts"),
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
        text(&collision).contains("after case folding"),
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
    write(
        "content/scenes/harbor.ts",
        "import {file} from '@origingame/engine/author';export default file.scene({entities:[]});",
    );
    write(
        "content/scenes/inland.ts",
        "import {file} from '@origingame/engine/author';export default file.scene({entities:[]});",
    );
    write(
        "content/looks/harbor.ts",
        "import {file} from '@origingame/engine/author';export default file.look({});",
    );
    write(
        "content/dialogue/arrival.ts",
        "import {file} from '@origingame/engine/author';export default file.dialogue({name:'arrival',start:'Start',nodes:[{id:'Start',steps:[{kind:'line',id:'welcome',text:{key:'arrival.welcome'}}]}]});",
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
            "scenes":{"harbor":"base/scenes/harbor","inland":"base/scenes/inland"},
            "profile":"3d","loading":"game","look":"base/looks/harbor",
            "flow":{"dialogue":"base/dialogue/arrival","screen":"game"}
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
    write(
        "content/scenes/harbor.ts",
        "import {file} from '@origingame/engine/author';export default file.prefab({entity:{name:'harbor'}});",
    );
    let wrong_kind = snapshot();
    assert!(!wrong_kind.status.success());
    assert!(
        text(&wrong_kind).contains("expected scene, got prefab"),
        "{}",
        text(&wrong_kind)
    );
    std::fs::remove_file(root.join("content/scenes/harbor.ts")).unwrap();
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
    for (name, declaration) in [
        (
            "surface",
            "file.effect({main:{texture:ref('base/volume#emission')},blend:'additive'})",
        ),
        (
            "emitter",
            "file.particles({maxParticles:1,rate:0,bursts:[{time:0,count:1}],render:{kind:'sprite'},array:{asset:ref('base/volume#array'),layers:3,mode:{kind:'life',cycles:1}},effect:ref('base/surface')})",
        ),
        (
            "composed",
            "file.vfx({duration:0.75,elements:[{node:{kind:'emitter',asset:ref('base/emitter')}}]})",
        ),
    ] {
        std::fs::write(
            root.join(format!("content/{name}.ts")),
            format!("import {{file,ref}} from '@origingame/engine/author'; export default {declaration};"),
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
    let bytes = bundle.read("base/composed").unwrap();
    let vfx = bundle.vfx("base/composed", &bytes).unwrap();
    assert_eq!(vfx.duration, 0.75);
    assert_eq!(vfx.references().emitters, ["base/emitter"]);
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
    // Checked input rejects unknown fields without a body-text bridge.
    std::fs::write(
        root.join("content/surface.ts"),
        "import {file} from '@origingame/engine/author'; export default file.effect({notAnEffectField:3});",
    )
    .unwrap();
    let out = cook("off");
    assert!(!out.status.success(), "invalid material accepted");
}

/// A graph asset resolves clip names at cook, keeps bindings in its
/// typed blob, and revalidates them when the source changes.
#[test]
fn animation_graph_cooks_with_tracked_clip_bindings() {
    use og_anim::cooked::BundleAnim;
    use og_cook::runtime::Bundle;
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
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(
        root.join("content/walker.ts"),
        "import {file} from '@origingame/engine/author';export default file.import({kind:'model',source:'assets/walker.glb',conform:'off'});",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("lib/rate.ts"), "export const rate=0.75;").unwrap();
    let graph = "import {file,ref} from '@origingame/engine/author';\nimport {rate} from '../lib/rate.ts';\nexport default file.animgraph({bindings:[{name:'stride',source:ref('base/walker'),clip:'walk'}],params:['x','y'],motions:{moving:{kind:'clip',clip:'stride',speed:rate,mirror:true,looping:false},blend:{kind:'blend1d',param:'x',children:[{motion:'moving',threshold:3},{motion:'plane',threshold:-2}]},plane:{kind:'blend2d',x:'x',y:'y',children:[{motion:'moving',point:[-3,2]}]}},layers:[{name:'base',states:[{name:'active',motion:'moving',speed:0.5,sync:'gait'},{name:'other',motion:'blend'}],entry:'active',mask:{hips:0.25,spine:0.75},transitions:[{to:'other',exit:1.25,fade:0.4,interrupt:'source',conditions:[{param:'y',op:'lt',value:-0.125}]}]}]});";
    std::fs::write(root.join("content/graph.ts"), graph).unwrap();
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
        .animgraph("base/graph", &bundle.read("base/graph").unwrap())
        .unwrap();
    assert!(!source.sprite);
    assert_eq!(
        (
            source.bindings[0].name.as_str(),
            source.bindings[0].clip.as_str()
        ),
        ("stride", "walk")
    );
    assert_eq!(source.bindings[0].source, "base/walker");
    assert_eq!(source.asset.params, ["x", "y"]);
    assert_eq!(source.asset.sync_groups, ["gait"]);
    assert_eq!(source.asset.layers[0].states[0].speed, 0.5);
    assert_eq!(source.asset.layers[0].transitions[0].exit, Some(1.25));
    assert_eq!(source.asset.layers[0].transitions[0].fade, 0.4);
    assert_eq!(
        source.asset.layers[0].transitions[0].conditions[0].value,
        -0.125
    );
    assert_eq!(source.asset.layers[0].mask[0].weight, 0.25);
    assert!(matches!(
        source.asset.nodes[1].motion,
        og_anim::Motion::Clip {
            speed: 0.75,
            mirror: true,
            looping: false,
            ..
        }
    ));
    assert!(
        matches!(&source.asset.nodes[0].motion, og_anim::Motion::Blend1D {param:0,children} if children == &[( -2.0, 2), (3.0, 1)])
    );
    assert!(
        matches!(&source.asset.nodes[2].motion, og_anim::Motion::Blend2D {x:0,y:1,children} if children == &[(og_core::glam::Vec2::new(-3.0, 2.0),1)])
    );
    let native = source.to_bytes();
    for end in 0..native.len() {
        assert!(og_anim::GraphSource::from_bytes(&native[..end]).is_err());
    }
    let mut trailing = native.clone();
    trailing.push(0);
    assert!(og_anim::GraphSource::from_bytes(&trailing).is_err());
    assert_eq!(
        og_anim::GraphSource::from_bytes(&native).unwrap().asset,
        source.asset
    );
    let graph_bytes = bundle.read("base/graph").unwrap().to_vec();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .read("base/graph")
            .unwrap()
            .as_ref(),
        graph_bytes
    );
    std::fs::write(root.join("lib/rate.ts"), "export const rate=1.125;").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let changed = Bundle::open(&root.join("cooked")).unwrap();
    let changed = changed
        .animgraph("base/graph", &changed.read("base/graph").unwrap())
        .unwrap();
    assert!(matches!(
        changed.asset.nodes[1].motion,
        og_anim::Motion::Clip { speed: 1.125, .. }
    ));
    let authored = r#"import {file} from '@origingame/engine/author';export default file.import({kind:'model',source:'assets/walker.glb',conform:'off',animation:{rate:4,compress:true,clips:[{name:'designed',channels:[{node:"hips",path:'translation',interpolation:'cubicSpline',keys:[{time:0,values:[0,0,0,2,3,5,8,0,0]},{time:1,values:[0,0,0,6,7,9,0,0,0]}]},{node:'body',path:'visibility',interpolation:'step',keys:[{time:0,values:[1]},{time:0.5,values:[0]},{time:1,values:[1]}]}]}],events:[{clip:'designed',marker:'half',time:0.5}]}});"#;
    std::fs::write(root.join("content/walker.ts"), authored).unwrap();
    std::fs::write(
        root.join("content/graph.ts"),
        graph.replace("'walk'", "'designed'"),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let animation = bundle.load_animation("base/walker").unwrap();
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
    let cold = bundle.read("base/walker#anim").unwrap();
    assert!(cook().status.success());
    assert_eq!(
        &*og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .read("base/walker#anim")
            .unwrap(),
        &*cold
    );
    // Reject unknown nodes, then prove changing only authored keys invalidates output.
    std::fs::write(
        root.join("content/walker.ts"),
        authored.replace("\"hips\"", "\"absent\""),
    )
    .unwrap();
    assert!(!cook().status.success());
    std::fs::write(
        root.join("content/walker.ts"),
        authored.replace("6,7,9", "10,7,9"),
    )
    .unwrap();
    assert!(cook().status.success());
    let changed = og_cook::runtime::Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_animation("base/walker")
        .unwrap();
    assert!((changed.clip("designed").unwrap().channels[0].lane(2, 0) - 7.0).abs() < 0.001);
    std::fs::write(
        root.join("content/walker.ts"),
        "import {file} from '@origingame/engine/author';export default file.import({kind:'model',source:'assets/walker.glb',conform:'off'});",
    )
    .unwrap();
    std::fs::write(root.join("content/graph.ts"), graph).unwrap();
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
        root.join("content/graph.ts"),
        graph.replace("'walk'", "'hike'"),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        root.join("content/graph.ts"),
        graph.replace(
            "bindings:[{name:'stride',source:ref('base/walker'),clip:'walk'}]",
            "sprite:true",
        ),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        root.join("content/graph.ts"),
        graph.replace("motion:'moving'", "motion:'absent'"),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success(), "invalid graph accepted");
    for bad in [
        graph.replace("threshold:3", "threshold:'wrong'"),
        graph.replace("motion:'plane'", "motion:'blend'"),
        graph.replace("base/walker", "base/absent"),
        graph.replace("mask:{hips:0.25", "mask:{hips:1.25"),
    ] {
        std::fs::write(root.join("content/graph.ts"), bad).unwrap();
        let out = cook();
        assert!(!out.status.success(), "invalid graph accepted");
        assert!(text(&out).contains("content/graph.ts"), "{}", text(&out));
    }
    std::fs::remove_file(root.join("content/graph.ts")).unwrap();
    std::fs::write(
        assets.join("old.ogt"),
        "type animgraph\nversion 1\n---\nsprite true\n",
    )
    .unwrap();
    assert!(!cook().status.success(), "old author graph accepted");
}

/// Public lattice cook/load and emitter consumption retain channel order,
/// float precision and tracked generator dependencies.
#[test]
fn generated_fields_load_losslessly_and_feed_particles() {
    let root = TempDir::new("og-generated-fields");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Field","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'field',entry:'src/main.ts'});").unwrap();
    std::fs::write(root.join("content/source.ts"), "import {file,input} from '@origingame/engine/author'; import {samples} from '../lib/field.ts'; export default file.field({dimensions: [2, 2, 2], channels: 3, origin: [-3, 2, 7], spacing: [0.5, 2, 3], values: samples(input.bytes('assets/input.bin')[0])});").unwrap();
    std::fs::write(assets.join("input.bin"), [7]).unwrap();
    std::fs::write(
        root.join("content/particles.ts"),
        "import {file,ref} from '@origingame/engine/author';export default file.particles({field:{kind:'asset',asset:ref('base/source'),strength:0.25,size:9}});",
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for (amplitude, divisor) in [(7, 32), (11, 32), (11, 16)] {
        std::fs::write(assets.join("input.bin"), [amplitude]).unwrap();
        std::fs::write(root.join("lib/field.ts"), format!("export const samples = (amplitude: number) => Float32Array.from({{length: 24}}, (_, i) => amplitude + i / {divisor});")).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let field = bundle.load_field("base/source").unwrap();
        assert_eq!(field.dimensions, [2, 2, 2]);
        assert_eq!(field.channels, 3);
        assert_eq!(field.origin, [-3.0, 2.0, 7.0]);
        assert_eq!(field.spacing, [0.5, 2.0, 3.0]);
        let expected: Vec<f32> = (0..24)
            .map(|i| f32::from(amplitude) + i as f32 / divisor as f32)
            .collect();
        assert_eq!(field.values, expected);
        let bytes = bundle.read("base/particles").unwrap();
        let emitter = bundle.emitter("base/particles", &bytes).unwrap().data;
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
        let cold_field = bundle.read("base/source").unwrap().to_vec();
        let cold_emitter = bytes.to_vec();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let warm = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        assert_eq!(&*warm.read("base/source").unwrap(), cold_field);
        assert_eq!(&*warm.read("base/particles").unwrap(), cold_emitter);
        std::fs::remove_dir_all(root.join("cooked")).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let restored = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        assert_eq!(&*restored.read("base/source").unwrap(), cold_field);
        assert_eq!(&*restored.read("base/particles").unwrap(), cold_emitter);
    }
    // A rectangular scalar lattice is valid by itself, not a particle grid.
    std::fs::write(root.join("content/source.ts"), "import {file} from '@origingame/engine/author';export default file.field({dimensions: [3, 2, 1], channels: 1, values: [117, -2, 0.125, 4, 9, 0]});").unwrap();
    let out = cook();
    assert!(!out.status.success(), "scalar particle grid accepted");
    assert!(
        text(&out).contains("cubic 3-channel lattice"),
        "{}",
        text(&out)
    );
    std::fs::remove_file(root.join("content/particles.ts")).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.load_field("base/source").unwrap().values,
        [117.0, -2.0, 0.125, 4.0, 9.0, 0.0]
    );
    // The same snapshot reaches terrain writing, navigation and projected
    // scatter. A changed mask must not leave any of those on a cached surface.
    std::fs::write(
        root.join("lib/heights.ts"),
        "export const field = {columns:5,rows:3,heights:Float32Array.from({length:15},(_,i)=>100+i)};",
    )
    .unwrap();
    std::fs::write(root.join("content/surface.ts"), "import {file,ref} from '@origingame/engine/author'; import {field} from '../lib/heights.ts'; export default file.terrain({source:{kind:'heightfield',field},size:[8,4],tile:16,process:{preserve:ref('base/protect'),channels:[{points:[[-4,80,0],[4,72,0]],width:4,depth:2,river:true}]}});").unwrap();
    std::fs::write(
        root.join("content/marker.ts"),
        "import {file} from '@origingame/engine/author';export default file.prefab({entity:{name:'marker'}});",
    )
    .unwrap();
    std::fs::write(root.join("content/placed.ts"), "import {file,ref} from '@origingame/engine/author';export default file.scatter({target:{kind:'prefab',asset:ref('base/marker')},project:ref('base/surface'),positions:[0,3,0]});").unwrap();
    std::fs::write(root.join("content/nav.ts"), "import {file,ref} from '@origingame/engine/author'; export default file.navmesh({terrains:[{terrain:ref('base/surface')}],cellSize:1,cellHeight:0.25,agents:[{radius:0,height:1,climb:2,slope:89}]});").unwrap();
    let mut previous_nav = None;
    for (mask, height) in [(0, 74.0), (1, 107.0)] {
        std::fs::write(root.join("content/protect.ts"), format!("import {{file}} from '@origingame/engine/author';export default file.field({{dimensions:[5,3,1],channels:1,values:Array(15).fill({mask})}});")).unwrap();
        std::fs::write(root.join("content/expected.ts"), format!("import {{file,ref}} from '@origingame/engine/author';export default file.scene({{entities:[{{name:'point-0',prefab:ref('base/marker'),components:{{'og.Transform':{{position:[0,{},0],quaternion:[0,0,0,1],scale:[1,1,1]}}}}}}]}});",height+3.0)).unwrap();
        for _ in 0..2 {
            let out = cook();
            assert!(out.status.success(), "{}", text(&out));
            let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
            let field = bundle
                .load_terrain("base/surface")
                .unwrap()
                .field()
                .unwrap();
            assert!((field.heights[7] - height).abs() < 0.001);
            assert_eq!(
                bundle.load_scene("base/placed").unwrap().components,
                bundle.load_scene("base/expected").unwrap().components
            );
        }
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let nav = bundle.read("base/nav").unwrap().to_vec();
        if let Some(previous) = previous_nav.replace(nav.clone()) {
            assert_ne!(previous, nav, "navigation reused a stale preserve field");
        }
    }
    for invalid in [
        "({dimensions: [3, 2, 1], channels: 1, values: [1, 2]})",
        "({dimensions: [0, 2, 1], channels: 1, values: []})",
        "({dimensions: [1, 1, 1], channels: 1, spacing: [1, 0, 1], values: [1]})",
        "({dimensions: [1, 1, 1], channels: 5, values: [1, 2, 3, 4, 5]})",
    ] {
        std::fs::write(
            root.join("content/source.ts"),
            format!("import {{file}} from '@origingame/engine/author';export default file.field{invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
    std::fs::remove_file(root.join("content/source.ts")).unwrap();
    std::fs::write(
        assets.join("old.ogt"),
        "type field\nversion 1\n---\ndimensions 1 1 1\nchannels 1\nvalues 3\n",
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success(), "removed author field accepted");
    assert!(
        text(&out).contains("OGT authoring is removed"),
        "{}",
        text(&out)
    );
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

/// Checked TS cells use the native map compiler, preserve metadata, and
/// invalidate warm outputs when a tracked helper/input changes.
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
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(root.join("content/tiles.ts"), "import {file} from '@origingame/engine/author';export default file.tileset({tileWidth:8,tileHeight:8,columns:3,rows:1});").unwrap();
    let map = r#"import {file,ref,input} from '@origingame/engine/author';
import {last} from '../lib/grid.ts';
export default file.tilemap({width:3,height:2,tileWidth:8,tileHeight:16,unit:3,
 tilesets:[ref('base/tiles')],layers:[
 {name:'cells',tileset:0,cells:new Uint32Array([input.bytes('assets/cell.bin')[0]+1,0,0x80000003,2,1,last]),z:4,parallax:[0.6,0.8],offset:[1.25,-2.5],opacity:0.75,visible:false,collision:true,colliderLayers:4,colliderMask:3,cellColliders:true,repeatX:true,repeatY:true,lit:false},
 {name:'manual',tileset:0,cells:[3,0,1,2,3,0]}],
 objects:[{name:'门',kind:'door',x:1.5,y:-4.25,width:2.5,height:6,properties:{target:'庭院'}}]});"#;
    std::fs::write(root.join("content/map.ts"), map).unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("lib/grid.ts"), "export const last=3;").unwrap();
    std::fs::write(assets.join("cell.bin"), [0]).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let loaded = || {
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_tilemap("base/map")
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let first = loaded();
    assert_eq!((first.width, first.height), (3, 2));
    assert_eq!(first.tile_size().to_array(), [3.0, 6.0]);
    // Stored cells distinguish tile zero from empty, preserve flips, and
    // catch transposed rows or an accidental second id+1 conversion.
    assert_eq!(first.layers[0].cells, [1, 0, 0x80000003, 2, 1, 3]);
    assert_eq!(first.layers[1].cells, [3, 0, 1, 2, 3, 0]);
    assert_eq!(first.layers[0].z, 4);
    assert_eq!(first.layers[0].parallax.to_array(), [0.6, 0.8]);
    let layer = &first.layers[0];
    assert_eq!(layer.offset.to_array(), [1.25, -2.5]);
    assert_eq!(layer.opacity, 0.75);
    assert!(!layer.visible && !layer.lit);
    assert!(layer.collision && layer.cell_colliders && layer.repeat_x && layer.repeat_y);
    assert_eq!((layer.collider_layers, layer.collider_mask), (4, 3));
    let object = &first.objects[0];
    assert_eq!((&*object.name, &*object.kind), ("门", "door"));
    assert_eq!(
        [object.x, object.y, object.width, object.height],
        [1.5, -4.25, 2.5, 6.0]
    );
    assert_eq!(object.properties, [("target".into(), "庭院".into())]);
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded(), first);
    std::fs::write(assets.join("cell.bin"), [2]).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded().layers[0].cells, [3, 0, 0x80000003, 2, 1, 3]);
    std::fs::write(root.join("lib/grid.ts"), "export const last=2;").unwrap();
    assert!(cook().status.success());
    assert_eq!(loaded().layers[0].cells, [3, 0, 0x80000003, 2, 1, 2]);
    for orientation in ["orthogonal", "isometric", "staggered", "hex", "hexFlat"] {
        std::fs::write(
            root.join("content/map.ts"),
            map.replace("width:3,", &format!("orientation:'{orientation}',width:3,")),
        )
        .unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        assert_eq!(loaded().orientation.name(), orientation);
    }
    for (old, new) in [
        ("cells:[3,0,1,2,3,0]", "cells:[3,0,1]"),
        ("name:'manual'", "name:'cells'"),
        ("tileset:0", "tileset:1"),
        ("0x80000003", "0x20000003"),
        ("width:3,height:2", "width:0,height:2"),
        ("width:3,height:2", "width:4294967295,height:4294967295"),
        ("opacity:0.75", "opacity:1.25"),
        ("collision:true", "collision:false"),
        ("ref('base/tiles')", "ref('base/missing')"),
    ] {
        let source = map.replace(old, new);
        assert_ne!(source, map);
        std::fs::write(root.join("content/map.ts"), &source).unwrap();
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted invalid map {source}: {}",
            text(&out)
        );
        assert!(text(&out).contains("content/map.ts"), "{}", text(&out));
    }
    std::fs::remove_file(root.join("content/map.ts")).unwrap();
    std::fs::write(assets.join("map.ogt"), "type tilemap\nversion 1\n---\n").unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("OGT authoring is removed"),
        "{}",
        text(&out)
    );
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
    std::fs::write(root.join("content/authored.ts"), "import {file} from '@origingame/engine/author';export default file.import({kind:'sound',source:'assets/authored.wav'});").unwrap();
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
    let authored = bundle
        .load_sound("base/authored")
        .unwrap()
        .decode()
        .unwrap();
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
    std::fs::write(root.join("content/sampled.ts"), "import {file} from '@origingame/engine/author'; export default file.sfx({layers:[{sample:{kind:'file',source:'assets/authored.wav',looping:true,reference:440},freq:880,attack:0,sustain:0.01,decay:0,gain:1}]});").unwrap();
    std::fs::write(root.join("content/sample-pcm.ts"), "import {file} from '@origingame/engine/author'; export default file.sfx({layers:[{sample:{kind:'pcm',samples:Float32Array.from({length:300},(_,i)=>[-24575/65536,-8191/65536,16383/65536][i%3]),rate:22050,looping:true,reference:440},freq:880,attack:0,sustain:0.01,decay:0,gain:1}]});").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let decoded = || {
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_sound("base/sampled")
            .unwrap()
            .decode()
            .unwrap()
            .samples
    };
    let first = decoded();
    assert!(first.iter().any(|&v| v != 0));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        first,
        bundle
            .load_sound("base/sample-pcm")
            .unwrap()
            .decode()
            .unwrap()
            .samples
    );
    // Changing only the recording must invalidate the sample-based synth,
    // including an existing shared-cache entry for the unchanged declaration.
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
    std::fs::write(root.join("content/zero.ts"), "import {file} from '@origingame/engine/author'; export default file.sfx({layers:[{sample:{kind:'pcm',samples:[0],rate:8000,looping:true},attack:0,sustain:0.01,decay:0,gain:1}]});").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_ne!(first, decoded());
    let silence = og_cook::runtime::Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_sound("base/zero")
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(decoded(), silence.samples);

    // Distinct canonical IDs let the generated declaration and listening WAV
    // coexist while describing the same sound.
    for out in ["content/preset.ts", "assets/listening-preset.wav"] {
        let generated = og()
            .args([
                "sfx", "laser", "--seed", "23", "--pitch", "1.17", "--length", "0.6", "--volume",
                "0.65", "--out", out,
            ])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(generated.status.success(), "{}", text(&generated));
    }
    std::fs::write(root.join("content/recording.ts"), "import {file} from '@origingame/engine/author';export default file.import({kind:'sound',source:'assets/listening-preset.wav'});").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle
            .load_sound("base/preset")
            .unwrap()
            .decode()
            .unwrap()
            .samples,
        bundle
            .load_sound("base/recording")
            .unwrap()
            .decode()
            .unwrap()
            .samples
    );

    for (source, message) in [
        ("{}", "needs a preset or at least one layer"),
        ("{layers:[{sustain:31}]}", "above the 30 s limit"),
        (
            "{layers:[{sample:{kind:'pcm',samples:[0],rate:8000},fmIndex:1}]}",
            "sample layers reject FM",
        ),
        ("{layers:[{freq:0}]}", "/layers"),
        (
            "{layers:[{sample:{kind:'file',source:'../authored.wav'}}]}",
            "sample.source",
        ),
    ] {
        std::fs::write(root.join("content/bad.ts"), format!("import {{file}} from '@origingame/engine/author'; export default file.sfx({source});")).unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {source}");
        assert!(text(&out).contains(message), "{}", text(&out));
    }
    std::fs::remove_file(root.join("content/bad.ts")).unwrap();
    std::fs::write(
        assets.join("old.ogt"),
        "type sfx\nversion 1\n---\npreset \"click\"\n",
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(
        text(&out).contains("sfx OGT authoring is removed"),
        "{}",
        text(&out)
    );
}

/// Typed source attributes cross the public cook/load boundary, including
/// a mirrored merge's tangent handedness and vertex colours without a map.
#[test]
fn generated_mesh_cooks_with_attributes_and_placed_frames() {
    let root = TempDir::new("og-generated-mesh");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Mesh","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'mesh',entry:'src/main.ts'});").unwrap();
    std::fs::write(
        root.join("content/mesh.ts"),
        r#"import {file} from '@origingame/engine/author';
import mesh from '../lib/mesh.ts';
export default file.model({kind:'geometry',desc:{conform:'off',items:[
 {name:'source',input:{kind:'part',desc:{geometry:{kind:'bufferGeometry',desc:mesh()}}}},
 {name:'placed',input:{kind:'part',desc:{geometry:{kind:'mergeGeometries',desc:{parts:[
   {geometry:{kind:'bufferGeometry',desc:mesh()},transform:{scale:[2,1,3]},mirror:'x'}
 ]}}}}},
 {name:'mixed',input:{kind:'part',desc:{geometry:{kind:'mergeGeometries',desc:{parts:[
   {geometry:{kind:'bufferGeometry',desc:mesh()}},
   {geometry:{kind:'bufferGeometry',desc:{positions:[10,0,0,12,0,0,10,1,0],uvs:[0,1,1,1,0,0]}}}
 ]}}}}}
]}});"#,
    )
    .unwrap();
    let source = "({positions: [-2, 0, 0, 3, 0, 0, 1, 4, 0], colors: [1,0,0, 0,1,0, 0,0,1], uvs: [0,1, 1,1, 0.6,0], tangents: [1,0,0,-1, 1,0,0,-1, 1,0,0,-1], indices: [0,1,2]})";
    std::fs::write(
        root.join("lib/mesh.ts"),
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
    let model = bundle.load_model("base/mesh").unwrap();
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
    std::fs::write(root.join("content/projection.ts"), "import {file} from '@origingame/engine/author';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'panel',input:{kind:'part',desc:{geometry:{kind:'planeGeometry',desc:{width:2,height:2}}}}}],process:{project:{size:32,padding:2,views:[{image:'assets/view.svg',position:[0,0,4],target:[0,0,0],projection:{kind:'orthographic',ortho:1}}]}}}});").unwrap();
    let image = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#e34a2c"/><rect x="24" width="8" height="32" fill="#2c69df"/></svg>"##;
    std::fs::write(assets.join("view.svg"), image).unwrap();
    // The wider variant sorts before its processed base. Transfer retains
    // metric distances rather than stretching the base image over the variant.
    std::fs::write(root.join("content/a-variant.ts"), "import {file,ref} from '@origingame/engine/author';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'panel',input:{kind:'part',desc:{geometry:{kind:'planeGeometry',desc:{width:3,height:2}}}}}],process:{transfer:{source:ref('base/projection'),size:32,padding:2}}}});").unwrap();
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
    let first = cooked_map("base/projection");
    assert_eq!((first.width, first.height), (32, 32));
    let pixel = |rgba: &[u8], x: usize| rgba[(16 * 32 + x) * 4..(16 * 32 + x) * 4 + 3].to_vec();
    let left = pixel(&first.rgba, 8);
    let right = pixel(&first.rgba, 28);
    assert!(left[0] > 180 && left[2] < 90, "{left:?}");
    assert!(right[2] > 180 && right[0] < 90, "{right:?}");
    assert_eq!(cooked_map("base/projection").rgba, first.rgba);
    let variant = cooked_map("base/a-variant");
    let middle = pixel(&variant.rgba, 20);
    assert!(
        middle[2] > 180 && middle[0] < 90,
        "metric transfer must not stretch the red region: {middle:?}"
    );
    assert_eq!(pixel(&variant.rgba, 8), pixel(&first.rgba, 8));
    assert_eq!(cooked_map("base/a-variant").rgba, variant.rgba);
    std::fs::write(assets.join("view.svg"), image.replace("#e34a2c", "#2ce34a")).unwrap();
    let changed = cooked_map("base/projection");
    let left = pixel(&changed.rgba, 8);
    assert!(left[1] > 180 && left[0] < 90, "{left:?}");
    assert_eq!(pixel(&changed.rgba, 28), right);
    let variant = cooked_map("base/a-variant");
    assert_eq!(pixel(&variant.rgba, 8), left);
    let middle = pixel(&variant.rgba, 20);
    assert!(middle[2] > 180 && middle[0] < 90, "{middle:?}");
    // The consumer sorts before its source. Import order is a dependency,
    // not filename order, and a selected node keeps its rest placement.
    let operand = "import {file,ref} from '@origingame/engine/author';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'copy',input:{kind:'part',desc:{geometry:{kind:'mergeGeometries',desc:{parts:[{geometry:{kind:'modelGeometry',desc:{source:ref('base/mesh'),node:'placed'}},transform:{position:[0,2,0]}}]}}}}}]}});";
    std::fs::write(root.join("content/a-copy.ts"), operand).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let copy = bundle.load_model("base/a-copy").unwrap();
    let p = &copy.primitives[0];
    assert_eq!((p.min.x, p.max.x, p.min.y, p.max.y), (-6.0, 4.0, 2.0, 6.0));
    for (v, t) in p.vertices.iter().zip(&p.tangents) {
        assert!((t[0] + 1.0).abs() < 0.001);
        assert_eq!(t[3], 1.0);
        assert_eq!(v.color.iter().sum::<f32>(), 2.0);
    }
    std::fs::write(
        root.join("lib/mesh.ts"),
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
        bundle.load_model("base/a-copy").unwrap().primitives[0]
            .max
            .x,
        8.0
    );
    std::fs::write(
        root.join("content/mesh.ts"),
        operand
            .replace("base/mesh", "base/a-copy")
            .replace(",node:'placed'", ""),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(text(&out).contains("dependency cycle"), "{}", text(&out));
    std::fs::remove_file(root.join("content/a-copy.ts")).unwrap();
    std::fs::write(root.join("content/mesh.ts"), "import {file} from '@origingame/engine/author';import mesh from '../lib/mesh.ts';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'source',input:{kind:'part',desc:{geometry:{kind:'bufferGeometry',desc:mesh()}}}}]}});").unwrap();
    for invalid in [
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,3]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], colors: [1,0,0]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], tangents: [1,0,0,-1]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,1.5]})",
    ] {
        std::fs::write(
            root.join("lib/mesh.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}

/// Computed and literal TS declarations share native scene/prefab packing,
/// including captured input invalidation and sub-scene cycle refusal.
#[test]
fn generated_scene_cooks_as_authored_declarations() {
    let root = TempDir::new("og-generated-scene");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("og-project.json"),
        r#"{"version":1,"name":"Scene","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::create_dir_all(root.join("src/schema")).unwrap();
    std::fs::write(root.join("src/schema/records.ts"), "import * as s from '@origingame/engine/schema';export const Stats=s.defineComponent('Stats',{weight:s.f32()});export const Run=s.defineResource('Run',{count:s.u32(),enabled:s.bool(),weights:s.list(s.f32(),4)});").unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'scene',entry:'src/main.ts',schemas:['src/schema/records.ts']});").unwrap();
    std::fs::write(
        root.join("content/sub.ts"),
        "import {file} from '@origingame/engine/author';export default file.scene({entities:[{name:'nested'}]});",
    )
    .unwrap();
    std::fs::write(
        root.join("content/model.ts"),
        "import {file} from '@origingame/engine/author';export default file.model({kind:'geometry',desc:{conform:'off',items:[{name:'part',input:{kind:'part',desc:{geometry:{kind:'boxGeometry',desc:{}}}}}]}});",
    )
    .unwrap();
    std::fs::write(root.join("content/base.ts"), "import {file} from '@origingame/engine/author';export default file.prefab({entity:{name:'base',components:{Stats:{weight:2}},children:[{name:'prefab-child',components:{Stats:{weight:4}}}]}});").unwrap();
    let declaration = |weight: &str| {
        format!(
            "import {{file,ref,input}} from '@origingame/engine/author';export default file.scene({{name:'Generated',entities:[{{name:'manual',components:{{Stats:{{weight:1}}}}}},{{name:'generated',prefab:ref('base/base'),components:{{Stats:{{weight:{weight}}}}},children:[{{name:'instance',model:ref('base/model'),static:true}},{{name:'cell',scene:ref('base/sub')}}]}}],resources:{{Run:{{count:3,enabled:true,weights:[2,7,3]}}}}}});"
        )
    };
    std::fs::write(
        root.join("content/made.ts"),
        declaration("input.bytes('assets/weight.bin')[0]"),
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for weight in [7, 9] {
        std::fs::write(assets.join("weight.bin"), [weight]).unwrap();
        std::fs::write(
            root.join("content/authored.ts"),
            declaration(&weight.to_string()),
        )
        .unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let made = bundle.load_scene("base/made").unwrap();
        let authored = bundle.load_scene("base/authored").unwrap();
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
        "({entities: [{name: 'bad', model: ref('base/missing')}]})",
        "({entities: [{name: 'bad', scene: ref('base/made')}]})",
        "({entities: [{name: 'bad', components: {Stats: {weight: {nested: 1}}}}]})",
        "({entities: [{name: 'bad', components: {entity: {}}}]})",
        "({entities: [{name: 'bad', unexpected: true}]})",
    ] {
        std::fs::write(
            root.join("content/made.ts"),
            format!("import {{file,ref}} from '@origingame/engine/author';export default file.scene{invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}
