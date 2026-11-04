//! Authored packs keep their assets and parse independently of bootstrap choices.
//! Contrast thresholds are not acceptance criteria (docs/ui.md §17).

use og_ui::*;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles")
}

#[test]
fn authored_packs_parse_and_keep_their_assets() {
    let root = root();
    let mut count = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.join("style.ogt").is_file() {
            continue;
        }
        count += 1;
        let text = std::fs::read_to_string(dir.join("style.ogt")).unwrap();
        let mut file = parse_style(&text).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        assert_eq!(file.version, 2, "{}", dir.display());
        let name = dir.file_name().unwrap().to_str().unwrap();
        file.resolve(&format!("{name}/style.ogt"), |id| root.join(id).exists());
        for asset in file.references() {
            let path = root.join(asset.strip_prefix("@styles/").unwrap_or(&asset));
            assert!(path.is_file(), "{name} references missing {asset}");
        }
        for event in og_ui::tokens::SoundEvent::ALL {
            for sound in file.feedback_sounds[event.index()].iter() {
                assert!(
                    root.join(sound.strip_prefix("@styles/").unwrap_or(sound))
                        .is_file(),
                    "{name}: {} references missing {sound}",
                    event.name()
                );
            }
        }
    }
    assert!(count > 0, "no authored packs discovered");
}

#[test]
fn a_styles_fonts_name_weights_italics_and_each_locales_companion() {
    let text = std::fs::read_to_string(root().join("scifi/style.ogt")).unwrap();
    let fonts = r#"fonts {
  body 300 "Light.ttf"
  body 600 "SemiBold.ttf"
  body italic "Italic.ttf"
  display 800 italic "BlackItalic.ttf"
  numeric "Mono.ttf"
  cjk "SC.ttf"
  cjk ja "JP.ttf"
  cjkDisplay ko 700 "KR-Bold.ttf"
}"#;
    let a = text.find("\nfonts {").expect("a fonts block") + 1;
    let b = a + text[a..].find("\n}").unwrap() + 2;
    let file = parse_style(&format!("{}{fonts}{}", &text[..a], &text[b..]))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(file.fonts[0].as_deref(), Some("Light.ttf"));
    assert_eq!(file.font_weights[0], 300);
    assert_eq!(file.fonts[1], None, "display names only an italic");
    assert_eq!(file.fonts[4].as_deref(), Some("SC.ttf"));
    let faces: Vec<_> = file
        .font_faces
        .iter()
        .map(|f| (f.role, f.cjk, f.locale, f.weight, f.italic, f.path.as_str()))
        .collect();
    assert_eq!(
        faces,
        [
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                600,
                false,
                "SemiBold.ttf"
            ),
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                400,
                true,
                "Italic.ttf"
            ),
            (
                FontRole::Display,
                false,
                CjkLocale::Hans,
                800,
                true,
                "BlackItalic.ttf"
            ),
            (FontRole::Body, true, CjkLocale::Ja, 400, false, "JP.ttf"),
            (
                FontRole::Display,
                true,
                CjkLocale::Ko,
                700,
                false,
                "KR-Bold.ttf"
            ),
        ]
    );
    // Each companion with its forms, so the cook ships only the shown ones.
    assert_eq!(
        file.companions(),
        [
            (CjkLocale::Hans, "SC.ttf".to_string()),
            (CjkLocale::Ja, "JP.ttf".to_string()),
            (CjkLocale::Ko, "KR-Bold.ttf".to_string()),
        ]
    );
    assert!(file.references().contains(&"BlackItalic.ttf".to_string()));
    for bad in [
        r#"body 650 "x.ttf""#,
        r#"cjk fr "x.ttf""#,
        r#"body ja "x.ttf""#,
    ] {
        let f = format!("fonts {{\n  {bad}\n}}");
        let e = parse_style(&format!("{}{f}{}", &text[..a], &text[b..])).unwrap_err();
        assert!(e.contains("line"), "{bad}: {e}");
    }
}
