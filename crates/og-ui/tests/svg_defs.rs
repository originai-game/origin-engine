//! SVG definitions in documents (docs/ui.md §3.3): a coin `<symbol>`
//! drawn by three `<use>`s at three sizes, a gauge cut by a `<clipPath>`,
//! a card faded by a gradient `<mask>`, and a hatched `<pattern>` fill.
//! The frames match `references/metal/svg-defs-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a use drawn at the symbol's own size, a clip or a
//! mask placed in view-box units instead of the box, or tiles that stop
//! short of the shape's edge.

use crate::support;

use og_ui::*;
use support::*;

const CARD: &str = r##"<meta name="component" content="defs">
<column id="Center">
<panel id="Card">
  <h2 id="Title">SVG definitions</h2>
  <svg id="Art" viewBox="0 0 200 80">
    <title>Purse</title>
    <defs>
      <symbol id="Coin" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="11" fill="#f0b429"/>
        <circle cx="12" cy="12" r="7" fill="none" stroke="#b7791f" stroke-width="2"/>
      </symbol>
      <clipPath id="Half"><rect x="70" y="0" width="30" height="80"/></clipPath>
      <linearGradient id="Ramp"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#000"/></linearGradient>
      <mask id="Fade"><rect x="110" y="10" width="40" height="60" fill="url(#Ramp)"/></mask>
      <pattern id="Hatch" patternUnits="userSpaceOnUse" width="6" height="6" patternTransform="rotate(45)">
        <rect x="0" y="0" width="2" height="6" fill="#58a6ff"/>
      </pattern>
    </defs>
    <use href="#Coin" x="4" y="28" width="24" height="24"/>
    <use href="#Coin" x="30" y="20" width="40" height="40"/>
    <use href="#Coin" x="8" y="60" width="16" height="16"/>
    <g clip-path="url(#Half)"><circle cx="85" cy="40" r="30" fill="#f85149"/></g>
    <g mask="url(#Fade)"><rect x="110" y="10" width="40" height="60" rx="6" fill="#7ee787"/></g>
    <rect x="160" y="10" width="36" height="60" rx="4" fill="url(#Hatch)" stroke="#58a6ff"/>
  </svg>
</panel>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; gap: 16px; }
#Title { margin: 0; }
#Art { width: 480px; height: 192px; }
@media (max-width: 500px) { #Art { width: 320px; height: 128px; } }
</style>"##;

#[test]
#[ignore = "requires GPU"]
fn svg_defs_match_their_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/defs.svelte", CARD).unwrap());
        ui.mount_document(
            reactive::SourceId::new("test", "defs"),
            "defs",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..4 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.overlay = Some(overlay);
        let bg = ui.theme().palette.backdrop.linear();
        scene.environment.background =
            Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
        scene.post.tonemap = Tonemap::None;
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        let pixels = renderer.read_offscreen().unwrap().pixels;
        let info = renderer.info().clone();
        let name = format!("svg-defs-{width}.png");
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
