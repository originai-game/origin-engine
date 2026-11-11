//! CSS effects in documents (docs/ui.md, CSS paint): group blending,
//! `isolation`, and `clip-path`'s `rect()`, `xywh()`, `shape()`, geometry
//! boxes and `url(#id)` of a `<clipPath>` in user space and in
//! objectBoundingBox units. The frames match `references/vulkan/css-effects-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a blend applied to each shape of an element rather
//! than to its group (two overlapping children of a blended element blend
//! with each other), an isolated group whose children blend with what is
//! under the group, a blend over a group's transparent pixels that
//! darkens them (multiply's fixed-function blend reads them as black), or
//! a blend inside a filtered layer dropped; `rect()`'s right and bottom
//! read as insets, a shape's `by`, curve controls or percentages against
//! the wrong axis, a geometry box that clips the border box, or a
//! `<clipPath>`'s units ignored.

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
  <div id="Clips">
    <div id="ByRect" class="cut"></div>
    <div id="ByXywh" class="cut"></div>
    <div id="ByShape" class="cut"></div>
    <div id="ByBox" class="cut"><div class="fill"></div></div>
    <div id="ByUrl" class="cut"></div>
    <div id="ByUnits" class="cut"></div>
  </div>
  <svg id="Defs" width="0" height="0">
    <clipPath id="Notch"><rect x="8" y="8" width="40" height="48"/><circle cx="64" cy="32" r="22"/></clipPath>
    <clipPath id="Diamond" clipPathUnits="objectBoundingBox"><path d="M0.5 0 L1 0.5 L0.5 1 L0 0.5 Z"/></clipPath>
  </svg>
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
#Clips { display: flex; flex-direction: row; gap: 16px; }
.cut { width: 80px; height: 64px; background: linear-gradient(135deg, #d2a8ff, #1f6feb); }
#ByRect { clip-path: rect(8px auto 75% 25%  round 8px); }
#ByXywh { clip-path: xywh(10% 10% 60% 70% round 12px); }
#ByShape { clip-path: shape(from 0% 100%, line to 50% 0%, curve to 100% 100% with 100% 20%, close); }
#ByBox { padding: 12px; border: 4px solid #f0b429; border-radius: 20px; clip-path: content-box; }
.fill { width: stretch; height: stretch; background: #7ee787; }
#ByUrl { clip-path: url(#Notch); }
#ByUnits { clip-path: url(#Diamond); }
@media (max-width: 500px) {
  #Clips { gap: 4px; }
  .cut { width: 56px; height: 48px; }
  #Blends { gap: 8px; }
  .well, #Iso, #Pair, #Sepia { width: 76px; height: 64px; }
  .dot { width: 40px; height: 40px; border-radius: 20px; }
  .b { inset-inline-start: 28px; }
}
</style>"##;

fn render(width: u32, height: u32) -> Rendered {
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
    render_ui(&ui, &mut scene, width, height)
}

#[test]
#[ignore = "requires GPU"]
fn css_effects_match_their_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let frame = render(width, height);
        let name = format!("css-effects-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}
