//! The typed wire readers (src/wire.rs over the generated types): what the
//! IDL refuses, the engine refuses.

#[path = "../../../tests/temp.rs"]
mod temp;

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
        std::fs::write(root.join(format!("content/{name}.ts")), format!("import {{file}} from '@origingame/engine/author';export default file.scene({{entities:[{{name:'{name}'}}]}});")).unwrap();
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
    assert!(og_load_scene(engine, "base/shared".into()).is_ok());
    std::fs::write(root.join("content/shared.ts"), "import {file} from '@origingame/engine/author';export default file.scene({entities:[{name:'updated-shared'}]});").unwrap();
    let updated = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(updated.ok(), "{:?}", updated.errors());
    let refreshed = dev_refresh(engine).unwrap();
    assert_eq!(refreshed["warnings"], json!([]), "{refreshed}");
    assert_eq!(og_load_status(engine, north), "ready");
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
