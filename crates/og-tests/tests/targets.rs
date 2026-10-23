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

#[test]
fn post_crt_matches_its_reference() {
    check("post-crt");
}

/// A post pass that does not compile is reported and skipped: the frame
/// is the frame without it, and the CRT after it still runs.
#[test]
fn a_failing_post_pass_is_reported_and_skipped() {
    use og_render::{PostShader, RenderTarget, Renderer};
    let plain = frame(&og_tests::target_scenes::room());
    let mut scene = og_tests::target_scenes::room();
    let broken = scene.add_post(PostShader {
        name: "broken".into(),
        code: "fn post(f: PostInput, p: Params) -> vec4<f32> { return nowhere(f.uv); }".into(),
        params: Vec::new(),
    });
    scene.custom_post.push(broken);
    let mut r = Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    r.render(&scene, &RenderTarget::Offscreen).expect("render");
    let skipped = r.read_offscreen().expect("capture");
    let errors = r.post_errors(&scene);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0, broken);
    assert!(errors[0].1.contains("nowhere"), "{}", errors[0].1);
    assert_eq!(
        skipped.pixels, plain.pixels,
        "the skipped pass changed the frame"
    );
    // The CRT after it runs: its corners are black.
    let crt = scene.add_post(og_render::example_crt());
    scene.custom_post.push(crt);
    r.render(&scene, &RenderTarget::Offscreen).expect("render");
    let bent = r.read_offscreen().expect("capture");
    assert_eq!(&bent.pixels[..3], &[0, 0, 0]);
    assert_ne!(&plain.pixels[..3], &[0, 0, 0]);
}

#[test]
fn views_layers_matches_its_reference() {
    check("views-layers");
}

/// A view draws what the main pass draws over its opaque scene: glass, a
/// world sprite, particles and lines on layer 1 alone change the monitor
/// when its camera sees layer 1 and nothing when it does not, and never
/// change the main frame around it (the main camera is on layer 0).
#[test]
fn a_view_draws_transparents_sprites_particles_and_lines_on_its_layers() {
    use og_tests::target_scenes::{ViewKind, view_kinds_scene};
    // The monitor on the back wall, with a pixel's margin.
    let on_monitor = |x: u32, y: u32| (543..738).contains(&x) && (181..293).contains(&y);
    let mut failures = Vec::new();
    for kind in ViewKind::ALL {
        let seen = frame(&view_kinds_scene(&[kind], 3, 60));
        let unseen = frame(&view_kinds_scene(&[kind], 1, 60));
        let (mut inside, mut outside) = (0, 0);
        for y in 0..seen.height {
            for x in 0..seen.width {
                let i = ((y * seen.width + x) * 4) as usize;
                let d = (0..3)
                    .map(|c| seen.pixels[i + c].abs_diff(unseen.pixels[i + c]))
                    .max()
                    .unwrap_or(0);
                if d > 24 {
                    if on_monitor(x, y) {
                        inside += 1;
                    } else {
                        outside += 1;
                    }
                }
            }
        }
        println!("{kind:?}: {inside} pixels moved on the monitor, {outside} elsewhere");
        if inside < 60 {
            failures.push(format!("{kind:?}: the view drew it on {inside} pixels"));
        }
        if outside > 0 {
            failures.push(format!(
                "{kind:?}: the main frame moved on {outside} pixels"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
