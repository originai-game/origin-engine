//! The per-frame texture upload budget (`set_upload_budget`, the web's)
//! never reaches the probe's capture: a renderer seeing its textures for
//! the first time (one made again after a lost device) captures every one
//! of them, not the 1×1 stand-ins its display frames draw until a later
//! frame uploads them. Its own test binary: the budget is the process's.
//!
//! What would break: `Renderer::capture_frame` drawing under the budget,
//! so a style pack's art captured on the web after a device loss was the
//! transparent stand-in (nothing drawn) while the backdrop drew.
use glam::Vec3;
use og_render::paint::{IMAGE_FILL, PaintKind, PaintQuad};
use og_render::{
    Background, ColorSpace, Mips, Overlay, RenderDesc, RenderScene, Renderer, TextureData,
    TextureHandle, Tonemap,
};

const W: u32 = 64;

fn solid(scene: &mut RenderScene, rgb: [u8; 3]) -> TextureHandle {
    scene.add_texture(TextureData {
        width: 16,
        height: 16,
        rgba: [rgb[0], rgb[1], rgb[2], 255].repeat(16 * 16),
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    })
}

fn image(t: TextureHandle, rect: [f32; 4]) -> PaintQuad {
    PaintQuad {
        rect,
        clip: [0.0, 0.0, W as f32, W as f32],
        color: [1.0; 4],
        radius: [0.0; 4],
        p: [0.0; 4],
        kind: PaintKind::Image as u32,
        a: t.to_bits(),
        b: IMAGE_FILL,
        xf: 0,
        uv: [0.0, 0.0, 1.0, 1.0],
        wipe: [0.0; 4],
        mask: 0,
        filter: 0,
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_capture_draws_every_texture_past_the_upload_budget() {
    // One byte: each frame uploads one texture, the rest are stand-ins.
    og_render::set_upload_budget(1);
    let mut scene = RenderScene::new();
    scene.ui.overlay = Some(Overlay::new(W as f32, W as f32, 1.0));
    let red = solid(&mut scene, [255, 0, 0]);
    let green = solid(&mut scene, [0, 255, 0]);
    let p = &mut scene.ui.paint;
    p.clear();
    (p.width, p.height) = (W as f32, W as f32);
    p.quads = vec![
        image(red, [0.0, 0.0, 32.0, W as f32]),
        image(green, [32.0, 0.0, 32.0, W as f32]),
    ];
    p.generation = 1;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("renderer");
    let (_, request) = renderer.capture_frame(&scene, W, W).expect("capture");
    let px = request
        .wait(&renderer, std::time::Duration::from_secs(30))
        .expect("readback")
        .pixels;
    let at = |x: u32| -> [u8; 3] {
        px[((W / 2 * W + x) * 4) as usize..][..3]
            .try_into()
            .unwrap()
    };
    assert_eq!(
        (at(16), at(48)),
        ([255, 0, 0], [0, 255, 0]),
        "the captured images: the first frame's budget uploads only one"
    );
    assert_eq!(og_render::textures_deferred(), 0, "nothing left for later");
}
