use og_ui::{Ui, UiDesc, dress::PartRef, style_pack::StylePack};

#[test]
fn pack_document_rules_respect_kit_game_layers_media_and_replacement() {
    use og_ui::{UiDocument, Viewport, bind::NoEnv, reactive::SourceId};
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            r#"
        :root { --size: 73px; }
        div { width: var(--size); height: 25px; }
        @media (max-width: 400px) { div { width: 51px; } }
        @media (prefers-reduced-motion: reduce) { div { width: 39px; } }
    "#
            .into(),
        )],
    )
    .unwrap();
    for (source, local, expected) in [
        (
            "ui-kits/test/main.svelte",
            "@layer structure { div { width: 91px; } }",
            73.0,
        ),
        (
            "ui/game.svelte",
            "@layer og-game { div { width: 87px; } }",
            87.0,
        ),
        ("ui/game.svelte", "div { width: 97px; }", 97.0),
    ] {
        let text = format!(
            r#"<meta name="screen" content="test"><div id="Box"></div><style>{local}</style>"#
        );
        let doc = UiDocument::parse(source, &text).unwrap();
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(doc);
        ui.set_style_pack(&pack).unwrap();
        ui.mount_document(
            SourceId::new("test", "test"),
            "test",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        let width = |ui: &mut Ui, w| {
            ui.begin(Default::default(), Viewport::new(w, 400.0));
            ui.build_retained_documents(&NoEnv).unwrap();
            ui.end();
            ui.elements()
                .iter()
                .find(|e| e.path.ends_with("/Box"))
                .unwrap()
                .width
        };
        assert_eq!(width(&mut ui, 600.0), expected);
        if source.starts_with("ui-kits/") {
            assert_eq!(width(&mut ui, 300.0), 51.0);
            let mut theme = *ui.theme();
            theme.reduced_motion = true;
            ui.set_theme(theme);
            assert_eq!(width(&mut ui, 300.0), 39.0);
            ui.style_documents(None).unwrap();
            assert_eq!(width(&mut ui, 600.0), 91.0);
        }
    }
}

#[test]
fn pack_imports_keep_their_own_asset_bases_and_cascade_order() {
    let pack = StylePack::parse(
        "styles/example",
        vec![
            (
                "styles/example/z.css".into(),
                "button::part(body) { background: #345678; }".into(),
            ),
            (
                "styles/example/a.css".into(),
                r#"
            @import "nested/type.css";
            :root { --pack-color: #abcdef; }
            button::part(body) { background: var(--pack-color); }
        "#
                .into(),
            ),
            (
                "styles/example/nested/type.css".into(),
                r#"
            @font-face { font-family: Pack; src: url(../../shared/body.ttf); }
            button::part(body) { border-image-source: url(../images/frame.png); }
        "#
                .into(),
            ),
        ],
    )
    .unwrap();
    assert_eq!(pack.name, "example");
    assert_eq!(pack.sheet.font_faces[0].src, ["styles/shared/body.ttf"]);
    assert_eq!(
        pack.assets,
        ["styles/example/images/frame.png", "styles/shared/body.ttf"]
    );
    let mut ui = Ui::new(UiDesc::default());
    ui.set_style_pack(&pack).unwrap();
    // A missing image is diagnosed during paint, not silently ignored.
    let errors = ui
        .resolve_part(
            &PartRef::new("button", "body", og_ui::tokens::ElementClass::Button),
            [100.0, 40.0],
        )
        .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("styles/example/images/frame.png")),
        "{errors:?}"
    );
}

#[test]
fn css_urls_ignore_comments_strings_and_local_fragments() {
    use og_ui::doc::at_rules::{rebase_urls, urls};
    let css = r#"/* url(fake.png) */ content: "url(fake.png)"; mask-image: url('../icons/a.svg'); filter: url(#blur); background-image: image-set(url(b.png) 1x, url(c.png) 2x);"#;
    let rebased = rebase_urls("styles/pack/nested/a.css", css);
    assert!(rebased.contains("/styles/pack/icons/a.svg"), "{rebased}");
    assert!(rebased.contains("/styles/pack/nested/b.png"), "{rebased}");
    assert!(rebased.contains("url(#blur)") || rebased.contains("url(\"#blur\")"));
    assert_eq!(urls(css), ["../icons/a.svg", "b.png", "c.png"]);
}

#[test]
fn pack_audio_preserves_sample_sets_ranges_silence_and_replay() {
    use og_ui::tokens::SoundEvent;
    let pack = StylePack::parse("styles/audio", vec![("styles/audio/main.css".into(),
        ":root { -og-sound: press url(a.wav) url(b.wav) volume(0.28 0.34) pitch(0.98 1.02), hover none; }".into())]).unwrap();
    let mut ui = Ui::new(UiDesc::default());
    ui.set_style_pack(&pack).unwrap();
    ui.seed_feedback(123);
    ui.set_css_sounds(
        [
            ("styles/audio/a.wav".into(), 11),
            ("styles/audio/b.wav".into(), 22),
        ]
        .into(),
    );
    ui.feedback(SoundEvent::Hover);
    assert!(ui.take_feedback().is_empty());
    let state = ui.save_state();
    let play = |ui: &mut Ui| {
        (0..8)
            .map(|_| {
                ui.feedback(SoundEvent::Press);
                let p = ui.take_feedback().pop().expect("CSS audio plays");
                assert!([11, 22].contains(&p.sound));
                assert!((0.28..=0.34).contains(&p.volume));
                assert!((0.98..=1.02).contains(&p.pitch));
                (p.sound, p.volume, p.pitch)
            })
            .collect::<Vec<_>>()
    };
    let first = play(&mut ui);
    assert!(first.windows(2).all(|p| p[0].0 != p[1].0));
    ui.restore_state(&state).unwrap();
    assert_eq!(first, play(&mut ui));
    for bad in [
        "press",
        "press url(a) volume(0.8 0.2)",
        "press url(a) pitch(0)",
        "missing url(a)",
    ] {
        assert!(og_ui::sound_css::parse(bad).is_err(), "{bad}");
    }
}
