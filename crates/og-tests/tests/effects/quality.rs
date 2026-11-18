//! Effects-off pass budgets and graphics quality preset frame tests.

use super::*;

/// Particles, trails and the post stack all off: no extra passes (the
/// features-off budget), counted from the frame graph.
#[test]
#[ignore = "requires GPU"]
fn effects_off_add_no_passes() {
    let built = scenes::build("gltf-reference").expect("scene");
    let (_, stats) = frame(&built.desc, &built.scene);
    let mut with = scenes::build("gltf-reference").expect("scene");
    with.scene.look.post.ao = Some(AmbientOcclusion::default());
    with.scene.look.post.depth_of_field = Some(DepthOfField::default());
    with.scene.look.post.motion_blur = Some(MotionBlur::default());
    with.scene.look.post.god_rays = Some(GodRays::default());
    let (_, all) = frame(&with.desc, &with.scene);
    println!(
        "graph passes: off {}, on {}",
        stats.graph_passes, all.graph_passes
    );
    assert_eq!(stats.prepass_draws, 0);
    assert_eq!(all.graph_passes, stats.graph_passes + 1 + 2 + 3 + 1 + 3);
    let _ = Curve::default();
}

/// Resolving the interleaved rays must not brighten a frame whose sky
/// contributes nothing to the ray mask, even with a visible sun.
#[test]
#[ignore = "requires GPU"]
fn an_empty_god_rays_mask_adds_no_light() {
    let mut built = scenes::build("god-rays").expect("scene");
    built.scene.look.post.god_rays.as_mut().unwrap().threshold = f32::MAX;
    let masked = gfx::render_built(&mut built).expect("masked rays").image;
    built.scene.look.post.god_rays = None;
    let off = gfx::render_built(&mut built).expect("rays off").image;
    assert!(masked.pixels == off.pixels, "an empty ray mask adds light");
}

/// Independently recorded, unjittered 1536-tap integrals of this fixture:
/// subdivide each original tap's centred interval into 32 midpoint taps,
/// retaining its decay weight and the march extent, with no spatial resolve.
/// The author's lavapipe 768-to-1536 check moved means by less than 0.005/255.
/// Recorded on NVIDIA L4, on the fixed-angular-size sun mask and physical
/// sun disc. Shipped interleaved rays differ by at most 0.173/255 here.
/// The author's fixed 48-tap lavapipe control undersampled the same mask:
/// its corner means were 0.95/255 too dark.
#[test]
#[ignore = "requires GPU"]
fn god_rays_match_a_converged_radial_integral() {
    let mean = |image: &RgbaImage, (x0, y0, x1, y1): (u32, u32, u32, u32)| {
        let mut rgb = [0.0f64; 3];
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * image.width + x) * 4) as usize;
                for (c, sum) in rgb.iter_mut().enumerate() {
                    *sum += f64::from(image.pixels[i + c]);
                }
            }
        }
        rgb.map(|sum| sum / f64::from((x1 - x0) * (y1 - y0)))
    };
    for (threshold, expected) in [
        (
            2.0,
            [
                [135.161930, 115.587394, 91.390422],
                [89.965521, 88.972292, 99.652917],
                [89.967292, 88.972396, 99.651458],
            ],
        ),
        (
            10.0,
            [
                [135.141935, 115.570474, 91.380028],
                [89.951146, 88.962500, 99.647292],
                [89.951250, 88.961667, 99.648125],
            ],
        ),
    ] {
        let mut built = scenes::build("god-rays").expect("scene");
        built.scene.look.post.god_rays.as_mut().unwrap().threshold = threshold;
        let image = gfx::render_built(&mut built).expect("rays").image;
        assert_eq!((image.width, image.height), (1280, 720));
        for (rect, reference) in [(0, 0, 1280, 720), (0, 0, 120, 80), (1160, 0, 1280, 80)]
            .into_iter()
            .zip(expected)
        {
            let actual = mean(&image, rect);
            println!("threshold {threshold}, {rect:?}: {actual:?}, reference {reference:?}");
            assert!(
                actual
                    .into_iter()
                    .zip(reference)
                    .all(|(a, b)| (a - b).abs() < 0.5),
                "threshold {threshold}, {rect:?}: {actual:?} differs from the converged {reference:?}"
            );
        }
    }
}

/// Scene `name` (`edit` applied) drawn at `quality`: its last frame.
fn at_quality(
    name: &str,
    quality: og_render::RenderQuality,
    edit: impl Fn(&mut RenderScene),
) -> RgbaImage {
    let mut built = scenes::build(name).expect("scene");
    edit(&mut built.scene);
    built.scene.frame.quality = quality;
    gfx::render_built(&mut built).expect("render").image
}

/// The bounding box of the pixels where `a` and `b` differ by more than
/// `by` in a channel, the count of them, and the mean luma of `a` and `b`
/// over the box.
fn moved(a: &RgbaImage, b: &RgbaImage, by: u8) -> Option<([u32; 4], usize, f32, f32)> {
    let (mut x0, mut y0, mut x1, mut y1, mut n) = (u32::MAX, u32::MAX, 0, 0, 0);
    for y in 0..a.height {
        for x in 0..a.width {
            let i = ((y * a.width + x) * 4) as usize;
            if (0..3).any(|k| a.pixels[i + k].abs_diff(b.pixels[i + k]) > by) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
                n += 1;
            }
        }
    }
    (n > 0).then(|| {
        let r = (x0, y0, x1, y1);
        ([x0, x1, y0, y1], n, region(a, r), region(b, r))
    })
}

/// The high graphics preset buys more than the shadow maps' side: ambient
/// occlusion at full resolution where the scene asked half, twice the
/// screen-space reflection steps and twice the froxel grid's slices; each
/// equal, byte for byte, to the frame the scene draws when it asks for
/// that itself (with high's shadows: maps at twice their side, four
/// cascades, sixteen shadowed lights). The
/// medium preset is the default frame byte for byte. Before the debt
/// round high was medium with doubled shadow maps and nothing else: the
/// three high frames below equalled the `shadows only` ones.
#[test]
#[ignore = "requires GPU"]
fn the_high_preset_refines_ao_reflections_and_fog_and_medium_moves_nothing() {
    use og_render::{AoResolution, QualityPreset, RenderQuality};
    let default = RenderQuality::default();
    let medium = RenderQuality::preset(QualityPreset::Medium);
    let high = RenderQuality::preset(QualityPreset::High);
    // Every shadow field high changes (its maps' side, its cascades and its
    // shadowed lights), so the frames below differ by the refinement alone.
    let shadows_only = RenderQuality {
        shadow_map_scale: high.shadow_map_scale,
        shadow_cascades: high.shadow_cascades,
        shadowed_lights: high.shadowed_lights,
        ..default
    };
    // What moved between `a` and `b`: everywhere, and in `rect` (x0, y0,
    // x1, y1) where the refinement shows (its mean luma and the pixels
    // moved there).
    let report = |what: &str, a: &RgbaImage, b: &RgbaImage, rect: (u32, u32, u32, u32)| {
        save(&format!("quality {what} a"), a);
        save(&format!("quality {what} b"), b);
        let m = moved(a, b, 2);
        let crop = |img: &RgbaImage| RgbaImage {
            width: rect.2 - rect.0,
            height: rect.3 - rect.1,
            pixels: (rect.1..rect.3)
                .flat_map(|y| {
                    let i = ((y * img.width + rect.0) * 4) as usize;
                    img.pixels[i..i + ((rect.2 - rect.0) * 4) as usize].to_vec()
                })
                .collect(),
        };
        let inside = moved(&crop(a), &crop(b), 2).map_or(0, |m| m.1);
        match m {
            Some((r, n, _, _)) => println!(
                "{what}: {n} pixels moved by more than 2 (within x {}..{} y {}..{}); in x {}..{} y {}..{}: {inside} of them, mean luma {:.2} -> {:.2}",
                r[0],
                r[1],
                r[2],
                r[3],
                rect.0,
                rect.2,
                rect.1,
                rect.3,
                region(a, rect),
                region(b, rect)
            ),
            None => println!("{what}: no pixel moved by more than 2"),
        }
        inside
    };

    // Ambient occlusion: the scene asks half resolution.
    let half = |s: &mut RenderScene| {
        if let Some(ao) = &mut s.look.post.ao {
            ao.resolution = AoResolution::Half;
        }
    };
    let full = |s: &mut RenderScene| {
        if let Some(ao) = &mut s.look.post.ao {
            ao.resolution = AoResolution::Full;
        }
    };
    let ao_default = at_quality("ao-gtao", default, half);
    assert!(at_quality("ao-gtao", medium, half).pixels == ao_default.pixels);
    let ao_high = at_quality("ao-gtao", high, half);
    let ao_shadows = at_quality("ao-gtao", shadows_only, half);
    assert!(
        ao_high.pixels == at_quality("ao-gtao", shadows_only, full).pixels,
        "high draws the scene's AO at full resolution"
    );
    // The broken column's foot on the setts and the drum lying by it.
    let r = (610, 600, 900, 690);
    assert!(
        report(
            "ao-gtao, half asked: shadows only -> high",
            &ao_shadows,
            &ao_high,
            r
        ) > 0
    );

    // Screen-space reflections: the scene asks 48 steps.
    let steps = |n: u32| {
        move |s: &mut RenderScene| {
            if let Some(ssr) = &mut s.look.post.ssr {
                ssr.steps = n;
            }
        }
    };
    let ssr_default = at_quality("ssr-wet-floor", default, |_| {});
    assert!(at_quality("ssr-wet-floor", medium, |_| {}).pixels == ssr_default.pixels);
    let ssr_high = at_quality("ssr-wet-floor", high, |_| {});
    let ssr_shadows = at_quality("ssr-wet-floor", shadows_only, |_| {});
    assert!(
        ssr_high.pixels == at_quality("ssr-wet-floor", shadows_only, steps(96)).pixels,
        "high marches twice the scene's 48 steps"
    );
    // The right lantern's light standing in the wet setts below it.
    let r = (745, 405, 885, 665);
    assert!(
        report(
            "ssr-wet-floor: shadows only -> high",
            &ssr_shadows,
            &ssr_high,
            r
        ) > 0
    );

    // Volumetric fog: 64 slices against 128.
    let fog_default = at_quality("volumetric-fog", default, |_| {});
    assert!(at_quality("volumetric-fog", medium, |_| {}).pixels == fog_default.pixels);
    let fog_high = at_quality("volumetric-fog", high, |_| {});
    let fog_shadows = at_quality("volumetric-fog", shadows_only, |_| {});
    // The fog in the middle of the wood, between the pines' shafts.
    let r = (640, 120, 720, 520);
    assert!(
        report(
            "volumetric-fog: shadows only -> high",
            &fog_shadows,
            &fog_high,
            r
        ) > 0
    );
    report("volumetric-fog: medium -> high", &fog_default, &fog_high, r);
}
