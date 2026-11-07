//! The UI pass's scene that cooks (`ui-vector`; og-tests' tests/ui.rs has
//! the others): against its reference frame, and its fill read from it.

use og_tests_cooked::gfx::{self, Rendered};
use og_tests_cooked::scenes::{self, GfxScene};

fn render(scene: &str) -> Rendered {
    let mut built: GfxScene = scenes::build(scene).expect("scene");
    gfx::render_built(&mut built).expect("render")
}

/// The sRGB bytes of pixel (x, y) (the overlay is 1280×720 CSS px on a
/// 1280×720 frame: one CSS px a pixel).
fn px(r: &Rendered, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * r.image.width + x) * 4) as usize;
    let p = &r.image.pixels;
    [f32::from(p[i]), f32::from(p[i + 1]), f32::from(p[i + 2])]
}

/// The cooked emblem's even-odd ring keeps its hole.
#[test]
#[ignore = "requires GPU"]
fn ui_vector_fills_by_its_rule() {
    let r = render("ui-vector");
    // The 640 px emblem: view box 120 at 640/120 px a unit from (600, 40).
    let k = 640.0 / 120.0;
    let at = |ux: f32, uy: f32| px(&r, (600.0 + ux * k) as u32, (40.0 + uy * k) as u32);
    let ring = at(60.0, 56.0 - 26.0);
    let hole = at(60.0 + 16.0, 56.0 + 6.0);
    println!("ring {ring:?} hole {hole:?}");
    assert!(ring[2] > ring[0] + 40.0, "the ring is not blue: {ring:?}");
    assert!(hole[0] > hole[2] + 60.0, "the hole is filled: {hole:?}");
}

/// `ui-rive`: converted Rive artboards draw where their numbers put them.
/// Each artboard is contained in a 280 px box from its cell's corner.
#[test]
#[ignore = "requires GPU"]
fn ui_rive_draws_its_artboards_where_the_file_puts_them() {
    let r = render("ui-rive");
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 3.0);
    let grey = |v: f32| [v; 3];
    // shapetest: 400 × 200 at 0.7 from (40, 60 + 70); the circle of radius
    // 65.94 about (205.14, 97.67), Rive's default grey 0x74 on 0x31.
    let (cx, cy) = (40.0 + 205.14 * 0.7, 130.0 + 97.67 * 0.7);
    let c = px(&r, cx as u32, cy as u32);
    assert!(near(c, grey(116.0)), "the circle: {c:?}");
    let out = px(&r, (cx + 65.94 * 0.7 + 6.0) as u32, cy as u32);
    assert!(near(out, grey(49.0)), "past the circle: {out:?}");
    // nested_artboard_opacity: the nested red artboard at 0.3275 over the
    // parent's blue, 125..375 of 500 at 0.56 from (340, 60).
    let inside = px(&r, 480, 200);
    let outside = px(&r, 360, 80);
    assert!(
        inside[0] > 100.0 && outside[0] < 30.0,
        "{inside:?} {outside:?}"
    );
    // cubic_value_test at frame 24 (third cell of the second row): the
    // key's own x, 333.98, of a 159.8 px rectangle at 0.56 from (640, 380).
    let x = 640.0 + 333.98 * 0.56;
    let y = 380.0 + 143.47 * 0.56;
    for dx in [-40.0, 40.0] {
        let p = px(&r, (x + dx) as u32, y as u32);
        assert!(
            near(p, grey(116.0)),
            "inside the keyed rectangle at {dx}: {p:?}"
        );
    }
    for dx in [-50.0, 50.0] {
        let p = px(&r, (x + dx) as u32, y as u32);
        assert!(
            near(p, grey(49.0)),
            "outside the keyed rectangle at {dx}: {p:?}"
        );
    }
}

/// A Rive text through the whole path: the `.riv` cooked (its artboard
/// and its font, the font's charset the run's characters), the artboard
/// bound to the cooked font, played in a document's frame by og-ui and
/// drawn by the UI pass. rive-runtime's hello_world.riv: "Hello World!" at
/// 60 px in 0x00f3ff, its text box's top left at (73.48, 175.79) of a
/// 500 px artboard on 0x313131, drawn at its own size.
///
/// Ways it could fail: text that cooks but draws nothing (an unbound font,
/// a dropped command), glyphs the charset lacks (missing glyphs, asserted
/// 0), the text placed by its baseline instead of its box's top or not
/// moved by its node, a run measured in one font and drawn in another.
#[test]
#[ignore = "requires GPU"]
fn ui_rive_text_draws_its_run_in_its_cooked_font() {
    use crate::support::text_scenes::add;
    use og_cook::runtime::Bundle;
    use og_render::{RenderDesc, RenderScene, RenderTarget, Renderer};
    use og_ui::*;
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-rive-text");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/ui")).unwrap();
    std::fs::copy(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/rive/assets/hello_world.riv"),
        root.join("assets/ui/hello_world.riv"),
    )
    .unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = root.join("cooked");
    let bundle = Bundle::open(&cooked).unwrap();
    let bytes = |id: &str| std::fs::read(cooked.join(&bundle.output(id).unwrap().file)).unwrap();
    let id = "ui/hello_world.riv#New Artboard";
    let mut board = rive::decode(&bundle.rive_artboard(id, &bytes(id)).unwrap()).unwrap();
    let font = bundle
        .font(
            "ui/hello_world.riv#font0",
            &bytes("ui/hello_world.riv#font0"),
        )
        .unwrap();
    let mut scene = RenderScene::new();
    let ui_font = add(&mut scene, &font, true);
    board.bind_fonts(&mut |f| (f == "ui/hello_world.riv#font0").then(|| ui_font.clone()));
    let doc = UiDocument::parse(
        "ui/hello.ogt",
        "type screen\nversion 2\n---\nname \"hello\"\ncolumn Hello {\n  width: 500px\n  height: 500px\n  -og-frame: rive(\"ui/hello_world.riv#New Artboard\")\n}\n",
    )
    .unwrap();
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: ui_font.clone(),
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    ui.define_rive(id, std::sync::Arc::new(board));
    ui.define_document(doc);
    ui.push_screen("hello", serde_json::Value::Null).unwrap();
    let vp = Viewport {
        width: 500.0,
        height: 500.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.end();
    }
    let frame = ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert!(
        frame.missing_glyphs.is_empty(),
        "{:?}",
        frame.missing_glyphs
    );
    let text = frame
        .elements
        .iter()
        .find(|e| e.kind == "rive" && e.id.ends_with("/Hello World"))
        .expect("the text as an element");
    assert_eq!(text.text, "Hello World!");
    // Its box: from the node, as wide as the run measures in its font.
    let width = ui_font.metrics.measure("Hello World!", 60.0);
    assert!(
        (text.x - 73.48).abs() < 0.1 && (text.y - 175.79).abs() < 0.1,
        "{text:?}"
    );
    assert!(
        (text.width - width).abs() < 0.1,
        "{} vs {width}",
        text.width
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    frame.emit(&EmitDesc::default(), &mut overlay);
    scene.overlay = Some(overlay);
    let desc = RenderDesc {
        width: 500,
        height: 500,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    assert_eq!(stats.missing_glyphs, 0);
    let image = renderer.read_offscreen().expect("read back");
    let out = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    std::fs::create_dir_all(&out).unwrap();
    image::save_buffer(
        out.join("ui-rive-text.png"),
        &image.pixels,
        500,
        500,
        image::ColorType::Rgba8,
    )
    .unwrap();
    // Cyan ink inside the text's box, none outside it.
    let cyan = |x: u32, y: u32| {
        let i = ((y * 500 + x) * 4) as usize;
        let p = &image.pixels[i..i + 3];
        p[0] < 80 && p[1] > 180 && p[2] > 200
    };
    let (mut inside, mut outside) = (0, 0);
    for y in 0..500 {
        for x in 0..500 {
            if cyan(x, y) {
                let (fx, fy) = (x as f32, y as f32);
                if fx >= text.x - 1.0
                    && fx <= text.x + text.width + 1.0
                    && fy >= text.y - 1.0
                    && fy <= text.y + text.height + 1.0
                {
                    inside += 1;
                } else {
                    outside += 1;
                }
            }
        }
    }
    println!("ink inside {inside}, outside {outside}, box {text:?}");
    assert!(inside > 2000, "the words are drawn: {inside}");
    assert_eq!(outside, 0);
    // The background is the artboard's.
    let i = ((20 * 500 + 20) * 4) as usize;
    assert!(
        image.pixels[i..i + 3]
            .iter()
            .all(|v| (*v as i32 - 0x31).abs() <= 2)
    );
}

/// The fixture COLR font cooked with `chars` and `color true`, its pages
/// added as og-runtime adds them (extract::add_font): the field's, the
/// colour pages (distance range 0), then the foreground pages.
fn colr_font(
    name: &str,
    chars: &str,
) -> (
    og_render::RenderScene,
    Vec<og_render::FontHandle>,
    og_cook::runtime::CookedFont,
) {
    use og_cook::runtime::Bundle;
    use og_render::{ColorSpace, Glyph, Mips, MsdfFont, RenderScene, TextureData};
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/fonts")).unwrap();
    std::fs::copy(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/ui-text/assets/fonts/test_glyphs-glyf_colr_1.ttf"),
        root.join("assets/fonts/colr.ttf"),
    )
    .unwrap();
    std::fs::write(
        root.join("assets/fonts/colr.ttf.ogt"),
        format!("type import\nversion 1\n---\nfont {{\n  charset\n  chars \"{chars}\"\n  color true\n}}\n"),
    )
    .unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = root.join("cooked");
    let bundle = Bundle::open(&cooked).unwrap();
    let out = bundle.output("fonts/colr.ttf").unwrap();
    let f = bundle
        .font(
            "fonts/colr.ttf",
            &std::fs::read(cooked.join(&out.file)).unwrap(),
        )
        .unwrap();
    let mut scene = RenderScene::new();
    let m = &f.meta;
    let mut handles = Vec::new();
    let all = f
        .pages
        .iter()
        .map(|p| (p, false, false))
        .chain(f.color_pages.iter().map(|p| (p, true, false)))
        .chain(f.foreground_pages.iter().map(|p| (p, true, true)));
    for (p, (data, colour, foreground)) in all.enumerate() {
        let atlas = scene.add_texture(if colour {
            data.clone()
        } else {
            TextureData {
                mips: Mips::None,
                color_space: ColorSpace::Linear,
                ..data.clone()
            }
        });
        let glyphs = f
            .glyphs
            .iter()
            .filter_map(|g| {
                let own = g.page as usize == p;
                Some(Glyph {
                    codepoint: char::from_u32(g.codepoint)?,
                    advance: g.advance,
                    plane_bounds: if own {
                        [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top]
                    } else {
                        [0.0; 4]
                    },
                    atlas_bounds: if own {
                        [
                            g.atlas_x as f32,
                            g.atlas_y as f32,
                            (g.atlas_x + g.atlas_w) as f32,
                            (g.atlas_y + g.atlas_h) as f32,
                        ]
                    } else {
                        [0.0; 4]
                    },
                })
            })
            .collect();
        handles.push(scene.add_font(MsdfFont {
            atlas,
            distance_range: if colour { 0.0 } else { m.px_range },
            foreground,
            line_height: m.ascender - m.descender + m.line_gap,
            glyphs,
            kerning: Vec::new(),
        }));
    }
    (scene, handles, f)
}

/// `text` as a 128 px og-ui label in `color` on a 400x200 frame, every
/// page of the font drawn from its layout (og-runtime's widgets): the
/// frame's sRGB pixels and the label's box (x, y, w, h).
fn colr_label(
    mut scene: og_render::RenderScene,
    handles: &[og_render::FontHandle],
    text: &str,
    color: og_ui::Srgba,
) -> (Vec<u8>, [f32; 4]) {
    use og_render::{RenderDesc, RenderTarget, Renderer};
    use og_ui::*;
    let font = UiFont::new(handles[0], scene.font(handles[0]).unwrap());
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: font.clone(),
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    let vp = Viewport {
        width: 400.0,
        height: 200.0,
        dpr: 1.0,
        ..Viewport::default()
    };
    let size = 128.0;
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        let step = TypeStep {
            size,
            line_height: size * 1.25,
            ..ui.theme().type_ramp.body
        };
        ui.label(
            "Emoji",
            text,
            LabelDesc {
                step: Some(step),
                color: Some(color),
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        ui.end();
    }
    let frame = ui.frame();
    assert!(
        frame.missing_glyphs.is_empty(),
        "{:?}",
        frame.missing_glyphs
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    frame.emit(&EmitDesc::default(), &mut overlay);
    let more: Vec<_> = overlay
        .texts
        .iter()
        .flat_map(|t| {
            handles[1..].iter().map(move |&h| og_render::Text {
                font: h,
                ..t.clone()
            })
        })
        .collect();
    overlay.texts.extend(more);
    scene.overlay = Some(overlay);
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 400,
        height: 200,
        ..RenderDesc::default()
    })
    .expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    assert_eq!(stats.missing_glyphs, 0);
    let image = renderer.read_offscreen().expect("read back");
    let e = frame
        .elements
        .iter()
        .find(|e| e.id.ends_with("Emoji"))
        .expect("the text's element");
    (image.pixels, [e.x, e.y, e.width, e.height])
}

/// COLR colour glyphs (`font { color true }`) drawn through og-ui's text:
/// the cooked colour page is a font page of its own, its glyphs images in
/// their gradients' colours, untinted by the label's colour, beside a
/// plain distance-field glyph of the same font.
///
/// Ways this could fail: the colour page goes through the MSDF shader (a
/// solid-coloured or empty glyph); the label's colour tints the image; the
/// page's glyphs are drawn on the field page too (a second, wrong glyph);
/// the gradient is mirrored or its stops misread.
#[test]
#[ignore = "requires GPU"]
fn colour_glyphs_draw_in_their_own_colours() {
    // U+1F100: a linear gradient red to blue left to right; U+1F600 (a
    // composite of two squares). No other glyph of the font has ink
    // outside its colour paint, so the field page is empty.
    let (scene, handles, f) = colr_font("og-colr", "\u{1F100}\u{1F600}");
    assert_eq!(f.color_pages.len(), 1);
    assert!(f.foreground_pages.is_empty());
    // A label colour that must not tint the images.
    let (pixels, [ex, ey, _, eh]) = colr_label(
        scene,
        &handles,
        "\u{1F100}\u{1F600}",
        og_ui::Srgba::rgb(0, 255, 0),
    );
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    std::fs::create_dir_all(&dir).unwrap();
    image::save_buffer(
        dir.join("ui-colr.png"),
        &pixels,
        400,
        200,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let size = 128.0;
    let px = |x: f32, y: f32| {
        let i = ((y as u32 * 400 + x as u32) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    // U+1F100 fills 0.1..0.9 em across: on some row of its box its fifth
    // from the left is red and its fifth from the right blue.
    let (lx, rx) = (ex + size * 0.26, ex + size * 0.74);
    let rows = (ey.max(0.0) as u32..(ey + eh).min(200.0) as u32)
        .filter(|&y| {
            let (l, r) = (px(lx, y as f32), px(rx, y as f32));
            l[0] > l[2] + 60 && r[2] > r[0] + 60 && l[1] < 60 && r[1] < 60
        })
        .count();
    println!("gradient rows {rows}");
    // The glyph is 0.7 em tall.
    assert!(rows as f32 > size * 0.6, "{rows} rows of red to blue");
    // No green anywhere: the label colour never reaches the images.
    let green = pixels
        .chunks(4)
        .filter(|p| p[1] > 150 && p[0] < 100 && p[2] < 100)
        .count();
    assert_eq!(green, 0);
}

/// COLR palette entry 0xFFFF is the text's colour: a glyph that mixes a
/// fixed colour with it draws in the label's colour, through gradients,
/// under each of two label colours, and its fixed colour stays put.
///
/// Ways this could fail: the foreground drawn black (the cooked default)
/// or white (the twin read as the glyph); the twin sampled at the wrong
/// offset (a neighbour's pixels or a transparent gutter); the mix done in
/// linear light (a darker mid-tone than the sRGB gradient the cook paints);
/// the fixed orange tinted by the label.
#[test]
#[ignore = "requires GPU"]
fn foreground_glyphs_draw_in_the_label_colour() {
    // U+1FB00: a linear gradient across 0.1..0.9 em, orange at both ends
    // and the foreground at its middle; U+1FB06: the foreground, solid.
    let text = "\u{1FB00}\u{1FB06}";
    let size = 128.0;
    for (name, label) in [("green", [0u8, 200, 60]), ("magenta", [220, 40, 200])] {
        let (scene, handles, f) = colr_font("og-colr-fg", text);
        assert_eq!(f.foreground_pages.len(), 1);
        let (pixels, [ex, ey, _, eh]) = colr_label(
            scene,
            &handles,
            text,
            og_ui::Srgba::rgb(label[0], label[1], label[2]),
        );
        let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
        std::fs::create_dir_all(&dir).unwrap();
        image::save_buffer(
            dir.join(format!("ui-colr-fg-{name}.png")),
            &pixels,
            400,
            200,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let px = |x: f32, y: f32| {
            let i = ((y as u32 * 400 + x as u32) * 4) as usize;
            [pixels[i], pixels[i + 1], pixels[i + 2]]
        };
        let near =
            |p: [u8; 3], q: [u8; 3], d: u8| p.iter().zip(q).all(|(&a, b)| a.abs_diff(b) <= d);
        // The glyph is 0.7 em tall (0.25..0.95 em up from the baseline):
        // count the rows of its box where each probe matches.
        let rows = |x: f32, want: [u8; 3], d: u8| {
            (ey.max(0.0) as u32..(ey + eh).min(200.0) as u32)
                .filter(|&y| near(px(x, y as f32), want, d))
                .count() as f32
        };
        let advance = 1.0 * size;
        let mid = rows(ex + size * 0.5, label, 8);
        let solid = rows(ex + advance + size * 0.5, label, 4);
        let ends = rows(ex + size * 0.12, [255, 165, 0], 24);
        println!("{name}: mid {mid}, solid {solid}, orange ends {ends}");
        assert!(
            mid > size * 0.6,
            "{name}: {mid} rows of the label colour mid-gradient"
        );
        assert!(
            solid > size * 0.6,
            "{name}: {solid} rows of the solid foreground"
        );
        assert!(
            ends > size * 0.6,
            "{name}: {ends} rows of orange at the end"
        );
        // A quarter of the way: the sRGB midpoint of orange and the label.
        let q = |k: usize| ((255.0f32 * [1.0, 0.647, 0.0][k] + f32::from(label[k])) / 2.0) as u8;
        let quarter = rows(ex + size * 0.3, [q(0), q(1), q(2)], 24);
        assert!(
            quarter > size * 0.6,
            "{name}: {quarter} rows of the sRGB mix"
        );
    }
}
