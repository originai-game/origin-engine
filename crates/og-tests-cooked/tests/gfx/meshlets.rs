//! The meshlet path (R1 batches 2 to 4; og_tests_cooked::meshlet_scenes): the
//! clustered tree's level picked and its clusters culled on the GPU.

use super::*;

/// The forest's frame holds all three tints, the coarser levels nearer
/// the horizon: the canopies stand above the eye, so a nearer one stands
/// higher in the frame; only the sky side of the horizon is counted (the
/// trunks, whose levels barely differ, draw the coarsest everywhere, and
/// stand mostly below it). A GPU that drew every tree at level 0 shows
/// only green, one that inverted the choice puts orange highest. Lit, the
/// tints read green (77, 211, 99), yellow (215, 211, 77) and orange
/// (218, 176, 50) on Metal: told apart by hue, not by a fixed channel.
#[test]
fn meshlet_forest_tints_coarser_levels_farther() {
    let img = gfx::render("meshlet-forest").expect("render").image;
    // The horizon is at row 352 of 720; the canopies' bottoms above it.
    let sky_side = img.height * 350 / 720;
    // The mean row of the pixels `is` picks, and how many.
    let rows = |is: &dyn Fn(i32, i32, i32) -> bool| {
        let (mut sum, mut n) = (0.0f64, 0.0f64);
        for y in 0..sky_side {
            for x in 0..img.width {
                let i = ((y * img.width + x) * 4) as usize;
                let p = &img.pixels[i..i + 3];
                if is(p[0] as i32, p[1] as i32, p[2] as i32) {
                    sum += y as f64;
                    n += 1.0;
                }
            }
        }
        (sum / n.max(1.0), n)
    };
    let green = |r: i32, g: i32, b: i32| g > 120 && g > r + 60 && g > b + 60;
    let yellow = |r: i32, g: i32, b: i32| r > 150 && (r - g).abs() < 20 && b < r - 80;
    let orange = |r: i32, g: i32, b: i32| r > 150 && (25..80).contains(&(r - g)) && b < g - 60;
    let (g, ng) = rows(&green);
    let (yl, ny) = rows(&yellow);
    let (o, no) = rows(&orange);
    assert!(
        ng > 2000.0 && ny > 200.0 && no > 50.0,
        "tints: green {ng}, yellow {ny}, orange {no}"
    );
    assert!(
        g < yl && g < o,
        "rows: green {g:.0}, yellow {yl:.0}, orange {o:.0}"
    );
}

/// TAA follows a moving instance along its own motion vectors on every
/// culling path: on the CPU, under GPU culling (its camera pass compacts
/// the previous transforms beside the current ones, cull_gpu.rs
/// `previous`) and through the meshlet path (which pulls them at the
/// instance's sorted slot, the visibility buffer's setup included). The
/// near tree slides across a still camera for a dozen frames; on each
/// path the frame with its previous transforms is nearer a fresh
/// history's frame of the same pose than the frame without them, which
/// reprojects the camera's motion alone and trails a ghost. A path that
/// binds the previous transforms at the wrong slots (the CPU path's
/// buffer over GPU culling's compacted ranges) moves the history the
/// wrong way and fails as surely as one without them.
#[test]
fn a_moving_instance_takes_its_history_along_on_every_culling_path() {
    use og_render::{
        Antialias, Culling, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    };
    let desc = |culling| RenderDesc {
        width: 320,
        height: 180,
        antialias: Antialias::Taa,
        culling,
        ..og_tests::scenes::base_desc()
    };
    let draw = |r: &mut Renderer, scene: &RenderScene| -> RgbaImage {
        r.render(scene, &RenderTarget::Offscreen).expect("frame");
        r.read_offscreen().expect("capture")
    };
    let diff = |a: &RgbaImage, b: &RgbaImage| -> f64 {
        a.pixels
            .iter()
            .zip(&b.pixels)
            .map(|(x, y)| f64::from(x.abs_diff(*y)))
            .sum::<f64>()
            / a.pixels.len() as f64
    };
    for (name, culling, meshlets) in [
        ("cpu", Culling::Cpu, false),
        ("gpu", Culling::Gpu, false),
        ("meshlets", Culling::Gpu, true),
    ] {
        let mut scene = og_tests_cooked::meshlet_scenes::near_scene(meshlets).expect("the tree");
        // Instance 0 is the ground; the rest are the tree's parts.
        let rest: Vec<glam::Mat4> = scene.instances.iter().map(|i| i.transform).collect();
        let at = |f: u32| {
            glam::Mat4::from_translation(glam::Vec3::new(-1.5 + 0.25 * f as f32, 0.0, 0.0))
        };
        let mut with = Renderer::new_blocking(&desc(culling)).expect("renderer");
        assert_eq!(
            with.info().antialias,
            Antialias::Taa,
            "GPU lacks TAA support"
        );
        let mut without = Renderer::new_blocking(&desc(culling)).expect("renderer");
        let (mut a, mut b) = (None, None);
        for f in 0..12 {
            let pose = |f: u32| -> Vec<glam::Mat4> {
                rest.iter()
                    .enumerate()
                    .map(|(k, m)| if k == 0 { *m } else { at(f) * *m })
                    .collect()
            };
            for (inst, m) in scene.instances.iter_mut().zip(pose(f)) {
                inst.transform = m;
            }
            scene.previous_transforms.clear();
            let plain = draw(&mut without, &scene);
            scene.previous_transforms = pose(f.saturating_sub(1));
            let moved = draw(&mut with, &scene);
            (a, b) = (Some(moved), Some(plain));
        }
        scene.previous_transforms.clear();
        let mut fresh = Renderer::new_blocking(&desc(culling)).expect("renderer");
        let still = draw(&mut fresh, &scene);
        let (d_with, d_without) = (diff(&a.unwrap(), &still), diff(&b.unwrap(), &still));
        println!(
            "{name}: mean byte difference from a fresh frame: with motion {d_with:.2}, camera only {d_without:.2}"
        );
        assert!(
            d_with < d_without,
            "{name}: motion vectors {d_with}, camera only {d_without}"
        );
    }
}
