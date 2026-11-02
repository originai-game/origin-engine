//! Octahedral impostors (R6): a model far away drawn as one camera-facing
//! card that shows the model as it looks from the camera's direction.
//!
//! **The atlas.** The cook renders the model (`model { impostor true }`,
//! og-cook's `impostor.rs`) orthographically from `grid × grid`
//! directions spread over the upper hemisphere by a hemi-octahedral map
//! with +y at the grid's centre and the horizon on its border (+x and −x
//! at opposite corners): frame `(i, j)` (column, row) looks along
//! [`frame_direction`]. A view from below the horizon draws as the view
//! from the horizon above it. (A full octahedron put the horizon between
//! rows, so a model seen level, as most standing models are, blended
//! frames from below it: a palm's fronds showed their undersides, lit
//! from beneath, and read dark beside the meshes.) Each frame is a square of the
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
//! samples each where that frame sees the card's point (its orthographic
//! image along the frame's direction, moved by one parallax step on the
//! frame's depth to where the eye's ray meets the surface), blends their
//! albedo, normals and depth by weight and coverage, cuts at half coverage
//! (alpha to coverage under MSAA), writes the surface's depth rather than
//! the card's and lights the result as a scattered copy is lit
//! (`shaders/impostor.wgsl`). A scatter's copy crossing to its impostor
//! draws the dither pixels its mesh drops, and only those. The impostor's material is a
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

/// The hemi-octahedral map of a unit direction to `[-1, 1]²`: +y at the
/// centre, the horizon on the border (+x at `(1, 1)`, +z at `(1, -1)`).
/// A direction below the horizon maps as the horizon above it (straight
/// down as straight up).
pub fn hemi_oct_encode(d: Vec3) -> Vec2 {
    let d = Vec3::new(d.x, d.y.max(0.0), d.z);
    let d = if d.length_squared() < 1e-12 {
        Vec3::Y
    } else {
        d
    };
    let n = d / (d.x.abs() + d.y + d.z.abs());
    Vec2::new(n.x + n.z, n.x - n.z)
}

/// The inverse of [`hemi_oct_encode`] (a direction with y ≥ 0).
pub fn hemi_oct_decode(p: Vec2) -> Vec3 {
    let (x, z) = ((p.x + p.y) * 0.5, (p.x - p.y) * 0.5);
    Vec3::new(x, (1.0 - x.abs() - z.abs()).max(0.0), z).normalize()
}

/// The direction frame `(i, j)` of a `grid`-sided atlas looks from (from
/// the model's centre towards the viewer).
pub fn frame_direction(i: u32, j: u32, grid: u32) -> Vec3 {
    let n = (grid.max(2) - 1) as f32;
    hemi_oct_decode(Vec2::new(i as f32 / n, j as f32 / n) * 2.0 - 1.0)
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
    let g = ((hemi_oct_encode(d) * 0.5 + 0.5) * n).clamp(Vec2::ZERO, Vec2::splat(n));
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
    fn the_hemi_octahedral_map_round_trips_above_the_horizon() {
        for d in [
            Vec3::Y,
            Vec3::X,
            -Vec3::Z,
            Vec3::new(0.3, 0.8, -0.5).normalize(),
            Vec3::new(-0.7, 0.1, 0.7).normalize(),
            Vec3::new(0.2, 0.01, -0.97).normalize(),
        ] {
            let back = hemi_oct_decode(hemi_oct_encode(d));
            assert!((back - d).length() < 1e-5, "{d} came back {back}");
        }
        // +y at the centre, the horizon on the border.
        assert_eq!(hemi_oct_encode(Vec3::Y), Vec2::ZERO);
        assert!((hemi_oct_encode(Vec3::X) - Vec2::ONE).length() < 1e-6);
        assert!((hemi_oct_encode(Vec3::Z) - Vec2::new(1.0, -1.0)).length() < 1e-6);
        // Below the horizon: the horizon above it; straight down, up.
        let below = Vec3::new(0.6, -0.5, 0.2).normalize();
        let level = Vec3::new(0.6, 0.0, 0.2).normalize();
        assert!((hemi_oct_encode(below) - hemi_oct_encode(level)).length() < 1e-6);
        assert_eq!(hemi_oct_encode(-Vec3::Y), Vec2::ZERO);
    }

    #[test]
    fn frames_span_the_hemisphere_and_blend_to_the_direction() {
        // An 8-sided grid: the centre four frames look down from above,
        // the border's from the horizon, a corner along ±x.
        assert!(frame_direction(3, 3, 8).y > 0.6);
        assert!((frame_direction(7, 7, 8) - Vec3::X).length() < 1e-6);
        assert!((frame_direction(0, 0, 8) + Vec3::X).length() < 1e-6);
        for k in 0..8 {
            assert!(frame_direction(k, 0, 8).y.abs() < 1e-6);
            assert!(frame_direction(0, k, 8).y.abs() < 1e-6);
        }
        // A level view blends frames on the horizon and above it only.
        for &((i, j), w) in &blend_frames(Vec3::new(0.3, 0.02, 0.95).normalize(), 8) {
            assert!(w < 1e-4 || frame_direction(i, j, 8).y >= -1e-6);
        }
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
