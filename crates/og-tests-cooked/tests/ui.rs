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
