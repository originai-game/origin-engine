//! Octahedral impostors (R6): a model far away drawn as one camera-facing
//! card that shows the model as it looks from the camera's direction.
//!
//! **The atlas.** The cook renders the model (`model { impostor true }`,
//! og-cook's `impostor.rs`) orthographically from `grid × grid`
//! directions spread over the whole sphere by an octahedral map with +y
//! at the grid's centre and −y at its corners: frame `(i, j)` (column,
//! row) looks along [`frame_direction`]. Each frame is a square of the
//! atlas, `1 / grid` of its side, covering the model's bounding sphere
//! (`center`, `radius`) in the frame's [`frame_basis`]: its right `x`, its
//! up `y`, `v` down the image. Two textures: the albedo with its coverage
//! in alpha (sRGB), and the surface's normal in the frame's basis (`rgb`,
//! `n · 0.5 + 0.5`) with its depth towards the viewer (`a`, `0.5 + 0.5 ×
//! depth / radius`).
//!
//! **Drawing.** An [`ImpostorDraw`] is a quad of the sphere's size about
//! its centre, turned to face the eye (in the shadow passes, the light).
//! Its vertex stage finds the three frames around the eye's direction on
//! the grid and their weights ([`blend_frames`]); its fragment stage
//! samples the three where the eye's ray through the pixel meets each
//! frame's plane, blends their albedo and normals by weight and coverage,
//! cuts at half coverage and lights the result as a scattered copy is lit
//! (`shaders/impostor.wgsl`). The impostor's material is a
//! StandardMaterial whose map is the albedo and whose normal map is the
//! normal atlas, bound in the material group the copies use: the lit
//! stage reads no texture it did not before.
//!
//! **Who draws one.** og-runtime picks a model's impostor as the last
//! level of its chain (`lod.rs`): the model's levels' errors end with the
//! impostor's, the size of one atlas texel on the model (`2 × radius /
//! frame pixels`), so a model switches to its impostor once a texel of it
//! covers less than a pixel; a chain's errors only rise, so the impostor
//! replaces the mesh levels coarser than a texel (drawn where the
//! coarsest of them would be). A [`crate::ScatterDraw`] with an impostor
//! draws its copies past [`crate::ScatterDraw::impostor_distance`] as
//! impostors (dithered across the last tenth before it).

use crate::scene::MaterialHandle;
use glam::{Mat4, Vec2, Vec3};

/// The most frames a side of the grid holds.
pub const MAX_IMPOSTOR_GRID: u32 = 32;

/// A model's impostor: its atlas as a material and how it was baked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impostor {
    /// A StandardMaterial: `map` the albedo atlas, `normalMap` the normal
    /// and depth atlas, `alphaTest` the coverage cut (0.5).
    pub material: MaterialHandle,
    /// Frames per side of the atlas (2 to [`MAX_IMPOSTOR_GRID`]).
    pub grid: u32,
    /// The bounding sphere the frames cover, in the model's space.
    pub center: Vec3,
    pub radius: f32,
}

/// One impostor drawn this frame ([`crate::RenderScene::impostors`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpostorDraw {
    pub impostor: Impostor,
    /// The model's world transform.
    pub transform: Mat4,
    pub cast_shadows: bool,
}

/// The octahedral map of a unit direction to `[-1, 1]²`, +y at the centre
/// and −y at the corners.
pub fn oct_encode(d: Vec3) -> Vec2 {
    let n = d / (d.x.abs() + d.y.abs() + d.z.abs()).max(1e-12);
    if n.y >= 0.0 {
        Vec2::new(n.x, n.z)
    } else {
        let s = |v: f32| if v >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new((1.0 - n.z.abs()) * s(n.x), (1.0 - n.x.abs()) * s(n.z))
    }
}

/// The inverse of [`oct_encode`].
pub fn oct_decode(p: Vec2) -> Vec3 {
    let y = 1.0 - p.x.abs() - p.y.abs();
    let (mut x, mut z) = (p.x, p.y);
    if y < 0.0 {
        let s = |v: f32| if v >= 0.0 { 1.0 } else { -1.0 };
        x = (1.0 - p.y.abs()) * s(p.x);
        z = (1.0 - p.x.abs()) * s(p.y);
    }
    Vec3::new(x, y, z).normalize()
}

/// The direction frame `(i, j)` of a `grid`-sided atlas looks from (from
/// the model's centre towards the viewer).
pub fn frame_direction(i: u32, j: u32, grid: u32) -> Vec3 {
    let n = (grid.max(2) - 1) as f32;
    oct_decode(Vec2::new(i as f32 / n, j as f32 / n) * 2.0 - 1.0)
}

/// A frame's right and up for a view from `d` (from the centre towards
/// the viewer): up is the one nearest +y (+z looking straight down or up).
pub fn frame_basis(d: Vec3) -> (Vec3, Vec3) {
    let reference = if d.y.abs() > 0.999 { Vec3::Z } else { Vec3::Y };
    let right = reference.cross(d).normalize();
    (right, d.cross(right))
}

/// The three frames around the direction `d` and their weights (summing
/// to 1): the triangle of the grid cell holding `d`'s point. As the vertex
/// stage of `impostor.wgsl` finds them.
pub fn blend_frames(d: Vec3, grid: u32) -> [((u32, u32), f32); 3] {
    let n = (grid.max(2) - 1) as f32;
    let g = ((oct_encode(d) * 0.5 + 0.5) * n).clamp(Vec2::ZERO, Vec2::splat(n));
    let cell = g.floor().min(Vec2::splat(n - 1.0));
    let f = g - cell;
    let (cx, cz) = (cell.x as u32, cell.y as u32);
    if f.x + f.y <= 1.0 {
        [
            ((cx, cz), 1.0 - f.x - f.y),
            ((cx + 1, cz), f.x),
            ((cx, cz + 1), f.y),
        ]
    } else {
        [
            ((cx + 1, cz + 1), f.x + f.y - 1.0),
            ((cx, cz + 1), 1.0 - f.x),
            ((cx + 1, cz), 1.0 - f.y),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_octahedral_map_round_trips() {
        for d in [
            Vec3::Y,
            -Vec3::Y,
            Vec3::X,
            Vec3::new(0.3, -0.8, -0.5).normalize(),
            Vec3::new(-0.7, 0.1, 0.7).normalize(),
            Vec3::new(0.2, -0.1, -0.97).normalize(),
        ] {
            let back = oct_decode(oct_encode(d));
            assert!((back - d).length() < 1e-5, "{d} came back {back}");
        }
        // +y at the centre, −y at every corner.
        assert_eq!(oct_encode(Vec3::Y), Vec2::ZERO);
        assert!((oct_decode(Vec2::new(1.0, -1.0)) + Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn frames_span_the_sphere_and_blend_to_the_direction() {
        // An 8-sided grid: the centre four frames look down from above,
        // the corners from straight below, the middle of an edge sideways.
        assert!(frame_direction(3, 3, 8).y > 0.6);
        assert!((frame_direction(0, 0, 8) + Vec3::Y).length() < 1e-6);
        assert!(frame_direction(0, 4, 8).y.abs() < 0.3);
        // A direction on a frame weighs that frame alone.
        let d = frame_direction(5, 2, 8);
        let w = blend_frames(d, 8);
        let on: Vec<_> = w.iter().filter(|(_, x)| *x > 1e-4).collect();
        assert_eq!(on.len(), 1);
        assert_eq!(on[0].0, (5, 2));
        // Between frames: three of them, weights summing to 1, the
        // nearest weighing most; their directions blend to near d.
        let d = Vec3::new(0.4, 0.3, -0.6).normalize();
        let w = blend_frames(d, 8);
        assert!((w.iter().map(|x| x.1).sum::<f32>() - 1.0).abs() < 1e-5);
        let mix: Vec3 = w
            .iter()
            .map(|&((i, j), x)| frame_direction(i, j, 8) * x)
            .sum();
        assert!(mix.normalize().dot(d) > 0.98);
        let best = w.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().0;
        let nearest = (0..8)
            .flat_map(|i| (0..8).map(move |j| (i, j)))
            .max_by(|a, b| {
                frame_direction(a.0, a.1, 8)
                    .dot(d)
                    .total_cmp(&frame_direction(b.0, b.1, 8).dot(d))
            })
            .unwrap();
        assert_eq!(best, nearest);
    }

    #[test]
    fn a_frame_basis_is_right_handed_and_upright() {
        for d in [
            Vec3::X,
            Vec3::new(0.3, 0.5, -0.8).normalize(),
            Vec3::Y,
            -Vec3::Y,
        ] {
            let (r, u) = frame_basis(d);
            assert!(r.dot(d).abs() < 1e-5 && u.dot(d).abs() < 1e-5 && r.dot(u).abs() < 1e-5);
            assert!((r.cross(u) - d).length() < 1e-5);
        }
        // Seen from the side, up is +y and right is horizontal.
        let (r, u) = frame_basis(Vec3::Z);
        assert!((u - Vec3::Y).length() < 1e-6 && (r - Vec3::X).length() < 1e-6);
    }
}
