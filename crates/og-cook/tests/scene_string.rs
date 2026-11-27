//! Scene text validation: a string field's text over the capacity, numbers
//! for a string or a string without the schema are refused with the line
//! and the fix.

use og_cook::cook::{CookOptions, cook_project};
use og_core::ogt::{Block, Document, Entry, Value};
use og_core::reflect::{FieldSchema, FieldType, Schema};
use std::path::{Path, PathBuf};

fn profile() -> Schema {
    Schema {
        name: "Profile".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            FieldSchema::new("name", FieldType::string(16).unwrap()),
            FieldSchema::new("level", FieldType::U32),
        ],
    }
}

fn project(tag: &str, scene: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("og-cook-scene-string-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    crate::init_project(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/s.ogt"), scene).unwrap();
    root
}

fn options(schemas: Vec<Schema>) -> CookOptions {
    CookOptions {
        threads: 1,
        schemas,
        ..CookOptions::default()
    }
}

/// The scene text of entities named after their Profile, canonical form.
fn scene_text(names: &[(&str, &str, u32)]) -> String {
    let mut body = Block::new().with(Entry::new("name").with(Value::String("Names".into())));
    for (entity, name, level) in names {
        let mut fields = Block::new().with(Entry::new("name").with(Value::String((*name).into())));
        if *level != 0 {
            fields.push(Entry::new("level").with(Value::Number(
                og_core::ogt::Number::parse(&level.to_string()).unwrap(),
            )));
        }
        body.push(
            Entry::new("entity")
                .with(Value::String((*entity).into()))
                .with_block(Block::new().with(Entry::new("Profile").with_block(fields))),
        );
    }
    Document {
        ty: "scene".into(),
        version: 1,
        body,
    }
    .to_text()
    .unwrap()
}

fn errors(root: &Path, schemas: Vec<Schema>) -> String {
    cook_project(root, &options(schemas))
        .unwrap()
        .errors()
        .join("\n")
}

#[test]
fn a_string_over_its_capacity_or_of_the_wrong_kind_is_refused_with_the_line() {
    // 17 bytes for 16.
    let root = project("long", &scene_text(&[("a", "Bartholomew Bold!", 0)]));
    let e = errors(&root, vec![profile()]);
    assert!(
        e.contains("Profile.name: 'Bartholomew Bold!' is 17 bytes of UTF-8 and the field holds 16")
            && e.contains("str(17)"),
        "{e}"
    );
    let _ = std::fs::remove_dir_all(&root);

    let root = project(
        "numbers",
        "type scene\nversion 1\n---\nentity \"a\" {\n  Profile {\n    name 1 2\n  }\n}\n",
    );
    let e = errors(&root, vec![profile()]);
    assert!(
        e.contains("line 6: Profile.name is a str16: give it one quoted string"),
        "{e}"
    );
    let _ = std::fs::remove_dir_all(&root);

    let root = project(
        "level",
        "type scene\nversion 1\n---\nentity \"a\" {\n  Profile {\n    level \"three\"\n  }\n}\n",
    );
    let e = errors(&root, vec![profile()]);
    assert!(e.contains("Profile.level is a u32, not a string"), "{e}");
    let _ = std::fs::remove_dir_all(&root);

    let root = project("schemaless", &scene_text(&[("a", "Mei", 0)]));
    let e = errors(&root, Vec::new());
    assert!(
        e.contains("Profile.name is a string; without Profile's schema"),
        "{e}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
