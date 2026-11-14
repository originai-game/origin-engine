//! Frame rendering.

use super::*;

mod overlay_setup;

mod prepare_pipelines;
use prepare_pipelines::PrepareFramePipelinesInputs;

mod crowd_cards;
mod draw_skinned;
use draw_skinned::DrawSkinnedInputs;

mod draw_list;
use draw_list::DrawListInputs;

mod draw_sprites;

mod draw_strips;
use draw_strips::StripPart;

mod draw_glow;

mod stylise_uniforms;

mod post_uniforms;
use post_uniforms::{FramePostHistory, WriteFramePostUniformsInputs};

mod pass_probes;
use pass_probes::{ProbesBindIn, ProbesIn, ProbesPass};

mod pass_view;
use pass_view::{ViewBindIn, ViewIn, ViewPass};

mod pass_prepass;
use pass_prepass::{PrepassPass, PrepassPlan};

mod pass_reflection;
use pass_reflection::{ReflectionBindIn, ReflectionIn, ReflectionPass};

mod pass_oit;
use pass_oit::BlendedPart;
mod pass_main;
use pass_main::{MainPass, MainPlan};

mod view_culling;

mod camera;

mod draw_ctx;
mod schedule;
mod shadow_encode;
use schedule::{PassDesc, Timed};
mod pass;
use pass::{CullFrame, EncodeCx, FrameCulling, FramePlans, ListsMoved, Pass, PassFrames};
mod pass_gi_probes;
use draw_ctx::DrawCtx;
use pass_gi_probes::GiProbesPass;
mod pass_live_sky;
use pass_live_sky::LiveSkyPass;
mod pass_clouds;
use pass_clouds::CloudsPass;
mod pass_volumetric;
use pass_volumetric::VolumetricPass;
mod pass_ssr;
use pass_ssr::SsrPass;
mod pass_light_2d;
use pass_light_2d::{Light2dFrame, Light2dPass};
mod pass_metaballs;
use pass_metaballs::MetaballsPass;
mod pass_fluid_3d;
use pass_fluid_3d::Fluid3dPass;
mod pass_particles;
pub(crate) use pass_particles::Particles;
use pass_particles::{ParticlesPass, ParticlesPlan};
mod pass_shadows;
mod pre_graph;
use pass_shadows::{CascadeIn, CascadePass, Casters, LocalFacePass, ShelterPass};
mod pass_glow;
use pass_glow::GlowPass;
mod pass_custom_post;
use pass_custom_post::{CustomPostPass, CustomPostPlan};
mod frame_visibility;
use frame_visibility::FrameVisibility;
mod instance_uploads;
use instance_uploads::InstanceUploadInputs;
mod gpu_cull_stage;
use gpu_cull_stage::GpuCullInputs;
mod sprite_arenas;
use sprite_arenas::SpriteArenaInputs;
mod world_prepare;
use world_prepare::PrepareWorldInputs;
mod lights_stage;
use lights_stage::PrepareLightsInputs;
mod instances_stage;
use instances_stage::UploadInstancesInputs;
mod uploads_stage;
use uploads_stage::UploadFrameInputs;
mod frame_uniform;
use frame_uniform::WriteFrameUniformsInputs;
mod sun_and_post;
use sun_and_post::WriteSunAndPostInputs;
mod fx_uniform;
use fx_uniform::WriteFxUniformsInputs;
mod view_uniforms;
use view_uniforms::{WriteViewUniformsInputs, WriteViewUniformsOutputs};
mod effects;
use effects::{DecideEffectsInputs, FrameEffects};
mod water_fluids;
use water_fluids::PrepareWaterAndFluidsInputs;
mod oit_frame;
use oit_frame::PlanOitFrameInputs;
mod sprite_groups;
use sprite_groups::PrepareSpriteGroupsInputs;
mod shadow_inputs;
use shadow_inputs::PrepareShadowInputsInputs;
mod culling_plan;
use culling_plan::PlanCullingInputs;
mod camera_lists;
use camera_lists::PlanCameraListsInputs;
mod pass_list;
use pass_list::PassListInputs;
mod stylise_frame;
use stylise_frame::{PrepareFrameStyliseInputs, PrepareFrameStyliseOutputs};
mod fsr_frame;
mod post_resources;
use fsr_frame::PrepareFsrInputs;
mod ui_upload;
use ui_upload::UploadUiInputs;
mod graph_prepare;
use graph_prepare::PrepareGraphInputs;
mod frame_stats;
use frame_stats::{FrameStatsInputs, FrameStatsOutputs};
mod fx_groups;
mod pre_graph_stage;
use pre_graph_stage::PreGraphInputs;
mod frame_binds;
use frame_binds::BindFrameInputs;
mod after_graph;
use after_graph::{
    FinishedFrame, HdrOutputPass, OverlayPass, OverlayPlan, UiLayersPass, ViewportViewsPass,
    ViewportViewsPlan,
};
mod submit_frame;
use submit_frame::SubmitFrameInputs;
mod skinned_pipelines;
use skinned_pipelines::SkinnedPipelines;
mod execute;
use execute::GraphRun;
mod draw_frame;
mod frame_stages;
use frame_stages::Stages;
mod setup;

/// A shadow pass into one layer, cleared (depth 1, and distance / range 1:
/// nothing nearer), its depth kept for a copy after it or not.
fn shadow_target<'e>(
    encoder: &'e mut crate::Encoder,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    keep_depth: bool,
) -> wgpu::RenderPass<'e> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("shadow"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: if keep_depth {
                    wgpu::StoreOp::Store
                } else {
                    wgpu::StoreOp::Discard
                },
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// A local shadow face's projection words (`ShadowPassUniform::paraboloid`):
/// for a paraboloid hemisphere, 0 below the light and 1 above (frame.rs).
fn local_paraboloid(paraboloid: bool, face: usize, side: u32) -> [f32; 4] {
    if paraboloid {
        [
            1.0,
            if face == 0 { 1.0 } else { -1.0 },
            side as f32,
            shadows::PARABOLOID_SCALE,
        ]
    } else {
        [0.0; 4]
    }
}

/// Contact shadows (contact.wgsl, `FxPost::contact` row 3):
/// Bend Studio's recommended defaults (third_party/bend-sss): every pixel a
/// slab 0.5 % of its depth to the far plane thick, an edge where two
/// neighbours' depths differ by 2 %, a contrast of 4, 60 samples (the
/// shadow's reach in pixels).
const CONTACT_PARAMS: [f32; 4] = [0.005, 0.02, 4.0, 60.0];

/// A paint canvas's mean height and spread (standard deviation), its red
/// channel in linear light, so the watercolour wash reads any canvas's
/// relief in units of its own grain (stylise.wgsl `paper_relief`).
fn canvas_stats(t: &TextureData) -> [f32; 2] {
    let decode = |v: u8| {
        let c = f64::from(v) / 255.0;
        match t.color_space {
            ColorSpace::Srgb if c > 0.04045 => ((c + 0.055) / 1.055).powf(2.4),
            ColorSpace::Srgb => c / 12.92,
            ColorSpace::Linear => c,
        }
    };
    let n = (t.rgba.len() / 4).max(1) as f64;
    let (sum, sum2) = t
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .fold((0.0, 0.0), |(s, s2), p| {
            let v = decode(p[0]);
            (s + v, s2 + v * v)
        });
    let mean = sum / n;
    [mean as f32, (sum2 / n - mean * mean).max(0.0).sqrt() as f32]
}

/// Whether an instance was drawn elsewhere in the previous frame
/// ([`RenderScene::previous_transforms`]): TAA then needs the prepass's
/// motion vectors.
fn objects_moved(scene: &RenderScene) -> bool {
    scene.objects.previous_transforms.len() == scene.objects.instances.len()
        && scene
            .objects
            .previous_transforms
            .iter()
            .zip(&scene.objects.instances)
            .any(|(p, i)| *p != i.transform)
}

impl Renderer {
    /// Whether `sorted` instances are culled on the GPU as far as the
    /// renderer's culling choice goes (the frame may still fall back: its
    /// motion blur, an empty frame, buffers over the device's limits).
    fn gpu_culls(&self, sorted: u32) -> bool {
        self.visibility.indirect
            && match self.visibility.desc_culling {
                Culling::Auto => sorted >= GPU_CULLING_MIN_INSTANCES,
                Culling::Gpu => true,
                Culling::Cpu => false,
            }
    }
}
