//! The sample screen rendered headless through og-render and compared with
//! `references/<backend-tier>/sample-screen.png` (og-tests' tolerance: a
//! channel off by more than 32/255 is an outlier; fail above 0.5 % outliers
//! or RGB RMSE 0.01). `OG_UI_CAPTURE=1` records this tier's reference. A tier
//! without its own reference is compared with the `metal` one and says so.
//! The text-fields-2 scene (`support::text_fields_shot`) is judged the same
//! way, with the scroll clipping, glow and glass checks below.

use crate::support;
use og_ui::prelude::*;

use og_core::glam::{Vec2, Vec3};
use og_render::{Background, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap};
use og_ui::*;
use std::path::PathBuf;
use support::*;

#[test]
#[ignore = "requires GPU"]
fn retained_documents_match_immediate_pixels_after_updates_and_restore() {
    use og_ui::reactive::{SourceId, wire};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    struct Env(Value);
    impl bind::Env for Env {
        fn lookup(&self, name: &str) -> Option<Value> {
            self.0.get(name.trim_start_matches('$')).cloned()
        }
    }
    let apply = |ui: &mut Ui, command: Value| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let document = UiDocument::parse(
        "retained.svelte",
        r#"
        <meta name="component" content="retained">
        <column id="Root" gap="12">
            <text id="Title">{$title}</text>
            <label>Enable captions <input type="checkbox" switch id="Choice" checked={$enabled} width={$enabled ? 300 : 240}></label>
            <label><input type="checkbox" id="Check" checked={$enabled}> Caption sound effects</label>
            <label><input type="radio" id="Radio" checked={$enabled}> Descriptive captions</label>
            <tabs id="Tabs" items="Off On" selected={$enabled ? 1 : 0} />
            <select id="Select" value={$enabled ? "on" : "off"}><option value="off">Off</option><option value="on">On</option></select>
            <text id="Details">Captions follow the selected language.</text>
        </column>
        <style>
            #Root { padding: 24px; }
            #Title { font: var(--text-xl); }
        </style>
    "#,
    )
    .unwrap();
    for (width, height) in [(640, 480), (390, 844)] {
        let mut scene = RenderScene::new();
        scene.lighting.environment.background = Background::Color(Vec3::splat(0.008));
        let font = add_test_font(&mut scene);
        let mut retained = ui_with_font(font.clone());
        let mut immediate = ui_with_font(font);
        retained.define_document(document.clone());
        immediate.define_document(document.clone());
        let title = SourceId::new("test", "title");
        let enabled = SourceId::new("test", "enabled");
        for (source, initial) in [
            (title.clone(), json!("Caption settings")),
            (enabled.clone(), json!(false)),
        ] {
            apply(
                &mut retained,
                json!({"op":"signal", "source":source, "initial":initial}),
            );
        }
        retained
            .mount_document(
                SourceId::new("test", "screen"),
                "retained",
                BTreeMap::from([
                    ("title".into(), title.clone()),
                    ("enabled".into(), enabled.clone()),
                ]),
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .unwrap();
        let mut saved = (Vec::new(), Vec::new());
        for (state, label, checked) in [
            ("default", "Caption settings", false),
            ("updated", "Captions enabled", true),
            ("restored", "Caption settings", false),
        ] {
            if state == "restored" {
                retained.restore_state(&saved.0).unwrap();
                immediate.restore_state(&saved.1).unwrap();
            } else {
                for (source, value) in [
                    (title.clone(), json!(label)),
                    (enabled.clone(), json!(checked)),
                ] {
                    apply(
                        &mut retained,
                        json!({"op":"set", "source":source, "value":value}),
                    );
                }
            }
            let env = Env(json!({"title":label,"enabled":checked}));
            for _ in 0..30 {
                retained.begin(
                    InputFrame::default(),
                    Viewport::new(width as f32, height as f32),
                );
                retained.build_retained_documents(&env).unwrap();
                retained.end();
                immediate.begin(
                    InputFrame::default(),
                    Viewport::new(width as f32, height as f32),
                );
                immediate.component("test:screen", "retained", &mut Value::Null, &env);
                immediate.end();
            }
            let mut pixels = Vec::new();
            for ui in [&immediate, &retained] {
                let mut overlay = og_render::Overlay::new(width as f32, height as f32, 1.0);
                ui.frame().emit(&EmitDesc::default(), &mut overlay);
                scene.ui.overlay = Some(overlay);
                scene.look.post.tonemap = Tonemap::None;
                renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
                pixels.push(renderer.read_offscreen().unwrap().pixels);
            }
            image::save_buffer(
                out_dir().join(format!("retained-{state}-{width}.png")),
                &pixels[1],
                width,
                height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            assert!(
                pixels[0] == pixels[1],
                "pixel difference: {state} {width}x{height}"
            );
            if state == "default" {
                saved = (retained.save_state(), immediate.save_state());
            }
        }
    }
}

#[test]
#[ignore = "requires GPU"]
fn core_documents_preserve_screen_pixels() {
    struct Env {
        vm: serde_json::Value,
        messages: std::collections::BTreeMap<String, String>,
        art: [og_render::TextureHandle; 3],
    }
    impl bind::Env for Env {
        fn lookup(&self, root: &str) -> Option<serde_json::Value> {
            self.vm.get(root.trim_start_matches('$')).cloned()
        }
        fn translate(&self, key: &str) -> Option<String> {
            self.messages.get(key).cloned()
        }
        fn texture(&self, handle: f64) -> Option<og_render::TextureHandle> {
            self.art.get(handle as usize).copied()
        }
    }
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui/screens");
    let mut failures = Vec::new();
    for name in [
        "calibration",
        "creator-tools",
        "credits",
        "game-over",
        "hud-editor",
        "leaderboard",
        "loading",
        "main-menu",
        "mode-select",
        "pause",
        "results",
        "save-load",
        "settings",
        "splash",
        "title",
        "tutorial",
    ] {
        let fixtures =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core-screens");
        let mut source = std::fs::read_to_string(directory.join(format!("{name}.svelte"))).unwrap();
        if matches!(name, "settings" | "creator-tools" | "hud-editor") {
            source.push_str(
                &std::fs::read_to_string(fixtures.join(format!("{name}.svelte"))).unwrap(),
            );
        }
        let mut new = UiDocument::parse(&format!("{name}.svelte"), &source).unwrap();
        // Supply art through the same authored asset props as a game. Keep
        // placeholder-only captures separate so an empty slot cannot hide here.
        let art_nodes: Vec<_> = new
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.kind == doc::NodeKind::Element("art"))
            .map(|(i, node)| (i, node.id == "Logo" || name == "splash"))
            .collect();
        for (i, logo) in art_nodes {
            let handle = if logo {
                "1"
            } else if matches!(name, "results" | "leaderboard") {
                "2"
            } else {
                "0"
            };
            new.set_prop(i, "texture", handle).unwrap();
            if logo {
                new.set_prop(i, "ratio", "3 1").unwrap();
            }
        }
        let table = match og_i18n::StringTable::parse(
            &std::fs::read_to_string(directory.join(format!("{name}.en.ogt"))).unwrap(),
        ) {
            Ok(table) => table,
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                continue;
            }
        };
        let messages: std::collections::BTreeMap<_, _> = table
            .entries
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        let base = new
            .fixtures
            .first()
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let cases: Vec<_> = if name == "calibration" {
            ["brightness", "offset", "controller"]
                .into_iter()
                .map(|step| {
                    let mut vm = base.clone();
                    vm["step"] = step.into();
                    (step.to_string(), vm)
                })
                .collect()
        } else {
            vec![("default".into(), base)]
        };
        for (case, vm) in cases {
            for (width, height, coarse_pointer) in [(1280u32, 720u32, false), (390, 844, true)] {
                let mut scene = RenderScene::new();
                let font = add_test_font(&mut scene);
                let art = ["coast.png", "logo.png", "thumbnail.png"].map(|file| {
                    let image = image::open(fixtures.join(file)).unwrap().to_rgba8();
                    scene.add_texture(og_render::TextureData {
                        width: image.width(),
                        height: image.height(),
                        rgba: image.into_raw(),
                        color_space: og_render::ColorSpace::Srgb,
                        mips: og_render::Mips::Generate,
                    })
                });
                let mut ui = ui_with_font(font);
                ui.set_style_name("conversion");
                ui.define_document(new.clone());
                ui.push_screen(name, vm.clone()).unwrap();
                let env = Env {
                    vm: vm.clone(),
                    messages: messages.clone(),
                    art,
                };
                let viewport = Viewport {
                    coarse_pointer,
                    ..Viewport::new(width as f32, height as f32)
                };
                for _ in 0..30 {
                    ui.begin(InputFrame::default(), viewport);
                    ui.build_layers(&env);
                    ui.end();
                }
                let mut overlay = og_render::Overlay::new(width as f32, height as f32, 1.0);
                ui.frame().emit(&EmitDesc::default(), &mut overlay);
                scene.ui.overlay = Some(overlay);
                scene.look.post.tonemap = Tonemap::None;
                let Rendered { image, info, .. } = render_frame(&scene, &sized(width, height));
                let filename = format!("{name}-{case}-{width}.png");
                save_actual(&filename, &image);
                let reference_path = reference_dir(og_tests::compare::RECORDING_TIER)
                    .join("documents")
                    .join(&filename);
                if capturing() && !reference_path.exists() {
                    record_reference(&reference_path, width, height, &image.pixels);
                }
                let reference = image::open(&reference_path).unwrap().to_rgba8();
                assert_eq!(reference.dimensions(), (width, height));
                let sum: f64 = reference
                    .as_raw()
                    .iter()
                    .zip(&image.pixels)
                    .enumerate()
                    .filter(|(i, _)| i % 4 != 3)
                    .map(|(_, (&a, &b))| (f64::from(a) - f64::from(b)).powi(2))
                    .sum();
                let rmse = (sum / (f64::from(width) * f64::from(height) * 3.0)).sqrt() / 255.0;
                println!("conversion {name} {case} {width}x{height}: RMSE {rmse:.8}");
                // The tier the frames were recorded on preserves them byte
                // for byte; another tier is judged against them under the
                // cross-platform tolerance (`tolerance_for`).
                let preserved = reference.as_raw() == &image.pixels
                    || (info.backend_tier() != og_tests::compare::RECORDING_TIER
                        && compare(&image.pixels, reference.as_raw(), &info, &reference_path)
                            .is_ok());
                if !preserved && capturing() {
                    record_reference(&reference_path, width, height, &image.pixels);
                } else if !preserved {
                    failures.push(format!("{name}/{case}/{width}: {rmse:.8}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "moved frames: {}", failures.join(", "));
}

#[test]
#[ignore = "requires GPU"]
fn missing_art_slot_kinds_have_intentional_placeholders() {
    use slot::SlotKind as K;
    let mut cases = Vec::new();
    for &kind in K::ALL {
        let (width, height) = match kind {
            K::KeyArt | K::MenuBackground => (640.0, 360.0),
            K::Capsule => (150.0, 240.0),
            K::CharacterSplash | K::FullBody => (135.0, 240.0),
            K::VnSprite => (120.0, 240.0),
            K::Banner => (400.0, 200.0),
            K::PlayerBanner => (400.0, 100.0),
            K::Logo => (240.0, 80.0),
            K::AppIcon
            | K::ItemIcon
            | K::AbilityIcon
            | K::Emblem
            | K::Achievement
            | K::Emote
            | K::Currency
            | K::MapGlyph
            | K::TalentIcon
            | K::IconSet => (40.0, 40.0),
            _ => {
                let ratio = kind.info().aspects.first().map_or(1.0, |a| a.ratio());
                if ratio >= 1.0 {
                    (200.0, 200.0 / ratio)
                } else {
                    (200.0 * ratio, 200.0)
                }
            }
        };
        cases.push((kind, width, height));
        if matches!(kind, K::KeyArt | K::MenuBackground) {
            cases.push((kind, 180.0, 390.0));
        }
    }
    // Shelf-pack the actual slot rectangles, leaving label space outside them.
    let (mut x, mut y, mut row_height) = (16.0f32, 16.0f32, 0.0f32);
    let mut placements = Vec::new();
    for (kind, width, height) in cases {
        let cell_width = width.max(150.0);
        if x + cell_width > 1384.0 {
            x = 16.0;
            y += row_height + 24.0;
            row_height = 0.0;
        }
        placements.push((kind, width, height, x, y));
        x += cell_width + 20.0;
        row_height = row_height.max(height + 48.0);
    }
    let vp = Viewport::new(1400.0, y + row_height + 16.0);
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    theme.unit_pin = 1.0;
    ui.set_theme(theme);
    ui.set_style_name("placeholder-review");
    ui.begin(InputFrame::default(), vp);
    for (index, (kind, width, height, x, y)) in placements.iter().copied().enumerate() {
        ui.open_with(
            format!("Cell{index}"),
            Element {
                direction: Direction::Column,
                width: Size::Fixed(width.max(150.0)),
                height: Size::Fixed(height + 48.0),
                ..Element::default()
            },
            Props {
                position: Position::Absolute,
                insets: Edges {
                    start: Some(x),
                    top: Some(y),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        ui.label(
            "Label",
            &format!("{}\n{width:.0} x {height:.0}", kind.name()),
            LabelDesc::default(),
        );
        ui.art(
            "Art",
            &ArtDesc {
                spec: slot::SlotSpec::new(kind.name(), kind),
                width: Size::Fixed(width),
                height: Size::Fixed(height),
                texture: None,
                size: None,
                label: String::new(),
                offset: [0.0; 2],
                nearest: false,
            },
        );
        ui.close();
    }
    ui.end();
    for (index, (_, width, height, _, _)) in placements.iter().enumerate() {
        let rect = ui.element(&format!("Cell{index}/Art")).unwrap();
        assert!(
            (rect.width - width).abs() <= 1.0 && (rect.height - height).abs() <= 1.0,
            "pixel-snapped slot {index}: {rect:?}, expected {width}x{height}"
        );
        assert!(rect.x + rect.width <= vp.width && rect.y + rect.height <= vp.height);
        let (cx, cy) = rect.center();
        let mark = ui
            .frame()
            .items
            .iter()
            .find_map(|item| match round_fill(&item.cmd) {
                Some((rect, radius))
                    if radius >= rect.width.min(rect.height) / 2.0
                        && (rect.center().0 - cx).abs() <= 0.5
                        && (rect.center().1 - cy).abs() <= 0.5 =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no pixel-snapped centred disc in case {index}: {rect:?}"));
        assert_eq!(mark.width, mark.height);
        assert!(mark.width <= 32.0 && mark.width <= width.min(*height) * 0.36 + 0.01);
    }
    let shot = shot(ui, scene, vp);
    let frame = render_frame(&shot.scene, &shot.desc);
    matches_reference(
        "art-slot-placeholders",
        &frame.image,
        &frame.info,
        Against::Recorded,
    );
}

#[test]
#[ignore = "requires GPU"]
fn document_spans_match_core_pixels_and_restore_motion() {
    let vp = Viewport::new(640.0, 180.0);
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    theme.unit_pin = 1.0;
    theme.type_ramp.body.size = 26.0;
    theme.type_ramp.body.line_height = 36.0;
    ui.set_theme(theme);
    ui.define_document(
        UiDocument::parse(
            "dialogue.svelte",
            r#"
<meta name="component" content="dialogue">
<text id="Line" spans={spans}>{body}</text>
<style>
text { width: 560px; }
text::part(speaker) { color: var(--highlight); font-weight: bold; }
text::part(motion) { color: var(--danger); -og-text-motion: wave; }
</style>
"#,
        )
        .unwrap(),
    );
    let spans = vec![
        TextSpan {
            name: "speaker".into(),
            position: 0,
            length: 4,
            properties: Default::default(),
        },
        TextSpan {
            name: "motion".into(),
            position: 5,
            length: 18,
            properties: Default::default(),
        },
    ];
    let body = "Ada: Keep moving slowly. [b] remains plain text.";
    let draw = |ui: &mut Ui, document: bool| {
        ui.begin(InputFrame::default(), vp);
        if document {
            ui.component(
                "Line",
                "dialogue",
                &mut serde_json::json!({"body": body, "spans": spans}),
                &bind::NoEnv,
            );
        } else {
            ui.rich_text_spans(
                "Line",
                body,
                &spans,
                RichTextDesc {
                    width: Size::Fixed(560.0),
                    ..Default::default()
                },
                |span| match span.name.as_str() {
                    "speaker" => TextSpanStyle {
                        color: Some(paint::ColorRef::token("highlight")),
                        bold: Some(true),
                        weight: Some(700),
                        ..Default::default()
                    },
                    "motion" => TextSpanStyle {
                        color: Some(paint::ColorRef::token("danger")),
                        motion: Some(markup::SpanFx::Wave),
                        ..Default::default()
                    },
                    _ => Default::default(),
                },
            );
        }
        ui.end();
    };
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 640,
        height: 180,
        ..Default::default()
    })
    .unwrap();
    let mut capture = |ui: &Ui| {
        let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        scene.look.post.tonemap = Tonemap::None;
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        renderer.read_offscreen().unwrap()
    };
    for _ in 0..13 {
        draw(&mut ui, true);
    }
    let saved = ui.save_state();
    draw(&mut ui, true);
    let document = capture(&ui);
    ui.restore_state(&saved).unwrap();
    draw(&mut ui, false);
    assert_eq!(
        document.pixels,
        capture(&ui).pixels,
        "document cascade must match core rendering, RMSE 0"
    );
    ui.restore_state(&saved).unwrap();
    draw(&mut ui, true);
    assert_eq!(document.pixels, capture(&ui).pixels);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let path = out_dir().join("document-rich-text.png");
    image::save_buffer(&path, &document.pixels, 640, 180, image::ColorType::Rgba8).unwrap();
    println!("{}", path.display());
    theme.reduced_motion = true;
    theme.palette.danger = Srgba::rgb(64, 218, 182);
    ui.set_theme(theme);
    draw(&mut ui, true);
    let still = capture(&ui);
    for _ in 0..13 {
        draw(&mut ui, true);
    }
    assert_eq!(still.pixels, capture(&ui).pixels);
    assert_ne!(document.pixels, still.pixels);
    let path = out_dir().join("document-rich-text-reduced.png");
    image::save_buffer(&path, &still.pixels, 640, 180, image::ColorType::Rgba8).unwrap();
    println!("{}", path.display());
}

#[test]
#[ignore = "requires GPU"]
fn structured_rich_text_draws_wraps_and_restores_mid_motion() {
    let vp = Viewport::new(880.0, 480.0);
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    theme.unit_pin = 1.0;
    ui.set_theme(theme);
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 880,
        height: 480,
        ..Default::default()
    })
    .expect("a headless adapter");
    let text = "Ada: Walk through the silent forest. Keep [b] as literal text, and keep \\paths untouched. The speaker, emphasis and motion are independent ranges in one wrapped paragraph.";
    let attrs: Vec<TextSpan> = [
        ("character", 0, 5),
        ("emphasis", 22, 13),
        ("motion", 22, 13),
        ("unknown-game-tag", 40, 8),
    ]
    .into_iter()
    .map(|(name, position, length)| TextSpan {
        name: name.into(),
        position,
        length,
        properties: Default::default(),
    })
    .collect();
    let build = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), vp);
        let t = *ui.theme();
        ui.column(
            "Review",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets::all(36.0),
                gap: 24.0,
                background: Some(t.palette.backdrop),
                ..Default::default()
            },
            |ui| {
                ui.label(
                    "Title",
                    "Structured dialogue / plain text + spans",
                    LabelDesc {
                        step: Some(t.type_ramp.xl),
                        ..Default::default()
                    },
                );
                ui.label(
                    "State",
                    if t.reduced_motion {
                        "Reduced motion / active-style token changed"
                    } else {
                        "Wave + emphasis / restored tick"
                    },
                    LabelDesc::default(),
                );
                ui.rich_text_spans(
                    "Dialogue",
                    text,
                    &attrs,
                    RichTextDesc {
                        width: Size::Fixed(620.0),
                        step: Some(TypeStep {
                            size: 26.0,
                            line_height: 36.0,
                            ..t.type_ramp.body
                        }),
                        ..Default::default()
                    },
                    |a| match a.name.as_str() {
                        "character" => TextSpanStyle {
                            bold: Some(true),
                            color: Some(paint::ColorRef::token("highlight")),
                            ..Default::default()
                        },
                        "emphasis" => TextSpanStyle {
                            bold: Some(true),
                            color: Some(paint::ColorRef::token("danger")),
                            ..Default::default()
                        },
                        "motion" => TextSpanStyle {
                            motion: Some(markup::SpanFx::Wave),
                            ..Default::default()
                        },
                        _ => TextSpanStyle::default(),
                    },
                );
                ui.label(
                    "Legend",
                    "Speaker prefix retained. Unknown attributes stay in game data.",
                    LabelDesc::default(),
                );
            },
        );
        ui.end();
    };
    let capture = |ui: &Ui, renderer: &mut Renderer, scene: &mut RenderScene| {
        let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        scene.look.post.tonemap = Tonemap::None;
        renderer.render(scene, &RenderTarget::Offscreen).unwrap();
        renderer.read_offscreen().unwrap()
    };
    for _ in 0..13 {
        build(&mut ui);
    }
    let saved = ui.save_state();
    build(&mut ui);
    let expected = capture(&ui, &mut renderer, &mut scene);
    for _ in 0..17 {
        build(&mut ui);
    }
    let changed = capture(&ui, &mut renderer, &mut scene);
    assert_ne!(
        expected.pixels, changed.pixels,
        "wave must move visible glyphs"
    );
    ui.restore_state(&saved).unwrap();
    build(&mut ui);
    let restored = capture(&ui, &mut renderer, &mut scene);
    assert_eq!(
        expected.pixels, restored.pixels,
        "GPU pixels replay after restoring mid-wave"
    );
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let path = out_dir().join("structured-rich-text.png");
    image::save_buffer(&path, &restored.pixels, 880, 480, image::ColorType::Rgba8).unwrap();
    println!("{}", path.display());
    theme.reduced_motion = true;
    theme.palette.danger = Srgba::rgb(64, 218, 182);
    ui.set_theme(theme);
    build(&mut ui);
    let still = capture(&ui, &mut renderer, &mut scene);
    for _ in 0..13 {
        build(&mut ui);
    }
    assert_eq!(still.pixels, capture(&ui, &mut renderer, &mut scene).pixels);
    let path = out_dir().join("structured-rich-text-reduced.png");
    image::save_buffer(&path, &still.pixels, 880, 480, image::ColorType::Rgba8).unwrap();
    println!("{}", path.display());
}

#[test]
#[ignore = "requires GPU"]
fn css_radius_token_changes_pixels_when_the_active_style_changes() {
    let source = "<meta name=\"screen\" content=\"tokens\" data-layer=\"hud\"><column id=\"Box\"/><style>#Box { --corner: var(--radius-lg); width: 96px; height: 64px; translate: 16px 16px; background: #ffffff; border-radius: var(--corner); }</style>";
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("tokens.ogt", source).unwrap());
    ui.push_screen("tokens", serde_json::Value::Null).unwrap();
    let vp = Viewport::new(128.0, 96.0);
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 96,
        ..RenderDesc::default()
    })
    .expect("a headless adapter");
    for radius in [0.0, 24.0] {
        let mut theme = Theme {
            unit_pin: 1.0,
            ..Theme::default()
        };
        theme.tokens.radius[og_ui::tokens::RadiusStep::Lg.index()] = radius;
        ui.set_theme(theme);
        for _ in 0..2 {
            ui.begin(InputFrame::default(), vp);
            ui.end();
        }
        let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let mut scene = RenderScene::new();
        scene.ui.overlay = Some(overlay);
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.look.post.tonemap = Tonemap::None;
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        let image = renderer.read_offscreen().unwrap();
        let red = |x: usize, y: usize| image.pixels[(y * 128 + x) * 4];
        assert!(red(64, 48) > 240, "the centre stays filled");
        if radius == 0.0 {
            assert!(red(17, 17) > 240, "the square corner is filled");
        } else {
            assert!(
                red(17, 17) < 10,
                "the changed token cuts the rounded corner"
            );
        }
        let path = out_dir().join(format!("css-radius-{radius:.0}.png"));
        image::save_buffer(
            &path,
            &image.pixels,
            image.width,
            image.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        println!("{}", path.display());
    }
}

#[test]
#[ignore = "requires GPU"]
fn keyed_virtual_content_is_measured_and_clipped_on_gpu() {
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 240,
        height: 160,
        ..Default::default()
    })
    .expect("a headless adapter");
    for grid in [false, true] {
        let mut ui = Ui::new(UiDesc {
            theme: Theme {
                unit_pin: 1.0,
                ..Default::default()
            },
            ..Default::default()
        });
        let keys: Vec<String> = (0..50).map(|i| i.to_string()).collect();
        for _ in 0..4 {
            ui.begin(InputFrame::default(), Viewport::new(240.0, 160.0));
            let range = if grid {
                ui.open_keyed_grid(
                    "Items",
                    &keys,
                    GridDesc {
                        columns: 2,
                        cell_width: 100.0,
                        width: Size::Fixed(220.0),
                        height: Some(Size::Fixed(128.0)),
                        ..Default::default()
                    },
                )
            } else {
                ui.open_keyed_list(
                    "Items",
                    &keys,
                    ListDesc {
                        row_height: 40.0,
                        width: Size::Fixed(220.0),
                        height: Some(Size::Fixed(128.0)),
                        ..Default::default()
                    },
                )
            };
            for i in range.first..range.last {
                if grid {
                    ui.grid_item(i, false);
                } else {
                    ui.list_item(i, false);
                }
                ui.open(
                    "Paint",
                    Element {
                        width: Size::Fixed(64.0),
                        height: Size::Fixed(if i % 2 == 0 { 24.0 } else { 56.0 }),
                        background: Some(Srgba::WHITE),
                        ..Default::default()
                    },
                );
                ui.close();
                ui.close();
            }
            ui.close_list();
            ui.end();
        }
        let mut overlay = og_render::Overlay::new(240.0, 160.0, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let mut scene = RenderScene::new();
        scene.ui.overlay = Some(overlay);
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.look.post.tonemap = Tonemap::None;
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        let image = renderer.read_offscreen().unwrap();
        let white = image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 240 && p[1] > 240 && p[2] > 240)
            .count();
        assert!(white > 3000, "variable content was painted");
        assert!(
            image.pixels[(140 * 240 + 40) * 4] < 10,
            "content below the scroll viewport is clipped"
        );
        let path = out_dir().join(if grid {
            "keyed-grid.png"
        } else {
            "keyed-list.png"
        });
        image::save_buffer(
            &path,
            &image.pixels,
            image.width,
            image.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        println!("{}", path.display());
    }
}

#[test]
#[ignore = "requires GPU"]
fn sample_screen_matches_its_reference_frame() {
    let (shot, s) = with_test_font(sample_screen_shot);
    let frame = shot.ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert_eq!(s.search, "pot", "the focused field took the typed text");
    assert!(
        shot.ui.element("tooltip").is_some(),
        "the tooltip shows after the pointer rests on Buy"
    );

    let rendered = render_frame(&shot.scene, &shot.desc);
    let stats = rendered.stats;

    // The whole UI is a handful of instanced draws: one atlas run and one
    // font run per painter level at most.
    let levels = frame.levels() as u32;
    println!(
        "sprites {} glyphs {} sprite draws {} levels {} missing glyphs {}",
        stats.sprites, stats.glyphs, stats.sprite_draws, levels, stats.missing_glyphs
    );
    assert!(levels <= 4, "{levels} painter levels");
    assert!(
        stats.sprite_draws <= 2 * levels,
        "{} sprite draws for {levels} levels",
        stats.sprite_draws
    );
    assert_eq!(stats.missing_glyphs, 0);

    matches_reference(
        "sample-screen",
        &rendered.image,
        &rendered.info,
        Against::OwnTier,
    );
}

/// What a scroll container draws stays inside it, per pixel: the rows it
/// scrolled half out are cut exactly at its edge (quads and text alike), and
/// nothing it holds shows outside its rectangle.
#[test]
#[ignore = "requires GPU"]
fn a_scroll_container_clips_everything_it_draws() {
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    let vp = Viewport::new(300.0, 200.0);
    let red = Srgba::rgb(230, 26, 26);
    let build = |ui: &mut Ui, input: InputFrame| {
        let t = *ui.theme();
        ui.begin(input, vp);
        ui.scroll(
            "List",
            ScrollDesc {
                width: Size::Fixed(200.0),
                height: Size::Fixed(100.0),
                gap: t.space.xs,
                ..ScrollDesc::default()
            },
            |ui| {
                for n in 0..10 {
                    ui.row(
                        ui_idi("Row", n),
                        Element {
                            width: Size::grow(),
                            padding: Insets::all(t.space.sm),
                            background: Some(red),
                            radius: 0.0,
                            ..Element::default()
                        },
                        |ui| ui.label("Name", &format!("Item {n}"), LabelDesc::default()),
                    );
                }
            },
        );
        ui.end();
    };
    build(&mut ui, InputFrame::default());
    let list = ui.element("List").expect("list").clone();
    build(
        &mut ui,
        InputFrame {
            wheel: Vec2::new(0.0, 47.0),
            ..InputFrame::hover(50.0, 50.0)
        },
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    // Rows reach past the list's edges before the clip: the cut is the GPU's.
    let (top, bottom) = (list.y, list.y + list.height);
    let shapes = &overlay.ui.shapes;
    let beyond = shapes
        .iter()
        .filter(|s| s.center.y - s.half.y < top - 0.5 || s.center.y + s.half.y > bottom + 0.5)
        .count();
    assert!(beyond > 0, "no row reaches past the list, nothing to clip");
    assert!(
        shapes.iter().all(|s| s.clip.is_some()),
        "every shape carries a clip"
    );
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let px = |x: u32, y: u32| {
        let i = ((y * image.width + x) * 4) as usize;
        [image.pixels[i], image.pixels[i + 1], image.pixels[i + 2]]
    };
    let reddish = |p: [u8; 3]| p[0] > 120 && p[1] < 80;
    let (x0, x1) = (list.x.ceil() as u32 + 4, (list.x + list.width) as u32 - 4);
    // Outside the list, above and below it: background only.
    for y in (0..top.floor() as u32).chain(bottom.ceil() as u32..image.height) {
        for x in (x0..x1).step_by(3) {
            assert!(
                !reddish(px(x, y)),
                "row colour at ({x}, {y}), outside the list {top}..{bottom}"
            );
        }
    }
    // Inside it, at the first and last pixel rows, the cut rows show.
    let row_at = |y: u32| (x0..x1).any(|x| reddish(px(x, y)));
    assert!(
        row_at(top.ceil() as u32) && row_at(bottom.floor() as u32 - 1),
        "the half-scrolled rows are cut at the list's edges, not dropped"
    );
}

/// A style's text glow is one soft halo read from the font's distance field
/// (og-render's MSDF shader), not offset copies of the text: the ink is the
/// same with and without it, the pixels just outside the glyphs take the
/// glow, it ends within the distance field's reach, each glyph costs two
/// instances (its glow, then its fill), and the frame matches
/// `references/<tier>/text-glow.png`.
///
/// Ways it could fail: the glow is drawn over the ink (the letters wash
/// out), offset copies move the ink, the halo never shows, or it reaches
/// past the distance range (where the field is flat, it would end at the
/// glyph quad's hard edge).
#[test]
#[ignore = "requires GPU"]
fn a_text_glow_is_a_halo_from_the_distance_field() {
    let render = |glow: bool| {
        let (scene, desc) = with_test_font(|f| glow_scene(f, glow, 1));
        let frame = render_frame(&scene, &desc);
        (frame.image, frame.stats, frame.info)
    };
    let (plain, plain_stats, _) = render(false);
    let (glow, glow_stats, info) = render(true);
    assert_eq!(
        glow_stats.overlay_glyphs,
        2 * plain_stats.overlay_glyphs,
        "a glowing glyph is two instances: its glow and its fill"
    );
    let (w, h) = (plain.width as usize, plain.height as usize);
    let at = |px: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [px[i] as i32, px[i + 1] as i32, px[i + 2] as i32]
    };
    let bg = at(&plain.pixels, 2, 2);
    let lum = |p: [i32; 3]| p[0] + p[1] + p[2];
    let is_ink = |x: usize, y: usize| lum(at(&plain.pixels, x, y)) - lum(bg) > 300;
    let (mut ink_same, mut ink, mut near, mut far) = (0usize, 0usize, 0usize, 0usize);
    for y in 0..h {
        for x in 0..w {
            let p = at(&plain.pixels, x, y);
            let g = at(&glow.pixels, x, y);
            let solid = x > 0
                && y > 0
                && x + 1 < w
                && y + 1 < h
                && (y - 1..=y + 1).all(|yy| (x - 1..=x + 1).all(|xx| is_ink(xx, yy)));
            if solid {
                // Solid ink (its neighbours ink too, no antialiased edge):
                // unchanged by the glow under it.
                ink += 1;
                if (0..3).all(|c| (p[c] - g[c]).abs() <= 8) {
                    ink_same += 1;
                }
            } else if p == bg {
                // Background: tinted next to the ink, untouched far from it.
                let d = (1usize..=8)
                    .find(|&r| {
                        let (x0, x1) = (x.saturating_sub(r), (x + r).min(w - 1));
                        let (y0, y1) = (y.saturating_sub(r), (y + r).min(h - 1));
                        (y0..=y1).any(|yy| (x0..=x1).any(|xx| is_ink(xx, yy)))
                    })
                    .unwrap_or(99);
                let tinted = g != bg;
                if d <= 2 && tinted {
                    near += 1;
                }
                if d > 6 && tinted {
                    far += 1;
                }
            }
        }
    }
    assert!(ink > 200, "the labels drew solid ink ({ink} px)");
    assert!(
        ink_same as f64 >= 0.95 * ink as f64,
        "the glow lies under the ink: {ink_same} of {ink} ink pixels unchanged"
    );
    assert!(
        near > 200,
        "the background next to the ink glows ({near} px)"
    );
    assert_eq!(far, 0, "nothing glows past the distance field's reach");
    matches_reference("text-glow", &glow, &info, Against::OwnTier);
}

/// A type step of the ramp.
type StepOf = fn(&TypeRamp) -> TypeStep;

/// One line of rich text rendered through og-render at a type step.
fn rich_frame(markup: &str, step: StepOf) -> (Vec<u8>, usize, usize, [u8; 3]) {
    inked_frame(markup, step, None)
}

/// [`rich_frame`] in an ink over a ground (`None`: the theme's ink over
/// its backdrop).
fn inked_frame(
    markup: &str,
    step: StepOf,
    colors: Option<(Srgba, Srgba)>,
) -> (Vec<u8>, usize, usize, [u8; 3]) {
    let vp = Viewport {
        dpr: 1.0,
        ..Viewport::new(360.0, 120.0)
    };
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        let t = *ui.theme();
        ui.column(
            "Card",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets::all(t.space.md),
                gap: t.space.md,
                background: Some(colors.map_or(t.palette.backdrop, |c| c.1)),
                ..Element::default()
            },
            |ui| {
                let step = step(&t.type_ramp);
                ui.rich_text(
                    "Line",
                    markup,
                    RichTextDesc {
                        step: Some(step),
                        color: colors.map(|c| c.0),
                        ..RichTextDesc::default()
                    },
                );
            },
        );
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = colors.map_or(ui.theme().palette.backdrop, |c| c.1).linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("read back");
    let bg = [img.pixels[0], img.pixels[1], img.pixels[2]];
    (img.pixels, img.width as usize, img.height as usize, bg)
}

/// `[b]` is heavier ink around the same glyphs at every size: never the
/// glyph quads filled. Small text from a narrow distance range (the test
/// font's 4 px at 32 px: at `text-xs` the field is too thin to thicken)
/// still turns bold: bold also draws a second copy a device pixel to the
/// right. A bold glyph advances one weight further than its plain self (a
/// bold face's added stem), so each glyph is compared with its plain
/// glyph moved by the weights of the glyphs before it.
///
/// Ways it could fail: the weight runs past the field's reach and fills
/// each glyph's quad (a box behind the digit, as the first cut drew at
/// the reading size), bold draws nothing heavier at small sizes (no
/// second copy), or it moves the glyphs further than their tracked
/// advance (a letter-spaced bold word).
#[test]
#[ignore = "requires GPU"]
fn bold_rich_text_is_heavier_ink_never_a_box() {
    let steps: [(&str, StepOf); 3] = [
        ("xs", |r| r.xs),
        ("reading", |r| r.reading),
        ("display", |r| r.display_md),
    ];
    const LINE: &str = "Slot 42 W";
    let metrics = test_font_metrics().metrics;
    let mut ui = ui_with_font(test_font_metrics());
    // Match rich_frame's viewport: its title role uses the landscape
    // breakpoint, not the desktop ramp returned before begin().
    ui.begin(InputFrame::default(), Viewport::new(360.0, 120.0));
    let theme = *ui.theme();
    for (name, step) in steps {
        let (plain, w, h, bg) = rich_frame(LINE, step);
        let (bold, ..) = rich_frame(&format!("[b]{LINE}[/b]"), step);
        // Each glyph's advance in the plain line (from the card's padding, at
        // 1 device pixel a CSS px) and how far bold moves it: a bold
        // face's added stem (og-ui's 0.056 em) for each glyph before it.
        let s = step(&theme.type_ramp);
        let weight = s.size * 0.056;
        let x0 = theme.space.md;
        let glyphs: Vec<(f32, f32, f32)> = LINE
            .char_indices()
            .filter(|(_, c)| !c.is_whitespace())
            .map(|(i, c)| {
                let left = x0 + metrics.measure_tracked(&LINE[..i], s.size, s.tracking_em);
                let end = i + c.len_utf8();
                let right = x0 + metrics.measure_tracked(&LINE[..end], s.size, s.tracking_em);
                (left, right, LINE[..i].chars().count() as f32 * weight)
            })
            .collect();
        let ink = |px: &[u8], x: usize, y: usize| {
            let i = (y * w + x) * 4;
            (0..3)
                .map(|c| (px[i + c] as i32 - bg[c] as i32).abs())
                .sum::<i32>()
                > 120
        };
        let count = |px: &[u8]| {
            (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| ink(px, x, y))
                .count()
        };
        let (p, b) = (count(&plain), count(&bold));
        assert!(p > 40, "the {name} line drew ink ({p} px)");
        assert!(
            b as f64 >= 1.15 * p as f64,
            "bold is heavier at the {name} size: {b} against {p} px"
        );
        // Every bold ink pixel lies within three pixels of its glyph's plain
        // ink, moved by its tracked advance (the added stem, 0.056 em, and
        // the one-pixel second copy).
        let near_plain = |x: i64, y: usize| {
            (y.saturating_sub(3)..=(y + 3).min(h - 1)).any(|yy| {
                (x - 3..=x + 3)
                    .any(|xx| xx >= 0 && (xx as usize) < w && ink(&plain, xx as usize, yy))
            })
        };
        let mut stray = 0;
        for y in 0..h {
            for x in 0..w {
                if ink(&bold, x, y) {
                    let xf = x as f32;
                    let near = glyphs.iter().any(|&(l, r, dx)| {
                        xf >= l + dx - 4.0
                            && xf <= r + dx + 4.0
                            && near_plain((xf - dx).round() as i64, y)
                    });
                    if !near {
                        stray += 1;
                    }
                }
            }
        }
        assert_eq!(stray, 0, "bold ink at the {name} size stays on its glyphs");
    }
}

/// Small dark text on a light ground keeps its weight: a label in the
/// neutral primary's near-black ink on its near-white fill draws, judged
/// in sRGB as a reader sees it, at least two thirds of the ink the same
/// label draws in near-white on near-black. Light on dark keeps the
/// linear blend's slightly heavier edge (the engine's established look),
/// so the pair need not be equal.
///
/// Ways it could fail: dark ink's coverage blends in linear light, which
/// keeps its antialiased edge nearly as light as the ground: the label
/// then draws half the ink of its light twin (0.49 at `text-xs`) and a
/// 14 px Play on the default style's primary reads pale grey though its
/// ink is #111113 (the kit's old main menu at 1280×720).
#[test]
#[ignore = "requires GPU"]
fn dark_small_text_on_light_keeps_its_weight() {
    let dark = Srgba::rgb(0x11, 0x11, 0x13);
    let light = Srgba::rgb(0xec, 0xec, 0xee);
    // Ink drawn: each pixel's sRGB distance from its ground, as a fraction
    // of the ink's own.
    let mass = |ink: Srgba, ground: Srgba, step: StepOf| {
        let (px, w, h, bg) = inked_frame("Play Continue 42", step, Some((ink, ground)));
        let full: f64 = (ink.rgb[0] as f64 - bg[0] as f64).abs();
        (0..w * h)
            .map(|i| (px[i * 4] as f64 - bg[0] as f64).abs() / full)
            .sum::<f64>()
    };
    let steps: [(&str, StepOf); 2] = [("xs", |r| r.xs), ("reading", |r| r.reading)];
    for (name, step) in steps {
        let on_light = mass(dark, light, step);
        let on_dark = mass(light, dark, step);
        let ratio = on_light / on_dark;
        eprintln!(
            "{name}: dark on light {on_light:.0}, light on dark {on_dark:.0}, ratio {ratio:.3}"
        );
        assert!(on_dark > 40.0, "the {name} line drew ink ({on_dark:.0})");
        assert!(
            (0.65..=1.15).contains(&ratio),
            "at the {name} size dark-on-light ink weighs {ratio:.2} of light-on-dark"
        );
    }
}

/// The stripes scene (`stripes_scene`) under `glass` panels, rendered and
/// read back.
fn stripes_frame(glass: &[og_render::Glass]) -> (Vec<u8>, usize, usize, og_render::FrameStats) {
    let (scene, desc) = stripes_scene(glass);
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("read back");
    (img.pixels, desc.width as usize, desc.height as usize, stats)
}

/// A glass panel shows the frame under it blurred, inside its rounded
/// rectangle only, and leaves every other pixel exactly as it was: the
/// fine stripes under it smooth to their average while the stripes beside
/// it keep their contrast, its corners stay sharp stripes, and a tint
/// covers the blur by its alpha.
///
/// Ways it could fail: the frame moves through the glass source and back
/// changed (the copy is not exact), the blur never reaches the panel
/// (sharp stripes inside), it spills past the rounded rectangle, the
/// pyramid is not decoded (a darker or brighter average), or the tint is
/// ignored.
#[test]
#[ignore = "requires GPU"]
fn a_glass_panel_blurs_the_frame_under_it_and_nothing_else() {
    let panel = glass_panel();
    let (plain, w, h, _) = stripes_frame(&[]);
    let (glass, _, _, stats) = stripes_frame(&[panel]);
    let px = |p: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [p[i] as i32, p[i + 1] as i32, p[i + 2] as i32]
    };
    let lum = |c: [i32; 3]| (c[0] + c[1] + c[2]) / 3;
    // Outside the panel, every pixel is the plain frame's.
    let mut moved = 0;
    for y in 0..h {
        for x in 0..w {
            let inside = (80..240).contains(&x) && (40..160).contains(&y);
            if !inside && px(&plain, x, y) != px(&glass, x, y) {
                moved += 1;
            }
        }
    }
    assert_eq!(moved, 0, "the glass leaves the frame outside it untouched");
    // The plain stripes run from black to white along a row.
    let row = |p: &[u8], y: usize, x0: usize, x1: usize| -> (i32, i32, f64) {
        let v: Vec<i32> = (x0..x1).map(|x| lum(px(p, x, y))).collect();
        let mean = v.iter().sum::<i32>() as f64 / v.len() as f64;
        (*v.iter().min().unwrap(), *v.iter().max().unwrap(), mean)
    };
    let (lo, hi, plain_mean) = row(&plain, 100, 120, 200);
    assert!(lo < 10 && hi > 245, "the stripes are sharp ({lo}..{hi})");
    let (glo, ghi, glass_mean) = row(&glass, 100, 120, 200);
    assert!(
        ghi - glo < 24,
        "under the glass the stripes blur to their average ({glo}..{ghi})"
    );
    // The average of linear light: half-white in linear is ~188 in sRGB.
    assert!(
        (glass_mean - 188.0).abs() < 16.0,
        "the blur averages light, not encoded values ({glass_mean:.0}; the plain row's mean {plain_mean:.0})"
    );
    // The rounded corner: its outer pixel shows the sharp stripes.
    assert_eq!(
        px(&glass, 81, 41),
        px(&plain, 81, 41),
        "outside the rounded corner the frame shows as it is"
    );
    assert!(stats.overlay_sprites >= 1, "the panel is drawn");
    // A tint covers the blur by its alpha.
    let red = og_render::Glass {
        tint: [1.0, 0.0, 0.0, 1.0],
        ..panel
    };
    let (tinted, ..) = stripes_frame(&[red]);
    let c = px(&tinted, 160, 100);
    assert!(
        c[0] > 245 && c[1] < 10 && c[2] < 10,
        "a full tint is the tint ({c:?})"
    );
}

/// `scene` rendered and read back.
fn rendered(scene: &RenderScene, desc: &RenderDesc) -> (Vec<u8>, og_render::FrameStats) {
    let frame = render_frame(scene, desc);
    (frame.image.pixels, frame.stats)
}

/// Glass over overlay content: the panel blurs the overlay layers drawn
/// below it as it blurs the scene (the orange bar of
/// `glass_overlay_content`, on the layer under the panel, spreads into the
/// stripes inside the panel and stays sharp outside it), what is drawn
/// above it (the cyan square) stays sharp, and the frame matches
/// `references/<tier>/glass.png`.
///
/// Ways it could fail: the overlay under the glass is drawn after the
/// pyramid was built (the bar sharp inside the panel, the old behaviour),
/// the overlay drawn into the glass source never reaches the output (the
/// bar missing outside the panel), the pyramid is rebuilt from a source
/// that already holds the content above the glass (the square smeared).
#[test]
#[ignore = "requires GPU"]
fn a_glass_panel_blurs_the_overlay_layers_below_it() {
    let (scene, desc) = glass_reference_scene();
    let Rendered { image, stats, info } = render_frame(&scene, &desc);
    let glass = &image.pixels;
    let (mut plain_scene, _) = glass_reference_scene();
    if let Some(o) = plain_scene.ui.overlay.as_mut() {
        o.glass.clear();
    }
    let (plain, _) = rendered(&plain_scene, &desc);
    let w = desc.width as usize;
    let px = |p: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [p[i] as i32, p[i + 1] as i32, p[i + 2] as i32]
    };
    // Outside the panel the bar is drawn as it is, sharp.
    assert_eq!(
        px(glass, 60, 100),
        px(&plain, 60, 100),
        "the bar outside the panel"
    );
    let orange = px(&plain, 60, 100);
    assert!(
        orange[0] > 240 && orange[2] < 20,
        "the bar is orange ({orange:?})"
    );
    assert_eq!(
        px(glass, 60, 86),
        px(&plain, 60, 86),
        "and the stripes above it"
    );
    // Inside the panel the bar is blurred: its primary reaches 8 px above
    // its edge, where the plain frame has grey stripes.
    for x in [150, 154, 158] {
        let (g, p) = (px(glass, x, 84), px(&plain, x, 84));
        assert_eq!(p[0], p[2], "the plain frame there is a grey stripe ({p:?})");
        assert!(
            g[0] - g[2] > 24,
            "the bar under the glass spreads ({x}, 84): {g:?}"
        );
    }
    // On the bar's middle row the stripes no longer show through.
    let row: Vec<i32> = (150..170).map(|x| px(glass, x, 100)[2]).collect();
    let (lo, hi) = (*row.iter().min().unwrap(), *row.iter().max().unwrap());
    assert!(
        hi - lo < 24,
        "the bar's row under the glass is smooth ({lo}..{hi})"
    );
    // The square above the glass is as sharp as without it.
    for (x, y) in [(190, 60), (198, 68), (205, 75)] {
        assert_eq!(
            px(glass, x, y),
            px(&plain, x, y),
            "the square above the glass ({x}, {y})"
        );
    }
    assert_ne!(px(glass, 188, 68), px(glass, 191, 68), "with a hard edge");
    assert!(stats.draw_calls > 0);
    matches_reference("glass", &image, &info, Against::OwnTier);
}

/// The glass's cost on this machine's GPU at 1280×720: the same frame with
/// no glass, four dialog-sized glass panels over the scene only, the four
/// over overlay content (a HUD bar under them: the pyramid is built after
/// it, once), and two layers of two panels with content under each (two
/// pyramids) (`cargo test -p og-tests-ui --release --test tests -- --ignored
/// gfx::glass_cost --nocapture`; the former test sample
/// the former test sample
#[test]
#[ignore]
fn glass_cost() {
    let (w, h) = (1280u32, 720u32);
    const MODES: [&str; 4] = [
        "no glass",
        "four panels over the scene",
        "four panels over overlay content",
        "two layers of two panels, content under each",
    ];
    let mut best = [f64::INFINITY; 4];
    for round in 0..12 {
        let mode = round % 4;
        let mut scene = RenderScene::new();
        scene.frame.camera = og_render::Camera::orthographic_2d(Vec2::new(640.0, 360.0), 720.0);
        for k in 0..160 {
            scene.sprite.sprites.push(og_render::Sprite {
                position: Vec3::new(k as f32 * 8.0 + 2.0, 360.0, 0.0),
                size: Vec2::new(4.0, 720.0),
                ..og_render::Sprite::default()
            });
        }
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.look.post.tonemap = Tonemap::None;
        let mut overlay = og_render::Overlay::new(w as f32, h as f32, 1.0);
        let bar = |layer: i32, y: f32| og_render::Sprite {
            position: Vec3::new(0.0, y, 0.0),
            size: Vec2::new(w as f32, 40.0),
            anchor: Vec2::ZERO,
            color: [1.0, 0.45, 0.0, 1.0],
            layer,
            ..og_render::Sprite::default()
        };
        if mode >= 2 {
            overlay.sprites.push(bar(-1, 300.0));
        }
        if mode == 3 {
            overlay.sprites.push(bar(1, 420.0));
        }
        if mode >= 1 {
            for k in 0..4 {
                overlay.glass.push(og_render::Glass {
                    position: Vec2::new(40.0 + k as f32 * 300.0, 160.0),
                    size: Vec2::new(280.0, 400.0),
                    radius: 24.0,
                    blur: 16.0,
                    tint: [0.1, 0.1, 0.12, 0.2],
                    alpha: 1.0,
                    layer: if mode == 3 && k >= 2 { 2 } else { 0 },
                    clip: None,
                });
            }
        }
        scene.ui.overlay = Some(overlay);
        let desc = RenderDesc {
            width: w,
            height: h,
            ..RenderDesc::default()
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("read back");
        let n = 600;
        let start = std::time::Instant::now();
        for _ in 0..n {
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("read back");
        let ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
        println!("round {round}: {} {ms:.3} ms", MODES[mode]);
        best[mode] = best[mode].min(ms);
    }
    let tier = Renderer::new_blocking(&RenderDesc::default())
        .expect("a headless adapter")
        .info()
        .backend_tier();
    for (k, m) in MODES.iter().enumerate() {
        println!(
            "best ({tier}): {m} {:.3} ms ({:+.3} ms against no glass)",
            best[k],
            best[k] - best[0]
        );
    }
}

/// The glow's cost on this machine's GPU: 24 display-size labels rendered
/// 300 times with and without it (`cargo test -p og-tests-ui --release --test tests
/// -- --ignored gfx::text_glow_cost --nocapture`; the former test sample
#[test]
#[ignore]
fn text_glow_cost() {
    let mut best = [f64::INFINITY; 2];
    for round in 0..6 {
        let glow = round % 2 == 1;
        let (scene, desc) = with_test_font(|f| glow_scene(f, glow, 24));
        let desc = RenderDesc {
            width: 1280,
            height: 1040,
            ..desc
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        let mut stats = renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("read back");
        let n = 600;
        let start = std::time::Instant::now();
        for _ in 0..n {
            stats = renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("read back");
        let ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
        best[usize::from(glow)] = best[usize::from(glow)].min(ms);
        println!(
            "glow {glow}: {} glyph instances, {ms:.4} ms per frame",
            stats.overlay_glyphs
        );
    }
    println!(
        "best of three: {:.4} ms without, {:.4} ms with the glow ({:+.4} ms)",
        best[0],
        best[1],
        best[1] - best[0]
    );
}

/// `references/<tier>/text-fields-2.png` (`support::text_fields_shot`): a
/// word a double click selected, a field after an undo, and a text area a
/// drag below it scrolled part way with its selection over the lines it
/// passed. Ways it could fail: the word's selection fill off the glyphs,
/// the undone text or the caret drawn wrong, the area's lines not cut at
/// its top edge mid-line (a scroll by whole lines), a missing glyph.
#[test]
#[ignore = "requires GPU"]
fn text_fields_match_their_reference_frame() {
    let shot = with_test_font(text_fields_shot);
    let frame = shot.ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert_eq!(
        frame.text_input.clone().expect("the words field").selection,
        "select"
    );
    let rendered = render_frame(&shot.scene, &shot.desc);
    assert_eq!(rendered.stats.missing_glyphs, 0);
    matches_reference(
        "text-fields-2",
        &rendered.image,
        &rendered.info,
        Against::OwnTier,
    );
}

/// A nine-slice's middle row squeezed along one axis keeps its outline as
/// dark as the art's own (debt2/ui-aniso, the root of ui-prose's band): the
/// sprite sampler the UI draws through filters anisotropically, so a piece
/// squeezed 6× in y and shown 1:1 in x reads its mip level by the x axis
/// (level 0), where trilinear filtering reads it by the larger axis (level
/// 2 to 3) and blurs the outline into the fill. The art is an edge column
/// of a mipmapped atlas: a 4-texel dark outline on a cream fill, uniform
/// down its 48 rows. It is drawn twice, 1:1 and 48 rows into 8 px. On
/// every tier the 1:1 piece shows the art exactly (its 4 outline columns
/// dark, cream beside them: an anisotropic filter that softens an
/// isotropic draw, as lavapipe's does, is not one the sampler may use).
/// Where the tier filters anisotropically (`RenderInfo::sprite_anisotropy`
/// above 1, RenderCapabilities.spriteAnisotropy), the squeezed piece's
/// outline reads the 1:1 one's darkness; where it does not (1: a
/// software rasterizer), og-ui's 2× bound on a
/// middle's squeeze is what keeps the band away, and that half is skipped.
///
/// Ways it could fail: the sampler reads the mip level by the larger axis
/// (the Linear-mip sampler: the outline lightens to the fill's side), or
/// the anisotropic filter blurs what is drawn 1:1.
#[test]
#[ignore = "requires GPU"]
fn a_middle_squeezed_six_times_keeps_its_outline_as_dark_as_the_art() {
    use og_render::{ColorSpace, Mips, Sprite, TextureData};
    let (w, h) = (64u32, 64u32);
    let (cream, ink) = ([240u8, 225, 190, 255], [40u8, 30, 30, 255]);
    let rgba = (0..w * h)
        .flat_map(|i| {
            if (8..12).contains(&(i % w)) {
                ink
            } else {
                cream
            }
        })
        .collect();
    let mut scene = RenderScene::new();
    let art = scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    });
    let piece = |x: f32, height: f32| Sprite {
        texture: Some(art),
        position: Vec3::new(x, 16.0, 0.0),
        size: Vec2::new(32.0, height),
        anchor: Vec2::ZERO,
        // 32 × 48 texels of the column, the outline at its texels 8 to 12.
        uv_rect: [0.0, 8.0 / 64.0, 0.5, 56.0 / 64.0],
        ..Default::default()
    };
    let mut overlay = og_render::Overlay::new(128.0, 80.0, 1.0);
    overlay.sprites.push(piece(16.0, 48.0));
    overlay.sprites.push(piece(80.0, 8.0));
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 128,
        height: 80,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let aniso = renderer.info().sprite_anisotropy;
    let tier = renderer.info().backend_tier();
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    // The 1:1 piece's row across the outline: cream, 4 ink, cream.
    let profile: Vec<u8> = (16 + 7..16 + 13)
        .map(|x| image.pixels[((40 * image.width + x) * 4) as usize])
        .collect();
    let near = |v: u8, want: u8| v.abs_diff(want) <= 4;
    assert!(
        near(profile[0], cream[0])
            && profile[1..5].iter().all(|&v| near(v, ink[0]))
            && near(profile[5], cream[0]),
        "{tier} (anisotropy {aniso}): the 1:1 piece reads {profile:?} across its outline, \
         not the art's {} ×4 between {}",
        ink[0],
        cream[0]
    );
    // The darkest red across the outline's columns, on the piece's middle row.
    let darkest = |x0: u32, y: u32| {
        (x0 + 6..x0 + 14)
            .map(|x| image.pixels[((y * image.width + x) * 4) as usize])
            .min()
            .expect("columns")
    };
    let level0 = darkest(16, 40);
    let squeezed = darkest(80, 20);
    let fill = image.pixels[((20 * image.width + 80 + 24) * 4) as usize];
    println!(
        "{tier} (anisotropy {aniso}): outline 1:1 {level0}, squeezed 6× {squeezed}, fill {fill}"
    );
    if aniso == 1 {
        println!("{tier}: no anisotropic filtering; the squeeze is og-ui's 2× bound's");
        return;
    }
    assert!(
        squeezed <= level0 + 6,
        "the outline through a 6× squeezed middle reads {squeezed}, the art's level 0 {level0} (fill {fill}): \
         the sampler picked the mip level by the squeezed axis"
    );
}

/// An element's skew reaches the UI pass whole (docs/ui.md §3.3, §2): a
/// box's background (an old rounded quad) and an element draw's ellipse,
/// both skewed 30° along x about their centres, cover the parallelogram,
/// not their upright boxes, and a turn keeps the draw's exact shape.
///
/// Ways it could fail: the skew dropped where the map is split into a turn
/// and a scale for sprites (U2's path: each box upright), the map applied
/// to the corners but not the shape's local space (a sheared box with
/// upright edges), or the draw's shapes left in the layout box.
#[test]
#[ignore = "requires GPU"]
fn a_skewed_element_draws_its_parallelogram() {
    let vp = Viewport {
        width: 400.0,
        height: 200.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let skew = Props {
        transform: Transform {
            skew_x: 30f32.to_radians(),
            ..Transform::IDENTITY
        },
        ..Props::default()
    };
    let red = Srgba::rgb(255, 0, 0);
    let mut ui = Ui::new(UiDesc::default());
    let build = |ui: &mut Ui, props: &Props| {
        ui.begin(InputFrame::default(), vp);
        ui.open(
            "Row",
            Element {
                direction: Direction::Row,
                padding: Insets::all(60.0),
                gap: 80.0,
                ..Element::default()
            },
        );
        for (id, draw) in [("Quad", false), ("Drawn", true)] {
            ui.open_with(
                id,
                Element {
                    width: Size::Fixed(100.0),
                    height: Size::Fixed(60.0),
                    background: (!draw).then_some(red),
                    radius: 0.0,
                    ..Element::default()
                },
                props.clone(),
            );
            if draw {
                ui.set_draw(og_ui::paint::ElementDraw {
                    shape: og_ui::paint::ShapeKind::Ellipse,
                    fills: vec![og_ui::paint::Paint::Solid(red.into())],
                    ..Default::default()
                });
            }
            ui.close();
        }
        ui.close();
        ui.end();
    };
    build(&mut ui, &Props::default());
    let boxes: Vec<[f32; 4]> = ["Row/Quad", "Row/Drawn"]
        .iter()
        .map(|p| {
            let e = ui.element(p).expect("laid out");
            [e.x, e.y, e.width, e.height]
        })
        .collect();
    build(&mut ui, &skew);
    let mut scene = RenderScene::new();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 400,
        height: 200,
        ..RenderDesc::default()
    };
    let (pixels, _) = rendered(&scene, &desc);
    let red_at = |x: f32, y: f32| {
        let i = (y as usize * 400 + x as usize) * 4;
        pixels[i] > 200 && pixels[i + 1] < 40
    };
    let t = 30f32.to_radians().tan();
    for (k, [x, y, w, h]) in boxes.into_iter().enumerate() {
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        // 5 px in from the top and bottom: the edges there sit 25·tan 30°
        // (14.4 px) left and right of the upright box's.
        let (top, bottom) = (cy - h / 2.0 + 5.0, cy + h / 2.0 - 5.0);
        let shift = t * (h / 2.0 - 5.0);
        // The quad's corners; the ellipse's widest row is its centre, so
        // its probes sit on the centre line moved by the skew there (none)
        // and on rows a quarter up and down.
        let (inside, outside) = if k == 0 {
            (
                [
                    (cx + w / 2.0 + shift - 4.0, bottom),
                    (cx - w / 2.0 - shift + 4.0, top),
                ],
                [
                    (cx + w / 2.0 - shift + 4.0, top),
                    (cx - w / 2.0 + shift - 4.0, bottom),
                ],
            )
        } else {
            let q = h / 4.0;
            let half_w = w / 2.0 * (1.0 - 0.25f32).sqrt();
            (
                [
                    (cx + half_w + t * q - 4.0, cy + q),
                    (cx - half_w - t * q + 4.0, cy - q),
                ],
                [
                    (cx + half_w - t * q + 4.0, cy - q),
                    (cx - half_w + t * q - 4.0, cy + q),
                ],
            )
        };
        for (px, py) in inside {
            assert!(
                red_at(px, py),
                "box {k}: ({px}, {py}) inside the skewed shape"
            );
        }
        for (px, py) in outside {
            assert!(
                !red_at(px, py),
                "box {k}: ({px}, {py}) outside the skewed shape"
            );
        }
    }
}

/// A gradient mask (`clip: mask`) fades the element's subtree as CSS's
/// mask does over the ground it fades against: halfway along a mask going
/// from whole to none, a black fill over white and a white fill over
/// black each draw CSS's sRGB 128, and the faded edge is the ground. The
/// black box masks its own fill; the white box masks a child's (its
/// parent draws nothing itself, over an opaque black backing).
///
/// Ways it could fail: the mask never reaches the layer (both boxes
/// solid), or it fades in linear light (188 halfway in both): a dark
/// scrim then hardly darkens until the mask is nearly whole, so its fade
/// reads as a line (U6c's frame scrim in round 13).
#[test]
#[ignore = "requires GPU"]
fn a_gradient_mask_fades_as_it_reads() {
    use og_ui::paint::{Clip, ElementDraw, Gradient, Mask, Paint, Stop};
    let vp = Viewport {
        width: 400.0,
        height: 200.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let white = Srgba::rgb(255, 255, 255);
    let black = Srgba::rgb(0, 0, 0);
    let fade = || {
        Clip::Mask(Mask::Gradient(Gradient {
            from: [0.0, 0.0],
            to: [1.0, 0.0],
            stops: vec![
                Stop {
                    offset: 0.0,
                    color: white.into(),
                },
                Stop {
                    offset: 1.0,
                    color: white.fade(0.0).into(),
                },
            ],
            dither: false,
            ..Gradient::default()
        }))
    };
    let mut ui = Ui::new(UiDesc::default());
    ui.begin(InputFrame::default(), vp);
    ui.open(
        "Row",
        Element {
            direction: Direction::Row,
            padding: Insets::all(50.0),
            gap: 50.0,
            ..Element::default()
        },
    );
    let boxed = Element {
        width: Size::Fixed(125.0),
        height: Size::Fixed(100.0),
        radius: 0.0,
        ..Element::default()
    };
    ui.open("Own", boxed);
    ui.set_draw(ElementDraw {
        fills: vec![Paint::Solid(black.into())],
        clip: fade(),
        ..Default::default()
    });
    ui.close();
    ui.open(
        "Backing",
        Element {
            background: Some(black),
            ..boxed
        },
    );
    ui.open("Parent", boxed);
    ui.set_draw(ElementDraw {
        clip: fade(),
        ..Default::default()
    });
    ui.open(
        "Child",
        Element {
            background: Some(white),
            ..boxed
        },
    );
    ui.close();
    ui.close();
    ui.close();
    ui.close();
    ui.end();
    let boxes: Vec<[f32; 4]> = ["Row/Own", "Row/Backing/Parent"]
        .iter()
        .map(|p| {
            let e = ui.element(p).expect("laid out");
            [e.x, e.y, e.width, e.height]
        })
        .collect();
    let mut scene = RenderScene::new();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ONE);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 400,
        height: 200,
        ..RenderDesc::default()
    };
    let (pixels, _) = rendered(&scene, &desc);
    if let Ok(dir) = std::env::var("OG_UI_SHOT") {
        let path = PathBuf::from(dir).join("mask-fade.png");
        image::save_buffer(&path, &pixels, 400, 200, image::ColorType::Rgba8).expect("saved");
    }
    let lum = |x: f32, y: f32| f32::from(pixels[(y as usize * 400 + x as usize) * 4]);
    for (k, ([x, y, w, h], (whole, ground))) in boxes
        .into_iter()
        .zip([(0.0, 255.0), (255.0, 0.0)])
        .enumerate()
    {
        let half = 128.0;
        let cy = y + h / 2.0;
        let (start, mid, end) = (lum(x + 1.0, cy), lum(x + w / 2.0, cy), lum(x + w - 1.0, cy));
        assert!(
            (start - whole).abs() <= 6.0
                && (mid - half).abs() <= 6.0
                && (end - ground).abs() <= 6.0,
            "box {k}: sRGB {start} at its start (want {whole}), {mid} halfway (want {half}, as CSS draws it), {end} at its faded edge (want the ground's {ground})"
        );
    }
}

/// A Rive artboard that cuts its content to its frame, played in an
/// element turned 45°, is cut by the turned box: a red square twice the
/// box's size fills the box's diamond, its tip just inside a turned corner
/// red, and a point inside the upright box but outside the turned one
/// stays the ground.
///
/// Ways it could fail: the cut taken in screen space from the layout box
/// (the content cut by its unturned rectangle: the tip missing and the
/// upright corners red), or no cut (red past the diamond).
#[test]
#[ignore = "requires GPU"]
fn a_turned_rive_frame_is_cut_by_its_turned_box() {
    use og_ui::rive::*;
    let vp = Viewport {
        width: 300.0,
        height: 300.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let node = |parent, kind, values| Node {
        name: "n".into(),
        parent,
        kind,
        values,
    };
    // 100 × 100, its content a 200 × 200 red square about its centre.
    let board = std::sync::Arc::new(Artboard {
        name: "Board".into(),
        size: Vec2::new(100.0, 100.0),
        fit: Fit::None,
        clip: true,
        nodes: vec![
            node(0, Kind::Artboard { paints: vec![] }, Values::default()),
            node(
                0,
                Kind::Shape {
                    paths: vec![2],
                    paints: vec![3],
                    hidden: false,
                    blend: og_render::UiBlend::Normal,
                },
                Values {
                    x: 50.0,
                    y: 50.0,
                    ..Values::default()
                },
            ),
            node(
                1,
                Kind::Path {
                    outline: Outline::Rect,
                    closed: true,
                },
                Values {
                    width: 200.0,
                    height: 200.0,
                    ..Values::default()
                },
            ),
            node(
                1,
                Kind::Fill {
                    rule: og_render::UiFillRule::NonZero,
                    shader: 4,
                    visible: true,
                },
                Values::default(),
            ),
            node(
                3,
                Kind::Solid,
                Values {
                    color: 0xFFFF_0000,
                    ..Values::default()
                },
            ),
        ],
        draws: vec![1],
        ..Artboard::default()
    });
    let mut ui = Ui::new(UiDesc::default());
    ui.begin(InputFrame::default(), vp);
    ui.open(
        "Stage",
        Element {
            width: Size::Fixed(300.0),
            height: Size::Fixed(300.0),
            align: Align::Center,
            cross: Cross::Center,
            ..Element::default()
        },
    );
    ui.open_with(
        "Turned",
        Element {
            width: Size::Fixed(100.0),
            height: Size::Fixed(100.0),
            ..Element::default()
        },
        Props {
            transform: Transform {
                rotate: 45f32.to_radians(),
                ..Transform::IDENTITY
            },
            ..Props::default()
        },
    );
    ui.set_draw(og_ui::paint::ElementDraw {
        rive: Some(og_ui::paint::RiveDraw(Player::new(board, None))),
        ..Default::default()
    });
    ui.close();
    ui.close();
    ui.end();
    let mut scene = RenderScene::new();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 300,
        height: 300,
        ..RenderDesc::default()
    };
    let (pixels, _) = rendered(&scene, &desc);
    let red_at = |x: f32, y: f32| {
        let i = (y as usize * 300 + x as usize) * 4;
        pixels[i] > 200 && pixels[i + 1] < 40
    };
    // The box's centre is the stage's; turned 45°, its corners point up,
    // down, left and right, 70.7 px out.
    let c = 150.0;
    for (x, y) in [(c, c - 66.0), (c + 66.0, c), (c, c + 66.0), (c - 66.0, c)] {
        assert!(red_at(x, y), "({x}, {y}) just inside a turned corner");
    }
    for (x, y) in [(c + 45.0, c - 45.0), (c - 45.0, c + 45.0)] {
        assert!(
            !red_at(x, y),
            "({x}, {y}) inside the upright box, outside the turned one"
        );
    }
    assert!(red_at(c, c), "the centre");
}

/// A card turned 45° about its vertical axis under its parent's 400 px
/// `perspective` (CSS Transforms 2): its left edge comes toward the viewer
/// and is seen taller than the card, its right edge recedes and is seen
/// shorter, as the projective map puts them; and the pointer hits it where
/// it is seen, not where it was laid out.
///
/// Ways it could fail: the layer is laid on its plane affinely (both edges
/// as tall), the plane reads the scene's depth and is hidden, or hit
/// testing ignores the projective row (a hit right of the seen card).
#[test]
#[ignore = "requires GPU"]
fn a_card_turned_in_perspective_draws_and_hits_where_it_is_seen() {
    use og_ui::doc::UiDocument;
    use serde_json::json;
    let vp = Viewport {
        width: 400.0,
        height: 300.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(
        UiDocument::parse(
            "m.svelte",
            r#"<meta name="screen" content="m" data-layer="hud">
<stack id="Stage"><button id="Card"/></stack>
<style>
#Stage { width: 400px; height: 300px; perspective: 400px; }
#Card { position: absolute; top: 50px; inset-inline-start: 100px; width: 200px; height: 200px;
        background: #ff0000; border-radius: 0; cursor: crosshair; transform: rotateY(45deg); }
</style>"#,
        )
        .unwrap(),
    );
    ui.push_screen("m", json!({})).unwrap();
    let frame = |ui: &mut Ui, p: Vec2| {
        ui.begin(
            InputFrame {
                pointer: p,
                pointer_inside: true,
                ..InputFrame::default()
            },
            vp,
        );
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    };
    // Where the card's corners are seen: centre (200, 150), perspective
    // origin the stage's centre (200, 150), d = 400. x off the centre is
    // u; turned, it lies at x = 200 + u·cos45, z = -u·sin45 (+x recedes)
    // and is seen scaled by d / (d - z).
    let seen = |u: f32, v: f32| {
        let (c, s) = (45f32.to_radians().cos(), 45f32.to_radians().sin());
        let k = 400.0 / (400.0 + u * s);
        (200.0 + u * c * k, 150.0 + v * k)
    };
    let (lx, ly) = seen(-100.0, -100.0);
    let (rx, ry) = seen(100.0, -100.0);
    assert!(ly < 50.0 && ry > 50.0, "{ly} {ry}");
    frame(&mut ui, Vec2::new(10.0, 10.0));
    let mut scene = RenderScene::new();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 400,
        height: 300,
        ..RenderDesc::default()
    };
    let (pixels, _) = rendered(&scene, &desc);
    image::save_buffer(
        out_dir().join("transform-3d.png"),
        &pixels,
        400,
        300,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let red_at = |x: f32, y: f32| {
        let i = (y as usize * 400 + x as usize) * 4;
        pixels[i] > 200 && pixels[i + 1] < 60
    };
    // The near edge reaches above the laid-out top, the far edge does not
    // reach it; both corners are where the map puts them.
    assert!(
        red_at(lx + 4.0, ly + 4.0),
        "near top corner at ({lx}, {ly})"
    );
    assert!(!red_at(lx + 4.0, ly - 4.0), "above the near top corner");
    assert!(red_at(rx - 4.0, ry + 4.0), "far top corner at ({rx}, {ry})");
    assert!(!red_at(rx - 4.0, ry - 4.0), "above the far top corner");
    assert!(!red_at(rx + 6.0, 150.0), "right of the far edge");
    // Hit where it is seen: inside near the far edge, and not past it
    // though the laid-out box reaches there.
    frame(&mut ui, Vec2::new(rx - 6.0, 150.0));
    frame(&mut ui, Vec2::new(rx - 6.0, 150.0));
    assert_eq!(ui.frame().cursor, Cursor::Crosshair, "over the seen card");
    frame(&mut ui, Vec2::new(rx + 10.0, 150.0));
    frame(&mut ui, Vec2::new(rx + 10.0, 150.0));
    assert_ne!(ui.frame().cursor, Cursor::Crosshair, "past the seen card");
}

/// Planes of one 3D rendering context (`transform-style: preserve-3d`)
/// draw back to front by depth, not in tree order: the cube's back face
/// is last in the tree and stays behind its front face. A card turned
/// to show its back with `backface-visibility: hidden` draws nothing.
///
/// Ways it could fail: the context's planes are drawn in tree order (the
/// back face's green over the front's red), the children are flattened
/// into the cube's own layer (nothing turned), or the hidden back face
/// still draws.
#[test]
#[ignore = "requires GPU"]
fn preserve_3d_planes_draw_by_depth_and_hide_their_backs() {
    use og_ui::doc::UiDocument;
    use serde_json::json;
    let vp = Viewport {
        width: 400.0,
        height: 300.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(
        UiDocument::parse(
            "m.svelte",
            r#"<meta name="screen" content="m" data-layer="hud">
<stack id="Stage"><stack id="Cube"><panel id="F"/><panel id="L"/><panel id="B"/></stack><panel id="Flip"/></stack>
<style>
#Stage { width: 400px; height: 300px; perspective: 600px; }
#Cube { position: absolute; top: 100px; inset-inline-start: 200px; width: 100px; height: 100px;
        transform-style: preserve-3d; transform: rotateY(30deg); }
#F, #L, #B { position: absolute; top: 0; inset-inline-start: 0; width: 100px; height: 100px; border-radius: 0; }
#F { background: #ff0000; transform: translateZ(50px); }
#L { background: #ffff00; transform: rotateY(-90deg) translateZ(50px); }
#B { background: #00ff00; transform: rotateY(180deg) translateZ(50px); }
#Flip { position: absolute; top: 100px; inset-inline-start: 40px; width: 100px; height: 100px;
        border-radius: 0; background: #0000ff; transform: rotateY(180deg); backface-visibility: hidden; }
</style>"#,
        )
        .unwrap(),
    );
    ui.push_screen("m", json!({})).unwrap();
    ui.begin(InputFrame::default(), vp);
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut scene = RenderScene::new();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: 400,
        height: 300,
        ..RenderDesc::default()
    };
    let (pixels, _) = rendered(&scene, &desc);
    image::save_buffer(
        out_dir().join("preserve-3d.png"),
        &pixels,
        400,
        300,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let at = |x: usize, y: usize| {
        let i = (y * 400 + x) * 4;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    // The front face is seen right of the cube's centre (250, 150): the
    // turn moves it right; the back face lies behind it there.
    let front = at(262, 150);
    assert!(front[0] > 200 && front[1] < 60, "front face red: {front:?}");
    // The left face shows left of the centre.
    let left = at(222, 150);
    assert!(left[0] > 200 && left[1] > 200, "left face yellow: {left:?}");
    let flip = at(90, 150);
    assert!(flip.iter().all(|&c| c < 20), "hidden back face: {flip:?}");
}
