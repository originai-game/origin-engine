//! Rich text: inline spans in one paragraph, driven tick by tick.
//!
//! Ways it could fail, each covered below:
//! - a span's style is lost (one ink, one size, no weight), an icon or a
//!   link's underline is not drawn;
//! - lines break inside a word that changes style mid-word, or a line
//!   overruns the width it was given, or a taller span does not raise its
//!   line;
//! - a link is not an element the probe can find and press, a click on it
//!   does not answer its id, keyboard focus cannot reach it;
//! - per-glyph motion does not move by the tick, or moves under reduced
//!   motion;
//! - markup it cannot read disappears silently instead of showing as text
//!   with a warning;
//! - in a right-to-left UI the spans run left to right, or punctuation
//!   between two spans sits on the wrong side of its space, or a Latin
//!   line's full stop is moved before it.

use crate::support;

use og_render::{ColorSpace, Mips, RenderScene, TextureData};
use og_ui::*;
use support::*;

fn rich_ui() -> Ui {
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let icons = scene.add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    let mut ui = ui_with_font(font);
    ui.set_icons(IconAtlas {
        texture: Some(icons),
        columns: 2,
        rows: 1,
        names: vec!["coin".into(), "star".into()],
        ..IconAtlas::default()
    });
    ui
}

fn step(ui: &mut Ui, input: InputFrame, markup: &str, width: Size) -> RichTextResponse {
    ui.begin(input, Viewport::new(600.0, 400.0));
    let mut r = RichTextResponse::default();
    ui.column("Card", Element::column(), |ui| {
        r = ui.rich_text(
            "Hint",
            markup,
            RichTextDesc {
                width,
                ..RichTextDesc::default()
            },
        );
    });
    ui.end();
    r
}

fn texts(ui: &Ui) -> Vec<(String, f32, f32, [f32; 4], f32, f32)> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text {
                text,
                x,
                baseline,
                color,
                size,
                weight,
                ..
            } => Some((text.clone(), *x, *baseline, *color, *size, *weight)),
            _ => None,
        })
        .collect()
}

fn attribute(name: &str, position: u32, length: u32) -> TextSpan {
    TextSpan {
        name: name.into(),
        position,
        length,
        properties: Default::default(),
    }
}

#[test]
fn unknown_attributes_do_not_split_visible_motion() {
    let render = |attributes: &[TextSpan]| {
        let mut ui = rich_ui();
        for _ in 0..12 {
            ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
            let r = ui.rich_text_spans(
                "Line",
                "Wonderful",
                attributes,
                RichTextDesc::default(),
                |a| {
                    if a.name == "wave" {
                        TextSpanStyle {
                            motion: Some(markup::SpanFx::Wave),
                            ..Default::default()
                        }
                    } else {
                        TextSpanStyle::default()
                    }
                },
            );
            assert_eq!(r.spans, attributes);
            ui.end();
        }
        texts(&ui)
    };
    let plain = render(&[attribute("wave", 0, 9)]);
    let with_metadata = render(&[attribute("wave", 0, 9), attribute("game", 3, 4)]);
    assert_eq!(
        with_metadata, plain,
        "metadata must not restart per-glyph wave phase"
    );
}

#[test]
fn structured_scalars_overlap_and_literal_text_preserve_unknown_metadata() {
    let mut ui = rich_ui();
    let text = "Ada: A😀BC [b] \\end";
    let mut spans = vec![
        attribute("character", 0, 5),
        attribute("emphasis", 7, 2),
        attribute("override", 8, 1),
        attribute("game-event", 6, 0),
        attribute("outside", u32::MAX, 2),
    ];
    spans[0]
        .properties
        .insert("name".into(), TextSpanValue::String("Ada".into()));
    spans[3]
        .properties
        .insert("enabled".into(), TextSpanValue::Bool(true));
    spans[3]
        .properties
        .insert("level".into(), TextSpanValue::Number(2.5));
    ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
    let r = ui.rich_text_spans("Line", text, &spans, RichTextDesc::default(), |a| {
        match a.name.as_str() {
            "character" => TextSpanStyle {
                color: Some(paint::ColorRef::token("highlight")),
                ..Default::default()
            },
            "emphasis" => TextSpanStyle {
                bold: Some(true),
                color: Some(paint::ColorRef::token("danger")),
                ..Default::default()
            },
            "override" => TextSpanStyle {
                color: Some(paint::ColorRef::token("success")),
                ..Default::default()
            },
            _ => TextSpanStyle::default(),
        }
    });
    ui.end();
    assert_eq!(
        r.spans, spans,
        "including the invalid range and the zero-length game marker"
    );
    assert!(ui.elements().iter().any(|e| e.text == text));
    let drawn = texts(&ui);
    let b = drawn
        .iter()
        .find(|v| v.0 == "B")
        .expect("scalar 7 is B, not the emoji or C");
    let c = drawn.iter().find(|v| v.0 == "C").expect("scalar 8 is C");
    assert!(near(b.3, ui.theme().palette.danger));
    assert!(near(c.3, ui.theme().palette.success));
    assert!(
        b.5 > 0.0 && c.5 > 0.0,
        "later colour must not clear earlier bold"
    );
    assert!(
        drawn.iter().any(|v| v.0.contains("[b]")),
        "brackets are never markup"
    );
    assert!(
        drawn.iter().any(|v| v.0.contains("\\end")),
        "escapes are never parsed again"
    );
    assert_eq!(ui.frame().warnings.len(), 1);
    assert!(ui.frame().warnings[0].contains("outside"));
    assert_eq!(
        serde_json::from_str::<Vec<TextSpan>>(&serde_json::to_string(&spans).unwrap()).unwrap(),
        spans
    );
    for invalid in ["null", "[]", "{}"] {
        let payload =
            format!(r#"{{"name":"game","position":0,"length":0,"properties":{{"x":{invalid}}}}}"#);
        assert!(serde_json::from_str::<TextSpan>(&payload).is_err());
    }
}

#[test]
fn document_spans_cascade_by_name_property_and_active_theme() {
    use serde_json::json;
    let document = UiDocument::parse(
        "line.svelte",
        r#"
<meta name="component" content="line">
<text id="Line" spans={spans} class:urgent={urgent}>{body}</text>
<style>
text::part(speaker) { color: var(--ink-muted); }
#Line::part(speaker)[name="Ada"] { color: var(--highlight); font-weight: bold; }
#Line.urgent::part(speaker)[name="Ada"] { color: var(--success); }
text::part(speaker)[name="Bo"] { color: var(--danger); }
text::part(emphasis) { text-decoration-line: underline; }
</style>
"#,
    )
    .unwrap();
    let mut ui = rich_ui();
    ui.define_document(document);
    let mut values = json!({"body": "Ada 😀 Bo", "urgent": true, "spans": [
        {"name":"speaker", "position":0, "length":3, "properties":{"name":"Ada"}},
        {"name":"speaker", "position":6, "length":2, "properties":{"name":"Bo"}}
    ]});
    let render = |ui: &mut Ui, values: &mut serde_json::Value| {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.component("Paragraph", "line", values, &bind::NoEnv);
        ui.end();
    };
    render(&mut ui, &mut values);
    let drawn = texts(&ui);
    let ada = drawn
        .iter()
        .find(|s| s.0 == "Ada")
        .unwrap_or_else(|| panic!("drawn: {drawn:?}; warnings: {:?}", ui.frame().warnings));
    let bo = drawn.iter().find(|s| s.0 == "Bo").unwrap();
    assert!(near(ada.3, ui.theme().palette.success));
    assert!(ada.5 > 0.0);
    assert!(near(bo.3, ui.theme().palette.danger));
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    values["urgent"] = json!(false);
    render(&mut ui, &mut values);
    assert!(near(
        texts(&ui).iter().find(|s| s.0 == "Ada").unwrap().3,
        ui.theme().palette.highlight
    ));
    let mut theme = *ui.theme();
    theme.palette.highlight = Srgba::rgb(18, 127, 57);
    ui.set_theme(theme);
    render(&mut ui, &mut values);
    assert!(near(
        texts(&ui).iter().find(|s| s.0 == "Ada").unwrap().3,
        theme.palette.highlight
    ));

    values["spans"][0]["position"] = json!("not a scalar index");
    render(&mut ui, &mut values);
    assert!(
        ui.frame()
            .warnings
            .iter()
            .any(|w| w.contains("line.svelte:")
                && w.contains("Paragraph.spans")
                && w.contains("TextSpan[]"))
    );
    assert!(!ui.elements().iter().any(|e| e.text == "Ada 😀 Bo"));
}

#[test]
fn structured_wrap_motion_restore_and_style_switch_use_existing_paragraph() {
    let mut ui = rich_ui();
    let spans = [attribute("emphasis", 0, 27), attribute("motion", 6, 11)];
    let draw = |ui: &mut Ui, motion| {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.rich_text_spans(
            "Line",
            "Alpha Bravo Delta Echo Foxtrot",
            &spans,
            RichTextDesc {
                width: Size::Fixed(130.0),
                ..Default::default()
            },
            |a| {
                if a.name == "emphasis" {
                    TextSpanStyle {
                        color: Some(paint::ColorRef::token("danger")),
                        ..Default::default()
                    }
                } else {
                    TextSpanStyle {
                        motion: Some(motion),
                        ..Default::default()
                    }
                }
            },
        );
        ui.end();
    };
    for fx in [
        markup::SpanFx::Wave,
        markup::SpanFx::Shake,
        markup::SpanFx::Pulse,
    ] {
        for _ in 0..12 {
            draw(&mut ui, fx);
        }
        let saved = ui.save_state();
        draw(&mut ui, fx);
        let next = texts(&ui);
        for _ in 0..9 {
            draw(&mut ui, fx);
        }
        assert_ne!(texts(&ui), next, "motion changes by the tick: {fx:?}");
        ui.restore_state(&saved).unwrap();
        draw(&mut ui, fx);
        assert_eq!(
            texts(&ui),
            next,
            "restore mid-motion replays exactly: {fx:?}"
        );
    }
    let mut theme = *ui.theme();
    theme.reduced_motion = true;
    theme.palette.danger = Srgba::rgb(23, 211, 187);
    ui.set_theme(theme);
    draw(&mut ui, markup::SpanFx::Wave);
    let still = texts(&ui);
    for _ in 0..8 {
        draw(&mut ui, markup::SpanFx::Wave);
    }
    assert_eq!(texts(&ui), still, "reduced motion is stationary");
    assert!(
        still.iter().any(|v| near(v.3, theme.palette.danger)),
        "active-style token changes the ink"
    );
    assert!(
        still.iter().map(|v| v.2).fold(f32::NEG_INFINITY, f32::max) > still[0].2 + 10.0,
        "paragraph wraps"
    );
}

#[test]
fn italic_underline_strike_and_outline_keep_their_span() {
    // Each of the new styles reaches only its own span: a slant and an
    // outline on its glyphs (the plain words around stay plain), a hairline
    // under [u] and one through [s] near the middle of the x-height.
    let mut ui = rich_ui();
    for _ in 0..2 {
        step(
            &mut ui,
            InputFrame::default(),
            "a [i]lean[/i] b [u]under[/u] c [s]gone[/s] d [outline=2,danger]loud[/outline]",
            Size::Fixed(560.0),
        );
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut styled: Vec<(String, [f32; 4], bool)> = Vec::new();
    let mut plain: Vec<(String, f32)> = Vec::new();
    let mut rules: Vec<Rect> = Vec::new();
    for i in &ui.frame().items {
        match &i.cmd {
            DrawCmd::StyledText { text, effects } => {
                if let DrawCmd::Text { text, .. } = &**text {
                    styled.push((text.clone(), effects.transform, effects.outline.is_some()));
                }
            }
            DrawCmd::Text { text, baseline, .. } => plain.push((text.clone(), *baseline)),
            DrawCmd::Quad { rect, .. } if rect.height < 3.0 => rules.push(*rect),
            _ => {}
        }
    }
    let slanted = |t: &[f32; 4]| t[2] < 0.0;
    assert!(
        styled
            .iter()
            .any(|(t, m, o)| t == "lean" && slanted(m) && !o),
        "[i] slants its own glyphs: {styled:?}"
    );
    assert!(
        styled
            .iter()
            .any(|(t, m, o)| t == "loud" && *o && !slanted(m)),
        "[outline] outlines its own glyphs: {styled:?}"
    );
    assert_eq!(styled.len(), 2, "the other words stay plain: {styled:?}");
    let baseline = plain
        .iter()
        .find(|p| p.0.contains("under"))
        .expect("under")
        .1;
    assert_eq!(rules.len(), 2, "one underline and one strike: {rules:?}");
    let (low, high) = if rules[0].y > rules[1].y {
        (rules[0], rules[1])
    } else {
        (rules[1], rules[0])
    };
    assert!(
        low.y >= baseline,
        "the underline is under the baseline: {low:?} at {baseline}"
    );
    assert!(
        high.y < baseline - 2.0,
        "the strike is through the letters: {high:?} at {baseline}"
    );
}

#[test]
fn a_registered_picture_and_an_actions_glyph_draw_inline() {
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let tex = |scene: &mut RenderScene| {
        scene.add_texture(TextureData {
            width: 2,
            height: 2,
            rgba: vec![255; 16],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        })
    };
    let (portrait, icons) = (tex(&mut scene), tex(&mut scene));
    let mut ui = ui_with_font(font);
    ui.set_icons(IconAtlas {
        texture: Some(icons),
        columns: 2,
        rows: 1,
        names: vec!["coin".into(), "ps.cross".into()],
        ..IconAtlas::default()
    });
    ui.set_images(
        [(
            "mira".to_string(),
            InlineImage {
                texture: portrait,
                uv: [0.0, 0.0, 0.5, 1.0],
                aspect: 2.0,
            },
        )]
        .into(),
    );
    ui.define_action(ActionDesc {
        name: "jump".into(),
        keys: vec!["KeyE".into()],
        buttons: vec!["a".into()],
        label: "Jump".into(),
        ..ActionDesc::default()
    });
    let markup = "[img=mira] says: press [action=jump] to jump";
    let draw = |ui: &mut Ui| {
        for _ in 0..2 {
            step(ui, InputFrame::default(), markup, Size::Fixed(560.0));
        }
        let images: Vec<(Option<og_render::TextureHandle>, Rect)> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Image { texture, rect, .. } => Some((*texture, *rect)),
                _ => None,
            })
            .collect();
        (images, texts(ui))
    };
    // A keyboard: the key's name on a cap, after the portrait, one em tall
    // and twice as wide.
    let (images, t) = draw(&mut ui);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let pic = images
        .iter()
        .find(|i| i.0 == Some(portrait))
        .unwrap_or_else(|| panic!("the portrait: {images:?}"));
    assert!((pic.1.width / pic.1.height - 2.0).abs() < 0.1, "{pic:?}");
    let says = t.iter().find(|t| t.0.contains("says")).expect("says").1;
    assert!(
        pic.1.right() <= says + 1.0,
        "the portrait sits before its line"
    );
    assert!(
        t.iter().any(|t| t.0 == "E"),
        "the key's name on a cap: {t:?}"
    );
    assert!(!images.iter().any(|i| i.0 == Some(icons)), "{images:?}");
    // The next frame on a PlayStation pad: the cross's icon, and no cap.
    ui.set_device(Device::PlayStation);
    let (images, t) = draw(&mut ui);
    assert!(
        images.iter().any(|i| i.0 == Some(icons)),
        "the cross: {images:?}"
    );
    assert!(!t.iter().any(|t| t.0 == "E"), "{t:?}");
    // An unknown picture or action: named in a warning, nothing drawn.
    step(
        &mut ui,
        InputFrame::default(),
        "[img=nobody] [action=fly]",
        Size::fit(),
    );
    let w = &ui.frame().warnings;
    assert!(
        w.iter().any(|w| w.contains("[img=nobody]"))
            && w.iter().any(|w| w.contains("[action=fly]")),
        "{w:?}"
    );
}
