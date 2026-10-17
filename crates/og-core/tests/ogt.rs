//! `.ogt`: the reader and the canonical writer round-trip.

use og_core::ogt::{parse, Block, Document, Entry, Number, Value};
use og_core::ErrorKind;
use std::path::{Path, PathBuf};

const CANONICAL: &str = r#"type scene
version 3
---
name "Main \"hall\" \\ east"
gravity 0 -9.81 0
seed 4294967295
tiny 1e-7
huge -6.02e23
flags true false linear og.Transform ease-in
note "line one\nline two\ttabbed, # not a comment, { not a block }"
unicode "天气 é \u{1}"
entity "player" {
  Transform {
    position 0 1.5 -2
    rotation 0 0 0 1
  }
  Health {
    hp 100
  }
  children {
    entity "sword" {
      Transform {
        position 0.25 0 0
      }
    }
  }
  empty {
  }
}
entity "camera" {
  Camera {
    fov 60
  }
}
"#;

fn values(entry: &Entry) -> Vec<String> {
    entry
        .values
        .iter()
        .map(|v| match v {
            Value::Number(n) => format!("n:{}", n.text()),
            Value::String(s) => format!("s:{s}"),
            Value::Word(w) => format!("w:{w}"),
        })
        .collect()
}

#[test]
fn canonical_text_round_trips_exactly() {
    let doc = parse(CANONICAL).unwrap();
    assert_eq!(doc.to_text().unwrap(), CANONICAL);
    assert_eq!(parse(&doc.to_text().unwrap()).unwrap(), doc);
}

#[test]
fn the_tree_holds_what_the_text_says() {
    let doc = parse(CANONICAL).unwrap();
    assert_eq!((doc.ty.as_str(), doc.version), ("scene", 3));
    let body = &doc.body;
    assert_eq!(
        body.get("name").unwrap().string(0).unwrap(),
        "Main \"hall\" \\ east"
    );
    assert_eq!(
        values(body.get("gravity").unwrap()),
        ["n:0", "n:-9.81", "n:0"]
    );
    assert_eq!(body.get("gravity").unwrap().f32(1).unwrap(), -9.81f32);
    assert_eq!(
        body.get("seed")
            .unwrap()
            .number(0)
            .unwrap()
            .as_u64()
            .unwrap(),
        4294967295
    );
    assert_eq!(body.get("tiny").unwrap().number(0).unwrap().as_f64(), 1e-7);
    let flags = body.get("flags").unwrap();
    assert_eq!(flags.values[0].as_bool(), Some(true));
    assert_eq!(flags.values[1].as_bool(), Some(false));
    assert_eq!(flags.word(3).unwrap(), "og.Transform");
    assert_eq!(
        body.get("note").unwrap().string(0).unwrap(),
        "line one\nline two\ttabbed, # not a comment, { not a block }"
    );
    assert_eq!(
        body.get("unicode").unwrap().string(0).unwrap(),
        "天气 é \u{1}"
    );

    let entities: Vec<_> = body.get_all("entity").collect();
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0].string(0).unwrap(), "player");
    assert_eq!(entities[0].line, 12);
    let player = entities[0].block.as_ref().unwrap();
    let position = player
        .get("Transform")
        .unwrap()
        .block
        .as_ref()
        .unwrap()
        .get("position")
        .unwrap();
    assert_eq!(values(position), ["n:0", "n:1.5", "n:-2"]);
    let sword = player
        .get("children")
        .unwrap()
        .block
        .as_ref()
        .unwrap()
        .get("entity")
        .unwrap();
    assert_eq!(sword.string(0).unwrap(), "sword");
    assert_eq!(player.get("empty").unwrap().block, Some(Block::new()));
    assert_eq!(
        entities[0].number(0).unwrap_err().kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn loose_text_reads_as_its_canonical_form() {
    let loose = "# a header comment\r\ntype   scene\r\nversion 3\n---\n\n\t name \"Main \\\"hall\\\" \\\\ east\"   # trailing\n gravity\t0 -9.81 0\nseed 4294967295\ntiny 1e-7\nhuge -6.02e23\nflags true false linear og.Transform ease-in\nnote \"line one\\nline two\\ttabbed, # not a comment, { not a block }\"\nunicode \"天气 \\u{e9} \\u{1}\"\n  entity \"player\" {\n      Transform {\n position 0 1.5 -2\nrotation 0 0 0 1\n}\nHealth {\nhp 100\n}\nchildren {\nentity \"sword\" {\nTransform {\nposition 0.25 0 0\n}\n}\n}\nempty {\n}\n}\nentity \"camera\" {\n# inside\nCamera {\nfov 60\n}\n}";
    let doc = parse(loose).unwrap();
    assert_eq!(doc, parse(CANONICAL).unwrap());
    assert_eq!(doc.to_text().unwrap(), CANONICAL);
}

#[test]
fn documents_built_in_memory_write_and_read_back() {
    let mut doc = Document::new("material", 1);
    doc.body.push(
        Entry::new("albedo")
            .with(Number::from_f32(0.1).unwrap())
            .with(Number::from_f32(0.7).unwrap())
            .with(Number::from_f32(1e-9).unwrap()),
    );
    doc.body
        .push(Entry::new("roughness").with(Number::from_f64(0.30000000000000004).unwrap()));
    doc.body.push(
        Entry::new("count")
            .with(Number::from(-42i64))
            .with(Number::from(u64::MAX)),
    );
    doc.body
        .push(Entry::new("path").with(Value::String("assets/a b.gltf".into())));
    doc.body.push(
        Entry::new("pass")
            .with(Value::Word("forward".into()))
            .with_block(Block::new().with(Entry::new("depth").with(Value::Word("true".into())))),
    );
    let text = doc.to_text().unwrap();
    let back = parse(&text).unwrap();
    assert_eq!(back, doc);
    assert_eq!(back.body.get("albedo").unwrap().f32(0).unwrap(), 0.1f32);
    assert_eq!(back.body.get("albedo").unwrap().f32(2).unwrap(), 1e-9f32);
    assert_eq!(
        back.body
            .get("roughness")
            .unwrap()
            .number(0)
            .unwrap()
            .as_f64(),
        0.30000000000000004
    );
    assert_eq!(
        back.body
            .get("count")
            .unwrap()
            .number(1)
            .unwrap()
            .as_u64()
            .unwrap(),
        u64::MAX
    );
    assert!(text.contains("albedo 0.1 0.7 1e-9\n"), "{text}");

    assert!(Number::from_f32(f32::NAN).is_err());
    let mut bad = Document::new("scene", 1);
    bad.body.push(Entry::new("two words"));
    assert_eq!(bad.to_text().unwrap_err().kind(), ErrorKind::Invalid);
    let mut bad = Document::new("scene", 1);
    bad.body
        .push(Entry::new("k").with(Value::Word("not a word".into())));
    assert!(bad.to_text().is_err());
}

#[test]
fn malformed_text_is_refused_with_its_line() {
    let h = "type scene\nversion 1\n---\n";
    let cases: &[(String, &str)] = &[
        (String::new(), "missing header"),
        (
            "version 1\ntype scene\n---\n".into(),
            "line 1: expected 'type",
        ),
        (
            "type scene\nversion x\n---\n".into(),
            "line 2: expected 'version",
        ),
        (
            "type scene\nversion 1\nname 2\n".into(),
            "line 3: expected '---'",
        ),
        (format!("{h}a \"open\n"), "line 4: unterminated string"),
        (
            format!("{h}a \"x\"y\n"),
            "line 4: a string must be followed",
        ),
        (format!("{h}a \"\\q\"\n"), "line 4: unknown escape"),
        (
            format!("{h}a {{\nb 1\n"),
            "line 4: block 'a' opened here is not closed",
        ),
        (format!("{h}a 1\n}}\n"), "line 5: unmatched '}'"),
        (format!("{h}\"k\" 1\n"), "line 4: a line starts with a key"),
        (format!("{h}true 1\n"), "line 4: a line starts with a key"),
        (
            format!("{h}a List<u32>\n"),
            "line 4: 'List<u32>' is not a number",
        ),
        (format!("{h}a 1.\n"), "line 4: '1.' is not a number"),
        (
            format!("{h}a 1e999\n"),
            "line 4: number 1e999 is out of range",
        ),
        (format!("{h}a {{ b\n}}\n"), "line 4: '{' must end its line"),
        (format!("{h}a {{\n}} x\n"), "line 5: '}' must be alone"),
    ];
    for (text, needle) in cases {
        let err = parse(text).expect_err(text);
        assert_eq!(err.kind(), ErrorKind::Parse);
        assert!(
            err.reason().contains(needle),
            "{text:?}: got '{}', expected '{needle}'",
            err.reason()
        );
    }
}

fn ogt_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name.as_ref(), "target" | ".git" | "node_modules")
                && !name.starts_with('.')
            {
                ogt_files(&path, out);
            }
        } else if name.ends_with(".ogt") {
            out.push(path);
        }
    }
}

/// Every `.ogt` file in the repository (the API IDL, templates, fixtures)
/// either round-trips through the canonical writer, or is a fixture that
/// declares `# expect: <line>: ...` and is refused at that line.
#[test]
fn every_ogt_file_in_the_repository_round_trips() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    ogt_files(&root, &mut files);
    files.sort();
    let mut round_tripped = 0;
    for path in &files {
        let text = std::fs::read_to_string(path).unwrap();
        let expect_line = text
            .lines()
            .find_map(|l| l.strip_prefix("# expect: "))
            .and_then(|e| e.split_once(':'))
            .and_then(|(line, _)| line.trim().parse::<usize>().ok());
        match parse(&text) {
            Ok(doc) => {
                let canonical = doc
                    .to_text()
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let again = parse(&canonical).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                assert_eq!(again, doc, "{}", path.display());
                assert_eq!(again.to_text().unwrap(), canonical, "{}", path.display());
                round_tripped += 1;
            }
            Err(err) => {
                let line = expect_line.unwrap_or_else(|| panic!("{}: {err}", path.display()));
                assert!(
                    err.reason().starts_with(&format!("line {line}:")),
                    "{}: refused as '{}', the fixture expects line {line}",
                    path.display(),
                    err.reason()
                );
            }
        }
    }
    assert!(
        round_tripped >= 1,
        "no .ogt file found under {}",
        root.display()
    );
}
