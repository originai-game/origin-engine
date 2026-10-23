//! Scene write-back round trip (`save_scene`, `og_cook::scene_text`): a
//! scene is cooked and loaded, its values are changed, the text is
//! rewritten from them and cooked again, and the second load is exactly the
//! changed world: every row byte for byte, nothing else moved, and the
//! lines that were not about a changed value (comments, order, the model
//! line) are the same text.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::{Bundle, SceneDesc};
use og_cook::scene_text::{FieldValue, rewrite};
use og_core::reflect::{FieldSchema, FieldType, Schema};

const TEXT: &str = "type scene
version 1
---
# A level for the write-back test.
name \"Yard\"
entity \"crate\" {
  Transform {
    position -1 0.5 0 # dropped from here
    scale 1 1 1
  }
  Switch {
    on false
  }
}
# The second one.
entity \"post\" {
  Transform {
    position 2 0 0.1
  }
}
";

fn schemas() -> Vec<Schema> {
    vec![
        Schema {
            name: "Transform".into(),
            version: 1,
            doc: String::new(),
            fields: vec![
                FieldSchema::new("position", FieldType::Vec3),
                FieldSchema::new("scale", FieldType::Vec3),
            ],
        },
        Schema {
            name: "Switch".into(),
            version: 1,
            doc: String::new(),
            fields: vec![FieldSchema::new("on", FieldType::Bool)],
        },
    ]
}

fn cook(root: &std::path::Path, text: &str) -> SceneDesc {
    std::fs::write(root.join("assets/scenes/yard.ogt"), text).unwrap();
    let options = CookOptions {
        threads: 1,
        schemas: schemas(),
        ..CookOptions::default()
    };
    cook_project(root, &options).expect("the scene cooks");
    Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene("scenes/yard.ogt")
        .unwrap()
}

fn lanes(row: &[u8], at: usize, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| f32::from_le_bytes(row[at + 4 * i..at + 4 * i + 4].try_into().unwrap()) as f64)
        .collect()
}

fn put(row: &mut [u8], at: usize, xs: &[f32]) {
    for (i, x) in xs.iter().enumerate() {
        row[at + 4 * i..at + 4 * i + 4].copy_from_slice(&x.to_le_bytes());
    }
}

/// The value a field of `scene` holds, as the world would answer it.
fn value(scene: &SceneDesc, entity: &str, component: &str, field: &str) -> Option<FieldValue> {
    let e = scene.entities.iter().position(|x| x.name == entity)?;
    let row = scene
        .components
        .iter()
        .find(|t| t.name == component)?
        .get(e)?;
    match (component, field) {
        ("Transform", "position") => Some(FieldValue::Lanes(lanes(row, 0, 3))),
        ("Transform", "scale") => Some(FieldValue::Lanes(lanes(row, 12, 3))),
        ("Switch", "on") => Some(FieldValue::Bool(row[0] != 0)),
        _ => None,
    }
}

#[test]
fn load_change_save_load_is_exact() {
    let root = std::env::temp_dir().join(format!("og-cook-write-back-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    let first = cook(&root, TEXT);

    // The change: the crate moved (to values that are not short decimals),
    // its switch flipped, the post scaled (a field its text lacks: tuned).
    let mut changed = first.clone();
    let crate_ = 0;
    let post = 1;
    let t = changed
        .components
        .iter_mut()
        .find(|t| t.name == "Transform")
        .unwrap();
    let row = &mut t.rows.iter_mut().find(|r| r.0 == crate_).unwrap().1;
    put(row, 0, &[-0.823_456_7, 0.591_234_5, 0.5]);
    let row = &mut t.rows.iter_mut().find(|r| r.0 == post).unwrap().1;
    put(row, 12, &[2.0, 0.25, 2.0]);
    let s = changed
        .components
        .iter_mut()
        .find(|t| t.name == "Switch")
        .unwrap();
    s.rows.iter_mut().find(|r| r.0 == crate_).unwrap().1[0] = 1;

    let tuned = vec![(
        "post".to_string(),
        "Transform".to_string(),
        "scale".to_string(),
    )];
    let saved = rewrite(TEXT, &|e, c, f| value(&changed, e, c, f), &tuned);
    assert!(saved.skipped.is_empty(), "{:?}", saved.skipped);
    let second = cook(&root, &saved.text);
    assert_eq!(second, changed, "the reloaded scene is the changed world");

    // Only the changed lines moved, comments and order kept.
    let want = TEXT
        .replace(
            "position -1 0.5 0 # dropped from here",
            "position -0.8234567 0.5912345 0.5 # dropped from here",
        )
        .replace("on false", "on true")
        .replace(
            "    position 2 0 0.1\n",
            "    position 2 0 0.1\n    scale 2 0.25 2\n",
        );
    assert_eq!(saved.text, want);
    assert_eq!(saved.lines, vec![8, 12, 19]);

    // Saving the reloaded world again writes nothing.
    let again = rewrite(&saved.text, &|e, c, f| value(&second, e, c, f), &tuned);
    assert_eq!(again.text, saved.text);
    assert!(again.lines.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

/// The scene's resources as the world would answer them (`Score.best`,
/// `Score.label` as packed in the scene's own subset schema).
fn resource_value(scene: &SceneDesc, resource: &str, field: &str) -> Option<FieldValue> {
    let r = scene.resources.iter().find(|r| r.name == resource)?;
    let layout = og_core::Layout::of(&r.schema);
    let i = r.schema.fields.iter().position(|f| f.name == field)?;
    let b = &r.bytes[layout.range(i)];
    Some(match field {
        "best" => FieldValue::Lanes(lanes(b, 0, 1)),
        "lives" => FieldValue::Lanes(vec![u32::from_le_bytes(b[..4].try_into().unwrap()) as f64]),
        _ => return None,
    })
}

const WITH_RESOURCE: &str = "type scene
version 1
---
name \"Yard\"
resource Score {
  best 3 # the record
}
entity \"crate\" {
  Transform {
    position -1 0.5 0
  }
}
";

/// The cook's half of resources and of entities spawned at run time: a
/// resource the text sets is rewritten in place, a tuned one it lacks gets
/// a block, and entities the world spawned are written under the marked
/// section; cooking the result loads exactly those values and entities, and
/// saving again changes nothing.
#[test]
fn resources_and_spawned_entities_round_trip_exactly() {
    use og_cook::scene_text::{
        Plan, RESOURCE, SPAWNED_MARK, SpawnedEntity, rewrite_with, spawned_names,
    };
    let root = std::env::temp_dir().join(format!("og-cook-write-back-r-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    let mut schemas = schemas();
    schemas.push(Schema {
        name: "Score".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("best", FieldType::F32),
            FieldSchema::new("lives", FieldType::U32),
        ],
    });
    let cook_with = |text: &str| -> SceneDesc {
        std::fs::write(root.join("assets/scenes/yard.ogt"), text).unwrap();
        let options = CookOptions {
            threads: 1,
            schemas: schemas.clone(),
            ..CookOptions::default()
        };
        cook_project(&root, &options).expect("the scene cooks");
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_scene("scenes/yard.ogt")
            .unwrap()
    };
    let first = cook_with(WITH_RESOURCE);
    // The scene sets Score.best only: its schema holds the fields it lists.
    assert_eq!(first.resources.len(), 1);
    assert_eq!(
        first.resources[0]
            .schema
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["best"]
    );
    assert_eq!(
        resource_value(&first, "Score", "best"),
        Some(FieldValue::Lanes(vec![3.0]))
    );

    // The world now: the record beaten (an f32 that is no short decimal),
    // lives tuned (a field the text lacks), and a bolt with a spark spawned.
    let world = |e: &str, c: &str, f: &str| -> Option<FieldValue> {
        match (e, c, f) {
            (RESOURCE, "Score", "best") => Some(FieldValue::Lanes(vec![7.123_456_7])),
            (RESOURCE, "Score", "lives") => Some(FieldValue::Lanes(vec![2.0])),
            _ => value(&first, e, c, f),
        }
    };
    let bolt = SpawnedEntity {
        name: "bolt".into(),
        components: vec![
            (
                "Transform".into(),
                vec![
                    (
                        "position".into(),
                        FieldValue::Lanes(vec![0.125, 2.333_333_3, -4.0]),
                    ),
                    ("scale".into(), FieldValue::Lanes(vec![0.5, 0.5, 0.5])),
                ],
            ),
            ("Switch".into(), vec![("on".into(), FieldValue::Bool(true))]),
        ],
        notes: vec![
            "not written: Mesh (it holds a geometry and a material, which scene text cannot name)"
                .into(),
        ],
        children: vec![SpawnedEntity {
            name: "spark".into(),
            components: vec![(
                "Transform".into(),
                vec![
                    ("position".into(), FieldValue::Lanes(vec![0.0, 0.25, 0.0])),
                    ("scale".into(), FieldValue::Lanes(vec![1.0, 1.0, 1.0])),
                ],
            )],
            ..SpawnedEntity::default()
        }],
        ..SpawnedEntity::default()
    };
    let plan = Plan {
        tuned: vec![(RESOURCE.into(), "Score".into(), "lives".into())],
        add_resources: true,
        spawned: Some(vec![bolt.clone()]),
    };
    let saved = rewrite_with(WITH_RESOURCE, &world, &plan);
    assert!(saved.skipped.is_empty(), "{:?}", saved.skipped);
    let lines: Vec<&str> = saved.text.lines().collect();
    // The resource's line rewritten in place with its comment, the tuned
    // field added inside its block; the authored entity untouched.
    assert_eq!(lines[5], "  best 7.1234565 # the record");
    assert_eq!(lines[6], "  lives 2");
    let mark = lines
        .iter()
        .position(|l| l.starts_with(SPAWNED_MARK))
        .expect("the marked section");
    assert!(
        lines[..mark].iter().all(|l| !l.contains("bolt")),
        "the spawned entity stays under the mark"
    );
    assert_eq!(spawned_names(&saved.text), ["bolt", "spark"]);

    let second = cook_with(&saved.text);
    // The resource: both fields, exactly.
    assert_eq!(
        resource_value(&second, "Score", "best"),
        Some(FieldValue::Lanes(vec![7.123_456_7_f32 as f64]))
    );
    assert_eq!(
        resource_value(&second, "Score", "lives"),
        Some(FieldValue::Lanes(vec![2.0]))
    );
    // The entities: the authored crate as it was, then the bolt and its spark under it.
    let names: Vec<(&str, Option<usize>)> = second
        .entities
        .iter()
        .map(|e| (e.name.as_str(), e.parent))
        .collect();
    assert_eq!(names, [("crate", None), ("bolt", None), ("spark", Some(1))]);
    assert_eq!(
        value(&second, "crate", "Transform", "position"),
        value(&first, "crate", "Transform", "position")
    );
    assert_eq!(
        value(&second, "bolt", "Transform", "position"),
        Some(FieldValue::Lanes(vec![0.125, 2.333_333_3_f32 as f64, -4.0]))
    );
    assert_eq!(
        value(&second, "bolt", "Transform", "scale"),
        Some(FieldValue::Lanes(vec![0.5, 0.5, 0.5]))
    );
    assert_eq!(
        value(&second, "bolt", "Switch", "on"),
        Some(FieldValue::Bool(true))
    );
    assert_eq!(
        value(&second, "spark", "Transform", "position"),
        Some(FieldValue::Lanes(vec![0.0, 0.25, 0.0]))
    );

    // Saved again from the reloaded world (the same values, the same
    // spawned entities): nothing changes.
    let again_world = |e: &str, c: &str, f: &str| -> Option<FieldValue> {
        if e == RESOURCE {
            return resource_value(&second, c, f);
        }
        value(&second, e, c, f)
    };
    let again = rewrite_with(&saved.text, &again_world, &plan);
    assert_eq!(again.text, saved.text);
    assert!(again.lines.is_empty(), "{:?}", again.lines);
    // Without spawned entities the section goes; without a plan for it, it stays as text.
    let gone = rewrite_with(
        &saved.text,
        &again_world,
        &Plan {
            spawned: Some(vec![]),
            ..plan.clone()
        },
    );
    assert!(!gone.text.contains(SPAWNED_MARK) && !gone.text.contains("bolt"));
    let kept = rewrite_with(
        &saved.text,
        &again_world,
        &Plan {
            spawned: None,
            ..plan
        },
    );
    assert_eq!(kept.text, saved.text);
    let _ = std::fs::remove_dir_all(&root);
}
