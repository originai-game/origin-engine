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

fn reference_dir(tier: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("references")
        .join(tier)
}

#[test]
fn sample_screen_matches_its_reference_frame() {
    let (shot, s) = with_test_font(sample_screen_shot);
    let frame = shot.ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert_eq!(s.search, "pot", "the focused field took the typed text");
    assert!(
        frame.element("tooltip").is_some(),
        "the tooltip shows after the pointer rests on Buy"
    );

    let mut renderer = Renderer::new_blocking(&shot.desc).expect("a headless adapter");
    let stats = renderer
        .render(&shot.scene, &RenderTarget::Offscreen)
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

/// A skin's text glow is one soft halo read from the font's distance field
/// (og-render's MSDF shader), not offset copies of the text: the ink is the
/// same with and without it, the pixels just outside the glyphs take the
/// glow, it ends within the distance field's reach, each glyph costs two
/// instances (its glow, then its fill), and the frame matches
/// `references/<tier>/text-glow.png`.
///
/// Ways it could fail: the glow is drawn over the ink (the letters wash
/// out), offset copies move the ink, the halo never shows, or it reaches
/// past the distance range (where the field is flat, it would end at the
/// glyph quad's hard edge).
#[test]
fn a_text_glow_is_a_halo_from_the_distance_field() {
    let render = |glow: bool| {
        let (scene, desc) = with_test_font(|f| glow_scene(f, glow, 1));
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        let stats = renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let tier = renderer.info().backend_tier();
        (renderer.read_offscreen().expect("read back"), stats, tier)
    };
    let (plain, plain_stats, _) = render(false);
    let (glow, glow_stats, tier) = render(true);
    assert_eq!(
        glow_stats.overlay_glyphs,
        2 * plain_stats.overlay_glyphs,
        "a glowing glyph is two instances: its glow and its fill"
    );
    let (w, h) = (plain.width as usize, plain.height as usize);
    let at = |px: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [px[i] as i32, px[i + 1] as i32, px[i + 2] as i32]
    };
    let bg = at(&plain.pixels, 2, 2);
    let lum = |p: [i32; 3]| p[0] + p[1] + p[2];
    let is_ink = |x: usize, y: usize| lum(at(&plain.pixels, x, y)) - lum(bg) > 300;
    let (mut ink_same, mut ink, mut near, mut far) = (0usize, 0usize, 0usize, 0usize);
    for y in 0..h {
        for x in 0..w {
            let p = at(&plain.pixels, x, y);
            let g = at(&glow.pixels, x, y);
            let solid = x > 0
                && y > 0
                && x + 1 < w
                && y + 1 < h
                && (y - 1..=y + 1).all(|yy| (x - 1..=x + 1).all(|xx| is_ink(xx, yy)));
            if solid {
                // Solid ink (its neighbours ink too, no antialiased edge):
                // unchanged by the glow under it.
                ink += 1;
                if (0..3).all(|c| (p[c] - g[c]).abs() <= 8) {
                    ink_same += 1;
                }
            } else if p == bg {
                // Background: tinted next to the ink, untouched far from it.
                let d = (1usize..=8)
                    .find(|&r| {
                        let (x0, x1) = (x.saturating_sub(r), (x + r).min(w - 1));
                        let (y0, y1) = (y.saturating_sub(r), (y + r).min(h - 1));
                        (y0..=y1).any(|yy| (x0..=x1).any(|xx| is_ink(xx, yy)))
                    })
                    .unwrap_or(99);
                let tinted = g != bg;
                if d <= 2 && tinted {
                    near += 1;
                }
                if d > 6 && tinted {
                    far += 1;
                }
            }
        }
    }
    assert!(ink > 200, "the labels drew solid ink ({ink} px)");
    assert!(
        ink_same as f64 >= 0.95 * ink as f64,
        "the glow lies under the ink: {ink_same} of {ink} ink pixels unchanged"
    );
    assert!(
        near > 200,
        "the background next to the ink glows ({near} px)"
    );
    assert_eq!(far, 0, "nothing glows past the distance field's reach");
    let out = out_dir().join(format!("text-glow-{tier}.png"));
    image::save_buffer(
        &out,
        &glow.pixels,
        glow.width,
        glow.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    let own = reference_dir(&tier).join("text-glow.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&out, &own).expect("record reference");
    }
    let reference = if own.exists() {
        own
    } else {
        reference_dir("metal").join("text-glow.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    if let Err(e) = compare(&glow.pixels, expected.as_raw()) {
        panic!(
            "{} differs from {}: {e}",
            out.display(),
            reference.display()
        );
    }
}

/// A type step of the ramp.
type StepOf = fn(&TypeRamp) -> TypeStep;

/// One line of rich text rendered through og-render at a type step.
fn rich_frame(markup: &str, step: StepOf) -> (Vec<u8>, usize, usize, [u8; 3]) {
    let vp = Viewport {
        dpr: 1.0,
        ..Viewport::new(360.0, 120.0)
    };
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        let t = *ui.theme();
        ui.column(
            "Card",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets::all(t.space.md),
                gap: t.space.md,
                background: Some(t.palette.bg),
                ..Element::default()
            },
            |ui| {
                let step = step(&t.type_ramp);
                ui.rich_text(
                    "Line",
                    markup,
                    RichTextDesc {
                        step: Some(step),
                        ..RichTextDesc::default()
                    },
                );
            },
        );
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.bg.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
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
    let img = renderer.read_offscreen().expect("read back");
    let bg = [img.pixels[0], img.pixels[1], img.pixels[2]];
    (img.pixels, img.width as usize, img.height as usize, bg)
}

/// `[b]` is heavier ink around the same glyphs at every size: never the
/// glyph quads filled. Small text from a narrow distance range (the test
/// font's 4 px at 32 px: at `text-xs` the field is too thin to thicken)
/// still turns bold: bold also draws a second copy a device pixel to the
/// right.
///
/// Ways it could fail: the weight runs past the field's reach and fills
/// each glyph's quad (a box behind the digit, as the first cut drew at
/// the reading size), bold draws nothing heavier at small sizes (no
/// second copy), or it moves the glyphs.
#[test]
fn bold_rich_text_is_heavier_ink_never_a_box() {
    let steps: [(&str, StepOf); 3] = [
        ("xs", |r| r.xs),
        ("reading", |r| r.reading),
        ("display", |r| r.display_md),
    ];
    for (name, step) in steps {
        let (plain, w, h, bg) = rich_frame("Slot 42 W", step);
        let (bold, ..) = rich_frame("[b]Slot 42 W[/b]", step);
        let ink = |px: &[u8], x: usize, y: usize| {
            let i = (y * w + x) * 4;
            (0..3)
                .map(|c| (px[i + c] as i32 - bg[c] as i32).abs())
                .sum::<i32>()
                > 120
        };
        let count = |px: &[u8]| {
            (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| ink(px, x, y))
                .count()
        };
        let (p, b) = (count(&plain), count(&bold));
        assert!(p > 40, "the {name} line drew ink ({p} px)");
        assert!(
            b as f64 >= 1.15 * p as f64,
            "bold is heavier at the {name} size: {b} against {p} px"
        );
        // Every bold ink pixel lies within three pixels of the plain ink
        // (the weight, at most 0.045 em, and the one-pixel second copy).
        let mut stray = 0;
        for y in 0..h {
            for x in 0..w {
                if ink(&bold, x, y) {
                    let near = (y.saturating_sub(3)..=(y + 3).min(h - 1)).any(|yy| {
                        (x.saturating_sub(3)..=(x + 3).min(w - 1)).any(|xx| ink(&plain, xx, yy))
                    });
                    if !near {
                        stray += 1;
                    }
                }
            }
        }
        assert_eq!(stray, 0, "bold ink at the {name} size stays on its glyphs");
    }
}

/// The stripes scene (`stripes_scene`) under `glass` panels, rendered and
/// read back.
fn stripes_frame(glass: &[og_render::Glass]) -> (Vec<u8>, usize, usize, og_render::FrameStats) {
    let (scene, desc) = stripes_scene(glass);
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("read back");
    (img.pixels, desc.width as usize, desc.height as usize, stats)
}

/// A glass panel shows the frame under it blurred, inside its rounded
/// rectangle only, and leaves every other pixel exactly as it was: the
/// fine stripes under it smooth to their average while the stripes beside
/// it keep their contrast, its corners stay sharp stripes, and a tint
/// covers the blur by its alpha.
///
/// Ways it could fail: the frame moves through the glass source and back
/// changed (the copy is not exact), the blur never reaches the panel
/// (sharp stripes inside), it spills past the rounded rectangle, the
/// pyramid is not decoded (a darker or brighter average), or the tint is
/// ignored.
#[test]
fn a_glass_panel_blurs_the_frame_under_it_and_nothing_else() {
    let panel = glass_panel();
    let (plain, w, h, _) = stripes_frame(&[]);
    let (glass, _, _, stats) = stripes_frame(&[panel]);
    let px = |p: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [p[i] as i32, p[i + 1] as i32, p[i + 2] as i32]
    };
    let lum = |c: [i32; 3]| (c[0] + c[1] + c[2]) / 3;
    // Outside the panel, every pixel is the plain frame's.
    let mut moved = 0;
    for y in 0..h {
        for x in 0..w {
            let inside = (80..240).contains(&x) && (40..160).contains(&y);
            if !inside && px(&plain, x, y) != px(&glass, x, y) {
                moved += 1;
            }
        }
    }
    assert_eq!(moved, 0, "the glass leaves the frame outside it untouched");
    // The plain stripes run from black to white along a row.
    let row = |p: &[u8], y: usize, x0: usize, x1: usize| -> (i32, i32, f64) {
        let v: Vec<i32> = (x0..x1).map(|x| lum(px(p, x, y))).collect();
        let mean = v.iter().sum::<i32>() as f64 / v.len() as f64;
        (*v.iter().min().unwrap(), *v.iter().max().unwrap(), mean)
    };
    let (lo, hi, plain_mean) = row(&plain, 100, 120, 200);
    assert!(lo < 10 && hi > 245, "the stripes are sharp ({lo}..{hi})");
    let (glo, ghi, glass_mean) = row(&glass, 100, 120, 200);
    assert!(
        ghi - glo < 24,
        "under the glass the stripes blur to their average ({glo}..{ghi})"
    );
    // The average of linear light: half-white in linear is ~188 in sRGB.
    assert!(
        (glass_mean - 188.0).abs() < 16.0,
        "the blur averages light, not encoded values ({glass_mean:.0}; the plain row's mean {plain_mean:.0})"
    );
    // The rounded corner: its outer pixel shows the sharp stripes.
    assert_eq!(
        px(&glass, 81, 41),
        px(&plain, 81, 41),
        "outside the rounded corner the frame shows as it is"
    );
    assert!(stats.overlay_sprites >= 1, "the panel is drawn");
    // A tint covers the blur by its alpha.
    let red = og_render::Glass {
        tint: [1.0, 0.0, 0.0, 1.0],
        ..panel
    };
    let (tinted, ..) = stripes_frame(&[red]);
    let c = px(&tinted, 160, 100);
    assert!(
        c[0] > 245 && c[1] < 10 && c[2] < 10,
        "a full tint is the tint ({c:?})"
    );
}

/// `scene` rendered and read back.
fn rendered(scene: &RenderScene, desc: &RenderDesc) -> (Vec<u8>, og_render::FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("a headless adapter");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    (renderer.read_offscreen().expect("read back").pixels, stats)
}

/// Glass over overlay content: the panel blurs the overlay layers drawn
/// below it as it blurs the scene (the amber bar of
/// `glass_overlay_content`, on the layer under the panel, spreads into the
/// stripes inside the panel and stays sharp outside it), what is drawn
/// above it (the cyan square) stays sharp, and the frame matches
/// `references/<tier>/glass.png`.
///
/// Ways it could fail: the overlay under the glass is drawn after the
/// pyramid was built (the bar sharp inside the panel, the old behaviour),
/// the overlay drawn into the glass source never reaches the output (the
/// bar missing outside the panel), the pyramid is rebuilt from a source
/// that already holds the content above the glass (the square smeared).
#[test]
fn a_glass_panel_blurs_the_overlay_layers_below_it() {
    let (scene, desc) = glass_reference_scene();
    let (glass, stats) = rendered(&scene, &desc);
    let (mut plain_scene, _) = glass_reference_scene();
    if let Some(o) = plain_scene.overlay.as_mut() {
        o.glass.clear();
    }
    let (plain, _) = rendered(&plain_scene, &desc);
    let w = desc.width as usize;
    let px = |p: &[u8], x: usize, y: usize| -> [i32; 3] {
        let i = (y * w + x) * 4;
        [p[i] as i32, p[i + 1] as i32, p[i + 2] as i32]
    };
    // Outside the panel the bar is drawn as it is, sharp.
    assert_eq!(
        px(&glass, 60, 100),
        px(&plain, 60, 100),
        "the bar outside the panel"
    );
    let amber = px(&plain, 60, 100);
    assert!(
        amber[0] > 240 && amber[2] < 20,
        "the bar is amber ({amber:?})"
    );
    assert_eq!(
        px(&glass, 60, 86),
        px(&plain, 60, 86),
        "and the stripes above it"
    );
    // Inside the panel the bar is blurred: its amber reaches 8 px above
    // its edge, where the plain frame has grey stripes.
    for x in [150, 154, 158] {
        let (g, p) = (px(&glass, x, 84), px(&plain, x, 84));
        assert_eq!(p[0], p[2], "the plain frame there is a grey stripe ({p:?})");
        assert!(
            g[0] - g[2] > 24,
            "the bar under the glass spreads ({x}, 84): {g:?}"
        );
    }
    // On the bar's middle row the stripes no longer show through.
    let row: Vec<i32> = (150..170).map(|x| px(&glass, x, 100)[2]).collect();
    let (lo, hi) = (*row.iter().min().unwrap(), *row.iter().max().unwrap());
    assert!(
        hi - lo < 24,
        "the bar's row under the glass is smooth ({lo}..{hi})"
    );
    // The square above the glass is as sharp as without it.
    for (x, y) in [(190, 60), (198, 68), (205, 75)] {
        assert_eq!(
            px(&glass, x, y),
            px(&plain, x, y),
            "the square above the glass ({x}, {y})"
        );
    }
    assert_ne!(px(&glass, 188, 68), px(&glass, 191, 68), "with a hard edge");
    assert!(stats.draw_calls > 0);
    // The reference frame.
    let (h, w) = (desc.height, desc.width);
    let tier = Renderer::new_blocking(&RenderDesc::default())
        .expect("a headless adapter")
        .info()
        .backend_tier();
    let out = out_dir().join(format!("glass-{tier}.png"));
    image::save_buffer(&out, &glass, w, h, image::ColorType::Rgba8).expect("write frame");
    let own = reference_dir(&tier).join("glass.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&out, &own).expect("record reference");
    }
    let reference = if own.exists() {
        own
    } else {
        reference_dir("metal").join("glass.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    if let Err(e) = compare(&glass, expected.as_raw()) {
        panic!(
            "{} differs from {}: {e}",
            out.display(),
            reference.display()
        );
    }
}

/// The glass's cost on this machine's GPU at 1280×720: a frame with four
/// dialog-sized glass panels against the same frame without
/// (`cargo test -p og-ui --release --test gfx -- --ignored glass_cost
/// --nocapture`; tests/evidence/P2/ui/glass-cost.md).
#[test]
#[ignore]
fn glass_cost() {
    let (w, h) = (1280u32, 720u32);
    let mut best = [f64::INFINITY; 2];
    for round in 0..6 {
        let on = round % 2 == 1;
        let mut scene = RenderScene::new();
        scene.camera = og_render::Camera::orthographic_2d(Vec2::new(640.0, 360.0), 720.0);
        for k in 0..160 {
            scene.sprites.push(og_render::Sprite {
                position: Vec3::new(k as f32 * 8.0 + 2.0, 360.0, 0.0),
                size: Vec2::new(4.0, 720.0),
                ..og_render::Sprite::default()
            });
        }
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        let mut overlay = og_render::Overlay::new(w as f32, h as f32, 1.0);
        if on {
            for k in 0..4 {
                overlay.glass.push(og_render::Glass {
                    position: Vec2::new(40.0 + k as f32 * 300.0, 160.0),
                    size: Vec2::new(280.0, 400.0),
                    radius: 24.0,
                    blur: 16.0,
                    tint: [0.1, 0.1, 0.12, 0.2],
                    alpha: 1.0,
                    layer: 0,
                    clip: None,
                });
            }
        }
        scene.overlay = Some(overlay);
        let desc = RenderDesc {
            width: w,
            height: h,
            ..RenderDesc::default()
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("read back");
        let n = 600;
        let start = std::time::Instant::now();
        for _ in 0..n {
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("read back");
        let ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
        best[usize::from(on)] = best[usize::from(on)].min(ms);
        println!("round {round}: glass {on}: {ms:.3} ms per frame");
    }
    println!(
        "best: without {:.3} ms, with four panels {:.3} ms, cost {:+.3} ms per frame ({})",
        best[0],
        best[1],
        best[1] - best[0],
        Renderer::new_blocking(&RenderDesc::default())
            .expect("a headless adapter")
            .info()
            .backend_tier()
    );
}

/// The glow's cost on this machine's GPU: 24 display-size labels rendered
/// 300 times with and without it (`cargo test -p og-ui --release --test gfx
/// -- --ignored text_glow_cost --nocapture`; tests/evidence/P2/ui/text-glow-cost.md).
#[test]
#[ignore]
fn text_glow_cost() {
    let mut best = [f64::INFINITY; 2];
    for round in 0..6 {
        let glow = round % 2 == 1;
        let (scene, desc) = with_test_font(|f| glow_scene(f, glow, 24));
        let desc = RenderDesc {
            width: 1280,
            height: 1040,
            ..desc
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
        let mut stats = renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("read back");
        let n = 600;
        let start = std::time::Instant::now();
        for _ in 0..n {
            stats = renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("read back");
        let ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
        best[usize::from(glow)] = best[usize::from(glow)].min(ms);
        println!(
            "glow {glow}: {} glyph instances, {ms:.4} ms per frame",
            stats.overlay_glyphs
        );
    }
    println!(
        "best of three: {:.4} ms without, {:.4} ms with the glow ({:+.4} ms)",
        best[0],
        best[1],
        best[1] - best[0]
    );
}
