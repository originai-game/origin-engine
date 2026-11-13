//! CSS box decorations: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/box_decor.rs.

use og_ui::*;

const PAGE: &str = r#"<meta name="component" content="t">
<main id="Page">
  <h2 i18n="h">Box decorations</h2>
  <div class="grid">
    <div class="card solid"><p i18n="a">Solid</p></div>
    <div class="card dashed"><p i18n="b">Dashed</p></div>
    <div class="card dotted"><p i18n="c">Dotted</p></div>
    <div class="card double"><p i18n="d">Double</p></div>
    <div class="card inset"><p i18n="e">Inset</p></div>
    <div class="card groove"><p i18n="k">Groove</p></div>
    <div class="card sides"><p i18n="f">Sides</p></div>
    <div class="card raised"><p i18n="g">Shadow</p></div>
    <div class="card outlined"><p i18n="h">Outline</p></div>
    <div class="card clipped"><div class="spill"></div></div>
    <div class="card corners"><div class="spill"></div></div>
    <div class="card shaped"><p i18n="s">Shapes</p></div>
  </div>
  <button class="go" i18n="go">Continue</button>
</main>
<style>
main { width: 100%; height: 100%; padding: 24px; gap: 16px; background: #4a5060; }
.grid { display: grid; grid-template-columns: 110px 110px 110px; gap: 24px; }
.card { height: 64px; padding: 8px; background: var(--panel); border-radius: 10px; }
.card p { margin: 0; }
.solid { border: 2px solid var(--accent); }
.dashed { border: 3px dashed var(--danger); }
.dotted { border: 2px dotted; }
.double { border: 6px double var(--accent); }
.inset { border: 4px inset var(--control-strong); border-radius: 0; }
.groove { border: 6px groove var(--control-strong); border-radius: 0; }
.sides { border-top: 6px solid var(--accent); border-left: 2px solid var(--danger); border-radius: 0; }
.raised { box-shadow: 0 6px 16px #000000aa, inset 0 0 0 1px #ffffff33; }
.outlined { outline: 2px solid var(--accent); outline-offset: 4px; }
.clipped { overflow: hidden; }
.spill { width: 160px; height: 90px; background: var(--danger); }
.corners { overflow: hidden; border: 3px solid var(--accent); border-radius: 40px 8px / 24px 8px; box-shadow: 0 6px 12px #000000aa; }
.shaped { border: 2px solid var(--accent); border-radius: 50% 20px 20px 20px; corner-shape: round squircle bevel scoop; }
.go { border: 2px solid var(--accent); border-radius: 20px; margin: 0 auto; }
@media (width < 600px) { .grid { grid-template-columns: 110px 110px; } }
</style>"#;

#[test]
#[ignore = "requires GPU"]
fn box_decor_matches_its_reference_frames() {
    frames(PAGE, "box-decor", &[(1280, 720), (390, 844)]);
}

/// Background, mask and border image layers.
const LAYERS: &str = r#"<meta name="component" content="t">
<main id="Page">
  <div class="grid">
    <div class="card tiles"><p i18n="a">Tiles</p></div>
    <div class="card origin"><p i18n="b">Content</p></div>
    <div class="card blend"><p i18n="c">Blend</p></div>
    <p class="clipped" i18n="d">Clipped text</p>
    <div class="card framed"><p i18n="e">Frame</p></div>
    <div class="card cut"><p i18n="f">Masks</p></div>
    <div class="card notched"><p i18n="g">Mask border</p></div>
    <div class="card fixed"></div>
  </div>
</main>
<style>
main { width: 100%; height: 100%; padding: 24px; background: #4a5060; }
.grid { display: grid; grid-template-columns: 150px 150px 150px 150px; gap: 24px; }
.card { height: 90px; padding: 12px; background: var(--panel); border-radius: 12px; }
.card p { margin: 0; }
.tiles { background: radial-gradient(circle, #ffd166 30%, transparent 32%) 0 0 / 20px 20px, #264653; }
.origin { border: 8px dashed #ffffff66; padding: 16px; background: linear-gradient(90deg, #e76f51, #2a9d8f) content-box no-repeat; }
.blend { background: linear-gradient(#ff0000, #0000ff), linear-gradient(90deg, #ffffff, #000000); background-blend-mode: multiply; }
.clipped { width: max-content; font: var(--text-display); background: linear-gradient(90deg, #ffd166, #ef476f); background-clip: text; color: transparent; }
.framed { border: 14px solid; border-image: repeating-linear-gradient(45deg, #ffd166 0 6px, #264653 6px 12px) 14 round; }
.cut { background: #2a9d8f; mask: linear-gradient(#000, transparent), radial-gradient(circle at 70% 50%, #000 30%, transparent 32%) add; mask-composite: exclude, add; }
.notched { padding: 1px 12px; background: #ef476f; mask-border: linear-gradient(#000, #000) 20 / 20px; mask-border-mode: alpha; }
.fixed { background: linear-gradient(#ffffff22, #00000055), repeating-linear-gradient(90deg, #e9c46a 0 10px, #f4a261 10px 20px) fixed; }
</style>"#;

#[test]
#[ignore = "requires GPU"]
fn box_layers_match_their_reference_frames() {
    frames(LAYERS, "box-layers", &[(1280, 720)]);
}

/// Renders `page` at each size and compares it with `references/vulkan/<name>-<width>.png`.
fn frames(page: &str, stem: &str, sizes: &[(u32, u32)]) {
    use crate::support::*;
    use og_render::*;
    for &(width, height) in sizes {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/t.svelte", page).unwrap());
        ui.mount_document(
            og_ui::reactive::SourceId::new("test", "t"),
            "t",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..3 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        let frame = render_ui(&ui, &mut scene, width, height);
        assert_eq!(frame.stats.missing_glyphs, 0);
        let name = format!("{stem}-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}
