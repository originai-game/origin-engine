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
        "/../../tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf"
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
        .filter(|g| g.codepoint >= og_bundle::format::GLYPH_ID_KEYS)
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
    // An ASCII-only atlas carries layout tables but no runtime outlines.
    let latin = b"type import\nversion 1\n---\nfont {\n  charset ascii\n}\n".to_vec();
    let root = project(
        "latin",
        &[
            (
                "fonts/ar.ttf",
                std::fs::read(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf"
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
    assert!(font.ttf.is_none());
    let tables = font.shaping.expect("paragraph layout tables");
    let face = ttf_parser::Face::parse(&tables, 0).unwrap();
    assert!(
        face.raw_face()
            .table(ttf_parser::Tag::from_bytes(b"glyf"))
            .is_none()
    );
    let file = og_ui::shape::FontFile::for_shaping(std::sync::Arc::new(tables)).unwrap();
    assert_eq!(
        file.advance('A'),
        None,
        "layout data must not enable runtime rasterization"
    );
}

/// A font pack (docs/ui.md §3.4): its index names its languages, its faces'
/// asset ids and what their cmaps cover; its faces cook to `.face.ogb`
/// (the web reads them on demand) with their outlines for run-time glyphs
/// and no atlas beyond a space; installing it changes nothing another font
/// cooks.
///
/// Ways it could fail: the pack index pulls the faces into another font's
/// stamp (every font re-cooks, its bytes or charset move); a face cooks a
/// script's whole atlas (megabytes of payload); the face loses runtime or
/// ships at boot; the coverage misses a character, so the runtime never
/// loads the pack for it.
#[test]
fn a_font_pack_indexes_its_faces_and_leaves_other_fonts_alone() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let display =
        std::fs::read(repo.join("templates/styles/shared/ChakraPetch-BoldItalic.ttf")).unwrap();
    let settings =
        b"type import\nversion 1\n---\nfont {\n  charset latin punct\n  em_px 48\n  px_range 16\n}\n".to_vec();
    let base = vec![
        ("fonts/Display.ttf", display.clone()),
        ("fonts/Display.ttf.ogt", settings.clone()),
    ];
    let mut with_pack = base.clone();
    let pack = repo.join("templates/fonts/arabic");
    let names: Vec<String> = std::fs::read_dir(&pack)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    let files: Vec<(String, Vec<u8>)> = names
        .iter()
        .map(|n| {
            (
                format!("fonts/arabic/{n}"),
                std::fs::read(pack.join(n)).unwrap(),
            )
        })
        .collect();
    with_pack.extend(files.iter().map(|(p, b)| (p.as_str(), b.clone())));

    let output = |root: &PathBuf, id: &str| {
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .manifest()
            .outputs()
            .find(|o| o.id == id)
            .cloned()
            .unwrap_or_else(|| panic!("no output {id}"))
    };
    let alone = project("pack-alone", &base);
    assert!(cook_project(&alone, &CookOptions::default()).unwrap().ok());
    let root = project("pack", &with_pack);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());

    // The other font: the same stamp, bytes and file; its charset as before.
    let (a, b) = (
        output(&alone, "fonts/Display.ttf"),
        output(&root, "fonts/Display.ttf"),
    );
    assert_eq!(
        (a.stamp, a.hash, a.bytes, &a.file),
        (b.stamp, b.hash, b.bytes, &b.file)
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let font = bundle.load_font("fonts/Display.ttf").unwrap();
    assert!(font.ttf.is_none(), "its run-time mode is its own (off)");
    assert!(
        !font
            .glyphs
            .iter()
            .any(|g| (0x0600..0x0700).contains(&g.codepoint))
    );

    // The index.
    let id = "fonts/arabic/extension.json";
    let out = output(&root, id);
    assert_eq!(out.ty, "og.FontPack");
    let text = bundle
        .font_pack_text(
            id,
            &std::fs::read(root.join("cooked").join(&out.file)).unwrap(),
        )
        .unwrap();
    let index = og_i18n::FontPack::parse(&text).unwrap();
    assert_eq!(index.name, "arabic");
    assert!(index.languages.iter().any(|l| l == "ar") && index.languages.iter().any(|l| l == "fa"));
    let face_id = "fonts/arabic/NotoSansArabic-Regular.ttf";
    assert_eq!(index.faces, vec![face_id.to_string()]);
    for c in ['ل', 'ی', '٣', 'ﻻ'] {
        assert!(index.covers(c), "{c:?} U+{:04X}", c as u32);
    }
    assert!(!index.covers('あ') && !index.covers('ก'));

    // The face: on demand, outlines kept, no atlas to speak of.
    let face = output(&root, face_id);
    assert_eq!(
        face.file,
        format!("{face_id}.{}", og_bundle::runtime::FONT_PACK_FACE)
    );
    let ttf = files.iter().find(|(p, _)| p == face_id).unwrap().1.len() as u64;
    let font = bundle.load_font(face_id).unwrap();
    assert_eq!(
        font.ttf.as_ref().map(Vec::len),
        Some(ttf as usize),
        "drawn from outlines at run time"
    );
    // Budget: the face's own bytes plus 8 KB of header, metrics and the
    // space's cell; the index within 2 KB for an alphabet.
    assert!(
        face.bytes <= ttf + 8 * 1024,
        "{} cooked for a {ttf}-byte face",
        face.bytes
    );
    assert!(out.bytes <= 2 * 1024, "index {} bytes", out.bytes);
}
