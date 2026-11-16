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
            // No font pack names Japanese, so none carries its dictionary.
            "assets/strings/ja.ogt: warning: locale 'ja': no font pack names its language, so none carries ICU's 'cjdict' dictionary and its words and lines break by the rules alone".to_string(),
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

/// docs/ui.md §3.4: a font pack's hyphenation dictionaries (CSS
/// `hyphens: auto`) ship in its cooked index only when the game's
/// documents declare `hyphens: auto`, and only for the locales its string
/// tables name; a shipped locale whose pack has no dictionary for it is
/// warned of, and still cooks.
///
/// Ways it could fail: every pack language's dictionary shipped (a
/// German or Hungarian one in an English game), a game without `hyphens:
/// auto` paying for them, a dictionary's bytes changed on the way, or a
/// locale left without hyphenation silently.
#[test]
fn a_font_pack_ships_hyphenation_dictionaries_only_for_the_games_locales() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pack = repo.join("templates/fonts/latin");
    let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(&pack)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .map(|n| {
            (
                format!("fonts/latin/{n}"),
                std::fs::read(pack.join(&n)).unwrap(),
            )
        })
        .collect();
    for l in ["en", "de", "vi"] {
        files.push((format!("strings/{l}.ogt"), table(l, &[])));
    }
    let document = |css: &str| {
        format!("<meta name=\"component\" content=\"t\">\n<p>Silbentrennung</p>\n<style>p {{ {css} }}</style>\n")
            .into_bytes()
    };
    let dictionaries = |name: &str, css: &str| {
        let mut all = files.clone();
        all.push(("ui/t.svelte".into(), document(css)));
        let all: Vec<(&str, Vec<u8>)> = all.iter().map(|(p, b)| (p.as_str(), b.clone())).collect();
        let root = project(name, &all);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let id = "fonts/latin/extension.json";
        let shipped = bundle
            .font_pack_dictionaries(id, &bundle.read(id).unwrap(), "hyphenation")
            .unwrap();
        (shipped, report.warnings)
    };

    let (shipped, warnings) = dictionaries("hyphens-auto", "hyphens: auto;");
    let languages: Vec<&str> = shipped.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(languages, ["en", "de"]);
    for (l, bytes) in &shipped {
        assert_eq!(
            bytes,
            &std::fs::read(pack.join(format!("{l}.hyph"))).unwrap(),
            "{l}"
        );
    }
    assert!(
        warnings.iter().any(|w| w
            .starts_with("assets/fonts/latin/extension.json: warning: hyphens: auto:")
            && w.contains("'vi'")),
        "{warnings:?}"
    );

    let (shipped, warnings) = dictionaries("hyphens-manual", "hyphens: manual;");
    assert!(shipped.is_empty());
    assert!(
        !warnings.iter().any(|w| w.contains("hyphens")),
        "{warnings:?}"
    );
}

/// docs/ui.md §3.4: ICU's word dictionary for a script written without
/// spaces ships in the font pack of a shipped locale's language, and only
/// then: a Thai game's `thai` pack carries `thaidict` (ICU's bytes as the
/// engine reads them), an English game's carries nothing, and a Japanese
/// locale no pack names is warned of.
///
/// Ways it could fail: a dictionary shipped for a locale the game lacks
/// (every game paying for Thai), none shipped where needed (Thai lines
/// breaking only at spaces), or bytes the runtime cannot read.
#[test]
fn a_font_pack_ships_word_dictionaries_only_for_the_games_locales() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pack = repo.join("templates/fonts/thai");
    let files: Vec<(String, Vec<u8>)> = std::fs::read_dir(&pack)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .map(|n| {
            (
                format!("fonts/thai/{n}"),
                std::fs::read(pack.join(&n)).unwrap(),
            )
        })
        .collect();
    let shipped = |name: &str, locales: &[&str]| {
        let mut all = files.clone();
        for l in locales {
            all.push((format!("strings/{l}.ogt"), table(l, &[])));
        }
        let all: Vec<(&str, Vec<u8>)> = all.iter().map(|(p, b)| (p.as_str(), b.clone())).collect();
        let root = project(name, &all);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let id = "fonts/thai/extension.json";
        let words = bundle
            .font_pack_dictionaries(id, &bundle.read(id).unwrap(), "segmentation")
            .unwrap();
        (words, report.warnings)
    };

    let (words, warnings) = shipped("words-thai", &["en", "th", "ja"]);
    let names: Vec<&str> = words.iter().map(|(a, _)| a.as_str()).collect();
    assert_eq!(names, ["thaidict"]);
    og_i18n::segmenter::install("thaidict", &words[0].1).unwrap();
    assert!(og_i18n::segmenter::installed("thaidict"));
    let chars: Vec<char> = "ภาษาไทยง่าย".chars().collect();
    assert_eq!(
        og_i18n::word_segments(&chars).len(),
        3,
        "cut by the shipped dictionary"
    );
    assert!(
        warnings.iter().any(
            |w| w.starts_with("assets/strings/ja.ogt: warning: locale 'ja':")
                && w.contains("'cjdict'")
        ),
        "{warnings:?}"
    );

    let (words, warnings) = shipped("words-english", &["en"]);
    assert!(words.is_empty());
    assert!(
        !warnings.iter().any(|w| w.contains("dictionary")),
        "{warnings:?}"
    );
}
