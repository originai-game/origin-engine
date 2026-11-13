//! Decals: boxes that project a material onto what they cover.

use super::*;

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

/// The most decals one frame draws (the rest, in scene order, are counted
/// in [`crate::FrameStats::decals_dropped`]). Decals reach the lit shader
/// through storage buffers grown to the frame's count and are binned into
/// the light clusters' grid ([`crate::CLUSTER_GRID`]), so a fragment walks
/// only the decals whose box can cover it.
pub const MAX_DECALS: usize = 65_536;

/// The most decals one cluster blends (the newest, by scene order, are kept;
/// the rest there are counted in [`crate::FrameStats::decals_dropped`]).
pub const MAX_CLUSTER_DECALS: usize = 255;

/// A projected decal (scorch marks, blood, posters, bullet holes): a unit
/// box (-0.5..0.5 on each axis) placed by `transform`, projected along its
/// local -z onto every lit opaque surface inside it, in the forward pass
/// before lighting, so the decal is shaded with the surface. Its image is
/// `uv_rect` of [`crate::RenderScene::decal_atlas`] (or a solid colour
/// without an atlas); with a `normal_rect`, the same atlas's tangent-space
/// normal map there bends the surface's normal (a crack, a bullet hole's
/// rim, embossed lettering). Later decals blend over earlier ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decal {
    pub transform: Mat4,
    /// `[u0, v0, u1, v1]` in the atlas, v down (local +x is u, +y is up).
    pub uv_rect: [f32; 4],
    /// Linear RGBA multiplied with the atlas; alpha is the opacity.
    pub color: [f32; 4],
    /// Linear emitted light added where the decal covers (glowing runes, lava).
    pub emissive: Vec3,
    /// Surface roughness under the decal (negative: keep the surface's).
    pub roughness: f32,
    /// Surfaces facing away from the projection direction by more than this
    /// angle (degrees) are not covered; the last 15° fade.
    pub angle_fade_degrees: f32,
    /// `[u0, v0, u1, v1]` of a tangent-space normal map in the atlas (glTF's
    /// convention: +x along u, +y up the image); an empty rect (the
    /// default) leaves the surface's normal. An sRGB atlas is read back to
    /// its stored values first.
    pub normal_rect: [f32; 4],
    /// The normal map's strength, as three.js `normalScale` (x and y both).
    pub normal_scale: f32,
}

impl Default for Decal {
    fn default() -> Self {
        Decal {
            transform: Mat4::IDENTITY,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: [1.0; 4],
            emissive: Vec3::ZERO,
            roughness: -1.0,
            angle_fade_degrees: 75.0,
            normal_rect: [0.0; 4],
            normal_scale: 1.0,
        }
    }
}

impl Decal {
    /// A decal lying on a surface at `position` facing along `normal`
    /// (projected against it), `size` wide and long, `depth` thick, turned by
    /// `rotation` radians about the normal.
    pub fn on_surface(
        position: Vec3,
        normal: Vec3,
        size: Vec2,
        depth: f32,
        rotation: f32,
    ) -> Decal {
        let z = normal.normalize_or(Vec3::Y);
        let helper = if z.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let x = helper.cross(z).normalize();
        let y = z.cross(x);
        let basis = Mat4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            position.extend(1.0),
        );
        Decal {
            transform: basis
                * Mat4::from_rotation_z(rotation)
                * Mat4::from_scale(Vec3::new(size.x, size.y, depth)),
            ..Default::default()
        }
    }
}
