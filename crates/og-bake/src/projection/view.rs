//! Cameras, what they see (a ray-cast geometry buffer) and the grey clay
//! render a model is asked to paint over.

use super::{Level, SurfaceHit, par_for};
use crate::trace::{Rng, cosine_dir, r2};
use glam::{Vec2, Vec3};

/// Where a camera came from: a player's view, the rings around the level, a
/// close pass over the roofs, or the last pass that only fills texels
/// nothing painted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewKind {
    Ground,
    Aerial,
    Roof,
    Fill,
}

impl ViewKind {
    pub fn name(self) -> &'static str {
        match self {
            ViewKind::Ground => "ground",
            ViewKind::Aerial => "aerial",
            ViewKind::Roof => "roof",
            ViewKind::Fill => "fill",
        }
    }

    pub fn parse(s: &str) -> Option<ViewKind> {
        Some(match s {
            "ground" => ViewKind::Ground,
            "aerial" => ViewKind::Aerial,
            "roof" => ViewKind::Roof,
            "fill" => ViewKind::Fill,
            _ => return None,
        })
    }
}

/// A square perspective camera. Yaw 0 looks down -Z; pitch is up from the
/// horizon; `fov` is the vertical (and horizontal) angle in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewSpec {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub kind: ViewKind,
}

impl ViewSpec {
    /// A camera at `eye` looking at `at`.
    pub fn looking(eye: Vec3, at: Vec3, fov: f32, kind: ViewKind) -> ViewSpec {
        let d = (at - eye).normalize_or(Vec3::NEG_Z);
        ViewSpec {
            position: eye,
            yaw: (-d.x).atan2(-d.z),
            pitch: d.y.clamp(-1.0, 1.0).asin(),
            fov,
            kind,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(-self.yaw.sin() * cp, self.pitch.sin(), -self.yaw.cos() * cp)
    }

    /// Forward, right and up.
    pub fn axes(&self) -> (Vec3, Vec3, Vec3) {
        let f = self.forward();
        let right = f.cross(Vec3::Y).normalize_or(Vec3::X);
        (f, right, right.cross(f))
    }

    /// The unit ray through the centre of pixel (x, y) of a `size`² image
    /// (rows from the top).
    pub fn ray(&self, x: f32, y: f32, size: u32) -> Vec3 {
        let (f, r, u) = self.axes();
        let k = (self.fov.to_radians() * 0.5).tan();
        let nx = 2.0 * (x + 0.5) / size as f32 - 1.0;
        let ny = 1.0 - 2.0 * (y + 0.5) / size as f32;
        (f + r * (nx * k) + u * (ny * k)).normalize()
    }

    /// Where `p` lands in a `size`² image (continuous pixel coordinates,
    /// (0, 0) the top left corner), if in front of the camera.
    pub fn project(&self, p: Vec3, size: u32) -> Option<Vec2> {
        let (f, r, u) = self.axes();
        let v = p - self.position;
        let z = v.dot(f);
        if z <= 1e-4 {
            return None;
        }
        let k = (self.fov.to_radians() * 0.5).tan();
        let nx = v.dot(r) / (z * k);
        let ny = v.dot(u) / (z * k);
        Some(Vec2::new(
            (nx + 1.0) * 0.5 * size as f32,
            (1.0 - ny) * 0.5 * size as f32,
        ))
    }
}

/// What one camera sees: the surface under every pixel's centre (rows from
/// the top), `None` for sky.
pub struct GBuffer {
    pub size: u32,
    pub view: ViewSpec,
    pub hits: Vec<Option<SurfaceHit>>,
}

/// One pixel's capture quality and its parts, as the projection reads them.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub hit: SurfaceHit,
    /// cos(view angle) / max(distance, 0.5): how well this pixel sees its texel.
    pub quality: f32,
}

impl GBuffer {
    pub fn cast(level: &Level, view: ViewSpec, size: u32, threads: usize) -> GBuffer {
        let mut hits = vec![None; (size * size) as usize];
        par_for(&mut hits, threads, |i, out| {
            let (x, y) = ((i as u32 % size) as f32, (i as u32 / size) as f32);
            *out = level.hit(view.position, view.ray(x, y, size), f32::MAX);
        });
        GBuffer { size, view, hits }
    }

    pub fn sample(&self, i: usize) -> Option<Sample> {
        let hit = self.hits[i]?;
        let to_cam = (self.view.position - hit.position).normalize_or(Vec3::Y);
        Some(Sample {
            hit,
            quality: hit.normal.dot(to_cam).max(0.0) / hit.t.max(0.5),
        })
    }

    pub fn geometry(&self) -> Vec<u8> {
        self.hits.iter().map(|h| h.is_some() as u8).collect()
    }

    /// Pixels within `radius` px of a silhouette (sky or a depth jump of more
    /// than 4 %) along any of 8 directions.
    pub fn silhouette(&self, radius: f32) -> Vec<u8> {
        let s = self.size as i32;
        let offsets: Vec<(i32, i32)> = (0..8)
            .map(|k| {
                let a = std::f32::consts::FRAC_PI_4 * k as f32;
                (
                    (a.cos() * radius).round() as i32,
                    (a.sin() * radius).round() as i32,
                )
            })
            .collect();
        (0..(s * s) as usize)
            .map(|i| {
                let Some(h) = self.hits[i] else { return 0 };
                let (x, y) = (i as i32 % s, i as i32 / s);
                offsets.iter().any(|&(dx, dy)| {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx < 0 || yy < 0 || xx >= s || yy >= s {
                        return true;
                    }
                    match self.hits[(yy * s + xx) as usize] {
                        None => true,
                        Some(n) => (n.t - h.t).abs() > 0.04 * h.t,
                    }
                }) as u8
            })
            .collect()
    }
}

/// The clay's light: an overcast sky (ambient occlusion within `radius`
/// metres, cosine-weighted, `samples` rays a pixel) and a soft key light
/// without cast shadows, so the texture painted over it comes out close to
/// albedo.
#[derive(Clone, Copy, Debug)]
pub struct ClayLight {
    pub samples: u32,
    pub radius: f32,
    pub albedo: f32,
    pub sky: Vec3,
    pub key: Vec3,
    pub key_strength: f32,
    pub threads: usize,
}

impl Default for ClayLight {
    fn default() -> Self {
        ClayLight {
            samples: 24,
            radius: 6.0,
            albedo: 0.62,
            sky: Vec3::new(0.85, 0.88, 0.92),
            key: Vec3::new(0.35, 0.85, 0.4).normalize(),
            key_strength: 0.35,
            threads: 0,
        }
    }
}

/// The clay render of `g`, sRGB 8-bit RGB rows from the top.
pub fn clay(level: &Level, g: &GBuffer, light: &ClayLight) -> Vec<[u8; 3]> {
    let n = g.hits.len();
    let eps = level.epsilon();
    let mut ao = vec![1.0f32; n];
    par_for(&mut ao, light.threads, |i, out| {
        let Some(h) = g.hits[i] else { return };
        let mut rng = Rng::new((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5eed);
        let offset = Vec2::new(rng.f32(), rng.f32());
        let origin = h.position + h.face * eps * 8.0;
        let mut open = 0u32;
        for k in 0..light.samples {
            let mut d = cosine_dir(h.normal, r2(k, offset));
            if d.dot(h.face) <= 0.0 {
                d = (d - h.face * d.dot(h.face) * 1.01).normalize_or(h.face);
            }
            if !level.occluded(origin, d, light.radius) {
                open += 1;
            }
        }
        *out = open as f32 / light.samples.max(1) as f32;
    });
    // An edge-aware 5×5 blur: the AO's noise without blurring across creases.
    let s = g.size as i32;
    let mut smooth = ao.clone();
    par_for(&mut smooth, light.threads, |i, out| {
        let Some(h) = g.hits[i] else { return };
        let (x, y) = (i as i32 % s, i as i32 / s);
        let (mut sum, mut wsum) = (0.0, 0.0);
        for dy in -2..=2 {
            for dx in -2..=2 {
                let (xx, yy) = (x + dx, y + dy);
                if xx < 0 || yy < 0 || xx >= s || yy >= s {
                    continue;
                }
                let j = (yy * s + xx) as usize;
                let Some(o) = g.hits[j] else { continue };
                if o.normal.dot(h.normal) < 0.9 || (o.t - h.t).abs() > 0.03 * h.t {
                    continue;
                }
                sum += ao[j];
                wsum += 1.0;
            }
        }
        *out = sum / wsum;
    });
    (0..n)
        .map(|i| match g.hits[i] {
            None => light.sky.to_array().map(srgb),
            Some(h) => {
                let up = 0.75 + 0.25 * h.normal.y;
                let e = light.sky * (smooth[i] * up)
                    + Vec3::splat(light.key_strength * h.normal.dot(light.key).max(0.0));
                (e * light.albedo).to_array().map(srgb)
            }
        })
        .collect()
}

fn srgb(v: f32) -> u8 {
    (v.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}
