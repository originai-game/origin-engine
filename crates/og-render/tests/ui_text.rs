//! Overlay text drawn from the glyph atlas's distance-field tier, read back
//! as pixels (no reference frame).
//!
//! What would break:
//! - a text's glyph read from the wrong place in its field page (its uv
//!   not divided by the page's side, or the page not uploaded before the
//!   overlay draws it)
//! - a field page bound without its range, so the edge is a hard step or
//!   the whole quad fills
//! - the glyph's quad placed from its em rectangle with y the wrong way up
use glam::Vec3;
use og_render::glyphs::{GlyphKey, GlyphTier};
use og_render::{
    Background, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer, Text, TextGlyph, Tonemap,
};

const W: usize = 128;

#[test]
#[ignore = "requires GPU"]
fn overlay_text_draws_its_field_glyph_inside_its_quad() {
    let mut scene = RenderScene::new();
    // A disc of radius 8 field pixels in a 32-pixel field whose range is
    // the atlas's (12 field pixels across the edge), offset to the top half
    // of the field so a quad drawn upside down misses it.
    let field: Vec<u8> = (0..32 * 32)
        .flat_map(|i| {
            let x = (i % 32) as f32 + 0.5 - 16.0;
            let y = (i / 32) as f32 + 0.5 - 10.0;
            let d = 8.0 - (x * x + y * y).sqrt();
            let v = ((0.5 + d / 12.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            [v; 4]
        })
        .collect();
    let slot = scene
        .ui
        .glyphs
        .lock()
        .unwrap()
        .insert(
            GlyphKey::new(GlyphTier::Field, 0, 0, 1, 0),
            (0, 0),
            (32, 32),
            &field,
        )
        .unwrap();
    let mut overlay = Overlay::new(W as f32, 64.0, 1.0);
    // One em is 32 CSS px: the field's pixels one to one, its quad from
    // (48, 16) to (80, 48) with the baseline at 48.
    overlay.texts.push(Text {
        glyphs: vec![TextGlyph {
            rect: [0.0, 0.0, 1.0, 1.0],
            tier: GlyphTier::Field,
            page: slot.page,
            uv: [slot.x, slot.y, slot.x + slot.width, slot.y + slot.height],
        }],
        position: Vec3::new(48.0, 48.0, 0.0),
        size: 32.0,
        color: [0.0, 0.0, 0.0, 1.0],
        ..Default::default()
    });
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ONE);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W as u32,
        height: 64,
        ..Default::default()
    })
    .expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let px = |x: usize, y: usize| &image.pixels[(y * W + x) * 4..][..3];
    // The disc's centre, 10 px below the quad's top.
    assert!(px(64, 26)[0] < 10, "the glyph's ink: {:?}", px(64, 26));
    // Below the disc (where an upside-down quad would put it) and the
    // quad's corners: the ground.
    for (x, y) in [(64, 42), (49, 17), (79, 17), (49, 47), (79, 47)] {
        assert!(
            px(x, y).iter().all(|&c| c >= 250),
            "{x},{y} outside the disc: {:?}",
            px(x, y)
        );
    }
    // The edge is antialiased over about a pixel: one whose centre lies
    // 0.28 px outside the disc (7.5 and 3.5 px from its centre at
    // (64, 26)) is between ink and ground.
    let edge = px(71, 29)[0];
    assert!((40..=235).contains(&edge), "the edge pixel: {edge}");
}
