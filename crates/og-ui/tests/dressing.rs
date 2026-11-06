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
use og_ui::dress::PartRef;
use og_ui::tokens::{ElementClass, Feedback, Haptic, HapticPreset, SoundEvent};
use og_ui::*;

/// The storybook seed (version 2, no atlas), with `dress` blocks added to
/// its dressings.
fn dressed(dress: &str) -> Ui {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/storybook/style.ogt"
    );
    let text = std::fs::read_to_string(path).expect("the storybook seed");
    let file = parse_style(&format!("{text}{dress}")).unwrap_or_else(|e| panic!("{e}"));
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    ui
}

fn fills_in(ui: &Ui, rect: Rect) -> Vec<[f32; 4]> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(r, _)| {
            (r.x - rect.x).abs() < 1.0
                && (r.y - rect.y).abs() < 1.0
                && (r.width - rect.width).abs() < 1.0
        })
        .map(|(_, c)| c)
        .collect()
}

fn near(a: [f32; 4], b: Srgba) -> bool {
    let l = b.linear();
    (0..3).all(|k| (a[k] - l[k]).abs() < 0.01)
}

#[test]
fn a_disabled_touch_controls_opacity_is_authored_by_its_state_dressing() {
    let mut ui = dressed(concat!(
        "dress \"ActionButton.face\" {\n appearance {\n opacity 0.8\n }\n}\n",
        "dress \"ActionButton.face[disabled]\" {\n appearance {\n opacity 0.2\n }\n}\n",
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

#[test]
fn widget_entrances_use_the_styles_curve_instead_of_a_fixed_curve() {
    for (curve, expected) in [("linear", 1.0 / 3.0), ("steps 4", 0.25)] {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/styles/storybook/style.ogt"
        );
        let source = std::fs::read_to_string(path).unwrap();
        let source = source.replacen(
            "motion {",
            &format!("motion {{\n ease widgetEnter {curve}"),
            1,
        );
        let style = parse_style(&source).unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(style.theme(None, [0; 6]));
        for tick in 0..=20 {
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.transition(
                "Panel",
                tick > 0,
                TransitionDesc {
                    kind: TransitionKind::Fade,
                    duration_ms: 1000.0,
                    ..TransitionDesc::default()
                },
                |ui| {
                    ui.button(
                        "Go",
                        "Go",
                        ButtonDesc {
                            variant: ButtonVariant::Primary,
                            ..ButtonDesc::default()
                        },
                    );
                },
            );
            ui.end();
        }
        let alpha = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .map(|(_, c)| c[3])
            .fold(0.0_f32, f32::max);
        assert!(
            (alpha - expected).abs() < 0.001,
            "{curve}: {alpha} instead of {expected}"
        );
    }
}

#[test]
fn a_parts_spacing_relayouts_its_children_from_style_data() {
    let mut ui =
        dressed("dress \"Card.frame\" {\n appearance {\n padding 13 3 7 11\n gap 17\n }\n}\n");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.open_panel(
        "Row",
        PanelDesc {
            row: true,
            ..PanelDesc::default()
        },
    );
    for (name, width) in [("First", 10.0), ("Second", 20.0)] {
        ui.open(
            name,
            Element {
                width: Size::Fixed(width),
                height: Size::Fixed(15.0),
                ..Element::default()
            },
        );
        ui.close();
    }
    ui.close();
    ui.end();
    let row = ui.frame().element("Row").unwrap();
    let first = ui.frame().element("Row/First").unwrap();
    let second = ui.frame().element("Row/Second").unwrap();
    assert_eq!((row.width, row.height), (67.0, 29.0));
    assert_eq!((first.x - row.x, first.y - row.y), (13.0, 3.0));
    assert_eq!(second.x - first.x - first.width, 17.0);
}

#[test]
fn meter_trails_and_separators_take_their_own_dressings() {
    let mut ui = dressed(concat!(
        "dress \"Meter.trail[loss]\" {\n fill \"#ff00ff\"\n}\n",
        "dress \"Meter.separator\" {\n fill \"#00ffff\"\n}\n",
    ));
    for value in [80.0, 20.0] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.meter(
            "Health",
            value,
            100.0,
            &MeterDesc {
                width: Some(Size::Fixed(100.0)),
                height: 10.0,
                segments: 4,
                ..MeterDesc::default()
            },
        );
        ui.end();
    }
    let fills: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .collect();
    let trails: Vec<_> = fills
        .iter()
        .filter(|(_, c)| near(*c, Srgba::rgb(255, 0, 255)))
        .collect();
    assert_eq!(trails.len(), 1);
    assert_eq!(trails[0].0.width, 80.0);
    let separators: Vec<_> = fills
        .iter()
        .filter(|(_, c)| near(*c, Srgba::rgb(0, 255, 255)))
        .collect();
    assert_eq!(separators.len(), 3);
    assert!(separators.iter().all(|(r, _)| r.width == 2.0));
}

#[test]
fn a_joystick_knob_can_be_recoloured_without_changing_its_radius() {
    let mut ui = dressed("dress \"Joystick.knob\" {\n fill \"#ff00ff\"\n}\n");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.joystick(
        "Move",
        JoystickDesc {
            size: 150.0,
            ..JoystickDesc::default()
        },
    );
    ui.end();
    let knob = ui
        .frame()
        .items
        .iter()
        .find_map(|item| {
            let (rect, color) = item.cmd.filled()?;
            if !near(color, Srgba::rgb(255, 0, 255)) {
                return None;
            }
            let radius = match &item.cmd {
                DrawCmd::Quad { radius, .. } => *radius,
                DrawCmd::Shape { shape, .. } => match shape.geometry {
                    og_render::UiGeometry::Rect { radii, .. } => radii[0],
                    _ => return None,
                },
                _ => return None,
            };
            Some((rect, radius.min(rect.width / 2.0).min(rect.height / 2.0)))
        })
        .expect("the styled knob is drawn");
    assert_eq!(knob.0.width, 58.0);
    assert_eq!(knob.0.height, 58.0);
    assert_eq!(knob.1, 29.0);
}

#[test]
fn a_decorations_size_and_offset_do_not_resize_its_controls_hit_box() {
    for (width, expected_width) in [("40", 40.0), ("percent 0.2", 30.0)] {
        let mut ui = dressed(&format!(
            "dress \"Joystick.knob\" {{\n fill \"#ff00ff\"\n appearance {{\n width {width}\n height 24\n x 9\n y -6\n opacity 0.5\n }}\n}}\n"
        ));
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.joystick(
            "Move",
            JoystickDesc {
                size: 150.0,
                ..JoystickDesc::default()
            },
        );
        ui.end();
        let knob = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| item.cmd.filled())
            .find(|(_, color)| near(*color, Srgba::rgb(255, 0, 255)))
            .expect("the styled knob is drawn");
        assert_eq!((knob.0.width, knob.0.height), (expected_width, 24.0));
        assert_eq!(knob.0.center(), (84.0, 69.0));
        assert_eq!(knob.1[3], 0.5);
        let control = ui.frame().element("Move").unwrap();
        assert_eq!((control.width, control.height), (150.0, 150.0));
        ui.begin(
            InputFrame {
                pointer: og_core::glam::Vec2::new(140.0, 75.0),
                pointer_inside: true,
                pressed: true,
                down: true,
                ..InputFrame::default()
            },
            Viewport::new(800.0, 600.0),
        );
        let response = ui.joystick(
            "Move",
            JoystickDesc {
                size: 150.0,
                ..JoystickDesc::default()
            },
        );
        ui.end();
        assert!(
            response.active,
            "the base still takes a press outside the restyled knob"
        );
    }
}

#[test]
fn a_parts_layout_size_and_translation_are_style_data() {
    let mut ui = dressed(
        "dress \"Card.frame\" {\n appearance {\n width 120\n height 55\n x 9\n y 7\n }\n}\n",
    );
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.open_panel(
        "Panel",
        PanelDesc {
            width: Size::Fixed(80.0),
            height: Size::Fixed(40.0),
            ..PanelDesc::default()
        },
    );
    ui.close();
    ui.end();
    let panel = ui.frame().element("Panel").unwrap();
    assert_eq!((panel.width, panel.height), (120.0, 55.0));
    assert_eq!((panel.x, panel.y), (9.0, 7.0));
}

#[test]
fn embedded_marks_sectors_and_labels_read_their_dressings() {
    let mut ui = dressed(concat!(
        "dress \"Checkbox.mark[stroke=long]\" {\n appearance {\n color \"#ff00ff\"\n width 17\n height 3\n opacity 0.4\n }\n}\n",
        "dress \"Cooldown.sweep\" {\n appearance {\n color \"#00ffff\"\n }\n}\n",
        "dress \"Cooldown.label\" {\n appearance {\n color \"#ff00ff\"\n step caption\n }\n}\n",
    ));
    ui.set_fonts(Fonts {
        body: add_test_font(&mut og_render::RenderScene::new()),
        ..Fonts::default()
    });
    for remaining in [0.75, 0.25, 0.0] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.checkbox("Check", &mut true, "Check");
        ui.cooldown(
            "Skill",
            remaining,
            &CooldownDesc {
                label: "Q".into(),
                size: 64.0,
                ..CooldownDesc::default()
            },
        );
        ui.end();
        let commands: Vec<_> = ui.frame().items.iter().map(|item| &item.cmd).collect();
        assert!(commands.iter().any(|cmd| matches!(cmd,
            DrawCmd::Bar { w, h, color, .. }
                if *w == 17.0 && *h == 3.0 && near(*color, Srgba::rgb(255, 0, 255)) && color[3] == 0.4
        )));
        assert!(
            commands.iter().any(|cmd| matches!(cmd,
                DrawCmd::Text { text, size, color, .. }
                    if text == "Q" && *size == 14.0 && near(*color, Srgba::rgb(255, 0, 255))
            )),
            "{commands:?}"
        );
        if remaining > 0.0 {
            assert!(commands.iter().any(|cmd| matches!(cmd,
                DrawCmd::Half { radius, color, .. }
                    if *radius == 32.0 && near(*color, Srgba::rgb(0, 255, 255))
            )));
        } else {
            assert!(commands.iter().any(|cmd| {
                cmd.filled().is_some_and(|(r, color)| {
                    r.width == 64.0 && near(color, Srgba::rgb(0, 255, 255))
                })
            }));
        }
    }
}

#[test]
fn slider_and_toggle_knob_geometry_is_style_data() {
    let mut ui = dressed(concat!(
        "dress \"Slider.knob\" {\n fill \"#ff00ff\"\n appearance {\n width 9\n height 31\n }\n}\n",
        "dress \"Toggle.knob\" {\n fill \"#ff00ff\"\n appearance {\n width 9\n height 31\n }\n}\n",
    ));
    ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
    ui.slider(
        "Volume",
        &mut 0.25,
        SliderDesc {
            width: Some(Size::Fixed(120.0)),
            ..SliderDesc::default()
        },
    );
    ui.toggle("Music", &mut true, "Music");
    ui.end();
    let knobs: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|item| item.cmd.filled())
        .filter(|(_, color)| near(*color, Srgba::rgb(255, 0, 255)))
        .collect();
    assert_eq!(knobs.len(), 2);
    assert!(
        knobs
            .iter()
            .all(|(rect, _)| rect.width == 9.0 && rect.height == 31.0)
    );
    assert_eq!(ui.frame().element("Volume").unwrap().width, 120.0);
}

#[test]
fn icon_geometry_and_tint_apply_to_atlases_and_glyphs() {
    for glyph in [false, true] {
        let mut ui = dressed(
            "dress \"Icon.image\" {\n appearance {\n width 18\n height 26\n x 3\n y -2\n color \"#00ffff\"\n opacity 0.3\n }\n}\n",
        );
        let mut scene = og_render::RenderScene::new();
        let texture = scene.add_texture(og_render::TextureData {
            width: 2,
            height: 2,
            rgba: vec![255; 16],
            color_space: og_render::ColorSpace::Srgb,
            mips: og_render::Mips::None,
        });
        ui.set_icons(IconAtlas {
            names: vec!["star".into()],
            texture: Some(texture),
            font: glyph.then(|| add_test_font(&mut scene)),
            ..IconAtlas::default()
        });
        ui.begin(InputFrame::default(), Viewport::new(100.0, 100.0));
        ui.icon(
            "Star",
            "star",
            IconDesc {
                size: 40.0,
                ..IconDesc::default()
            },
        );
        ui.end();
        let (rect, color) = ui
            .frame()
            .items
            .iter()
            .find_map(|item| match &item.cmd {
                DrawCmd::Image { rect, color, .. } if !glyph => Some((*rect, *color)),
                DrawCmd::Text { bounds, color, .. } if glyph => Some((*bounds, *color)),
                _ => None,
            })
            .unwrap();
        assert_eq!(rect.center(), (23.0, 18.0));
        assert_eq!(rect.width, 18.0);
        assert_eq!(rect.height, if glyph { 18.0 } else { 26.0 });
        assert!(near(color, Srgba::rgb(0, 255, 255)));
        assert_eq!(color[3], 0.3);
        assert_eq!(ui.frame().element("Star").unwrap().width, 40.0);
    }
}

#[test]
fn the_most_specific_dressing_wins_whatever_order_the_style_writes_them() {
    let ui = dressed(concat!(
        "dress \"Button.base[primary][pressed]\" {\n  fill \"#00ff00\"\n}\n",
        "dress \"Button.base\" {\n  fill \"#ff0000\"\n  stroke 3 \"#ffffff\"\n}\n",
        "dress \"Button.base[primary]\" {\n  fill \"#0000ff\"\n}\n",
    ));
    let fill = |sel: &str| {
        let part = PartRef::parse(sel, ElementClass::Button).unwrap();
        let r = ui.resolve_part(&part);
        (format!("{:?}", r.draw.fills), r.draw.strokes.len())
    };
    let (pressed, strokes) = fill("Button.base[primary][pressed]");
    assert!(
        pressed.contains("[0, 255, 0]"),
        "pressed primary: {pressed}"
    );
    assert_eq!(strokes, 1, "and keeps the base's stroke it does not set");
    assert!(fill("Button.base[primary]").0.contains("[0, 0, 255]"));
    assert!(
        fill("Button.base[secondary][pressed]")
            .0
            .contains("[255, 0, 0]")
    );
}

#[test]
fn a_games_attribute_dresses_only_the_cells_that_carry_it() {
    let epic = Srgba::rgb(0xa3, 0x35, 0xee);
    let mut ui = dressed("dress \"Slot.frame[rarity=epic]\" {\n  fill \"#a335ee\"\n}\n");
    let build = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        let range = ui.open_grid(
            "Bag",
            GridDesc {
                count: 2,
                cell_width: 64.0,
                columns: 2,
                ..GridDesc::default()
            },
        );
        for i in range.first..range.last {
            ui.grid_item(i, false);
            if i == 0 {
                ui.part_attr("rarity", "epic");
            }
            ui.close();
        }
        ui.close_list();
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui);
    }
    let cell = |k: u32| {
        let e = ui
            .frame()
            .element(&format!("Bag/Cell#{k}"))
            .unwrap_or_else(|| panic!("cell {k}"))
            .clone();
        Rect::new(e.x, e.y, e.width, e.height)
    };
    assert!(
        fills_in(&ui, cell(0)).iter().any(|c| near(*c, epic)),
        "the epic cell draws the dressing's fill"
    );
    assert!(
        !fills_in(&ui, cell(1)).iter().any(|c| near(*c, epic)),
        "a plain cell does not"
    );
}

#[test]
fn part_attr_answers_whether_the_style_tells_the_attribute_apart() {
    // The style dresses one tier: every tier answers true (its frame carries
    // the tier, so a caller adds no second stroke); an attribute it never
    // dresses answers false (the caller draws its own mark).
    let mut ui = dressed("dress \"Slot.frame[rarity=epic]\" {\n  fill \"#a335ee\"\n}\n");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    let range = ui.open_grid(
        "Bag",
        GridDesc {
            count: 3,
            cell_width: 64.0,
            columns: 3,
            ..GridDesc::default()
        },
    );
    let mut answers = Vec::new();
    for i in range.first..range.last {
        ui.grid_item(i, false);
        answers.push(match i {
            0 => ui.part_attr("rarity", "epic"),
            1 => ui.part_attr("rarity", "rare"),
            _ => ui.part_attr("team", "red"),
        });
        ui.close();
    }
    ui.close_list();
    ui.end();
    assert_eq!(answers, [true, true, false]);
}

#[test]
fn an_undressed_panel_draws_its_line_and_elevation_from_the_tokens() {
    let mut ui = ui_with_font(test_font_metrics());
    // No dressings at all (the default style dresses cards flat).
    ui.set_dressings(Vec::new());
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.panel("Card", PanelDesc::default(), |ui| {
        ui.label("T", "Title", LabelDesc::default());
    });
    ui.end();
    let modes: Vec<&og_render::UiMode> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } => Some(&shape.mode),
            _ => None,
        })
        .collect();
    assert!(
        modes
            .iter()
            .any(|m| matches!(m, og_render::UiMode::Stroke(_))),
        "its line: {modes:?}"
    );
    assert!(
        modes
            .iter()
            .any(|m| matches!(m, og_render::UiMode::Shadow { .. })),
        "its elevation's shadow: {modes:?}"
    );
}

#[test]
fn an_undressed_surface_part_with_no_fill_draws_no_outline() {
    // A pattern names its layout groups as parts (so a style can dress
    // them); undressed and unfilled they are invisible, not hairline boxes.
    let strokes = |background: Option<Srgba>| {
        let mut ui = ui_with_font(test_font_metrics());
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        let el = Element {
            part: PartRef::parse("Group.frame", ElementClass::Panel).ok(),
            background,
            ..Element::column()
        };
        ui.column("Group", el, |ui| {
            ui.label("T", "Title", LabelDesc::default());
        });
        ui.end();
        ui.frame()
            .items
            .iter()
            .filter(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_))))
            .count()
    };
    assert_eq!(strokes(None), 0, "a bare group draws no line");
    assert!(
        strokes(Some(Theme::default().palette.panel)) > 0,
        "a filled panel keeps its line"
    );
}

#[test]
fn a_rim_draws_before_content_and_raised_badges() {
    // The same ordering applies to game and style draws: the rim precedes
    // content, and a raised corner badge is above both.
    let mut ui = ui_with_font(test_font_metrics());
    let (blue, gold) = (Srgba::rgb(0, 0, 255), Srgba::rgb(255, 200, 0));
    ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
    ui.open(
        "Card",
        Element {
            width: Size::Fixed(200.0),
            height: Size::Fixed(120.0),
            ..Element::column()
        },
    );
    ui.set_draw(og_ui::paint::ElementDraw {
        fills: vec![og_ui::paint::Paint::Solid(Srgba::rgb(50, 50, 50).into())],
        strokes: vec![og_ui::paint::Stroke {
            width: 3.0,
            align: og_ui::paint::StrokeAlign::Inside,
            dash: [0.0; 2],
            dash_offset: 0.0,
            dash_cap: og_ui::paint::Cap::Butt,
            paint: og_ui::paint::Paint::Solid(Srgba::rgb(0, 255, 0).into()),
        }],
        ..Default::default()
    });
    for (id, z, color) in [("Tag", 1, gold), ("Art", 0, blue)] {
        ui.open_with(
            id,
            Element {
                width: Size::Fixed(40.0),
                height: Size::Fixed(20.0),
                background: Some(color),
                ..Element::default()
            },
            Props {
                z,
                ..Props::default()
            },
        );
        ui.close();
    }
    ui.close();
    ui.end();
    let items = &ui.frame().items;
    let at = |want: Srgba| {
        items
            .iter()
            .position(|i| i.cmd.filled().is_some_and(|(_, c)| near(c, want)))
            .expect("drawn")
    };
    let stroke = items
        .iter()
        .position(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_))))
        .expect("the border");
    assert!(stroke < at(blue), "the rim precedes content");
    assert!(at(blue) < at(gold), "a raised badge is above both");
}

#[test]
fn a_hover_moves_the_buttons_fill_on_the_ui_spring() {
    let mut ui = ui_with_font(test_font_metrics());
    let p = Theme::default().palette;
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                variant: ButtonVariant::Primary,
                ..ButtonDesc::default()
            },
        );
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui, InputFrame::default());
    }
    let e = ui.frame().element("Go").unwrap().clone();
    let rect = Rect::new(e.x, e.y, e.width, e.height);
    let (x, y) = e.center();
    let fill = |ui: &Ui| fills_in(ui, rect)[0];
    assert!(near(fill(&ui), p.primary), "at rest: primary");
    build(&mut ui, InputFrame::hover(x, y));
    build(&mut ui, InputFrame::hover(x, y));
    let mid = fill(&ui);
    assert!(
        !near(mid, p.primary) && !near(mid, p.primary_hover),
        "two ticks in it is on its way: {mid:?}"
    );
    for _ in 0..90 {
        build(&mut ui, InputFrame::hover(x, y));
    }
    assert!(near(fill(&ui), p.primary_hover), "and arrives");
}

#[test]
fn a_ghost_button_shows_its_edge_at_rest_and_a_square_one_is_square() {
    let strokes = |variant: ButtonVariant| {
        let mut ui = ui_with_font(test_font_metrics());
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                variant,
                square: true,
                ..ButtonDesc::default()
            },
        );
        ui.end();
        let e = ui.frame().element("Go").unwrap().clone();
        assert!((e.width - e.height).abs() < 0.5, "square: {e:?}");
        ui.frame()
            .items
            .iter()
            .filter(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_))))
            .count()
    };
    assert_eq!(
        strokes(ButtonVariant::Quiet),
        0,
        "a quiet button is text at rest"
    );
    assert!(
        strokes(ButtonVariant::Ghost) > 0,
        "a ghost button shows its outline at rest"
    );
}

#[test]
fn an_invalid_field_draws_the_danger_outline_from_the_tokens() {
    let danger = |invalid: bool| {
        let mut ui = ui_with_font(test_font_metrics());
        let mut text = String::from("x");
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.text_field(
            "Name",
            &mut text,
            &FieldDesc {
                invalid,
                ..FieldDesc::default()
            },
        );
        ui.end();
        let d = Theme::default().palette.danger.linear();
        ui.frame().items.iter().any(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } => {
                matches!(shape.mode, og_render::UiMode::Stroke(_))
                    && matches!(shape.paint, og_render::UiPaint::Solid(c) if (0..3).all(|k| (c[k] - d[k]).abs() < 0.02))
            }
            _ => false,
        })
    };
    assert!(!danger(false), "a valid field's outline is its line");
    assert!(danger(true), "an invalid one's is danger");
}

#[test]
fn a_bare_field_draws_no_frame_of_its_own() {
    // Wrong: a bare field still filling its box or drawing its outline (a
    // frame in the component's frame).
    let mut ui = ui_with_font(test_font_metrics());
    let mut text = String::from("x");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.text_field(
        "Name",
        &mut text,
        &FieldDesc {
            bare: true,
            ..FieldDesc::default()
        },
    );
    ui.end();
    let e = ui.frame().element("Name").unwrap().clone();
    let rect = Rect::new(e.x, e.y, e.width, e.height);
    assert!(fills_in(&ui, rect).is_empty(), "no fill");
    assert!(
        !ui.frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_)))),
        "no outline"
    );
}

fn theme_with_set() -> Theme {
    let mut t = Theme::default();
    t.tokens.feedback[SoundEvent::Select.index()] = Feedback {
        sounds: [11, 12, 13, 0],
        count: 3,
        volume: [0.5, 0.8],
        pitch: [0.9, 1.2],
        haptic: Haptic::preset(HapticPreset::Tap),
    };
    t
}

fn plays(ui: &mut Ui, n: usize) -> Vec<FeedbackPlay> {
    for _ in 0..n {
        ui.feedback(SoundEvent::Select);
    }
    ui.take_feedback()
}

#[test]
fn a_sound_set_never_repeats_and_plays_within_its_ranges() {
    let mut ui = Ui::default();
    ui.set_theme(theme_with_set());
    let saved = ui.save_state();
    let a = plays(&mut ui, 60);
    assert_eq!(a.len(), 60);
    for w in a.windows(2) {
        assert_ne!(w[0].sound, w[1].sound, "the same sound twice in a row");
    }
    for s in [11, 12, 13] {
        assert!(a.iter().any(|f| f.sound == s), "{s} never plays");
    }
    assert!(a.iter().all(|f| (0.5..=0.8).contains(&f.volume)));
    assert!(a.iter().all(|f| (0.9..=1.2).contains(&f.pitch)));
    let lo = a.iter().map(|f| f.pitch).fold(f32::MAX, f32::min);
    let hi = a.iter().map(|f| f.pitch).fold(f32::MIN, f32::max);
    assert!(
        hi - lo > 0.15,
        "the pitch spreads over its range: {lo}..{hi}"
    );
    // The choices are UI state: a restore replays them.
    ui.restore_state(&saved).unwrap();
    let b = plays(&mut ui, 60);
    let key = |v: &[FeedbackPlay]| -> Vec<(u64, u32)> {
        v.iter().map(|f| (f.sound, f.pitch.to_bits())).collect()
    };
    assert_eq!(key(&a), key(&b));
    // A rising cascade climbs the range from its bottom to its top.
    ui.feedback_rising(SoundEvent::Select, 0.0);
    ui.feedback_rising(SoundEvent::Select, 1.0);
    let rise = ui.take_feedback();
    assert_eq!((rise[0].pitch, rise[1].pitch), (0.9, 1.2));
}

#[test]
fn the_players_haptics_setting_scales_the_pattern_and_zero_turns_it_off() {
    let mut ui = Ui::default();
    ui.set_theme(theme_with_set());
    let full = plays(&mut ui, 1)[0].haptic;
    assert!(full.count > 0);
    ui.set_haptics(0.5);
    let half = plays(&mut ui, 1)[0].haptic;
    assert_eq!(half.count, full.count);
    let (f, h) = (full.pulses()[0], half.pulses()[0]);
    assert!((h.high - f.high * 0.5).abs() < 1e-6 && (h.low - f.low * 0.5).abs() < 1e-6);
    ui.set_haptics(0.0);
    let off = plays(&mut ui, 1);
    assert_eq!(off[0].haptic.count, 0, "off plays no pattern");
    assert!(off[0].sound != 0, "but still its sound");
}

/// tooltip a level (five).
#[test]
fn a_parts_border_draws_under_its_label() {
    let (shot, _) = with_test_font(sample_screen_shot);
    assert!(
        shot.ui.frame().levels() <= 4,
        "{} levels",
        shot.ui.frame().levels()
    );
}

/// which drew them by the `chip` class (Lantern Vale's settings on metal).
#[test]
fn knobs_and_switch_tracks_stay_round_in_a_square_style() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../templates/archetypes/roguelike/assets/styles/lampdive/style.ogt");
    let file = parse_style(&std::fs::read_to_string(dir).unwrap()).unwrap();
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let (mut v, mut on) = (0.5, true);
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.slider("S", &mut v, SliderDesc::default());
    ui.toggle("T", &mut on, "Reduce motion");
    ui.end();
    // Every fill drawn for the slider's knob and the switch (shorter than
    // the slider's 4 px track): its corner reaches half its short side.
    let fills: Vec<(f32, f32)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } if shape.mode == og_render::UiMode::Fill => {
                let og_render::UiGeometry::Rect { radii, .. } = shape.geometry else {
                    return None;
                };
                Some((radii[0], shape.half.x.min(shape.half.y)))
            }
            _ => None,
        })
        .filter(|&(_, half)| half > 4.0)
        .collect();
    assert_eq!(
        fills.len(),
        3,
        "the knob, the track and its knob: {fills:?}"
    );
    for (radius, half) in fills {
        assert!(
            radius >= half,
            "a corner of {radius} on a half side of {half}"
        );
    }
}

/// a background.
#[test]
fn an_anchor_without_a_background_draws_no_frame() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    for seed in ["storybook", "kawaii"] {
        let text = std::fs::read_to_string(dir.join(seed).join("style.ogt")).unwrap();
        let file = parse_style(&text).unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(file.theme(None, [0; 6]));
        ui.set_dressings(file.dressings.clone());
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.anchor("Score", AnchorDesc::default(), |ui| {
            ui.label("Value", "680", LabelDesc::default());
        });
        ui.end();
        let drawn: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter(|it| !matches!(it.cmd, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }))
            .map(|it| format!("{:?}", std::mem::discriminant(&it.cmd)))
            .collect();
        assert!(drawn.is_empty(), "{seed}: {drawn:?}");
    }
}

/// A focus ring is a hollow stroke after the fill, not a pair of opaque
/// quads whose cut would cover a translucent or painted control.
#[test]
fn a_focus_ring_is_hollow_and_drawn_after_the_fill() {
    let mut ui = ui_with_font(test_font_metrics());
    let red = Srgba::rgb(230, 26, 26);
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.open(
            "Card",
            Element {
                width: Size::Fixed(120.0),
                height: Size::Fixed(40.0),
                interactive: true,
                focusable: true,
                ..Element::row()
            },
        );
        ui.set_draw(og_ui::paint::ElementDraw {
            fills: vec![og_ui::paint::Paint::Solid(red.into())],
            ..Default::default()
        });
        ui.close();
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui, InputFrame::default());
    }
    build(
        &mut ui,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::default()
        },
    );
    build(&mut ui, InputFrame::default());
    let e = ui.frame().element("Card").unwrap().clone();
    // Every fill that covers the card's middle, in drawing order: the last one is what shows.
    let (cx, cy) = e.center();
    let over: Vec<[f32; 4]> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(r, _)| r.contains(cx, cy) && r.width < 200.0)
        .map(|(_, c)| c)
        .collect();
    assert_eq!(over.len(), 1, "only the control fills its middle: {over:?}");
    assert!(
        near(*over.last().unwrap(), red),
        "the card's own fill remains visible: {over:?}"
    );
    let items = &ui.frame().items;
    let fill = items
        .iter()
        .position(|i| i.cmd.filled().is_some_and(|(_, c)| near(c, red)))
        .unwrap();
    let ring = items.iter().position(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. }
        if matches!(shape.mode, og_render::UiMode::Stroke(s) if s.align == og_render::UiAlign::Outside))).unwrap();
    assert!(
        fill < ring,
        "focus is above the control's fill and selection"
    );
}

/// A component's own look for a part (`Ui::set_draw_dressable`): a style
/// that dresses the part repaints it and the component's geometry stays;
/// a game's plain `set_draw` is never restyled. Ways it could fail: the
/// dressing ignored (the component's blue drawn in every style), the arc
/// replaced by the dressing's rectangle, or a plain draw repainted too.
#[test]
fn a_components_dressable_draw_takes_the_dressing_and_keeps_its_shape() {
    use og_ui::paint::{Cap, ColorRef, ElementDraw, Paint, ShapeKind};
    let gold = Srgba::rgb(0xff, 0xd3, 0x4a);
    let blue = Srgba::rgb(0, 0, 255);
    let mut ui = dressed("dress \"Gauge.ring\" {\n  fill \"#ffd34a\"\n}\n");
    let ring = || ElementDraw {
        shape: ShapeKind::Arc {
            start: 0.0,
            sweep: 3.0,
            thickness: 6.0,
            cap: Cap::Round,
        },
        fills: vec![Paint::Solid(ColorRef::Literal(blue))],
        ..ElementDraw::default()
    };
    let el = || Element {
        width: Size::px(48.0),
        height: Size::px(48.0),
        part: Some(PartRef::parse("Gauge.ring", ElementClass::Bar).unwrap()),
        ..Element::default()
    };
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.open("Dressable", el());
    ui.set_draw_dressable(ring());
    ui.close();
    ui.open("Plain", el());
    ui.set_draw(ring());
    ui.close();
    ui.end();
    let arcs: Vec<[f32; 4]> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if shape.mode == og_render::UiMode::Fill
                    && matches!(shape.geometry, og_render::UiGeometry::Arc { .. }) =>
            {
                it.cmd.filled().map(|(_, c)| c)
            }
            _ => None,
        })
        .collect();
    assert_eq!(arcs.len(), 2, "both rings keep their arc: {arcs:?}");
    assert!(
        near(arcs[0], gold),
        "the dressable ring takes the style's fill: {:?}",
        arcs[0]
    );
    assert!(
        near(arcs[1], blue),
        "the plain draw keeps the game's: {:?}",
        arcs[1]
    );
}

/// A part with no element class is a game's decoration: undressed, it
/// takes nothing from the tokens (no outline, no elevation, no recess),
/// with or without a fill of its own, and keeps its element's corner. A
/// part that declares itself a card keeps the card's hairline and raised
/// shadow, and a stroke the style dresses a classless part with draws.
///
/// Ways it could fail: a classless part falls back to a card (the old
/// default: the hairline round U6c's unit-frame scrim and U6d's saga
/// nodes), or a declared card loses its outline with the fix, or the
/// style's own dressing stops reaching a classless part.
#[test]
fn a_part_without_a_class_takes_nothing_from_the_tokens() {
    use og_render::UiMode;
    let modes = |ui: &mut Ui, sel: &str, class: Option<ElementClass>, fill: Option<Srgba>| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.open(
            "Part",
            Element {
                width: Size::px(100.0),
                height: Size::px(60.0),
                background: fill,
                radius: 6.0,
                part: Some(PartRef::parse(sel, class).unwrap()),
                ..Element::default()
            },
        );
        ui.close();
        ui.end();
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => Some(shape.mode),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let stroke = |m: &[UiMode]| m.iter().any(|m| matches!(m, UiMode::Stroke(_)));
    let shadow = |m: &[UiMode]| m.iter().any(|m| matches!(m, UiMode::Shadow { .. }));
    let mut ui = dressed("dress \"Node.ring\" {\n  stroke 2 \"#ffd34a\" inside\n}\n");
    let ground = Some(Srgba::rgb(40, 40, 40));
    let bare = modes(&mut ui, "UnitFrame.scrim", None, None);
    assert!(
        bare.is_empty(),
        "a classless part without a fill draws {bare:?}"
    );
    let filled = modes(&mut ui, "UnitFrame.scrim", None, ground);
    assert_eq!(
        filled,
        vec![UiMode::Fill],
        "a classless part draws only its fill"
    );
    let card = modes(&mut ui, "Shop.card", Some(ElementClass::Card), ground);
    assert!(
        stroke(&card) && shadow(&card),
        "a declared card keeps its hairline and shadow: {card:?}"
    );
    let ring = modes(&mut ui, "Node.ring", None, None);
    assert!(
        ring.iter()
            .any(|m| matches!(m, UiMode::Stroke(s) if (s.width - 2.0).abs() < 0.01)),
        "the style's stroke on a classless part draws: {ring:?}"
    );
}

/// setDraw draws the element added last: a second call meant for its
/// parent, made after a childless picture closed, replaces the picture's
/// draw (the CG gallery's tiles lost their art), so og-ui names it.
#[test]
fn a_set_draw_that_replaces_a_draw_set_this_tick_is_named() {
    let solid = |c: Srgba| og_ui::paint::ElementDraw {
        fills: vec![og_ui::paint::Paint::Solid(c.into())],
        ..Default::default()
    };
    let mut ui = ui_with_font(test_font_metrics());
    ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
    ui.open("Tile", Element::column());
    ui.open(
        "Picture",
        Element {
            width: Size::Fixed(120.0),
            height: Size::Fixed(80.0),
            ..Element::default()
        },
    );
    ui.set_draw(solid(Srgba::rgb(0, 0, 255)));
    ui.close();
    // Meant for the tile, but the picture is the element added last.
    ui.set_draw(solid(Srgba::rgb(0, 0, 0)));
    ui.close();
    ui.end();
    let w = &ui.frame().warnings;
    assert!(
        w.iter()
            .any(|m| m.contains("setDraw replaced") && m.contains("Tile/Picture")),
        "{w:?}"
    );
}

/// A style's display drop is one hard shadow a few pixels off at most, and
/// none under an element that draws its own text effects (a HUD outline
/// and halo), which wrap every text command: there a style copy printed a
/// second outlined, haloed glyph (cartoon's 3rd, 2,450 and the ×3 badge).
#[test]
fn a_style_drop_never_prints_a_second_glyph() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let text = std::fs::read_to_string(dir.join("cartoon").join("style.ogt")).unwrap();
    let file = parse_style(&text).unwrap();
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let step = TypeStep {
        size: 18.0,
        line_height: 22.0,
        ..ui.theme().type_ramp.body
    };
    let run = |ui: &mut Ui, own: bool| {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
        ui.label(
            "Value",
            "x3",
            LabelDesc {
                step: Some(step),
                font: FontRole::Display,
                ..LabelDesc::default()
            },
        );
        if own {
            ui.set_draw(og_ui::paint::ElementDraw {
                text: Some(og_ui::paint::TextEffects {
                    shadows: Vec::new(),
                    outline: None,
                    fill: None,
                }),
                ..Default::default()
            });
        }
        ui.end();
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Text { x, baseline, .. } => Some((*x, *baseline)),
                DrawCmd::StyledText { text, .. } => match &**text {
                    DrawCmd::Text { x, baseline, .. } => Some((*x, *baseline)),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let plain = run(&mut ui, false);
    assert_eq!(plain.len(), 2, "the drop and the glyph: {plain:?}");
    let (dx, dy) = (plain[0].0 - plain[1].0, plain[0].1 - plain[1].1);
    assert!(
        dx.hypot(dy) <= 18.0 * 0.06 + 0.51,
        "a tight drop at 18 px: {dx}, {dy}"
    );
    let own = run(&mut ui, true);
    assert_eq!(own.len(), 1, "its own effects, one glyph: {own:?}");
}

/// In a pixel font (`pixel 12`) a text's effects sit on its pixels: the
/// style's drop one font pixel off and hard, an element's own outline and
/// shadow whole font pixels (at least one), never a fractional stroke that
/// softens the cells' edges. At 24 px a font pixel is 2 px.
#[test]
fn a_pixel_fonts_effects_are_whole_pixels() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let text = std::fs::read_to_string(dir.join("cartoon").join("style.ogt")).unwrap();
    let file = parse_style(&text).unwrap();
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    let mut fonts = ui.fonts().clone();
    for f in [&mut fonts.body].into_iter().chain(fonts.display.as_mut()) {
        f.metrics = f.metrics.clone().with_pixel(12);
    }
    fonts.link();
    ui.set_fonts(fonts);
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let step = TypeStep {
        size: 24.0,
        line_height: 30.0,
        ..ui.theme().type_ramp.body
    };
    let run = |ui: &mut Ui, own: bool| {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
        ui.label(
            "Value",
            "x3",
            LabelDesc {
                step: Some(step),
                font: FontRole::Display,
                ..LabelDesc::default()
            },
        );
        if own {
            ui.set_draw(og_ui::paint::ElementDraw {
                text: Some(og_ui::paint::TextEffects {
                    shadows: vec![og_ui::paint::TextShadow {
                        offset: [3.0, 0.7],
                        blur: 2.0,
                        color: og_ui::paint::ColorRef::token("shadow"),
                    }],
                    outline: Some(og_ui::paint::TextOutline {
                        width: 1.5,
                        color: og_ui::paint::ColorRef::token("ink"),
                        align: Default::default(),
                    }),
                    fill: None,
                }),
                ..Default::default()
            });
        }
        ui.end();
        ui.frame()
            .items
            .iter()
            .map(|it| it.cmd.clone())
            .collect::<Vec<_>>()
    };
    let texts: Vec<(f32, f32)> = run(&mut ui, false)
        .iter()
        .filter_map(|c| match c {
            DrawCmd::Text { x, baseline, .. } => Some((*x, *baseline)),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 2, "the drop and the glyph: {texts:?}");
    let (dx, dy) = (texts[0].0 - texts[1].0, texts[0].1 - texts[1].1);
    assert!(
        (dx.abs() == 2.0 || dx == 0.0) && (dy.abs() == 2.0 || dy == 0.0) && dx.hypot(dy) > 0.0,
        "one font pixel: {dx}, {dy}"
    );
    let fx = run(&mut ui, true)
        .into_iter()
        .find_map(|c| match c {
            DrawCmd::StyledText { effects, .. } => Some(*effects),
            _ => None,
        })
        .expect("styled text");
    assert_eq!(fx.outline.expect("an outline").width, 2.0);
    assert_eq!(fx.shadows[0].offset, og_core::glam::Vec2::new(4.0, 2.0));
    assert_eq!(fx.shadows[0].blur, 0.0);
}

/// A pixel face keeps its hard drop over the scene and drops none on a shape
/// its parent fills (a cost coin, a gem): the drop haloed a dark cost on a
/// light coin (pixel r3).
#[test]
fn a_pixel_face_on_a_filled_shape_takes_no_drop() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let file =
        parse_style(&std::fs::read_to_string(dir.join("cartoon").join("style.ogt")).unwrap())
            .unwrap();
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    let mut fonts = ui.fonts().clone();
    for f in [&mut fonts.body].into_iter().chain(fonts.display.as_mut()) {
        f.metrics = f.metrics.clone().with_pixel(12);
    }
    fonts.link();
    ui.set_fonts(fonts);
    ui.set_theme(file.theme(None, [0; 6]));
    let label = LabelDesc {
        font: FontRole::Display,
        ..LabelDesc::default()
    };
    ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
    ui.label("Scene", "3", label);
    ui.column("Coin", Element::default(), |ui| {
        ui.set_draw(og_ui::paint::ElementDraw {
            shape: og_ui::paint::ShapeKind::Ellipse,
            fills: vec![og_ui::paint::Paint::Solid(og_ui::paint::ColorRef::token(
                "highlight",
            ))],
            ..Default::default()
        });
        ui.label("Value", "3", label);
    });
    ui.end();
    let texts_in = |path: &str| {
        let e = ui.frame().element(path).expect(path);
        // The text commands whose centre lies in the element's box.
        ui.frame()
            .items
            .iter()
            .filter(|i| match &i.cmd {
                DrawCmd::Text { bounds: b, .. } => {
                    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
                    cx >= e.x && cx <= e.x + e.width && cy >= e.y && cy <= e.y + e.height
                }
                _ => false,
            })
            .count()
    };
    assert_eq!(
        texts_in("Scene"),
        2,
        "over the scene: the drop and the glyph"
    );
    assert_eq!(texts_in("Coin/Value"), 1, "on the coin: the glyph alone");
}

#[test]
fn a_packs_selected_chip_slot_and_tab_share_one_grammar_and_its_wells_read_3_to_1() {
    // Wrong: a pack's selected chip inverting (the default's grammar), a
    // selected slot keeping its thin rim and recess, a tab no hairline; or
    // an unchecked box's rim the 1.3:1 `line`, lost on its panel.
    use og_ui::dress::{token_effects, token_strokes, well_rim};
    use og_ui::paint::{ColorRef, Paint};
    let accent = |s: &[og_ui::paint::Stroke]| {
        s.len() == 1
            && matches!(&s[0].paint, Paint::Solid(ColorRef::Token { name, .. }) if name == "primary")
    };
    let ui = dressed("");
    let pack = ui.theme();
    let hair = pack.tokens.stroke(og_ui::tokens::StrokeWidth::Hairline);
    for (component, part, class) in [
        ("Chip", "base", ElementClass::Chip),
        ("Slot", "frame", ElementClass::Slot),
        ("Tabs", "tab", ElementClass::Tab),
    ] {
        let p = PartRef::new(component, part, class).flag("selected");
        let s = token_strokes(&p, pack);
        assert!(
            accent(&s) && s[0].width == hair,
            "{component}: an accent hairline"
        );
        assert!(
            token_effects(&p, pack).is_empty(),
            "{component}: flat, no recess or shadow"
        );
        let def = token_strokes(&p, &Theme::default());
        let thin = Theme::default()
            .tokens
            .stroke(og_ui::tokens::StrokeWidth::Thin);
        match component {
            "Slot" => assert!(
                accent(&def) && def[0].width == thin,
                "the default's slot rim"
            ),
            _ => assert!(def.is_empty(), "{component}: the default's, no rim"),
        }
    }
    // The chip's ground is one step up and its words the accent's ink.
    let mut ui = dressed("");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.chip(
        "On",
        "All",
        ChipDesc {
            selected: true,
            ..ChipDesc::default()
        },
    );
    ui.end();
    let e = ui.frame().element("On").unwrap().clone();
    let fills = fills_in(&ui, Rect::new(e.x, e.y, e.width, e.height));
    assert!(
        fills.iter().any(|f| near(*f, pack.palette.control_strong)),
        "on controlStrong: {fills:?}"
    );
    assert!(
        !fills.iter().any(|f| near(*f, pack.palette.ink)),
        "not inverted"
    );
    // Every pack's unchecked well is found at 3:1 on its well and panel.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    for e in std::fs::read_dir(dir).unwrap() {
        let f = e.unwrap().path().join("style.ogt");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let t = parse_style(&text).unwrap().theme(None, [0; 6]);
        let rim = well_rim(&t);
        for ground in [t.palette.panel, t.palette.panel_raised] {
            assert!(
                rim.contrast(ground.fade(1.0)) >= 3.0,
                "{}: {}",
                f.display(),
                rim.contrast(ground)
            );
        }
        let p = PartRef::new("Checkbox", "box", ElementClass::Field);
        assert!(
            matches!(&token_strokes(&p, &t)[..], [s] if matches!(s.paint, Paint::Solid(ColorRef::Literal(c)) if c == rim))
        );
    }
}

#[test]
fn slice_none_drops_a_less_specific_dressings_art() {
    // Wrong: a `[danger]` dressing inheriting `Button.base`'s slice, so its
    // own fill (scifi's red ABORT) never shows under the pack's art.
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/scifi/style.ogt"
    );
    let file = parse_style(&std::fs::read_to_string(path).unwrap()).unwrap();
    let set = og_ui::dress::DressSet::new(file.dressings.clone());
    let part = |f| PartRef::new("Button", "base", ElementClass::Button).flag(f);
    assert!(
        set.resolve(&part("secondary")).frame.slice.is_some(),
        "the pack's buttons are art"
    );
    let danger = set.resolve(&part("danger"));
    assert!(danger.frame.slice.is_none(), "slice none");
    assert!(!danger.draw.fills.is_empty(), "its fill is what draws");
}

#[test]
fn a_segmented_track_fills_whole_segments_along_its_ramp() {
    // Wrong: the fill cut mid-segment at the value, every filled segment in
    // one colour, or the segments not reaching the track's end.
    let mut ui = dressed(
        "dress \"Slider.track\" {\n  segments 5 4 ramp \"#ff0000\" \"#0000ff\"\n appearance {\n height 9\n }\n}\ndress \"Slider.fill\" {\n  stroke none\n  effect none\n appearance {\n height 6\n }\n}\n",
    );
    let mut v = 0.45;
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.slider(
            "Volume",
            &mut v,
            SliderDesc {
                width: Some(Size::Fixed(304.0)),
                ..SliderDesc::default()
            },
        );
        ui.end();
    }
    let e = ui.frame().element("Volume").unwrap().clone();
    let fills: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(r, _)| {
            r.x >= e.x - 0.5
                && r.right() <= e.x + e.width + 0.5
                && r.width < 64.0
                && r.height < 12.0
        })
        .collect();
    // Five segments of 57.6 px, 4 px apart, then two filled (0.45 × 5 is
    // 2.25: the nearest whole segment count is 2) in red and a quarter blue.
    let mut xs: Vec<f32> = fills.iter().map(|(r, _)| r.x.round()).collect();
    xs.sort_by(f32::total_cmp);
    xs.dedup();
    assert_eq!(xs.len(), 5, "segments at {xs:?}");
    assert!(fills.iter().any(|(r, _)| r.height == 9.0));
    assert!(fills.iter().any(|(r, _)| r.height == 6.0));
    let last = fills.iter().map(|(r, _)| r.right()).fold(0.0, f32::max);
    assert!(
        (last - (e.x + e.width)).abs() <= 1.0,
        "the last segment ends at the track's end"
    );
    let red = Srgba::rgb(255, 0, 0);
    let quarter = red.mix(Srgba::rgb(0, 0, 255), 0.25);
    assert!(
        fills.iter().any(|(_, c)| near(*c, red)),
        "the first filled segment is the ramp's start"
    );
    assert!(
        fills.iter().any(|(_, c)| near(*c, quarter)),
        "the second a quarter along it"
    );
    let mid = red.mix(Srgba::rgb(0, 0, 255), 0.5);
    assert!(
        !fills.iter().any(|(_, c)| near(*c, mid)),
        "no third segment filled"
    );
}

#[test]
fn every_packs_bag_cell_takes_its_slots_selected_and_tier_marks() {
    // Wrong: a pack's tier rims and selection dressed on `Slot.frame` only,
    // so the kit's bag (`Inventory.item[rarity=…][selected]`) draws none.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let mut packs = 0;
    for e in std::fs::read_dir(dir).unwrap() {
        let f = e.unwrap().path().join("style.ogt");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        packs += 1;
        let set = og_ui::dress::DressSet::new(parse_style(&text).unwrap().dressings);
        for tier in ["none", "common", "rare", "legendary"] {
            for sel in [false, true] {
                let of = |c, p| {
                    PartRef::new(c, p, ElementClass::Slot)
                        .attr("rarity", tier)
                        .flag_if(sel, "selected")
                };
                let slot = set.resolve(&of("Slot", "frame"));
                let bag = set.resolve(&of("Inventory", "item"));
                assert_eq!(
                    bag.draw,
                    slot.draw,
                    "{}: {tier} selected={sel}",
                    f.display()
                );
            }
        }
    }
    assert!(packs >= 12, "{packs} packs");
}

/// A focus ring sits outside its control: a row as wide as its scroll had
/// the ring cut to the scroll's box, so only its top and bottom showed
/// (cartoon's gallery rows). The ring's clip now reaches past the row's
/// sides, still inside what clips the scroll.
#[test]
fn a_focus_ring_in_a_scroll_keeps_its_sides() {
    let mut ui = ui_with_font(test_font_metrics());
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.open(
            "Page",
            Element {
                padding: Insets::all(40.0),
                ..Element::column()
            },
        );
        ui.open(
            "List",
            Element {
                width: Size::Fixed(200.0),
                height: Size::Fixed(300.0),
                scroll: true,
                ..Element::column()
            },
        );
        ui.open(
            "Row",
            Element {
                width: Size::grow(),
                height: Size::Fixed(40.0),
                interactive: true,
                focusable: true,
                background: Some(Srgba::rgb(40, 40, 40)),
                ..Element::row()
            },
        );
        ui.close();
        ui.close();
        ui.close();
        ui.end();
    };
    for _ in 0..3 {
        build(&mut ui, InputFrame::default());
    }
    build(
        &mut ui,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::default()
        },
    );
    build(&mut ui, InputFrame::default());
    let row = ui.frame().element("Row").unwrap().clone();
    let (rx, rw) = (row.x, row.width);
    let rings: Vec<(Rect, Rect)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, clip } if matches!(shape.mode, og_render::UiMode::Stroke(s) if s.align == og_render::UiAlign::Outside)
                => Some((Rect::new(shape.center.x - shape.half.x, shape.center.y - shape.half.y, shape.half.x * 2.0, shape.half.y * 2.0), *clip)),
            _ => None,
        })
        .collect();
    assert!(!rings.is_empty(), "the ring is drawn");
    for (r, clip) in rings {
        assert!(
            clip.x < rx - 1.0 && clip.x + clip.width > rx + rw + 1.0,
            "the ring {r:?} keeps its sides: clip {clip:?}, row x {rx} width {rw}"
        );
    }
}

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
    let dress = r##"dress "Poem.text" {
  ink "#c0392b"
  step lg
}
dress "Poem.placeholder" {
  ink "#7f8c8d"
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
        "{style}dress \"Poem.frame\" {{\n  ink accent\n}}\n"
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

/// A selected part's rim is its mark: it draws over the item's art (a
/// selected inventory slot's frame over the sword that fills it), where an
/// unselected part's rim stays under its content
/// (`a_rim_draws_before_content_and_raised_badges`).
#[test]
fn a_selected_parts_rim_draws_over_its_items_art() {
    let mut ui = ui_with_font(test_font_metrics());
    let blue = Srgba::rgb(0, 0, 255);
    ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
    ui.open(
        "Slot",
        Element {
            width: Size::Fixed(80.0),
            height: Size::Fixed(80.0),
            part: Some(PartRef::new("Slot", "frame", ElementClass::Slot).flag("selected")),
            ..Element::column()
        },
    );
    ui.set_draw(og_ui::paint::ElementDraw {
        strokes: vec![og_ui::paint::Stroke {
            width: 4.0,
            align: og_ui::paint::StrokeAlign::Inside,
            dash: [0.0; 2],
            dash_offset: 0.0,
            dash_cap: og_ui::paint::Cap::Butt,
            paint: og_ui::paint::Paint::Solid(Srgba::rgb(0, 255, 0).into()),
        }],
        ..Default::default()
    });
    ui.container(
        "Art",
        Element {
            width: Size::grow(),
            height: Size::grow(),
            background: Some(blue),
            ..Element::default()
        },
        |_| {},
    );
    ui.close();
    ui.end();
    let items = &ui.frame().items;
    let art = items
        .iter()
        .position(|i| i.cmd.filled().is_some_and(|(_, c)| near(c, blue)))
        .expect("the art");
    let rim = items
        .iter()
        .position(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_))))
        .expect("the rim");
    assert!(rim > art, "the selected rim ({rim}) over the art ({art})");
}

#[test]
fn scroll_bars_and_heading_flourishes_have_independent_dressings() {
    for authored in [false, true] {
        let mut ui = dressed(if authored {
            concat!(
                "dress \"Scroll.bar[axis=horizontal]\" {\n fill \"#ff00ff\"\n appearance {\n height 7\n }\n}\n",
                "dress \"Scroll.bar[axis=vertical]\" {\n fill \"#00ffff\"\n appearance {\n width 9\n }\n}\n",
                "dress \"Heading.flourish\" {\n appearance {\n width 12\n height 5\n color \"#ff00ff\"\n opacity 0.4\n }\n}\n",
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
fn loading_indicators_take_geometry_and_period_from_the_style() {
    let mut ui = dressed(concat!(
        "dress \"Skeleton.frame\" {\n appearance {\n width 100\n gap 9\n }\n}\n",
        "dress \"Skeleton.line\" {\n appearance {\n height 6\n }\n}\n",
        "dress \"Skeleton.line[last]\" {\n appearance {\n width 37\n }\n}\n",
        "dress \"Skeleton.sheen\" {\n fill \"#ff00ff\"\n appearance {\n motion tween 200 linear\n }\n}\n",
        "dress \"Spinner.dot\" {\n appearance {\n motion tween 200 linear\n }\n}\n",
        "dress \"Spinner.dot[age=0]\" {\n fill \"#00ffff\"\n}\n",
    ));
    for _ in 0..4 {
        ui.begin(InputFrame::default(), Viewport::new(200.0, 100.0));
        ui.column("Content", Element::column(), |ui| {
            ui.skeleton(
                "Loading",
                SkeletonDesc {
                    lines: 2,
                    ..Default::default()
                },
            );
            ui.spinner("Spinner", 40.0);
        });
        ui.end();
    }
    assert_eq!(ui.tick(), 4);
    let frame = ui.frame();
    let loading = frame.element("Loading").unwrap();
    assert_eq!(loading.width, 100.0);
    assert_eq!(loading.height, 21.0);
    let spinner = frame.element("Spinner").unwrap();
    let dot = frame
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .find(|(_, c)| *c == Srgba::rgb(0, 255, 255).linear())
        .unwrap()
        .0;
    assert_eq!(
        dot.center().0,
        spinner.x + 36.0,
        "200 ms cycle reaches the right-hand dot"
    );
    assert_eq!(dot.center().1, spinner.y + 20.0);
    let sheen: Vec<_> = frame
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(_, c)| *c == Srgba::rgb(255, 0, 255).linear())
        .map(|(r, _)| r)
        .collect();
    assert_eq!(sheen.len(), 2);
    assert!(
        sheen.iter().any(|r| (r.width - 30.0).abs() < 1.0),
        "{sheen:?}"
    );
    assert!(
        sheen.iter().any(|r| (r.width - 11.1).abs() < 1.0),
        "{sheen:?}"
    );
}

#[test]
fn a_dressed_focus_ring_keeps_navigation_and_changes_only_its_drawing() {
    let mut ui = dressed(
        "dress \"Button.focus[player=0]\" {\n stroke 7 \"#ff00ff\" outside\n appearance {\n width 140\n height 55\n x 9\n }\n}\n",
    );
    for keys in [vec![], vec![UiKey::Tab], vec![]] {
        ui.begin(
            InputFrame {
                keys,
                ..InputFrame::default()
            },
            Viewport::new(800.0, 600.0),
        );
        ui.open(
            "Inset",
            Element {
                padding: Insets::all(40.0),
                ..Element::default()
            },
        );
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                width: Some(Size::Fixed(100.0)),
                ..ButtonDesc::default()
            },
        );
        ui.close();
        ui.end();
    }
    let frame = ui.frame();
    let button = frame.element("Go").unwrap();
    assert_eq!(button.width, 100.0);
    assert!(button.states.iter().any(|s| s == "focused"));
    let ring = frame.items.iter().find_map(|item| match &item.cmd {
        DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(s) if s.width == 7.0) => Some(shape),
        _ => None,
    }).expect("authored focus stroke");
    assert_eq!(ring.half.x * 2.0, 140.0);
    assert_eq!(ring.half.y * 2.0, 55.0);
    assert_eq!(ring.center.x, button.x + button.width / 2.0 + 9.0);
}

/// A focus ring draws over everything around its control: the next
/// control in a tight column and the fade of the scroll it sits in both
/// came after it and covered its edge.
#[test]
fn a_focus_ring_draws_over_its_neighbours_and_its_scrolls_fade() {
    let mut ui = ui_with_font(test_font_metrics());
    let build = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, Viewport::new(800.0, 600.0));
        ui.scroll(
            "List",
            ScrollDesc {
                width: Size::Fixed(300.0),
                height: Size::Fixed(150.0),
                fade: 32.0,
                ..ScrollDesc::default()
            },
            |ui| {
                for k in 0..8 {
                    ui.button(
                        format!("B{k}"),
                        "Go",
                        ButtonDesc {
                            width: Some(Size::grow()),
                            ..ButtonDesc::default()
                        },
                    );
                }
            },
        );
        ui.end();
    };
    build(&mut ui, InputFrame::default());
    build(
        &mut ui,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::default()
        },
    );
    build(&mut ui, InputFrame::default());
    let f = ui.frame();
    let b0 = f.element("List/B0").expect("B0").clone();
    assert!(b0.states.iter().any(|s| s == "focused"), "B0 has focus");
    let b1 = f.element("List/B1").expect("B1").clone();
    let center = |c: &DrawCmd| match c {
        DrawCmd::Shape { shape, .. } => Some((shape.center.x, shape.center.y, shape.half.x)),
        DrawCmd::Quad { rect, .. } => Some((
            rect.x + rect.width / 2.0,
            rect.y + rect.height / 2.0,
            rect.width / 2.0,
        )),
        _ => None,
    };
    let (b0x, b0y) = b0.center();
    let ring = f
        .items
        .iter()
        .rposition(|i| {
            matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_render::UiMode::Stroke(_)))
                && center(&i.cmd).is_some_and(|(x, y, hx)| {
                    (x - b0x).abs() < 1.0 && (y - b0y).abs() < 1.0 && hx > b0.width / 2.0
                })
        })
        .expect("B0's focus ring");
    let (b1x, b1y) = b1.center();
    let next = f
        .items
        .iter()
        .rposition(|i| {
            i.cmd.filled().is_some()
                && center(&i.cmd)
                    .is_some_and(|(x, y, _)| (x - b1x).abs() < 1.0 && (y - b1y).abs() < 1.0)
        })
        .expect("B1's fill");
    let fade = f
        .items
        .iter()
        .rposition(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if matches!(shape.paint, og_render::UiPaint::Gradient(_))))
        .expect("the list's fade");
    assert!(
        ring > next,
        "the ring ({ring}) over the next button ({next})"
    );
    assert!(
        ring > fade,
        "the ring ({ring}) over the scroll's fade ({fade})"
    );
}

/// The timing table's toast hold (docs/ui.md §7.7): a style's `toastHold`
/// row overrides it, a style without one keeps the table's 3200 ms. At
/// 60 ticks a second a toast held 1000 ms is leaving by tick 70 in the
/// overriding style and still up there in the other.
///
/// Ways it could fail: the row parsed but the toast still taking the
/// fixed default, or the table's default moved for every style.
#[test]
fn a_styles_toast_hold_overrides_the_timing_table() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/storybook/style.ogt"
    );
    let source = std::fs::read_to_string(path).unwrap();
    for (extra, gone) in [("\n toastHold 1000", true), ("", false)] {
        let style =
            parse_style(&source.replacen("motion {", &format!("motion {{{extra}"), 1)).unwrap();
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(style.theme(None, [0; 6]));
        ui.toast("Saved", ToastDesc::default());
        for _ in 0..90 {
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.end();
        }
        assert_eq!(ui.toasts().is_empty(), gone, "toastHold {extra:?}");
    }
}
