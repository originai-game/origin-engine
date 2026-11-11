//! SVG text in documents (docs/ui.md §3.3): a bar chart's axis labels
//! anchored at their ticks, a gauge's value centred on its ring by
//! `dominant-baseline="central"`, a styled `<tspan>`, and a 24-unit badge
//! glyph drawn ten times its size (the scale the GPU takes on the text).
//! The frames match `references/vulkan/svg-text-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: labels at their unscaled place or size, a start
//! anchor where an end one was asked, a ring's value off its centre, or
//! glyphs blurred by the scale.

use crate::support;

use og_ui::*;
use support::*;

const CARD: &str = r##"<meta name="component" content="defs">
<column id="Center">
<panel id="Card">
  <h2 id="Title">SVG text</h2>
  <row id="Row">
    <svg id="Chart" viewBox="0 0 120 80">
      <g fill="#58a6ff">
        <rect x="14" y="40" width="16" height="30"/>
        <rect x="44" y="20" width="16" height="50"/>
        <rect x="74" y="50" width="16" height="20"/>
      </g>
      <line x1="8" y1="70" x2="112" y2="70" stroke="#8b949e" stroke-width="1"/>
      <g font-size="7" fill="#c9d1d9" text-anchor="middle">
        <text x="22" y="78">Mon</text>
        <text x="52" y="78">Tue</text>
        <text x="82" y="78">Wed</text>
      </g>
      <text x="112" y="10" text-anchor="end" font-size="8" fill="#c9d1d9">Peak <tspan font-weight="bold" fill="#f0b429">50</tspan></text>
    </svg>
    <svg id="Gauge" viewBox="0 0 40 40">
      <circle cx="20" cy="20" r="16" fill="none" stroke="#30363d" stroke-width="4"/>
      <circle cx="20" cy="20" r="16" fill="none" stroke="#7ee787" stroke-width="4" pathLength="1" stroke-dasharray="0.72 1"/>
      <text x="20" y="20" text-anchor="middle" dominant-baseline="central" font-size="10" fill="#e6edf3">72%</text>
    </svg>
    <svg id="Badge" viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="11" fill="#f85149"/>
      <text x="12" y="12" text-anchor="middle" dominant-baseline="central" font-size="14" font-weight="bold" fill="#fff">A</text>
    </svg>
  </row>
</panel>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; gap: 16px; }
#Title { margin: 0; }
#Row { gap: 24px; align-items: center; }
#Chart { width: 360px; height: 240px; }
#Gauge { width: 160px; height: 160px; }
#Badge { width: 240px; height: 240px; }
@media (max-width: 500px) {
  #Row { flex-direction: column; gap: 12px; }
  #Chart { width: 300px; height: 200px; }
  #Gauge { width: 120px; height: 120px; }
  #Badge { width: 240px; height: 240px; }
}
</style>"##;

/// `source` mounted and rendered at `width`×`height`: its pixels and the
/// renderer's info.
fn render(source: &str, width: u32, height: u32) -> (Vec<u8>, og_render::RenderInfo) {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/defs.svelte", source).unwrap());
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
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background =
        Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
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
fn svg_text_matches_its_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (pixels, info) = render(CARD, width, height);
        let name = format!("svg-text-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(og_tests::compare::RECORDING_TIER)
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
