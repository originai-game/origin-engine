//! SVG definitions in documents (docs/ui.md §3.3): a coin `<symbol>`
//! drawn by three `<use>`s at three sizes, a gauge cut by a `<clipPath>`,
//! a card faded by a gradient `<mask>`, and a hatched `<pattern>` fill
//! cut to a diamond path.
//! The frames match `references/vulkan/svg-defs-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a use drawn at the symbol's own size, a clip or a
//! mask placed in view-box units instead of the box, or tiles that stop
//! short of the shape's edge, or a path clip that draws uncut.

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
      <clipPath id="Gem"><path d="M178 6 Q196 22 198 40 L178 74 L158 40 Q160 22 178 6 Z"/></clipPath>
    </defs>
    <use href="#Coin" x="4" y="28" width="24" height="24"/>
    <use href="#Coin" x="30" y="20" width="40" height="40"/>
    <use href="#Coin" x="8" y="60" width="16" height="16"/>
    <g clip-path="url(#Half)"><circle cx="85" cy="40" r="30" fill="#f85149"/></g>
    <g mask="url(#Fade)"><rect x="110" y="10" width="40" height="60" rx="6" fill="#7ee787"/></g>
    <g clip-path="url(#Gem)"><rect x="160" y="10" width="36" height="60" rx="4" fill="url(#Hatch)" stroke="#58a6ff"/></g>
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
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
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

/// A `<clipPath>` of a triangle path cuts a red square to the triangle:
/// red in its top left, the backdrop past the diagonal.
///
/// Fails when a path clip draws uncut (red everywhere) or cuts all of it.
#[test]
#[ignore = "requires GPU"]
fn a_path_clip_cuts_to_the_path() {
    const TRI: &str = r##"<meta name="component" content="defs">
<svg id="Art" viewBox="0 0 100 100">
  <defs><clipPath id="Tri"><path d="M0 0 L100 0 L0 100 Z"/></clipPath></defs>
  <g clip-path="url(#Tri)"><rect x="0" y="0" width="100" height="100" fill="#ff0000"/></g>
</svg>
<style>#Art { width: 200px; height: 200px; }</style>"##;
    let (pixels, _) = render(TRI, 200, 200);
    let at = |x: usize, y: usize| {
        let i = (y * 200 + x) * 4;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    let red = |p: [u8; 3]| p[0] > 200 && p[1] < 60 && p[2] < 60;
    assert!(red(at(40, 40)), "inside the triangle: {:?}", at(40, 40));
    assert!(!red(at(160, 160)), "past the diagonal: {:?}", at(160, 160));
    assert!(
        !red(at(104, 104)),
        "just past the diagonal: {:?}",
        at(104, 104)
    );
}

/// `clip-rule="evenodd"` on a ring of two same-way circles cuts its hole:
/// red on the ring, the backdrop at the centre.
///
/// Fails when the rule is read as nonzero (the centre red) or the ring
/// is cut away too.
#[test]
#[ignore = "requires GPU"]
fn an_evenodd_clip_cuts_a_ring() {
    const RING: &str = r##"<meta name="component" content="defs">
<svg id="Art" viewBox="0 0 100 100">
  <defs><clipPath id="Ring"><path clip-rule="evenodd" d="M10 50 A40 40 0 1 1 90 50 A40 40 0 1 1 10 50 Z M30 50 A20 20 0 1 1 70 50 A20 20 0 1 1 30 50 Z"/></clipPath></defs>
  <g clip-path="url(#Ring)"><rect x="0" y="0" width="100" height="100" fill="#ff0000"/></g>
</svg>
<style>#Art { width: 200px; height: 200px; }</style>"##;
    let (pixels, _) = render(RING, 200, 200);
    let at = |x: usize, y: usize| {
        let i = (y * 200 + x) * 4;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    let red = |p: [u8; 3]| p[0] > 200 && p[1] < 60 && p[2] < 60;
    assert!(red(at(40, 100)), "on the ring: {:?}", at(40, 100));
    assert!(!red(at(100, 100)), "in the hole: {:?}", at(100, 100));
    assert!(!red(at(4, 4)), "outside: {:?}", at(4, 4));
}

#[test]
#[ignore = "requires GPU"]
fn svg_defs_match_their_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (pixels, info) = render(CARD, width, height);
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
