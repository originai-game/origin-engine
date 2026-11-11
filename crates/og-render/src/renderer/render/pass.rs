//! A frame pass as one unit: what it needs of the device, what it keeps
//! between frames, what a frame plans for it and how it encodes.
//!
//! A pass declares its kind and needs in [`Pass::DESC`], which is its
//! entry in the capability table (`schedule.rs` `PASSES`). [`passes!`]
//! lists the passes in frame order and makes [`FramePlans`], one plan per
//! pass, and the dispatch from a graph pass to its pass, without boxing.

use super::*;

/// What a pass encodes with: the frame's drawing state, its encoder and
/// statistics, and the frame's shared bindings and targets.
pub(super) struct EncodeCx<'a, 'f> {
    pub ctx: &'a DrawCtx<'f>,
    pub encoder: &'a mut Encoder,
    pub stats: &'a mut FrameStats,
    /// The frame's group 0 (the camera's uniform, shadows, IBL, lights).
    pub frame_bind_group: &'f wgpu::BindGroup,
    /// The depth the scene's passes read: the prepass's, or the main
    /// pass's own.
    pub scene_depth: &'f wgpu::TextureView,
    /// The sky pipelines' group.
    pub sky_group: &'f wgpu::BindGroup,
    /// This frame's slot of the two-frame histories.
    pub slot: usize,
}

/// A kind of frame pass.
pub(super) trait Pass {
    /// Its kind, its name and the capabilities it needs.
    const DESC: PassDesc;
    /// What it keeps between frames.
    type State;
    /// What a frame planned for it; it runs in the frames that plan it.
    type Plan<'f>;
    /// Its state in the renderer.
    fn state(r: &Renderer) -> &Self::State;
    /// What this frame's plan of it needs beyond [`Pass::DESC`]'s.
    fn needs(_plan: &Self::Plan<'_>) -> Caps {
        Caps::NONE
    }
    /// Records the pass (instance `pass`, for an instanced kind).
    fn encode<'f>(
        st: &Self::State,
        plan: &Self::Plan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    );
}

/// The frame's passes, in frame order: `plan field: pass type => the
/// graph passes it encodes`.
macro_rules! passes {
    ($($field:ident: $ty:ty => $pat:pat),* $(,)?) => {
        /// What this frame planned for each pass of [`passes!`]; `None`
        /// for one that does not run.
        #[derive(Default)]
        pub(super) struct FramePlans<'f> {
            $(pub $field: Option<<$ty as Pass>::Plan<'f>>,)*
        }

        impl<'f> FramePlans<'f> {
            /// What this frame's plan of `pass` needs beyond its kind's.
            pub(super) fn needs(&self, pass: FramePass) -> Caps {
                match pass {
                    $($pat => self.$field.as_ref().map_or(Caps::NONE, <$ty as Pass>::needs),)*
                    _ => Caps::NONE,
                }
            }

            /// Encodes `pass` when it is one of [`passes!`]; false for a
            /// pass `render` still encodes itself.
            pub(super) fn encode(&self, pass: FramePass, cx: &mut EncodeCx<'_, 'f>) -> bool {
                let r = cx.ctx.r;
                match pass {
                    $($pat => <$ty as Pass>::encode(
                        <$ty as Pass>::state(r),
                        self.$field.as_ref().expect("a pass in the graph is planned"),
                        pass,
                        cx,
                    ),)*
                    _ => return false,
                }
                true
            }
        }
    };
}

passes! {
    live_sky: pass_live_sky::LiveSkyPass => FramePass::LiveSky,
    gi_probes: pass_gi_probes::GiProbesPass => FramePass::GiProbes,
    cascade: pass_shadows::CascadePass => FramePass::Cascade(_),
    shelter: pass_shadows::ShelterPass => FramePass::Shelter,
    local_face: pass_shadows::LocalFacePass => FramePass::LocalFace(_),
    particles: pass_particles::ParticlesPass => FramePass::Particles,
    light_2d: pass_light_2d::Light2dPass => FramePass::Light2d,
    metaballs: pass_metaballs::MetaballsPass => FramePass::Metaballs,
    fluid_3d: pass_fluid_3d::Fluid3dPass => FramePass::Fluid3d,
    volumetric: pass_volumetric::VolumetricPass => FramePass::Volumetric,
    ssr: pass_ssr::SsrPass => FramePass::Ssr,
    probes: pass_probes::ProbesPass => FramePass::Probes,
}
