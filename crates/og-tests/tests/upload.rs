//! Instance uploads write only what changed (og-render's upload.rs, both
//! culling paths): a scene that stands still uploads no transform on its
//! second frame, moving one instance uploads a block or two of 4 KiB, and
//! the frame after such a partial upload is byte for byte the frame a fresh
//! renderer draws of the same scene (nothing stale is left in the buffer).
//! And only what a draw reads: the CPU path's sorted records that no list
//! draws whole are not written.

use og_render::{Culling, FrameStats, RenderTarget, Renderer};
use og_tests::scenes;

fn frame(renderer: &mut Renderer, scene: &og_render::RenderScene) -> FrameStats {
    let mut stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.finish_stats(&mut stats);
    stats
}

/// With a good share of the field out of view, the CPU path draws copies
/// of the spheres in view, appended after the sorted records, and no list
/// draws the sorted ones: only the copies are uploaded, a record of 64
/// bytes per sphere drawn. (It wrote the sorted records too: 20,000
/// records and the copies, 116 bytes a sphere at a fifth out of view.)
#[test]
#[ignore = "requires GPU"]
fn only_the_records_a_draw_reads_are_uploaded() {
    let mut built = scenes::build("spheres-20k").expect("scene");
    built.desc.culling = Culling::Cpu;
    built.scene.lighting.light.cast_shadows = false;
    // Looking past the field's edge.
    let camera = &mut built.scene.frame.camera;
    camera.target += (camera.target - camera.position)
        .cross(glam::Vec3::Y)
        .normalize()
        * 80.0;
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let first = frame(&mut renderer, &built.scene);
    let drawn = (first.instances - first.culled) as u64;
    assert!(
        first.culled * 10 > first.instances * 2 && drawn > 1000,
        "the view is meant to leave out a good share: {} of {} out of view",
        first.culled,
        first.instances
    );
    // The copies in whole 4 KiB blocks and the ground's record (a batch of
    // one, drawn whole from the sorted records) in a block of its own.
    let bound = (drawn * 64).next_multiple_of(4096) + 4096;
    assert!(
        first.instance_upload_bytes >= drawn * 64 && first.instance_upload_bytes <= bound,
        "{} sphere records drawn wrote {} bytes ({:.1} a record drawn; at most {bound})",
        drawn,
        first.instance_upload_bytes,
        first.instance_upload_bytes as f64 / drawn as f64
    );
}
