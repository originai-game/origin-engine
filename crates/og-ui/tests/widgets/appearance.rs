use super::*;

#[test]
fn anchored_groups_keep_out_of_the_host_control() {
    let mut rig = Rig::new(());
    let b = &mut |ui: &mut Ui, _: &mut ()| {
        for (id, anchor) in [
            ("Place", Anchor::TopLeft),
            ("Clock", Anchor::Top),
            ("Times", Anchor::TopRight),
        ] {
            ui.anchor(
                id,
                AnchorDesc {
                    anchor,
                    ..AnchorDesc::default()
                },
                |ui| {
                    ui.label("Text", "1/8  lap 2/3", LabelDesc::default());
                },
            );
        }
    };
    rig.idle(1, b);
    let at = |rig: &Rig<()>, id: &str| rig.ui.frame().element(id).expect(id).clone();
    let (place, clock, times) = (at(&rig, "Place"), at(&rig, "Clock"), at(&rig, "Times"));
    assert!(place.x < 40.0, "no control: the corner itself ({place:?})");
    // The web shell's Back: 12, 12 to 52, 52 CSS px.
    rig.vp.covered = Rect::new(12.0, 12.0, 40.0, 40.0);
    rig.idle(1, b);
    let moved = at(&rig, "Place");
    assert!(
        moved.x >= 52.0 && (moved.y - place.y).abs() < 0.5,
        "the top-left group moves right of the control, at the same height: {moved:?}"
    );
    assert_eq!(at(&rig, "Clock"), clock, "a group clear of it stays");
    assert_eq!(at(&rig, "Times"), times, "a group clear of it stays");
}

/// On a phone a top-left group moved right of the host's control would
/// land on the top-right group beside it (a shooter HUD's vitals on its
/// kills and pause): it steps down past the control instead, at the same
/// x, and a group with room to its right still moves right (Brasswick #15).
#[test]
fn a_corner_group_steps_down_past_the_control_when_moving_along_lands_on_another() {
    let mut rig = Rig::new(());
    rig.vp = Viewport {
        width: 390.0,
        height: 844.0,
        dpr: 3.0,
        safe_area: Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        },
        coarse_pointer: true,
        ..Viewport::default()
    };
    let wide = std::cell::Cell::new(240.0);
    let b = &mut |ui: &mut Ui, _: &mut ()| {
        ui.anchor(
            "Vitals",
            AnchorDesc {
                anchor: Anchor::TopLeft,
                width: Size::Fixed(wide.get()),
                height: Size::Fixed(20.0),
                ..AnchorDesc::default()
            },
            |_| {},
        );
        ui.anchor(
            "Kills",
            AnchorDesc {
                anchor: Anchor::TopRight,
                width: Size::Fixed(110.0),
                height: Size::Fixed(44.0),
                ..AnchorDesc::default()
            },
            |_| {},
        );
    };
    rig.idle(1, b);
    let at = |rig: &Rig<()>, id: &str| rig.ui.frame().element(id).expect(id).clone();
    let (free, kills) = (at(&rig, "Vitals"), at(&rig, "Kills"));
    assert!(free.x + free.width <= kills.x, "apart without the control");
    // The web shell's Back, top left inside the safe area.
    rig.vp.covered = Rect::new(12.0, 59.0, 40.0, 40.0);
    rig.idle(1, b);
    let moved = at(&rig, "Vitals");
    assert_eq!(at(&rig, "Kills"), kills, "the group clear of it stays");
    assert!(
        (moved.x - free.x).abs() < 0.5 && moved.y >= 99.0,
        "down past the control at the same x: {moved:?}"
    );
    assert!(
        moved.x + moved.width <= kills.x || moved.y >= kills.y + kills.height,
        "and off the other group: {moved:?} {kills:?}"
    );
    // A narrow group has room along the edge: it moves right, as on a desktop.
    wide.set(120.0);
    rig.idle(1, b);
    let narrow = at(&rig, "Vitals");
    assert!(
        narrow.x >= 52.0 && (narrow.y - free.y).abs() < 0.5,
        "a group with room moves right at the same height: {narrow:?}"
    );
}

#[test]
fn anchors_sit_in_the_safe_area_and_touch_controls_keep_44px() {
    let mut rig = Rig::new(JoystickResponse::default());
    rig.vp = Viewport {
        width: 390.0,
        height: 844.0,
        dpr: 3.0,
        safe_area: Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        },
        coarse_pointer: true,
        ..Viewport::default()
    };
    let b = &mut |ui: &mut Ui, s: &mut JoystickResponse| {
        ui.anchor(
            "Score",
            AnchorDesc {
                anchor: Anchor::TopRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.label("Points", "1200", LabelDesc::default());
            },
        );
        ui.anchor(
            "Pad",
            AnchorDesc {
                anchor: Anchor::BottomLeft,
                ..AnchorDesc::default()
            },
            |ui| {
                *s = ui.joystick(
                    "Stick",
                    JoystickDesc {
                        size: 60.0,
                        ..JoystickDesc::default()
                    },
                );
            },
        );
        ui.anchor(
            "Buttons",
            AnchorDesc {
                anchor: Anchor::BottomRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.action_button(
                    "Jump",
                    "Jump",
                    &ActionButtonDesc {
                        size: 20.0,
                        ..ActionButtonDesc::default()
                    },
                );
                ui.dpad(
                    "Dpad",
                    DpadDesc {
                        size: 60.0,
                        ..DpadDesc::default()
                    },
                );
            },
        );
    };
    rig.idle(1, b);
    assert_eq!(rig.ui.breakpoint(), Breakpoint::PhonePortrait);
    let gutter = rig.ui.sizes().gutter;
    let score = rig.ui.frame().element("Score").expect("score").clone();
    assert!(
        (score.y - (47.0 + gutter)).abs() < 0.5,
        "under the notch: {score:?}"
    );
    assert!((score.x + score.width - (390.0 - gutter)).abs() < 0.5);
    let pad = rig.ui.frame().element("Pad").expect("pad").clone();
    assert!(
        (pad.y + pad.height - (844.0 - 34.0 - gutter)).abs() < 0.5,
        "above the home indicator: {pad:?}"
    );
    for id in ["Stick", "Jump", "Dpad/Up", "Dpad/Left"] {
        let e = rig.ui.frame().element(id).expect(id);
        assert!(
            e.width >= 44.0 && e.height >= 44.0,
            "{id} is {}×{}",
            e.width,
            e.height
        );
    }
    // Drag the stick to the right edge.
    let c = rig.center("Stick");
    let r = rig.ui.frame().element("Stick").unwrap().width / 2.0;
    rig.step(press(c), b);
    rig.step(
        InputFrame {
            pointer: c + Vec2::new(r * 2.0, 0.0),
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        },
        b,
    );
    assert!(
        rig.state.active && (rig.state.x - 1.0).abs() < 1e-4 && rig.state.y.abs() < 1e-4,
        "{:?}",
        rig.state
    );
    rig.step(release(c), b);
    assert!(!rig.state.active && rig.state.x == 0.0);
}

/// A game's own primary colour: hover and press move it towards opposite
/// ends of the scale (a dark one lightens hovered, a light one darkens),
/// and the ink on it is whichever of the palette's inks reads better.
///
/// Ways it could fail: the hover and press tones swapped for a light
/// primary, the ink picked by luminance alone, the soft wash losing the
/// palette's alpha.
#[test]
fn a_game_primary_derives_its_hover_press_and_ink() {
    let dark = Theme::dark();
    let navy = dark.with_primary(Srgba::rgb(0x20, 0x30, 0x80));
    let p = navy.palette;
    assert_eq!(p.primary, Srgba::rgb(0x20, 0x30, 0x80));
    assert!(
        p.primary_hover.luminance() > p.primary.luminance(),
        "a dark primary lightens hovered"
    );
    assert!(
        p.primary_active.luminance() < p.primary.luminance(),
        "and darkens pressed"
    );
    assert_eq!(p.on_primary, dark.palette.ink, "light ink on navy");
    assert_eq!(p.primary_soft.alpha, dark.palette.primary_soft.alpha);
    assert_eq!(p.primary_soft.rgb, p.primary.rgb);
    let lemon = dark.with_primary(Srgba::rgb(0xf0, 0xe0, 0x60)).palette;
    assert!(
        lemon.primary_hover.luminance() < lemon.primary.luminance(),
        "a light primary darkens hovered"
    );
    assert!(lemon.primary_active.luminance() > lemon.primary.luminance());
    assert_eq!(
        lemon.on_primary, dark.palette.ink_inverse,
        "dark ink on lemon"
    );
    // The accent ink follows the game's colour (not the style's) and reads
    // on its own wash over a panel; a hue in the middle of the scale moves
    // towards the scheme's ink until it does.
    for c in [
        Srgba::rgb(0xf0, 0xe0, 0x60),
        Srgba::rgb(0xbe, 0x1e, 0x32),
        Srgba::rgb(0x20, 0x30, 0x80),
    ] {
        for base in [Theme::dark(), Theme::light()] {
            let q = base.with_primary(c).palette;
            assert_ne!(
                q.highlight, base.palette.highlight,
                "{c:?}: highlight follows the primary"
            );
            let wash = q.primary_soft.over_linear(q.panel);
            assert!(
                q.highlight.contrast(wash) >= 5.0,
                "{c:?}: the accent reads {:.2}:1 on its wash",
                q.highlight.contrast(wash)
            );
        }
    }
    let hc = Theme::high_contrast(Scheme::Dark);
    assert_eq!(
        hc.palette.ink_muted, dark.palette.ink,
        "muted ink takes ink"
    );
    assert!(hc.outline);
}

/// A type step's tracking spaces Latin glyphs (each advance plus tracking
/// em: the label is wider by that per character and its draw command
/// carries it to og-render), and leaves CJK text as the font sets it.
/// Ways it could fail: the ramp's tracking is ignored (equal widths),
/// measured but not drawn (the text overruns its box), or applied to CJK.
#[test]
fn a_steps_tracking_spaces_latin_glyphs_and_not_cjk() {
    let width = |tracking: f32, text: &str| {
        let mut scene = og_render::RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let mut t = *ui.theme();
        t.type_ramp.xl.tracking_em = tracking;
        ui.set_theme(t);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "L",
            text,
            LabelDesc {
                step: Some(t.type_ramp.xl),
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        ui.end();
        let drawn = ui
            .frame()
            .items
            .iter()
            .find_map(|i| match &i.cmd {
                DrawCmd::Text { tracking, .. } => Some(*tracking),
                _ => None,
            })
            .unwrap_or(f32::NAN);
        (ui.frame().element("L").expect("label").width, drawn)
    };
    let size = Theme::default().type_ramp.xl.size;
    let (plain, _) = width(0.0, "Origin");
    let (wide, drawn) = width(0.1, "Origin");
    assert!(
        (wide - plain - 6.0 * 0.1 * size).abs() < 0.75,
        "six glyphs, each 0.1 em further: {plain} then {wide}"
    );
    assert_eq!(drawn, 0.1, "the draw command carries the tracking");
    let (cjk_plain, _) = width(0.0, "原点引擎");
    let (cjk_wide, cjk_drawn) = width(0.1, "原点引擎");
    assert_eq!(cjk_plain, cjk_wide, "CJK keeps the font's spacing");
    assert_eq!(cjk_drawn, 0.0);
}

/// The player's text scale (`Theme::text_scale`) draws every text at
/// that multiple and lets the layout grow around it: a label's box and a
/// button's height grow, and the button's text stays inside it. Ways it
/// could fail: the ramp scales but the role steps do not (the button's
/// label stays small), the text grows inside a fixed control height (it is
/// clipped), or reapplying the theme scales the ramp again.
#[test]
fn text_scale_grows_every_text_and_the_layout_around_it() {
    let run = |scale: f32| {
        let mut scene = og_render::RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let mut t = *ui.theme();
        t.text_scale = scale;
        ui.set_theme(t);
        // A theme read back and set again is scaled once.
        let again = *ui.theme();
        ui.set_theme(again);
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label("L", "Origin", LabelDesc::default());
        ui.button("B", "Play", ButtonDesc::default());
        ui.end();
        let f = ui.frame();
        let texts: Vec<(f32, Rect, Rect)> = f
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Text {
                    size, bounds, clip, ..
                } => Some((*size, *bounds, *clip)),
                _ => None,
            })
            .collect();
        let (l, b) = (
            f.element("L").expect("label").clone(),
            f.element("B").expect("button").clone(),
        );
        (texts, l, b)
    };
    let (t1, l1, b1) = run(1.0);
    let (t2, l2, b2) = run(1.5);
    assert_eq!(t1.len(), 2);
    assert_eq!(t2.len(), 2);
    for (a, b) in t1.iter().zip(&t2) {
        assert!(
            (b.0 - 1.5 * a.0).abs() < 0.01,
            "every text half again its size: {} then {}",
            a.0,
            b.0
        );
    }
    assert!(
        l2.height >= 1.5 * l1.height - 0.5,
        "the label's line box grows as much: {} then {}",
        l1.height,
        l2.height
    );
    assert!(
        b2.height > b1.height,
        "the button grows: {} then {}",
        b1.height,
        b2.height
    );
    let (_, bounds, clip) = t2[1];
    assert!(
        bounds.y >= b2.y - 0.5 && bounds.y + bounds.height <= b2.y + b2.height + 0.5,
        "the button's text inside it: {bounds:?} in {b2:?}"
    );
    assert!(
        clip.height >= bounds.height - 0.5,
        "and not clipped: {bounds:?} under {clip:?}"
    );
    // Out of range is clamped to the bar (200 % deferred): 2 draws as 1.5.
    let (t3, _, _) = run(2.0);
    assert!((t3[0].0 - t2[0].0).abs() < 0.01);
}

/// An icon set (an MTSDF font, `IconAtlas::font`): `icon` draws its glyph
/// (U+E000 + the icon's index) fitted to the icon's square in its ink,
/// rich text's `[icon=]` draws the same glyph inline, and the probe reads
/// the icon's name.
#[test]
fn an_icon_set_draws_its_glyphs_in_their_squares() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut scene = og_render::RenderScene::new();
    let set = add_test_font(&mut scene);
    let font = set.handle.expect("the test font draws");
    ui.set_icons(IconAtlas {
        font: Some(set),
        names: vec!["heart".into(), "star".into()],
        ..IconAtlas::default()
    });
    assert!(ui.has_icon("star") && !ui.has_icon("moon"));
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
        ui.icon(
            "Star",
            "star",
            IconDesc {
                size: 32.0,
                tint: Some(Srgba::rgb(255, 0, 0)),
                secondary: Some("heart".into()),
                secondary_tint: Some(Srgba::rgb(0, 255, 0)),
                ..IconDesc::default()
            },
        );
        ui.rich_text("Line", "Gold [icon=heart] left", RichTextDesc::default());
        ui.end();
    }
    let star = ui.frame().element("Star").expect("the icon").clone();
    assert_eq!(star.text, "star", "the probe reads its name");
    let glyphs: Vec<(String, f32, Rect, [f32; 4])> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Text {
                text,
                size,
                bounds,
                color,
                font: f,
                ..
            } if *f == font && text.chars().all(|c| ('\u{E000}'..='\u{E001}').contains(&c)) => {
                Some((text.clone(), *size, *bounds, *color))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        glyphs.len(),
        3,
        "two icon layers and one inline glyph: {glyphs:?}"
    );
    let (text, size, bounds, color) = &glyphs[0];
    assert_eq!(text, "\u{E001}", "star is icon 1");
    assert_eq!(*size, 32.0);
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (star.x, star.y, 32.0, 32.0),
        "fitted to the icon's square"
    );
    assert_eq!(color[0], 1.0, "in its ink");
    assert_eq!(glyphs[1].0, "\u{E000}");
    assert_eq!(glyphs[1].2, *bounds, "the second layer shares the square");
    assert_eq!(glyphs[1].3, Srgba::rgb(0, 255, 0).linear());
    assert_eq!(
        glyphs[2].0, "\u{E000}",
        "rich text's [icon=heart] is icon 0"
    );
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// A style pack paints a few icons; the game's whole set
/// (`UiDesc.fallbackIcons`) draws the names the pack lacks, in `icon` and
/// in rich text, and never replaces a name the pack painted.
#[test]
fn a_fallback_set_draws_the_icons_a_pack_lacks() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut scene = og_render::RenderScene::new();
    let set = add_test_font(&mut scene);
    let font = set.handle.expect("the test font draws");
    let painted = scene.add_texture(og_render::TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    });
    ui.set_icons(IconAtlas {
        texture: Some(painted),
        columns: 2,
        rows: 1,
        names: vec!["coin".into(), "star".into()],
        fallback: Some(Box::new(IconAtlas {
            font: Some(set),
            names: vec!["star".into(), "settings".into(), "bell".into()],
            ..IconAtlas::default()
        })),
        ..IconAtlas::default()
    });
    assert!(ui.has_icon("bell") && ui.has_icon("coin") && !ui.has_icon("moon"));
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
        ui.icon("Star", "star", IconDesc::default());
        ui.icon("Bell", "bell", IconDesc::default());
        ui.rich_text("Line", "Ring [icon=settings]", RichTextDesc::default());
        ui.end();
    }
    let items = &ui.frame().items;
    let images: Vec<[f32; 4]> = items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Image {
                texture: Some(t),
                uv,
                ..
            } if *t == painted => Some(*uv),
            _ => None,
        })
        .collect();
    assert_eq!(
        images,
        vec![[0.5, 0.0, 1.0, 1.0]],
        "star keeps the pack's cell"
    );
    let glyphs: Vec<String> = items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Text { text, font: f, .. }
                if *f == font && text.chars().all(|c| c >= '\u{E000}') =>
            {
                Some(text.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        glyphs,
        vec!["\u{E002}".to_string(), "\u{E001}".to_string()],
        "bell and the inline settings from the fallback set"
    );
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// Kestrel-stair FINDINGS #15: a grid lays out exactly `columns` cells a
/// row, whatever its width: a fit grid that many cells of the asked width,
/// a growing one with the cells sharing its width (wider than asked, or
/// narrower down to the touch target), so a level select's two columns
/// are two and an inventory's six are six.
#[test]
fn a_grid_lays_out_exactly_its_columns_whatever_its_width() {
    struct G {
        columns: u32,
        width: Size,
        cell: f32,
    }
    let b = &mut |ui: &mut Ui, g: &mut G| {
        ui.open(
            "Box",
            Element {
                width: Size::Fixed(500.0),
                ..Element::default()
            },
        );
        ui.grid(
            "Grid",
            GridDesc {
                count: 13,
                cell_width: g.cell,
                columns: g.columns,
                width: g.width,
                height: Some(Size::Fixed(560.0)),
                ..GridDesc::default()
            },
            None,
            |_, _| {},
        );
        ui.close();
    };
    let rows = |rig: &Rig<G>| -> Vec<Vec<(f32, f32)>> {
        let mut rows: Vec<Vec<(f32, f32)>> = Vec::new();
        for i in 0..13 {
            let Some(e) = rig.ui.frame().element(&format!("Box/Grid/Cell#{i}")) else {
                continue;
            };
            match rows.last_mut() {
                Some(r) if (r[0].1 - e.y).abs() < 0.5 => r.push((e.width, e.y)),
                _ => rows.push(vec![(e.width, e.y)]),
            }
        }
        rows
    };
    // Two columns in 500 px: two cards of (500 + 8) / 2 - 8 = 246 px, not four of 104.
    let mut rig = Rig::new(G {
        columns: 2,
        width: Size::grow(),
        cell: 104.0,
    });
    rig.idle(3, b);
    let r = rows(&rig);
    assert!(
        r.iter().take(6).all(|row| row.len() == 2),
        "rows of two: {r:?}"
    );
    assert!(
        (r[0][0].0 - 246.0).abs() < 0.1,
        "the cards share the width: {r:?}"
    );
    // Six columns of 104 asked in 500 px: six narrower cells a row, not four.
    let mut rig = Rig::new(G {
        columns: 6,
        width: Size::grow(),
        cell: 104.0,
    });
    rig.idle(3, b);
    let r = rows(&rig);
    assert!(
        r.iter().take(2).all(|row| row.len() == 6),
        "rows of six: {r:?}"
    );
    assert!(r[0][0].0 >= 44.0 && r[0][0].0 < 104.0, "{r:?}");
    // A fit grid: the columns at the asked width.
    let mut rig = Rig::new(G {
        columns: 3,
        width: Size::fit(),
        cell: 72.0,
    });
    rig.idle(3, b);
    let r = rows(&rig);
    assert!(
        r.iter().take(4).all(|row| row.len() == 3) && r[0][0].0 == 72.0,
        "{r:?}"
    );
    // No columns: as many of the asked width as fit (500 / 112 → 4).
    let mut rig = Rig::new(G {
        columns: 0,
        width: Size::grow(),
        cell: 104.0,
    });
    rig.idle(3, b);
    let r = rows(&rig);
    assert!(r[0].len() == 4 && r[0][0].0 == 104.0, "{r:?}");
}

/// A menu taller than the screen (a title menu on a landscape phone): the
/// frame names the control the player cannot reach; the same menu in a
/// scroll container, or on a screen it fits, raises nothing.
#[test]
fn a_control_past_the_screen_edge_with_nothing_to_scroll_to_it_is_named() {
    let menu = |ui: &mut Ui, scrolls: bool| {
        let rows = |ui: &mut Ui| {
            for k in 0..6u32 {
                ui.button(ui_idi("Item", k), "Play", ButtonDesc::default());
            }
        };
        if scrolls {
            ui.scroll(
                "Menu",
                ScrollDesc {
                    height: Size::Fixed(110.0),
                    ..ScrollDesc::default()
                },
                rows,
            );
        } else {
            ui.column("Menu", Element::default(), rows);
        }
    };
    let warnings = |w: f32, h: f32, scrolls: bool| {
        let mut ui = ui_with_font(test_font_metrics());
        ui.begin(InputFrame::default(), Viewport::new(w, h));
        menu(&mut ui, scrolls);
        ui.end().warnings.clone()
    };
    let short = warnings(800.0, 120.0, false);
    let cut: Vec<&String> = short
        .iter()
        .filter(|w| w.contains("past the 800x120 screen"))
        .collect();
    assert!(
        cut.iter().any(|w| w.contains("Item#5")),
        "the last row is named: {short:?}"
    );
    assert!(
        !cut.iter().any(|w| w.contains("Item#0")),
        "the first row fits: {short:?}"
    );
    assert!(
        warnings(800.0, 600.0, false).is_empty(),
        "a screen it fits raises nothing"
    );
    assert!(
        warnings(800.0, 120.0, true).is_empty(),
        "a scroll container reaches every row"
    );
}

/// An ultrawide HUD: on a 32:9 surface (5120×1440 CSS px) the corner groups
/// lay out in a centred 21:9 band (UiDesc.hudMaxAspect's default), at the
/// same size they have at 16:10, with the band's edges where the corners'
/// gutter starts; 0 puts them at the screen's edges again; a dialog keeps
/// the whole surface; a surface narrower than the band (16:10) is the
/// whole safe area.
///
/// Ways it could fail: the corners at the far edges of a 32:9 screen (the
/// player turns their head to read them); a band centred on the screen but
/// not on the safe area; the HUD stretched to the width (a bar 3× wider);
/// the band applied below 21:9, moving every 16:9 game's corners; a modal
/// squeezed into the band.
#[test]
fn hud_corners_sit_in_a_centred_band_on_an_ultrawide() {
    let mut rig = Rig::new(());
    // One og-ui unit at every size: the band, not the unit, is the subject.
    let mut t = *rig.ui.theme();
    t.unit_pin = 1.0;
    rig.ui.set_theme(t);
    let b = &mut |ui: &mut Ui, _: &mut ()| {
        for (id, anchor) in [("Vitals", Anchor::TopLeft), ("Map", Anchor::TopRight)] {
            ui.anchor(
                id,
                AnchorDesc {
                    anchor,
                    width: Size::Fixed(240.0),
                    height: Size::Fixed(48.0),
                    ..AnchorDesc::default()
                },
                |_| {},
            );
        }
    };
    let at = |rig: &Rig<()>, id: &str| rig.ui.frame().element(id).expect(id).clone();
    // 16:10: narrower than the band, so the corners are the safe area's.
    rig.vp = Viewport::new(1680.0, 1050.0);
    rig.idle(1, b);
    let (vitals, map) = (at(&rig, "Vitals"), at(&rig, "Map"));
    let gutter = vitals.x;
    assert!(gutter > 0.0 && gutter < 64.0, "{vitals:?}");
    assert!(
        (1680.0 - (map.x + map.width) - gutter).abs() < 0.5,
        "{map:?}"
    );
    assert_eq!(rig.ui.hud_area(), Rect::new(0.0, 0.0, 1680.0, 1050.0));

    // 32:9 with a safe-area inset on the left (a notch-like 100 px).
    rig.vp = Viewport {
        safe_area: Insets {
            left: 100.0,
            ..Insets::default()
        },
        ..Viewport::new(5120.0, 1440.0)
    };
    rig.idle(1, b);
    let band = 1440.0 * 21.0 / 9.0;
    let left = 100.0 + (5020.0 - band) / 2.0;
    assert_eq!(rig.ui.hud_area(), Rect::new(left, 0.0, band, 1440.0));
    let (wide_vitals, wide_map) = (at(&rig, "Vitals"), at(&rig, "Map"));
    assert!(
        (wide_vitals.x - (left + gutter)).abs() < 0.5,
        "top left of the band, not the screen: {wide_vitals:?}"
    );
    assert!(
        (left + band - (wide_map.x + wide_map.width) - gutter).abs() < 0.5,
        "top right of the band: {wide_map:?}"
    );
    assert_eq!(
        (wide_vitals.width, wide_vitals.height),
        (vitals.width, vitals.height),
        "the HUD never stretches"
    );

    // 0: the whole safe area again.
    rig.ui.set_hud_max_aspect(0.0);
    rig.idle(1, b);
    assert!((at(&rig, "Vitals").x - (100.0 + gutter)).abs() < 0.5);
    assert!((5120.0 - (at(&rig, "Map").x + 240.0) - gutter).abs() < 0.5);

    // A dialog's backdrop covers the whole surface, band or not.
    rig.ui.set_hud_max_aspect(21.0 / 9.0);
    let d = &mut |ui: &mut Ui, _: &mut ()| {
        ui.modal("Pause", true, &ModalDesc::default(), |_| {});
    };
    rig.idle(2, d);
    let scrim = at(&rig, "Pause");
    assert_eq!((scrim.x, scrim.width), (0.0, 5120.0), "{scrim:?}");
}

#[test]
fn an_action_buttons_label_widens_it_when_its_icon_is_not_in_the_atlas() {
    // Wrong: an icon name the atlas lacks draws the label instead, but the
    // button keeps its square size, so a long or translated label is cut.
    let mut ui = ui_with_font(test_font_metrics());
    let desc = ActionButtonDesc {
        icon: "no-such-icon".into(),
        ..ActionButtonDesc::default()
    };
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.action_button("Steer", "Accelerate hard now", &desc);
    ui.end();
    let e = ui.frame().element("Steer").expect("the button").clone();
    assert!(e.width > e.height + 1.0, "widened to its label: {e:?}");
}

#[test]
fn a_charged_action_button_is_a_disc_ringed_in_its_tone() {
    // Wrong: a charge ring drawn round a square pad pokes past its corners
    // (art r4, note 2), or a long label (200 % text) widens the disc into
    // a capsule the ring no longer fits, or the ring takes the wrong tone.
    let mut ui = ui_with_font(test_font_metrics());
    let danger = ui.theme().palette.danger.linear();
    let draw = |ui: &mut Ui, label: &str, charge: Option<f32>| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.action_button(
            "Brake",
            label,
            &ActionButtonDesc {
                size: 64.0,
                charge,
                charge_tone: Tone::Danger,
                ..ActionButtonDesc::default()
            },
        );
        ui.end();
        let e = ui.frame().element("Brake").expect("the pad").clone();
        let lit = ui
            .frame()
            .items
            .iter()
            .any(|i| matches!(&i.cmd, DrawCmd::Quad { color, .. } if *color == danger));
        (e, lit)
    };
    let (e, lit) = draw(&mut ui, "B", Some(1.0));
    assert_eq!((e.width, e.height), (64.0, 64.0), "{e:?}");
    assert!(lit, "a full charge lights the ring in danger");
    let (_, lit) = draw(&mut ui, "B", None);
    assert!(!lit, "no charge, no ring");
    let (e, _) = draw(&mut ui, "Accelerate hard now", Some(0.4));
    assert!(
        e.width > 64.0 && e.width == e.height,
        "a long label grows the disc: {e:?}"
    );
}
