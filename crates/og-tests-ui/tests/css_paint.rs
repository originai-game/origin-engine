//! CSS painting (images, gradients, masks and border images): the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/css_paint.rs.

use crate::support;
use og_ui::prelude::*;
use og_ui::*;
use support::*;

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
fn masks_hide_invisible_boxes_and_fade_image_content() {
    use og_render::*;
    struct Env(TextureHandle);
    impl bind::Env for Env {
        fn texture(&self, _: f64) -> Option<TextureHandle> {
            Some(self.0)
        }
    }
    let mut scene = RenderScene::new();
    let env = Env(scene.add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: [255, 0, 0, 255].repeat(4),
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    }));
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let text = r##"<meta name="component" content="masks"/>
<div id="Root"><div id="Hidden"></div><div id="Visible"></div><img id="Image" src={1} alt=""/><div id="Parent"><div id="Child"></div></div><div id="Map" class="rpg-minimap"><Minimap texture={1} size={128} u0={0} v0={0} u1={1} v1={1} markers={$markers} label=""/></div></div>
<style>
#Root { width: 128px; height: 640px; background: white; }
#Hidden, #Visible, #Image, #Parent { position: absolute; left: 0; width: 128px; height: 128px; mask-image: linear-gradient(90deg, #0000, #000); }
#Hidden { top: 0; background: black; visibility: hidden; }
#Visible { top: 128px; background: black; mask-image: linear-gradient(90deg, #000 0, #000 calc(100% - 36px), #0000 100%); }
#Image { top: 256px; opacity: 0.5; object-fit: cover; }
#Parent { top: 384px; background: red; visibility: hidden; }
#Child { width: 128px; height: 128px; background: blue; visibility: visible; }
.rpg-minimap { position: absolute; top: 512px; width: 128px; height: 128px; }
</style>"##;
    ui.define_document(
        UiDocument::parse(
            "ui-kits/core/minimap.svelte",
            include_str!("../../../packages/engine/src/ui/components/minimap.svelte"),
        )
        .unwrap(),
    );
    ui.define_document(UiDocument::parse("ui-kits/test/masks.svelte", text).unwrap());
    let pack = style_pack::StylePack::parse("styles/test", vec![("styles/test/main.css".into(), ".rpg-minimap #Frame { border-radius: 50%; mask-image: radial-gradient(circle, #000 69%, #0000 70.5%); }".into())]).unwrap();
    ui.set_style_pack(&pack).unwrap();
    let markers = reactive::SourceId::new("test", "markers");
    let reply = reactive::wire::decode(
        &ui.apply_reactive(
            &reactive::wire::encode(&serde_json::json!([
                {"op": "collection", "source": markers, "key": "id", "items": []}
            ]))
            .unwrap(),
        ),
    )
    .unwrap();
    assert_eq!(reply[0]["ok"], true);
    ui.mount_document(
        reactive::SourceId::new("test", "masks"),
        "masks",
        [("markers".into(), markers)].into(),
        Default::default(),
        &env,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(128.0, 640.0));
    ui.build_retained_documents(&env).unwrap();
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut overlay = Overlay::new(128.0, 640.0, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 640,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-mask-visibility-image.png"),
        &pixels,
        128,
        640,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let rgb = |x: usize, y: usize| &pixels[(y * 128 + x) * 4..][..3];
    let hidden = [rgb(4, 64), rgb(64, 64), rgb(124, 64)];
    let div = [rgb(4, 192), rgb(124, 192)];
    let img = [rgb(4, 320), rgb(124, 320)];
    let child = [rgb(4, 448), rgb(124, 448)];
    let minimap = [rgb(4, 516), rgb(64, 576), rgb(124, 636)];
    assert!(
        minimap[0][1] > 248 && minimap[1][1] < 8 && minimap[2][1] > 248,
        "round minimap: {minimap:?}"
    );
    assert!(
        hidden.iter().all(|c| c.iter().all(|v| *v > 248))
            && div[0][1] < 8
            && div[1][1] > 235
            && rgb(90, 192)[1] < 8
            && (180..=200).contains(&rgb(110, 192)[1])
            && img[0][1] > 235
            && img[1][1] < 205
            && child[0][0] > 235
            && child[1][0] < 90
            && child[1][2] > 248,
        "hidden={hidden:?}; masked div={div:?}; masked img={img:?}; visible child={child:?}"
    );
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
    let mut d = crate::support::Doc::with_font(text, &[], true, add_test_font(&mut scene));
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
