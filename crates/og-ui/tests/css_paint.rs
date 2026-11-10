//! CSS paint in documents (docs/ui.md, CSS paint): gradient backgrounds
//! (linear, radial, conic, repeating), `mask-image`, `clip-path` shapes,
//! `text-shadow`, `filter` chains (a drop shadow that follows a clipped
//! star's alpha, colour functions in order, opacity and blur), and
//! `backdrop-filter` colour functions over stripes, and image layers
//! (`url()` of atlas icons) sized, placed and repeated: 16 px tiles cut
//! by a radius, a cover photo under a badge 6 px from its corner, a
//! contained icon over a gradient, and tiles spaced edge to edge.
//! The frames match `references/metal/css-paint-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a gradient's angle read in maths' convention or not
//! lengthened to the corners of a wide box, a clip's percentages read
//! against the wrong box (or the clip drawn uncut), a mask that fades the
//! wrong way, or a retained element whose resolved paint rebuilds every
//! frame (idle must build nothing).

use crate::support;

use og_ui::*;
use support::*;

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
#Tiles { background: url(coin) 0 0 / 16px 16px, #1f6feb; border-radius: 16px; }
#Photo { background: url(coin) right 6px bottom 6px / 24px no-repeat, url("sky") center / cover no-repeat;
  border-radius: 32px; }
#Fit { background: url(coin) center / contain no-repeat, linear-gradient(#30363d, #0d1117); }
#Spaced { background: url(coin), #238636; background-size: 20px; background-repeat: space; }
#Glow { margin: 0; color: #ffffff; text-shadow: 0 2px 6px #58a6ff; }
@media (max-width: 500px) {
  #Row, #Row2, #Row3, #Row4 { gap: 8px; }
  #Sunset, #Orb, #Dial, #Stripes, #Gem, #Tag, #Fade, #Grey, #Star, #Chain, #Negative, #Ghost,
  #Tiles, #Photo, #Fit, #Spaced { width: 72px; height: 56px; }
}
</style>"##;

/// A 2×1 atlas of 64 px cells: `coin`, a gold disc with a dark rim on
/// clear, and `sky`, blue over orange with a white sun up and to the
/// left (an image whose crop shows where cover centres it).
fn atlas() -> og_render::TextureData {
    let (w, h) = (128u32, 64u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (cx, fy) = ((x % 64) as f32 + 0.5, y as f32 + 0.5);
            let px = if x < 64 {
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

fn render(width: u32, height: u32) -> (Vec<u8>, og_render::RenderInfo) {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.set_icons(IconAtlas {
        texture: Some(scene.add_texture(atlas())),
        columns: 2,
        rows: 1,
        names: vec!["coin".into(), "sky".into()],
        ..IconAtlas::default()
    });
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
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    (pixels, renderer.info().clone())
}

#[test]
#[ignore = "requires GPU"]
fn css_paint_matches_its_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (pixels, info) = render(width, height);
        let name = format!("css-paint-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references/metal")
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&pixels, expected.as_raw(), &info, &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
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
    scene.overlay = Some(overlay);
    scene.post.tonemap = Tonemap::None;
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
