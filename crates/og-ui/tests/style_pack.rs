use og_ui::{Ui, UiDesc, dress::PartRef, style_pack::StylePack};

#[test]
fn pack_selectors_cross_components_but_document_selectors_stay_local() {
    use og_ui::{
        Viewport,
        bind::NoEnv,
        reactive::{SourceId, wire},
    };
    use serde_json::json;
    let mut ui = Ui::new(UiDesc::default());
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
        let doc = ui.parse_document(path, text, Vec::new()).unwrap();
        ui.define_document(doc);
    }
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
    use og_ui::{Viewport, bind::NoEnv, reactive::SourceId};
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
        let shown = |pack: Option<&StylePack>| {
            let mut ui = Ui::new(UiDesc::default());
            if let Some(pack) = pack {
                ui.set_style_pack(pack).unwrap();
            }
            let doc = ui.parse_document(source, &text, Vec::new()).unwrap();
            ui.define_document(doc);
            ui.mount_document(
                SourceId::new("test", "test"),
                "test",
                Default::default(),
                Default::default(),
                &NoEnv,
            )
            .unwrap();
            ui
        };
        let mut ui = shown(Some(&pack));
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
            // Without the pack its own layer stands.
            assert_eq!(width(&mut shown(None), 600.0), 91.0);
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

/// A pack's CSS is read once and shared by every document it dresses, so
/// what one document's own `<style>` declares must stay that document's:
/// each gets the pack's keyframes and rules, and its own on top. Ways it
/// could fail: a document's keyframes or layers left in the shared sheet
/// (the next document animates with them), a document's own keyframes
/// replacing nothing because the pack's were kept apart, or a document's
/// rule losing to the pack's lower layer.
#[test]
fn documents_share_a_pack_s_rules_and_keep_their_own() {
    use og_ui::UiDocument;
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            "@keyframes drift { to { opacity: 0.5; } }\n.card { width: 30px; height: 10px; animation: drift 2s; }".into(),
        )],
    )
    .unwrap();
    let parse = |source: &str, text: &str| pack.parse_document(source, text, Vec::new()).unwrap();
    let first = parse(
        "ui/first.svelte",
        r#"<meta name="component" content="First"><div id="A" class="card"></div>
<style>
@layer extra { .card { height: 44px; } }
@keyframes sway { to { opacity: 0; } }
@keyframes drift { to { opacity: 0.25; } }
.card { width: 61px; }
</style>"#,
    );
    let second = parse(
        "ui/second.svelte",
        r#"<meta name="component" content="Second"><div id="B" class="card"></div>"#,
    );
    let keyframes = |doc: &UiDocument| -> Vec<String> {
        let mut names: Vec<String> = doc.sheet.keyframes.iter().map(|(k, _)| k.clone()).collect();
        names.retain(|n| n == "drift" || n == "sway");
        names.sort();
        names
    };
    assert_eq!(keyframes(&first), ["drift", "sway"]);
    assert_eq!(
        keyframes(&second),
        ["drift"],
        "the first document's are its own"
    );
    assert!(first.sheet.layers.iter().any(|l| l == "extra"));
    assert!(!second.sheet.layers.iter().any(|l| l == "extra"));
    let mut ui = Ui::new(UiDesc::default());
    ui.set_style_pack(&pack).unwrap();
    ui.define_document(first);
    ui.define_document(second);
    use og_ui::{Viewport, bind::NoEnv, reactive::SourceId};
    for name in ["First", "Second"] {
        ui.mount_document(
            SourceId::new("test", name),
            name,
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
    }
    ui.begin(Default::default(), Viewport::new(300.0, 300.0));
    ui.build_retained_documents(&NoEnv).unwrap();
    ui.end();
    // A mount's root is at its mount's name.
    let size = |id: &str| {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with(id))
            .unwrap_or_else(|| panic!("no element {id}"));
        (e.width, e.height)
    };
    // The document's unlayered rule and its own layer win over the pack's.
    assert_eq!(size("First"), (61.0, 44.0));
    assert_eq!(size("Second"), (30.0, 10.0));
}

/// A `<button>` is its plate (docs/ui.md §3.2): its own padding is above
/// the style's `button::part(base)` padding, under a pack as without one.
/// Ways it could fail: the pack's part rules read as the document's own,
/// so the pack's inline padding (`--button-pad-inline`) beats the button's.
#[test]
fn a_buttons_own_padding_is_above_a_packs_part_rules() {
    use og_ui::{Viewport, bind::NoEnv, reactive::SourceId};
    let face = |pack: Option<&StylePack>| {
        let mut ui = Ui::new(UiDesc::default());
        if let Some(pack) = pack {
            ui.set_style_pack(pack).unwrap();
        }
        let doc = ui
            .parse_document(
                "ui/plate.svelte",
                r#"<meta name="component" content="Plate"><div id="Root"><button id="B"><div id="Face"></div></button></div>
<style>#Root { width: 200px; } #B { width: stretch; padding: 2px; } #Face { width: stretch; height: 10px; }</style>"#,
                Vec::new(),
            )
            .unwrap();
        ui.define_document(doc);
        ui.mount_document(
            SourceId::new("test", "plate"),
            "Plate",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        ui.begin(Default::default(), Viewport::new(300.0, 300.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        let face = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Face"))
            .unwrap();
        face.width
    };
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --button-pad-inline: 24px; }".into(),
        )],
    )
    .unwrap();
    assert_eq!(face(None), 196.0, "the default style");
    assert_eq!(face(Some(&pack)), 196.0, "under a pack");
}

/// A rule ranks by its tree context before its layer, as CSS Cascade 5
/// §6.1 ranks a shadow tree's: the default style and a pack are adopted
/// into every tree and rank there by layer, a button's own box
/// declarations are its plate's over the style's `::part()` rules, and a
/// document's own `::part()` rule reaches into the control over them;
/// `!important` reverses each. Ways it could fail: a rule ranked by the
/// list it was appended to (26968a77e), the plate laid over every rule
/// whatever its importance, or a built-in component's own rules taken as
/// a game's, above the pack.
#[test]
fn a_rule_ranks_by_its_tree_context_then_its_layer() {
    use og_ui::{Viewport, bind::NoEnv, reactive::SourceId};
    // The face inside a 200px button, so 200 less twice the padding won.
    let face = |source: &str, css: &str, pack: &str| {
        let mut ui = Ui::new(UiDesc::default());
        let pack = StylePack::parse(
            "styles/test",
            vec![("styles/test/main.css".into(), pack.into())],
        )
        .unwrap();
        ui.set_style_pack(&pack).unwrap();
        let doc = ui
            .parse_document(
                source,
                &format!(
                    r#"<meta name="component" content="Plate"><div id="Root"><button id="B"><div id="Face"></div></button></div>
<style>#Root {{ width: 200px; }} #B {{ width: stretch; }} #Face {{ width: stretch; height: 10px; }} {css}</style>"#
                ),
                Vec::new(),
            )
            .unwrap();
        ui.define_document(doc);
        ui.mount_document(
            SourceId::new("test", "plate"),
            "Plate",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        ui.begin(Default::default(), Viewport::new(300.0, 300.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Face"))
            .unwrap()
            .width
    };
    let (game, builtin) = ("ui/plate.svelte", "@ui/plate.svelte");
    let pack = "button::part(base) { padding: 24px; }";
    let pack_important = "button::part(base) { padding: 24px !important; }";
    let pack_face = "button::part(base) { padding: 24px; } #Face { width: 100px; }";
    let pack_face_important =
        "button::part(base) { padding: 24px; } #Face { width: 100px !important; }";
    for (case, source, css, pack, want) in [
        (
            "a game's button over the pack",
            game,
            "#B { padding: 2px; }",
            pack,
            196.0,
        ),
        (
            "a built-in component's button over the pack",
            builtin,
            "#B { padding: 2px; }",
            pack,
            196.0,
        ),
        (
            "the pack's !important over a button's own",
            game,
            "#B { padding: 2px; }",
            pack_important,
            152.0,
        ),
        (
            "the pack's !important over a button's own !important",
            game,
            "#B { padding: 2px !important; }",
            pack_important,
            152.0,
        ),
        (
            "the pack over a built-in component's own rule",
            builtin,
            "",
            pack_face,
            100.0,
        ),
        (
            "a game's own rule over the pack",
            game,
            "",
            pack_face,
            152.0,
        ),
        (
            "a built-in component's !important over the pack's",
            builtin,
            "#Face { width: stretch !important; }",
            pack_face_important,
            152.0,
        ),
        (
            "a document's ::part() over its button's own",
            game,
            "#B { padding: 2px; } #B::part(base) { padding: 10px; }",
            pack,
            180.0,
        ),
        (
            "a button's own !important over its document's ::part()",
            game,
            "#B { padding: 2px !important; } #B::part(base) { padding: 10px !important; }",
            pack,
            196.0,
        ),
    ] {
        assert_eq!(face(source, css, pack), want, "{case}");
    }
}

/// A role part wears the default dress from the role tokens, a pack restyles
/// it by setting those tokens alone, and a pack token may read any default
/// token.
#[test]
fn a_pack_restyles_role_parts_through_role_tokens_alone() {
    use og_ui::{Viewport, bind::NoEnv, reactive::SourceId};
    let inner = |pack: Option<&str>| {
        let mut ui = Ui::new(UiDesc::default());
        if let Some(css) = pack {
            let pack = StylePack::parse(
                "styles/test",
                vec![("styles/test/main.css".into(), css.into())],
            )
            .unwrap();
            ui.set_style_pack(&pack).unwrap();
        }
        let doc = ui.parse_document(
            "@ui/test/main.svelte",
            r#"<meta name="component" content="Shop"><meta name="parts" content="Shop.frame"><section id="Box" part="Shop.frame"><div id="In"></div></section><style>[part="Shop.frame"] { display: flex; width: 300px; } #In { width: stretch; height: 10px; }</style>"#,
            Vec::new(),
        )
        .unwrap();
        ui.define_document(doc);
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

#[test]
fn a_packs_light_block_applies_under_the_light_scheme() {
    use og_ui::{Scheme, Theme, Viewport, bind::NoEnv, reactive::SourceId};
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --size: 73px; }\n:root[data-scheme=light] { --size: 41px; }\ndiv { width: var(--size); height: 25px; }"
                .into(),
        )],
    )
    .unwrap();
    for source in ["@ui/test/main.svelte", "ui/game.svelte"] {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_style_pack(&pack).unwrap();
        let doc = ui
            .parse_document(
                source,
                r#"<meta name="screen" content="test"><div><div id="Box"></div></div>"#,
                Vec::new(),
            )
            .unwrap();
        ui.define_document(doc);
        ui.mount_document(
            SourceId::new("test", "test"),
            "test",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        for (scheme, expected) in [
            (Scheme::Dark, 73.0),
            (Scheme::Light, 41.0),
            (Scheme::Dark, 73.0),
        ] {
            ui.set_theme(Theme::new(scheme));
            ui.begin(Default::default(), Viewport::new(600.0, 400.0));
            ui.build_retained_documents(&NoEnv).unwrap();
            ui.end();
            let width = ui
                .elements()
                .iter()
                .find(|e| e.path.ends_with("/Box"))
                .unwrap()
                .width;
            assert_eq!(width, expected, "{source} {scheme:?}");
        }
    }
}

#[test]
fn a_headings_flourish_part_draws_its_css_box_beside_the_line() {
    use og_ui::{DrawCmd, Viewport, bind::NoEnv, reactive::SourceId};
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            "h1::part(flourish) { background: #ff0000; width: 40px; height: 4px; }".into(),
        )],
    )
    .unwrap();
    let mut ui = crate::support::ui_with_font(crate::support::test_font());
    ui.set_style_pack(&pack).unwrap();
    let doc = ui
        .parse_document(
            "ui/game.svelte",
            r#"<meta name="screen" content="test"><div><h1>Title</h1><p>body</p></div>"#,
            Vec::new(),
        )
        .unwrap();
    ui.define_document(doc);
    ui.mount_document(
        SourceId::new("test", "test"),
        "test",
        Default::default(),
        Default::default(),
        &NoEnv,
    )
    .unwrap();
    ui.begin(Default::default(), Viewport::new(600.0, 400.0));
    ui.build_retained_documents(&NoEnv).unwrap();
    ui.end();
    let items = &ui.frame().items;
    let title = items
        .iter()
        .find_map(|item| match item.cmd.unstyled() {
            DrawCmd::Text { text, bounds, .. } if text == "Title" => Some(*bounds),
            _ => None,
        })
        .unwrap();
    // A start-aligned heading takes the flourish after its line only.
    let boxes: Vec<_> = items
        .iter()
        .filter_map(|item| match &item.cmd {
            DrawCmd::Shape { shape, .. }
                if shape.paint == og_draw::UiPaint::Solid([1.0, 0.0, 0.0, 1.0]) =>
            {
                Some((shape.center, shape.half))
            }
            _ => None,
        })
        .collect();
    assert_eq!(boxes.len(), 1, "{boxes:?}");
    let (center, half) = boxes[0];
    assert_eq!((half.x, half.y), (20.0, 2.0));
    assert!(
        center.x - half.x > title.x + title.width,
        "after the line: {center:?} {title:?}"
    );
    assert!(
        (center.y - (title.y + title.height / 2.0)).abs() < 1.0,
        "centred on the line"
    );
}

/// A pack's own seeds (`:root { --seed-primary: … }`) are what the default
/// style's derived colours resolve against, in both schemes: the primary
/// fill a document paints follows the pack's seed, not the default's, and a
/// game's seeds (UiDesc.seeds) win over the pack's.
#[test]
fn pack_seeds_reach_the_derived_palette() {
    use og_ui::{Scheme, Srgba, Theme, Viewport, bind::NoEnv, reactive::SourceId};
    let pack = StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --seed-primary: #22c7f5; }".into(),
        )],
    )
    .unwrap();
    let red = Srgba::rgb(0xc8, 0x10, 0x2e).linear();
    for (scheme, game) in [
        (Scheme::Dark, false),
        (Scheme::Light, false),
        (Scheme::Dark, true),
        (Scheme::Light, true),
    ] {
        let mut theme = Theme::new(scheme);
        if game {
            theme.seeds.0[0] = Some([red[0], red[1], red[2]]);
        }
        let mut ui = Ui::new(UiDesc {
            theme,
            ..Default::default()
        });
        ui.set_style_pack(&pack).unwrap();
        let doc = ui
            .parse_document(
                "ui/game.svelte",
                r#"<meta name="screen" content="test"><div id="Box"></div><style>#Box { width: 23px; height: 23px; background: var(--primary); }</style>"#,
                Vec::new(),
            )
            .unwrap();
        ui.define_document(doc);
        ui.mount_document(
            SourceId::new("test", "test"),
            "test",
            Default::default(),
            Default::default(),
            &NoEnv,
        )
        .unwrap();
        ui.begin(Default::default(), Viewport::new(300.0, 300.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        let fill = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .find(|(r, _)| r.width == 23.0)
            .expect("the box's fill")
            .1;
        // The pack's cyan (blue and green well over red) or the game's red,
        // whatever tone the scheme moves it to.
        let [r, g, b, _] = fill;
        if game {
            assert!(
                r > g * 2.0 && r > b * 2.0,
                "{scheme:?}: {fill:?} is not the game's red"
            );
        } else {
            assert!(
                b > r * 2.0 && g > r * 2.0,
                "{scheme:?}: {fill:?} is not the pack's cyan"
            );
        }
    }
}

/// A document is parsed once, against the style pack of the UI it is
/// defined in, and nothing reads its text again: not defining it, not
/// setting the pack in effect once more (every `og.ui.configure` call of a
/// game does), not a build. The UI takes a document as it was parsed.
///
/// Ways it could fail: `define_document` reading a document again; a
/// repeated `set_style_pack` restyling the UI's documents;
/// `Ui::parse_document` reading the default style under a pack, or the
/// pack after it is cleared; a clone of the pack taken for another pack,
/// or another reading of its files for the same.
#[test]
fn a_document_is_parsed_once_against_the_uis_pack() {
    use og_ui::{Theme, UiDocument, Viewport, bind::NoEnv, reactive::SourceId};
    let files = || {
        vec![(
            "styles/test/main.css".to_string(),
            "div.box { width: 73px; }".to_string(),
        )]
    };
    let pack = StylePack::parse("styles/test", files()).unwrap();
    let text = |name: &str| {
        format!(
            "<meta name=\"component\" content=\"{name}\"><div id=\"Box\" class=\"box\"></div><style>#Box {{ height: 20px; }}</style>"
        )
    };
    let define = |ui: &mut Ui, name: &str| {
        let doc = ui
            .parse_document(&format!("ui/{name}.svelte"), &text(name), Vec::new())
            .unwrap();
        ui.define_document(doc);
    };
    let width = |ui: &mut Ui, name: &str| {
        let mount = SourceId::new("test", name);
        // Mounted once; a later reading builds the same mount again.
        let _ = ui.mount_document(mount, name, Default::default(), Default::default(), &NoEnv);
        ui.begin(Default::default(), Viewport::new(300.0, 300.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        ui.end();
        let found = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == format!("test:{name}"))
            .unwrap();
        found.width
    };
    let mut ui = Ui::new(UiDesc::default());
    assert!(ui.styled_by(None));
    ui.set_style_pack(&pack).unwrap();
    assert!(ui.styled_by(Some(&pack)) && !ui.styled_by(None));
    let names = ["A", "B", "C", "D", "E"];
    let before = UiDocument::parses();
    for name in names {
        define(&mut ui, name);
    }
    assert_eq!(UiDocument::parses() - before, 5, "one parse a document");
    for name in names {
        assert_eq!(width(&mut ui, name), 73.0, "{name} has the pack's rule");
    }
    // The pack in effect set again, a clone as a host holds it, and
    // another theme: the documents stand as they are.
    ui.set_style_pack(&pack.clone()).unwrap();
    ui.set_theme(Theme::light());
    for name in names {
        assert_eq!(width(&mut ui, name), 73.0);
    }
    assert_eq!(UiDocument::parses() - before, 5, "nothing is read again");
    // Another reading of the same files is another pack.
    assert!(!ui.styled_by(Some(&StylePack::parse("styles/test", files()).unwrap())));
    // A document parsed with no pack is taken as it is too.
    ui.define_document(UiDocument::parse("ui/Plain.svelte", &text("Plain")).unwrap());
    let plain = width(&mut ui, "Plain");
    assert_ne!(plain, 73.0, "it carries the default style's rules only");
    // The pack cleared: a document parsed from now on has no rule of it,
    // and those defined keep theirs.
    ui.clear_style_pack();
    assert!(ui.styled_by(None));
    define(&mut ui, "After");
    assert_eq!(width(&mut ui, "After"), plain);
    assert_eq!(width(&mut ui, "A"), 73.0);
    assert_eq!(UiDocument::parses() - before, 7);
}
