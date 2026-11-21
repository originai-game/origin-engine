//! CSS painting (images, gradients, masks and border images): the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/css_paint.rs.

use crate::support;
use og_ui::*;
use support::*;

/// An intervening overflow box does not clip an absolute descendant
/// whose containing block is outside it. A real containing block and a
/// clip-path still cut that descendant, including after retained reuse.
#[test]
#[ignore = "requires GPU"]
fn absolute_descendants_escape_only_ineligible_overflow_clips() {
    use og_render::*;
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(og_core::glam::Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    for (list, row, visible) in [
        ("overflow: hidden auto;", "", true),
        ("overflow: hidden; border-radius: 16px;", "", true),
        ("overflow: hidden auto;", "opacity: 0.5;", true),
        ("overflow: hidden auto;", "clip-path: inset(0);", false),
        ("overflow: hidden auto; position: relative;", "", false),
        ("overflow: hidden auto; clip-path: inset(0);", "", false),
    ] {
        let text = format!(
            r#"<meta name="component" content="t">
<div id="Body"><ul id="List"><li id="Row"><p id="Plate" style:left={{$left}}>Side plate</p><div id="Flow"></div></li></ul></div>
<style>
#Body {{ position: relative; display: grid; grid-template-columns: 1fr 1fr; width: 400px; height: 300px; background: black; }}
#List {{ min-width: 0; margin: 0; padding: 0; list-style: none; {list} }}
#Row {{ {row} }}
#Plate {{ position: absolute; top: 0; width: 80px; height: 40px; margin: 0; background: #ff00ff; }}
#Flow {{ width: 350px; height: 80px; background: lime; }}
</style>"#
        );
        let mut d = Doc::new(&text, &[("left", serde_json::json!("220px"))]);
        for frame in 0..4 {
            if frame == 2 {
                d.set("left", serde_json::json!("250px"));
            } else if frame > 0 {
                d.step(InputFrame::default());
            }
            let mut overlay = Overlay::new(400.0, 300.0, 1.0);
            d.ui.frame().emit(&EmitDesc::default(), &mut overlay);
            scene.ui.overlay = Some(overlay);
            let image = render_frame(&scene, &sized(400, 300)).image;
            let x = if frame < 2 { 240 } else { 270 };
            let pixel = &image.pixels[(20 * 400 + x) * 4..][..3];
            assert_eq!(
                pixel[0] > pixel[1].saturating_add(40) && pixel[2] > pixel[1].saturating_add(40),
                visible,
                "{list} {row}, frame={frame}: {pixel:?}"
            );
            let flow = &image.pixels[(60 * 400 + 240) * 4..][..3];
            assert_eq!(
                flow,
                &[0, 0, 0],
                "flow must stay clipped: {list} {row}, frame={frame}"
            );
        }
    }
}

/// A text-clipped background must clear the rest of a wide block, not
/// just the rectangle occupied by its glyphs.
#[test]
#[ignore = "requires GPU"]
fn text_clipped_backgrounds_leave_no_ink_outside_the_glyphs() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    scene.lighting.environment.background = Background::Color(og_core::glam::Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let draw = |scene: &mut RenderScene, css: &str| {
        let text = format!(
            r#"<meta name="component" content="clip"><div><p>1-1</p></div>
<style>
div {{ width: 320px; height: 120px; padding: 20px; box-sizing: border-box; }}
p {{ margin: 0; width: 280px; font-size: 52px; line-height: 1; {css} }}
</style>"#
        );
        let d = Doc::with_font(&text, &[], font.clone());
        let mut overlay = Overlay::new(320.0, 120.0, 1.0);
        d.ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        render_frame(scene, &sized(320, 120)).image
    };
    for transform in ["", "transform: translate(7px, 3px) rotate(4deg);"] {
        let mask = draw(&mut scene, &format!("color: white; {transform}"));
        let clipped = draw(
            &mut scene,
            &format!(
                "color: transparent; background: linear-gradient(90deg, red, blue); background-clip: text; {transform}",
            ),
        );
        let mut visible = 0;
        for (i, (mask, ink)) in mask
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(clipped.pixels.as_chunks::<4>().0)
            .enumerate()
        {
            if mask[..3].iter().all(|&v| v == 0) {
                assert!(
                    ink[..3].iter().all(|&v| v <= 2),
                    "background escaped the glyphs at {},{} ({transform}): {ink:?}",
                    i % 320,
                    i / 320,
                );
            } else if ink[..3].iter().any(|&v| v > 30) {
                visible += 1;
            }
        }
        assert!(visible > 100, "the gradient still paints the glyphs");
    }
}

/// A CSS shadow is the Gaussian convolution of the ink, including pixels
/// outside the atlas quads. Wide shadows must not become solid glyph boxes.
#[test]
#[ignore = "requires GPU"]
fn text_shadow_blurs_the_ink_beyond_the_glyph_quads() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    scene.lighting.environment.background = Background::Color(og_core::glam::Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let draw = |scene: &mut RenderScene, shadow: Option<f32>, colored: bool| {
        let mut ui = ui_with_font(font.clone());
        let ink = if shadow.is_some() { "black" } else { "white" };
        let ground = if colored { "#204060" } else { "black" };
        let shadow = shadow.map_or("none".into(), |r| {
            if colored {
                format!("7px -3px {r}px rgb(204 128 51 / 60%)")
            } else {
                format!("0 0 {r}px white")
            }
        });
        ui.define_document(UiDocument::parse("shadow.svelte", &format!(
            "<meta name=\"component\" content=\"shadow\"><div><p>1-1</p></div><style>div {{ position: fixed; inset: 0; background: {ground}; }} p {{ position: fixed; left: 100px; top: 85px; margin: 0; font-size: 52px; font-weight: 400; line-height: 1; color: {ink}; text-shadow: {shadow}; }}</style>"
        )).unwrap());
        ui.mount_document(
            reactive::SourceId::new("test", "shadow"),
            "shadow",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        ui.begin(InputFrame::default(), Viewport::new(320.0, 240.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let mut overlay = Overlay::new(320.0, 240.0, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        render_frame(scene, &sized(320, 240)).image
    };
    let linear = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let mask = draw(&mut scene, None, false);
    let ink: Vec<_> = mask
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, p)| p[0] > 0)
        .map(|(i, p)| ((i % 320) as f64, (i / 320) as f64, linear(p[0])))
        .collect();
    assert!(!ink.is_empty());
    let top = ink.iter().map(|p| p.1 as u32).min().unwrap();
    for (radius, colored) in [(8.0, false), (22.0, false), (48.0, false), (22.0, true)] {
        let image = draw(&mut scene, Some(radius), colored);
        let sigma = f64::from(radius) / 2.0;
        let mut checked = 0;
        // Only outside the ink: the black foreground cannot cover these
        // samples. Derive expectations from a separately rendered mask,
        // not the implementation's atlas range or filter parameters.
        for dy in [4, 8, 16, 24] {
            let y = top - dy;
            for x in (100..185).step_by(7) {
                let coverage: f64 = ink
                    .iter()
                    .map(|&(ix, iy, a)| {
                        let (dx, dy) = if colored { (7.0, -3.0) } else { (0.0, 0.0) };
                        let d2 =
                            (ix + dx - f64::from(x)).powi(2) + (iy + dy - f64::from(y)).powi(2);
                        a * (-d2 / (2.0 * sigma * sigma)).exp()
                            / (2.0 * std::f64::consts::PI * sigma * sigma)
                    })
                    .sum();
                if coverage < 0.004 {
                    continue;
                }
                // CSS source-over of white over black has encoded output
                // equal to coverage. Linear-light compositing would encode
                // that coverage again and make the halo much brighter.
                for channel in 0..3 {
                    let expected = if colored {
                        let ground = [32.0, 64.0, 96.0][channel] / 255.0;
                        let source = [204.0, 128.0, 51.0][channel] / 255.0;
                        ground + (source - ground) * coverage * 0.6
                    } else {
                        coverage
                    };
                    let actual =
                        f64::from(image.pixels[((y * 320 + x) * 4) as usize + channel]) / 255.0;
                    assert!(
                        (actual - expected).abs() < 0.008 + coverage * 0.12,
                        "radius {radius}, colored {colored}, ({x},{y}) channel {channel}: {actual}, Gaussian source-over {expected}"
                    );
                }
                checked += 1;
            }
        }
        assert!(checked > 5, "radius {radius}: no meaningful halo samples");
        // Shadows belong below opaque foreground ink, not over it.
        let opaque = mask
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .position(|p| p[0] == 255)
            .unwrap();
        assert!(
            image.pixels[opaque * 4..opaque * 4 + 3]
                .iter()
                .all(|c| *c <= 1)
        );
    }
}

#[test]
#[ignore = "requires GPU"]
fn image_sets_render_density_correct_backgrounds_masks_and_borders() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    for (name, size, rgba) in [
        ("low.png", 8, [255, 0, 0, 255]),
        ("high.png", 16, [0, 0, 255, 255]),
    ] {
        let texture = scene.add_texture(TextureData {
            width: size,
            height: size,
            rgba: rgba.repeat((size * size) as usize),
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        ui.define_texture(&format!("ui/{name}"), texture);
    }
    ui.define_document(UiDocument::parse("ui/density.svelte", r#"
<meta name="component" content="density" />
<div id="row"><div id="auto"></div><div id="type"></div><div id="zero"></div><div id="mask"></div><div id="border"></div><div id="gradient"></div><button></button></div>
<style>
#row { display: flex; }
#row > div { width: 32px; height: 32px; flex-shrink: 0; }
#auto { background: image-set("low.png" 1x, "high.png" calc(192dpcm / 2.54)) no-repeat; }
#type { background: image-set("absent.avif" type("image/avif"), "absent.png" type("image/png; bad"), "high.png" type('image/png; note="a;bÿcalc(bad)"'), "low.png" 1x) no-repeat; }
#zero { background: lime; mask-image: image-set("absent.png" 0x); }
#mask { background: lime; mask: image-set("low.png" 1x, "high.png" 2x) no-repeat; }
#border { border: 1px solid transparent; border-image-source: image-set("high.png" 2x); border-image-slice: 25%; border-image-width: auto; }
#gradient { background: image-set(linear-gradient(red, red) 1x, linear-gradient(blue, blue) 2x); }
button { width: 32px; height: 32px; flex-shrink: 0; }
button::part(base) { background: image-set("low.png" min(1x, 2dppx), "high.png" max(96dpi, 2x)) no-repeat; border: 0; border-radius: 0; padding: 0; box-shadow: none; }
</style>"#).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "density"),
        "density",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    scene.lighting.environment.background = Background::Color(og_core::glam::Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    for dpr in [1u32, 2, 1] {
        let (width, height) = (224 * dpr, 32 * dpr);
        let mut viewport = Viewport::new(224.0, 32.0);
        viewport.dpr = dpr as f32;
        ui.begin(InputFrame::default(), viewport);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let mut overlay = Overlay::new(width as f32, height as f32, dpr as f32);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .unwrap();
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        let pixels = renderer.read_offscreen().unwrap().pixels;
        image::save_buffer(
            out_dir().join(format!("css-image-set-{dpr}.png")),
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let check = |x: u32, y: u32, expected: [u8; 3]| {
            let at = ((y * dpr * width + x * dpr) * 4) as usize;
            let actual = &pixels[at..at + 3];
            assert!(
                actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) < 8),
                "DPR {dpr}, ({x},{y}): {actual:?}, expected {expected:?}"
            );
        };
        let chosen = if dpr == 1 { [255, 0, 0] } else { [0, 0, 255] };
        check(3, 3, chosen);
        check(10, 3, [0; 3]); // Both 8px@1x and 16px@2x occupy 8 CSS px.
        check(42, 3, [0, 0, 255]); // Type filtering precedes first-density wins.
        check(50, 3, [0; 3]);
        check(67, 3, [0; 3]); // A zero-resolution mask is transparent, not absent.
        check(99, 3, [0, 255, 0]);
        check(106, 3, [0; 3]);
        check(140, 1, [0, 0, 255]); // Four source texels make a 2 CSS px auto border.
        check(140, 3, [0; 3]);
        check(180, 16, chosen); // Gradients use the box, not raster density sizing.
        check(195, 3, chosen);
        check(202, 3, [0; 3]); // Persistent widget parts use the same density-aware painter.
    }
}

const CARD: &str = r##"<meta name="component" content="paint">
<column id="Center">
<div id="Card">
  <h2 id="Title">CSS paint</h2>
  <div id="Row">
    <div id="Sunset"></div>
    <div id="Orb"></div>
    <div id="Dial"></div>
    <div id="Stripes"></div>
  </div>
  <div id="Row2">
    <div id="Gem"></div>
    <div id="Tag"></div>
    <div id="Fade"></div>
    <div id="Grey"></div>
  </div>
  <div id="Row3">
    <div id="Lift"><div id="Star"></div></div>
    <div id="Chain"></div>
    <div id="Negative"></div>
    <div id="Ghost"></div>
  </div>
  <div id="Pane"><div id="Lens"></div></div>
  <div id="Row4">
    <div id="Tiles"></div>
    <div id="Photo"></div>
    <div id="Fit"></div>
    <div id="Spaced"></div>
  </div>
  <p id="Glow">Shadowed text</p>
</div>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; display: flex; flex-direction: column; gap: 16px;
  background: linear-gradient(160deg, #1f2a44, #0d1117 70%); border-radius: 16px; }
#Title { margin: 0; }
#Row, #Row2, #Row3, #Row4 { display: flex; flex-direction: row; gap: 16px; }
#Sunset, #Orb, #Dial, #Stripes, #Gem, #Tag, #Fade, #Grey, #Star, #Chain, #Negative, #Ghost,
#Tiles, #Photo, #Fit, #Spaced { width: 96px; height: 64px; }
#Sunset { background: linear-gradient(to top right, #ff7b72, #f0b429 50%, #58a6ff); border-radius: 8px; }
#Orb { background: radial-gradient(circle at 30% 30%, #ffffff, #7ee787 40%, #0d1117); border-radius: 32px; }
#Dial { background: conic-gradient(from 90deg, #f85149, #f0b429, #7ee787, #58a6ff, #f85149); border-radius: 32px; }
#Stripes { background: repeating-linear-gradient(45deg, #58a6ff 0px, #58a6ff 6px, #0000 6px, #0000 12px); }
#Gem { background: #d2a8ff; clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%); }
#Tag { background: #f0b429; clip-path: path(evenodd, "M8 8 L88 8 L88 56 L8 56 Z M36 20 L60 20 L60 44 L36 44 Z"); }
#Fade { background: #7ee787; mask-image: linear-gradient(to right, #000, transparent); }
#Grey { background: linear-gradient(#ff7b72, #58a6ff); filter: grayscale(1); }
#Lift { filter: drop-shadow(4px 6px 3px #000000cc) saturate(1.6); }
#Star { background: linear-gradient(#f0b429, #ff7b72);
  clip-path: polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%); }
#Chain { background: conic-gradient(#f85149, #f0b429, #7ee787, #58a6ff, #f85149); border-radius: 8px;
  filter: sepia(0.8) contrast(1.4) hue-rotate(180deg); }
#Negative { background: linear-gradient(to right, #ffffff, #58a6ff); filter: invert(1); }
#Ghost { background: #7ee787; border-radius: 8px; filter: opacity(40%) blur(2px); }
#Pane { width: 100%; height: 40px; display: flex; flex-direction: row;
  background: repeating-linear-gradient(90deg, #f0b429 0px, #f0b429 10px, #1f6feb 10px, #1f6feb 20px); }
#Lens { width: 50%; height: 100%; backdrop-filter: invert(1) grayscale(0.5); }
#Tiles { background: url(coin.png) 0 0 / 16px 16px, #1f6feb; border-radius: 16px; }
#Photo { background: url(coin.png) right 6px bottom 6px / 24px no-repeat, url("sky.png") center / cover no-repeat;
  border-radius: 32px; }
#Fit { background: url(coin.png) center / contain no-repeat, linear-gradient(#30363d, #0d1117); }
#Spaced { background: url(coin.png), #238636; background-size: 20px; background-repeat: space; }
#Glow { margin: 0; color: #ffffff; text-shadow: 0 2px 6px #58a6ff; }
@media (max-width: 500px) {
  #Row, #Row2, #Row3, #Row4 { gap: 8px; }
  #Sunset, #Orb, #Dial, #Stripes, #Gem, #Tag, #Fade, #Grey, #Star, #Chain, #Negative, #Ghost,
  #Tiles, #Photo, #Fit, #Spaced { width: 72px; height: 56px; }
}
</style>"##;

/// Independent 64 px images: a gold disc with a dark rim on
/// clear, or blue over orange with a white sun up and to the
/// left (an image whose crop shows where cover centres it).
fn picture(coin: bool) -> og_render::TextureData {
    let (w, h) = (64u32, 64u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (cx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let px = if coin {
                let d = ((cx - 32.0).powi(2) + (fy - 32.0).powi(2)).sqrt();
                if d > 28.0 {
                    [0, 0, 0, 0]
                } else if d > 23.0 {
                    [120, 72, 10, 255]
                } else {
                    [240, 180, 40, 255]
                }
            } else {
                let t = fy / 64.0;
                let sun = ((cx - 20.0).powi(2) + (fy - 20.0).powi(2)).sqrt() < 9.0;
                if sun {
                    [255, 255, 240, 255]
                } else {
                    [
                        (40.0 + 200.0 * t) as u8,
                        (90.0 + 40.0 * t) as u8,
                        (200.0 - 150.0 * t) as u8,
                        255,
                    ]
                }
            };
            rgba.extend_from_slice(&px);
        }
    }
    og_render::TextureData {
        width: w,
        height: h,
        rgba,
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::Generate,
    }
}

fn render(width: u32, height: u32) -> Rendered {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    for (name, coin) in [("coin.png", true), ("sky.png", false)] {
        ui.define_texture(&format!("ui/{name}"), scene.add_texture(picture(coin)));
    }
    ui.define_document(UiDocument::parse("ui/paint.svelte", CARD).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "paint"),
        "paint",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let vp = Viewport::new(width as f32, height as f32);
    let mut stats = None;
    for _ in 0..4 {
        ui.begin(InputFrame::default(), vp);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        stats = Some(ui.end().stats);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let stats = stats.unwrap();
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    render_ui(&ui, &mut scene, width, height)
}

#[test]
#[ignore = "requires GPU"]
fn css_paint_matches_its_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let frame = render(width, height);
        let name = format!("css-paint-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}

#[test]
#[ignore = "requires GPU"]
fn repeating_gradients_repeat_the_stop_interval() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let text = r##"<meta name="component" content="repeat"/>
<div>
  <div id="radial"></div><div id="negative"></div><div id="conic"></div>
  <div id="collapsed"></div>
</div>
<style>
div { margin: 0; }
#radial, #negative, #conic, #collapsed { width: 128px; height: 128px; }
#radial { background: repeating-radial-gradient(circle at 0% 50%, black 8px, black 16px, white 16px, white 24px); }
#negative { background: repeating-radial-gradient(circle at 0% 50%, black -8px, black 0px, white 0px, white 8px); }
#conic { background: repeating-conic-gradient(black 0deg, black 45deg, white 45deg, white 90deg); }
#collapsed { background: repeating-linear-gradient(black 8px, white 8px, black 8px); }
</style>"##;
    ui.define_document(UiDocument::parse("ui/repeat.svelte", text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "repeat"),
        "repeat",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(128.0, 512.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let mut overlay = Overlay::new(128.0, 512.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 512,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-repeating-gradients.png"),
        &pixels,
        128,
        512,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let check = |x: usize, y: usize, white: bool| {
        let rgb = &pixels[(y * 128 + x) * 4..][..3];
        assert!(
            rgb.iter().all(|c| if white { *c > 248 } else { *c < 7 }),
            "({x}, {y}): expected {}, got {rgb:?}",
            if white { "white" } else { "black" }
        );
    };
    // The period is 16px, not the last stop's position; it extends backward too.
    for y in [64, 192] {
        for (x, white) in [(4, true), (12, false), (20, true), (28, false), (52, true)] {
            check(x, y, white);
        }
    }
    // Sample both halves of all four quarters, away from stop boundaries.
    for (x, y, white) in [
        (80, 24, false),
        (104, 48, true),
        (104, 80, false),
        (80, 104, true),
        (48, 104, false),
        (24, 80, true),
        (24, 48, false),
        (48, 24, true),
    ] {
        check(x, y + 256, white);
    }
    // Three equally spaced stops integrate to half white, not their 1/3 mean.
    for (x, y) in [(4, 400), (64, 448), (120, 500)] {
        let rgb = &pixels[(y * 128 + x) * 4..][..3];
        assert!(rgb.iter().all(|c| (126..=129).contains(c)), "{rgb:?}");
    }
}

#[test]
#[ignore = "requires GPU"]
fn nonrepeating_gradients_keep_stops_outside_the_image() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let text = r##"<meta name="component" content="outside"/>
<div>
  <div id="negative"></div><div id="beyond"></div>
  <div id="radial"></div><div id="conic"></div>
</div>
<style>
div { margin: 0; }
#negative, #beyond, #radial, #conic { width: 128px; height: 128px; }
#negative { background: linear-gradient(to right, black calc(0% - 128px), white 128px); }
#beyond { background: linear-gradient(to right, black 0px, white calc(100% + 128px)); }
#radial { background: radial-gradient(circle at 0% 50%, black -64px, white 64px); }
#conic { background: conic-gradient(black -1turn, white 1turn); }
</style>"##;
    ui.define_document(UiDocument::parse("ui/outside.svelte", text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "outside"),
        "outside",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(128.0, 512.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let mut overlay = Overlay::new(128.0, 512.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 512,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-outside-gradients.png"),
        &pixels,
        128,
        512,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let check = |x: usize, y: usize, fraction: f32| {
        // On the neutral OKLab axis L=t, linear luminance is t³. Encode
        // that independently as sRGB; neither stop clamping nor the
        // renderer's gradient evaluator contributes the expectation.
        let linear = fraction.clamp(0.0, 1.0).powi(3);
        let encoded = if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        let expected = encoded * 255.0;
        let rgb = &pixels[(y * 128 + x) * 4..][..3];
        assert!(
            rgb.iter().all(|c| (*c as f32 - expected).abs() < 3.0),
            "({x}, {y}): expected {expected:.1}, got {rgb:?}"
        );
    };
    for x in [8, 32, 80, 120] {
        let at = x as f32 + 0.5;
        check(x, 64, (at + 128.0) / 256.0);
        check(x, 192, at / 256.0);
        check(x, 320, (at.hypot(0.5) + 64.0) / 128.0);
    }
    for (x, y) in [(80, 24), (104, 80), (48, 104), (24, 48)] {
        let angle = (y as f32 + 0.5 - 64.0).atan2(x as f32 + 0.5 - 64.0);
        let turn = (angle + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::TAU;
        check(x, y + 384, (turn + 1.0) / 2.0);
    }
}

#[test]
#[ignore = "requires GPU"]
fn gradients_mix_in_the_colour_space_they_name() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let text = r##"<meta name="component" content="spaces"/>
<div>
  <div id="longer"></div><div id="shorter"></div><div id="srgb"></div>
</div>
<style>
div { margin: 0; }
#longer, #shorter, #srgb { width: 256px; height: 32px; }
#longer { background: linear-gradient(to right in hsl longer hue, red, red); }
#shorter { background: linear-gradient(to right in hsl, red, blue); }
#srgb { background: linear-gradient(to right in srgb, red, blue); }
</style>"##;
    ui.define_document(UiDocument::parse("ui/spaces.svelte", text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "spaces"),
        "spaces",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(256.0, 96.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let mut overlay = Overlay::new(256.0, 96.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 256,
        height: 96,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-gradient-spaces.png"),
        &pixels,
        256,
        96,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let check = |x: usize, y: usize, expected: [f32; 3]| {
        // Averaged over a 4 x 4 block: the dither's grain cancels.
        let mut sum = [0.0f32; 3];
        for dy in 0..4 {
            for dx in 0..4 {
                let at = ((y + dy) * 256 + x + dx) * 4;
                for k in 0..3 {
                    sum[k] += f32::from(pixels[at + k]) / 16.0;
                }
            }
        }
        // Within one just-noticeable difference (deltaEOK 0.02): sRGB codes
        // near 0 are too steep for a code tolerance.
        let lab = |c: [f32; 3]| {
            og_core::color::oklab(c.map(|v| og_core::color::srgb_to_linear(v / 255.0)))
        };
        let (a, b) = (lab(sum), lab(expected));
        let d = (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt();
        assert!(
            d < 0.02,
            "({x}, {y}): expected {expected:?}, got {sum:?} (deltaEOK {d})"
        );
    };
    // HSL at full saturation and half lightness, by CSS Color 4's formula,
    // at the hue of the block's centre (2 px in).
    let hsl = |hue: f32| {
        let f = |n: f32| {
            let k = (n + hue / 30.0).rem_euclid(12.0);
            (0.5 - 0.5 * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)) * 255.0
        };
        [f(0.0), f(8.0), f(4.0)]
    };
    let t = |x: usize| (x as f32 + 2.0) / 256.0;
    // A whole turn the long way round: yellow, green, cyan, blue, magenta.
    for x in [40, 84, 126, 168, 210] {
        check(x, 14, hsl(360.0 * t(x)));
    }
    // Red to blue the short way, through magenta (hue 360 down to 240).
    for x in [60, 126, 190] {
        check(x, 46, hsl(360.0 - 120.0 * t(x)));
    }
    // In sRGB, halfway is an even purple, not OKLab's lighter one.
    let k = t(126);
    let encode = |v: f32| v * 255.0;
    check(126, 78, [encode(1.0 - k), 0.0, encode(k)]);
}

/// `background-clip: border-area` (CSS Backgrounds 4 §2.1) paints a layer
/// only under the border: the border box less the padding box, whatever
/// the border's own colour. A colour clipped so leaves the padding box
/// empty, and an image layer clipped so draws over the colour only there.
///
/// Ways it could fail: the layer drawn over the whole box (the clip read
/// as border-box), the padding box cut from the layers under it too, or
/// the ring inset by the padding instead of the border.
#[test]
#[ignore = "requires GPU"]
fn border_area_paints_a_layer_only_under_the_border() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let text = r##"<meta name="component" content="area"/>
<div id="Row">
  <div id="Colour"></div><div id="Layer"></div>
</div>
<style>
#Row { display: flex; }
#Colour, #Layer { width: 32px; height: 32px; box-sizing: border-box; padding: 4px;
  border: 6px solid transparent; flex-shrink: 0; }
#Colour { background-color: #00ff00; background-clip: border-area; }
#Layer { background: linear-gradient(#ff0000, #ff0000) border-area, #0000ff; }
</style>"##;
    ui.define_document(UiDocument::parse("ui/area.svelte", text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "area"),
        "area",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(64.0, 32.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut overlay = Overlay::new(64.0, 32.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(og_core::glam::Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 64,
        height: 32,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-border-area.png"),
        &pixels,
        64,
        32,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let check = |x: usize, y: usize, expected: [u8; 3]| {
        let at = (y * 64 + x) * 4;
        let actual = &pixels[at..at + 3];
        assert!(
            actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) < 8),
            "({x},{y}): {actual:?}, expected {expected:?}"
        );
    };
    // Under the 6 px border, at each side, and inside the padding box
    // (its edge 6 px in, before the 4 px padding).
    for (x, y) in [(2, 16), (29, 16), (16, 2), (16, 29), (2, 2)] {
        check(x, y, [0, 255, 0]);
        check(x + 32, y, [255, 0, 0]);
    }
    for (x, y) in [(7, 16), (16, 16), (24, 7)] {
        check(x, y, [0, 0, 0]);
        check(x + 32, y, [0, 0, 255]);
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_gradient_leaving_the_gamut_keeps_its_lightness_and_hue() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    // Red to lime in OKLCh passes orange and yellow at a chroma sRGB cannot
    // show: CSS Color 4 §13.2 lowers the chroma at the pixel's lightness and
    // hue, where a per-channel clip turns both (a crease along the ramp).
    let text = r##"<meta name="component" content="wide"/>
<div id="wide"></div>
<style>
#wide { margin: 0; width: 256px; height: 16px; background: linear-gradient(to right in oklch, red, lime); }
</style>"##;
    ui.define_document(UiDocument::parse("ui/wide.svelte", text).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "wide"),
        "wide",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(256.0, 16.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let mut overlay = Overlay::new(256.0, 16.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 256,
        height: 16,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    // CSS's red and lime in OKLCh (lightness, chroma, hue in degrees).
    let (red, lime) = (
        [0.6279554, 0.2576833, 29.2338],
        [0.8664396, 0.2948272, 142.4953],
    );
    for x in (8..248).step_by(16) {
        // Averaged over a 4 x 4 block: the dither's grain cancels.
        let mut sum = [0.0f32; 3];
        for dy in 0..4 {
            for dx in 0..4 {
                let at = ((6 + dy) * 256 + x + dx) * 4;
                for k in 0..3 {
                    sum[k] += f32::from(pixels[at + k]) / (16.0 * 255.0);
                }
            }
        }
        let lab = og_core::color::oklab(sum.map(og_core::color::srgb_to_linear));
        let t = (x as f32 + 2.0) / 256.0;
        let [l, _, h] = [0, 1, 2].map(|k| red[k] + (lime[k] - red[k]) * t);
        // The drawn colour against the mix's lightness and hue at the drawn
        // chroma: within a just-noticeable difference (deltaEOK 0.02).
        let chroma = lab[1].hypot(lab[2]);
        let (sin, cos) = h.to_radians().sin_cos();
        let d = [lab[0] - l, lab[1] - chroma * cos, lab[2] - chroma * sin];
        let d = d.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            d < 0.02,
            "x {x}: expected lightness {l} hue {h}, got {lab:?} (deltaEOK {d})"
        );
    }
}

#[test]
#[ignore = "requires GPU"]
fn flex_ratios_absolute_fades_and_state_border_images_render() {
    use og_render::*;
    let mut scene = RenderScene::new();
    let text = r##"<meta name="component" content="t">
<div>
<div id="Page"><div class="box"></div><div class="box"></div><div id="Fade"></div></div>
<div id="Doll"><div id="Column"><div id="First"></div><div id="Second"></div></div></div>
<button id="Border"></button>
</div>
<style>
#Page { position: relative; display: flex; flex-direction: column; width: 240px; height: 160px; overflow: hidden; }
.box { flex: none; height: 200px; background: blue; }
#Fade { position: absolute; left: 0; right: 0; bottom: 0; height: 80px; background: linear-gradient(180deg, #ff000000, #ff0000); }
#Doll { display: flex; width: 240px; height: 168px; padding-left: 120px; }
#Column { display: flex; flex-direction: column; height: 100%; gap: 8px; align-items: center; }
#First, #Second { flex: 1 1 0; min-height: 0; aspect-ratio: 1; max-width: 100px; background: lime; }
#Second { aspect-ratio: 2; background: cyan; }
#Border { width: 240px; height: 64px; border: 12px solid transparent; border-image: linear-gradient(red, red) 12 / 12px; }
#Border:hover { border-image-source: linear-gradient(lime, lime); }
</style>"##;
    let mut d = crate::support::Doc::with_font(text, &[], add_test_font(&mut scene));
    d.viewport = Viewport::new(240.0, 392.0);
    d.step(InputFrame::default());
    d.step(InputFrame {
        pointer: og_core::glam::Vec2::new(120.0, 350.0),
        pointer_inside: true,
        ..Default::default()
    });
    let mut overlay = Overlay::new(240.0, 392.0, 1.0);
    d.ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 240,
        height: 392,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-rpg-layout.png"),
        &pixels,
        240,
        392,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let rgb = |x: usize, y: usize| &pixels[(y * 240 + x) * 4..][..3];
    assert!(
        rgb(120, 20)[2] > 248 && rgb(120, 158)[0] > 248,
        "absolute fade overlays overflowing children"
    );
    assert!(
        rgb(85, 200)[1] > 248 && rgb(75, 280)[2] > 248,
        "flex-derived ratio boxes paint their width"
    );
    assert!(
        rgb(120, 332)[1] > 248 && rgb(120, 332)[0] < 8,
        "hover longhand replaces shorthand source: {:?}",
        rgb(120, 332)
    );
}

/// `references/vulkan/accent-color-{dark,light}.png` (640 × 240): a row of the
/// built-in controls (a checked box, an empty one, a switch that is on, a
/// chosen radio, a range, a progress bar) under a gold accent, a navy one
/// and none, in each scheme. Ways it could fail: the accent reaching the
/// commands but not the frame, a check mark unreadable on its accent, or
/// an accent leaking into the unaccented row.
#[test]
#[ignore = "requires GPU"]
fn accent_color_controls_match_their_reference_frames() {
    use og_render::*;
    let row = |id: &str| {
        format!(
            r#"<div class="row" id="{id}"><span>{id}</span><input type="checkbox" checked /><input type="checkbox" /><input type="checkbox" switch checked /><input type="radio" checked /><input type="range" min="0" max="10" value="6" /><progress value="0.6"></progress></div>"#
        )
    };
    let text = format!(
        r#"<meta name="component" content="accent"/>
<column id="Page">{}{}{}</column>
<style>
#Page {{ background: var(--backdrop); padding: 12px; gap: 8px; width: stretch; height: stretch; }}
.row {{ display: flex; align-items: center; gap: 12px; }}
.row > span {{ width: 56px; }}
.row > input[type=range], .row > progress {{ width: 120px; }}
#Gold {{ accent-color: #ffd60a; }}
#Navy {{ accent-color: #1a237e; }}
</style>"#,
        row("Gold"),
        row("Navy"),
        row("None")
    );
    for (name, theme) in [("dark", Theme::dark()), ("light", Theme::light())] {
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        ui.set_theme(theme);
        ui.define_document(UiDocument::parse("ui/accent.svelte", &text).unwrap());
        ui.mount_document(
            reactive::SourceId::new("test", "accent"),
            "accent",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        for _ in 0..4 {
            ui.begin(InputFrame::default(), Viewport::new(640.0, 240.0));
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let frame = render_ui(&ui, &mut scene, 640, 240);
        matches_reference(
            &format!("accent-color-{name}"),
            &frame.image,
            &frame.info,
            Against::Recorded,
        );
    }
}

#[test]
fn a_p3_colour_reaches_the_renderer_on_a_p3_output() {
    use og_render::*;
    // A document's colours are resolved when it builds: on a Display P3
    // output a colour outside sRGB keeps its gamut through the UI's style
    // and layout, so og-render draws it (extended linear sRGB: a negative
    // channel), and on an sRGB output it is gamut mapped into sRGB.
    let solids = |wide: bool| {
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let text = r##"<meta name="component" content="p3"/>
<div id="root"><div id="a" style="background: color(display-p3 1 0 0)"></div><div id="b"></div></div>
<style>
#a, #b { width: 32px; height: 32px; }
#b { background: oklch(0.6 0.25 25); }
</style>"##;
        ui.define_document(UiDocument::parse("ui/p3.svelte", text).unwrap());
        ui.mount_document(
            reactive::SourceId::new("test", "p3"),
            "p3",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let mut viewport = Viewport::new(64.0, 64.0);
        viewport.wide_gamut = wide;
        ui.begin(InputFrame::default(), viewport);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let mut overlay = Overlay::new(64.0, 64.0, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        overlay
            .ui
            .shapes
            .iter()
            .filter_map(|s| match s.paint {
                UiPaint::Solid(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let p3 = solids(true);
    assert_eq!(p3.len(), 2, "{p3:?}");
    // Display P3's red, 1 0 0 in P3 (CSS Color 4 §10's matrix).
    let red = p3[0];
    assert!(
        (red[0] - 1.2249).abs() < 0.01
            && (red[1] + 0.0421).abs() < 0.01
            && (red[2] + 0.0196).abs() < 0.01,
        "{red:?}"
    );
    // The oklch red is outside sRGB too: a channel below zero.
    assert!(p3[1][..3].iter().any(|v| *v < -0.001), "{:?}", p3[1]);
    for c in solids(false) {
        assert!(c[..3].iter().all(|v| (0.0..=1.0).contains(v)), "{c:?}");
    }
}
