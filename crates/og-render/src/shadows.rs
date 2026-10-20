//! Local light shadows by tier (`docs/design.md` §2.6): how many point and
//! spot lights cast shadows in one frame, and how a point light's shadow is
//! stored, as six cube faces or two paraboloid hemispheres.
//!
//! Every shadowed light gets consecutive layers of one R32Float array (512²
//! a layer, [`crate::LOCAL_SHADOW_SIZE`]): a spot light one perspective
//! view, a point light six cube faces or, dual-paraboloid, two hemispheres
//! (the one below the light and the one above), each a paraboloid
//! projection drawn in one pass. The array starts at twelve layers and
//! grows once to the tier's capacity when a frame needs more.
//!
//! When more lights in view ask for shadows than the tier allows, the ones
//! with the most radiance at the camera keep them (`clusters.rs`'s score,
//! ties in scene order); their layers are handed out in scene order.

use crate::Tier;

/// How a point light's shadow map is stored ([`crate::RenderDesc::point_shadows`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointShadows {
    /// Cube faces on the full tier, paraboloids on the no-compute tier
    /// (WebGL2), where every extra pass costs a GL draw setup and the six
    /// faces of four lights would be 24 MB.
    #[default]
    Auto,
    /// Six 90° perspective faces: six passes and six layers per light.
    Cube,
    /// Two paraboloid hemispheres: two passes and two layers per light,
    /// coarser (about twice the texel angle at a hemisphere's centre) and
    /// with straight-edged triangles bent by the projection, so large
    /// casters' silhouettes are approximate.
    Paraboloid,
}

impl PointShadows {
    /// What `Auto` means on `tier` (never `Auto`).
    pub fn resolve(self, tier: Tier) -> PointShadows {
        match self {
            PointShadows::Auto if tier == Tier::Full => PointShadows::Cube,
            PointShadows::Auto => PointShadows::Paraboloid,
            other => other,
        }
    }

    /// Layers (and passes) one point light takes.
    pub fn layers(self) -> usize {
        match self {
            PointShadows::Paraboloid => 2,
            _ => 6,
        }
    }
}

/// The most point and spot lights that cast shadows in one frame on the
/// full tier (WebGPU, Metal, Vulkan): [`crate::RenderInfo::shadowed_lights`].
pub const MAX_SHADOWED_LIGHTS: usize = 8;
/// The same on the no-compute tier (WebGL2).
pub const NO_COMPUTE_SHADOWED_LIGHTS: usize = 4;
/// Layers of the local shadow array the frame uniform addresses: every
/// shadowed light a point light with cube faces.
pub(crate) const LOCAL_SHADOW_LAYERS: usize = MAX_SHADOWED_LIGHTS * 6;
/// The layers the local shadow array starts with (two lights' cube faces).
pub(crate) const INITIAL_LOCAL_LAYERS: usize = 12;
/// How far into the unit disc a hemisphere's edge (90° from its axis)
/// lands: the 10 % left over holds up to 96°, so filtering at the seam
/// reads drawn texels (`PARABOLOID_SCALE` in lighting.wgsl).
pub(crate) const PARABOLOID_SCALE: f32 = 0.9;

/// The shadowed lights a tier allows.
pub(crate) fn shadowed_lights(tier: Tier) -> usize {
    match tier {
        Tier::Full => MAX_SHADOWED_LIGHTS,
        Tier::NoCompute => NO_COMPUTE_SHADOWED_LIGHTS,
    }
}

/// The local shadow array's capacity in layers: every allowed light a point light.
pub(crate) fn capacity(lights: usize, mode: PointShadows) -> usize {
    lights * mode.layers()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_is_cube_on_the_full_tier_and_paraboloid_without_compute() {
        assert_eq!(PointShadows::Auto.resolve(Tier::Full), PointShadows::Cube);
        assert_eq!(
            PointShadows::Auto.resolve(Tier::NoCompute),
            PointShadows::Paraboloid
        );
        assert_eq!(
            PointShadows::Paraboloid.resolve(Tier::Full),
            PointShadows::Paraboloid
        );
        assert_eq!(
            PointShadows::Cube.resolve(Tier::NoCompute),
            PointShadows::Cube
        );
        assert_eq!(capacity(8, PointShadows::Cube), 48);
        assert_eq!(capacity(4, PointShadows::Paraboloid), 8);
        assert!(capacity(MAX_SHADOWED_LIGHTS, PointShadows::Cube) <= LOCAL_SHADOW_LAYERS);
    }
}
