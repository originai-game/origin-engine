use crate::support::CORE_STYLE;
use og_ui::*;

#[test]
fn an_ornament_atlas_needs_no_part_slices() {
    let file = parse_style(&format!(
        "{CORE_STYLE}\natlas \"ornaments.png\" 64 32\ncorner 16 8 24 12 24 12\n"
    ))
    .unwrap();
    assert_eq!(file.atlas.as_deref(), Some("ornaments.png"));
    assert_eq!(
        file.theme.art.corner.unwrap().uv,
        [0.25, 0.25, 0.625, 0.625]
    );
    assert!(file.theme.art.slices.iter().all(Option::is_none));
}

#[test]
fn a_styles_fonts_name_weights_italics_and_each_locales_companion() {
    let text = CORE_STYLE;
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

#[test]
fn a_styles_motion_tokens_drive_the_motion_code() {
    for text in [CORE_STYLE] {
        let theme = parse_style(text).expect("it parses").theme;
        let tokens = &theme.tokens;
        for (name, got) in [
            (og_ui::tokens::SpringName::Ui, theme.motion.ui),
            (og_ui::tokens::SpringName::Playful, theme.motion.playful),
        ] {
            let want = tokens.spring(name);
            assert_eq!(got.duration_ms, want.duration_ms, "{name:?}");
            let damping = want.to_spring().damping;
            assert!(
                (1.0 - got.bounce - damping).abs() < 1e-4 || got.bounce == -0.95,
                "{name:?}: bounce {} for damping {damping}",
                got.bounce
            );
        }
        assert_eq!(theme.motion.stagger_ms, tokens.motion.stagger_ms);
    }
    let stepped = CORE_STYLE.replace("ease enter 0.23 1 0.32 1", "ease enter steps 3");
    let theme = parse_style(&stepped).unwrap().theme;
    assert_eq!(
        theme.motion.enter_ease.apply(0.375),
        1.0 / 3.0,
        "style steps must not become a linear UI transition"
    );
    assert_eq!(
        theme.motion.exit_ease,
        Ease::CubicBezier([0.4, 0.0, 1.0, 1.0]),
        "the style's exit is not a GSAP approximation"
    );
}
