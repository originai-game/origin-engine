//! Screen-space GI (`ssgi.rs`) against a path-traced reference of gi-room,
//! and its cost. `reference` and `cost` are evidence for docs/roadmap.md's
//! GI row, run by hand on the GPU host:
//!
//! ```text
//! cargo test -p og-tests --release --test tests gi::reference -- --ignored --exact --nocapture
//! cargo test -p og-tests --release --test tests gi::cost -- --ignored --exact --nocapture
//! ```
//!
//! `OG_GI=rays=2,steps=16,reach=8,thickness=0.5,half,taa,frames=32,grid=40,samples=256`
//! overrides the screen GI's settings, the capture and the reference's
//! samples; `OG_GI_OUT=dir` saves the frames, and `gi::frames` writes the
//! room as the reference suite lights it for review. `OG_GI_COST=WxH` is
//! the size `gi::cost` draws at.
//!
//! The reference is a CPU path tracer over an analytic copy of the room
//! (`Room`, checked against the renderer's albedo view at every sample):
//! at pixels on a grid it gathers the bounced light E/π a diffuse surface
//! receives under a black sky, 256 cosine-weighted rays each followed
//! four bounces further. The renderer's share of it is its frame with
//! screen GI less the frame without, over the albedo, both in the units
//! the sun's direct light calibrates.
//!
//! Ways the pass could fail that the share shows: rays that stop short of
//! the far wall or spend their steps off screen (0.40 with a 4 m march),
//! hits that read no light (0 without last frame's image), light counted
//! twice or guessed (above the ceiling: what of the reference comes from
//! surfaces the camera sees; the pass's GI_BEYOND goes a little over it on
//! the walls beside the frame's edge).

use glam::{Vec2, Vec3};
use og_render::{
    Antialias, AoResolution, Background, DebugView, Exposure, RenderDesc, RenderScene,
    RenderTarget, Renderer, RgbaImage, ScreenSpaceGlobalIllumination, Tonemap,
};
use og_tests::{fx_scenes as fx, scenes};

/// The sun of `fx::gi_scene`.
const SUN: Vec3 = Vec3::new(0.45, 0.8, 0.4);
const SUN_INTENSITY: f32 = 3.0;
/// The capture's exposure: the brightest lit pixel (0.8 · 3 / π and its
/// bounce) stays under the 8-bit frame's 1.
const EXPOSURE: f32 = 0.25;

#[derive(Clone, Copy)]
enum Shape {
    /// The ground (y = 0, 30 m square) with `shapes::checker`'s 1 m cells.
    Ground,
    Box {
        centre: Vec3,
        half: Vec3,
    },
    Sphere {
        centre: Vec3,
        radius: f32,
    },
}

/// `fx::gi_scene`'s surfaces: (name, shape, albedo).
const ROOM: [(&str, Shape, Vec3); 6] = [
    ("floor", Shape::Ground, Vec3::ZERO),
    (
        "back wall",
        Shape::Box {
            centre: Vec3::new(0.0, 1.5, -2.5),
            half: Vec3::new(3.0, 1.5, 0.1),
        },
        Vec3::new(0.8, 0.8, 0.8),
    ),
    (
        "red wall",
        Shape::Box {
            centre: Vec3::new(-2.9, 1.5, 0.0),
            half: Vec3::new(0.1, 1.5, 2.5),
        },
        Vec3::new(0.8, 0.08, 0.06),
    ),
    (
        "green wall",
        Shape::Box {
            centre: Vec3::new(2.9, 1.5, 0.0),
            half: Vec3::new(0.1, 1.5, 2.5),
        },
        Vec3::new(0.08, 0.7, 0.1),
    ),
    (
        "block",
        Shape::Box {
            centre: Vec3::new(-1.0, 0.6, -0.8),
            half: Vec3::new(0.5, 0.6, 0.5),
        },
        Vec3::new(0.8, 0.8, 0.8),
    ),
    (
        "sphere",
        Shape::Sphere {
            centre: Vec3::new(1.1, 0.5, 0.2),
            radius: 0.5,
        },
        Vec3::new(0.8, 0.8, 0.8),
    ),
];

struct Hit {
    t: f32,
    n: Vec3,
    albedo: Vec3,
    surface: usize,
}

fn srgb(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// The nearest surface along `d` from `o`.
fn hit(o: Vec3, d: Vec3) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    for (surface, &(_, shape, albedo)) in ROOM.iter().enumerate() {
        let found = match shape {
            Shape::Ground => {
                let t = -o.y / d.y;
                let p = o + d * t;
                (d.y < 0.0 && t > 1e-4 && p.x.abs() <= 15.0 && p.z.abs() <= 15.0).then(|| {
                    let even = ((p.x + 15.0).floor() + (p.z + 15.0).floor()) as i64 % 2 == 0;
                    (t, Vec3::Y, Vec3::splat(srgb(if even { 170 } else { 120 })))
                })
            }
            Shape::Box { centre, half } => {
                let (lo, hi) = ((centre - half - o) / d, (centre + half - o) / d);
                let (near, far) = (lo.min(hi), lo.max(hi));
                let (t0, t1) = (near.max_element(), far.min_element());
                (t0 <= t1 && t0 > 1e-4).then(|| {
                    let axis = (0..3).find(|&a| near[a] == t0).unwrap();
                    let mut n = Vec3::ZERO;
                    n[axis] = -d[axis].signum();
                    (t0, n, albedo)
                })
            }
            Shape::Sphere { centre, radius } => {
                let oc = o - centre;
                let b = oc.dot(d);
                let disc = b * b - (oc.length_squared() - radius * radius);
                let t = -b - disc.max(0.0).sqrt();
                (disc >= 0.0 && t > 1e-4).then(|| (t, (oc + d * t) / radius, albedo))
            }
        };
        if let Some((t, n, albedo)) = found
            && best.as_ref().is_none_or(|b| t < b.t)
        {
            best = Some(Hit {
                t,
                n,
                albedo,
                surface,
            });
        }
    }
    best
}

/// The sun's light on a surface at `p` facing `n`, over the albedo (and
/// over the 1/π `direct_scale` measures).
fn sun(p: Vec3, n: Vec3) -> f32 {
    let l = SUN.normalize();
    let ndl = n.dot(l);
    if ndl <= 0.0 || hit(p + n * 1e-3, l).is_some() {
        0.0
    } else {
        SUN_INTENSITY * ndl
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A direction about `n` with density cos θ / π.
    fn cosine(&mut self, n: Vec3) -> Vec3 {
        let (u, v) = (self.next(), self.next());
        let (r, phi) = (u.sqrt(), std::f32::consts::TAU * v);
        let (t, b) = n.any_orthonormal_pair();
        (t * (r * phi.cos()) + b * (r * phi.sin()) + n * (1.0 - u).max(0.0).sqrt()).normalize()
    }
}

/// The light leaving the surface `d` meets from `p`, towards `p`: the
/// sun's and `depth` further bounces'. Also where it is.
fn radiance(p: Vec3, d: Vec3, rng: &mut Rng, depth: u32) -> Option<(Vec3, Vec3)> {
    let h = hit(p, d)?;
    let q = p + d * h.t;
    let mut e = Vec3::splat(sun(q, h.n));
    if depth > 0 {
        let o = q + h.n * 1e-3;
        if let Some((l, _)) = radiance(o, rng.cosine(h.n), rng, depth - 1) {
            e += l;
        }
    }
    Some((h.albedo * e, q))
}

struct Camera {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    /// tan(fov / 2) across and up.
    tan: Vec2,
}

impl Camera {
    fn of(scene: &RenderScene, w: u32, h: u32) -> Camera {
        let c = &scene.frame.camera;
        let og_render::Projection::Perspective { fov_y_degrees } = c.projection else {
            panic!("gi-room's camera is a perspective one");
        };
        let forward = (c.target - c.position).normalize();
        let right = forward.cross(c.up).normalize();
        let t = (fov_y_degrees.to_radians() * 0.5).tan();
        Camera {
            eye: c.position,
            forward,
            right,
            up: right.cross(forward),
            tan: Vec2::new(t * w as f32 / h as f32, t),
        }
    }

    /// The ray through `uv` (0..1, v down).
    fn ray(&self, uv: Vec2) -> Vec3 {
        let ndc = Vec2::new(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0) * self.tan;
        (self.forward + self.right * ndc.x + self.up * ndc.y).normalize()
    }

    /// Whether the surface point `q` lies in the camera's frame.
    fn frames(&self, q: Vec3) -> bool {
        let v = q - self.eye;
        let z = v.dot(self.forward);
        z > 0.0
            && (Vec2::new(v.dot(self.right), v.dot(self.up)) / (z * self.tan))
                .abs()
                .max_element()
                <= 1.0
    }

    /// Whether the camera's frame shows the surface point `q`.
    fn sees(&self, q: Vec3) -> bool {
        let v = q - self.eye;
        let dist = v.length();
        self.frames(q) && hit(self.eye, v / dist).is_none_or(|h| h.t > dist - 2e-3)
    }
}

struct Settings {
    gi: ScreenSpaceGlobalIllumination,
    taa: bool,
    frames: u32,
    grid: u32,
    samples: u32,
}

impl Settings {
    /// The defaults, with `OG_GI`'s overrides.
    fn from_env() -> Settings {
        let mut s = Settings {
            gi: ScreenSpaceGlobalIllumination::default(),
            taa: false,
            frames: 16,
            grid: 40,
            samples: 256,
        };
        let spec = std::env::var("OG_GI").unwrap_or_default();
        for part in spec.split(',').filter(|p| !p.is_empty()) {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            let number = || {
                value
                    .parse::<f32>()
                    .unwrap_or_else(|_| panic!("OG_GI: {part}"))
            };
            match key {
                "rays" => s.gi.rays = number() as u32,
                "steps" => s.gi.steps = number() as u32,
                "reach" => s.gi.max_distance = number(),
                "thickness" => s.gi.thickness = number(),
                "half" => s.gi.resolution = AoResolution::Half,
                "taa" => s.taa = true,
                "frames" => s.frames = number() as u32,
                "grid" => s.grid = number() as u32,
                "samples" => s.samples = number() as u32,
                _ => panic!("OG_GI: unknown setting {part}"),
            }
        }
        s
    }
}

/// gi-room under a black sky, untonemapped, at [`EXPOSURE`]: only the sun
/// and what it bounces.
fn room(gi: Option<ScreenSpaceGlobalIllumination>) -> RenderScene {
    let mut scene = fx::gi_scene(false);
    scene.look.post.gi = gi;
    scene.lighting.environment.intensity = 1e-3;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.frame.camera.exposure = Exposure::linear(EXPOSURE);
    scene.look.post.tonemap = Tonemap::None;
    scene
}

fn capture(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> RgbaImage {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    for frame in 0..frames.max(2) {
        // The full variants in place before the frames that are read.
        if frame == frames / 2 {
            og_render::settle_pipelines();
        }
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("frame");
    }
    renderer.read_offscreen().expect("capture")
}

/// The pixel's linear colour over the exposure.
fn linear(image: &RgbaImage, x: u32, y: u32) -> Vec3 {
    let i = ((y * image.width + x) * 4) as usize;
    Vec3::new(
        srgb(image.pixels[i]),
        srgb(image.pixels[i + 1]),
        srgb(image.pixels[i + 2]),
    )
}

/// What the measurement found, each a sum over the grid's samples.
struct Shares {
    /// The renderer's bounce over the reference's, per channel.
    gathered: Vec3,
    /// The same per surface of [`ROOM`] with its ceiling, the channels'
    /// mean (0 where the grid has no sample of it).
    surfaces: [(f32, f32); ROOM.len()],
    /// What of the reference surfaces the camera sees send: the most a
    /// screen-space method gathers without guessing.
    ceiling: Vec3,
    /// What of the reference comes from inside the camera's frame, seen
    /// or hidden behind something nearer.
    framed: Vec3,
    /// The renderer's direct light over the tracer's: 1/π, less what the
    /// specular lobe takes.
    direct_scale: Vec3,
    samples: u32,
}

fn measure(s: &Settings) -> Shares {
    let mut desc = scenes::base_desc();
    if s.taa {
        desc.antialias = Antialias::Taa;
    }
    let (w, h) = (desc.width, desc.height);
    let beauty = capture(&desc, &room(Some(s.gi)), s.frames);
    let base = capture(&desc, &room(None), 2);
    let mut flat = room(None);
    flat.frame.debug = DebugView::Albedo;
    flat.frame.camera.exposure = Exposure::linear(1.0);
    let albedo = capture(&desc, &flat, 2);
    if let Ok(dir) = std::env::var("OG_GI_OUT") {
        let dir = std::path::Path::new(&dir);
        og_tests::gfx::save_png(&dir.join("gi-beauty.png"), &beauty).unwrap();
        og_tests::gfx::save_png(&dir.join("gi-base.png"), &base).unwrap();
    }

    let camera = Camera::of(&room(None), w, h);
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut gpu, mut reference, mut seen, mut framed) =
        (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
    let (mut direct_gpu, mut direct_ref) = (Vec3::ZERO, Vec3::ZERO);
    let mut by_surface = [(0.0f32, 0.0f32, 0.0f32); ROOM.len()];
    let mut samples = 0;
    for py in (s.grid / 2..h).step_by(s.grid as usize) {
        for px in (s.grid / 2..w).step_by(s.grid as usize) {
            let d =
                camera.ray((Vec2::new(px as f32, py as f32) + 0.5) / Vec2::new(w as f32, h as f32));
            let Some(first) = hit(camera.eye, d) else {
                continue;
            };
            let p = camera.eye + d * first.t;
            let a = linear(&albedo, px, py);
            // The analytic room is the rendered one: a floor sample by a
            // checker cell's edge reads the neighbour's or a mip's mean.
            if first.surface != 0 {
                assert!(
                    (a - first.albedo).abs().max_element() < 0.03,
                    "({px}, {py}): the albedo view reads {a}, {} is {}",
                    ROOM[first.surface].0,
                    first.albedo
                );
            }
            let bounce = (linear(&beauty, px, py) - linear(&base, px, py))
                / EXPOSURE
                / a.max(Vec3::splat(1e-3));
            let origin = p + first.n * 1e-3;
            let (mut e, mut e_seen, mut e_framed) = (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
            for _ in 0..s.samples {
                if let Some((l, q)) = radiance(origin, rng.cosine(first.n), &mut rng, 4) {
                    e += l;
                    if camera.sees(q) {
                        e_seen += l;
                    }
                    if camera.frames(q) {
                        e_framed += l;
                    }
                }
            }
            e /= s.samples as f32;
            e_seen /= s.samples as f32;
            gpu += bounce;
            reference += e;
            seen += e_seen;
            framed += e_framed / s.samples as f32;
            by_surface[first.surface].0 += bounce.element_sum();
            by_surface[first.surface].1 += e.element_sum();
            by_surface[first.surface].2 += e_seen.element_sum();
            let lit = sun(p, first.n);
            if lit > 0.2 * SUN_INTENSITY {
                direct_gpu += linear(&base, px, py) / EXPOSURE;
                direct_ref += a * lit;
            }
            samples += 1;
        }
    }
    let direct_scale = direct_gpu / direct_ref;
    Shares {
        gathered: gpu / reference / direct_scale,
        surfaces: by_surface.map(|(g, r, seen)| {
            if r > 0.0 {
                (g / r / direct_scale.element_sum() * 3.0, seen / r)
            } else {
                (0.0, 0.0)
            }
        }),
        ceiling: seen / reference,
        framed: framed / reference,
        direct_scale,
        samples,
    }
}

#[test]
#[ignore = "evidence for docs/roadmap.md's GI row: run by hand on the GPU"]
fn reference() {
    let s = Settings::from_env();
    let m = measure(&s);
    println!(
        "gi-room\t{} rays x {} steps, reach {} m, thickness {} m, {:?}{}, {} frames, {} samples",
        s.gi.rays,
        s.gi.steps,
        s.gi.max_distance,
        s.gi.thickness,
        s.gi.resolution,
        if s.taa { ", TAA" } else { "" },
        s.frames,
        m.samples,
    );
    println!(
        "gathered\t{:.3}\t{:.3}\t{:.3}",
        m.gathered.x, m.gathered.y, m.gathered.z
    );
    println!(
        "ceiling\t{:.3}\t{:.3}\t{:.3}",
        m.ceiling.x, m.ceiling.y, m.ceiling.z
    );
    println!(
        "in frame\t{:.3}\t{:.3}\t{:.3}",
        m.framed.x, m.framed.y, m.framed.z
    );
    println!(
        "direct\t{:.4}\t{:.4}\t{:.4}\t(1/pi {:.4})",
        m.direct_scale.x,
        m.direct_scale.y,
        m.direct_scale.z,
        std::f32::consts::FRAC_1_PI
    );
    for (i, (name, ..)) in ROOM.iter().enumerate() {
        println!(
            "surface\t{name}\t{:.3}\tof {:.3}",
            m.surfaces[i].0, m.surfaces[i].1
        );
    }
}

/// gi-room as the reference suite lights it, for review under `OG_GI_OUT`:
/// without GI, with screen GI (still, under TAA, at half resolution) and
/// with the probes as well.
#[test]
#[ignore = "frames for review: run by hand on the GPU"]
fn frames() {
    let s = Settings::from_env();
    let dir = std::env::var("OG_GI_OUT").unwrap_or_else(|_| "target/gi".into());
    let dir = std::path::Path::new(&dir);
    let half = ScreenSpaceGlobalIllumination {
        resolution: AoResolution::Half,
        ..s.gi
    };
    for (name, gi, taa, probes, frames) in [
        ("off", None, false, false, 2),
        ("screen", Some(s.gi), false, false, s.frames),
        ("screen-taa", Some(s.gi), true, false, 2 * s.frames),
        ("screen-half", Some(half), false, false, s.frames),
        ("screen-half-taa", Some(half), true, false, 2 * s.frames),
        ("both-taa", Some(s.gi), true, true, 2 * s.frames),
    ] {
        let mut desc = scenes::base_desc();
        if taa {
            desc.antialias = Antialias::Taa;
        }
        let mut scene = fx::gi_probe_scene(probes);
        scene.look.post.gi = gi;
        let image = capture(&desc, &scene, frames);
        og_tests::gfx::save_png(&dir.join(format!("gi-room-{name}.png")), &image).unwrap();
    }
}

/// A 1080p frame of the room under TAA with no GI, the probes, screen GI
/// and both, under `og_gpu_acquire_all`: each pass's GPU time, the median
/// of 64 frames after 16 to settle.
#[test]
#[ignore = "evidence for docs/roadmap.md's GI row: run by hand on the GPU, alone"]
fn cost() {
    fn clock() -> f64 {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1e3
    }
    let s = Settings::from_env();
    let size = std::env::var("OG_GI_COST").unwrap_or_else(|_| "1920x1080".into());
    let (w, h) = size.split_once('x').expect("WxH");
    for (label, probes, screen) in [
        ("none", false, false),
        ("probes", true, false),
        ("screen", false, true),
        ("both", true, true),
    ] {
        let mut desc = scenes::base_desc();
        (desc.width, desc.height) = (w.parse().unwrap(), h.parse().unwrap());
        desc.antialias = Antialias::Taa;
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        if let Some(why) = renderer.gpu_timing_absent() {
            panic!("GPU lacks timing support: {why}");
        }
        renderer.set_clock(Some(clock));
        let mut scene = fx::gi_probe_scene(probes);
        scene.look.post.gi = screen.then_some(s.gi);
        let mut passes: Vec<(String, Vec<f64>)> = Vec::new();
        let mut total = Vec::new();
        for f in 0..80 {
            if f == 8 {
                og_render::settle_pipelines();
            }
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("frame");
            for frame in renderer.gpu_timings(true) {
                if f < 16 {
                    continue;
                }
                total.push(frame.ms());
                for p in &frame.passes {
                    let Some(ms) = p.ms() else { continue };
                    match passes.iter_mut().find(|q| q.0 == p.name) {
                        Some(q) => q.1.push(ms),
                        None => passes.push((p.name.clone(), vec![ms])),
                    }
                }
            }
        }
        let median = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[(v.len() - 1) / 2]
        };
        let each: Vec<String> = passes
            .iter_mut()
            .map(|(name, ms)| format!("{name} {:.3}", median(ms)))
            .collect();
        println!(
            "cost\t{size}\t{label}\tframe {:.3} ms\t{}",
            median(&mut total),
            each.join(", ")
        );
    }
}
