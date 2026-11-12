//! A frame pass as one unit: what it needs of the device, what it keeps
//! between frames, its steps in a frame (plan, upload, bind) and how it
//! encodes.
//!
//! A pass declares its kind and needs in [`Pass::DESC`], which is its
//! entry in the capability table (`schedule.rs` `PASSES`). `render` calls
//! each pass's steps among the frame's shared stages, handing each its
//! typed inputs: [`Pass::plan`] decides the frame's work for the pass
//! (its lists against the frame's culling, [`FrameCulling`]) into its
//! [`Pass::Frame`], [`Pass::upload`] writes what that needs on the
//! device, and [`Pass::bind`] makes its bind groups and the plan its
//! encode reads. [`passes!`] lists the passes in frame order and makes
//! [`PassFrames`] and [`FramePlans`], one of each per pass, and the
//! dispatch from a graph pass to its pass, without boxing.

use super::*;

/// The frame's culling as the passes that cull after the main camera
/// extend it (camera_lists.rs makes it): the passes registered with the
/// GPU culling and the meshlet dispatch, the CPU cull's sorted copies and
/// their origins, and its scratch.
pub(super) struct FrameCulling {
    pub vis: FrameVisibility,
    pub transforms: Vec<instances::GpuTransform>,
    pub origins: Vec<u32>,
    pub cull_inside: CullScratch,
}

/// What the passes that cull read of the frame: its batches, the sorted
/// instances' spheres and submission order, whether the GPU culls and
/// its draws per pass, and the meshlet path's batches.
#[derive(Clone, Copy)]
pub(super) struct CullFrame<'a> {
    pub batches: &'a [Batch],
    pub spheres: &'a [(Vec3, f32)],
    pub submission: &'a [u32],
    pub gpu_cull: bool,
    pub cull_n: u32,
    /// Whether a batch is drawn by a meshlet view.
    pub in_meshlet: &'a dyn Fn(&Batch) -> bool,
    /// The meshlet batches on any of a mask's layers.
    pub on_layers: &'a dyn Fn(u32) -> Vec<u32>,
}

/// Where the CPU-culled lists' instances are in the uploaded records:
/// with the GPU culling, which takes the sorted instances to its own
/// buffer, `sorted` places earlier (instances_stage.rs).
#[derive(Clone, Copy)]
pub(super) struct ListsMoved {
    pub gpu_cull: bool,
    pub sorted: u32,
}

impl ListsMoved {
    /// Moves `lists`' CPU-culled ranges where the records put them.
    pub(super) fn apply<'l>(self, lists: impl IntoIterator<Item = &'l mut DrawList>) {
        if !self.gpu_cull {
            return;
        }
        for list in lists {
            for r in &mut list.ranges {
                r.1 -= self.sorted;
            }
        }
    }
}

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
    /// What it keeps of one frame until the frame is encoded: its draw
    /// lists, views, cache plans and bind groups (`()` for a pass that
    /// keeps nothing of its own).
    type Frame: Default;
    /// What its plan step reads.
    type PlanIn<'a>;
    /// What its upload step reads.
    type UploadIn<'a>;
    /// What its bind step reads: the frame's shared bindings and what the
    /// stages before it made.
    type BindIn<'f>;
    /// What a frame planned for it; it runs in the frames that plan it.
    type Plan<'f>;
    /// Its state in the renderer.
    fn state(r: &Renderer) -> &Self::State;
    /// Decides the frame's work for the pass (its lists, against the
    /// frame's culling where it culls): by default nothing.
    fn plan(_r: &mut Renderer, _input: Self::PlanIn<'_>) -> Self::Frame {
        Self::Frame::default()
    }
    /// Writes what its frame needs on the device, or moves its lists where
    /// the frame's uploads put their instances: by default nothing.
    fn upload(_r: &mut Renderer, _frame: &mut Self::Frame, _input: Self::UploadIn<'_>) {}
    /// Makes its bind groups into its frame and answers what its encode
    /// reads: `None` in a frame it does not run.
    fn bind<'f>(
        r: &'f Renderer,
        frame: &'f mut Self::Frame,
        input: Self::BindIn<'f>,
    ) -> Option<Self::Plan<'f>>;
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
        /// What this frame keeps for each pass of [`passes!`]
        /// ([`Pass::Frame`]), from its plan step to its encode.
        #[derive(Default)]
        pub(super) struct PassFrames {
            $(pub $field: <$ty as Pass>::Frame,)*
        }

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
    reflection: pass_reflection::ReflectionPass => FramePass::Reflection(_),
    view: pass_view::ViewPass => FramePass::View(_),
    prepass: pass_prepass::PrepassPass => FramePass::Prepass,
    clouds: pass_clouds::CloudsPass => FramePass::Clouds,
    main: pass_main::MainPass => FramePass::Main,
    glow: pass_glow::GlowPass => FramePass::Glow,
    custom_post: pass_custom_post::CustomPostPass => FramePass::CustomPost(_),
    viewport_views: after_graph::ViewportViewsPass => FramePass::ViewportViews,
    ui_layers: after_graph::UiLayersPass => FramePass::UiLayers,
    overlay: after_graph::OverlayPass => FramePass::Overlay,
    hdr_output: after_graph::HdrOutputPass => FramePass::HdrOutput,
}
