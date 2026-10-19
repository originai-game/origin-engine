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
//!   between two spans sits on the wrong side of its space.

mod support;

use og_core::glam::Vec2;
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
    });
    ui
}

const MARKUP: &str = "Press [b]A[/b] to [color=danger]jump[/color], [size=xl]run[/size] [icon=star] and [link=help]read more[/link].";

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

#[test]
fn spans_keep_their_style_and_links_are_elements() {
    let mut ui = rich_ui();
    for _ in 0..2 {
        step(&mut ui, InputFrame::default(), MARKUP, Size::fit());
    }
    let frame = ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    let hint = frame.element("Hint").expect("the paragraph").clone();
    assert_eq!(
        hint.text, "Press A to jump, run  and read more.",
        "the probe reads the plain text"
    );
    let link = frame
        .element("Hint/help")
        .expect("the link is an element")
        .clone();
    assert_eq!(
        (link.kind.as_str(), link.text.as_str()),
        ("link", "read more")
    );
    assert!(
        link.x >= hint.x && link.x + link.width <= hint.x + hint.width + 0.5,
        "the link lies in its paragraph"
    );
    let t = texts(&ui);
    let theme = *ui.theme();
    let find = |s: &str| {
        t.iter()
            .find(|x| x.0 == s)
            .unwrap_or_else(|| panic!("no piece {s:?} in {t:?}"))
    };
    assert!(
        find("A").5 > 0.0 && find("Press ").5 == 0.0,
        "[b] thickens its glyphs only"
    );
    assert_eq!(
        find("jump").3,
        theme.palette.danger.linear(),
        "[color=danger] takes the token"
    );
    assert_eq!(
        find("run").4,
        theme.type_ramp.xl.size,
        "[size=xl] takes the step"
    );
    assert_eq!(
        find("read more").3,
        theme.palette.accent_ink.linear(),
        "a link is in accent ink"
    );
    let same_baseline = t.iter().map(|x| x.2).fold(f32::NAN, |a, b| {
        if a.is_nan() || a == b {
            b
        } else {
            f32::INFINITY
        }
    });
    assert!(
        same_baseline.is_finite(),
        "every span sits on one baseline: {t:?}"
    );
    let icon = ui.icons().uv(1);
    assert!(
        frame
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Image { uv, .. } if *uv == icon)),
        "[icon=star] draws the star cell"
    );
    let underline = frame.items.iter().any(|i| matches!(&i.cmd, DrawCmd::Quad { rect, .. }
        if rect.x >= link.x - 0.5 && rect.right() <= link.x + link.width + 0.5 && rect.height <= 2.0));
    assert!(underline, "the link is underlined");
}

#[test]
fn lines_break_between_words_and_keep_a_restyled_word_whole() {
    let mut ui = rich_ui();
    let markup = "alpha beta un[b]break[/b]able gamma [size=xl]delta[/size] epsilon zeta";
    for _ in 0..2 {
        step(&mut ui, InputFrame::default(), markup, Size::Fixed(110.0));
    }
    let hint = ui.frame().element("Hint").expect("paragraph").clone();
    let t = texts(&ui);
    let mut baselines: Vec<f32> = t.iter().map(|x| x.2).collect();
    baselines.dedup();
    assert!(baselines.len() >= 3, "it wraps at 110 px: {t:?}");
    for x in &t {
        assert!(x.1 >= hint.x - 0.5, "{x:?} starts inside");
    }
    let line_of = |s: &str| {
        t.iter()
            .find(|x| x.0.contains(s))
            .map(|x| x.2)
            .unwrap_or_else(|| panic!("no {s}"))
    };
    assert_eq!(
        line_of("un"),
        line_of("break"),
        "un|break|able stays on one line"
    );
    assert_eq!(line_of("break"), line_of("able"));
    // The line holding the xl span is taller than a body line.
    let gaps: Vec<f32> = baselines.windows(2).map(|w| w[1] - w[0]).collect();
    let xl_line = baselines
        .iter()
        .position(|&b| b == line_of("delta"))
        .expect("delta's line");
    assert!(
        gaps.iter().copied().fold(0.0, f32::max)
            > gaps.iter().copied().fold(f32::INFINITY, f32::min) + 2.0
            && xl_line > 0,
        "the xl span raises its line: {baselines:?}"
    );
}

#[test]
fn a_link_answers_a_click_and_keyboard_focus() {
    let mut ui = rich_ui();
    for _ in 0..2 {
        step(&mut ui, InputFrame::default(), MARKUP, Size::fit());
    }
    let link = ui.frame().element("Hint/help").expect("link").clone();
    let (x, y) = link.center();
    let p = Vec2::new(x, y);
    let press = InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    };
    let release = InputFrame {
        pointer: p,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    step(&mut ui, InputFrame::hover(x, y), MARKUP, Size::fit());
    let hovered = step(&mut ui, InputFrame::hover(x, y), MARKUP, Size::fit());
    assert_eq!(hovered.hovered.as_deref(), Some("help"));
    step(&mut ui, press, MARKUP, Size::fit());
    let r = step(&mut ui, release, MARKUP, Size::fit());
    assert_eq!(
        r.link.as_deref(),
        Some("help"),
        "a click answers the link's id"
    );
    // Tab reaches it; Enter presses it.
    let tab = InputFrame {
        keys: vec![UiKey::Tab],
        ..InputFrame::default()
    };
    let r = step(&mut ui, tab, MARKUP, Size::fit());
    assert!(!r.response.focused || r.link.is_none());
    let r = step(&mut ui, InputFrame::default(), MARKUP, Size::fit());
    assert!(r.response.focused, "Tab focuses the link");
    let enter = InputFrame {
        keys: vec![UiKey::Enter],
        ..InputFrame::default()
    };
    let r = step(&mut ui, enter, MARKUP, Size::fit());
    assert_eq!(r.link.as_deref(), Some("help"), "Enter presses it");
}

#[test]
fn per_glyph_motion_moves_by_the_tick_and_rests_under_reduced_motion() {
    let ys = |reduced: bool| {
        let mut ui = rich_ui();
        let mut t = *ui.theme();
        t.reduced_motion = reduced;
        ui.set_theme(t);
        let mut out = Vec::new();
        for _ in 0..6 {
            step(
                &mut ui,
                InputFrame::default(),
                "a [wave]wavy[/wave] b",
                Size::fit(),
            );
            out.push(
                texts(&ui)
                    .iter()
                    .filter(|x| x.0.chars().count() == 1)
                    .map(|x| x.2)
                    .collect::<Vec<f32>>(),
            );
        }
        out
    };
    let moving = ys(false);
    assert_eq!(moving[0].len(), 4, "a moving span is drawn glyph by glyph");
    assert!(
        moving.windows(2).any(|w| w[0] != w[1]),
        "the glyphs move between ticks: {moving:?}"
    );
    assert!(
        moving[5].windows(2).any(|w| w[0] != w[1]),
        "each glyph its own phase"
    );
    let still = ys(true);
    assert!(
        still.iter().all(|v| v.is_empty()),
        "under reduced motion the span is one still piece"
    );
    // Two runs draw the same bits.
    assert_eq!(moving, ys(false));
}

#[test]
fn markup_it_cannot_read_shows_as_text_with_a_warning() {
    let mut ui = rich_ui();
    step(
        &mut ui,
        InputFrame::default(),
        "a [blink]b[/blink] [color=nope]c[/color]",
        Size::fit(),
    );
    let w = ui.frame().warnings.join("\n");
    assert!(w.contains("[blink] is not a tag"), "{w}");
    assert!(w.contains("[color=nope]"), "{w}");
    assert_eq!(
        ui.frame().element("Hint").expect("p").text,
        "a [blink]b[/blink] c",
        "the unknown tag stays visible"
    );
}

#[test]
fn a_right_to_left_paragraph_runs_its_spans_from_the_right() {
    let mut ui = rich_ui();
    ui.set_direction(og_i18n::Direction::Rtl);
    for _ in 0..2 {
        step(
            &mut ui,
            InputFrame::default(),
            "one [b]2[/b]. [color=danger]three[/color]",
            Size::fit(),
        );
    }
    let t = texts(&ui);
    let x = |s: &str| {
        t.iter()
            .find(|p| p.0 == s)
            .unwrap_or_else(|| panic!("no piece {s:?} in {t:?}"))
            .1
    };
    // Reading from the right: one, 2, the stop, its space, three.
    assert!(
        x("one ") > x("2") && x("2") > x(" .") && x(" .") > x("three"),
        "the spans run from the right: {t:?}"
    );
}
