//! CSS effects in documents (docs/ui.md, CSS paint): group blending and
//! `isolation`. The frames match `references/metal/css-effects-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a blend applied to each shape of an element rather
//! than to its group (two overlapping children of a blended element blend
//! with each other), an isolated group whose children blend with what is
//! under the group, a blend over a group's transparent pixels that
//! darkens them (multiply's fixed-function blend reads them as black), or
//! a blend inside a filtered layer dropped.

use crate::support;

use og_ui::*;
use support::*;

const CARD: &str = r##"<meta name="component" content="effects">
<column id="Center">
<div id="Card">
  <h2 id="Title">CSS effects</h2>
  <div id="Blends">
    <div id="Loose" class="well"><div class="dot a"></div><div class="dot b"></div></div>
    <div id="Isolated" class="well"><div id="Iso"><div class="dot a"></div><div class="dot b"></div></div></div>
    <div id="Grouped" class="well"><div id="Pair"><div class="dot a"></div><div class="dot b"></div></div></div>
    <div id="Filtered" class="well"><div id="Sepia"><div class="dot a"></div><div class="dot b"></div></div></div>
  </div>
</div>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; display: flex; flex-direction: column; gap: 16px;
  background: linear-gradient(160deg, #1f2a44, #0d1117 70%); border-radius: 16px; }
#Title { margin: 0; }
#Blends { display: flex; flex-direction: row; gap: 16px; }
.well { width: 112px; height: 80px; border-radius: 8px;
  background: linear-gradient(to right, #f0b429, #58a6ff); }
.dot { position: absolute; top: 12px; width: 56px; height: 56px; border-radius: 28px; }
.a { inset-inline-start: 12px; background: #ff7b72; }
.b { inset-inline-start: 44px; background: #7ee787; }
#Loose .dot { mix-blend-mode: multiply; }
#Iso { position: absolute; top: 0; inset-inline-start: 0; width: 112px; height: 80px; isolation: isolate; }
#Iso .dot { mix-blend-mode: multiply; }
#Pair { position: absolute; top: 0; inset-inline-start: 0; width: 112px; height: 80px; mix-blend-mode: difference; }
#Sepia { position: absolute; top: 0; inset-inline-start: 0; width: 112px; height: 80px; filter: sepia(1); }
#Sepia .dot { mix-blend-mode: screen; }
@media (max-width: 500px) {
  #Blends { gap: 8px; }
  .well, #Iso, #Pair, #Sepia { width: 76px; height: 64px; }
  .dot { width: 40px; height: 40px; border-radius: 20px; }
  .b { inset-inline-start: 28px; }
}
</style>"##;

fn render(width: u32, height: u32) -> (Vec<u8>, og_render::RenderInfo) {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/effects.svelte", CARD).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "effects"),
        "effects",
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
fn css_effects_match_their_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (pixels, info) = render(width, height);
        let name = format!("css-effects-{width}.png");
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
