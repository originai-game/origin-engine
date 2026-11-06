//! The depth wave's render targets (docs/depth.md §8): the reference
//! frames of a security monitor fed by a second camera (`views-monitor`),
//! a mirror (`views-mirror`) and split-screen (`views-split`), and from
//! frames: an object on a layer only the monitor's camera sees shows on
//! the monitor and nowhere else.

use og_tests::gfx;

fn frame(scene: &og_render::RenderScene) -> og_render::RgbaImage {
    let mut r =
        og_render::Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    r.render(scene, &og_render::RenderTarget::Offscreen)
        .expect("render");
    r.read_offscreen().expect("capture")
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

/// A view draws the main pass's effects from its own eye: each effect in
/// front of the viewport view's camera moves its half of the frame (x
/// 640..1280) and leaves the main camera's half alone. Before the debt
/// round 2 a view drew none of them (ribbons, trails and beams; 2D
/// lighting; 2D fluids), and its soft particles read a placeholder depth
/// and vanished.
#[test]
fn a_view_draws_the_main_passs_effects() {
    use og_tests::target_scenes::{ViewFx, view_fx_scene};
    let mut failures = Vec::new();
    for fx in ViewFx::ALL {
        let on = frame(&view_fx_scene(fx, true));
        let off = frame(&view_fx_scene(fx, false));
        if let Ok(dir) = std::env::var("OG_VIEW_FX_OUT") {
            gfx::save_png(
                std::path::Path::new(&format!("{dir}/view-fx-{fx:?}.png")),
                &on,
            )
            .expect("save");
            gfx::save_png(
                std::path::Path::new(&format!("{dir}/view-fx-{fx:?}-off.png")),
                &off,
            )
            .expect("save");
        }
        let (mut view, mut main) = (0, 0);
        for y in 0..on.height {
            for x in 0..on.width {
                let i = ((y * on.width + x) * 4) as usize;
                let d = (0..3)
                    .map(|c| on.pixels[i + c].abs_diff(off.pixels[i + c]))
                    .max()
                    .unwrap_or(0);
                if d > 12 {
                    if x >= on.width / 2 {
                        view += 1;
                    } else {
                        main += 1;
                    }
                }
            }
        }
        println!("{fx:?}: {view} pixels moved in the view, {main} in the main frame");
        if view < 400 {
            failures.push(format!("{fx:?}: the view drew it on {view} pixels"));
        }
        if main > 0 {
            failures.push(format!("{fx:?}: the main frame moved on {main} pixels"));
        }
        // Soft particles fade where they meet the wall: they differ from
        // the same particles drawn hard.
        if fx == ViewFx::SoftParticles {
            let hard = frame(&view_fx_scene(ViewFx::HardParticles, true));
            let faded = (0..(on.width * on.height) as usize)
                .filter(|&i| {
                    (0..3).any(|c| on.pixels[i * 4 + c].abs_diff(hard.pixels[i * 4 + c]) > 12)
                })
                .count();
            println!("SoftParticles: {faded} pixels differ from the hard ones");
            if faded < 400 {
                failures.push(format!("SoftParticles: {faded} pixels faded"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
