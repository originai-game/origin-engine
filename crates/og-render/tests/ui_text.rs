//! Distance-field text effects must disappear before the glyph quad ends,
//! including when synthetic weight has consumed some of the field's reach.
use glam::Vec3;
use og_render::{
    Background, ColorSpace, Glyph, Mips, MsdfFont, Overlay, RenderDesc, RenderScene, RenderTarget,
    Renderer, Text, TextureData, Tonemap, UiAlign, UiTextEffects, UiTextOutline,
};

#[test]
#[ignore = "requires GPU"]
fn oversized_stroke_on_weighted_text_never_paints_the_glyph_quad() {
    let mut scene = RenderScene::new();
    // A circular glyph with a known true-distance field, saturated to zero
    // well before the quad's corners. Match the HUD's 11px range at 32px/em.
    let rgba = (0..32 * 32)
        .flat_map(|i| {
            let x = (i % 32) as f32 + 0.5 - 16.0;
            let y = (i / 32) as f32 + 0.5 - 16.0;
            let d = 8.0 - x.hypot(y);
            let v = ((0.5 + d / 11.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            [v; 4]
        })
        .collect();
    let atlas = scene.add_texture(TextureData {
        width: 32,
        height: 32,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let font = scene.add_font(MsdfFont {
        atlas,
        distance_range: 11.0,
        foreground: false,
        silhouette: false,
        line_height: 1.0,
        glyphs: vec![Glyph {
            key: 'O'.into(),
            advance: 1.0,
            plane_bounds: [0.0, 0.0, 1.0, 1.0],
            atlas_bounds: [0.0, 0.0, 32.0, 32.0],
        }],
        kerning: Vec::new(),
    });
    let mut overlay = Overlay::new(128.0, 64.0, 1.0);
    // Regular is the control. The two bold weights distinguish an atlas
    // limit from a constant fudge factor; 11px reproduces the small HUD.
    for (i, weight) in [0.0, 0.25, 0.6].into_iter().enumerate() {
        let x = 16.0 + i as f32 * 32.0;
        overlay.texts.push(Text {
            size: 11.0,
            position: Vec3::new(x, 27.0, 0.0),
            weight,
            color: [0.0, 0.0, 0.0, 1.0],
            ..Text::new(font, "O")
        });
        overlay.ui.text_effects.push(UiTextEffects {
            text: i as u32,
            outline: Some(UiTextOutline {
                width: 1.5,
                color: [0.0, 0.0, 0.0, 1.0],
                align: UiAlign::Outside,
            }),
            ..Default::default()
        });
    }
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::ONE);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 64,
        ..Default::default()
    })
    .expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    for i in 0..3 {
        let x = 16 + i * 32;
        for (xx, yy) in [(x, 16), (x + 10, 16), (x, 26), (x + 10, 26)] {
            let rgb = pixel(&image.pixels, xx, yy);
            assert!(
                rgb.iter().all(|&c| c >= 254),
                "glyph {i} corner {xx},{yy}: {rgb:?}"
            );
        }
        assert!(
            pixel(&image.pixels, x + 5, 21)[0] < 10,
            "glyph {i} still draws its ink"
        );
        assert!(
            pixel(&image.pixels, x + 1, 21)[0] < 245,
            "glyph {i} still draws its stroke"
        );
    }
}

fn pixel(pixels: &[u8], x: usize, y: usize) -> &[u8] {
    &pixels[(y * 128 + x) * 4..][..3]
}
