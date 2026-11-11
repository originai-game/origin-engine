//! Reflection probes beyond one room (`og_tests::probe_scenes`): their
//! reference frames, and from the frames and the captures read back: each
//! of two rooms sharing a sphere mesh reflects its own room; a capture
//! draws skinned meshes in their pose, terrain, grass and translucent
//! meshes; terrain and grass inside a probe's box are lit by it.

use og_render::{RenderDesc, RenderTarget, Renderer, RgbaImage};
use og_tests::gfx;
use og_tests::probe_scenes::{
    FLOOR_PROBE, Outdoor, add_floor_fluid, outdoor_scene, posed_figure_scene, probe_floor_scene,
    two_rooms_scene,
};
use og_tests::scenes;

/// The frame of `scene` and, when `probe` is given, that probe's capture:
/// level 0's six faces (+x, -x, +y, -y, +z, -z) of linear RGB.
fn render(
    desc: &RenderDesc,
    scene: &og_render::RenderScene,
    probe: Option<u64>,
) -> (RgbaImage, Option<(u32, Vec<f32>)>) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let capture = probe.map(|id| {
        let (size, mut levels) = renderer
            .read_reflection_probe(id)
            .expect("captured")
            .expect("read back");
        (size, levels.swap_remove(0))
    });
    (renderer.read_offscreen().expect("frame"), capture)
}

/// The mean linear RGB of the middle half of cube face `face`.
fn face_mean(capture: &(u32, Vec<f32>), face: usize) -> [f32; 3] {
    let (n, data) = (capture.0 as usize, &capture.1);
    let mut s = [0.0f32; 3];
    let mut count = 0.0;
    for y in n / 4..3 * n / 4 {
        for x in n / 4..3 * n / 4 {
            let at = ((face * n + y) * n + x) * 3;
            for k in 0..3 {
                s[k] += data[at + k];
            }
            count += 1.0;
        }
    }
    s.map(|v| v / count)
}

/// A skinned figure takes the probe of the room its pose stands in: a
/// chrome cylinder whose instance stands in the red room and whose joints
/// carry it into the blue one reflects the blue room, as the same cylinder
/// placed there by its transform does. Picked by its bind pose's bounds,
/// as before the debt round 2, it reflected the red room it is not in.
#[test]
#[ignore = "requires GPU"]
fn a_skinned_instance_takes_the_probe_of_its_pose() {
    let desc = scenes::base_desc();
    let frame = |posed: Option<bool>| {
        let scene = posed.map_or_else(|| two_rooms_scene(true), posed_figure_scene);
        let (img, _) = render(&desc, &scene, None);
        if let (Ok(dir), Some(posed)) = (std::env::var("OG_PROBE_OUT"), posed) {
            gfx::save_png(
                std::path::Path::new(&format!("{dir}/posed-figure-{posed}.png")),
                &img,
            )
            .expect("save");
        }
        img
    };
    let (bare, posed, placed) = (frame(None), frame(Some(true)), frame(Some(false)));
    // The cylinder's pixels: where the placed one changed the room.
    let px = |i: usize| &bare.pixels[i * 4..i * 4 + 3];
    let cylinder: Vec<usize> = (0..(bare.width * bare.height) as usize)
        .filter(|&i| {
            let q = &placed.pixels[i * 4..i * 4 + 3];
            px(i)
                .iter()
                .zip(q)
                .map(|(a, b)| a.abs_diff(*b) as u32)
                .sum::<u32>()
                > 60
        })
        .collect();
    let mean = |img: &RgbaImage| {
        let mut s = [0.0f32; 3];
        for &i in &cylinder {
            for (k, v) in s.iter_mut().enumerate() {
                *v += img.pixels[i * 4 + k] as f32 / cylinder.len() as f32;
            }
        }
        s
    };
    let (posed, placed) = (mean(&posed), mean(&placed));
    println!(
        "the cylinder ({} px): posed {posed:?}, placed {placed:?}",
        cylinder.len()
    );
    assert!(cylinder.len() > 2000, "the cylinder is in view");
    assert!(
        placed[2] > placed[0] + 6.0,
        "placed in the blue room its sides reflect blue: {placed:?}"
    );
    assert!(
        (0..3).all(|k| (posed[k] - placed[k]).abs() < 3.0),
        "posed into the blue room it reflects the blue room: {posed:?} against {placed:?}"
    );
}

/// Local lights reach terrain, grass and 3D fluids in a capture: a blue
/// point light under the outdoor probe turns the meadow on its -y face
/// blue, and one between the floor probe and the red fluid ball lights the
/// ball's near side on its +x face. A face's frame has no light clusters;
/// before the debt round 3 these shaders still read one (the main camera's
/// first), so the light was missing from the face as it is here bare.
#[test]
#[ignore = "requires GPU"]
fn local_lights_reach_terrain_and_fluids_in_a_capture() {
    let desc = scenes::base_desc();
    let blue = |position| og_render::Light {
        position,
        color: glam::Vec3::new(0.1, 0.3, 1.0),
        intensity: 6.0,
        range: 4.0,
        ..Default::default()
    };
    let blue_share = |c: [f32; 3]| c[2] / (c[0] + c[1] + c[2]).max(1e-4);
    let mut failures = Vec::new();

    let mut meadow = outdoor_scene(Outdoor {
        content: false,
        probe: Some(1.0),
    });
    let ny0 = face_mean(&render(&desc, &meadow, Some(7)).1.unwrap(), 3);
    meadow.lighting.lights.push(blue(
        og_tests::probe_scenes::outdoor_probe() - glam::Vec3::Y * 1.2,
    ));
    let ny = face_mean(&render(&desc, &meadow, Some(7)).1.unwrap(), 3);
    println!("meadow -y: lit {ny:?}, bare {ny0:?}");
    if ny[2] < ny0[2] + 0.05 || blue_share(ny) < blue_share(ny0) + 0.1 {
        failures.push(format!("the meadow on -y: lit {ny:?}, bare {ny0:?}"));
    }

    let mut fluid = probe_floor_scene();
    add_floor_fluid(&mut fluid);
    let px0 = face_mean(&render(&desc, &fluid, Some(FLOOR_PROBE.0)).1.unwrap(), 0);
    fluid
        .lighting
        .lights
        .push(blue(FLOOR_PROBE.1 + glam::Vec3::new(0.6, 0.0, 0.0)));
    let px = face_mean(&render(&desc, &fluid, Some(FLOOR_PROBE.0)).1.unwrap(), 0);
    println!("fluid +x: lit {px:?}, bare {px0:?}");
    if px[2] < px0[2] + 0.03 {
        failures.push(format!("the fluid on +x: lit {px:?}, bare {px0:?}"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
