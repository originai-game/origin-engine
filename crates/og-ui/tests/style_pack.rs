use og_ui::{Ui, UiDesc, dress::PartRef, style_pack::StylePack};

#[test]
fn pack_selectors_cross_components_but_document_selectors_stay_local() {
    use og_ui::{
        UiDocument, Viewport,
        bind::NoEnv,
        reactive::{SourceId, wire},
    };
    use serde_json::json;
    let mut ui = Ui::new(UiDesc::default());
    for (path, text) in [
        (
            "@ui/test/child.svelte",
            r#"<meta name="component" content="Child"><div><div id="Target"></div></div><style>#Target { width: 21px; height: 20px; }</style>"#,
        ),
        (
            "@ui/test/parent.svelte",
            r#"<meta name="component" content="Parent"><div class:active={$on}><Child/></div><style>.active #Target { height: 99px; }</style>"#,
        ),
    ] {
        ui.define_document(UiDocument::parse(path, text).unwrap());
    }
    ui.set_style_pack(
        &StylePack::parse(
            "styles/test",
            vec![(
                "styles/test/main.css".into(),
                ".active #Target { width: 73px; }".into(),
            )],
        )
        .unwrap(),
    )
    .unwrap();
    let source = SourceId::new("test", "on");
    let apply = |ui: &mut Ui, command| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    apply(
        &mut ui,
        json!({"op":"signal", "source":source, "initial":false}),
    );
    ui.mount_document(
        SourceId::new("test", "parent"),
        "Parent",
        [("on".into(), source.clone())].into(),
        Default::default(),
        &NoEnv,
    )
    .unwrap();
    for on in [false, true, false] {
        apply(&mut ui, json!({"op":"set", "source":source, "value":on}));
        ui.begin(Default::default(), Viewport::new(300.0, 300.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        let target = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Target"))
            .unwrap();
        assert_eq!(target.width, if on { 73.0 } else { 21.0 }, "on={on}");
        assert_eq!(
            target.height, 20.0,
            "document selector must not cross component boundaries"
        );
    }
}

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
            "@ui/test/main.svelte",
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
        if source.starts_with("@ui/") {
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
    // A url() held in a root token resolves against the file that declares
    // it, not the one that reads it: a variant reusing another pack's
    // tokens (templates/styles/fantasy's --button-image) keeps its art.
    let variant = StylePack::parse(
        "styles/variant",
        vec![
            (
                "styles/variant/main.css".into(),
                r#"
            @import "../base/tokens.css";
            button::part(body) { border-image-source: var(--button-image); }
        "#
                .into(),
            ),
            (
                "styles/base/tokens.css".into(),
                ":root { --button-image: url(art/button-iron.svg); }".into(),
            ),
        ],
    )
    .unwrap();
    assert_eq!(variant.assets, ["styles/base/art/button-iron.svg"]);
    ui.set_style_pack(&variant).unwrap();
    let errors = ui
        .resolve_part(
            &PartRef::new("button", "body", og_ui::tokens::ElementClass::Button),
            [100.0, 40.0],
        )
        .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("styles/base/art/button-iron.svg")),
        "{errors:?}"
    );
}

#[test]
fn css_urls_ignore_comments_strings_and_local_fragments() {
    use og_ui::doc::at_rules::{rebase_urls, urls};
    let css = r#"/* url(fake.png) */ content: "url(fake.png)"; mask-image: url('../icons/a.svg'); filter: url(#blur); background-image: image-set("unsupported.avif" type("image/avif") 1x, "b.png" type("image/png") 1x, url(duplicate.png) 1x, url(c.png) 2x, url(zero.png) 0x);"#;
    let rebased = rebase_urls("styles/pack/nested/a.css", css);
    assert!(rebased.contains("/styles/pack/icons/a.svg"), "{rebased}");
    assert!(rebased.contains("/styles/pack/nested/b.png"), "{rebased}");
    assert!(rebased.contains("type(\"image/png\")"), "{rebased}");
    assert!(rebased.contains("url(#blur)") || rebased.contains("url(\"#blur\")"));
    assert_eq!(urls(css), ["../icons/a.svg", "b.png", "c.png"]);
    for (mime, supported) in [
        ("IMAGE/PNG; profile=test", true),
        ("image/png \t;\tprofile=test; mode=fast", true),
        (r#"image/png; note="a;b""#, true),
        (r#"image/png; note="a\"b""#, true),
        ("image/png; note=\"\"", true),
        ("image/png; note=\"ÿ\"", true),
        ("image/png;", false),
        ("image/png; bad", false),
        ("image/png; note=", false),
        ("image/png; note =test", false),
        ("image/png; note=\"unterminated", false),
        ("image/png; note=\"λ\"", false),
        ("image/avif; profile=test", false),
    ] {
        let escaped = mime.replace('\\', "\\\\").replace('"', "\\\"");
        let css = format!(
            "background: image-set(\"typed.png\" type(\"{escaped}\"), \"fallback.png\" 1x);"
        );
        assert_eq!(
            urls(&css),
            [if supported {
                "typed.png"
            } else {
                "fallback.png"
            }],
            "MIME {mime:?}"
        );
    }
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
    // A pack names its sound files as root tokens and sets them by var().
    let tokens = StylePack::parse("styles/audio", vec![("styles/audio/main.css".into(),
        ":root { --tap: url(a.wav); --tap2: url(b.wav); -og-sound: press var(--tap) var(--tap2) volume(0.28 0.34) pitch(0.98 1.02), hover none; }".into())]).unwrap();
    ui.set_style_pack(&tokens).unwrap();
    ui.restore_state(&state).unwrap();
    assert_eq!(first, play(&mut ui), "the sound set read through var()");
    for bad in [
        "press",
        "press url(a) volume(0.8 0.2)",
        "press url(a) pitch(0)",
        "missing url(a)",
    ] {
        assert!(og_ui::sound_css::parse(bad).is_err(), "{bad}");
    }
}

/// A role part wears the default dress from the role tokens, a pack restyles
/// it by setting those tokens alone, and a pack token may read any default
/// token.
#[test]
fn a_pack_restyles_role_parts_through_role_tokens_alone() {
    use og_ui::{UiDocument, Viewport, bind::NoEnv, reactive::SourceId};
    let inner = |pack: Option<&str>| {
        let doc = UiDocument::parse(
            "@ui/test/main.svelte",
            r#"<meta name="component" content="Shop"><meta name="parts" content="Shop.frame"><section id="Box" part="Shop.frame"><div id="In"></div></section><style>[part="Shop.frame"] { display: flex; width: 300px; } #In { width: stretch; height: 10px; }</style>"#,
        )
        .unwrap();
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(doc);
        if let Some(css) = pack {
            let pack = StylePack::parse(
                "styles/test",
                vec![("styles/test/main.css".into(), css.into())],
            )
            .unwrap();
            ui.set_style_pack(&pack).unwrap();
        }
        ui.mount_document(
            SourceId::new("test", "test"),
            "Shop",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        ui.begin(Default::default(), Viewport::new(800.0, 600.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        ui.elements()
            .iter()
            .find(|e| e.path.ends_with("/In"))
            .unwrap()
            .width
    };
    let default = inner(None);
    assert!(
        default < 300.0,
        "the default frame pads its content: {default}"
    );
    assert_eq!(
        inner(Some(":root { --frame-pad: 21px; --frame-rim-width: 4px; }")),
        250.0
    );
    assert_eq!(
        inner(Some(
            ":root { --frame-pad: calc(var(--font-weight-bold) * 0.01px); --frame-rim-width: 0px; }"
        )),
        286.0
    );
}
