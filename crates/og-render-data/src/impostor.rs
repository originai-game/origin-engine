//! Octahedral impostors (R6): a model far away drawn as one camera-facing
//! card that shows the model as it looks from the camera's direction.
//!
//! **The atlas.** The cook renders the model (`model { impostor true }`,
//! og-cook's `impostor.rs`) orthographically from `grid × grid`
//! directions spread over the upper hemisphere by a hemi-octahedral map
//! with +y at the grid's centre and the horizon on its border (+x and −x
//! at opposite corners): frame `(i, j)` (column, row) looks along
//! [`frame_direction`]. (A full octahedron put the horizon between
//! rows, so a model seen level, as most standing models are, blended
//! frames from below it: a palm's fronds showed their undersides, lit
//! from beneath, and read dark beside the meshes.)
//!
//! **From below.** A model also seen from below its horizon (on a cliff
//! over the camera, hanging, flying) is cooked with `impostorBelow true`
//! ([`Impostor::below`]): the atlases are then twice as wide, their right
//! half the lower hemisphere's frames, each the mirror in y of the frame
//! left of it (column `i + grid` looks along frame `i`'s direction with
//! its y negated), so the horizon's frames stand on both halves' borders
//! and a view never blends frames across it. Without that half a view
//! from below draws as the view from the horizon above it, on a card
//! standing upright: whole, and flatter the steeper the view.
//!
//! Each frame is a square of the
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
    /// Whether `material`'s roughness map is the cook's material atlas
    /// (`#impostor-material`: r roughness, g translucency, b metallic),
    /// which the cards are shaded by (G5); else its factors, opaque.
    pub material_atlas: bool,
    /// Whether the atlases hold the lower hemisphere's frames in their
    /// right half ([module docs](self), `impostorBelow`).
    pub below: bool,
}

/// One impostor drawn this frame ([`crate::RenderScene::impostors`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpostorDraw {
    pub impostor: Impostor,
    /// The model's world transform.
    pub transform: Mat4,
    pub cast_shadows: bool,
    /// How much the wind sways it (`og.Model.sway`; 0: still) and the
    /// model's top above its origin, which the sway reaches: the card
    /// leans as the model's meshes do nearer, by the wind every
    /// vegetation tier reads.
    pub sway: f32,
    pub top: f32,
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
/// the model's centre towards the viewer). Columns from `grid` on are the
/// lower hemisphere's ([`Impostor::below`]): frame `(i - grid, j)`'s
/// direction with its y negated.
pub fn frame_direction(i: u32, j: u32, grid: u32) -> Vec3 {
    let grid = grid.max(2);
    let n = (grid - 1) as f32;
    let d = hemi_oct_decode(Vec2::new((i % grid) as f32 / n, j as f32 / n) * 2.0 - 1.0);
    if i >= grid {
        Vec3::new(d.x, -d.y, d.z)
    } else {
        d
    }
}

/// A frame's right and up for a view from `d` (from the centre towards
/// the viewer): up is the one nearest +y (+z looking straight down or up).
pub fn frame_basis(d: Vec3) -> (Vec3, Vec3) {
    let reference = if d.y.abs() > 0.999 { Vec3::Z } else { Vec3::Y };
    let right = reference.cross(d).normalize();
    (right, d.cross(right))
}

/// The three frames around the direction `d` and their weights (summing
/// to 1): the triangle of the grid cell holding `d`'s point. With `below`
/// ([`Impostor::below`]) a direction under the horizon takes the lower
/// hemisphere's frames; without, the horizon's above it. As the vertex
/// stage of `impostor.wgsl` finds them.
pub fn blend_frames(d: Vec3, grid: u32, below: bool) -> [((u32, u32), f32); 3] {
    let grid = grid.max(2);
    let n = (grid - 1) as f32;
    let lower = below && d.y < 0.0;
    let up = Vec3::new(d.x, if lower { -d.y } else { d.y }, d.z);
    let g = ((hemi_oct_encode(up) * 0.5 + 0.5) * n).clamp(Vec2::ZERO, Vec2::splat(n));
    let cell = g.floor().min(Vec2::splat(n - 1.0));
    let f = g - cell;
    let (cx, cz) = (cell.x as u32 + if lower { grid } else { 0 }, cell.y as u32);
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
