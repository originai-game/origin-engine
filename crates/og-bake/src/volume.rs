//! Light volumes: a grid of probes over a box, each the light arriving from
//! the scene in every direction (bounced light, the sky, emissive surfaces,
//! static lights' direct light; not a stationary light's direct term, which
//! the renderer draws) projected onto first-order spherical harmonics,
//! ready for the shader's irradiance.
//!
//! A probe stores, per colour channel, `(c1, c0)` such that the irradiance
//! over π for a normal n is `c0 + c1 · n` (Ramamoorthi and Hanrahan's
//! convolution of the L1 radiance with the cosine lobe: A0 = π, A1 = 2π/3,
//! divided by π). Probes that start inside geometry take the mean of their
//! neighbours.

use crate::scene::Scene;
use crate::trace::{Rng, Tracer};
use glam::Vec3;

/// A box of probes, axis-aligned in world space.
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeDesc {
    pub min: Vec3,
    pub max: Vec3,
    /// The distance between neighbouring probes (at most; the box is
    /// divided evenly, with a probe on every face).
    pub spacing: f32,
}

/// The baked probes: `counts` along x, y, z (x fastest), each the irradiance
/// harmonics of red, green and blue as `[c1.x, c1.y, c1.z, c0]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Volume {
    pub min: Vec3,
    pub max: Vec3,
    pub counts: [u32; 3],
    pub probes: Vec<[[f32; 4]; 3]>,
}

impl VolumeDesc {
    /// The probes along each axis: at least two, no further apart than `spacing`.
    pub fn counts(&self) -> [u32; 3] {
        let size = (self.max - self.min).max(Vec3::ZERO);
        let s = self.spacing.max(1e-3);
        [0, 1, 2].map(|k| ((size[k] / s).ceil() as u32 + 1).clamp(2, 64))
    }
}

/// Points of each static light a probe's direct light is the mean of.
const STATIC_SAMPLES: u32 = 16;

/// The most probes a volume may have (64 per axis bounds it too).
pub const MAX_PROBES: usize = 65_536;

const Y0: f32 = 0.282_094_8;
const Y1: f32 = 0.488_602_5;

/// The `i`th of `n` points of a Fibonacci sphere, turned by `turn`.
fn fibonacci(i: u32, n: u32, turn: f32) -> Vec3 {
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let y = 1.0 - (i as f32 + 0.5) / n as f32 * 2.0;
    let r = (1.0 - y * y).max(0.0).sqrt();
    let phi = golden * i as f32 + turn;
    Vec3::new(r * phi.cos(), y, r * phi.sin())
}

/// Bakes one volume with `samples` rays per probe.
pub fn bake(
    scene: &Scene,
    desc: &VolumeDesc,
    samples: u32,
    bounces: u32,
    seed: u64,
    threads: usize,
) -> Volume {
    let tracer = Tracer::new(scene, bounces);
    let statics = tracer.has_static();
    let counts = desc.counts();
    let n = (counts[0] * counts[1] * counts[2]) as usize;
    let step = (desc.max - desc.min)
        / Vec3::new(
            (counts[0] - 1) as f32,
            (counts[1] - 1) as f32,
            (counts[2] - 1) as f32,
        );
    let at = |i: usize| {
        let x = i as u32 % counts[0];
        let y = i as u32 / counts[0] % counts[1];
        let z = i as u32 / (counts[0] * counts[1]);
        desc.min + step * Vec3::new(x as f32, y as f32, z as f32)
    };
    let samples = samples.max(16);
    let threads = if threads == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        threads
    }
    .max(1);
    // Per probe: the radiance harmonics (L00, L1-1, L10, L11 per channel)
    // and whether it lies inside geometry.
    let mut out: Vec<([Vec3; 4], bool)> = vec![([Vec3::ZERO; 4], false); n];
    let chunk = n.div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (c, slice) in out.chunks_mut(chunk).enumerate() {
            let tracer = &tracer;
            s.spawn(move || {
                for (k, probe) in slice.iter_mut().enumerate() {
                    let i = c * chunk + k;
                    let p = at(i);
                    let mut rng = Rng::new(seed ^ (i as u64).wrapping_mul(0x9E37_79B9));
                    let turn = rng.next_f32() * std::f32::consts::TAU;
                    let mut sh = [Vec3::ZERO; 4];
                    let mut back = 0;
                    for j in 0..samples {
                        let d = fibonacci(j, samples, turn);
                        let (l, inside) = tracer.radiance(p, d, &mut rng);
                        back += inside as u32;
                        sh[0] += l * Y0;
                        sh[1] += l * (Y1 * d.y);
                        sh[2] += l * (Y1 * d.z);
                        sh[3] += l * (Y1 * d.x);
                    }
                    let w = 4.0 * std::f32::consts::PI / samples as f32;
                    let mut sh = sh.map(|c| c * w);
                    // The static lights' direct light: each a lobe of
                    // irradiance from its direction (rect lights sampled
                    // at STATIC_SAMPLES points).
                    if statics {
                        let k = 1.0 / STATIC_SAMPLES as f32;
                        for _ in 0..STATIC_SAMPLES {
                            tracer.arriving(p, None, true, &mut rng, &mut |e, l| {
                                sh[0] += e * (Y0 * k);
                                sh[1] += e * (Y1 * l.y * k);
                                sh[2] += e * (Y1 * l.z * k);
                                sh[3] += e * (Y1 * l.x * k);
                            });
                        }
                    }
                    *probe = (sh, back * 10 > samples * 3);
                }
            });
        }
    });
    // Probes inside geometry: the mean of their valid neighbours, ring by ring.
    for _ in 0..8 {
        let src = out.clone();
        let mut left = false;
        for i in 0..n {
            if !src[i].1 {
                continue;
            }
            let (x, y, z) = (
                i as i32 % counts[0] as i32,
                i as i32 / counts[0] as i32 % counts[1] as i32,
                i as i32 / (counts[0] * counts[1]) as i32,
            );
            let mut sum = [Vec3::ZERO; 4];
            let mut k = 0.0;
            for (dx, dy, dz) in [
                (-1, 0, 0),
                (1, 0, 0),
                (0, -1, 0),
                (0, 1, 0),
                (0, 0, -1),
                (0, 0, 1),
            ] {
                let (a, b, c) = (x + dx, y + dy, z + dz);
                if a < 0
                    || b < 0
                    || c < 0
                    || a >= counts[0] as i32
                    || b >= counts[1] as i32
                    || c >= counts[2] as i32
                {
                    continue;
                }
                let j = (a + b * counts[0] as i32 + c * (counts[0] * counts[1]) as i32) as usize;
                if !src[j].1 {
                    for (s, c) in sum.iter_mut().zip(src[j].0) {
                        *s += c;
                    }
                    k += 1.0;
                }
            }
            if k > 0.0 {
                out[i] = (sum.map(|s| s / k), false);
            } else {
                left = true;
            }
        }
        if !left {
            break;
        }
    }
    // The cosine convolution over π: c0 = L00 Y0, c1 = (2/3) Y1 (L11, L1-1, L10).
    let probes = out
        .iter()
        .map(|(sh, _)| {
            [0, 1, 2].map(|ch| {
                [
                    sh[3][ch] * Y1 * (2.0 / 3.0),
                    sh[1][ch] * Y1 * (2.0 / 3.0),
                    sh[2][ch] * Y1 * (2.0 / 3.0),
                    sh[0][ch] * Y0,
                ]
            })
        })
        .collect();
    Volume {
        min: desc.min,
        max: desc.max,
        counts,
        probes,
    }
}

/// The irradiance over π a probe gives a normal (what the shader computes).
pub fn irradiance(probe: &[[f32; 4]; 3], n: Vec3) -> Vec3 {
    let e = |c: &[f32; 4]| (c[3] + c[0] * n.x + c[1] * n.y + c[2] * n.z).max(0.0);
    Vec3::new(e(&probe[0]), e(&probe[1]), e(&probe[2]))
}
