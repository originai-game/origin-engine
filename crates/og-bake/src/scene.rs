//! What the tracer sees: the static geometry in world space with its
//! materials, the lights, and the sky.
//!
//! Units are the renderer's (docs/design.md §2.6): a light's `radiance` is
//! its colour times its intensity, a surface lit with irradiance E sends out
//! `albedo / π × E`, a point or spot light gives `radiance × window² / d²`
//! (window = clamp(1 − (d / range)⁴), UE4's falloff, as pbr.wgsl), a
//! directional light `radiance` over the plane facing it.

use glam::{Vec2, Vec3};

/// An RGBA8 image a material samples (rows top to bottom, uv (0, 0) at its
/// top left, repeat addressing, bilinear).
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    /// Linear RGBA, one f32 per channel (sRGB decoded on load).
    pub texels: Vec<[f32; 4]>,
}

impl Image {
    /// Bilinear, repeating.
    pub fn sample(&self, uv: Vec2) -> [f32; 4] {
        if self.width == 0 || self.height == 0 {
            return [1.0; 4];
        }
        let (w, h) = (self.width as i64, self.height as i64);
        let x = uv.x * w as f32 - 0.5;
        let y = uv.y * h as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |i: i64, j: i64| self.texels[(j.rem_euclid(h) * w + i.rem_euclid(w)) as usize];
        let (x0, y0) = (x0 as i64, y0 as i64);
        let (a, b, c, d) = (
            at(x0, y0),
            at(x0 + 1, y0),
            at(x0, y0 + 1),
            at(x0 + 1, y0 + 1),
        );
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * fx;
            let bottom = c[k] + (d[k] - c[k]) * fx;
            top + (bottom - top) * fy
        })
    }
}

/// A surface's diffuse response and emission.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Linear base colour (times the map and the vertex colour).
    pub base_color: Vec3,
    pub base_map: Option<usize>,
    /// Metalness: a metal reflects no diffuse light.
    pub metallic: f32,
    /// Emitted radiance (linear, times the map).
    pub emissive: Vec3,
    pub emissive_map: Option<usize>,
    /// Lit and hit from both sides (an open surface).
    pub double_sided: bool,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vec3::ONE,
            base_map: None,
            metallic: 0.0,
            emissive: Vec3::ZERO,
            emissive_map: None,
            double_sided: false,
        }
    }
}

/// One mesh in world space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// The material's uvs (empty: none).
    pub uvs: Vec<Vec2>,
    /// Linear vertex colours (empty: white).
    pub colors: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub material: usize,
    /// Whether the mesh receives a lightmap (a static entity's part); every
    /// mesh casts and bounces light.
    pub lightmapped: bool,
}

/// A light the bake traces towards from every path vertex past the first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Light {
    /// Parallel light travelling along `-towards`.
    Directional { towards: Vec3, radiance: Vec3 },
    Point {
        position: Vec3,
        radiance: Vec3,
        range: f32,
    },
    /// A cone along `direction`: cos of its outer and inner half-angles.
    Spot {
        position: Vec3,
        direction: Vec3,
        radiance: Vec3,
        range: f32,
        cos_outer: f32,
        cos_inner: f32,
    },
    /// A rectangle centred at `position` spanning ±`half_x` and ±`half_y`,
    /// emitting `radiance` from its front (`half_x × half_y`'s direction
    /// reversed: along the entity's -Z) or both faces.
    Rect {
        position: Vec3,
        half_x: Vec3,
        half_y: Vec3,
        radiance: Vec3,
        two_sided: bool,
    },
}

/// Everything the bake lights and traces.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub images: Vec<Image>,
    pub lights: Vec<Light>,
    /// The radiance of every ray that leaves the scene.
    pub sky: Vec3,
}
