//! The meshlet path (R1 batches 2 to 4; og_tests_cooked::meshlet_scenes): the
//! clustered tree's level picked and its clusters culled on the GPU.

use super::*;

/// The forest's level-of-detail inset (`meshlet_scenes::INSET`, the
/// plantation from 30 m up, `View::lod_debug`) holds the full mesh's tint
/// and level 1's, the coarser one farther (level 2's orange stands behind
/// it): the near rows stand low in the inset, so green's mean row is below
/// yellow's. A GPU that drew every pine at level 0 shows only green, one
/// that inverted the choice puts yellow lowest, and a view that ignored
/// its own tint shows neither. The main shot outside the inset is
/// untinted. Lit, the tints read green (77, 211, 99) and yellow
/// (215, 211, 77): told apart by hue.
#[test]
#[ignore = "requires GPU"]
fn meshlet_forest_tints_coarser_levels_farther() {
    let img = gfx::render("meshlet-forest").expect("render").image;
    let [ix, iy, iw, ih] = og_tests_cooked::meshlet_scenes::INSET;
    let (w, h) = (img.width as f32, img.height as f32);
    let inset = |x: u32, y: u32| {
        let (x, y) = (x as f32 / w, y as f32 / h);
        x >= ix && x < ix + iw && y >= iy && y < iy + ih
    };
    // The mean row of the pixels `is` picks inside (or outside) the inset, and how many.
    let rows = |is: &dyn Fn(i32, i32, i32) -> bool, inside: bool| {
        let (mut sum, mut n) = (0.0f64, 0.0f64);
        for y in 0..img.height {
            for x in 0..img.width {
                if inset(x, y) != inside {
                    continue;
                }
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
    let (g, ng) = rows(&green, true);
    let (yl, ny) = rows(&yellow, true);
    assert!(ng > 5000.0 && ny > 3000.0, "tints: green {ng}, yellow {ny}");
    assert!(g > yl, "rows: green {g:.0}, yellow {yl:.0}");
    let (_, outside) = rows(&yellow, false);
    assert!(outside < 500.0, "{outside} yellow pixels outside the inset");
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
#[ignore = "requires GPU"]
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
        // Instance 0 is the hills (the ground is a terrain); the rest are
        // the tree's parts.
        let rest: Vec<glam::Mat4> = scene
            .objects
            .instances
            .iter()
            .map(|i| i.transform)
            .collect();
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
            for (inst, m) in scene.objects.instances.iter_mut().zip(pose(f)) {
                inst.transform = m;
            }
            scene.objects.previous_transforms.clear();
            let plain = draw(&mut without, &scene);
            scene.objects.previous_transforms = pose(f.saturating_sub(1));
            let moved = draw(&mut with, &scene);
            (a, b) = (Some(moved), Some(plain));
        }
        scene.objects.previous_transforms.clear();
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
