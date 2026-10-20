//! Instance uploads write only what changed (og-render's upload.rs, both
//! culling paths): a scene that stands still uploads no transform on its
//! second frame, moving one instance uploads a block or two of 4 KiB, and
//! the frame after such a partial upload is byte for byte the frame a fresh
//! renderer draws of the same scene (nothing stale is left in the buffer).

use og_render::{Culling, FrameStats, RenderTarget, Renderer, Tier};
use og_tests::scenes;

fn frame(renderer: &mut Renderer, scene: &og_render::RenderScene) -> FrameStats {
    let mut stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.finish_stats(&mut stats);
    stats
}

fn check(culling: Culling) {
    let mut built = scenes::build("spheres-20k").expect("scene");
    built.desc.culling = culling;
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    if culling == Culling::Gpu && renderer.info().tier != Tier::Full {
        return;
    }
    let n = built.scene.instances.len() as u64;
    let first = frame(&mut renderer, &built.scene);
    assert!(
        // An instance is 48 bytes on the GPU (its affine 3x4's rows).
        first.instance_upload_bytes >= n * 48,
        "{culling:?}: the first frame writes every transform ({} bytes for {n})",
        first.instance_upload_bytes
    );
    let still = frame(&mut renderer, &built.scene);
    assert_eq!(
        still.instance_upload_bytes, 0,
        "{culling:?}: a scene that stood still wrote transforms again"
    );
    // One sphere in the middle of the field rises by a metre.
    let k = built.scene.instances.len() / 2;
    built.scene.instances[k].transform =
        glam::Mat4::from_translation(glam::Vec3::Y) * built.scene.instances[k].transform;
    let moved = frame(&mut renderer, &built.scene);
    assert!(
        moved.instance_upload_bytes > 0 && moved.instance_upload_bytes <= 4 * 4096,
        "{culling:?}: moving one sphere wrote {} bytes",
        moved.instance_upload_bytes
    );
    let partial = renderer.read_offscreen().expect("capture");
    let mut fresh = Renderer::new_blocking(&built.desc).expect("renderer");
    frame(&mut fresh, &built.scene);
    let whole = fresh.read_offscreen().expect("capture");
    assert!(
        partial.pixels == whole.pixels,
        "{culling:?}: the frame after a partial upload differs from a whole upload's"
    );
}

#[test]
fn a_still_scene_uploads_no_transforms_and_a_moved_instance_one_block() {
    check(Culling::Cpu);
}

#[test]
fn gpu_culling_uploads_only_changed_transforms() {
    check(Culling::Gpu);
}
