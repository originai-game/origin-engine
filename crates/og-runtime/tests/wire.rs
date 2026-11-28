//! The typed wire readers (src/wire.rs over the generated types): what the
//! IDL refuses, the engine refuses.

#[path = "../../../tests/temp.rs"]
mod temp;

/// Tune discovers checked targets even for unnamed prefab instances; runtime
/// ids and overridden instance names are not author identities.
#[test]
fn tune_discovers_authored_targets_from_live_scene_placements() {
    use og_runtime::*;
    use serde_json::{Value, json};
    use std::rc::Rc;
    struct Files;
    impl Hooks for Files {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
            match std::fs::read(path) {
                Ok(bytes) => Fetch::Ready(bytes),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
    }
    let root = temp::TempDir::new("tune-authored-targets");
    crate::init_project(&root);
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(root.join("content/rig.ts"), "import {file} from '@origingame/engine/author';export default file.prefab({entity:{name:'rig',children:[{name:'lens',components:{'og.OrbitCamera':{distance:13}}}]}});").unwrap();
    std::fs::write(root.join("content/room.ts"), "import {file,ref} from '@origingame/engine/author';export default file.scene({entities:[{name:'placed',prefab:ref('base/rig'),edits:[{kind:'override',path:'lens',components:{'og.OrbitCamera':{azimuth:0.7}}}]}]});").unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Files));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed":1}).to_string()));
    let created: Value = serde_json::from_str(&og_engine_create(
        game,
        json!({"root":root.to_str().unwrap()}).to_string(),
    ))
    .unwrap();
    assert_eq!(created["ok"], true, "{created}");
    let engine = created["value"].as_u64().unwrap() as u32;
    let scene = og_load_scene(engine, "base/room".into())
        .unwrap()
        .as_f64()
        .unwrap();
    og_world_instantiate(engine, scene, None).unwrap();
    let bounds: Value =
        serde_json::from_str(&og_runtime_probe_bounds(engine, "placed/lens".into())).unwrap();
    for field in ["distance", "azimuth"] {
        assert_eq!(
            bounds[format!("og.OrbitCamera.{field}")]["authoredTarget"],
            json!({"scene":"base/room","entity":"placed/placed/lens","component":"og.OrbitCamera","field":field})
        );
    }
    assert!(
        bounds["og.OrbitCamera.polar"]
            .get("authoredTarget")
            .is_none()
    );
    let prefab = og_load_prefab(engine, "base/rig".into())
        .unwrap()
        .as_f64()
        .unwrap();
    for _ in 0..2 {
        let placed = og_world_instantiate(engine, prefab, None)
            .unwrap()
            .as_f64()
            .unwrap();
        let rig = og_world_children(engine, placed)[0].as_f64().unwrap();
        let lens = og_world_children(engine, rig)[0].as_f64().unwrap();
        let entity = og_ecs::Entity::from_bits(lens as u64);
        let id = format!("#{}.{}", entity.index(), entity.generation());
        let bounds: Value = serde_json::from_str(&og_runtime_probe_bounds(engine, id)).unwrap();
        assert_eq!(
            bounds["og.OrbitCamera.distance"]["authoredTarget"],
            json!({"scene":"base/rig","entity":"rig/lens","component":"og.OrbitCamera","field":"distance"})
        );
        assert!(
            bounds["og.OrbitCamera.azimuth"]
                .get("authoredTarget")
                .is_none()
        );
    }
    // A source site is development data, not a second authoring path in a
    // publish blob. Even a development probe cannot recover it after refresh.
    let published = og_cook::cook_project(
        &root,
        &og_cook::CookOptions {
            publish: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(published.ok(), "{:?}", published.errors());
    let refreshed = dev_refresh(engine).unwrap();
    assert_eq!(refreshed["warnings"], json!([]));
    let bounds: Value =
        serde_json::from_str(&og_runtime_probe_bounds(engine, "placed/lens".into())).unwrap();
    assert!(bounds.as_object().is_some_and(|rows| !rows.is_empty()));
    assert!(
        bounds
            .as_object()
            .unwrap()
            .values()
            .all(|row| row.get("authoredTarget").is_none())
    );
    assert!(og_engine_destroy(engine));
}

/// A delivery dependency is available before its parent's completion, cannot
/// be released under that parent, and cannot shadow an already-open identity.
#[test]
fn delivery_dependencies_are_pinned_and_asset_identities_are_global() {
    use og_runtime::*;
    use serde_json::{Value, json};
    use std::{cell::Cell, rc::Rc};
    struct Files(Rc<Cell<bool>>);
    impl Hooks for Files {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
            if self.0.get() && path.ends_with("delivery/shared/.cook.ogt") {
                return Fetch::Pending;
            }
            match std::fs::read(path) {
                Ok(bytes) => Fetch::Ready(bytes),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
    }
    let root = temp::TempDir::new("runtime-delivery");
    crate::init_project(&root);
    std::fs::create_dir_all(root.join("content")).unwrap();
    std::fs::write(root.join("project.ts"), "import {defineProject,ref} from '@origingame/engine/author';export default defineProject({id:'delivery',entry:'src/main.ts',packs:[{id:'shared',include:['base/shared'],roots:[ref('base/shared')]},{id:'north',include:['base/north'],dependsOn:['shared'],roots:[ref('base/north')]}]});").unwrap();
    for name in ["shared", "north"] {
        std::fs::write(root.join(format!("content/{name}.ts")), format!("import {{file}} from '@origingame/engine/author';export default file.scene({{entities:[{{name:'{name}',components:{{'og.Transform':{{position:[2,3,0],quaternion:[0,0,0,1],scale:[1,1,1]}}}}}}]}});")).unwrap();
    }
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let delayed = Rc::new(Cell::new(true));
    set_hooks(Rc::new(Files(delayed.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed":1}).to_string()));
    let created: Value = serde_json::from_str(&og_engine_create(
        game,
        json!({"root":root.to_str().unwrap()}).to_string(),
    ))
    .unwrap();
    assert_eq!(created["ok"], true, "{created}");
    let engine = created["value"].as_u64().unwrap() as u32;
    let north = og_load_bundle(engine, "cooked/delivery/north".into())
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(og_load_status(engine, north), "pending");
    delayed.set(false);
    let frame = og_engine_advance(engine, 1);
    assert!(frame.contains("\"ok\":true"), "{frame}");
    assert_eq!(og_load_status(engine, north), "ready");
    let scene = og_load_scene(engine, "base/shared".into())
        .unwrap()
        .as_f64()
        .unwrap();
    let placed = og_world_instantiate(engine, scene, None)
        .unwrap()
        .as_f64()
        .unwrap();
    let entity = og_world_children(engine, placed)[0].as_f64().unwrap();
    // Runtime state on an unedited field must survive the source refresh.
    og_world_insert(
        engine,
        entity,
        "og.Transform".into(),
        Some(json!({"scale":{"x":4,"y":5,"z":6}}).to_string()),
    )
    .unwrap();
    std::fs::write(root.join("content/shared.ts"), "import {file} from '@origingame/engine/author';export default file.scene({entities:[{name:'shared',components:{'og.Transform':{position:[7,4,1],quaternion:[0,0,0,1],scale:[1,1,1]}}}]});").unwrap();
    let updated = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(updated.ok(), "{:?}", updated.errors());
    let refreshed = dev_refresh(engine).unwrap();
    assert_eq!(refreshed["warnings"], json!([]), "{refreshed}");
    assert_eq!(og_load_status(engine, north), "ready");
    let transform = og_world_get(engine, entity, "og.Transform".into()).unwrap();
    assert_eq!(transform["position"], json!({"x":7.0,"y":4.0,"z":1.0}));
    assert_eq!(transform["scale"], json!({"x":4.0,"y":5.0,"z":6.0}));
    // The updated cooked scene, not a retained edit, is used by later spawns.
    og_world_despawn(engine, placed).unwrap();
    let later = og_world_instantiate(engine, scene, None)
        .unwrap()
        .as_f64()
        .unwrap();
    let later_entity = og_world_children(engine, later)[0].as_f64().unwrap();
    let later_transform = og_world_get(engine, later_entity, "og.Transform".into()).unwrap();
    assert_eq!(
        later_transform["position"],
        json!({"x":7.0,"y":4.0,"z":1.0})
    );
    assert_eq!(later_transform["scale"], json!({"x":1.0,"y":1.0,"z":1.0}));
    let shared = og_load_bundle(engine, "cooked/delivery/shared".into())
        .unwrap()
        .as_f64()
        .unwrap();
    og_load_release(engine, shared).unwrap();
    assert!(
        og_load_error(engine, shared)
            .unwrap()
            .contains("still requires")
    );
    // A failed release keeps the original manifest alive; another bundle
    // cannot take over its IDs even when the bytes are identical.
    let duplicate = root.join("duplicate");
    std::fs::create_dir_all(&duplicate).unwrap();
    std::fs::copy(
        root.join("cooked/delivery/north/.cook.ogt"),
        duplicate.join(".cook.ogt"),
    )
    .unwrap();
    let other = og_load_bundle(engine, duplicate.to_str().unwrap().into())
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        og_load_error(engine, other)
            .unwrap()
            .contains("duplicate global asset identity")
    );
    assert!(og_engine_destroy(engine));
}

#[test]
fn a_system_desc_the_idl_refuses_refuses_the_engine() {
    use og_runtime::{og_engine_create, og_game_game, og_host_begin_bundle};
    og_host_begin_bundle();
    // Read by hand, a `when` the IDL does not name ran as `during`.
    og_runtime::og_system_define_system("late".into(), r#"{"when":"after"}"#.into(), 0);
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let answer = og_engine_create(game, r#"{}"#.into());
    assert!(
        answer.contains("og.system 'late': desc.when: 'after' is not a StateWhen"),
        "{answer}"
    );
}
