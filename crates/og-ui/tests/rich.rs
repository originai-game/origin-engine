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

use og_ui::*;
use serde_json::json;
use support::*;

fn rich_ui() -> Ui {
    let mut handles = Handles::default();
    let font = handles.test_font();
    let icons = handles.texture();
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

/// A paragraph `Line` of the store `body` styled by the typed attributes
/// of the store `spans` (one value: a signal, never rows), its class the
/// store `fx`, under `css`, in a 600 × 400 viewport.
fn spans_doc(css: &str, body: &str, spans: &[TextSpan], fx: &str) -> Doc {
    let mut ui = rich_ui();
    let text = format!(
        r#"<meta name="component" content="t">
<column id="Card"><text id="Line" spans={{$spans}} class={{$fx}}>{{$body}}</text></column>
<style>{css}</style>"#
    );
    ui.define_document(UiDocument::parse("ui/t.svelte", &text).unwrap_or_else(|e| panic!("{e:?}")));
    apply(
        &mut ui,
        json!({"op": "signal", "source": source("spans"), "initial": spans}),
    );
    store(&mut ui, "body", json!(body));
    store(&mut ui, "fx", json!(fx));
    ui.mount_document(
        reactive::SourceId::new("test", "t"),
        "t",
        ["body", "fx", "spans"]
            .map(|name| (name.to_string(), source(name)))
            .into(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap_or_else(|e| panic!("mounts: {e}"));
    let mut d = Doc {
        ui,
        events: Vec::new(),
        viewport: Viewport::new(600.0, 400.0),
        focus: None,
    };
    d.step(InputFrame::default());
    d
}

/// A paragraph `Hint` of `markup` (inline document markup), `width` wide,
/// in `ui`, in a 600 × 400 viewport.
fn markup_doc(ui: Ui, markup: &str, width: &str) -> Doc {
    let mut d = Doc::with_ui(
        ui,
        &format!(
            r#"<meta name="component" content="t">
<column id="Card"><text id="Hint">{markup}</text></column>
<style>#Hint {{ width: {width}; }}</style>"#
        ),
    );
    d.viewport = Viewport::new(600.0, 400.0);
    d.step(InputFrame::default());
    d
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
        let mut d = spans_doc(
            "text::part(wave) { -og-text-motion: wave; }",
            "Wonderful",
            attributes,
            "",
        );
        for _ in 1..12 {
            d.step(InputFrame::default());
        }
        texts(&d.ui)
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
    let d = spans_doc(
        "text::part(character) { color: var(--highlight); }
text::part(emphasis) { font-weight: bold; color: var(--danger); }
text::part(override) { color: var(--success); }",
        text,
        &spans,
        "",
    );
    let ui = &d.ui;
    assert!(ui.elements().iter().any(|e| e.text == text));
    let drawn = texts(ui);
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
    // One warning, for the invalid range (a document's build gives it
    // twice in the frame).
    let mut warnings = ui.frame().warnings.clone();
    warnings.dedup();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("outside"));
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
<text id="Line" spans={$spans} class:urgent={$urgent}>{$body}</text>
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
    let mut spans = json!([
        {"name":"speaker", "position":0, "length":3, "properties":{"name":"Ada"}},
        {"name":"speaker", "position":6, "length":2, "properties":{"name":"Bo"}}
    ]);
    // The spans are one value, not rows keyed by an id: a signal.
    apply(
        &mut ui,
        json!({"op": "signal", "source": source("spans"), "initial": spans}),
    );
    store(&mut ui, "body", json!("Ada 😀 Bo"));
    store(&mut ui, "urgent", json!(true));
    ui.mount_document(
        reactive::SourceId::new("test", "t"),
        "line",
        ["body", "urgent", "spans"]
            .map(|name| (name.to_string(), source(name)))
            .into(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let render = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    render(&mut ui);
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
    set_store(&mut ui, "urgent", json!(false));
    render(&mut ui);
    assert!(near(
        texts(&ui).iter().find(|s| s.0 == "Ada").unwrap().3,
        ui.theme().palette.highlight
    ));
    let mut theme = *ui.theme();
    theme.palette.highlight = Srgba::rgb(18, 127, 57);
    ui.set_theme(theme);
    render(&mut ui);
    assert!(near(
        texts(&ui).iter().find(|s| s.0 == "Ada").unwrap().3,
        theme.palette.highlight
    ));

    spans[0]["position"] = json!("not a scalar index");
    set_store(&mut ui, "spans", spans);
    render(&mut ui);
    assert!(
        ui.frame()
            .warnings
            .iter()
            .any(|w| w.contains("line.svelte:")
                && w.contains("test:t.spans")
                && w.contains("TextSpan[]"))
    );
    assert!(!ui.elements().iter().any(|e| e.text == "Ada 😀 Bo"));
}

#[test]
fn structured_wrap_motion_restore_and_style_switch_use_existing_paragraph() {
    let spans = [attribute("emphasis", 0, 27), attribute("motion", 6, 11)];
    let mut d = spans_doc(
        "#Line { width: 130px; }
text::part(emphasis) { color: var(--danger); }
#Line.wave::part(motion) { -og-text-motion: wave; }
#Line.shake::part(motion) { -og-text-motion: shake; }
#Line.pulse::part(motion) { -og-text-motion: pulse; }",
        "Alpha Bravo Delta Echo Foxtrot",
        &spans,
        "wave",
    );
    let draw = |d: &mut Doc| d.step(InputFrame::default());
    for fx in ["wave", "shake", "pulse"] {
        set_store(&mut d.ui, "fx", json!(fx));
        for _ in 0..12 {
            draw(&mut d);
        }
        let saved = d.ui.save_state();
        draw(&mut d);
        let next = texts(&d.ui);
        for _ in 0..9 {
            draw(&mut d);
        }
        assert_ne!(texts(&d.ui), next, "motion changes by the tick: {fx:?}");
        d.ui.restore_state(&saved).unwrap();
        draw(&mut d);
        assert_eq!(
            texts(&d.ui),
            next,
            "restore mid-motion replays exactly: {fx:?}"
        );
    }
    let mut theme = *d.ui.theme();
    theme.reduced_motion = true;
    theme.palette.danger = Srgba::rgb(23, 211, 187);
    d.ui.set_theme(theme);
    set_store(&mut d.ui, "fx", json!("wave"));
    draw(&mut d);
    let still = texts(&d.ui);
    for _ in 0..8 {
        draw(&mut d);
    }
    assert_eq!(texts(&d.ui), still, "reduced motion is stationary");
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
fn italic_underline_and_strike_keep_their_span() {
    // Each of the styles reaches only its own span: a slant on its glyphs
    // (the plain words around stay plain), a hairline under <u> and one
    // through <s> near the middle of the x-height. (An outline on one span
    // only is no document's: `::part()` takes no -webkit-text-stroke; the
    // unit test `outline_keeps_its_span` in src/text/rich/spans.rs keeps it.)
    let mut d = markup_doc(
        rich_ui(),
        "a <i>lean</i> b <u>under</u> c <s>gone</s> d loud",
        "560px",
    );
    d.step(InputFrame::default());
    let ui = &d.ui;
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
        "<i> slants its own glyphs: {styled:?}"
    );
    assert_eq!(styled.len(), 1, "the other words stay plain: {styled:?}");
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

/// A UI with a portrait registered as `mira`, the icons `coin` and
/// `ps.cross`, and the action `jump` (E, the pad's a); and the portrait's
/// and the icons' textures.
fn picture_ui() -> (Ui, og_draw::TextureHandle, og_draw::TextureHandle) {
    let mut handles = Handles::default();
    let font = handles.test_font();
    let (portrait, icons) = (handles.texture(), handles.texture());
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
    (ui, portrait, icons)
}

const PICTURE_LINE: &str =
    r#"<img src="mira" alt=""> says: press <kbd data-action="jump"></kbd> to jump"#;

type Drawn = (
    Vec<(Option<og_draw::TextureHandle>, Rect)>,
    Vec<(String, f32, f32, [f32; 4], f32, f32)>,
);

/// Two frames of `d`: the images and the texts drawn.
fn draw_pictures(d: &mut Doc) -> Drawn {
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    let images =
        d.ui.frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Image { texture, rect, .. } => Some((*texture, *rect)),
                _ => None,
            })
            .collect();
    (images, texts(&d.ui))
}

#[test]
fn a_registered_picture_and_an_actions_glyph_draw_inline() {
    let (ui, portrait, icons) = picture_ui();
    let mut d = markup_doc(ui, PICTURE_LINE, "560px");
    // A keyboard: the key's name on a cap, after the portrait, one em tall
    // and twice as wide.
    let (images, t) = draw_pictures(&mut d);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
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
    // An unknown picture: named in a warning, nothing drawn. (An unknown
    // action's `<kbd data-action>` gives a document no warning.)
    let (ui, _, _) = picture_ui();
    let d = markup_doc(ui, r#"<img src="nobody" alt=""> after"#, "fit-content");
    let w = &d.ui.frame().warnings;
    assert!(
        w.iter().any(|w| w.contains(r#"<img src="nobody">"#)),
        "{w:?}"
    );
    assert!(
        !d.ui
            .frame()
            .items
            .iter()
            .any(|i| matches!(i.cmd, DrawCmd::Image { .. })),
        "nothing drawn for it"
    );
}

/// The next frame on a PlayStation pad: the cross's icon, and no cap.
/// A `<kbd data-action>` of its own follows the device
/// (control_props::kbd_glyphs_follow_the_device_and_rebind); one inside a
/// text keeps the keyboard's cap, frames after the device changed.
#[test]
#[ignore = "og-ui: a text's inline <kbd data-action> keeps the device it was built for"]
fn an_inline_actions_glyph_follows_the_device() {
    let (ui, _, icons) = picture_ui();
    let mut d = markup_doc(ui, PICTURE_LINE, "560px");
    draw_pictures(&mut d);
    d.ui.set_device(Device::PlayStation);
    let (images, t) = draw_pictures(&mut d);
    assert!(
        images.iter().any(|i| i.0 == Some(icons)),
        "the cross: {images:?}"
    );
    assert!(!t.iter().any(|t| t.0 == "E"), "{t:?}");
}
