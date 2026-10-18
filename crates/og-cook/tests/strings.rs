//! String tables and shaped-script fonts through the cook and back.
//!
//! Ways this could fail: a table is cooked without being validated (a
//! malformed message reaches the game), or refused without its line; a
//! locale's missing key, changed arguments or extra key goes unreported;
//! the cooked table is not the text the runtime parses; an Arabic font is
//! cooked without its joined forms or its font file, so og-ui cannot shape.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-strings")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn table(locale: &str, entries: &[(&str, &str)]) -> Vec<u8> {
    let mut s = format!("type strings\nversion 1\n---\nlocale \"{locale}\"\n");
    for (k, v) in entries {
        s.push_str(&format!("string \"{k}\" \"{v}\"\n"));
    }
    s.into_bytes()
}

#[test]
fn tables_are_validated_and_every_locale_is_checked_against_the_base() {
    let root = project(
        "check",
        &[
            (
                "strings/en.ogt",
                table(
                    "en",
                    &[
                        ("a", "A"),
                        ("b", "{n, plural, one {# b} other {# bs}}"),
                        ("c", "Hi {name}"),
                    ],
                ),
            ),
            (
                "strings/ja.ogt",
                table("ja", &[("a", "あ"), ("c", "やあ {who}"), ("x", "extra")]),
            ),
            (
                "strings/bad.ogt",
                b"type strings\nversion 1\n---\nlocale \"fr\"\nstring \"a\" \"{oops\"\n".to_vec(),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].starts_with("assets/strings/bad.ogt: line 5: 'a':"),
        "{errors:?}"
    );
    assert_eq!(
        report.warnings,
        vec![
            "assets/strings/ja.ogt: missing key 'b' (en has it)".to_string(),
            "assets/strings/ja.ogt: key 'c' takes other arguments (ja takes {who}, en {name})"
                .to_string(),
            "assets/strings/ja.ogt: key 'x' is not in the base table (en does not have it)"
                .to_string(),
        ]
    );
    let ja = report
        .assets
        .iter()
        .find(|a| a.source == "strings/ja.ogt")
        .unwrap();
    assert!(
        ja.notes.iter().any(|n| n.contains("missing key 'b'")),
        "{:?}",
        ja.notes
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let text = bundle
        .strings_text("strings/en.ogt", &bundle.read("strings/en.ogt").unwrap())
        .unwrap();
    assert_eq!(
        text.as_bytes(),
        table(
            "en",
            &[
                ("a", "A"),
                ("b", "{n, plural, one {# b} other {# bs}}"),
                ("c", "Hi {name}")
            ]
        )
    );
}

#[test]
fn an_arabic_font_carries_its_joined_forms_and_its_file() {
    let ttf = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    ))
    .unwrap();
    let settings =
        b"type import\nversion 1\n---\nfont {\n  charset ascii arabic\n  em_px 24\n}\n".to_vec();
    let root = project(
        "arabic",
        &[
            ("fonts/ar.ttf", ttf.clone()),
            ("fonts/ar.ttf.ogt", settings),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let font = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_font("fonts/ar.ttf")
        .unwrap();
    assert_eq!(
        font.ttf.as_deref(),
        Some(ttf.as_slice()),
        "the font file rides along for shaping"
    );
    let letters = font
        .glyphs
        .iter()
        .filter(|g| (0x0600..0x0700).contains(&g.codepoint))
        .count();
    let forms = font
        .glyphs
        .iter()
        .filter(|g| g.codepoint >= 0xF_0000)
        .count();
    assert!(
        letters >= 20,
        "the Arabic letters the subset holds: {letters}"
    );
    assert!(
        forms >= 2 * letters,
        "their contextual forms, reached through GSUB: {forms} for {letters} letters"
    );
    let note = &report
        .assets
        .iter()
        .find(|a| a.source == "fonts/ar.ttf")
        .unwrap()
        .notes[0];
    assert!(note.contains("shaped script"), "{note}");
    // A Latin font carries neither.
    let latin = b"type import\nversion 1\n---\nfont {\n  charset ascii\n}\n".to_vec();
    let root = project(
        "latin",
        &[
            (
                "fonts/ar.ttf",
                std::fs::read(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
                ))
                .unwrap(),
            ),
            ("fonts/ar.ttf.ogt", latin),
        ],
    );
    cook_project(&root, &CookOptions::default()).unwrap();
    let font = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_font("fonts/ar.ttf")
        .unwrap();
    assert!(font.ttf.is_none() && font.glyphs.iter().all(|g| g.codepoint < 0xF_0000));
}
