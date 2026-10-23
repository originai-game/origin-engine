//! The depth wave's render targets (docs/depth.md §8): the reference
//! frames of a security monitor fed by a second camera (`views-monitor`),
//! a mirror (`views-mirror`) and split-screen (`views-split`), and from
//! frames: an object on a layer only the monitor's camera sees shows on
//! the monitor and nowhere else.

use og_tests::gfx;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-targets")
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn views_monitor_matches_its_reference() {
    check("views-monitor");
}

#[test]
fn views_mirror_matches_its_reference() {
    check("views-mirror");
}

#[test]
fn views_split_matches_its_reference() {
    check("views-split");
}

/// The magenta cube is on layer 1 alone: the main camera (layer 0) leaves
/// it out, the monitor's camera (layers 0 and 1) draws it on the screen.
#[test]
fn a_layer_the_main_camera_lacks_shows_only_on_the_monitor() {
    let frame = gfx::render("views-monitor").expect("render").image;
    let magenta = |x: u32, y: u32| {
        let p = &frame.pixels[((y * frame.width + x) * 4) as usize..][..3];
        p[0] > 150 && p[2] > 130 && p[1] < 90
    };
    // The monitor on the back wall spans about x 545..735, y 183..290.
    let (mut on_screen, mut elsewhere) = (0, 0);
    for y in 0..frame.height {
        for x in 0..frame.width {
            if magenta(x, y) {
                if (545..735).contains(&x) && (183..290).contains(&y) {
                    on_screen += 1;
                } else {
                    elsewhere += 1;
                }
            }
        }
    }
    assert!(on_screen > 20, "{on_screen} magenta pixels on the monitor");
    assert_eq!(elsewhere, 0, "the main camera drew the layer-1 cube");
}

#[test]
fn shader_full_dissolve_matches_its_reference() {
    check("shader-full-dissolve");
}

fn frame(scene: &og_render::RenderScene) -> og_render::RgbaImage {
    let mut r =
        og_render::Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    r.render(scene, &og_render::RenderTarget::Offscreen)
        .expect("render");
    r.read_offscreen().expect("capture")
}

/// A full shader's cut reaches the shadow passes: the ground under the
/// dissolving panel holds fewer shadowed pixels than under the
/// whole ones, and the objects themselves show the ground through.
#[test]
fn a_full_shaders_shadow_follows_its_cut() {
    use og_tests::target_scenes::full_dissolve_scene;
    let shadowed = |img: &og_render::RgbaImage| {
        let mut n = 0;
        // The panel's shadow on the ground, right of it (no object there).
        for y in 350..440 {
            for x in 790..1030 {
                let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
                // The lit ground is light grey; its shadow a darker blue-grey.
                if p[0] < 140 && p[2] > p[0] && p[2] < 190 {
                    n += 1;
                }
            }
        }
        n
    };
    let whole = shadowed(&frame(&full_dissolve_scene(0.0)));
    let cut = shadowed(&frame(&full_dissolve_scene(0.45)));
    assert!(
        (cut as f64) < whole as f64 * 0.9,
        "shadowed pixels whole {whole}, dissolved {cut}"
    );
}
