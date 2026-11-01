//! Presentation (og-render's `renderer/presentation.rs`, G1-4): a lost
//! device and the renderer made again on a new one, the render scale, and
//! the GPU memory the renderer counts. GPU tests: the coordinator runs them
//! on the runners (metal on macrunner, DX12 on studio-win).
//!
//! Ways this could fail: a lost device's frame drawn anyway (a panic, or a
//! black frame passed as drawn); a recreated renderer missing what only
//! the first one uploaded (a texture, a material, the IBL), so its first
//! frame differs; the render scale shrinking the UI with the scene, or not
//! shrinking the scene's targets; a texture or target left out of the
//! memory count.

use og_render::{RenderError, RenderScene, RenderTarget, Renderer, RgbaImage, TextureData};
use og_tests::scenes;

/// `frames` frames of `scene` (TAA's history, as the reference harness
/// draws), the last read back.
fn draw(renderer: &mut Renderer, scene: &RenderScene, frames: u32) -> RgbaImage {
    for _ in 0..frames.max(1) {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("frame");
    }
    renderer.read_offscreen().expect("capture")
}

/// The frame after a loss is the frame before it: a renderer made again
/// from the same description uploads the whole scene (meshes, textures,
/// materials, the sky's light) on its first frame, and draws it as the
/// first renderer drew its first.
#[test]
fn a_renderer_made_again_after_a_lost_device_draws_the_same_frame() {
    let g = scenes::build("materials").expect("scene");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let before = draw(&mut renderer, &g.scene, g.frames);
    assert!(renderer.lost().is_none());

    renderer.lose_device();
    let why = renderer.lost().expect("the device-lost callback ran");
    assert!(why.starts_with("destroyed"), "{why}");
    match renderer.render(&g.scene, &RenderTarget::Offscreen) {
        Err(RenderError::DeviceLost(m)) => assert_eq!(m, why),
        other => panic!("a frame on a lost device: {:?}", other.map(|_| ())),
    }

    let mut again = renderer.recreate_blocking().expect("a new device");
    drop(renderer);
    assert!(again.lost().is_none());
    let after = draw(&mut again, &g.scene, g.frames);
    assert_eq!((after.width, after.height), (before.width, before.height));
    assert!(
        after.pixels == before.pixels,
        "the frame after the loss differs from the frame before it"
    );
}

/// At render scale 0.5 the scene's targets are a quarter of the pixels and
/// the output keeps its size; the frame is the scene upscaled, so it is
/// close to the full-scale frame but not equal; and the render targets'
/// memory falls.
#[test]
fn the_render_scale_draws_the_scene_smaller_and_the_output_at_its_size() {
    let g = scenes::build("materials").expect("scene");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let full = draw(&mut renderer, &g.scene, g.frames);
    let full_targets = renderer.gpu_memory().render_targets;
    renderer.set_render_scale(0.5);
    let half = draw(&mut renderer, &g.scene, g.frames);
    let (w, h) = renderer.size();
    assert_eq!(renderer.scene_size_drawn(), (w / 2, h / 2));
    assert_eq!((half.width, half.height), (full.width, full.height));
    let diff = half
        .pixels
        .iter()
        .zip(&full.pixels)
        .map(|(a, b)| f64::from(a.abs_diff(*b)))
        .sum::<f64>()
        / half.pixels.len() as f64;
    assert!(diff > 0.0, "the half-scale frame is the full one");
    assert!(diff < 12.0, "the half-scale frame is not the scene: {diff}");
    assert!(
        renderer.gpu_memory().render_targets < full_targets,
        "the scene's targets did not shrink"
    );
    renderer.set_render_scale(f32::NAN);
    assert_eq!(renderer.render_scale(), 1.0);
}

/// A texture added is counted at its allocated size (RGBA8 with its mip
/// chain), and freed when removed.
#[test]
fn gpu_memory_counts_a_texture_where_it_is_created_and_frees_it() {
    let g = scenes::build("materials").expect("scene");
    let mut scene = g.scene;
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    draw(&mut renderer, &scene, 1);
    let before = renderer.gpu_memory();
    let t = scene.add_texture(TextureData {
        width: 256,
        height: 256,
        rgba: vec![255; 256 * 256 * 4],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::Generate,
    });
    draw(&mut renderer, &scene, 1);
    let with = renderer.gpu_memory();
    // 256² RGBA8 and its chain down to 1×1: 349,524 bytes.
    let chain: u64 = (0..9).map(|l| (256u64 >> l).pow(2) * 4).sum();
    assert_eq!(with.textures - before.textures, chain);
    assert!(with.total() > before.total());
    assert!(scene.remove_texture(t));
    draw(&mut renderer, &scene, 1);
    assert_eq!(renderer.gpu_memory().textures, before.textures);
}
