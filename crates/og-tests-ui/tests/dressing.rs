//! CSS parts: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/dressing.rs.

use crate::support::*;
use og_ui::*;

#[test]
#[ignore = "requires GPU"]
fn css_parts_render_their_background_border_and_shadow() {
    use og_render::*;
    // Three buttons in the states the frame shows: at rest, hovered, and a
    // primary one of an epic rarity hovered (both forced, as the probe
    // forces them: the pointer alone never hovers two at once).
    const STATES: &str = r#"<meta name="component" content="states">
<row id="States">
  <button id="Rest"></button>
  <button id="Hover"></button>
  <button id="Epic" variant="primary" data-rarity="epic"></button>
</row>
<style>
#States { padding: 24px; gap: 24px; }
button, button::part(base) { width: 120px; height: 40px; }
</style>"#;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.set_part_css(include_str!("../../og-ui/tests/fixtures/parts.css"))
        .unwrap();
    let (width, height) = (464, 96);
    let vp = Viewport::new(width as f32, height as f32);
    mount_screen_document(&mut ui, "states", STATES, &[]);
    step_screen(&mut ui, InputFrame::default(), vp);
    for id in ["Hover", "Epic"] {
        ui.force_state(&format!("screen:states/{id}"), "interaction", Some("hover"));
    }
    for _ in 0..2 {
        step_screen(&mut ui, InputFrame::default(), vp);
    }
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .unwrap();
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-parts.png"),
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for (x, expected) in [
        (24, [254, 220, 186]),
        (84, [0x12, 0x34, 0x56]),
        (228, [255, 255, 0]),
        (372, [0, 0, 255]),
    ] {
        let at = (44 * width as usize + x) * 4;
        for c in 0..3 {
            assert!(
                (i16::from(pixels[at + c]) - expected[c]).abs() <= 3,
                "pixel at {x},44: {:?}",
                &pixels[at..at + 4]
            );
        }
    }
    let shadow = (67 * width as usize + 84) * 4;
    let ground = (85 * width as usize + 84) * 4;
    assert!(
        pixels[shadow] < pixels[ground],
        "the outer shadow darkens the ground"
    );
}
