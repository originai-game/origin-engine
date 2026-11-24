//! Presentation (og-render's `renderer/presentation.rs`): a lost device and
//! the renderer made again on a new one. GPU test: the coordinator runs it
//! on the runners (metal on macrunner, DX12 on studio-win).
//!
//! Ways this could fail: a lost device's frame drawn anyway (a panic, or a
//! black frame passed as drawn); a recreated renderer missing what only
//! the first one uploaded (a texture, a material, the IBL), so its first
//! frame differs.

use og_render::{RenderError, RenderScene, RenderTarget, Renderer, RgbaImage};
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
#[ignore = "requires GPU"]
fn a_renderer_made_again_after_a_lost_device_draws_the_same_frame() {
    let mut g = scenes::build("materials").expect("scene");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let before = draw(&mut renderer, &g.scene, g.frames);
    assert!(renderer.lost().is_none());

    let texture = og_render::TextureHandle::from_index(0);
    let source = g.scene.texture(texture).expect("material texture").clone();
    let recovered = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    assert!(renderer.texture_uploaded(&g.scene, texture));
    let reads = recovered.clone();
    g.scene.release_texture_source(texture, move || {
        reads.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(og_render::TextureSource::Rgba(source.clone()))
    });
    // The preceding CPU inspection pins it for one frame. No further CPU
    // reads: the second release must leave only its reload source.
    g.scene
        .release_texture_source(texture, || unreachable!("reload already registered"));

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
    assert_eq!(recovered.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!((after.width, after.height), (before.width, before.height));
    assert!(
        after.pixels == before.pixels,
        "the frame after the loss differs from the frame before it"
    );
}
