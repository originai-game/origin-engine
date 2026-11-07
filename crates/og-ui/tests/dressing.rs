//! A style dresses widgets' parts (docs/ui.md §12.3), plays its feedback
//! sets (§7.8) and moves every state on its motion tokens.
//!
//! Ways these could fail, each covered below:
//! - the dressing written first wins over a more specific one written
//!   after it, or a less specific one overrides what a more specific one
//!   set (`Button.base` repainting `Button.base[primary][pressed]`);
//! - a game's attribute never reaches the part (`Slot.frame[rarity=epic]`
//!   dresses no cell, or every cell);
//! - a part the style does not dress draws as a bare flat quad (no line,
//!   no elevation) instead of from the tokens;
//! - a hover jumps to its colour in one tick instead of moving on the
//!   style's `ui` spring;
//! - a ghost button at rest draws as a quiet one (no edge, so it does not
//!   read as a button), or a square icon button keeps its label padding;
//! - a set of sounds repeats one sound twice in a row, ignores its volume
//!   or pitch range, differs after a restore, or the player's haptics
//!   setting at 0 still plays the pattern;
//! - a focus ring's cut (what is behind, between the ring and the
//!   control) covers a drawn control's fills.

use crate::support::*;
use og_ui::*;

#[path = "../../../templates/styles/tests/support.rs"]
mod pack_support;
use pack_support::*;

/// A tight field has no inset and its text's line for a height, so a
/// caller's own box holds the line exactly (the 大观园 page's verse box);
/// its style's text parts set the text's step and inks per component,
/// and an `ink` on a frame is refused. Wrong: the text took the
/// body step and ink, the placeholder inkFaint, inside a control's inset.
#[test]
fn a_tight_field_takes_its_components_text_parts() {
    let style = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/storybook/style.ogt"
    ))
    .unwrap();
    let dress = r##"Poem::text {
  -og-ink: #c0392b
  -og-step: lg
}
Poem::placeholder {
  -og-ink: #7f8c8d
}
"##;
    let file = parse_style(&format!("{style}{dress}")).unwrap_or_else(|e| panic!("{e}"));
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let lg = ui.theme().type_ramp.lg;
    let body = ui.theme().type_ramp.body;
    let texts = |ui: &Ui| -> Vec<(String, f32, f32, [f32; 4])> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Text {
                    text,
                    x,
                    size,
                    color,
                    ..
                } => Some((text.clone(), *x, *size, *color)),
                DrawCmd::StyledText { text, .. } => match &**text {
                    DrawCmd::Text {
                        text,
                        x,
                        size,
                        color,
                        ..
                    } => Some((text.clone(), *x, *size, *color)),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    };
    let run = |ui: &mut Ui, poem: &str, plain: &str| {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 300.0));
        ui.column("Page", Element::column(), |ui| {
            ui.text_field(
                "Verse",
                &mut poem.to_string(),
                &FieldDesc {
                    placeholder: "A line".into(),
                    width: Some(Size::Fixed(320.0)),
                    bare: true,
                    tight: true,
                    component: Some("Poem".into()),
                    ..FieldDesc::default()
                },
            );
            ui.text_field("Plain", &mut plain.to_string(), &FieldDesc::default());
        });
        ui.end();
    };
    run(&mut ui, "", "x");
    let verse = ui
        .frame()
        .elements
        .iter()
        .find(|e| e.path == "Page/Verse")
        .cloned()
        .unwrap();
    assert_eq!(
        verse.height, lg.line_height,
        "one line of its lg text, no inset"
    );
    let t = texts(&ui);
    let hint = t.iter().find(|x| x.0 == "A line").expect("the placeholder");
    assert!(
        (hint.1 - verse.x).abs() < 0.5,
        "no inset: {} at {}",
        hint.1,
        verse.x
    );
    assert_eq!(hint.2, lg.size, "the placeholder takes the text's step");
    assert!(
        near(hint.3, og_ui::style_file::parse_color("#7f8c8d").unwrap()),
        "{:?}",
        hint.3
    );
    let plain = t
        .iter()
        .find(|x| x.0 == "x")
        .expect("the plain field's value");
    assert_eq!(plain.2, body.size, "an undressed field keeps the body step");
    assert!(near(plain.3, ui.theme().palette.ink));
    run(&mut ui, "Spring", "x");
    let line = texts(&ui)
        .into_iter()
        .find(|x| x.0 == "Spring")
        .expect("the value");
    assert_eq!(line.2, lg.size);
    assert!(
        near(line.3, og_ui::style_file::parse_color("#c0392b").unwrap()),
        "{:?}",
        line.3
    );
    let e = parse_style(&format!(
        "{style}Poem::frame {{\n  -og-ink: var(--primary)\n}}\n"
    ))
    .unwrap_err();
    assert!(e.contains("text part"), "{e}");
    // On a touch screen a tight field is still a finger's target.
    let touch = Viewport {
        coarse_pointer: true,
        ..Viewport::new(600.0, 300.0)
    };
    ui.begin(InputFrame::default(), touch);
    ui.column("Page", Element::column(), |ui| {
        let tight = FieldDesc {
            bare: true,
            tight: true,
            ..FieldDesc::default()
        };
        ui.text_field("Verse", &mut String::new(), &tight);
        let area = TextAreaDesc {
            rows: 1,
            tight: true,
            ..TextAreaDesc::default()
        };
        ui.text_area("Line", &mut String::new(), &area);
    });
    ui.end();
    for path in ["Page/Verse", "Page/Line"] {
        let e = ui
            .frame()
            .elements
            .iter()
            .find(|e| e.path == path)
            .cloned()
            .unwrap();
        assert!(
            e.height >= 44.0 - 0.01,
            "{path} is {} tall on a touch screen",
            e.height
        );
    }
}

#[test]
fn scroll_bars_and_heading_flourishes_have_independent_dressings() {
    for authored in [false, true] {
        let mut ui = dressed(if authored {
            concat!(
                "Scroll::bar[axis=horizontal] {\n -og-fill: #ff00ff\n height: 7px\n}\n",
                "Scroll::bar[axis=vertical] {\n -og-fill: #00ffff\n width: 9px\n}\n",
                "Heading::flourish {\n width: 12px\n height: 5px\n color: #ff00ff\n opacity: 0.4\n}\n",
            )
        } else {
            ""
        });
        let mut scene = og_render::RenderScene::new();
        let mut fonts = ui.fonts().clone();
        fonts.body = add_test_font(&mut scene);
        ui.set_fonts(fonts);
        let texture = scene.add_texture(og_render::TextureData {
            width: 2,
            height: 2,
            rgba: vec![255; 16],
            color_space: og_render::ColorSpace::Srgb,
            mips: og_render::Mips::None,
        });
        let mut theme = *ui.theme();
        theme.art.atlas = Some(texture);
        theme.art.flourish = Some(og_ui::style_file::Ornament {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 10.0,
            h: 10.0,
            ink: og_ui::style_file::Ink::Authored,
            offset: 0.0,
        });
        ui.set_theme(theme);
        ui.begin(InputFrame::default(), Viewport::new(200.0, 130.0));
        ui.column("Content", Element::column(), |ui| {
            ui.label(
                "Title",
                "Go",
                LabelDesc {
                    heading: true,
                    align: TextAlign::Center,
                    step: Some(TypeStep {
                        size: 16.0,
                        ..theme.type_ramp.body
                    }),
                    ..Default::default()
                },
            );
            ui.scroll(
                "List",
                ScrollDesc {
                    width: Size::Fixed(100.0),
                    height: Size::Fixed(50.0),
                    axes: ScrollAxes::Both,
                    ..Default::default()
                },
                |ui| {
                    ui.open(
                        "Content",
                        Element {
                            width: Size::Fixed(200.0),
                            height: Size::Fixed(120.0),
                            ..Default::default()
                        },
                    );
                    ui.close();
                },
            );
        });
        ui.end();
        let frame = ui.frame();
        assert_eq!(frame.element("List").unwrap().width, 100.0);
        let flourishes: Vec<_> = frame
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Image { rect, color, .. } => Some((*rect, *color)),
                _ => None,
            })
            .collect();
        assert_eq!(
            flourishes.len(),
            if authored { 2 } else { 0 },
            "small headings opt in only when dressed"
        );
        if authored {
            for (rect, color) in flourishes {
                assert_eq!((rect.width, rect.height), (12.0, 5.0));
                assert_eq!(color, Srgba::rgb(255, 0, 255).fade(0.4).linear());
            }
            let fills: Vec<_> = frame
                .items
                .iter()
                .filter_map(|item| item.cmd.filled())
                .collect();
            assert!(
                fills
                    .iter()
                    .any(|(r, c)| r.height == 7.0 && *c == Srgba::rgb(255, 0, 255).linear())
            );
            assert!(
                fills
                    .iter()
                    .any(|(r, c)| r.width == 9.0 && *c == Srgba::rgb(0, 255, 255).linear())
            );
        }
    }
}

#[test]
fn a_disabled_touch_controls_opacity_is_authored_by_its_state_dressing() {
    let mut ui = dressed(concat!(
        "ActionButton::face {\n opacity: 0.8\n}\n",
        "ActionButton::face:disabled {\n opacity: 0.2\n}\n",
    ));
    for disabled in [false, true, false] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.action_button(
            "Fire",
            "Fire",
            &ActionButtonDesc {
                disabled,
                primary: true,
                ..ActionButtonDesc::default()
            },
        );
        ui.end();
        let element = ui.frame().element("Fire").unwrap();
        let rect = Rect::new(element.x, element.y, element.width, element.height);
        let fills = fills_in(&ui, rect);
        assert!(!fills.is_empty(), "the action face is drawn");
        let expected = if disabled { 0.2 } else { 0.8 };
        assert!(
            fills.iter().all(|c| (c[3] - expected).abs() < 0.001),
            "{fills:?}"
        );
    }
}

/// A game's draw (`set_draw`) paints over its element's background: a rim
/// stroked on an element with a background draws after the background
/// quad, so the background never covers it (it showed only at the corners
/// the radius left uncovered).
#[test]
fn a_draw_strokes_over_its_elements_background() {
    let vp = Viewport::new(400.0, 200.0);
    let mut ui = Ui::new(UiDesc::default());
    // Twice: the second frame reuses the background's kept commands.
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.open(
            "Rimmed",
            Element {
                width: Size::Fixed(120.0),
                height: Size::Fixed(60.0),
                background: Some(Srgba::rgb(20, 40, 200)),
                radius: 12.0,
                ..Element::default()
            },
        );
        ui.set_draw(og_ui::paint::ElementDraw {
            strokes: vec![og_ui::paint::Stroke {
                width: 3.0,
                align: og_ui::paint::StrokeAlign::Inside,
                dash: [0.0; 2],
                dash_offset: 0.0,
                dash_cap: og_ui::paint::Cap::Butt,
                paint: og_ui::paint::Paint::Solid(Srgba::rgb(255, 255, 255).into()),
            }],
            ..Default::default()
        });
        ui.close();
        ui.end();
        // Painter order: level, then emission.
        let items = &ui.frame().items;
        let order = |want: &dyn Fn(&DrawCmd) -> bool| {
            items
                .iter()
                .enumerate()
                .find(|(_, it)| want(&it.cmd))
                .map(|(k, it)| (it.level, k))
        };
        let background =
            order(&|c| matches!(c, DrawCmd::Quad { .. })).expect("the background quad");
        let rim = order(&|c| {
            matches!(c, DrawCmd::Shape { shape, .. }
                if matches!(shape.mode, og_render::UiMode::Stroke(_)))
        })
        .expect("the rim");
        assert!(
            rim > background,
            "rim {rim:?} under background {background:?}"
        );
    }
}
