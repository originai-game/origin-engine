//! A list field in scene text: its elements on its one line (`passes 3 5`),
//! the key alone for the empty list, cooked as og-core's list layout (a
//! count lane, the elements, zeros) in a component or a resource, and
//! written back as the same line. A list over its capacity, a string for a
//! list or a fraction for a whole-number element is refused with the line.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use og_cook::scene_text::{FieldValue, RESOURCE, rewrite};
use og_core::reflect::{FieldSchema, FieldType, Scalar, Schema};
use std::path::PathBuf;

fn chain() -> Schema {
    Schema {
        name: "Chain".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("passes", FieldType::list(Scalar::U32, 3).unwrap()),
            FieldSchema::new("weights", FieldType::list(Scalar::F32, 2).unwrap()),
        ],
    }
}

fn project(tag: &str, scene: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("og-cook-scene-list-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/s.ogt"), scene).unwrap();
    root
}

fn options() -> CookOptions {
    CookOptions {
        threads: 1,
        schemas: vec![chain()],
        ..CookOptions::default()
    }
}

/// A u32[3] and an f32[2] as og-core packs them.
fn packed(passes: &[u32], weights: &[f32]) -> Vec<u8> {
    let mut b = (passes.len() as u32).to_le_bytes().to_vec();
    passes.iter().for_each(|p| b.extend(p.to_le_bytes()));
    b.resize(16, 0);
    b.extend((weights.len() as f32).to_le_bytes());
    weights.iter().for_each(|w| b.extend(w.to_le_bytes()));
    b.resize(16 + 12, 0);
    b
}

const SCENE: &str = "type scene
version 1
---
entity \"a\" {
  Chain {
    passes 3 0
    weights 0.5
  }
}
entity \"b\" {
  Chain {
    passes
  }
}
entity \"c\" {
  Chain {
    passes 0
  }
}
resource Chain {
  passes 7 8 9
}
";

#[test]
fn a_list_on_its_line_cooks_to_its_field_and_writes_back_the_same() {
    let root = project("ok", SCENE);
    let report = cook_project(&root, &options()).expect("the scene cooks");
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    let scene = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene("scenes/s.ogt")
        .unwrap();
    let table = scene.components.iter().find(|c| c.name == "Chain").unwrap();
    let rows = [packed(&[3, 0], &[0.5]), packed(&[], &[]), packed(&[0], &[])];
    for (i, want) in rows.iter().enumerate() {
        let row = table.get(i).unwrap();
        chain().check_packed(row).unwrap();
        assert_eq!(row, want.as_slice(), "row {i}");
    }
    assert_ne!(rows[1], rows[2], "`passes` alone is [], `passes 0` is [0]");
    let resource = &scene.resources[0];
    assert_eq!(resource.name, "Chain");
    assert_eq!(resource.schema.fields.len(), 1, "only the field it lists");
    assert_eq!(resource.bytes, packed(&[7, 8, 9], &[])[..16]);
    let _ = std::fs::remove_dir_all(&root);

    // Writing the same values back keeps the text; another list rewrites
    // the line, an empty one to the key alone.
    let same = |entity: &str, _: &str, field: &str| -> Option<FieldValue> {
        Some(FieldValue::Lanes(match (entity, field) {
            ("a", "passes") => vec![3.0, 0.0],
            ("a", "weights") => vec![0.5],
            ("b", "passes") => vec![],
            ("c", "passes") => vec![0.0],
            (RESOURCE, "passes") => vec![7.0, 8.0, 9.0],
            _ => return None,
        }))
    };
    let back = rewrite(SCENE, &same, &[]);
    assert_eq!(back.text, SCENE);
    assert!(back.lines.is_empty(), "{:?}", back.lines);
    let moved = |entity: &str, c: &str, field: &str| match (entity, field) {
        ("a", "passes") => Some(FieldValue::Lanes(vec![])),
        ("b", "passes") => Some(FieldValue::Lanes(vec![4.0, 2.0])),
        _ => same(entity, c, field),
    };
    let back = rewrite(SCENE, &moved, &[]);
    assert!(
        back.text.contains("    passes\n    weights 0.5"),
        "{}",
        back.text
    );
    assert!(back.text.contains("    passes 4 2\n"), "{}", back.text);
}

#[test]
fn a_list_over_its_capacity_or_of_the_wrong_values_is_refused_with_the_line() {
    let refused = |tag: &str, line: &str| {
        let root = project(
            tag,
            &format!(
                "type scene\nversion 1\n---\nentity \"a\" {{\n  Chain {{\n    {line}\n  }}\n}}\n"
            ),
        );
        let e = cook_project(&root, &options()).unwrap().errors().join("\n");
        let _ = std::fs::remove_dir_all(&root);
        e
    };
    let e = refused("long", "passes 1 2 3 4");
    assert!(
        e.contains("line 6: Chain.passes: 4 elements and the field holds 3"),
        "{e}"
    );
    let e = refused("text", "passes \"1 2\"");
    assert!(
        e.contains("Chain.passes is a u32[3]: give it up to 3"),
        "{e}"
    );
    let e = refused("fraction", "passes 1.5");
    assert!(e.contains("Chain.passes: 1.5 is not a u32"), "{e}");
}
