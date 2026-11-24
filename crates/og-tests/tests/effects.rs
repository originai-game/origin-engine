//! The effects, asserted on frames: particles (determinism of a 600-tick
//! run on both paths).

use og_render::{
    FrameStats, ParticleSimulation, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
};
use og_tests::fx_scenes as fx;
use og_tests::harness::save;
use og_tests::scenes;

fn small() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        ..scenes::base_desc()
    }
}

/// Renders `frames` frames (calling `update(scene, frame)` before each) and
/// returns the last one.
fn run(
    desc: &RenderDesc,
    scene: &mut RenderScene,
    frames: u32,
    update: impl Fn(&mut RenderScene, u32),
) -> (RgbaImage, FrameStats, Renderer) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for f in 0..frames.max(1) {
        update(scene, f);
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats, renderer)
}

/// The 600-tick determinism run for one simulation path: two identical
/// runs give byte-identical frames, and jumping straight to tick 600 (a
/// replay of the last lifetime) gives the same frame as stepping every tick.
fn particles_are_deterministic(path: ParticleSimulation) {
    let desc = RenderDesc {
        particles: path,
        ..small()
    };
    let stepped = |scene: &mut RenderScene, f: u32| fx::tick_all(scene, f as u64);
    // The two runs one after the other on this thread, never at once on
    // two: the Vulkan loader from 1.3.290 to 1.4.344 (Debian trixie's
    // 1.4.309, CI's lavapipe) frees an instance's unused drivers in its
    // first vkEnumeratePhysicalDevices under one lock while another
    // thread's device walks that list under another (debug names, device
    // function lookups), and the walk segfaults on the freed entry
    // (the former test sample fixed upstream in
    // 1.4.345). Each renderer is dropped before the next is made.
    // Display frames may switch lit shader variants as they compile.
    // Compare captures, whose full variants do not depend on that timing.
    let capture = |scene: &mut RenderScene, frames, update: fn(&mut RenderScene, u32)| {
        let (_, _, mut renderer) = run(&desc, scene, frames, update);
        let stats = renderer
            .render_capture(scene, &RenderTarget::Offscreen)
            .expect("render capture");
        (renderer.read_offscreen().expect("capture"), stats)
    };
    let (a, sa) = capture(&mut fx::particles_scene(), 601, stepped);
    let (b, _) = capture(&mut fx::particles_scene(), 601, stepped);
    let (c, _) = capture(&mut fx::particles_scene(), 1, |s, _| fx::tick_all(s, 600));
    save("og-effects", &format!("particles-600-{path:?}"), &a);
    assert!(sa.particle_systems >= 6, "{} systems", sa.particle_systems);
    let lit = a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 200)
        .count();
    assert!(lit > 200, "the particles drew only {lit} bright pixels");
    assert_eq!(a.pixels, b.pixels, "two 600-tick runs differ ({path:?})");
    assert_eq!(
        a.pixels, c.pixels,
        "a jump to tick 600 differs from stepping there ({path:?})"
    );
}

#[test]
#[ignore = "requires GPU"]
fn a_600_tick_gpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Auto);
}

#[test]
#[ignore = "requires GPU"]
fn a_600_tick_cpu_particle_run_is_deterministic() {
    particles_are_deterministic(ParticleSimulation::Cpu);
}
