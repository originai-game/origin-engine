//! Sub-scenes (G3-3): an entity's `scene "<id>"` line places another scene,
//! which stays its own blob and is named in the placing scene's
//! `subscenes` table; a placement that reaches the placing scene again is
//! refused naming the chain, and so is one of a document that is not a
//! scene.

use og_cook::cook::{CookOptions, Report, cook_project};
use og_cook::runtime::Bundle;
use std::path::{Path, PathBuf};

fn project(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("og-cook-subscene-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(root.join("assets/prefabs")).unwrap();
    root
}

fn write(root: &Path, id: &str, text: &str) {
    std::fs::write(root.join("assets").join(id), text).unwrap();
}

fn cook(root: &Path) -> Report {
    let options = CookOptions {
        threads: 1,
        ..CookOptions::default()
    };
    cook_project(root, &options).expect("the cook runs")
}

fn scene(body: &str) -> String {
    format!("type scene\nversion 1\n---\n{body}")
}

#[test]
fn a_placed_sub_scene_is_its_own_blob_named_by_the_entity_that_places_it() {
    let root = project("placed");
    write(
        &root,
        "scenes/world.ogt",
        &scene(
            "entity \"village\" {\n  scene \"scenes/village.ogt\"\n  StreamCell {\n    size 20 10 20\n    load 10\n  }\n}\nentity \"plain\" {\n}\n",
        ),
    );
    write(
        &root,
        "scenes/village.ogt",
        &scene("entity \"well\" {\n  Transform {\n    position 1 0 2\n  }\n}\n"),
    );
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let world = bundle.load_scene("scenes/world.ogt").unwrap();
    let subs: Vec<(&str, Option<&str>)> = world
        .entities
        .iter()
        .map(|e| (e.name.as_str(), e.subscene.as_deref()))
        .collect();
    assert_eq!(
        subs,
        [("village", Some("scenes/village.ogt")), ("plain", None)]
    );
    // Not flattened: the village's entities are in its own blob only.
    assert_eq!(world.entities.len(), 2);
    let village = bundle.load_scene("scenes/village.ogt").unwrap();
    assert_eq!(village.entities[0].name, "well");
}

#[test]
fn a_sub_scene_that_reaches_its_placer_again_is_refused_naming_the_chain() {
    let root = project("cycle");
    write(
        &root,
        "scenes/a.ogt",
        &scene("entity \"x\" {\n  scene \"scenes/b.ogt\"\n}\n"),
    );
    write(
        &root,
        "scenes/b.ogt",
        &scene("entity \"y\" {\n  entity \"z\" {\n    scene \"scenes/a.ogt\"\n  }\n}\n"),
    );
    let report = cook(&root);
    let errors = format!("{:?}", report.errors());
    assert!(
        errors.contains("scenes/a.ogt -> scenes/b.ogt -> scenes/a.ogt"),
        "{errors}"
    );
    assert!(
        errors.contains("scenes/b.ogt -> scenes/a.ogt -> scenes/b.ogt"),
        "{errors}"
    );
}

#[test]
fn a_sub_scene_line_naming_no_scene_is_refused() {
    let root = project("not-scene");
    write(
        &root,
        "prefabs/crate.ogt",
        "type prefab\nversion 1\n---\nentity \"crate\" {\n}\n",
    );
    write(
        &root,
        "scenes/a.ogt",
        &scene(
            "entity \"x\" {\n  scene \"prefabs/crate.ogt\"\n}\nentity \"y\" {\n  scene \"scenes/missing.ogt\"\n}\n",
        ),
    );
    let errors = format!("{:?}", cook(&root).errors());
    assert!(
        errors.contains("scene 'prefabs/crate.ogt' is not a scene of this project"),
        "{errors}"
    );
}
