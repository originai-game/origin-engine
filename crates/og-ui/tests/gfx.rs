//! The sample screen rendered headless through og-render and compared with
//! `references/<backend-tier>/sample-screen.png` (og-tests' tolerance: a
//! channel off by more than 32/255 is an outlier; fail above 0.5 % outliers
//! or RGB RMSE 0.01). `OG_UI_CAPTURE=1` records this tier's reference. A tier
//! without its own reference is compared with the `metal` one and says so.

mod support;

use og_core::glam::{Vec2, Vec3};
use og_render::{Background, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap};
use og_ui::*;
use std::path::PathBuf;
use support::*;

/// Ticks of the sample session: focus the search field and type, scroll the
/// list a little, then rest the pointer on a Buy button until its tooltip shows.
fn session(ui: &mut Ui, s: &mut Sample) {
    let vp = viewport();
    let step = |ui: &mut Ui, s: &mut Sample, input: InputFrame| {
        ui.begin(input, vp);
        sample_screen(ui, s);
        ui.end();
    };
    step(ui, s, InputFrame::default());
    let at = |ui: &Ui, path: &str| {
        let e = ui
            .frame()
            .element(path)
            .unwrap_or_else(|| panic!("no element {path}"));
        let (x, y) = e.center();
        Vec2::new(x, y)
    };
    let field = at(ui, "Search");
    let press = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    };
    let release = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    step(ui, s, press(field));
    step(ui, s, release(field));
    step(
        ui,
        s,
        InputFrame {
            text: "pot".into(),
            ..InputFrame::hover(field.x, field.y)
        },
    );
    let list = at(ui, "List");
    step(
        ui,
        s,
        InputFrame {
            wheel: Vec2::new(0.0, 22.0),
            ..InputFrame::hover(list.x, list.y)
        },
    );
    let buy = at(ui, "Row#2/Buy");
    for _ in 0..40 {
        step(ui, s, InputFrame::hover(buy.x, buy.y));
    }
}

fn reference_dir(tier: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("references")
        .join(tier)
}

#[test]
fn sample_screen_matches_its_reference_frame() {
    let vp = viewport();
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let mut s = Sample::default();
    session(&mut ui, &mut s);
    let frame = ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert_eq!(s.search, "pot", "the focused field took the typed text");
    assert!(
        frame.element("tooltip").is_some(),
        "the tooltip shows after the pointer rests on Buy"
    );

    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    frame.emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.bg.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;

    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let tier = renderer.info().backend_tier();
    let out = out_dir();
    let actual = out.join(format!("sample-screen-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");

    // The whole UI is a handful of instanced draws: one atlas run and one
    // font run per painter level at most.
    let levels = frame.levels() as u32;
    println!(
        "sprites {} glyphs {} sprite draws {} levels {} missing glyphs {}",
        stats.sprites, stats.glyphs, stats.sprite_draws, levels, stats.missing_glyphs
    );
    assert!(levels <= 4, "{levels} painter levels");
    assert!(
        stats.sprite_draws <= 2 * levels,
        "{} sprite draws for {levels} levels",
        stats.sprite_draws
    );
    assert_eq!(stats.missing_glyphs, 0);

    let own = reference_dir(&tier).join("sample-screen.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        let metal = reference_dir("metal").join("sample-screen.png");
        println!(
            "no {tier} reference ({}); comparing with the metal one. Record it with OG_UI_CAPTURE=1 on that tier",
            own.display()
        );
        metal
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    assert_eq!(
        (expected.width(), expected.height()),
        (image.width, image.height)
    );
    match compare(&image.pixels, expected.as_raw()) {
        Ok((rmse, outliers)) => println!(
            "{tier}: RMSE {rmse:.5}, outliers {:.3}% against {}",
            outliers * 100.0,
            reference.display()
        ),
        Err(e) => panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        ),
    }
}

/// What a scroll container draws stays inside it, per pixel: the rows it
/// scrolled half out are cut exactly at its edge (quads and text alike), and
/// nothing it holds shows outside its rectangle.
#[test]
fn a_scroll_container_clips_everything_it_draws() {
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let vp = Viewport::new(300.0, 200.0);
    let red = Srgba::rgb(230, 26, 26);
    let build = |ui: &mut Ui, input: InputFrame| {
        let t = *ui.theme();
        ui.begin(input, vp);
        ui.scroll(
            "List",
            ScrollDesc {
                width: Size::Fixed(200.0),
                height: Size::Fixed(100.0),
                gap: t.space.xs,
                ..ScrollDesc::default()
            },
            |ui| {
                for n in 0..10 {
                    ui.row(
                        ui_idi("Row", n),
                        Element {
                            width: Size::grow(),
                            padding: Insets::all(t.space.sm),
                            background: Some(red),
                            radius: 0.0,
                            ..Element::default()
                        },
                        |ui| ui.label("Name", &format!("Item {n}"), LabelDesc::default()),
                    );
                }
            },
        );
        ui.end();
    };
    build(&mut ui, InputFrame::default());
    let list = ui.frame().element("List").expect("list").clone();
    build(
        &mut ui,
        InputFrame {
            wheel: Vec2::new(0.0, 47.0),
            ..InputFrame::hover(50.0, 50.0)
        },
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    // Rows reach past the list's edges before the clip: the cut is the GPU's.
    let (top, bottom) = (list.y, list.y + list.height);
    let beyond = overlay
        .sprites
        .iter()
        .filter(|s| s.position.y < top - 0.5 || s.position.y + s.size.y > bottom + 0.5)
        .count();
    assert!(beyond > 0, "no row reaches past the list, nothing to clip");
    assert!(
        overlay.sprites.iter().all(|s| s.clip.is_some()),
        "every sprite carries a clip"
    );
    scene.overlay = Some(overlay);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let px = |x: u32, y: u32| {
        let i = ((y * image.width + x) * 4) as usize;
        [image.pixels[i], image.pixels[i + 1], image.pixels[i + 2]]
    };
    let reddish = |p: [u8; 3]| p[0] > 120 && p[1] < 80;
    let (x0, x1) = (list.x.ceil() as u32 + 4, (list.x + list.width) as u32 - 4);
    // Outside the list, above and below it: background only.
    for y in (0..top.floor() as u32).chain(bottom.ceil() as u32..image.height) {
        for x in (x0..x1).step_by(3) {
            assert!(
                !reddish(px(x, y)),
                "row colour at ({x}, {y}), outside the list {top}..{bottom}"
            );
        }
    }
    // Inside it, at the first and last pixel rows, the cut rows show.
    let row_at = |y: u32| (x0..x1).any(|x| reddish(px(x, y)));
    assert!(
        row_at(top.ceil() as u32) && row_at(bottom.floor() as u32 - 1),
        "the half-scrolled rows are cut at the list's edges, not dropped"
    );
}
