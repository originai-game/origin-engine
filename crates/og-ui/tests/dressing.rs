//! A style dresses widgets' parts (docs/ui.md §3.2), plays its feedback
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

/// A tight field has no inset and its text's line for a height, so a
/// caller's own box holds the line exactly (the 大观园 page's verse box);
/// its style's text parts set the text's step and inks per component,
/// and an `ink` on a frame is refused. Wrong: the text took the
/// body step and ink, the placeholder inkFaint, inside a control's inset.
#[test]
fn a_tight_field_takes_its_components_text_parts() {
    let style = CORE_STYLE;
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
        .elements()
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
            .elements()
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
        assert_eq!(ui.element("List").unwrap().width, 100.0);
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
        let element = ui.element("Fire").unwrap();
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

/// `fill none` on a control's state clears the control's own fill: a quiet
/// button's disabled state, dressed `fill none`, draws no box (it drew the
/// control fill, louder than the clear quiet button at rest), while a
/// disabled secondary button the rule does not name keeps its fill.
#[test]
fn fill_none_clears_a_controls_own_fill() {
    let mut ui = dressed("Button::base.quiet:disabled {\n -og-fill: none\n}\n");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    for (id, variant) in [
        ("Quiet", ButtonVariant::Quiet),
        ("Plain", ButtonVariant::Secondary),
    ] {
        ui.button(
            id,
            "Equip",
            ButtonDesc {
                variant,
                disabled: true,
                ..ButtonDesc::default()
            },
        );
    }
    ui.end();
    let fills = |ui: &Ui, id: &str| {
        let e = ui.element(id).unwrap();
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    assert_eq!(
        fills(&ui, "Quiet"),
        Vec::<[f32; 4]>::new(),
        "the cleared quiet button"
    );
    assert!(
        !fills(&ui, "Plain").is_empty(),
        "a disabled secondary keeps its fill"
    );
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

/// A document element's `part` attribute is the part its style dresses
/// (docs/ui.md §3.2), with its classes and ARIA states as qualifiers and a
/// control's own states kept: a kit's `<button part="Loadout.item">` is
/// `Loadout::item:hover` under the pointer. A core frame with a qualifier,
/// literal or bound (`Slot.frame[rarity=epic]`), needs no `parts` entry and
/// takes the style's rarity dressing. Wrong: the attribute reached no draw
/// (a bare box, a `Button::base` button), a class or `aria-selected` never
/// qualified it, the control lost its hover and variant, or a core frame
/// was refused as a part the document does not list.
#[test]
fn a_document_part_takes_its_dressing_classes_and_states() {
    use og_ui::reactive::SourceId;
    let mut ui = dressed(concat!(
        "Kit::box {\n  -og-fill: #00ff00\n}\n",
        "Kit::box.hot {\n  -og-fill: #ffff00\n}\n",
        "Kit::item {\n  -og-fill: #00ffff\n}\n",
        "Kit::item:hover {\n  -og-fill: #ffffff\n}\n",
        "Kit::item:selected {\n  -og-fill: #ff8800\n}\n",
        "Slot::frame[rarity=epic] {\n  -og-fill: #ff00ff\n}\n",
        "Slot::frame[rarity=rare] {\n  -og-fill: #0000ff\n}\n",
    ));
    let doc = r#"<meta name="component" content="kit">
<meta name="parts" content="Kit.box Kit.item">
<div id="Root">
  <div id="Box" part="Kit.box">a</div>
  <div id="Hot" part="Kit.box" class:hot={true}>b</div>
  <button id="Item" part="Kit.item">c</button>
  <button id="Picked" part="Kit.item" aria-selected={true}>d</button>
  <div id="Epic" part="Slot.frame[rarity=epic]">e</div>
  <div id="Rare" part={"Slot.frame[rarity=" + "rare" + "]"}>f</div>
</div>
<style>
#Root { width: stretch; padding: 8px; gap: 8px; }
#Box, #Hot, #Epic, #Rare { width: 100px; height: 30px; }
</style>"#;
    ui.define_document(UiDocument::parse("ui/kit.svelte", doc).unwrap());
    ui.mount_document(
        SourceId::new("test", "kit"),
        "kit",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui, pointer: Option<(f32, f32)>| {
        let input = InputFrame {
            pointer: pointer.map_or(Default::default(), |(x, y)| og_core::glam::Vec2::new(x, y)),
            pointer_inside: pointer.is_some(),
            ..InputFrame::default()
        };
        ui.begin(input, Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    for _ in 0..3 {
        step(&mut ui, None);
    }
    let part = |ui: &Ui, id: &str| -> String {
        let k = ui
            .elements()
            .iter()
            .position(|e| e.path.ends_with(&format!("/{id}")))
            .unwrap_or_else(|| panic!("{id}"));
        ui.parts()[k].map(|p| p.to_string()).unwrap_or_default()
    };
    let fill = |ui: &Ui, id: &str| -> Vec<[f32; 4]> {
        let e = ui
            .elements()
            .iter()
            .find(|e| e.path.ends_with(&format!("/{id}")))
            .unwrap()
            .clone();
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    let has = |fills: Vec<[f32; 4]>, hex: &str| {
        let c = og_ui::style_file::parse_color(hex).unwrap();
        fills.iter().any(|f| near(*f, c))
    };
    assert_eq!(part(&ui, "Box"), "Kit.box");
    assert!(has(fill(&ui, "Box"), "#00ff00"), "{:?}", fill(&ui, "Box"));
    assert!(part(&ui, "Hot").contains("[hot]"), "{}", part(&ui, "Hot"));
    assert!(has(fill(&ui, "Hot"), "#ffff00"));
    let item = part(&ui, "Item");
    assert!(
        item.starts_with("Kit.item[secondary]"),
        "the control's variant stays: {item}"
    );
    assert!(has(fill(&ui, "Item"), "#00ffff"));
    assert!(part(&ui, "Picked").contains("[selected]"));
    assert!(has(fill(&ui, "Picked"), "#ff8800"));
    assert_eq!(part(&ui, "Epic"), "Slot.frame[rarity=epic]");
    assert!(has(fill(&ui, "Epic"), "#ff00ff"), "{:?}", fill(&ui, "Epic"));
    assert_eq!(part(&ui, "Rare"), "Slot.frame[rarity=rare]");
    assert!(has(fill(&ui, "Rare"), "#0000ff"), "{:?}", fill(&ui, "Rare"));
    let e = ui
        .elements()
        .iter()
        .find(|e| e.path.ends_with("/Item"))
        .unwrap()
        .clone();
    for _ in 0..30 {
        step(&mut ui, Some((e.x + e.width / 2.0, e.y + e.height / 2.0)));
    }
    assert!(
        part(&ui, "Item").contains("[hover]"),
        "{}",
        part(&ui, "Item")
    );
    assert!(has(fill(&ui, "Item"), "#ffffff"), "{:?}", fill(&ui, "Item"));
}

/// A kit's text element that names its part takes the part's `-og-ink` and
/// `-og-step` (docs/ui.md §3.2), qualified by its classes, as a field's
/// `text` part does: a pack inks a killfeed's names by team. Wrong: the
/// style refused `ink` on a kit part, the ink reached no text or every
/// part's text, a qualified dressing lost to the bare one, or text that
/// names no part took it.
#[test]
fn a_kit_text_part_takes_its_ink_and_step() {
    use og_ui::reactive::SourceId;
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    let file = parse_style(&format!(
        "{CORE_STYLE}{}",
        concat!(
            "Kit::name {\n  -og-ink: #c0392b\n  -og-step: lg\n}\n",
            "Kit::name.ally {\n  -og-ink: #2e86de\n}\n",
        )
    ))
    .unwrap_or_else(|e| panic!("{e}"));
    ui.set_theme(file.theme(None, [0; 6]));
    ui.set_dressings(file.dressings.clone());
    let doc = r#"<meta name="component" content="kit">
<meta name="parts" content="Kit.name">
<div id="Root">
  <span id="Foe" part="Kit.name">Foe</span>
  <span id="Ally" part="Kit.name" class:ally={true}>Ally</span>
  <span id="Free">Free</span>
</div>"#;
    ui.define_document(UiDocument::parse("ui/kit.svelte", doc).unwrap());
    ui.mount_document(
        SourceId::new("test", "kit"),
        "kit",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    }
    let text = |want: &str| -> (f32, [f32; 4]) {
        ui.frame()
            .items
            .iter()
            .find_map(|it| match &it.cmd {
                DrawCmd::Text {
                    text, size, color, ..
                } if text == want => Some((*size, *color)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text {want}"))
    };
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    let (lg, body) = (ui.theme().type_ramp.lg.size, ui.theme().type_ramp.body.size);
    let foe = text("Foe");
    assert!(near(foe.1, colour("#c0392b")), "{:?}", foe.1);
    assert_eq!(foe.0, lg, "the part's step");
    let ally = text("Ally");
    assert!(
        near(ally.1, colour("#2e86de")),
        "the qualified ink wins: {:?}",
        ally.1
    );
    assert_eq!(ally.0, lg, "the bare dressing's step still applies");
    let free = text("Free");
    assert!(near(free.1, ui.theme().palette.ink), "{:?}", free.1);
    assert_eq!(free.0, body);
}

#[test]
fn css_parts_share_the_css_cascade_and_markup_hosts() {
    use og_ui::dress::PartRef;
    use og_ui::paint::{ColorRef, Effect, Paint, ShapeKind};
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(include_str!("fixtures/parts.css")).unwrap();
    let part = PartRef::of("Button", "base", None);
    let color = |hex: &str| Paint::Solid(ColorRef::Literal(style_file::parse_color(hex).unwrap()));
    let draw = ui.resolve_part(&part, [120.0, 40.0]).unwrap();
    assert_eq!(draw.fills, vec![color("#123456")]);
    assert!(matches!(draw.shape, ShapeKind::Rect { radii: Some(r), .. } if r == [7.0; 4]));
    assert!(matches!(
        draw.effects.as_slice(),
        [Effect::DropShadow {
            offset: [1.0, 3.0],
            blur: 5.0,
            spread: 2.0,
            ..
        }]
    ));
    assert_eq!(
        draw.borders.as_ref().unwrap().borders[0].unwrap().width,
        2.0
    );
    let hover = part.flag("primary").flag("hover");
    assert_eq!(
        ui.resolve_part(&hover, [120.0, 40.0]).unwrap().fills,
        vec![color("#ffff00")]
    );
    let epic = hover.attr("data-rarity", "epic").flag("pressed");
    let draw = ui.resolve_part(&epic, [120.0, 40.0]).unwrap();
    assert_eq!(draw.fills, vec![color("#0000ff")]);
    assert_eq!(draw.opacity, 0.4);
    ui.set_theme(Theme::new(Scheme::Light));
    assert_eq!(
        ui.resolve_part(&part, [120.0, 40.0]).unwrap().fills,
        vec![color("#abcdef")]
    );
    for (component, name, on, expected) in [
        ("Slider", "track", false, "#112233"),
        ("Checkbox", "base", true, "#334455"),
        ("Toggle", "base", true, "#556677"),
        ("Dropdown", "base", false, "#778899"),
        ("TextEdit", "frame", false, "#99aabb"),
    ] {
        let p = PartRef::of(component, name, None).flag_if(on, "on");
        assert_eq!(
            ui.resolve_part(&p, [80.0, 30.0]).unwrap().fills,
            vec![color(expected)],
            "{component}"
        );
    }
    let slot = ui
        .resolve_part(&PartRef::of("Slot", "frame", None), [120.0, 40.0])
        .unwrap();
    let layers = slot.boxes.unwrap();
    assert_eq!(layers.backgrounds.len(), 1);
    assert!(layers.border.is_some());
    assert!(matches!(
        slot.effects.as_slice(),
        [Effect::BackdropFilter(_)]
    ));
    let empty = ui
        .resolve_part(&PartRef::of("Button", "unknown", None), [120.0, 40.0])
        .unwrap();
    assert!(empty.fills.is_empty() && empty.effects.is_empty() && empty.borders.is_none());
    for invalid in [
        "Button::base",
        "Button.base",
        "Button::part(base)",
        "button::part(base).primary",
    ] {
        assert!(
            ui.set_part_css(&format!("{invalid} {{ background: red; }}"))
                .is_err(),
            "{invalid}"
        );
    }
    let doc = UiDocument::parse(
        "parts.svelte",
        r#"<meta name="component" content="parts"><button>OK</button><style>button::part(base):hover { background: red; }</style>"#,
    )
    .unwrap();
    assert_eq!(doc.part_rules.len(), 1);
}

#[test]
#[ignore = "requires GPU"]
fn css_parts_render_their_background_border_and_shadow() {
    use og_render::*;
    use og_ui::dress::PartRef;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.set_part_css(include_str!("fixtures/parts.css")).unwrap();
    let (width, height) = (464, 96);
    ui.begin(
        InputFrame::default(),
        Viewport::new(width as f32, height as f32),
    );
    ui.row(
        "States",
        Element {
            padding: Insets::all(24.0),
            gap: 24.0,
            ..Element::row()
        },
        |ui| {
            let base = PartRef::of("Button", "base", None);
            for (name, part) in [
                ("Rest", base),
                ("Hover", base.flag("hover")),
                (
                    "Epic",
                    base.flag("primary")
                        .flag("hover")
                        .attr("data-rarity", "epic"),
                ),
            ] {
                let draw = ui.resolve_part(&part, [120.0, 40.0]).unwrap();
                ui.column(
                    name,
                    Element {
                        width: Size::px(120.0),
                        height: Size::px(40.0),
                        ..Element::column()
                    },
                    |ui| ui.set_draw(draw),
                );
            }
        },
    );
    ui.end();
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.overlay = Some(overlay);
    scene.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .unwrap();
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-parts.png"),
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for (x, expected) in [
        (24, [254, 220, 186]),
        (84, [0x12, 0x34, 0x56]),
        (228, [255, 255, 0]),
        (372, [0, 0, 255]),
    ] {
        let at = (44 * width as usize + x) * 4;
        for c in 0..3 {
            assert!(
                (i16::from(pixels[at + c]) - expected[c]).abs() <= 3,
                "pixel at {x},44: {:?}",
                &pixels[at..at + 4]
            );
        }
    }
    let shadow = (67 * width as usize + 84) * 4;
    let ground = (85 * width as usize + 84) * 4;
    assert!(
        pixels[shadow] < pixels[ground],
        "the outer shadow darkens the ground"
    );
}
