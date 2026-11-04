//! Grading, screen effects and depth-based post-processing frame tests.

use super::*;

// ---------------------------------------------------------------------------
// The grading stack and screen effects.
// ---------------------------------------------------------------------------

fn colour_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let q = |c: [f32; 4], x: f32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        size: Vec2::new(1.5, 4.0),
        color: c,
        ..Default::default()
    };
    scene.add_sprite(q([0.8, 0.1, 0.05, 1.0], -3.0));
    scene.add_sprite(q([0.1, 0.6, 0.1, 1.0], -1.0));
    scene.add_sprite(q([0.05, 0.1, 0.8, 1.0], 1.0));
    scene.add_sprite(q([0.5, 0.5, 0.5, 1.0], 3.0));
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 5.0);
    scene.environment.background = Background::Color(Vec3::splat(0.2));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The identity LUT changes nothing beyond 8-bit rounding; a red/blue swap
/// LUT turns the red bar blue (so the table's axes are read right).
#[test]
fn colour_grading_reads_the_lut() {
    let (plain, _) = frame(&small(), &colour_scene());
    let mut id = colour_scene();
    let lut = id.add_lut(ColorLut::identity());
    id.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    let (graded, _) = frame(&small(), &id);
    let worst = plain
        .pixels
        .iter()
        .zip(&graded.pixels)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    println!("identity LUT: largest channel change {worst}");
    assert!(worst <= 3, "identity LUT changed a channel by {worst}");
    let mut swap = colour_scene();
    let lut = swap.add_lut(ColorLut::from_fn(|c| Vec3::new(c.z, c.y, c.x)));
    swap.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    let (swapped, _) = frame(&small(), &swap);
    let cam = colour_scene().camera;
    let (x, y) = project(&cam, 320, 180, Vec3::new(-3.0, 0.0, 0.0));
    let (before, after) = (px(&plain, x, y), px(&swapped, x, y));
    println!("red bar {before:?} -> {after:?}");
    assert!(before[0] > before[2] + 100 && after[2] > after[0] + 100);
    // The .cube parser: an identity cube resampled to 32³ is still identity.
    let mut cube = String::from("LUT_3D_SIZE 2\n");
    for b in 0..2 {
        for g in 0..2 {
            for r in 0..2 {
                cube.push_str(&format!("{r} {g} {b}\n"));
            }
        }
    }
    assert_eq!(
        ColorLut::from_cube(&cube).expect("cube"),
        ColorLut::identity()
    );
}

/// Colour adjustments: saturation -1 greys the red bar at its luma, the
/// hue turns red to green, and after a red/blue swap LUT the same turn
/// brings the swapped (blue) bar back to red, so the LUT is applied first
/// and both are in the one table. A renderer that changes the adjustments
/// between frames draws what a fresh one draws (the table is baked again
/// in place). Warm white balance raises red over blue, cool the reverse,
/// and positive tint lowers green.
#[test]
fn colour_adjustments_bake_after_the_lut() {
    let cam = colour_scene().camera;
    let (x, y) = project(&cam, 320, 180, Vec3::new(-3.0, 0.0, 0.0));
    let adjusted = |a: ColorAdjust, swap: bool| {
        let mut s = colour_scene();
        if swap {
            let lut = s.add_lut(ColorLut::from_fn(|c| Vec3::new(c.z, c.y, c.x)));
            s.post.color_grading = Some(ColorGrading {
                lut,
                intensity: 1.0,
            });
        }
        s.post.color_adjust = Some(a);
        s
    };
    let (plain, _) = frame(&small(), &colour_scene());
    let grey = ColorAdjust {
        saturation: -1.0,
        ..Default::default()
    };
    let (greyed, _) = frame(&small(), &adjusted(grey, false));
    let (before, after) = (px(&plain, x, y), px(&greyed, x, y));
    println!("saturation -1: red bar {before:?} -> {after:?}");
    assert!(after.iter().max().unwrap() - after.iter().min().unwrap() <= 3);
    assert!((luma(after) - luma(before)).abs() < 3.0);
    let turn = ColorAdjust {
        hue: 120.0,
        ..Default::default()
    };
    let (turned, _) = frame(&small(), &adjusted(turn, false));
    let (swapped, _) = frame(&small(), &adjusted(turn, true));
    let (t, s) = (px(&turned, x, y), px(&swapped, x, y));
    println!("hue 120: red bar {t:?}; after the swap LUT {s:?}");
    assert!(t[1] > t[0] + 100 && t[1] > t[2] + 100, "hue alone: {t:?}");
    assert!(
        s[0] > s[1] + 100 && s[0] > s[2] + 100,
        "swap then hue: {s:?}"
    );
    // The same renderer and scene, the adjustments changed: the table is
    // baked again.
    let mut renderer = Renderer::new_blocking(&small()).expect("renderer");
    let mut scene = adjusted(grey, true);
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    scene.post.color_adjust = Some(turn);
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let again = renderer.read_offscreen().expect("capture");
    assert_eq!(again.pixels, swapped.pixels, "a stale table after a change");
    // White balance on a mid grey.
    let wb = |temperature: f32, tint: f32| {
        ColorAdjust {
            temperature,
            tint,
            ..Default::default()
        }
        .apply(Vec3::splat(0.5))
    };
    let (warm, cool, magenta) = (wb(0.5, 0.0), wb(-0.5, 0.0), wb(0.0, 0.5));
    println!("grey 0.5: warm {warm}, cool {cool}, tint +0.5 {magenta}");
    assert!(warm.x > warm.z + 0.05 && cool.z > cool.x + 0.05);
    assert!(magenta.y + 0.03 < magenta.x.min(magenta.z));
    assert_eq!(
        ColorAdjust::default().apply(Vec3::splat(0.5)),
        Vec3::splat(0.5)
    );
}

/// Khronos PBR Neutral against the curve computed here from its
/// definition: the colour bars, whose peaks are under the knee once a
/// dielectric's 0.04 is off (less near black), and an over-bright bar,
/// rolled off and mixed towards white; through the frame's tonemap and
/// through a view's resolve drawn over the whole frame.
#[test]
fn neutral_tone_curve_follows_khronos() {
    // KhronosGroup/ToneMapping, PBR Neutral.
    let neutral = |c: Vec3| {
        let (start_compression, desaturation) = (0.8 - 0.04, 0.15);
        let x = c.min_element();
        let c = c - if x < 0.08 { x - 6.25 * x * x } else { 0.04 };
        let peak = c.max_element();
        if peak < start_compression {
            return c;
        }
        let d = 1.0 - start_compression;
        let new_peak = 1.0 - d * d / (peak + d - start_compression);
        let g = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
        (c * (new_peak / peak)).lerp(Vec3::splat(new_peak), g)
    };
    let encode = |v: f32| {
        let e = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (e * 255.0).round() as i32
    };
    let bars = [
        (Vec3::new(0.8, 0.1, 0.05), -3.0),
        (Vec3::new(0.1, 0.6, 0.1), -1.0),
        (Vec3::new(0.05, 0.1, 0.8), 1.0),
        (Vec3::new(0.5, 0.5, 0.5), 3.0),
        (Vec3::new(4.0, 2.0, 1.0), 5.0),
    ];
    for through_view in [false, true] {
        let mut scene = colour_scene();
        scene.add_sprite(Sprite {
            position: Vec3::new(5.0, 0.0, 0.0),
            size: Vec2::new(1.5, 4.0),
            color: [4.0, 2.0, 1.0, 1.0],
            ..Default::default()
        });
        scene.post.tonemap = Tonemap::Neutral;
        if through_view {
            scene.views.push(View {
                camera: scene.camera,
                target: ViewTarget::Viewport([0.0, 0.0, 1.0, 1.0]),
                ..Default::default()
            });
        }
        let (img, _) = frame(&small(), &scene);
        for (c, x) in bars {
            let (px_x, px_y) = project(&scene.camera, 320, 180, Vec3::new(x, 0.0, 0.0));
            let got = px(&img, px_x, px_y);
            let n = neutral(c);
            let want = [encode(n.x), encode(n.y), encode(n.z)];
            println!("neutral (view {through_view}): {c} -> {got:?}, the curve {want:?}");
            assert!(
                got.iter()
                    .zip(want)
                    .all(|(g, w)| (*g as i32 - w).abs() <= 2),
                "neutral (view {through_view}) of {c}: {got:?} against {want:?}"
            );
        }
    }
}

/// Lens distortion moves a marker on each axis of a 16:9 frame to where
/// its formula, solved here, puts it: out from the centre under a fitted
/// barrel and under a pincushion, by amounts that differ between the axes
/// because the radius is measured in pixels. With the centre moved, a
/// marker there stays put, the other moves by the fit to the farthest
/// corner, and no corner of the frame is black. The lens is one pass,
/// none at amount 0.
#[test]
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
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
        // Dark grey, so a black corner (beyond the image) shows.
        s.environment.background = Background::Color(Vec3::splat(0.05));
        s.post.tonemap = Tonemap::None;
        s.post.lens_distortion = lens;
        s
    };
    // The centroid of the white marker's pixels within 30 px of `near`, in
    // pixels (a pixel's centre at +0.5), weighted by how far each is above
    // the background (63 in sRGB).
    let centroid = |img: &RgbaImage, near: Vec2| {
        let (mut sum, mut n) = (Vec2::ZERO, 0.0);
        for y in (near.y as i32 - 30).max(0)..(near.y as i32 + 30).min(h as i32) {
            for x in (near.x as i32 - 30).max(0)..(near.x as i32 + 30).min(w as i32) {
                let k = (luma(px(img, x as u32, y as u32)) - 63.0).max(0.0) / 192.0;
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
        let corners = [(0, 0), (319, 0), (0, 179), (319, 179)].map(|(x, y)| px(&bent, x, y));
        assert!(
            corners.iter().all(|&p| p == px(&plain, 0, 0)),
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

/// The screen blur takes its taps where its formula puts them: with four
/// taps (t = ±0.25, ±0.75) a white dot on a dark frame shows as four
/// copies, the one of tap t where that tap of a pixel lands on the dot,
/// each within 1 px of the formula solved here. Zoom: q + (c − q) s t = P
/// (towards an off-centre centre); spin: P turned about the centre by
/// −a t in pixels, a circle on the 16:9 frame (turned in uv it is an
/// ellipse, 3 px and more off); directional at 30° counter-clockwise from
/// the right, 160 px at 720 lines (40 px here), so the copies run down to
/// the left of the dot and up to its right (a flipped y runs them the
/// other way). One pass while on, none at strength 0, and no prepass for
/// it.
#[test]
fn screen_blur_taps_where_its_formula_says() {
    let (w, h) = (320.0f32, 180.0f32);
    let scene = |blur: Option<ScreenBlur>| {
        let mut s = RenderScene::new();
        s.add_sprite(Sprite {
            position: Vec3::new(4.0, 1.5, 0.0),
            size: Vec2::splat(0.25),
            color: [1.0; 4],
            ..Default::default()
        });
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
        s.environment.background = Background::Color(Vec3::splat(0.05));
        s.post.tonemap = Tonemap::None;
        s.post.screen_blur = blur;
        s
    };
    // The centroid of the bright pixels within 5 px of `near`, weighted
    // by how far each is above the background (63 in sRGB).
    let centroid = |img: &RgbaImage, near: Vec2| {
        let (mut sum, mut n) = (Vec2::ZERO, 0.0);
        for y in (near.y as i32 - 5).max(0)..(near.y as i32 + 6).min(h as i32) {
            for x in (near.x as i32 - 5).max(0)..(near.x as i32 + 6).min(w as i32) {
                let at = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let k = (luma(px(img, x as u32, y as u32)) - 63.0).max(0.0);
                if k > 4.0 && at.distance(near) <= 5.0 {
                    sum += at * k;
                    n += k;
                }
            }
        }
        (n > 0.0).then(|| sum / n)
    };
    let (plain, off) = frame(&small(), &scene(None));
    let dot = centroid(&plain, Vec2::new(232.0, 63.0)).expect("the dot");
    let (_, zero) = frame(
        &small(),
        &scene(Some(ScreenBlur {
            strength: 0.0,
            ..Default::default()
        })),
    );
    assert_eq!(zero.graph_passes, off.graph_passes, "a blur of strength 0");
    let size = Vec2::new(w, h);
    let turn =
        |v: Vec2, a: f32| Vec2::new(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos());
    let cases = [
        (
            "zoom",
            ScreenBlur {
                kind: ScreenBlurKind::Zoom,
                strength: 0.4,
                center: Vec2::new(0.3, 0.6),
                samples: 4,
                ..Default::default()
            },
        ),
        (
            "spin",
            ScreenBlur {
                kind: ScreenBlurKind::Spin,
                strength: 0.3,
                center: Vec2::new(0.4, 0.55),
                samples: 4,
                ..Default::default()
            },
        ),
        (
            "directional",
            ScreenBlur {
                kind: ScreenBlurKind::Directional,
                strength: 160.0,
                angle: 30f32.to_radians(),
                samples: 4,
                ..Default::default()
            },
        ),
    ];
    for (name, blur) in cases {
        let (img, stats) = frame(&small(), &scene(Some(blur)));
        assert_eq!(stats.graph_passes, off.graph_passes + 1, "{name}: one pass");
        let c = blur.center * size;
        for t in [-0.75f32, -0.25, 0.25, 0.75] {
            let want = match blur.kind {
                ScreenBlurKind::Zoom => (dot - c * blur.strength * t) / (1.0 - blur.strength * t),
                ScreenBlurKind::Spin => c + turn(dot - c, -blur.strength * t),
                ScreenBlurKind::Directional => {
                    let d =
                        Vec2::new(blur.angle.cos(), -blur.angle.sin()) * blur.strength * h / 720.0;
                    dot - d * t
                }
            };
            let got = centroid(&img, want);
            println!("{name}: tap {t} shows the dot at {got:?}, the formula {want}");
            assert!(
                got.is_some_and(|g| g.distance(want) < 1.0),
                "{name}: tap {t}'s copy at {got:?}, not {want}"
            );
        }
    }
}

/// Vignette darkens the corners, not the centre; chromatic aberration
/// fringes edges near the corners more than at the centre; grain is the
/// same for a tick and changes with it; a full flash is the flash colour.
#[test]
fn vignette_aberration_grain_and_flash() {
    let mut base = colour_scene();
    base.environment.background = Background::Color(Vec3::splat(0.5));
    let (plain, _) = frame(&small(), &base);
    let mut v = colour_scene();
    v.environment.background = Background::Color(Vec3::splat(0.5));
    v.post.vignette = Some(Vignette {
        intensity: 0.8,
        ..Default::default()
    });
    let (vig, _) = frame(&small(), &v);
    assert!(luma(px(&vig, 2, 2)) < 0.6 * luma(px(&plain, 2, 2)));
    assert!((luma(px(&vig, 160, 90)) - luma(px(&plain, 160, 90))).abs() < 3.0);

    let mut ca = base_scene_with(|s| {
        s.post.chromatic_aberration = Some(ChromaticAberration { intensity: 6.0 })
    });
    let (fringed, _) = frame(&small(), &ca);
    let diff_at = |x0: u32, x1: u32| {
        let mut d = 0u32;
        for y in 40..140 {
            for x in x0..x1 {
                let (a, b) = (px(&plain, x, y), px(&fringed, x, y));
                d += a[0].abs_diff(b[0]) as u32 + a[2].abs_diff(b[2]) as u32;
            }
        }
        d
    };
    let (edge, centre) = (diff_at(20, 60), diff_at(140, 180));
    println!("aberration: near the edge {edge}, at the centre {centre}");
    assert!(edge > 3 * centre.max(1));
    ca.post.chromatic_aberration = None;

    let grainy = |tick: u64| {
        let mut s = base_scene_with(|s| {
            s.post.film_grain = Some(FilmGrain {
                intensity: 0.1,
                size: 1.0,
            })
        });
        s.tick = tick;
        frame(&small(), &s).0
    };
    let (g1, g1b, g2) = (grainy(1), grainy(1), grainy(2));
    assert_eq!(g1.pixels, g1b.pixels, "grain is a function of the tick");
    assert!(
        differing(&g1, &g2, 2.0) > 10_000,
        "grain did not change with the tick"
    );
    assert!(differing(&g1, &plain, 2.0) > 10_000);

    let mut f = colour_scene();
    f.post.flash = Some(ScreenFlash {
        color: Vec3::new(1.0, 0.0, 0.0),
        amount: 1.0,
    });
    let (flashed, _) = frame(&small(), &f);
    assert!(
        flashed
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[0] == 255 && p[1] == 0 && p[2] == 0)
    );
}

fn base_scene_with(f: impl Fn(&mut RenderScene)) -> RenderScene {
    let mut s = colour_scene();
    s.environment.background = Background::Color(Vec3::splat(0.5));
    f(&mut s);
    s
}

/// Camera shake moves the view (not the image): the frame shifts, and a
/// zero shake is the unshaken frame; trauma shake is deterministic per tick.
#[test]
fn camera_shake_moves_the_view() {
    let scene = colour_scene();
    let (still, _) = frame(&small(), &scene);
    let mut shaken = colour_scene();
    shaken.shake = CameraShake {
        offset: Vec2::new(0.05, 0.0),
        roll_degrees: 0.0,
    };
    let (moved, _) = frame(&small(), &shaken);
    // 5% of the view height (5 units) is 0.25 units: about 9 pixels.
    let edge = |img: &RgbaImage| {
        (0..320)
            .find(|&x| px(img, x, 90)[0] > 150)
            .expect("red bar")
    };
    let (a, b) = (edge(&still), edge(&moved));
    println!("red bar's left edge: {a} -> {b}");
    assert!((7..=11).contains(&(a as i32 - b as i32)));
    shaken.shake = CameraShake::default();
    assert_eq!(frame(&small(), &shaken).0.pixels, still.pixels);
    let s = |t| CameraShake::trauma(0.8, t, 60.0, 9, 0.05, 3.0, 10.0);
    assert_eq!(s(17), s(17));
    assert_ne!(s(17), s(18));
    assert_eq!(
        CameraShake::trauma(0.0, 5, 60.0, 9, 0.05, 3.0, 10.0),
        CameraShake::default()
    );
}

// ---------------------------------------------------------------------------
// Ambient occlusion, depth of field, motion blur, god rays.
// ---------------------------------------------------------------------------

/// Both AO methods darken the contact under a sphere and the wall corner
/// while open ground barely changes.
#[test]
fn ambient_occlusion_darkens_contacts() {
    let desc = scenes::base_desc();
    let mut off = fx::ao_scene(AoMethod::Gtao);
    off.post.ao = None;
    let (plain, plain_stats) = frame(&desc, &off);
    for method in [AoMethod::Gtao, AoMethod::Ssao] {
        let scene = fx::ao_scene(method);
        let (img, stats) = frame(&desc, &scene);
        save(&format!("ao-{method:?}"), &img);
        assert!(stats.prepass_draws > 0 && stats.graph_passes >= plain_stats.graph_passes + 3);
        let contact = project(&scene.camera, 1280, 720, Vec3::new(0.3 + 0.52, 0.03, 0.2));
        let open = project(&scene.camera, 1280, 720, Vec3::new(1.5, 0.0, 2.0));
        let c = (
            luma(px(&img, contact.0, contact.1)),
            luma(px(&plain, contact.0, contact.1)),
        );
        let o = (
            luma(px(&img, open.0, open.1)),
            luma(px(&plain, open.0, open.1)),
        );
        println!(
            "{method:?}: contact {} vs {}, open {} vs {}",
            c.0, c.1, o.0, o.1
        );
        assert!(c.0 < c.1 - 6.0, "{method:?} did not darken the contact");
        assert!(
            (o.0 - o.1).abs() < 0.04 * o.1 + 2.0,
            "{method:?} darkened open ground"
        );
    }
}

/// Local contrast: mean squared difference between horizontal neighbours
/// (a blur spreads an edge's step over more pixels, which lowers it).
fn sharpness(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in r.1..r.3 {
        for x in r.0..r.2 - 1 {
            sum += (luma(px(img, x, y)) - luma(px(img, x + 1, y))).powi(2);
            n += 1.0;
        }
    }
    sum / n
}

/// Depth of field keeps the focus band sharp and blurs the near ground; the
/// high quality gathers more taps (a different, smoother result).
#[test]
fn depth_of_field_blurs_outside_the_focus_band() {
    let desc = scenes::base_desc();
    let low = fx::dof_scene(DofQuality::Low).expect("scene");
    let (blurred, _) = frame(&desc, &low);
    let mut off = fx::dof_scene(DofQuality::Low).expect("scene");
    off.post.depth_of_field = None;
    let (sharp, _) = frame(&desc, &off);
    let (high, _) = frame(&desc, &fx::dof_scene(DofQuality::High).expect("scene"));
    // The near ground (bottom rows) and the focused figure (z = 2 - 2.6 * 2... the 3rd).
    let near = (200, 650, 1080, 715);
    let (n_on, n_off) = (sharpness(&blurred, near), sharpness(&sharp, near));
    println!("near ground sharpness: dof {n_on}, off {n_off}");
    assert!(n_on < 0.5 * n_off);
    let focus = project(
        &low.camera,
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
        s.post.depth_of_field = None;
        s.post.bloom = None;
    };
    let alone = |background: Vec3| {
        let mut s = fx::dof_scene(DofQuality::Low).expect("scene");
        s.instances.retain(focused);
        plain(&mut s);
        s.environment.background = Background::Color(background);
        frame(&desc, &s).0
    };
    let (on_black, on_white) = (alone(Vec3::ZERO), alone(Vec3::ONE));
    let whole = |flash: bool| {
        let mut s = fx::dof_scene(DofQuality::Low).expect("scene");
        plain(&mut s);
        for i in s.instances.iter_mut().filter(|i| focused(i)) {
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
        let (a, b) = (px(&on_black, x, y), px(&on_white, x, y));
        (0..3).all(|c| a[c].abs_diff(b[c]) <= 2)
            && luma(px(&flashed, x, y)) - luma(px(&unflashed, x, y)) > 30.0
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
                let fall = luma(px(&sharp, first.0, first.1)) - luma(px(&sharp, last.0, last.1));
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
            luma(px(img, x, y))
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

/// With the previous transforms equal to the current ones and a still
/// camera, motion blur changes nothing; a moving object smears.
#[test]
fn motion_blur_follows_object_and_camera_motion() {
    let desc = small();
    let mut still = fx::motion_blur_scene();
    still.camera = Camera {
        position: Vec3::new(0.0, 2.4, 6.0),
        target: Vec3::new(0.0, 0.8, -1.0),
        ..Default::default()
    };
    still.previous_transforms = still.instances.iter().map(|i| i.transform).collect();
    let (a, _, _) = run(&desc, &mut still, 2, |_, _| {});
    let mut none = fx::motion_blur_scene();
    none.camera = still.camera;
    none.post.motion_blur = None;
    let (b, _, _) = run(&desc, &mut none, 2, |_, _| {});
    assert_eq!(a.pixels, b.pixels, "static motion blur altered the frame");
    let mut moving = fx::motion_blur_scene();
    let (m, _, _) = run(&desc, &mut moving, 4, fx::move_blur);
    let mut moving_off = fx::motion_blur_scene();
    moving_off.post.motion_blur = None;
    let (m0, _, _) = run(&desc, &mut moving_off, 4, fx::move_blur);
    save("motion-blur", &m);
    let whole = (0, 0, 320, 180);
    let (s1, s0) = (sharpness(&m, whole), sharpness(&m0, whole));
    println!("sharpness with motion blur {s1}, without {s0}");
    assert!(s1 < 0.8 * s0);
}

/// Every effect pass at once under TAA (depth of field, motion blur, god
/// rays, ambient occlusion, outlines): the render graph must not alias a
/// target a pass writes with the prepass textures it binds (it did: depth
/// of field, motion blur and TAA together failed wgpu's validation, and
/// motion blur after depth of field drew nothing), and motion blur still
/// moves the frame behind depth of field.
#[test]
fn every_effect_pass_together_under_taa() {
    let desc = RenderDesc {
        antialias: og_render::Antialias::Taa,
        ..small()
    };
    let all = |mb: bool| {
        let mut s = fx::motion_blur_scene();
        s.post.depth_of_field = Some(DepthOfField {
            focus_distance: 4.0,
            ..Default::default()
        });
        s.post.ao = Some(AmbientOcclusion::default());
        s.post.outline = Some(Outline::default());
        s.post.god_rays = Some(GodRays::default());
        if !mb {
            s.post.motion_blur = None;
        }
        let (img, _, _) = run(&desc, &mut s, 4, fx::move_blur);
        img
    };
    let (with, without) = (all(true), all(false));
    let moved = differing(&with, &without, 8.0);
    println!("motion blur behind depth of field changed {moved} pixels");
    assert!(
        moved > 500,
        "motion blur after depth of field changed {moved} pixels"
    );
}

/// The lens flare at full strength and without the starburst, so a ghost's
/// brightness reads as its source's visible share.
fn flare_probe(scene: &mut RenderScene) {
    scene.post.lens_flare = Some(LensFlare {
        intensity: 1.0,
        starburst: false,
        ..Default::default()
    });
}

/// The mean luma the flare adds (`on` less `off`) in a square of 13 pixels
/// round the mirror of the pixel `source` projects to through the frame's
/// centre: where its first ghost lands (Filament samples the ghosts from
/// `1 - uv`).
fn ghost_gain(on: &RgbaImage, off: &RgbaImage, cam: &Camera, source: Vec3) -> f32 {
    let (w, h) = (on.width, on.height);
    let (x, y) = project(cam, w, h, source);
    let (mx, my) = (w - 1 - x.min(w - 1), h - 1 - y.min(h - 1));
    let r = (
        mx.saturating_sub(6),
        my.saturating_sub(6),
        (mx + 7).min(w),
        (my + 7).min(h),
    );
    region(on, r) - region(off, r)
}

/// A frame of the scene `build` makes with the probe's flare and one
/// without any.
fn flare_pair(desc: &RenderDesc, build: impl Fn() -> RenderScene) -> (RgbaImage, RgbaImage) {
    let mut on = build();
    flare_probe(&mut on);
    let mut off = build();
    off.post.lens_flare = None;
    (frame(desc, &on).0, frame(desc, &off).0)
}

/// The sun's flare follows how much of its disc the depth shows: sinking
/// behind a wall's top edge, its ghost fades monotonically from full to
/// nothing, half of it at the edge. Without the occlusion (a ghost of the
/// sun's direction whatever stands in front, or the sun's sky pixels
/// ghosting with the wall over them clamped at full) the hidden sun would
/// still flare.
#[test]
fn lens_flare_fades_as_the_sun_sinks_behind_a_wall() {
    let desc = small();
    let gains: Vec<f32> = [0.36f32, 0.32, 0.30, 0.28, 0.24]
        .into_iter()
        .map(|elevation| {
            let scene = fx::flare_edge_scene(elevation);
            let (on, off) = flare_pair(&desc, || fx::flare_edge_scene(elevation));
            let sun = scene.camera.position
                + Vec3::new(0.0, elevation, -1.0).normalize() * scene.camera.far * 0.5;
            ghost_gain(&on, &off, &scene.camera, sun)
        })
        .collect();
    println!("sun ghost gain, sinking behind the wall: {gains:?}");
    assert!(gains[0] > 6.0, "the visible sun flares: {gains:?}");
    for pair in gains.windows(2) {
        assert!(
            pair[1] <= pair[0] + 0.5,
            "the flare fades monotonically: {gains:?}"
        );
    }
    assert!(
        gains[2] < 0.8 * gains[0] && gains[2] > gains[4] + 1.0,
        "half the disc, part of the flare: {gains:?}"
    );
    assert!(gains[4].abs() < 0.5, "the hidden sun casts none: {gains:?}");
}

/// The edge scene's visible sun over the procedural sky under a layer of
/// cloud of `coverage`, lit as every clouded scene is: so thin a layer
/// that its mean cover (`Clouds::light`) dims the scene's sun by a tenth,
/// while the sky draws it closing over the disc.
fn clouded_sun_scene(coverage: f32) -> RenderScene {
    let mut scene = fx::flare_edge_scene(0.36);
    let sky = &scene.environment.sky;
    scene.environment.background = Background::Procedural {
        sun_direction: sky.sun_direction,
        sun_color: sky.sun_color,
    };
    let clouds = og_render::Clouds {
        coverage,
        density: 0.004,
        layer: 0.0,
        detail: 0.0,
        wind: Vec3::ZERO,
        ..Default::default()
    };
    clouds.light_scene(&mut scene);
    scene.clouds = Some(clouds);
    scene
}

/// A sun behind cloud flares by what the cloud lets through as the sky
/// draws it, not by the cover's mean dimming of the scene's light: under a
/// layer the sky draws closed over the disc its ghost all but goes, though
/// the light it dims is nine tenths of the clear one's; and with no cloud
/// drawn over it (no cover) the ghost is the clear sky's.
#[test]
fn lens_flare_dims_through_the_clouds_over_the_sun() {
    let desc = small();
    let gain = |coverage: Option<f32>| {
        let build = || match coverage {
            Some(c) => clouded_sun_scene(c),
            None => {
                let mut s = clouded_sun_scene(0.0);
                s.clouds = None;
                s
            }
        };
        let scene = build();
        let (on, off) = flare_pair(&desc, build);
        let sun =
            scene.camera.position + Vec3::new(0.0, 0.36, -1.0).normalize() * scene.camera.far * 0.5;
        ghost_gain(&on, &off, &scene.camera, sun)
    };
    let (clear, open, closed) = (gain(None), gain(Some(0.0)), gain(Some(1.0)));
    let dim = og_render::Clouds {
        coverage: 1.0,
        density: 0.004,
        ..Default::default()
    }
    .light()
    .0;
    println!(
        "sun ghost gain: clear {clear}, no cover {open}, behind cloud {closed} (the light's dimming {dim})"
    );
    assert!(
        dim > 0.85,
        "the fixture's cover barely dims the light: {dim}"
    );
    assert!(clear > 6.0, "the clear sun flares: {clear}");
    assert!(
        (open - clear).abs() < 0.1 * clear,
        "no cloud drawn over the sun leaves its flare: {open} against {clear}"
    );
    assert!(
        closed < 0.1 * clear,
        "the sun behind cloud all but stops flaring: {closed}"
    );
}

/// A lamp at `x` behind a pillar whose right edge stands on x = 0 at
/// z = -4, the light 14 m from the eye and the pillar 8 m.
fn pillar_lamp_at(x: f32) -> Vec3 {
    Vec3::new(x, 2.4, -10.0)
}

fn pillar_lamp_scene(x: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    fx::terrain(&mut scene, 60.0);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 6.0, 0.4)));
    let stone = scene.add_material(Material {
        base_color: [0.5, 0.48, 0.45, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    // Its right edge on x = 0 at z = -4.
    scene.add_instance(
        pillar,
        stone,
        Mat4::from_translation(Vec3::new(-0.5, 3.0, -4.0)),
    );
    scene.lights.push(og_render::Light {
        position: pillar_lamp_at(x),
        intensity: 30.0,
        range: 30.0,
        ..Default::default()
    });
    scene.light.intensity = 0.05;
    scene.environment.intensity = 0.05;
    scene.environment.background = Background::Color(Vec3::splat(0.005));
    scene.post.bloom = Some(og_render::Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.6, 4.0),
        target: Vec3::new(-1.5, 1.4, -10.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A lamp moving out from behind a pillar: its flare grows monotonically
/// from nothing as more of its disc clears the pillar's edge (the visible
/// share from the depth, the light 14 m away and the pillar 8 m).
#[test]
fn lens_flare_grows_as_a_lamp_clears_a_pillar() {
    let desc = small();
    // Its disc (0.2 m) clears the pillar's edge fully past x = 0.2.
    let gains: Vec<f32> = [-0.6f32, -0.15, 0.0, 0.15, 0.6]
        .into_iter()
        .map(|x| {
            let scene = pillar_lamp_scene(x);
            let (on, off) = flare_pair(&desc, || pillar_lamp_scene(x));
            ghost_gain(&on, &off, &scene.camera, pillar_lamp_at(x))
        })
        .collect();
    println!("lamp ghost gain, clearing the pillar: {gains:?}");
    assert!(
        gains[0].abs() < 0.5,
        "the hidden lamp casts none: {gains:?}"
    );
    for pair in gains.windows(2) {
        assert!(
            pair[1] + 0.5 >= pair[0],
            "the flare grows monotonically: {gains:?}"
        );
    }
    assert!(gains[4] > 6.0, "the clear lamp flares: {gains:?}");
    assert!(
        gains[2] > gains[0] + 1.0 && gains[2] < 0.8 * gains[4],
        "half the disc, part of the flare: {gains:?}"
    );
}

/// A lamp 20 m off, alone in the dark, high on the right of the frame (uv
/// 0.85, 0.25 under the default 50° field of view), so its first ghost
/// lands low on the left, 0.43 of the frame from the centre.
fn ghost_lamp() -> Vec3 {
    Vec3::new(11.6, 6.27, -16.0)
}

fn ghost_lamp_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    fx::terrain(&mut scene, 60.0);
    scene.lights.push(og_render::Light {
        position: ghost_lamp(),
        intensity: 120.0,
        range: 30.0,
        ..Default::default()
    });
    scene.light.intensity = 0.05;
    scene.environment.intensity = 0.05;
    scene.environment.background = Background::Color(Vec3::splat(0.005));
    scene.post.bloom = Some(og_render::Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.6, 4.0),
        target: Vec3::new(0.0, 1.6, -10.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A source's ghost is a smooth disc a few times its source's size, the
/// lens's defocused aperture: at the art frames' 1280 x 720, along two
/// lines across the lamp's first ghost (along the circle round the frame's
/// centre and at 45° to it, so stripes at any one angle cross at least
/// one), the light it adds rises to one peak and falls from it, never
/// dipping more than 1.5 below the brightest it has reached on the way up
/// or rising more than 1.5 over the dimmest on the way down: no dark line
/// inside, however wide; and 48 pixels out each way it is under a quarter
/// of its peak: the ghost of a lamp 20 m off is three times its disc (a
/// radius of 23 pixels), not the sun's 8 % of the height (58). With the
/// starburst on the whole flare its rays striped every ghost, and drawn at
/// a few quarter-frame texels the texels' rows showed; both are a couple
/// of pixels across at this size, read pixel by pixel across the line
/// (a 3 x 3 box averaged the rays to a third of the rule), and were
/// averaged away at the effects tests' 320 x 180 with 2-pixel steps. The
/// ghost stands well out from the centre: the starburst leaves the flare
/// whole within 0.3 of the centre (post.wgsl), so a ghost there (the
/// pillar lamp's, which this test once read) stayed a smooth disc even
/// under a starburst on the whole flare. The lamp is bright enough for a
/// peak near 60 of 255, so the rays' few per cent of it clear the rule.
#[test]
fn lens_flare_ghosts_are_smooth_discs() {
    let desc = RenderDesc {
        width: 1280,
        height: 720,
        ..small()
    };
    let mut on = ghost_lamp_scene();
    on.post.lens_flare = Some(LensFlare {
        intensity: 1.0,
        ..Default::default()
    });
    let mut off = ghost_lamp_scene();
    off.post.lens_flare = None;
    let (on, off) = (frame(&desc, &on).0, frame(&desc, &off).0);
    let (w, h) = (on.width, on.height);
    let (x, y) = project(&ghost_lamp_scene().camera, w, h, ghost_lamp());
    let (mx, my) = (w - 1 - x.min(w - 1), h - 1 - y.min(h - 1));
    let (dx, dy) = (mx as f32 - w as f32 * 0.5, my as f32 - h as f32 * 0.5);
    assert!(
        (dx * dx / (w * w) as f32 + dy * dy / (h * h) as f32).sqrt() > 0.35,
        "the ghost lands outside the starburst's smooth middle: ({mx}, {my})"
    );
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let (rx, ry) = (dx / len, dy / len);
    let (tx, ty) = (-ry, rx);
    let s = std::f32::consts::FRAC_1_SQRT_2;
    for (name, (ax, ay)) in [
        ("along the circle", (tx, ty)),
        ("at 45 degrees", ((tx - ty) * s, (ty + tx) * s)),
    ] {
        // One pixel along the line, the mean of three along the radial
        // direction: a starburst ray runs radially, 2 pixels wide at this
        // distance from the centre (post.wgsl's rows have 256 cells per
        // radian), so a box across the line averaged it under the rule;
        // along its own direction it is the same ray three times.
        let added = |t: f32| {
            (-1..=1)
                .map(|j| {
                    let cx = (mx as f32 + ax * t + rx * j as f32).round() as u32;
                    let cy = (my as f32 + ay * t + ry * j as f32).round() as u32;
                    let r = (cx, cy, cx + 1, cy + 1);
                    region(&on, r) - region(&off, r)
                })
                .sum::<f32>()
                / 3.0
        };
        let profile: Vec<f32> = (-48..=48).map(|i| added(i as f32)).collect();
        println!("lamp's ghost {name}: {profile:?}");
        let peak = (0..profile.len())
            .max_by(|&a, &b| profile[a].total_cmp(&profile[b]))
            .unwrap();
        assert!(
            profile[peak] > 6.0,
            "the lamp's ghost shows {name}: {profile:?}"
        );
        // Up to the peak from its left, and down from it to the right.
        let mut high = profile[0];
        for (i, &v) in profile.iter().enumerate().take(peak + 1) {
            assert!(
                v >= high - 1.5,
                "no dark line inside the ghost {name} (pixel {i}): {profile:?}"
            );
            high = high.max(v);
        }
        let mut low = profile[peak];
        for (i, &v) in profile.iter().enumerate().skip(peak) {
            assert!(
                v <= low + 1.5,
                "no dark line inside the ghost {name} (pixel {i}): {profile:?}"
            );
            low = low.min(v);
        }
        let (first, last) = (profile[0], profile[profile.len() - 1]);
        assert!(
            first < 0.25 * profile[peak] && last < 0.25 * profile[peak],
            "the lamp's ghost is a few times its disc, not sun-sized {name}: {profile:?}"
        );
    }
}

/// Under TAA a camera turning onto the sun draws its ghost where a camera
/// held still there draws it: the flare is made after TAA from sources
/// placed with the frame's own camera, so it never trails behind the turn
/// (sources placed with the previous frame's camera, as a pass fed before
/// the camera's update would be, land a ghost's width off at this turn's
/// speed). The jitter the sources leave out is the same on both runs' last
/// frame, so this comparison cannot see it; it is sub-pixel.
#[test]
fn lens_flare_ghosts_do_not_trail_under_taa() {
    let desc = RenderDesc {
        antialias: og_render::Antialias::Taa,
        ..small()
    };
    let scene = fx::flare_edge_scene(0.36);
    let target = scene.camera.target;
    let turn = move |s: &mut RenderScene, f: u32| {
        s.camera.target = target + Vec3::new(0.06 * (5.0 - f as f32), 0.0, 0.0);
    };
    let hold = |_: &mut RenderScene, _: u32| {};
    let gain = |moving: bool| {
        let (mut on, mut off) = (fx::flare_edge_scene(0.36), fx::flare_edge_scene(0.36));
        flare_probe(&mut on);
        off.post.lens_flare = None;
        let (on, ..) = if moving {
            run(&desc, &mut on, 6, turn)
        } else {
            run(&desc, &mut on, 6, hold)
        };
        let (off, ..) = if moving {
            run(&desc, &mut off, 6, turn)
        } else {
            run(&desc, &mut off, 6, hold)
        };
        let sun =
            scene.camera.position + Vec3::new(0.0, 0.36, -1.0).normalize() * scene.camera.far * 0.5;
        ghost_gain(&on, &off, &scene.camera, sun)
    };
    let (moving, still) = (gain(true), gain(false));
    println!("sun ghost gain under TAA: turning {moving}, held {still}");
    assert!(still > 6.0, "the held camera's sun flares: {still}");
    assert!(
        (moving - still).abs() < 0.1 * still,
        "the turning camera's ghost is where the held one's is: {moving} vs {still}"
    );
}

/// God rays brighten the shafts between columns; with the sun behind the
/// camera nothing changes.
#[test]
fn god_rays_shine_between_the_columns() {
    let desc = small();
    let scene = fx::god_rays_scene();
    let (rays, _) = frame(&desc, &scene);
    let mut off = fx::god_rays_scene();
    off.post.god_rays = None;
    let (plain, _) = frame(&desc, &off);
    let gain = region(&rays, (100, 40, 220, 120)) - region(&plain, (100, 40, 220, 120));
    println!("god rays: +{gain} luma around the sun");
    assert!(gain > 8.0);
    let mut behind = fx::god_rays_scene();
    behind.light.direction_to_light = Vec3::new(0.1, 0.3, 1.0);
    let (b_rays, _) = frame(&desc, &behind);
    behind.post.god_rays = None;
    let (b_plain, _) = frame(&desc, &behind);
    assert_eq!(differing(&b_rays, &b_plain, 1.0), 0);
}
