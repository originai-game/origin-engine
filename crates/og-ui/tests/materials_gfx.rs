//! og-ui's materials frame (`references/<tier>/materials.png`), rendered
//! headless and compared under og-tests' tolerance (`OG_UI_CAPTURE=1`
//! records this tier's): a card in the dark prototype style with its edge light
//! (a 0.5 px highlight along the top and a hairline inside), a row of the
//! UI kit's iconoir icons cooked by og-cook's icon cook (a distance-field
//! set drawn as glyphs, at 24 and 48 px, in two inks) and one inline in
//! rich text, a picture keeping 16:9 across a growing column, a zoom view
//! pinched to 2x by two touches, and a list two touches scrolled by 60 px.
//!
//! It needs the cook (native only), so the browser tiers do not run it.
//!
//! Ways it could fail: an icon missing or drawn as a box (a missing glyph,
//! asserted 0), the icons blurred or cut (the field's range or the em
//! square wrong), the edge light absent or outside the card, the picture
//! out of proportion, the pinch or the two-finger scroll not applied.

mod support;

use og_cook::runtime::{Bundle, CookedFont};
use og_cook::{CookOptions, cook_project};
use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Glyph, Mips, MsdfFont, RenderDesc, RenderScene, RenderTarget, Renderer,
    TextureData, Tonemap,
};
use og_ui::*;
use std::path::PathBuf;
use support::*;

/// The kit's iconoir set and body font cooked into a scratch project.
fn cooked() -> (CookedFont, CookedFont) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui-materials");
    let _ = std::fs::remove_dir_all(&root);
    let kit = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/ui-kit/assets");
    let icons = root.join("assets/icons/iconoir");
    std::fs::create_dir_all(&icons).expect("icons dir");
    for e in std::fs::read_dir(kit.join("icons/iconoir")).expect("iconoir") {
        let e = e.expect("entry");
        std::fs::copy(e.path(), icons.join(e.file_name())).expect("copy svg");
    }
    std::fs::copy(
        kit.join("icons/iconoir.ogt"),
        root.join("assets/icons/iconoir.ogt"),
    )
    .expect("copy set");
    let font = "NotoSansSC-Regular-subset.ttf";
    std::fs::create_dir_all(root.join("assets/fonts")).expect("fonts dir");
    std::fs::copy(
        kit.join("fonts").join(font),
        root.join("assets/fonts").join(font),
    )
    .expect("copy font");
    std::fs::write(
        root.join("assets/fonts").join(format!("{font}.ogt")),
        "type import\nversion 1\n---\nfont {\n  charset latin\n  px_range 4\n}\n",
    )
    .expect("settings");
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).expect("bundle");
    (
        bundle
            .load_font("icons/iconoir.ogt")
            .expect("the icon set loads"),
        bundle
            .load_font(&format!("fonts/{font}"))
            .expect("the font loads"),
    )
}

/// A cooked font (one page) in the scene, as og-runtime adds it.
fn add(scene: &mut RenderScene, f: &CookedFont) -> UiFont {
    assert_eq!(f.pages.len(), 1, "{}: one page", f.id);
    let atlas = scene.add_texture(TextureData {
        mips: Mips::None,
        color_space: ColorSpace::Linear,
        ..f.pages[0].clone()
    });
    let m = &f.meta;
    let font = MsdfFont {
        atlas,
        distance_range: m.px_range,
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter_map(|g| {
                Some(Glyph {
                    codepoint: char::from_u32(g.codepoint)?,
                    advance: g.advance,
                    plane_bounds: [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top],
                    atlas_bounds: [
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ],
                })
            })
            .collect(),
        kerning: Vec::new(),
    };
    let metrics = FontMetrics::from_msdf(&font);
    UiFont {
        handle: Some(scene.add_font(font)),
        metrics,
    }
}

/// A 64×36 picture: a vertical dusk gradient with a sun disc.
fn picture(scene: &mut RenderScene) -> og_render::TextureHandle {
    let (w, h) = (64u32, 36u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let t = y as f32 / (h - 1) as f32;
            let (dx, dy) = (x as f32 - 44.0, y as f32 - 14.0);
            let sun = dx * dx + dy * dy < 36.0;
            let c = if sun {
                [255, 200, 120]
            } else {
                [
                    (40.0 + 200.0 * t) as u8,
                    (30.0 + 90.0 * t) as u8,
                    (90.0 - 30.0 * t) as u8,
                ]
            };
            rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

const ICONS: [&str; 16] = [
    "heart",
    "star",
    "coins",
    "trophy",
    "crown",
    "gift",
    "shield",
    "flash",
    "fire-flame",
    "potion",
    "magic-wand",
    "map",
    "compass",
    "clock",
    "gamepad",
    "settings",
];

fn vp() -> Viewport {
    Viewport {
        width: 640.0,
        height: 400.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

fn build(ui: &mut Ui, tex: og_render::TextureHandle) -> ZoomResponse {
    let t = *ui.theme();
    let mut zoom = None;
    ui.panel(
        "Card",
        PanelDesc {
            width: Size::grow(),
            height: Size::grow(),
            ..PanelDesc::default()
        },
        |ui| {
            ui.row(
                "Icons",
                Element {
                    gap: t.space.sm,
                    cross: Cross::Center,
                    ..Element::row()
                },
                |ui| {
                    for (i, name) in ICONS.iter().enumerate() {
                        let accent = i % 5 == 0;
                        ui.icon(
                            ui_idi("Icon", i as u32),
                            name,
                            IconDesc {
                                size: if i == 0 { 48.0 } else { 24.0 },
                                tint: Some(if accent {
                                    t.palette.primary
                                } else {
                                    t.palette.ink_muted
                                }),
                                ..IconDesc::default()
                            },
                        );
                    }
                },
            );
            ui.rich_text(
                "Rich",
                "Collect [icon=coins] coins, then claim the [icon=trophy] trophy.",
                RichTextDesc::default(),
            );
            ui.row(
                "Pictures",
                Element {
                    width: Size::grow(),
                    gap: t.space.md,
                    ..Element::row()
                },
                |ui| {
                    ui.column(
                        "Left",
                        Element {
                            width: Size::grow(),
                            ..Element::column()
                        },
                        |ui| {
                            ui.image(
                                "Picture",
                                ImageDesc {
                                    texture: Some(tex),
                                    width: Size::grow(),
                                    aspect: 16.0 / 9.0,
                                    ..ImageDesc::default()
                                },
                            )
                        },
                    );
                    ui.column(
                        "Middle",
                        Element {
                            width: Size::grow(),
                            ..Element::column()
                        },
                        |ui| {
                            zoom = Some(ui.zoom_view(
                                "Map",
                                &ZoomViewDesc {
                                    texture: Some(tex),
                                    width: Size::grow(),
                                    aspect: 16.0 / 9.0,
                                    ..ZoomViewDesc::default()
                                },
                            ));
                        },
                    );
                    ui.scroll(
                        "Quests",
                        ScrollDesc {
                            width: Size::px(140.0),
                            height: Size::px(150.0),
                            gap: t.space.xs,
                            ..ScrollDesc::default()
                        },
                        |ui| {
                            for k in 0..10 {
                                ui.button(
                                    ui_idi("Quest", k),
                                    &format!("Quest {}", k + 1),
                                    ButtonDesc {
                                        size: ControlSize::Small,
                                        width: Some(Size::grow()),
                                        ..ButtonDesc::default()
                                    },
                                );
                            }
                        },
                    );
                },
            );
        },
    );
    zoom.expect("the map")
}

/// Two touches at `a` and `b` (pointer 0 is og.Input's pointer).
fn touches(p: &mut og_input::Pointers, at: [Vec2; 2]) -> InputFrame {
    p.end_tick();
    for (id, pos) in at.into_iter().enumerate() {
        let action = if p.down(id as u32) {
            og_input::PointerAction::Move
        } else {
            og_input::PointerAction::Down
        };
        p.apply(&og_input::InjectedInput::Pointer {
            action,
            x: pos.x,
            y: pos.y,
            button: og_input::PointerButton::Left,
            id: id as u32,
        })
        .expect("a pointer id under five");
    }
    InputFrame {
        pointer: p.position(0),
        pointer_inside: true,
        down: true,
        pressed: p.is_pressed(0),
        pointers: *p,
        ..InputFrame::default()
    }
}

#[test]
fn the_materials_frame_matches_its_reference() {
    let (set, body) = cooked();
    assert_eq!(
        set.names.len(),
        81,
        "the kit's set names 72 iconoir icons and 9 of them again by the kit's names"
    );
    // Every name the kit's screens draw (its old PNG grid's 32) is in the set.
    for name in [
        "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem",
        "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back",
        "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music",
        "person",
    ] {
        assert!(set.names.iter().any(|n| n == name), "{name} is in the set");
    }
    let mut scene = RenderScene::new();
    let icons = add(&mut scene, &set);
    let font = add(&mut scene, &body);
    let tex = picture(&mut scene);
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: font,
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    ui.set_icons(IconAtlas {
        font: Some(icons),
        names: set.names.clone(),
        ..IconAtlas::default()
    });
    let step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        let z = build(ui, tex);
        ui.end();
        z
    };
    for _ in 0..30 {
        step(&mut ui, InputFrame::default());
    }
    let center = |ui: &Ui, path: &str| {
        let (x, y) = ui.frame().element(path).expect(path).center();
        Vec2::new(x, y)
    };
    // A pinch on the map about its centre: 60 px apart to 120.
    let map = center(&ui, "Card/Pictures/Middle/Map");
    let mut p = og_input::Pointers::default();
    for d in [30.0, 45.0, 60.0] {
        step(
            &mut ui,
            touches(&mut p, [map - Vec2::new(d, 0.0), map + Vec2::new(d, 0.0)]),
        );
    }
    // The fingers lift.
    step(&mut ui, InputFrame::default());
    // Two fingers up the list by 60.
    let list = center(&ui, "Card/Pictures/Quests");
    let mut q = og_input::Pointers::default();
    for dy in [0.0, 30.0, 60.0] {
        step(
            &mut ui,
            touches(
                &mut q,
                [
                    list + Vec2::new(-20.0, 40.0 - dy),
                    list + Vec2::new(20.0, 40.0 - dy),
                ],
            ),
        );
    }
    let mut zoom = step(&mut ui, InputFrame::default());
    for _ in 0..2 {
        zoom = step(&mut ui, InputFrame::default());
    }
    assert!(
        (zoom.zoom - 2.0).abs() < 1e-4,
        "pinched to 2x ({})",
        zoom.zoom
    );
    let pic = ui
        .frame()
        .element("Card/Pictures/Left/Picture")
        .expect("picture");
    assert!(
        (pic.width / pic.height - 16.0 / 9.0).abs() < 0.02,
        "the picture keeps 16:9 ({}x{})",
        pic.width,
        pic.height
    );
    // The frame.
    let atlas = scene.add_texture(atlas_texture());
    let v = vp();
    let mut overlay = og_render::Overlay::new(v.width, v.height, v.dpr);
    ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (v.width * v.dpr) as u32,
        height: (v.height * v.dpr) as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    assert_eq!(
        stats.missing_glyphs, 0,
        "every icon and character is in its font"
    );
    let image = renderer.read_offscreen().expect("read back");
    let tier = renderer.info().backend_tier();
    let actual = out_dir().join(format!("materials-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    let refs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references");
    let own = refs.join(&tier).join("materials.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        println!("no {tier} reference for materials; comparing with metal");
        refs.join("metal").join("materials.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    match compare(&image.pixels, expected.as_raw()) {
        Ok((rmse, outliers)) => println!(
            "materials {tier}: RMSE {rmse:.5}, outliers {:.3}%",
            outliers * 100.0
        ),
        Err(e) => panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        ),
    }
}
