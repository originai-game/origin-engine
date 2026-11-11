//! Shadows and antialiasing (`crates/og-render/README.md`
//! §Shadows and antialiasing): the reference frames of the round's
//! scenes, and behaviour read from frames and stats. More than two local
//! lights cast shadows, the ones left out past the tier's count are the
//! least radiant at the camera, dual-paraboloid point shadows draw the
//! cube's shadows in a third of the passes, PCSS widens the sun's penumbra
//! with the caster's height, and FXAA and TAA's sharpening work on edges.

use og_render::{
    Culling, FrameStats, PointShadows, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
    Tonemap,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::{scenes, shadow_scenes};

fn frame(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats)
}

fn luma(p: &[u8]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// The contact scenes' camera (`shadow_scenes::local_contact_camera`)
/// for a frame of `img`'s size.
fn contact_view_proj(img: &RgbaImage) -> glam::Mat4 {
    let cam = shadow_scenes::local_contact_camera();
    glam::camera::rh::proj::directx::perspective(
        50f32.to_radians(),
        img.width as f32 / img.height as f32,
        0.1,
        100.0,
    ) * glam::camera::rh::view::look_at_mat4(cam.position, cam.target, glam::Vec3::Y)
}

/// A caster of the contact scenes, for the false-contact probes' geometry.
#[derive(Clone, Copy)]
enum Caster {
    /// Least and greatest corners.
    Box(glam::Vec3, glam::Vec3),
    /// Centre and radius.
    Ball(glam::Vec3, f32),
}

impl Caster {
    /// Where the ray from `o` along unit `d` enters it, if ahead of `o`.
    fn enter(self, o: glam::Vec3, d: glam::Vec3) -> Option<f32> {
        match self {
            Caster::Box(lo, hi) => {
                let inv = d.recip();
                let (a, b) = ((lo - o) * inv, (hi - o) * inv);
                let near = a.min(b).max_element();
                let far = a.max(b).min_element();
                (near <= far && near > 0.0).then_some(near)
            }
            Caster::Ball(c, r) => {
                let oc = o - c;
                let b = oc.dot(d);
                let disc = b * b - (oc.length_squared() - r * r);
                let t = -b - disc.max(0.0).sqrt();
                (disc >= 0.0 && t > 0.0).then_some(t)
            }
        }
    }

    fn distance(self, q: glam::Vec3) -> f32 {
        match self {
            Caster::Box(lo, hi) => (lo - q).max(q - hi).max(glam::Vec3::ZERO).length(),
            Caster::Ball(c, r) => q.distance(c) - r,
        }
    }
}

/// Whether the segment from `a` to `b` passes through one of `casters`.
fn blocked(a: glam::Vec3, b: glam::Vec3, casters: &[Caster]) -> bool {
    let (d, len) = ((b - a).normalize(), a.distance(b));
    casters
        .iter()
        .any(|c| c.enter(a, d).is_some_and(|t| t < len - 1e-4))
}

/// Where a contact scene's frame shows lit floor or wall that lies just
/// past the silhouette of one of `behind`, at a depth gap: points the
/// light reaches, clear of every caster by 8 cm at their distance from it
/// (more than a local map's filter and normal offset reach there, so no
/// map shades them), whose pixel and its neighbours two pixels away show
/// that receiver, and from which the ray towards the light passes behind
/// one of `behind` as the camera sees it within 48 pixels (inside a
/// screen-space trace's reach) without touching anything. A trace that
/// takes a silhouette in front of a far surface for an occluder darkens
/// them. With no `behind`, every such point clear of the casters by 30 cm,
/// passing behind nothing: open lit floor and wall, which shadow acne
/// (a map comparing a lit surface against itself) darkens.
fn behind_silhouettes(
    img: &RgbaImage,
    casters: &[Caster],
    behind: Option<&[Caster]>,
) -> Vec<(u32, u32)> {
    use glam::Vec3;
    let light = shadow_scenes::CONTACT_LIGHT;
    let eye = shadow_scenes::local_contact_camera().position;
    let view_proj = contact_view_proj(img);
    let inv = view_proj.inverse();
    let (w, h) = (img.width as f32, img.height as f32);
    let to_px = |p: Vec3| {
        let q = view_proj.project_point3(p);
        ((q.x * 0.5 + 0.5) * w, (0.5 - q.y * 0.5) * h)
    };
    // What the camera sees through a pixel's centre: 0 the floor, 1 the
    // wall, 2 a caster.
    let seen = |x: f32, y: f32| -> u8 {
        let ndc = Vec3::new((x + 0.5) / w * 2.0 - 1.0, 1.0 - (y + 0.5) / h * 2.0, 0.5);
        let d = (inv.project_point3(ndc) - eye).normalize();
        let floor = if d.y < 0.0 { -eye.y / d.y } else { f32::MAX };
        let wall = if d.z < 0.0 {
            (-2.0 - eye.z) / d.z
        } else {
            f32::MAX
        };
        let caster = casters
            .iter()
            .filter_map(|c| c.enter(eye, d))
            .fold(f32::MAX, f32::min);
        if caster < floor.min(wall) {
            2
        } else if floor < wall {
            0
        } else {
            1
        }
    };
    let clearance = |p: Vec3| {
        let far = p.distance(light);
        (1..100)
            .map(|i| p.lerp(light, i as f32 / 100.0))
            .flat_map(|q| {
                casters
                    .iter()
                    .map(move |c| c.distance(q) * far / q.distance(light))
            })
            .fold(f32::MAX, f32::min)
    };
    let floor = (-60..=60).flat_map(|x| {
        (-39..40).map(move |z| (Vec3::new(x as f32 * 0.05, 0.0, z as f32 * 0.05), 0u8))
    });
    let wall = (-60..=60).flat_map(|x| {
        (1..50).map(move |y| (Vec3::new(x as f32 * 0.05, y as f32 * 0.05, -2.0), 1u8))
    });
    let mut out = Vec::new();
    for (p, receiver) in floor.chain(wall) {
        let (x, y) = to_px(p);
        if x < 4.0 || y < 4.0 || x >= w - 4.0 || y >= h - 4.0 || blocked(eye, p, casters) {
            continue;
        }
        let (x, y) = (x.floor(), y.floor());
        let clean = [-2.0, 0.0, 2.0].iter().all(|&ox| {
            [-2.0, 0.0, 2.0]
                .iter()
                .all(|&oy| seen(x + ox, y + oy) == receiver)
        });
        let Some(behind) = behind else {
            if clean && clearance(p) >= 0.3 {
                out.push((x as u32, y as u32));
            }
            continue;
        };
        if !clean || clearance(p) < 0.08 {
            continue;
        }
        let d = (light - p).normalize();
        let passes_behind = (1..200)
            .map(|i| p + d * (i as f32 * 0.01))
            .take_while(|&q| {
                let (qx, qy) = to_px(q);
                (qx - x).hypot(qy - y) <= 48.0
            })
            .any(|q| blocked(eye, q, behind));
        if passes_behind {
            out.push((x as u32, y as u32));
        }
    }
    out
}

/// The share of their direct light that `points` keep in `shadowed`
/// (`lit`: the light unshadowed, `dark`: off): the least and the mean, and
/// how many points the light reaches by 16 levels.
fn kept_light(
    points: &[(u32, u32)],
    shadowed: &RgbaImage,
    lit: &RgbaImage,
    dark: &RgbaImage,
) -> (f32, f32, usize) {
    let at = |img: &RgbaImage, (x, y): (u32, u32)| {
        let i = ((y * img.width + x) * 4) as usize;
        luma(&img.pixels[i..i + 3])
    };
    // Only where the light gives 16 levels or more (inside a spot's cone).
    let ratios: Vec<f32> = points
        .iter()
        .filter(|&&q| at(lit, q) - at(dark, q) >= 16.0)
        .map(|&q| (at(shadowed, q) - at(dark, q)) / (at(lit, q) - at(dark, q)))
        .collect();
    let least = ratios.iter().copied().fold(f32::MAX, f32::min);
    (
        least,
        ratios.iter().sum::<f32>() / ratios.len().max(1) as f32,
        ratios.len(),
    )
}

/// The standard deviation of the share of their direct light that the
/// lit `points` keep (as `kept_light`), and their count.
fn kept_spread(
    points: &[(u32, u32)],
    shadowed: &RgbaImage,
    lit: &RgbaImage,
    dark: &RgbaImage,
) -> (usize, f32) {
    let at = |img: &RgbaImage, (x, y): (u32, u32)| {
        let i = ((y * img.width + x) * 4) as usize;
        luma(&img.pixels[i..i + 3])
    };
    let r: Vec<f32> = points
        .iter()
        .filter(|&&q| at(lit, q) - at(dark, q) >= 16.0)
        .map(|&q| (at(shadowed, q) - at(dark, q)) / (at(lit, q) - at(dark, q)))
        .collect();
    let n = r.len().max(1) as f32;
    let mean = r.iter().sum::<f32>() / n;
    (
        r.len(),
        (r.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n).sqrt(),
    )
}

/// Contact shadows never darken what lies behind a silhouette, and the
/// local maps never darken open lit surfaces (acne): the lit
/// floor and wall just past the contact scene's box, and past a large
/// ball and a thin pole, at a depth gap behind them, where a screen-space
/// trace towards the light passes behind them (the false contact the art
/// director held on contact 497: a blurred bar beside the box's corner,
/// a dark column along the ball). Each keeps its direct light, for a spot
/// and for both point shadow storages.
#[test]
#[ignore = "requires GPU"]
fn contact_shadows_never_darken_what_lies_behind_a_silhouette() {
    use shadow_scenes::{CONTACT_BOX, SILHOUETTE_BALL, SILHOUETTE_POLE};
    let desc = scenes::base_desc();
    let half = CONTACT_BOX * 0.5;
    let boxed = Caster::Box(
        glam::Vec3::new(-half.x, 0.0, -half.z),
        glam::Vec3::new(half.x, CONTACT_BOX.y, half.z),
    );
    let small_ball = Caster::Ball(glam::Vec3::new(-1.0, 0.25, 0.9), 0.25);
    let ball = Caster::Ball(SILHOUETTE_BALL.0, SILHOUETTE_BALL.1);
    let pole = Caster::Box(SILHOUETTE_POLE.0, SILHOUETTE_POLE.1);
    for (point, storage) in [
        (false, PointShadows::Cube),
        (true, PointShadows::Cube),
        (true, PointShadows::Paraboloid),
    ] {
        let desc = RenderDesc {
            point_shadows: storage,
            ..desc
        };
        // The scene shadowed, its light unshadowed, and its light off.
        let frames = |scene: &dyn Fn() -> RenderScene| {
            let take = |shadows: bool, on: bool| {
                let mut s = scene();
                s.look.post.tonemap = Tonemap::None;
                s.lights[0].cast_shadows = shadows;
                if !on {
                    s.lights[0].intensity = 0.0;
                }
                frame(&desc, &s, 1).0
            };
            (take(true, true), take(false, true), take(false, false))
        };
        let contact = frames(&|| shadow_scenes::local_contact_scene(point, true));
        let silhouettes = frames(&|| shadow_scenes::contact_silhouettes_scene(point));
        for (name, (shadowed, lit, dark), casters, behind, least) in [
            ("box", &contact, vec![boxed, small_ball], vec![boxed], 60),
            ("ball", &silhouettes, vec![ball, pole], vec![ball], 25),
            ("thin pole", &silhouettes, vec![ball, pole], vec![pole], 60),
        ] {
            let points = behind_silhouettes(shadowed, &casters, Some(&behind));
            let (min, mean, n) = kept_light(&points, shadowed, lit, dark);
            println!(
                "point {point} {storage:?}: {n} lit points behind the {name}'s silhouette keep {min:.3} to {mean:.3} (mean) of their direct light"
            );
            assert!(n >= least, "{name}: {n} lit points of {}", points.len());
            assert!(
                min > 0.85 && mean > 0.95,
                "{name}: the floor and wall behind its silhouette darkened (least {min:.3}, mean {mean:.3})"
            );
        }
        // Open lit floor and wall, far from every caster: no acne.
        for (name, (shadowed, lit, dark), casters) in [
            ("contact", &contact, vec![boxed, small_ball]),
            ("silhouettes", &silhouettes, vec![ball, pole]),
        ] {
            let points = behind_silhouettes(shadowed, &casters, None);
            let (min, mean, n) = kept_light(&points, shadowed, lit, dark);
            println!(
                "point {point} {storage:?}: {n} open lit points of the {name} scene keep {min:.3} to {mean:.3} (mean) of their direct light"
            );
            assert!(n >= 1_000, "{name}: {n} lit points of {}", points.len());
            // Stripes or banding: the share kept varies from point to point.
            let (_, spread) = kept_spread(&points, shadowed, lit, dark);
            println!("  spread of the share kept: {spread:.4}");
            assert!(
                spread < 0.02,
                "{name}: open lit floor and wall vary by {spread:.4} (stripes or banding)"
            );
            assert!(
                min > 0.9 && mean > 0.98,
                "{name}: open lit floor and wall darkened, shadow acne (least {min:.3}, mean {mean:.3})"
            );
        }
    }
}

/// The static shadow cache (og-render's shadow_cache.rs): a sphere rolls
/// for four frames among the pillars, under the sun's cascades, a spot and
/// a point light's cube faces, then stands still. While it rolls, the
/// layers it does not reach keep their static casters and the ones it
/// reaches draw it over theirs; once it has stood still long enough every
/// layer is kept and nothing is drawn. The last frame is the one a
/// renderer that saw only the final scene draws, on the CPU and the GPU
/// culling paths.
#[test]
#[ignore = "requires GPU"]
fn cached_shadow_layers_draw_what_fresh_ones_do() {
    for culling in [Culling::Cpu, Culling::Gpu] {
        let desc = RenderDesc {
            culling,
            ..scenes::base_desc()
        };
        let mut scene = scenes::point_spot_lights();
        scene.light.cast_shadows = true;
        // The sphere at (0.2, 0.5, -2.0), the last instance.
        let mover = scene.instances.len() - 1;
        let start = scene.instances[mover].transform;
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        let mut stats = Vec::new();
        for f in 0..16 {
            let x = 0.25 * f.min(4) as f32;
            scene.instances[mover].transform =
                glam::Mat4::from_translation(glam::Vec3::new(x, 0.0, 0.0)) * start;
            stats.push(
                renderer
                    .render(&scene, &RenderTarget::Offscreen)
                    .expect("render"),
            );
        }
        let cached = renderer.read_offscreen().expect("capture");
        let still = frame(&desc, &scene, 16).0;
        let fresh = frame(&desc, &scene, 1).0;
        for (i, s) in stats.iter().enumerate() {
            println!(
                "{culling:?} frame {i}: {} shadow passes, {} cached, {} casters drawn",
                s.shadow_passes, s.shadow_cached, s.shadow_casters
            );
        }
        let passes = stats[0].shadow_passes;
        assert!(passes >= 8, "the cascades and seven local faces: {passes}");
        assert_eq!(
            stats[0].shadow_cached, 0,
            "the first frame draws every layer"
        );
        // The first frame it moves, the layers its old sphere reaches draw
        // their static casters again (and save them); the others are kept.
        let first = &stats[0];
        let moved = &stats[1];
        assert!(
            moved.shadow_cached > 0 && moved.shadow_cached < passes,
            "{culling:?}: {moved:?}"
        );
        // Rolling on (frames 2 to 11, dynamic until 8 still frames): every
        // layer comes from the cache, and only the mover and the casters
        // drawn every frame are drawn over them: far fewer than all.
        // GPU culling counts its casters on the GPU and reads none back
        // here (`counts_lag` unset): the CPU path checks the counts, both
        // check which layers are kept and the frames.
        let counts = culling == Culling::Cpu;
        for s in &stats[2..=11] {
            assert_eq!(s.shadow_cached, passes, "{culling:?}: {s:?}");
            assert!(
                !counts || (s.shadow_casters > 0 && 2 * s.shadow_casters < first.shadow_casters),
                "{culling:?}: {s:?} against {first:?}"
            );
        }
        // Static again at frame 12: the layers its sphere reaches redraw.
        assert!(
            stats[12].shadow_cached < passes,
            "{culling:?}: {:?}",
            stats[12]
        );
        // Nothing drawn from frame 13 on.
        let last = stats.last().expect("frames");
        assert_eq!(last.shadow_passes, passes);
        assert_eq!(last.shadow_cached, passes, "{culling:?}: every layer kept");
        if counts {
            assert_eq!(last.shadow_casters, 0, "{culling:?}: nothing drawn");
        }
        let exact = Tolerance {
            channel: 1,
            max_outlier_fraction: 0.0,
            rmse: 0.001,
            ..Tolerance::default()
        };
        for (name, other) in [("still", &still), ("fresh", &fresh)] {
            let diff = compare(
                &cached.pixels,
                &other.pixels,
                cached.width,
                cached.height,
                &exact,
            )
            .expect("same size");
            println!("{culling:?} cached against {name}: {diff:?}");
            diff.check(&exact)
                .unwrap_or_else(|e| panic!("{culling:?}: cached against {name}: {e}"));
        }
    }
}
