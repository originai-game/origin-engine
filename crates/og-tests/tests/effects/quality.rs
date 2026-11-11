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
    assert_eq!(all.graph_passes, stats.graph_passes + 1 + 2 + 3 + 1 + 2);
    let _ = Curve::default();
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
    // The contacts under the larger sphere and the box beside it.
    let r = (600, 440, 760, 530);
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
    // The emissive slab's reflection in the floor.
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
    // The fog in front of the dark middle slot of the wall, between two
    // shafts.
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
