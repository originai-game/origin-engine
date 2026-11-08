//! CSS paint in documents (docs/ui.md, CSS paint): gradient backgrounds
//! (linear, radial, conic, repeating), `mask-image`, `clip-path` shapes,
//! `text-shadow`, `filter`, `mix-blend-mode` and `backdrop-filter`.
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
  <p id="Glow">Shadowed text</p>
</div>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; display: flex; flex-direction: column; gap: 16px;
  background: linear-gradient(160deg, #1f2a44, #0d1117 70%); border-radius: 16px; }
#Title { margin: 0; }
#Row, #Row2 { display: flex; flex-direction: row; gap: 16px; }
#Sunset, #Orb, #Dial, #Stripes, #Gem, #Tag, #Fade, #Grey { width: 96px; height: 64px; }
#Sunset { background: linear-gradient(to top right, #ff7b72, #f0b429 50%, #58a6ff); border-radius: 8px; }
#Orb { background: radial-gradient(circle at 30% 30%, #ffffff, #7ee787 40%, #0d1117); border-radius: 32px; }
#Dial { background: conic-gradient(from 90deg, #f85149, #f0b429, #7ee787, #58a6ff, #f85149); border-radius: 32px; }
#Stripes { background: repeating-linear-gradient(45deg, #58a6ff 0px, #58a6ff 6px, #0000 6px, #0000 12px); }
#Gem { background: #d2a8ff; clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%); }
#Tag { background: #f0b429; clip-path: path(evenodd, "M8 8 L88 8 L88 56 L8 56 Z M36 20 L60 20 L60 44 L36 44 Z"); }
#Fade { background: #7ee787; mask-image: linear-gradient(to right, #000, transparent); }
#Grey { background: linear-gradient(#ff7b72, #58a6ff); filter: grayscale(1); }
#Glow { margin: 0; color: #ffffff; text-shadow: 0 2px 6px #58a6ff; }
@media (max-width: 500px) {
  #Row, #Row2 { gap: 8px; }
  #Sunset, #Orb, #Dial, #Stripes, #Gem, #Tag, #Fade, #Grey { width: 72px; height: 56px; }
}
</style>"##;

fn render(width: u32, height: u32) -> (Vec<u8>, og_render::RenderInfo) {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
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
