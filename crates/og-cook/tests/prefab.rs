//! Prefabs (`docs/design.md` §2.5, G2-1): a scene's instances flattened by
//! the cook, overrides kept across a prefab edit and a re-cook, variants
//! chained, cycles and name clashes refused, orphaned overrides noted in a
//! dev cook and refused in a publish's, and every placed entity's origin
//! in the blob.

use og_cook::cook::{CookOptions, Report, cook_project};
use og_cook::runtime::{Bundle, PrefabOrigin, SceneDesc};
use og_core::reflect::{FieldSchema, FieldType, Schema};
use std::path::{Path, PathBuf};

fn schemas() -> Vec<Schema> {
    let schema = |name: &str, fields: Vec<FieldSchema>| Schema {
        name: name.into(),
        version: 1,
        doc: String::new(),
        fields,
    };
    vec![
        schema(
            "Transform",
            vec![
                FieldSchema::new("position", FieldType::Vec3),
                FieldSchema::new("scale", FieldType::Vec3),
            ],
        ),
        schema(
            "Health",
            vec![
                FieldSchema::new("hp", FieldType::F32),
                FieldSchema::new("armor", FieldType::F32),
            ],
        ),
        schema("Speed", vec![FieldSchema::new("value", FieldType::F32)]),
    ]
}

/// A fresh project directory named for the test.
fn project(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("og-cook-prefab-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(root.join("assets/prefabs")).unwrap();
    root
}

fn write(root: &Path, id: &str, text: &str) {
    std::fs::write(root.join("assets").join(id), text).unwrap();
}

fn cook_with(root: &Path, publish: bool) -> Report {
    let options = CookOptions {
        threads: 1,
        schemas: schemas(),
        publish,
        ..CookOptions::default()
    };
    cook_project(root, &options).expect("the cook runs")
}

fn load(root: &Path, id: &str) -> SceneDesc {
    Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene(id)
        .unwrap()
}

/// `component.field` of the entity named `entity`, as f32 lanes.
fn field(scene: &SceneDesc, entity: &str, component: &str, field: &str) -> Option<Vec<f32>> {
    let e = scene.entities.iter().position(|x| x.name == entity)?;
    let table = scene.components.iter().find(|t| t.name == component)?;
    let row = table.get(e)?;
    let (i, _) = table
        .schema
        .fields
        .iter()
        .enumerate()
        .find(|(_, f)| f.name == field)?;
    let at = og_core::Layout::of(&table.schema).range(i).start;
    let n = table.schema.fields[i].ty.lanes();
    Some(
        (0..n)
            .map(|k| f32::from_le_bytes(row[at + 4 * k..at + 4 * k + 4].try_into().unwrap()))
            .collect(),
    )
}

fn names(scene: &SceneDesc) -> Vec<&str> {
    scene.entities.iter().map(|e| e.name.as_str()).collect()
}

const CRATE: &str = "type prefab
version 1
---
entity \"crate\" {
  Transform {
    scale 1 1 1
  }
  entity \"lid\" {
    Health {
      hp 10
    }
    entity \"hinge\" {
      Speed {
        value 1
      }
    }
  }
}
";

const YARD: &str = "type scene
version 1
---
name \"Yard\"
entity \"crate1\" {
  prefab \"prefabs/crate.ogt\"
  Transform {
    position 4 0 2
  }
  override \"lid\" {
    Health {
      hp 40
    }
  }
}
entity \"crate2\" {
  prefab \"prefabs/crate.ogt\"
}
entity \"post\" {
  Transform {
    position 1 0 0
  }
}
";

#[test]
fn an_override_survives_a_prefab_edit_and_a_re_cook() {
    let root = project("edit");
    write(&root, "prefabs/crate.ogt", CRATE);
    write(&root, "scenes/yard.ogt", YARD);
    let report = cook_with(&root, false);
    assert!(report.ok(), "{:?}", report.errors());
    let first = load(&root, "scenes/yard.ogt");
    assert_eq!(
        names(&first),
        [
            "crate1",
            "crate1/lid",
            "crate1/lid/hinge",
            "crate2",
            "crate2/lid",
            "crate2/lid/hinge",
            "post"
        ]
    );
    assert_eq!(
        field(&first, "crate1/lid", "Health", "hp"),
        Some(vec![40.0])
    );
    assert_eq!(
        field(&first, "crate2/lid", "Health", "hp"),
        Some(vec![10.0])
    );
    // The root's own block overrides one field; the other follows the prefab.
    assert_eq!(
        field(&first, "crate1", "Transform", "position"),
        Some(vec![4.0, 0.0, 2.0])
    );
    assert_eq!(
        field(&first, "crate1", "Transform", "scale"),
        Some(vec![1.0, 1.0, 1.0])
    );

    // The prefab changes: hp, a new field, the scale. Only the prefab's
    // text moved, and the scene re-cooks.
    let edited = CRATE
        .replace("hp 10", "hp 20\n      armor 5")
        .replace("scale 1 1 1", "scale 2 2 2");
    write(&root, "prefabs/crate.ogt", &edited);
    let report = cook_with(&root, false);
    assert!(report.ok(), "{:?}", report.errors());
    let yard = report
        .assets
        .iter()
        .find(|a| a.source == "scenes/yard.ogt")
        .unwrap();
    assert!(
        yard.outputs[0].cooked,
        "editing a prefab re-cooks the scene"
    );
    let second = load(&root, "scenes/yard.ogt");
    // The override kept its value; everything it does not override follows.
    assert_eq!(
        field(&second, "crate1/lid", "Health", "hp"),
        Some(vec![40.0])
    );
    assert_eq!(
        field(&second, "crate1/lid", "Health", "armor"),
        Some(vec![5.0])
    );
    assert_eq!(
        field(&second, "crate2/lid", "Health", "hp"),
        Some(vec![20.0])
    );
    assert_eq!(
        field(&second, "crate1", "Transform", "scale"),
        Some(vec![2.0, 2.0, 2.0])
    );
    assert_eq!(
        field(&second, "crate1", "Transform", "position"),
        Some(vec![4.0, 0.0, 2.0])
    );

    // Nothing changed: the same blob again, byte for byte.
    write(&root, "scenes/yard.ogt", &format!("{YARD}# touched\n"));
    assert!(cook_with(&root, false).ok());
    assert_eq!(load(&root, "scenes/yard.ogt"), second);
}

#[test]
fn every_placed_entity_records_its_prefab_node_and_root() {
    let root = project("links");
    write(&root, "prefabs/crate.ogt", CRATE);
    // A prefab placing another: its entities link to the outermost.
    write(
        &root,
        "prefabs/stack.ogt",
        "type prefab
version 1
---
entity \"stack\" {
  entity \"bottom\" {
    prefab \"prefabs/crate.ogt\"
  }
}
",
    );
    write(
        &root,
        "scenes/yard.ogt",
        "type scene
version 1
---
entity \"post\" {
}
entity \"s\" {
  prefab \"prefabs/stack.ogt\"
  remove \"bottom/lid/hinge\"
  entity \"flag\" {
    Speed {
      value 3
    }
  }
}
",
    );
    let report = cook_with(&root, false);
    assert!(report.ok(), "{:?}", report.errors());
    let yard = load(&root, "scenes/yard.ogt");
    assert_eq!(
        names(&yard),
        ["post", "s", "s/bottom", "s/bottom/lid", "flag"]
    );
    assert_eq!(yard.prefabs, ["prefabs/stack.ogt"]);
    let origin = |i: usize| yard.entities[i].prefab;
    assert_eq!(origin(0), None, "an authored entity");
    // The stack's table: stack 0, bottom 1, bottom/lid 2, bottom/lid/hinge 3.
    for (i, node) in [(1, 0), (2, 1), (3, 2)] {
        assert_eq!(
            origin(i),
            Some(PrefabOrigin {
                prefab: 0,
                node,
                root: 1
            })
        );
    }
    assert_eq!(origin(4), None, "an entity the instance added");
    // The prefab's own blob: every entity linked to it, from its root.
    let stack = load(&root, "prefabs/stack.ogt");
    assert_eq!(names(&stack), ["stack", "bottom", "lid", "hinge"]);
    assert_eq!(stack.prefabs, ["prefabs/stack.ogt"]);
    assert!(stack.entities.iter().enumerate().all(|(i, e)| e.prefab
        == Some(PrefabOrigin {
            prefab: 0,
            node: i as u32,
            root: 0
        })));
    // A scene without prefabs has no table.
    write(
        &root,
        "scenes/plain.ogt",
        "type scene\nversion 1\n---\nentity \"a\" {\n}\n",
    );
    assert!(cook_with(&root, false).ok());
    let plain = load(&root, "scenes/plain.ogt");
    assert!(plain.prefabs.is_empty());
    assert!(plain.entities.iter().all(|e| e.prefab.is_none()));
}

#[test]
fn a_variant_chain_applies_each_base_then_its_own() {
    let root = project("variants");
    write(
        &root,
        "prefabs/goblin.ogt",
        "type prefab
version 1
---
entity \"goblin\" {
  Health {
    hp 10
    armor 1
  }
  Speed {
    value 1
  }
  entity \"club\" {
    Speed {
      value 1
    }
  }
}
",
    );
    write(
        &root,
        "prefabs/chief.ogt",
        "type prefab
version 1
---
base \"prefabs/goblin.ogt\"
entity \"chief\" {
  Health {
    hp 30
  }
  override \"club\" {
    Speed {
      value 2
    }
  }
}
",
    );
    write(
        &root,
        "prefabs/king.ogt",
        "type prefab
version 1
---
base \"prefabs/chief.ogt\"
entity \"king\" {
  Speed {
    value 3
  }
  entity \"crown\" {
    Health {
      hp 1
    }
  }
}
",
    );
    write(
        &root,
        "scenes/hall.ogt",
        "type scene
version 1
---
entity \"boss\" {
  prefab \"prefabs/king.ogt\"
  Health {
    armor 9
  }
}
",
    );
    let report = cook_with(&root, false);
    assert!(report.ok(), "{:?}", report.errors());
    let hall = load(&root, "scenes/hall.ogt");
    assert_eq!(names(&hall), ["boss", "boss/club", "boss/crown"]);
    // goblin's armor 1 → the instance's 9; goblin's hp 10 → chief's 30.
    assert_eq!(field(&hall, "boss", "Health", "hp"), Some(vec![30.0]));
    assert_eq!(field(&hall, "boss", "Health", "armor"), Some(vec![9.0]));
    // goblin's speed 1 → king's 3; the club, goblin's 1 → chief's 2.
    assert_eq!(field(&hall, "boss", "Speed", "value"), Some(vec![3.0]));
    assert_eq!(field(&hall, "boss/club", "Speed", "value"), Some(vec![2.0]));
    assert_eq!(field(&hall, "boss/crown", "Health", "hp"), Some(vec![1.0]));
    // The variant's own blob keeps its root's name.
    let king = load(&root, "prefabs/king.ogt");
    assert_eq!(names(&king), ["king", "club", "crown"]);
    // Editing the base re-cooks the whole chain's users.
    write(
        &root,
        "prefabs/goblin.ogt",
        &std::fs::read_to_string(root.join("assets/prefabs/goblin.ogt"))
            .unwrap()
            .replace("armor 1", "armor 2")
            .replace("  entity \"club\" {\n    Speed {\n      value 1\n    }\n  }\n", "  entity \"club\" {\n    Speed {\n      value 1\n    }\n  }\n  entity \"bag\" {\n  }\n"),
    );
    assert!(cook_with(&root, false).ok());
    let hall = load(&root, "scenes/hall.ogt");
    assert_eq!(
        names(&hall),
        ["boss", "boss/club", "boss/bag", "boss/crown"]
    );
    assert_eq!(load(&root, "prefabs/king.ogt").entities.len(), 4);
}

#[test]
fn a_cycle_is_refused_naming_the_chain() {
    let root = project("cycle");
    write(
        &root,
        "prefabs/a.ogt",
        "type prefab\nversion 1\n---\nentity \"a\" {\n  entity \"b\" {\n    prefab \"prefabs/b.ogt\"\n  }\n}\n",
    );
    write(
        &root,
        "prefabs/b.ogt",
        "type prefab\nversion 1\n---\nbase \"prefabs/c.ogt\"\nentity \"b\" {\n}\n",
    );
    write(
        &root,
        "prefabs/c.ogt",
        "type prefab\nversion 1\n---\nentity \"c\" {\n  entity \"x\" {\n    prefab \"prefabs/a.ogt\"\n  }\n}\n",
    );
    write(
        &root,
        "scenes/s.ogt",
        "type scene\nversion 1\n---\nentity \"one\" {\n  prefab \"prefabs/a.ogt\"\n}\n",
    );
    let report = cook_with(&root, false);
    let errors = report.errors().join("\n");
    assert!(
        errors.contains(
            "prefab cycle: prefabs/a.ogt → prefabs/b.ogt → prefabs/c.ogt → prefabs/a.ogt"
        ),
        "{errors}"
    );
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.contains("scenes/s.ogt") && e.contains("prefab cycle")),
        "{errors}"
    );
}

#[test]
fn an_orphaned_override_is_noted_in_dev_and_refused_by_a_publish() {
    let root = project("orphan");
    write(&root, "prefabs/crate.ogt", CRATE);
    write(
        &root,
        "scenes/yard.ogt",
        "type scene
version 1
---
entity \"crate1\" {
  prefab \"prefabs/crate.ogt\"
  override \"lid\" {
    Health {
      hp 40
      shield 3
    }
  }
  override \"top\" {
    Health {
      hp 1
    }
  }
  remove \"lid/sticker\"
}
",
    );
    let report = cook_with(&root, false);
    assert!(report.ok(), "{:?}", report.errors());
    let yard = report
        .assets
        .iter()
        .find(|a| a.source == "scenes/yard.ogt")
        .unwrap();
    let notes = yard.notes.join("\n");
    for want in [
        "instance 'crate1': Health.shield of node 'lid' is not in the prefab",
        "instance 'crate1': node 'top' is not in the prefab",
        "instance 'crate1': node 'lid/sticker' is not in the prefab",
    ] {
        assert!(notes.contains(want), "{want} in {notes}");
    }
    assert_eq!(
        report
            .warnings
            .iter()
            .filter(|w| w.contains("orphaned override"))
            .count(),
        3
    );
    // What still applies, applies.
    let scene = load(&root, "scenes/yard.ogt");
    assert_eq!(
        field(&scene, "crate1/lid", "Health", "hp"),
        Some(vec![40.0])
    );
    // A publish never ships it.
    let report = cook_with(&root, true);
    let errors = report.errors().join("\n");
    assert!(!report.ok());
    assert!(
        errors.contains("a published game never ships an override that does nothing")
            && errors.contains("Health.shield of node 'lid'")
            && errors.contains("instance 'crate1'"),
        "{errors}"
    );
}

#[test]
fn a_generated_name_taken_by_another_entity_is_refused_naming_both() {
    let root = project("names");
    write(&root, "prefabs/crate.ogt", CRATE);
    write(
        &root,
        "scenes/yard.ogt",
        "type scene
version 1
---
entity \"crate1/lid\" {
}
entity \"crate1\" {
  prefab \"prefabs/crate.ogt\"
}
",
    );
    let errors = cook_with(&root, false).errors().join("\n");
    assert!(
        errors.contains(
            "'crate1/lid' (from instance 'crate1' of prefabs/crate.ogt) collides with the hand-written entity 'crate1/lid'"
        ),
        "{errors}"
    );
    // Two siblings of one name in a prefab cannot be addressed.
    write(
        &root,
        "prefabs/twins.ogt",
        "type prefab\nversion 1\n---\nentity \"t\" {\n  entity \"a\" {\n  }\n  entity \"a\" {\n  }\n}\n",
    );
    let errors = cook_with(&root, false).errors().join("\n");
    assert!(
        errors.contains("two children of the root are named 'a'"),
        "{errors}"
    );
}
