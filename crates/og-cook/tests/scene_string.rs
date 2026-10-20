//! A string field in scene text: one quoted string, cooked as its UTF-8 then
//! zeros to the field's capacity (og-core's `strN` layout), read back as the
//! same text, so a scene that writes the value back is the text it was read
//! from. A text over the capacity, numbers for a string or a string without
//! the schema are refused with the line and the fix.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
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
fn a_quoted_string_cooks_to_its_field_and_reads_back_as_the_same_text() {
    let names = [("a", "Mei", 3), ("b", "小灯 \"lantern\"", 0), ("c", "", 1)];
    let text = scene_text(&names);
    assert!(text.contains("name \"小灯 \\\"lantern\\\"\""), "{text}");
    let root = project("ok", &text);
    let report = cook_project(&root, &options(vec![profile()])).expect("the scene cooks");
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    let scene = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene("scenes/s.ogt")
        .unwrap();
    let table = scene
        .components
        .iter()
        .find(|c| c.name == "Profile")
        .unwrap();

    // Bytes: 16 of zero-padded UTF-8, then the u32; the packed check passes.
    let mut back = Vec::new();
    for (i, (entity, name, level)) in names.iter().enumerate() {
        let row = table.get(i).unwrap();
        assert_eq!(row.len(), 16 + 4);
        profile().check_packed(row).unwrap();
        assert_eq!(&row[..name.len()], name.as_bytes());
        assert!(row[name.len()..16].iter().all(|b| *b == 0), "zero-padded");
        assert_eq!(u32::from_le_bytes(row[16..20].try_into().unwrap()), *level);
        back.push((*entity, og_core::unpack_str(&row[..16]).to_owned(), *level));
    }
    // The text written back from the cooked values is the source text.
    let back: Vec<(&str, &str, u32)> = back.iter().map(|(e, n, l)| (*e, n.as_str(), *l)).collect();
    assert_eq!(scene_text(&back), text);

    // The same text cooks to the same bytes (stable hashes and diffs).
    let again = project("again", &text);
    cook_project(&again, &options(vec![profile()])).unwrap();
    let read = |r: &Path| {
        let blob = |e: &std::fs::DirEntry| e.path().extension().is_some_and(|x| x == "ogb");
        let dir = r.join("cooked/scenes");
        let file = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .find(blob)
            .expect("a cooked scene");
        std::fs::read(file.path()).unwrap()
    };
    assert_eq!(read(&root), read(&again));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&again);
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
