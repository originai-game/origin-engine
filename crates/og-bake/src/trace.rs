//! The path tracer: diffuse paths with next-event estimation towards every
//! light at each vertex past the first, the sky where a path leaves, the
//! emission of whatever it meets.
//!
//! What a lightmap texel or a light volume probe stores is the light that
//! arrives from the scene (bounced light, the sky, emissive surfaces); the
//! direct light of a stationary light is the renderer's, every frame, with
//! its shadows. So [`Tracer::radiance`] never adds a light's direct term at
//! the point it starts from, only at the points its rays meet; a static
//! light's direct term at that point is added by the lightmap and the
//! volumes themselves ([`Tracer::direct`], [`Tracer::arriving`]). The
//! lights are the renderer's own (`og_render_data::light`), falloff and
//! cone included.

use crate::bvh::{Bvh, Hit};
use crate::scene::Scene;
use glam::{Vec2, Vec3};
use og_render_data::light::{LightKind, LightMobility, range_window};
use std::f32::consts::PI;

/// The bake's generator, seeded per texel and per probe so a bake is the
/// same whatever thread draws which texel.
pub use og_core::Pcg32 as Rng;

/// An orthonormal basis around `n` (Duff et al. 2017).
pub fn basis(n: Vec3) -> (Vec3, Vec3) {
    let sign = 1.0f32.copysign(n.z);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;
    (
        Vec3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
        Vec3::new(b, sign + n.y * n.y * a, -n.y),
    )
}

/// A cosine-weighted direction around `n` from two uniform numbers
/// (Shirley and Chiu's concentric map onto the disc, lifted).
pub fn cosine_dir(n: Vec3, u: Vec2) -> Vec3 {
    let a = 2.0 * u - Vec2::ONE;
    let (r, phi) = if a.x == 0.0 && a.y == 0.0 {
        (0.0, 0.0)
    } else if a.x.abs() > a.y.abs() {
        (a.x, PI / 4.0 * (a.y / a.x))
    } else {
        (a.y, PI / 2.0 - PI / 4.0 * (a.x / a.y))
    };
    let (x, y) = (r * phi.cos(), r * phi.sin());
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    let (t, b) = basis(n);
    (t * x + b * y + n * z).normalize()
}

/// The R2 low-discrepancy sequence's `i`th point, shifted by `offset`
/// (Cranley–Patterson): stratified directions for a texel's samples.
pub fn r2(i: u32, offset: Vec2) -> Vec2 {
    const G: f64 = 1.324_717_957_244_746;
    let a1 = 1.0 / G;
    let a2 = 1.0 / (G * G);
    let x = (0.5 + a1 * i as f64).fract() as f32;
    let y = (0.5 + a2 * i as f64).fract() as f32;
    Vec2::new((x + offset.x).fract(), (y + offset.y).fract())
}

/// A surface point a ray met.
#[derive(Clone, Copy, Debug)]
pub struct SurfacePoint {
    pub position: Vec3,
    /// The shading normal, on the side the ray came from for a two-sided
    /// surface.
    pub normal: Vec3,
    /// The geometric normal on the same side.
    pub face: Vec3,
    pub albedo: Vec3,
    pub emissive: Vec3,
    /// The ray met the back of a one-sided surface (inside a closed mesh).
    pub inside: bool,
}

/// The scene with its BVH.
pub struct Tracer<'a> {
    pub scene: &'a Scene,
    bvh: Bvh,
    /// Per triangle: its mesh and corners.
    triangles: Vec<(u32, [u32; 3])>,
    /// How far rays start off a surface, against self-hits: a millionth
    /// of the scene's size, at least a hundredth of a millimetre.
    pub epsilon: f32,
    /// Path vertices past the first a path may bounce from.
    pub bounces: u32,
}

impl<'a> Tracer<'a> {
    pub fn new(scene: &'a Scene, bounces: u32) -> Tracer<'a> {
        let mut corners = Vec::new();
        let mut triangles = Vec::new();
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for (mi, m) in scene.meshes.iter().enumerate() {
            for t in m.indices.as_chunks::<3>().0 {
                let [a, b, c] = [t[0], t[1], t[2]];
                let p = [a, b, c].map(|i| m.positions[i as usize]);
                for q in p {
                    min = min.min(q);
                    max = max.max(q);
                }
                corners.push(p);
                triangles.push((mi as u32, [a, b, c]));
            }
        }
        let size = if corners.is_empty() {
            1.0
        } else {
            (max - min).max_element()
        };
        Tracer {
            scene,
            bvh: Bvh::new(corners),
            triangles,
            epsilon: (size * 1e-5).max(1e-5),
            bounces,
        }
    }

    /// The closest surface along a ray.
    pub fn hit(&self, origin: Vec3, dir: Vec3) -> Option<(Hit, SurfacePoint)> {
        let h = self.bvh.closest(origin, dir, f32::MAX)?;
        Some((h, self.surface(&h, dir)))
    }

    pub fn occluded(&self, origin: Vec3, dir: Vec3, dist: f32) -> bool {
        self.bvh.occluded(origin, dir, dist)
    }

    fn surface(&self, h: &Hit, dir: Vec3) -> SurfacePoint {
        let (mi, [a, b, c]) = self.triangles[h.triangle as usize];
        let m = &self.scene.meshes[mi as usize];
        let mat =
            &self.scene.materials[m.material.min(self.scene.materials.len().saturating_sub(1))];
        let w = Vec3::new(1.0 - h.u - h.v, h.u, h.v);
        let lerp3 = |v: &[Vec3]| v[a as usize] * w.x + v[b as usize] * w.y + v[c as usize] * w.z;
        let position = lerp3(&m.positions);
        let face = (m.positions[b as usize] - m.positions[a as usize])
            .cross(m.positions[c as usize] - m.positions[a as usize])
            .normalize_or(Vec3::Y);
        let mut normal = if m.normals.len() == m.positions.len() {
            lerp3(&m.normals).normalize_or(face)
        } else {
            face
        };
        let uv = if m.uvs.len() == m.positions.len() {
            m.uvs[a as usize] * w.x + m.uvs[b as usize] * w.y + m.uvs[c as usize] * w.z
        } else {
            Vec2::ZERO
        };
        let tint = if m.colors.len() == m.positions.len() {
            lerp3(&m.colors)
        } else {
            Vec3::ONE
        };
        let map = |i: Option<usize>| {
            i.and_then(|i| self.scene.images.get(i))
                .map_or(Vec3::ONE, |img| {
                    let s = img.sample(uv);
                    Vec3::new(s[0], s[1], s[2])
                })
        };
        let albedo =
            mat.base_color * map(mat.base_map) * tint * (1.0 - mat.metallic.clamp(0.0, 1.0));
        let emissive = mat.emissive * map(mat.emissive_map);
        let mut face_n = face;
        let back = face.dot(dir) > 0.0;
        let inside = back && !mat.double_sided;
        if back {
            face_n = -face;
            normal = -normal;
        }
        SurfacePoint {
            position,
            normal,
            face: face_n,
            albedo: albedo.clamp(Vec3::ZERO, Vec3::splat(0.98)),
            emissive: if inside { Vec3::ZERO } else { emissive },
            inside,
        }
    }

    /// The irradiance the scene's lights give at `p` with normal `n`
    /// (shadow rays from `p` lifted along `face`), one sample of each rect
    /// light: every light, or with `static_only` those whose mobility is
    /// static (whose direct light the bake holds).
    pub fn direct(&self, p: Vec3, n: Vec3, face: Vec3, rng: &mut Rng, static_only: bool) -> Vec3 {
        let origin = p + face * self.epsilon * 4.0;
        let mut e = Vec3::ZERO;
        self.arriving(origin, Some(n), static_only, rng, &mut |light, l| {
            e += light * n.dot(l);
        });
        e
    }

    /// Whether a light's direct light is baked (its mobility static).
    pub fn has_static(&self) -> bool {
        let s = |m: LightMobility| m == LightMobility::Static;
        self.scene.sun.is_some_and(|l| s(l.mobility))
            || self.scene.lights.iter().any(|l| s(l.mobility))
    }

    /// Calls `f` with the irradiance each light sends `origin` on a surface
    /// facing it and the unit direction towards it, unshadowed (the renderer's
    /// falloff and cone, `LightDesc::falloff` and `LightDesc::cone`; one
    /// point of each rect light). With `facing`, a light behind that normal
    /// is skipped; with `static_only`, every light but the static ones.
    pub fn arriving(
        &self,
        origin: Vec3,
        facing: Option<Vec3>,
        static_only: bool,
        rng: &mut Rng,
        f: &mut dyn FnMut(Vec3, Vec3),
    ) {
        let keep = |m: LightMobility| !static_only || m == LightMobility::Static;
        let front = |l: Vec3| facing.is_none_or(|n| n.dot(l) > 0.0);
        if let Some(sun) = self.scene.sun.filter(|s| keep(s.mobility)) {
            let l = sun.direction_to_light.normalize_or(Vec3::Y);
            if front(l) && !self.occluded(origin, l, f32::MAX) {
                f(sun.color * sun.intensity, l);
            }
        }
        for light in self.scene.lights.iter().filter(|l| keep(l.mobility)) {
            let (to, area_term) = match light.kind {
                LightKind::Rect {
                    half_width,
                    half_height,
                    two_sided,
                } => {
                    let cross = half_width.cross(half_height);
                    let area = 4.0 * cross.length();
                    if area <= 0.0 {
                        continue;
                    }
                    let nl = -cross / (area / 4.0);
                    let q = light.position
                        + half_width * (2.0 * rng.next_f32() - 1.0)
                        + half_height * (2.0 * rng.next_f32() - 1.0);
                    let w = q - origin;
                    let d2 = w.length_squared().max(1e-8);
                    let mut cos_l = nl.dot(-w / d2.sqrt());
                    if two_sided {
                        cos_l = cos_l.abs();
                    }
                    if cos_l <= 0.0 {
                        continue;
                    }
                    // Faded to nothing at its range from its centre, as
                    // area.wgsl does.
                    let window =
                        range_window((light.position - origin).length_squared(), light.range);
                    (w, area * cos_l / d2 * window)
                }
                LightKind::Point | LightKind::Spot { .. } => {
                    let to = light.position - origin;
                    let d2 = to.length_squared().max(1e-4);
                    (to, light.falloff(d2) * light.cone(to / d2.sqrt()))
                }
            };
            let d = to.length().max(1e-4);
            let l = to / d;
            if area_term <= 0.0 || !front(l) {
                continue;
            }
            let reach = match light.kind {
                LightKind::Rect { .. } => d - self.epsilon * 8.0,
                _ => d,
            };
            if !self.occluded(origin, l, reach) {
                f(light.radiance() * area_term, l);
            }
        }
    }

    /// The radiance arriving at `origin` from `dir`: what the ray meets
    /// emits, plus that surface's diffuse response to the lights and to
    /// the light arriving at it in turn, up to `bounces` vertices, the sky
    /// where it escapes. The second answer: the ray's first surface was
    /// the back of a one-sided one (the start lies inside a mesh).
    pub fn radiance(&self, origin: Vec3, dir: Vec3, rng: &mut Rng) -> (Vec3, bool) {
        let mut throughput = Vec3::ONE;
        let mut out = Vec3::ZERO;
        let (mut o, mut d) = (origin, dir);
        for bounce in 0..=self.bounces {
            let Some((_, s)) = self.hit(o, d) else {
                out += throughput * self.scene.sky;
                break;
            };
            if s.inside {
                return (out, bounce == 0);
            }
            out += throughput * s.emissive;
            if bounce == self.bounces {
                break;
            }
            out +=
                throughput * s.albedo / PI * self.direct(s.position, s.normal, s.face, rng, false);
            // The next direction is cosine-weighted, so the estimator of the
            // diffuse integral is the albedo alone.
            throughput *= s.albedo;
            if bounce >= 2 {
                let keep = throughput.max_element().clamp(0.05, 1.0);
                if rng.next_f32() >= keep {
                    break;
                }
                throughput /= keep;
            }
            o = s.position + s.face * self.epsilon * 4.0;
            d = cosine_dir(s.normal, Vec2::new(rng.next_f32(), rng.next_f32()));
            if d.dot(s.face) <= 0.0 {
                d = s.face;
            }
        }
        (out, false)
    }
}
