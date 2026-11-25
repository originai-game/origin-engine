//! What a shadow pass draws into, and the state of the caster pipelines
//! that draw it (`docs/design.md` §2.6).
//!
//! Both directional cascades (plus the weather shelter and ground height
//! maps) and local lights' shadow faces draw casters' back faces in a single
//! pass into an `R32Float` layer: front-sided casters cull their front faces
//! and a depth test keeps the nearest back face. Because every shadow pass
//! uses the same format, color write mask, depth compare, and face culling,
//! every caster compiles one shadow pipeline shared across cascades and
//! local lights (with `shadow_pass.light.w > 0.0` switching between clip
//! depth and radial distance in `shadow_pass.wgsl`).

use crate::{DEPTH_FORMAT, Side};

/// The local shadow array's format: single-pass radial distance / range in red.
pub(crate) const LOCAL_SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

/// What one shadow pass's cache and depth restoration target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShadowTarget {
    /// A cascade, the weather shelter map, or the ground height map (`R32Float`, clip depth).
    Cascade,
    /// A local light's shadow layer (`R32Float`, radial distance / range).
    Local,
}

impl ShadowTarget {
    /// The faces a caster of `side` culls: front-sided casters cull front faces
    /// so back faces are projected into the shadow map; double-sided casters
    /// cull neither side.
    pub(crate) fn cull(side: Side) -> Option<wgpu::Face> {
        match side {
            Side::Front => Some(wgpu::Face::Front),
            Side::Back => Some(wgpu::Face::Back),
            Side::Double => None,
        }
    }

    /// The depth test: nearest surface (`Less`).
    pub(crate) fn depth() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }
    }

    /// The colour target (`R32Float`, red channel).
    pub(crate) fn color() -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format: wgpu::TextureFormat::R32Float,
            blend: None,
            write_mask: wgpu::ColorWrites::RED,
        }
    }
}
