//! Grading, screen effects and depth-based post-processing frame tests.

use super::*;

/// A stationary marker above a moving card cannot be swept by horizontal
/// card motion. They are independent meshes/materials/instances, no parent.
#[test]
#[ignore = "requires GPU"]
fn motion_blur_does_not_borrow_a_neighbors_perpendicular_velocity() {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: -Vec3::Z,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 26.991467,
        },
        near: 0.1,
        far: 100.0,
        exposure: og_render::Exposure::linear(1.0),
    };
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.light.intensity = 0.0;
    scene.look.post.tonemap = Tonemap::None;
    let white = scene.add_standard_material(og_render::StandardMaterial::basic(Vec3::splat(0.8)));
    let red = scene.add_standard_material(og_render::StandardMaterial::basic(Vec3::new(
        0.8, 0.03, 0.01,
    )));
    let card = scene.add_mesh(shapes::cuboid(Vec3::new(0.25, 0.7, 0.002)));
    scene.add_instance(
        card,
        white,
        Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)),
    );
    let marker = scene.add_mesh(shapes::sphere(24, 16));
    scene.add_instance(
        marker,
        red,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.04),
            glam::Quat::IDENTITY,
            Vec3::new(0.0, 0.5, -5.0),
        ),
    );
    let desc = RenderDesc {
        antialias: og_render::Antialias::Off,
        ..small()
    };
    scene.look.post.motion_blur = Some(MotionBlur {
        shutter: 0.5,
        samples: 32,
        max_pixels: 64.0,
    });
    let mut renderer = Renderer::new_blocking(&desc).unwrap();
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    scene.objects.previous_transforms = scene
        .objects
        .instances
        .iter()
        .map(|i| i.transform)
        .collect();
    scene.objects.instances[0].transform = Mat4::from_translation(Vec3::new(0.15, 0.0, -5.0));
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let moving = renderer.read_offscreen().unwrap();
    scene.look.post.motion_blur.as_mut().unwrap().shutter = 0.0;
    let (sharp, _) = frame(&desc, &scene);
    save("og-effects", "motion-neighbor-sharp", &sharp);
    save("og-effects", "motion-neighbor-object", &moving);
    let marker_region = (152, 46, 168, 59);
    let change = |a: &RgbaImage, b: &RgbaImage, (x0, y0, x1, y1)| {
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .map(|(x, y)| px_clamped(a, x, y)[0].abs_diff(px_clamped(b, x, y)[0]))
            .max()
            .unwrap()
    };
    let marker_change = change(&moving, &sharp, marker_region);
    println!("stationary marker max red-channel change: {marker_change}");
    let peak_red = (marker_region.1..marker_region.3)
        .flat_map(|y| (marker_region.0..marker_region.2).map(move |x| (x, y)))
        .map(|(x, y)| px_clamped(&sharp, x, y)[0])
        .max()
        .unwrap();
    assert!(peak_red > 100, "marker must be visible");
    assert!(
        change(&moving, &sharp, (150, 75, 190, 105)) > 20,
        "moving card must still blur"
    );
    assert!(marker_change <= 2, "stationary marker borrowed card motion");
    // Negative control: moving the camera must blur the marker itself.
    scene.look.post.motion_blur.as_mut().unwrap().shutter = 0.5;
    scene.objects.previous_transforms.clear();
    scene.frame.camera.position.x = 0.15;
    scene.frame.camera.target.x = 0.15;
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pan = renderer.read_offscreen().unwrap();
    save("og-effects", "motion-neighbor-camera", &pan);
    scene.look.post.motion_blur.as_mut().unwrap().shutter = 0.0;
    let (pan_sharp, _) = frame(&desc, &scene);
    assert!(
        change(&pan, &pan_sharp, (140, 46, 163, 59)) > 20,
        "camera motion must still blur the marker"
    );
}

/// A thin highlight must streak continuously, not repeat as sixteen
/// separated copies. The source is at x=240; zoom samples x + (160-x)t/2,
/// so every pixel from 230 to 310 sees it strictly inside the tent.
#[test]
#[ignore = "requires GPU"]
fn zoom_blur_keeps_a_thin_highlight_continuous() {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    scene.add_sprite(Sprite {
        position: Vec3::new(80.0 / 18.0, 0.0, 0.0),
        size: Vec2::new(0.08, 10.0),
        color: [1.0; 4],
        ..Default::default()
    });
    scene.look.post.screen_blur = Some(og_render::ScreenBlur {
        strength: 0.5,
        ..Default::default()
    });
    let (image, _) = frame(&small(), &scene);
    for x in 230..310 {
        let brightness = region(&image, (x, 80, x + 1, 100));
        assert!(
            brightness > 5.0,
            "gap in the highlight at x={x}: {brightness}"
        );
    }
    assert_eq!(px_clamped(&image, 190, 90), [0; 3], "outside the streak");
    let (again, _) = frame(&small(), &scene);
    assert_eq!(
        image.pixels, again.pixels,
        "jitter is stable between frames"
    );
    scene.look.post.screen_blur.as_mut().unwrap().strength = 0.0;
    let (zero, _) = frame(&small(), &scene);
    scene.look.post.screen_blur = None;
    let (off, _) = frame(&small(), &scene);
    assert_eq!(
        zero.pixels, off.pixels,
        "zero strength leaves the frame alone"
    );
    // Strength is not capped at one: the footprint then crosses the
    // centre. A log-radius factorization alone would silently lose this.
    scene.look.post.screen_blur = Some(og_render::ScreenBlur {
        strength: 3.2,
        ..Default::default()
    });
    let (crossed, _) = frame(&small(), &scene);
    assert!(
        region(&crossed, (80, 80, 81, 100)) > 5.0,
        "the highlight crosses the centre"
    );
}

/// Lens distortion moves a marker on each axis of a 16:9 frame to where
/// its formula, solved here, puts it: out from the centre under a fitted
/// barrel and under a pincushion, by amounts that differ between the axes
/// because the radius is measured in pixels. With the centre moved, a
/// marker there stays put, the other moves by the fit to the farthest
/// corner, and no corner of the frame is black. The lens is one pass,
/// none at amount 0.
#[test]
#[ignore = "requires GPU"]
fn lens_distortion_bends_by_its_formula() {
    let (w, h) = (320.0f32, 180.0f32);
    let scene = |lens: Option<LensDistortion>| {
        let mut s = RenderScene::new();
        for (x, y) in [(-4.0, 0.0), (0.0, 2.5)] {
            s.add_sprite(Sprite {
                position: Vec3::new(x, y, 0.0),
                size: Vec2::splat(0.25),
                color: [1.0; 4],
                ..Default::default()
            });
        }
        s.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
        // Dark grey, so a black corner (beyond the image) shows.
        s.lighting.environment.background = Background::Color(Vec3::splat(0.05));
        s.look.post.tonemap = Tonemap::None;
        s.look.post.lens_distortion = lens;
        s
    };
    // The centroid of the white marker's pixels within 30 px of `near`, in
    // pixels (a pixel's centre at +0.5), weighted by how far each is above
    // the background (63 in sRGB).
    let centroid = |img: &RgbaImage, near: Vec2| {
        let (mut sum, mut n) = (Vec2::ZERO, 0.0);
        for y in (near.y as i32 - 30).max(0)..(near.y as i32 + 30).min(h as i32) {
            for x in (near.x as i32 - 30).max(0)..(near.x as i32 + 30).min(w as i32) {
                let k = (luma(&px_clamped(img, x as u32, y as u32)) - 63.0).max(0.0) / 192.0;
                if k > 0.05 {
                    sum += Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * k;
                    n += k;
                }
            }
        }
        assert!(n > 0.0, "nothing bright near {near}");
        sum / n
    };
    // Where image point `p` (pixels) shows: the t along the ray from the
    // centre with t (1 + amount t² r²) / (1 + max(amount, 0) R²) = 1, r²
    // being p's squared distance over the half-diagonal's and R² the
    // farthest corner's, by bisection.
    let shown_at = |l: LensDistortion, p: Vec2| {
        let c = l.center * Vec2::new(w, h);
        let half = 0.25 * (w * w + h * h);
        let r2 = (p - c).length_squared() / half;
        let far = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)]
            .map(|(x, y)| (Vec2::new(x, y) - c).length_squared() / half)
            .into_iter()
            .fold(0.0, f32::max);
        let (mut lo, mut hi) = (0.0f32, 2.0f32);
        for _ in 0..40 {
            let t = 0.5 * (lo + hi);
            if t * (1.0 + l.amount * t * t * r2) / (1.0 + l.amount.max(0.0) * far) < 1.0 {
                lo = t;
            } else {
                hi = t;
            }
        }
        c + (p - c) * (0.5 * (lo + hi))
    };
    let (plain, off) = frame(&small(), &scene(None));
    let marks = [
        centroid(&plain, Vec2::new(80.0, 90.0)),
        centroid(&plain, Vec2::new(160.0, 45.0)),
    ];
    let (_, zero) = frame(
        &small(),
        &scene(Some(LensDistortion {
            amount: 0.0,
            ..Default::default()
        })),
    );
    assert_eq!(zero.graph_passes, off.graph_passes, "a lens of amount 0");
    let cases = [
        ("barrel", 0.5, Vec2::splat(0.5)),
        ("pincushion", -0.5, Vec2::splat(0.5)),
        ("barrel at the upper mark", 0.5, marks[1] / Vec2::new(w, h)),
    ];
    for (name, amount, center) in cases {
        let lens = LensDistortion { amount, center };
        let (bent, stats) = frame(&small(), &scene(Some(lens)));
        assert_eq!(stats.graph_passes, off.graph_passes + 1, "{name}: one pass");
        let corners =
            [(0, 0), (319, 0), (0, 179), (319, 179)].map(|(x, y)| px_clamped(&bent, x, y));
        assert!(
            corners.iter().all(|&p| p == px_clamped(&plain, 0, 0)),
            "{name}: a corner is not the background: {corners:?}"
        );
        for &m in &marks {
            let want = shown_at(lens, m);
            let got = centroid(&bent, want);
            println!("{name}: the mark at {m} shows at {got}, the formula {want}");
            assert!(got.distance(want) < 1.0, "{name}: {m} at {got}, not {want}");
        }
    }
}

/// Local contrast: mean squared difference between horizontal neighbours
/// (a blur spreads an edge's step over more pixels, which lowers it).
fn sharpness(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 - 1 {
            sum += (luma(&px_clamped(img, x, y)) - luma(&px_clamped(img, x + 1, y))).powi(2);
            n += 1.0;
        }
    }
    sum / n
}

/// Depth of field keeps the focus band sharp and blurs the near ground; the
/// high quality gathers more taps (a different, smoother result).
#[test]
#[ignore = "requires GPU"]
fn depth_of_field_blurs_outside_the_focus_band() {
    let desc = scenes::base_desc();
    let low = fx::dof_scene(DofQuality::Low).expect("scene");
    let (blurred, _) = frame(&desc, &low);
    let mut off = fx::dof_scene(DofQuality::Low).expect("scene");
    off.look.post.depth_of_field = None;
    let (sharp, _) = frame(&desc, &off);
    let (high, _) = frame(&desc, &fx::dof_scene(DofQuality::High).expect("scene"));
    // The near ground (bottom rows) and the focused figure (z = 2 - 2.6 * 2... the 3rd).
    let near = (200, 650, 1080, 715);
    let (n_on, n_off) = (sharpness(&blurred, near), sharpness(&sharp, near));
    println!("near ground sharpness: dof {n_on}, off {n_off}");
    assert!(n_on < 0.5 * n_off);
    let focus = project(
        &low.frame.camera,
        1280,
        720,
        Vec3::new(-2.0 + 0.8 * 2.0, 1.0, 2.0 - 2.6 * 2.0),
    );
    // A light-to-dark edge wholly on the focused figure, found in this
    // frame rather than fixed in pixels (a fixed probe broke when the
    // figure's mesh was turned to face its bind pose). The figure's own
    // pixels: it drawn alone over a black and over a white background,
    // without DOF or bloom, where the two agree (its silhouette's blended
    // pixels differ and are left out). Of the 6-pixel horizontal runs
    // within 40 px of its projected centre whose every pixel and both
    // neighbours lie on it, the one falling most in the sharp frame: an
    // interior edge, so no defocused background is in the probe (the old
    // 60x60 gradient-energy ROI included floor that anisotropic filtering
    // legitimately sharpens; aec2aab37).
    //
    // On it means seen, too: the nearer, defocused figures overlap it
    // (facing the camera, the second one's arm crosses it), and an edge of
    // theirs over it is sharp only without DOF. So a pixel counts when
    // the figure is frontmost there (the whole scene, without DOF or
    // bloom, changes when the figure alone is flashed white) and so is
    // every pixel within MARGIN of it, beyond the largest blur radius,
    // so no defocused neighbour or background spreads into the probe.
    const MARGIN: u32 = 12;
    let focused = |i: &og_render::Instance| {
        let t = i.transform.w_axis;
        (t.x - (-2.0 + 0.8 * 2.0)).abs() < 1e-3 && (t.z - (2.0 - 2.6 * 2.0)).abs() < 1e-3
    };
    let plain = |s: &mut RenderScene| {
        s.look.post.depth_of_field = None;
        s.look.post.bloom = None;
    };
    let alone = |background: Vec3| {
        let mut s = fx::dof_scene(DofQuality::Low).expect("scene");
        s.objects.instances.retain(focused);
        plain(&mut s);
        s.lighting.environment.background = Background::Color(background);
        frame(&desc, &s).0
    };
    let (on_black, on_white) = (alone(Vec3::ZERO), alone(Vec3::ONE));
    let whole = |flash: bool| {
        let mut s = fx::dof_scene(DofQuality::Low).expect("scene");
        plain(&mut s);
        for i in s.objects.instances.iter_mut().filter(|i| focused(i)) {
            if flash {
                i.fx = og_render::InstanceFx {
                    flash: 1.0,
                    flash_color: Vec3::ONE,
                    ..og_render::InstanceFx::NONE
                };
            }
        }
        frame(&desc, &s).0
    };
    let (flashed, unflashed) = (whole(true), whole(false));
    let seen = |x: u32, y: u32| {
        let (a, b) = (px_clamped(&on_black, x, y), px_clamped(&on_white, x, y));
        (0..3).all(|c| a[c].abs_diff(b[c]) <= 2)
            && luma(&px_clamped(&flashed, x, y)) - luma(&px_clamped(&unflashed, x, y)) > 30.0
    };
    // A summed-area table of the pixels not seen around the search area,
    // so a window's count is four reads.
    let reach = 40 + 6 + MARGIN;
    let (x0, y0) = (focus.0.saturating_sub(reach), focus.1.saturating_sub(reach));
    let (w, h) = (2 * reach + 1, 2 * reach + 1);
    let mut table = vec![0u32; ((w + 1) * (h + 1)) as usize];
    for j in 0..h {
        for i in 0..w {
            let miss = !seen(x0 + i, y0 + j) as u32;
            let k = |i: u32, j: u32| (j * (w + 1) + i) as usize;
            table[k(i + 1, j + 1)] =
                miss + table[k(i, j + 1)] + table[k(i + 1, j)] - table[k(i, j)];
        }
    }
    // Whether every pixel of the box (inclusive) is seen.
    let all_seen = |(ax, ay): (u32, u32), (bx, by): (u32, u32)| {
        let k = |x: u32, y: u32| ((y - y0) * (w + 1) + (x - x0)) as usize;
        let (ax, ay, bx, by) = (ax - MARGIN, ay - MARGIN, bx + MARGIN + 1, by + MARGIN + 1);
        table[k(bx, by)] + table[k(ax, ay)] - table[k(ax, by)] - table[k(bx, ay)] == 0
    };
    // (x, y, step): six pixels from x along step (+1 right, -1 left).
    let at = |(x, y, step): (u32, u32, i32), i: u32| ((x as i32 + step * i as i32) as u32, y);
    let mut best: Option<((u32, u32, i32), f32)> = None;
    for y in focus.1.saturating_sub(40)..focus.1 + 40 {
        for x in focus.0.saturating_sub(40)..focus.0 + 40 {
            for step in [1i32, -1] {
                let run = (x, y, step);
                let (first, last) = (at(run, 0), at(run, 5));
                let (lo, hi) = ((first.0.min(last.0), y), (first.0.max(last.0), y));
                if !all_seen(lo, hi) {
                    continue;
                }
                let fall = luma(&px_clamped(&sharp, first.0, first.1))
                    - luma(&px_clamped(&sharp, last.0, last.1));
                if best.is_none_or(|(_, f)| fall > f) {
                    best = Some((run, fall));
                }
            }
        }
    }
    let (run, fall) =
        best.expect("the focused figure shows clear of its neighbours near its centre");
    println!("focused edge: run {run:?}, fall {fall} in the sharp frame");
    let profile = |img: &RgbaImage| -> [f32; 6] {
        std::array::from_fn(|i| {
            let (x, y) = at(run, i as u32);
            luma(&px_clamped(img, x, y))
        })
    };
    let reference = profile(&sharp);
    let dark = reference.iter().copied().fold(f32::INFINITY, f32::min);
    let light = reference.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        light - dark > 80.0,
        "the focused edge must have measurable contrast ({reference:?})"
    );
    // Use the sharp frame's levels for every image: renormalizing a blurred
    // profile would hide contrast loss. Missing crossings mean lost detail.
    let width = |img: &RgbaImage| {
        let p = profile(img);
        let crossing = |fraction: f32| {
            let level = dark + fraction * (light - dark);
            p.windows(2).enumerate().find_map(|(i, v)| {
                (v[0] >= level && v[1] < level).then(|| i as f32 + (v[0] - level) / (v[0] - v[1]))
            })
        };
        crossing(0.9)
            .zip(crossing(0.1))
            .map_or(f32::INFINITY, |(a, b)| b - a)
    };
    let (w_on, w_off) = (width(&blurred), width(&sharp));
    println!("focused edge 10–90% width: dof {w_on}, off {w_off}");
    assert!(w_off > 0.0 && w_off.is_finite());
    let retains_focus = |w: f32| w > 0.0 && w_off / w > 0.7;
    assert!(retains_focus(w_on), "DOF broadened the focused edge");
    // A deliberately misplaced focus plane must fail the same retention
    // requirement, rather than merely producing a different image.
    let mut defocused = low;
    defocused
        .look
        .post
        .depth_of_field
        .as_mut()
        .unwrap()
        .focus_distance = 3.0;
    let (defocused, _) = frame(&desc, &defocused);
    let w_wrong = width(&defocused);
    println!("deliberately defocused edge width: {w_wrong}");
    assert!(
        !retains_focus(w_wrong),
        "the focus check accepted a blurred figure"
    );
    assert!(
        differing(&blurred, &high, 3.0) > 1_000,
        "the quality levels draw the same"
    );
}
