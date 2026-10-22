//! The path tracer: diffuse paths with next-event estimation towards every
//! light at each vertex past the first, the sky where a path leaves, the
//! emission of whatever it meets.
//!
//! What a lightmap texel or a light volume probe stores is the light that
//! arrives from the scene (bounced light, the sky, emissive surfaces); the
//! direct light of the scene's lights is the renderer's, every frame, with
//! its shadows. So [`Tracer::radiance`] never adds a light's direct term at
//! the point it starts from, only at the points its rays meet.

use crate::bvh::{Bvh, Hit};
use crate::scene::{Light, Scene};
use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// A small fast generator (PCG32), seeded per texel and per probe so a
/// bake is the same whatever thread draws which texel.
#[derive(Clone, Copy, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut r = Rng {
            state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03,
        };
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}

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

    /// The irradiance every light gives at `p` with normal `n` (shadow rays
    /// from `p` lifted along `face`), one sample of each area light.
    pub fn direct(&self, p: Vec3, n: Vec3, face: Vec3, rng: &mut Rng) -> Vec3 {
        let origin = p + face * self.epsilon * 4.0;
        let mut e = Vec3::ZERO;
        for light in &self.scene.lights {
            match *light {
                Light::Directional { towards, radiance } => {
                    let l = towards.normalize_or(Vec3::Y);
                    let cos = n.dot(l);
                    if cos > 0.0 && !self.occluded(origin, l, f32::MAX) {
                        e += radiance * cos;
                    }
                }
                Light::Point {
                    position,
                    radiance,
                    range,
                } => {
                    e += radiance * self.local(origin, n, position, range, None);
                }
                Light::Spot {
                    position,
                    direction,
                    radiance,
                    range,
                    cos_outer,
                    cos_inner,
                } => {
                    let cone = (direction.normalize_or(Vec3::NEG_Y), cos_outer, cos_inner);
                    e += radiance * self.local(origin, n, position, range, Some(cone));
                }
                Light::Rect {
                    position,
                    half_x,
                    half_y,
                    radiance,
                    two_sided,
                } => {
                    let cross = half_x.cross(half_y);
                    let area = 4.0 * cross.length();
                    if area <= 0.0 {
                        continue;
                    }
                    let nl = -cross / (area / 4.0);
                    let q = position
                        + half_x * (2.0 * rng.f32() - 1.0)
                        + half_y * (2.0 * rng.f32() - 1.0);
                    let w = q - origin;
                    let d2 = w.length_squared().max(1e-8);
                    let d = d2.sqrt();
                    let l = w / d;
                    let cos_s = n.dot(l);
                    let mut cos_l = nl.dot(-l);
                    if two_sided {
                        cos_l = cos_l.abs();
                    }
                    if cos_s > 0.0
                        && cos_l > 0.0
                        && !self.occluded(origin, l, d - self.epsilon * 8.0)
                    {
                        e += radiance * (area * cos_s * cos_l / d2);
                    }
                }
            }
        }
        e
    }

    /// A point or spot light's falloff times n·l at `p` (0 when shadowed),
    /// as pbr.wgsl's `local_light`.
    fn local(&self, p: Vec3, n: Vec3, at: Vec3, range: f32, cone: Option<(Vec3, f32, f32)>) -> f32 {
        let to = at - p;
        let d2 = to.length_squared().max(1e-4);
        let ratio = d2 / (range * range).max(1e-8);
        let window = (1.0 - ratio * ratio).clamp(0.0, 1.0);
        let mut atten = window * window / d2;
        let d = d2.sqrt();
        let l = to / d;
        if let Some((axis, cos_outer, cos_inner)) = cone {
            let t = ((-l).dot(axis) - cos_outer) / (cos_inner - cos_outer).max(1e-4);
            let t = t.clamp(0.0, 1.0);
            atten *= t * t;
        }
        let cos = n.dot(l);
        if atten <= 0.0 || cos <= 0.0 || self.occluded(p, l, d) {
            return 0.0;
        }
        atten * cos
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
            out += throughput * s.albedo / PI * self.direct(s.position, s.normal, s.face, rng);
            // The next direction is cosine-weighted, so the estimator of the
            // diffuse integral is the albedo alone.
            throughput *= s.albedo;
            if bounce >= 2 {
                let keep = throughput.max_element().clamp(0.05, 1.0);
                if rng.f32() >= keep {
                    break;
                }
                throughput /= keep;
            }
            o = s.position + s.face * self.epsilon * 4.0;
            d = cosine_dir(s.normal, Vec2::new(rng.f32(), rng.f32()));
            if d.dot(s.face) <= 0.0 {
                d = s.face;
            }
        }
        (out, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Material, Mesh};

    fn quad(center: Vec3, half_x: Vec3, half_y: Vec3, material: usize) -> Mesh {
        let n = half_x.cross(half_y).normalize();
        let p = [
            center - half_x - half_y,
            center + half_x - half_y,
            center + half_x + half_y,
            center - half_x + half_y,
        ];
        Mesh {
            positions: p.to_vec(),
            normals: vec![n; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            material,
            ..Default::default()
        }
    }

    #[test]
    fn an_open_sky_gives_its_radiance_from_every_direction() {
        let scene = Scene {
            sky: Vec3::new(0.2, 0.3, 0.4),
            ..Default::default()
        };
        let t = Tracer::new(&scene, 3);
        let mut rng = Rng::new(1);
        let (l, inside) = t.radiance(Vec3::ZERO, Vec3::Y, &mut rng);
        assert_eq!(l, scene.sky);
        assert!(!inside);
    }

    /// A white floor under a sun: a ray from above meets it and returns
    /// albedo / π × E, E the sun's radiance times cos θ.
    #[test]
    fn a_lit_floor_returns_its_diffuse_radiance() {
        let scene = Scene {
            meshes: vec![quad(Vec3::ZERO, Vec3::X * 10.0, Vec3::NEG_Z * 10.0, 0)],
            materials: vec![Material {
                base_color: Vec3::splat(0.5),
                ..Default::default()
            }],
            lights: vec![Light::Directional {
                towards: Vec3::new(0.0, 1.0, 1.0).normalize(),
                radiance: Vec3::splat(3.0),
            }],
            ..Default::default()
        };
        let t = Tracer {
            bounces: 1,
            ..Tracer::new(&scene, 1)
        };
        let mut rng = Rng::new(2);
        let (l, _) = t.radiance(Vec3::new(0.0, 1.0, 0.0), Vec3::NEG_Y, &mut rng);
        let want = 0.5 / PI * 3.0 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((l.x - want).abs() < 1e-4, "{l} vs {want}");
    }

    /// A point light's falloff is pbr.wgsl's: window² / d² with the window
    /// clamp(1 - (d / range)^4), times n·l.
    #[test]
    fn a_point_light_falls_off_as_the_renderer_draws_it() {
        let scene = Scene {
            meshes: vec![],
            materials: vec![Material::default()],
            lights: vec![Light::Point {
                position: Vec3::new(0.0, 2.0, 0.0),
                radiance: Vec3::splat(10.0),
                range: 4.0,
            }],
            ..Default::default()
        };
        let t = Tracer::new(&scene, 1);
        let mut rng = Rng::new(3);
        let e = t.direct(Vec3::ZERO, Vec3::Y, Vec3::Y, &mut rng);
        let d2: f32 = 4.0;
        let ratio = d2 / 16.0;
        let window = 1.0 - ratio * ratio;
        let want = 10.0 * window * window / d2;
        assert!((e.x - want).abs() < 1e-3 * want, "{e} vs {want}");
    }

    /// A rect light's irradiance on the point below its centre: one-sided
    /// it lights only below its front (its -Z: half_x × half_y reversed).
    #[test]
    fn a_rect_light_lights_its_front_only() {
        let rect = |two_sided| Light::Rect {
            position: Vec3::new(0.0, 1.0, 0.0),
            // half_x × half_y = +y, so it emits downwards.
            half_x: Vec3::new(0.05, 0.0, 0.0),
            half_y: Vec3::new(0.0, 0.0, -0.05),
            radiance: Vec3::splat(100.0),
            two_sided,
        };
        let scene = Scene {
            materials: vec![Material::default()],
            lights: vec![rect(false)],
            ..Default::default()
        };
        let t = Tracer::new(&scene, 1);
        let mut rng = Rng::new(4);
        let mut below = 0.0;
        let mut above = 0.0;
        for _ in 0..256 {
            below += t.direct(Vec3::ZERO, Vec3::Y, Vec3::Y, &mut rng).x;
            above += t
                .direct(Vec3::new(0.0, 2.0, 0.0), Vec3::NEG_Y, Vec3::NEG_Y, &mut rng)
                .x;
        }
        // A small rect: E ≈ L × A / d² = 100 × 0.01 / 1.
        assert!((below / 256.0 - 1.0).abs() < 0.02, "{}", below / 256.0);
        assert_eq!(above, 0.0);
        let scene2 = Scene {
            lights: vec![rect(true)],
            ..scene
        };
        let t2 = Tracer::new(&scene2, 1);
        let up = t2
            .direct(Vec3::new(0.0, 2.0, 0.0), Vec3::NEG_Y, Vec3::NEG_Y, &mut rng)
            .x;
        assert!(up > 0.9, "two-sided lights its back too ({up})");
    }

    #[test]
    fn a_ray_inside_a_closed_box_reports_it() {
        // The floor seen from below its front: its back.
        let scene = Scene {
            meshes: vec![quad(Vec3::ZERO, Vec3::X, Vec3::NEG_Z, 0)],
            materials: vec![Material::default()],
            ..Default::default()
        };
        let t = Tracer::new(&scene, 2);
        let mut rng = Rng::new(5);
        let (_, inside) = t.radiance(Vec3::new(0.0, -1.0, 0.0), Vec3::Y, &mut rng);
        assert!(inside);
    }

    #[test]
    fn cosine_directions_stay_above_the_surface_and_average_its_normal() {
        let n = Vec3::new(0.3, 0.8, -0.2).normalize();
        let mut mean = Vec3::ZERO;
        for i in 0..4096 {
            let d = cosine_dir(n, r2(i, Vec2::new(0.13, 0.71)));
            assert!(d.dot(n) >= -1e-5);
            mean += d;
        }
        // E[d] for a cosine lobe is n × 2/3.
        let mean = mean / 4096.0;
        assert!((mean - n * (2.0 / 3.0)).length() < 0.01, "{mean}");
    }
}
