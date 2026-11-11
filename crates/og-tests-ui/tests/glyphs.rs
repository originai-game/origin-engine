//! Missing glyphs: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/glyphs.rs.

/// A text made only of characters its face lacks measures no width (a
/// missing glyph takes no space). It must still reach the renderer on every
/// frame, so `FrameStats::missing_glyphs`, which every reference test
/// asserts, names it, as it does when a drawn character shares the line.
/// The case that hid: a list marker `•` in a font without it vanished
/// with a count of 0.
#[test]
#[ignore = "requires GPU"]
fn a_text_of_only_missing_glyphs_is_counted_by_the_renderer() {
    use crate::support::*;
    use og_render::*;
    use og_ui::{EmitDesc, InputFrame, LabelDesc, Viewport};
    for (text, missing) in [
        ("\u{2022}", 1),
        ("a\u{2022}", 1),
        ("\u{2022}\u{25E6}", 2),
        ("a", 0),
    ] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        // Two frames: og-ui names a missing character once in its life, on
        // its first frame; the renderer must count it on the later one too.
        for _ in 0..2 {
            ui.begin(InputFrame::default(), Viewport::new(200.0, 100.0));
            ui.label("Marker", text, LabelDesc::default());
            ui.end();
        }
        let mut overlay = Overlay::new(200.0, 100.0, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.ui.overlay = Some(overlay);
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width: 200,
            height: 100,
            ..Default::default()
        })
        .expect("a headless adapter");
        let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        assert_eq!(stats.missing_glyphs, missing, "{text:?}");
    }
}
