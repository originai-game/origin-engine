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
