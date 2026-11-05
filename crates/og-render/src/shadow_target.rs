//! What a shadow pass draws into, and the state of the caster pipelines
//! that draw it (`docs/design.md` §2.6).
//!
//! A cascade (and the weather's occlusion map) keeps its casters' far
//! sides: front-sided casters cull their front faces and a depth test
//! keeps the nearest of the rest. A local light's layer keeps both sides,
//! in two channels (shadow_pass.wgsl `local_value`): red the nearest
//! surface facing away from the light (the cascades' far side), green the
//! nearest facing it, and the lit shaders compare against the midpoint of
//! the two (Woo, "The Shadow Depth Map Revisited", Graphics Gems III,
//! 1992; lighting.wgsl `local_depth`). The midpoint lies inside a closed
//! caster and at a zero-thickness one's plane, so no receiver compares
//! against its own surface: a box's base reads as shadow, and a floor of
//! two coplanar quads facing apart as lit.
//!
//! Both sides are drawn in one render pass: every caster's far sides
//! first, depth-tested and written as a cascade's, then its near sides
//! over them, tested less-or-equal and written too. A near side is in
//! front of its own caster's far side, so the far sides' depths reject
//! only near sides hidden behind another caster's far side, and the
//! less-or-equal test keeps a zero-thickness caster's two quads at one
//! depth. No blending is needed, so every backend draws it the same way.
//! On the M4 Pro (8 cube point lights, 48 faces, moving every frame at
//! 1080p) the shadow passes add 3.2 ms of GPU time and 2.0 ms of encoding
//! to a frame, against 4.5 and 3.3 when the near sides were a second,
//! mirrored raster resolved by a compute pass, and 1.3 and 1.9 for far
//! sides alone.

use crate::{DEPTH_FORMAT, Side};

/// The local shadow array's format: far sides in red, near sides in green.
pub(crate) const LOCAL_SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;

/// What one shadow pass's caster pipelines draw into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShadowTarget {
    /// A cascade or the weather's occlusion map (R32Float): far sides.
    Cascade,
    /// A local layer's far sides (red), drawn first.
    LocalFar,
    /// Then its near sides (green), in the same pass.
    LocalNear,
}

impl ShadowTarget {
    /// The draws of a local layer's pass, in order.
    pub(crate) const LOCAL: [ShadowTarget; 2] = [ShadowTarget::LocalFar, ShadowTarget::LocalNear];

    /// The faces a caster of `side` culls.
    pub(crate) fn cull(self, side: Side) -> Option<wgpu::Face> {
        let (front, back) = (Some(wgpu::Face::Front), Some(wgpu::Face::Back));
        match (self, side) {
            (_, Side::Double) => None,
            (ShadowTarget::Cascade | ShadowTarget::LocalFar, Side::Front)
            | (ShadowTarget::LocalNear, Side::Back) => front,
            (ShadowTarget::Cascade | ShadowTarget::LocalFar, Side::Back)
            | (ShadowTarget::LocalNear, Side::Front) => back,
        }
    }

    /// The depth test: the nearest, or for the near sides the nearest
    /// not behind a far side drawn before them (equal kept).
    pub(crate) fn depth(self) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(if self == ShadowTarget::LocalNear {
                wgpu::CompareFunction::LessEqual
            } else {
                wgpu::CompareFunction::Less
            }),
            stencil: Default::default(),
            bias: Default::default(),
        }
    }

    /// The colour target.
    pub(crate) fn color(self) -> wgpu::ColorTargetState {
        let (format, write_mask) = match self {
            ShadowTarget::Cascade => (wgpu::TextureFormat::R32Float, wgpu::ColorWrites::RED),
            ShadowTarget::LocalFar => (LOCAL_SHADOW_FORMAT, wgpu::ColorWrites::RED),
            ShadowTarget::LocalNear => (LOCAL_SHADOW_FORMAT, wgpu::ColorWrites::GREEN),
        };
        wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask,
        }
    }

    /// The fragment stage's constants: the caster's sides
    /// (shadow_pass.wgsl `SHADOW_SIDE`), which decide its faces' channels.
    pub(crate) fn constants(side: Side) -> &'static [(&'static str, f64)] {
        match side {
            Side::Front => &[("SHADOW_SIDE", 0.0)],
            Side::Back => &[("SHADOW_SIDE", 1.0)],
            Side::Double => &[("SHADOW_SIDE", 2.0)],
        }
    }
}

/// A caster's pipelines, one per [`ShadowTarget`].
pub(crate) struct ShadowSet([wgpu::RenderPipeline; 3]);

impl ShadowSet {
    pub(crate) fn new(mut make: impl FnMut(ShadowTarget) -> wgpu::RenderPipeline) -> ShadowSet {
        ShadowSet([
            make(ShadowTarget::Cascade),
            make(ShadowTarget::LocalFar),
            make(ShadowTarget::LocalNear),
        ])
    }

    pub(crate) fn get(&self, target: ShadowTarget) -> &wgpu::RenderPipeline {
        &self.0[target as usize]
    }
}
