//! Shape effects read back through the public renderer, without reference frames.
use glam::{Vec2, Vec3};
use og_render::{
    Background, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap, UiCorner,
    UiCssCorners, UiGeometry, UiMode, UiShape,
};

#[test]
#[ignore = "requires GPU"]
fn css_bevel_glows_stay_continuous_at_corner_endpoints() {
    const W: u32 = 256;
    let mut scene = RenderScene::new();
    let mut overlay = Overlay::new(W as f32, 160.0, 1.0);
    // Unequal horizontal/vertical radii exercise both ends of each bevel.
    let horizontal = [24.0, 16.0, 20.0, 12.0];
    let vertical = [12.0, 24.0, 14.0, 20.0];
    overlay.ui.shapes.push(UiShape {
        geometry: UiGeometry::Rect {
            radii: horizontal,
            corner: UiCorner::Css(UiCssCorners {
                vertical,
                shape: [0.0; 4],
            }),
        },
        mode: UiMode::Shadow {
            offset: Vec2::ZERO,
            blur: 24.0,
            spread: 0.0,
        },
        ..UiShape::rect([40.0, 40.0, 160.0, 54.0], 0.0, [1.0; 4])
    });
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: 160,
        ..Default::default()
    })
    .expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let pixel = |x: u32, y: u32| image.pixels[((y * W + x) * 4) as usize];
    // A 24px blur changes gradually across adjacent pixels 16px outside
    // the box. Switching from the bevel plane to a box-edge distance here
    // used to introduce bright straight seams through the glow.
    for (a, b) in [
        ((63, 24), (64, 24)),
        ((183, 24), (184, 24)),
        ((179, 109), (180, 109)),
        ((51, 109), (52, 109)),
        ((24, 51), (24, 52)),
        ((215, 63), (215, 64)),
        ((215, 79), (215, 80)),
        ((24, 73), (24, 74)),
    ] {
        let (a, b) = (pixel(a.0, a.1), pixel(b.0, b.1));
        assert!(a > 30 && b > 30, "the glow must draw: {a}, {b}");
        assert!(a.abs_diff(b) <= 8, "a smooth glow has no seam: {a}, {b}");
    }
    assert!(pixel(0, 0) < 4, "the glow still fades to the ground");
}
