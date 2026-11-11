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
use pass_probes::{ProbesPass, ProbesPlan};

mod pass_view;
use pass_view::{ViewPass, ViewPlan};

mod pass_prepass;
use pass_prepass::{PrepassPass, PrepassPlan};

mod pass_reflection;
use pass_reflection::{ReflectionPass, ReflectionPlan};

mod pass_oit;
use pass_oit::BlendedPart;
mod pass_main;
use pass_main::{MainPass, MainPlan};

mod view_culling;

mod camera;

mod draw_ctx;
mod schedule;
use schedule::PassDesc;
mod pass;
use pass::{EncodeCx, FramePlans, Pass};
mod pass_gi_probes;
use draw_ctx::DrawCtx;
use pass_gi_probes::GiProbesPass;
mod pass_live_sky;
use pass_live_sky::LiveSkyPass;
mod pass_volumetric;
use pass_volumetric::VolumetricPass;
mod pass_ssr;
use pass_ssr::SsrPass;
mod pass_light_2d;
use pass_light_2d::Light2dPass;
mod pass_metaballs;
use pass_metaballs::MetaballsPass;
mod pass_fluid_3d;
use pass_fluid_3d::Fluid3dPass;
mod pass_particles;
pub(crate) use pass_particles::Particles;
use pass_particles::{ParticlesPass, ParticlesPlan};
mod pre_graph;
use pre_graph::pre_graph;
mod pass_shadows;
use pass_shadows::{
    CascadePass, CascadePlan, LocalFacePass, LocalFacePlan, ShelterPass, ShelterPlan,
};
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
use sprite_arenas::{SpriteArenaInputs, SpriteArenas};
mod shadow_plan;
use shadow_plan::{PlanShadowsInputs, PlanShadowsOutputs};
mod reflection_lists;
use reflection_lists::{PlanReflectionListsInputs, PlanReflectionListsOutputs};
mod probe_lists;
use probe_lists::{PlanProbeListsInputs, PlanProbeListsOutputs};
mod view_lists;
use view_lists::{PlanViewListsInputs, PlanViewListsOutputs};
mod world_prepare;
use world_prepare::{PrepareWorldInputs, PrepareWorldOutputs};
mod lights_stage;
use lights_stage::{PrepareLightsInputs, PrepareLightsOutputs};
mod instances_stage;
use instances_stage::{UploadInstancesInputs, UploadInstancesOutputs};
mod uploads_stage;
use uploads_stage::{UploadFrameInputs, UploadFrameOutputs};
mod frame_uniform;
use frame_uniform::{WriteFrameUniformsInputs, WriteFrameUniformsOutputs};
mod sun_and_post;
use sun_and_post::{WriteSunAndPostInputs, WriteSunAndPostOutputs};
mod fx_uniform;
use fx_uniform::{WriteFxUniformsInputs, WriteFxUniformsOutputs};
mod view_uniforms;
use view_uniforms::{WriteViewUniformsInputs, WriteViewUniformsOutputs};
mod effects;
use effects::{DecideEffectsInputs, FrameEffects};
mod water_fluids;
use water_fluids::{PrepareWaterAndFluidsInputs, PrepareWaterAndFluidsOutputs};
mod oit_frame;
use oit_frame::{PlanOitFrameInputs, PlanOitFrameOutputs};
mod sprite_groups;
use sprite_groups::PrepareSpriteGroupsInputs;
mod shadow_inputs;
use shadow_inputs::{PrepareShadowInputsInputs, PrepareShadowInputsOutputs};
mod culling_plan;
use culling_plan::{PlanCullingInputs, PlanCullingOutputs};
mod camera_lists;
use camera_lists::{PlanCameraListsInputs, PlanCameraListsOutputs};
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
use pre_graph_stage::{EncodePreGraphInputs, EncodePreGraphOutputs};
mod setup;

impl Renderer {
    /// Draws one frame of `scene` into `target`.
    pub fn render(
        &mut self,
        scene: &RenderScene,
        target: &RenderTarget,
    ) -> Result<FrameStats, RenderError> {
        if let Some(reason) = self.lost() {
            return Err(RenderError::DeviceLost(reason));
        }
        if let Some(stale) = scene.stale_handle() {
            return Err(RenderError::InvalidHandle(stale));
        }
        // What this frame waits for compiles (FrameStats::compile_wait_us).
        crate::deferred::take_waited_us();
        // A capture draws with the full variants (pipeline_features.rs):
        // what a display frame draws with depends on what has compiled.
        self.frame_features = if self.capturing {
            crate::pipeline_features::Features::ALL
        } else {
            crate::pipeline_features::Features::of(scene)
        };
        self.terrain.features = self.frame_features;
        // Match the surface's two-frame latency. Wait for the oldest frame,
        // not the entire queue: motion and readbacks share this device.
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(target, RenderTarget::Offscreen) && self.offscreen_submissions.len() == 2 {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: self.offscreen_submissions.front().cloned(),
                    timeout: Some(crate::capture::BACKLOG_TIMEOUT),
                })
                .map_err(|e| {
                    RenderError::Readback(format!("waiting for an offscreen frame: {e}"))
                })?;
            self.offscreen_submissions.pop_front();
        }
        self.sync(scene)?;
        // What the frame plans for each of its passes (pass.rs).
        let mut plans = FramePlans::default();
        // The wind field's steps since the last tick drawn (wind_field.rs):
        // its grid is what every vegetation pass below reads it by, its
        // lean cap what their culling bounds widen by.
        self.wind_field.prepare(&self.device, &self.queue, scene);
        self.trample.prepare(&self.device, &self.queue, scene);
        self.refresh_foliage_bounds(scene);
        let (upscale, grid_offset) = self.prepare_frame_resources(scene);
        let (probe_frame, active_probes, probe_boxes) = self.prepare_frame_probes(scene);
        let (mut batches, sorted_instances) = self.build_batches(scene)?;
        let instances::Sorted {
            mut transforms,
            mut spheres,
            submission,
            mut sphere_box,
            serial: sorted_serial,
            changed: sorted_changed,
            ..
        } = sorted_instances;
        // The debris layer's instances are culled by everywhere their
        // pieces can reach: the GPU moves them (debris.rs).
        let debris_instances = crate::debris::instances(scene);
        if !debris_instances.is_empty() {
            for (s, &i) in submission.iter().enumerate().take(spheres.len()) {
                if let Some(&(p, _)) = debris_instances.get(&i) {
                    spheres[s] = scene.debris[p].sphere();
                }
            }
            sphere_box = instances::fold_box(None, spheres.iter().copied());
        }
        let mut origins = submission.clone();
        self.prepare_full_meshes(scene, &batches);
        let mut skinned = self.build_skinned(scene, &probe_boxes)?;
        // The planar water this frame draws: none where the 3D world is
        // not built in (WORLD_3D).
        let scene_water: &[Water] = if WORLD_3D { &scene.water } else { &[] };

        let (aspect, temporal, camera, cam, view_frustum) = self.prepare_frame_camera(scene);
        let taa = self.info.antialias == Antialias::Taa;
        let smaa_t2x = self.info.antialias == Antialias::SmaaT2x;
        let (volumetric_fog, frame_fog, reflections) = self.prepare_frame_reflections(
            scene,
            scene_water,
            &camera,
            &cam,
            &view_frustum,
            &aspect,
        );
        let reflection = reflections.first();
        // Crowds (crowd.rs): their members as the crowd cull's tables, and
        // where each member's previous transform goes under TAA (after the
        // palette's previous section, skin.rs).
        let crowd_motion = temporal || scene.post.motion_blur.is_some();
        let crowd_frame = self.build_crowds(scene, crowd_motion)?;
        let crowd_batches = self.crowd_batches(scene, &crowd_frame);
        let crowd_previous = crowd_motion.then(|| {
            (skinned.matrices.len()
                + skin::previous_section_len(
                    &scene.joint_palettes,
                    &scene.previous_joint_palettes,
                    skinned.matrices.len(),
                    skinned.previous_slots.len(),
                )) as u32
        });
        if crowd_previous.is_some_and(|at| at as usize + crowd_frame.previous.len() > MAX_JOINTS) {
            return Err(RenderError::InvalidData(format!(
                "{} crowd members' previous transforms past the palette's other slots (at most {MAX_JOINTS} in all)",
                crowd_frame.previous.len()
            )));
        }
        // On the GPU-driven path (crowd_gpu.rs) where the device draws
        // indirect with a first instance and every view that draws them is
        // a GPU culling pass (secondary views and probe faces cull skinned
        // instances on the CPU); else as skinned instances.
        let crowd_gpu = crowd_frame.len() > 0
            && self.crowd_indirect
            && self.visibility.desc_culling != Culling::Cpu
            && scene.views.is_empty()
            && probe_frame.due.is_empty()
            && crowd_batches.iter().all(|b| !b.transparent)
            && {
                let limits = self.device.limits();
                let max = limits
                    .max_storage_buffer_binding_size
                    .min(limits.max_buffer_size);
                let rows = 4
                    + MAX_CASCADES as usize
                    + scene.lights.iter().filter(|l| l.cast_shadows).count() * 6;
                // GPU culling must take the frame too (its pyramid, its
                // passes): checked for every instance the frame may sort.
                let culls = self
                    .visibility
                    .gpu_cull
                    .get_or_insert_with(|| {
                        cull_gpu::GpuCulling::new(&self.device, max, self.info.indirect_per_draw)
                    })
                    .fits(
                        scene.instances.len() as u32,
                        scene.skinned_instances.len() as u32,
                        rows,
                    );
                culls
                    && self
                        .crowd_gpu
                        .get_or_insert_with(|| {
                            crowd_gpu::CrowdCulling::new(
                                &self.device,
                                max,
                                self.info.indirect_per_draw,
                            )
                        })
                        .fits(crowd_frame.len(), rows)
            };
        // The CPU path's card records (`crowd_impostor.rs` draws them whole).
        let cpu_cards = if !crowd_gpu && crowd_frame.len() > 0 {
            self.append_crowds(
                scene,
                &crowd_frame,
                &crowd_batches,
                &mut skinned,
                crowd_previous,
            )
        } else {
            Default::default()
        };
        skinned.baked_moved |= crowd_frame.moved;
        let (view_plans, mut view_skinned) =
            self.prepare_view_culling(scene, &camera, &mut skinned);
        let (probe_faces, mut probe_skinned) =
            self.prepare_probe_culling(&probe_frame, &active_probes, &camera, &mut skinned);
        let (mut skinned_visible, mut skinned_culled) =
            Self::cull_skinned(&mut skinned, &view_frustum, scene.camera_layers);
        let mut skinned_batches = std::mem::take(&mut skinned.batches);
        // The skinned instances' batches; the crowds' GPU draws after them.
        let skinned_plain = skinned_batches.len();
        if crowd_gpu {
            skinned_batches.extend(crowd_batches.iter().map(|b| Batch { ..*b }));
            skinned_visible.extend(crowd_batches.iter().map(|b| Batch { ..*b }));
        }
        let skinned_data = std::mem::take(&mut skinned.data);
        let mut joint_matrices = std::mem::take(&mut skinned.matrices);
        let joints_drawn = (joint_matrices.len() + self.instances.palette.baked_len()) as u32;
        // TAA's skinned motion vectors: the previous frame's joints and
        // blendshape weights in a second section of the palette (skin.rs,
        // its layout), read by the skinned prepass at `Fx::decal_info.w`.
        let taa_on = temporal;
        let skinned_motion = if taa_on || scene.post.motion_blur.is_some() {
            skin::append_previous(
                &scene.joint_palettes,
                &scene.previous_joint_palettes,
                &scene.skinned_instances,
                &skinned.previous_slots,
                &mut joint_matrices,
            )
        } else {
            None
        };
        // The crowd members' previous transforms, after every other slot.
        if let Some(at) = crowd_previous {
            debug_assert_eq!(at as usize, joint_matrices.len());
            joint_matrices.extend_from_slice(&crowd_frame.previous);
        }

        // Effects this frame.
        let post_fx = &scene.post;
        let particle_frame = self.particles.gpu.prepare(
            &self.device,
            &self.queue,
            scene,
            // Without compute every emitter simulates on the CPU.
            self.particles.simulation == ParticleSimulation::Auto
                && self.runs(FramePass::Particles),
            self.cache.textures.len(),
            camera.position,
            cam.forward,
        )?;
        // Mesh particles read the f32 vertices of a compact mesh too.
        for &m in &particle_frame.meshes {
            self.ensure_full(scene, m);
        }
        // The effect materials particles and strips draw with (effect.rs).
        {
            let textures = &self.cache.textures;
            self.particles.gpu.effects.prepare(
                &self.device,
                &self.queue,
                scene.effect_materials(),
                scene.shaders(),
                |t| textures.get(t.index()).map(|g| &g.view),
            )?;
        }
        // The frame's effects and its depth prepass (effects.rs).
        let effects = self.decide_effects(DecideEffectsInputs {
            scene,
            cam: &cam,
            batches: &batches,
            skinned_batches: &skinned_batches,
            skinned: &skinned,
            skinned_motion,
            temporal,
            taa_on,
            volumetric_fog: &volumetric_fog,
            particle_frame: &particle_frame,
            scene_water,
        });
        let FrameEffects {
            ao,
            gi,
            ao_target,
            contact,
            bent,
            outline: _,
            dof,
            motion_blur,
            screen_blur,
            god_rays,
            lens_flare,
            flare_sources_on,
            fluids_3d,
            depth_bands,
            ssr,
            custom_posts: _,
            motion_read,
            ui_depth: _,
            prepass,
        } = effects;
        plans.ssr = ssr;
        // What the frame will draw with beyond its batches starts compiling
        // on worker threads before the batches' pipelines (deferred.rs):
        // the terrains', the frame's longest compiles, the sky's and the
        // post chain's.
        if WORLD_3D {
            self.terrain.start_passes(scene, prepass);
        }
        {
            use crate::deferred::Start;
            match scene.environment.background {
                Background::Sky => self.sky_pipeline.start(),
                Background::Procedural { .. } => self.sky_look_pipeline.start(),
                Background::Color(_) => {}
            }
        }
        self.post.start();
        self.water.start(scene_water, self.frame_features);
        let (face_fluid, face_depth, object_space) =
            self.prepare_frame_pipelines(PrepareFramePipelinesInputs {
                scene,
                ao: ao_target || contact,
                batches: &mut batches,
                skinned_batches: &mut skinned_batches,
                skinned_visible: &mut skinned_visible,
                view_skinned: &mut view_skinned,
                probe_skinned: &mut probe_skinned,
                particle_frame: &particle_frame,
                view_plans: &view_plans,
                probe_frame: &probe_frame,
                fluids_3d,
                scene_water,
                prepass,
            });
        // Texture filters: a sprite group made with the other sampler is remade;
        // before any group is made this frame (the overlay's, first, below).
        for i in 0..self.cache.textures.len() {
            let f = scene.texture_filter(TextureHandle::from_index(i));
            if self.cache.filters.len() <= i {
                self.cache.filters.resize(i + 1, TextureFilter::Linear);
            }
            if self.cache.filters[i] != f {
                self.cache.filters[i] = f;
                self.cache.sprite_groups[i] = None;
            }
        }
        frame_pipelines(|| {
            self.particles.gpu.ensure_pipelines(
                &self.device,
                &particle_frame,
                self.samples,
                scene.shaders(),
            )
        });
        let fonts = &self.cache.fonts;
        let arrays = &self.cache.texture_arrays;
        let array_layers = |a: TextureArrayHandle| {
            scene
                .contains_texture_array(a)
                .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                .flatten()
                .map(|g| g.info.layers)
        };
        let overlay_frame = match &scene.overlay {
            Some(o) => Some(overlay::build(
                o,
                |i| fonts.get(i).map(|f| &f.layout),
                self.cache.textures.len(),
                array_layers,
                |v| scene.ui_vectors.get(v.index()),
                |t| scene.target_texture(t),
                overlay::OverlayTarget {
                    scale: overlay::OverlayRenderer::scale(self.out_height, o),
                    width: self.out_width,
                    height: self.out_height,
                },
                &mut self.overlay.mass_eases,
            )?),
            None => None,
        };
        let glass_on = self.prepare_frame_overlay(scene, &overlay_frame);
        // The frame goes through the glass source when glass reads it or a
        // colour filter recolours it on its way to the output.
        let color_filter = scene.color_filter.filter(|_| self.glass_targets.is_some());
        let routed = glass_on || color_filter.is_some();
        let line_frame = self.prepare_frame_lines(scene, &cam);
        // The strips and sprites of the camera, the views and the probe faces,
        // appended in that order to the frame's strip and sprite buffers
        // (sprite_arenas.rs).
        let SpriteArenas {
            ribbon_frame,
            light_slots,
            lit_range,
            view_ribbons,
            fluids,
            gpu_sprites,
            sprite_frame,
            view_sprites,
            view_glow_runs,
            view_boards,
            face_sprites,
            face_boards,
        } = self.build_sprite_arenas(SpriteArenaInputs {
            scene,
            camera: &camera,
            cam: &cam,
            aspect,
            view_frustum: &view_frustum,
            view_plans: &view_plans,
            particle_frame: &particle_frame,
            probe_frame: &probe_frame,
            probe_faces: &probe_faces,
            active_probes: &active_probes,
        })?;
        plans.metaballs = (fluids > 0).then_some(());
        // Order-independent transparency's frame mode (oit_frame.rs).
        let PlanOitFrameOutputs {
            oit_candidates,
            oit_frame,
        } = self.plan_oit_frame(PlanOitFrameInputs {
            scene,
            batches: &batches,
            skinned_batches: &skinned_batches,
            sprite_frame: &sprite_frame,
            ribbon_frame: &ribbon_frame,
            particle_frame: &particle_frame,
        });
        // The sprites' groups, tiles and sprite meshes (sprite_groups.rs).
        self.prepare_sprite_groups(PrepareSpriteGroupsInputs {
            scene,
            sprite_frame: &sprite_frame,
            view_sprites: &view_sprites,
            view_glow_runs: &view_glow_runs,
            face_sprites: &face_sprites,
            view_boards: &view_boards,
            face_boards: &face_boards,
            gpu_sprites: &gpu_sprites,
            light_slots: &light_slots,
            lit_range,
            particle_frame: &particle_frame,
        });
        if let Some(t) = scene.decal_atlas
            && t.index() >= self.cache.textures.len()
        {
            return Err(RenderError::InvalidHandle(format!(
                "decal atlas {}",
                t.index()
            )));
        }
        let lut_index = post_fx.color_grading.map(|g| g.lut.index());
        if let Some(g) = post_fx.color_grading
            && !(scene.contains_lut(g.lut)
                && self
                    .cache
                    .luts
                    .get(g.lut.index())
                    .is_some_and(Option::is_some))
        {
            return Err(RenderError::InvalidHandle(format!(
                "colour grading lut {}",
                g.lut.index()
            )));
        }
        let glow = post_fx.glow.is_some() && !sprite_frame.glow_runs.is_empty();

        // GPU-driven culling (cull_gpu.rs): each pass's list is indirect
        // draws the compute fills; the CPU sorts the transparent instances
        // and culls the skinned ones as before.
        // Per-object motion for motion blur and TAA's motion vectors: the
        // previous transforms at the same slots as the instances (GPU
        // culling compacts its camera pass's beside its own).
        let previous_valid = !scene.previous_transforms.is_empty()
            && scene.previous_transforms.len() == scene.instances.len();
        let blur_motion = motion_blur.is_some() && previous_valid;
        let sorted = transforms.len() as u32;
        // The skinned instances (their records in `skinned_data`, before
        // the CPU lists' copies): GPU culling tests their posed boxes
        // beside the instances when it runs (cull_gpu.rs `SkinnedCull`).
        let skinned_n = skinned_batches
            .iter()
            .map(|b| b.first + b.count)
            .max()
            .unwrap_or(0);
        // The static shadow cache's split and the shelter map (shadow_inputs.rs).
        let PrepareShadowInputsOutputs {
            shadow_split,
            shelter,
            shelter_redraw,
        } = self.prepare_shadow_inputs(PrepareShadowInputsInputs {
            scene,
            camera: &camera,
            batches: &batches,
            sorted,
            submission: &submission,
            sorted_serial,
            sorted_changed: sorted_changed.as_deref(),
            transforms: &transforms,
            spheres: &spheres,
            debris_instances: &debris_instances,
        });
        // How the frame culls (culling_plan.rs).
        let PlanCullingOutputs {
            max_passes,
            meshlet_taken,
            meshlet_max,
            meshlet_items,
            gpu_cull,
            skinned_gpu,
            cull_n,
            object_motion,
            auxiliary_instances,
            meshlet_casters,
            meshlet_all,
            meshlet_firsts,
        } = self.plan_culling(PlanCullingInputs {
            scene,
            batches: &batches,
            view_plans: &view_plans,
            reflection,
            shelter_redraw,
            sorted,
            skinned_n,
            crowd_gpu,
            object_space,
            skinned_batches: &skinned_batches,
            skinned_visible: &mut skinned_visible,
            skinned_culled: &mut skinned_culled,
            previous_valid,
            blur_motion,
            temporal,
        });
        let in_meshlet = |b: &Batch| b.count > 0 && meshlet_firsts.binary_search(&b.first).is_ok();
        let on_layers = |mask: u32| -> Vec<u32> {
            meshlet_all
                .iter()
                .copied()
                .filter(|&b| batches[b as usize].layers & mask != 0)
                .collect()
        };
        // The main camera's lists (camera_lists.rs).
        let PlanCameraListsOutputs {
            mut vis,
            mut cull_inside,
            mask,
            oit_mode,
            main_list,
            culled,
            mut blended_list,
        } = self.plan_camera_lists(PlanCameraListsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            batches: &batches,
            gpu_cull,
            cull_n,
            meshlet_items,
            meshlet_max,
            meshlet_taken,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            submission: &submission,
            sprite_frame: &sprite_frame,
            skinned_visible: &skinned_visible,
            skinned_culled,
            oit_frame,
            oit_candidates,
        });
        // The planar reflections' lists (reflection_lists.rs).
        let PlanReflectionListsOutputs {
            mut reflection_list,
            reflection_meshlets,
            mut reflection_blended,
        } = self.plan_reflection_lists(PlanReflectionListsInputs {
            batches: &batches,
            vis: &mut vis,
            gpu_cull,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            cull_inside: &mut cull_inside,
            submission: &submission,
            in_meshlet: &in_meshlet,
            on_layers: &on_layers,
            reflections: &reflections,
            mask,
            cull_n,
        });
        // The reflection probes' face lists (probe_lists.rs).
        let PlanProbeListsOutputs {
            mut probe_lists,
            mut probe_blended,
            probe_views,
        } = self.plan_probe_lists(PlanProbeListsInputs {
            batches: &batches,
            vis: &mut vis,
            gpu_cull,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            cull_inside: &mut cull_inside,
            submission: &submission,
            in_meshlet: &in_meshlet,
            probe_faces: &probe_faces,
            active_probes: &active_probes,
            probe_frame: &probe_frame,
            face_depth,
            meshlet_all: &meshlet_all,
            face_boards: &face_boards,
        });
        // The views' lists (view_lists.rs).
        let PlanViewListsOutputs {
            mut view_blended,
            view_views,
            mut view_lists,
        } = self.plan_view_lists(PlanViewListsInputs {
            batches: &batches,
            vis: &mut vis,
            gpu_cull,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            cull_inside: &mut cull_inside,
            submission: &submission,
            in_meshlet: &in_meshlet,
            on_layers: &on_layers,
            view_plans: &view_plans,
            view_boards: &view_boards,
            particle_frame: &particle_frame,
            cull_n,
        });
        // Terrains, scattered copies and grass (world_prepare.rs).
        let PrepareWorldOutputs {
            prev_view_proj,
            terrain_draws,
            terrain_casters,
            mut scatter_batches,
            grass_frame,
        } = self.prepare_world(PrepareWorldInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            prepass,
            crowd_frame: &crowd_frame,
            cpu_cards: &cpu_cards,
        })?;
        // The directional light's cascades.
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let shadow_size = self.info.shadow_map_size;
        // Every kind of caster: grass and scattered copies on a terrain
        // that casts nothing still need the cascades (their shadows, and
        // the grass canopy's own shading, read them).
        let has_casters = batches.iter().chain(&skinned_batches).any(|b| b.cast)
            || terrain_casters
            || grass_frame.casts()
            || scatter_batches.casts();
        // The local lights (lights_stage.rs).
        let PrepareLightsOutputs {
            lights_gpu,
            local_shadows,
            shadowed_dropped,
            contact_rows,
            volumetric,
        } = self.prepare_lights(PrepareLightsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            contact,
            volumetric_fog: &volumetric_fog,
            reflections: &reflections,
        });
        plans.volumetric = volumetric;
        // The shadow passes' plan and lists (shadow_plan.rs).
        let PlanShadowsOutputs {
            cascades,
            cascade_views,
            local_views,
            cascade_plans,
            cascade_lists,
            shelter_view,
            shelter_list,
            local_lists,
            local_frustums,
            local_plans,
        } = self.plan_shadows(PlanShadowsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            batches: &batches,
            light,
            has_casters,
            sphere_box,
            to_light,
            shadow_size,
            aspect,
            cull_n,
            meshlet_casters: &meshlet_casters,
            shadow_split: &shadow_split,
            skinned_batches: &skinned_batches,
            scatter_batches: &scatter_batches,
            grass_frame: &grass_frame,
            terrain_casters,
            terrain_draws: &terrain_draws,
            gpu_cull,
            vis: &mut vis,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            cull_inside,
            shelter: &shelter,
            local_shadows: &local_shadows,
        });
        // The GPU culling's encode and the instance records (instances_stage.rs).
        let UploadInstancesOutputs {
            cull_commands,
            gpu_counts,
            instance_upload_bytes,
            debris_commands,
        } = self.upload_instances(UploadInstancesInputs {
            scene,
            cam: &cam,
            batches: &batches,
            transforms,
            spheres,
            origins: &origins,
            debris_instances: &debris_instances,
            sorted,
            vis: &vis,
            gpu_cull,
            max_passes,
            auxiliary_instances,
            previous_valid,
            object_motion,
            prepass,
            skinned_gpu,
            skinned_batches: &skinned_batches,
            skinned_plain,
            skinned_data: &skinned_data,
            skinned: &skinned,
            reflection_list: &mut reflection_list,
            reflection_blended: &mut reflection_blended,
            blended_list: &mut blended_list,
            probe_lists: &mut probe_lists,
            probe_blended: &mut probe_blended,
            view_lists: &mut view_lists,
            view_blended: &mut view_blended,
        });
        // The skinned, sprite, particle, overlay and 2D light uploads (uploads_stage.rs).
        let UploadFrameOutputs {
            ui_uniform,
            lights_2d,
            occluders_2d,
        } = self.upload_frame(UploadFrameInputs {
            scene,
            target,
            cam: &cam,
            crowd_gpu,
            skinned_data: &skinned_data,
            joint_matrices: &joint_matrices,
            crowd_frame: &crowd_frame,
            scatter_batches: &mut scatter_batches,
            ribbon_frame: &ribbon_frame,
            sprite_frame: &sprite_frame,
            particle_frame: &particle_frame,
            gpu_sprites: &gpu_sprites,
            overlay_frame: &overlay_frame,
            light_slots: &light_slots,
        });
        // The frame uniform, its shadow slots and its copies (frame_uniform.rs).
        let WriteFrameUniformsOutputs { uniform, time: t } =
            self.write_frame_uniforms(WriteFrameUniformsInputs {
                scene,
                camera: &camera,
                cam: &cam,
                light,
                to_light,
                shadow_size,
                taa,
                frame_fog: &frame_fog,
                scene_water,
                lights_gpu: &lights_gpu,
                cascades: &cascades,
                local_shadows: &local_shadows,
                shelter: &shelter,
                probe_frame: &probe_frame,
                active_probes: &active_probes,
                probe_faces: &probe_faces,
                reflections: &reflections,
                ssr: ssr.as_ref(),
            });
        // The sun on screen, the flare's and the post chain's uniforms (sun_and_post.rs).
        let WriteSunAndPostOutputs {
            sun_ndc,
            sun_fade,
            god_visible,
        } = self.write_sun_and_post(WriteSunAndPostInputs {
            scene,
            camera: &camera,
            cam: &cam,
            aspect,
            light,
            to_light,
            uniform: &uniform,
            post_fx,
            god_rays,
            lens_flare,
            flare_sources_on,
            glow,
            sprite_frame: &sprite_frame,
        });
        // The effects uniform and the decals (fx_uniform.rs).
        let WriteFxUniformsOutputs {
            fx_uniform,
            decals,
            decals_dropped,
        } = self.write_fx_uniforms(WriteFxUniformsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            prev_view_proj,
            motion_read,
            skinned_motion,
            joints_drawn,
            ao_target,
            ao,
            gi: gi.is_some(),
            bent,
            reflection,
            reflections: &reflections,
        });
        // The views' uniforms (view_uniforms.rs).
        let WriteViewUniformsOutputs { viewport_format } =
            self.write_view_uniforms(WriteViewUniformsInputs {
                scene,
                target,
                view_plans: &view_plans,
                uniform: &uniform,
                fx_uniform: &fx_uniform,
                lights_on: !lights_gpu.is_empty(),
                light_slots: &light_slots,
                fluids,
                particle_soft: particle_frame.soft,
                post_fx,
                view_glow_runs: &view_glow_runs,
            });
        // The post chain's uniforms, the GI's history, SSR and TAA (post_uniforms.rs).
        let FramePostHistory { gi_copy, taa_blend } =
            self.write_frame_post_uniforms(WriteFramePostUniformsInputs {
                cam: &cam,
                camera: &camera,
                post_fx,
                scene,
                ao,
                gi,
                dof,
                motion_blur,
                god_rays,
                sun_ndc,
                god_visible,
                sun_fade,
                depth_bands,
                screen_blur,
                contact: contact_rows,
                ssr: ssr.as_ref(),
                taa,
                temporal,
                prepass,
            });
        let slot = (self.frame_index % 2) as usize;
        let gtao = match ao.map(|a| a.method) {
            Some(AoMethod::Gtao) => true,
            Some(AoMethod::Ssao | AoMethod::Screen) => false,
            _ => true,
        };
        self.shadows.shelter_key = shelter.as_ref().map(|s| s.2);
        let lens = post_fx.lens_distortion.filter(|l| l.amount != 0.0);
        plans.live_sky = LiveSkyPass::plan(&self.sky_capture);
        plans.gi_probes = GiProbesPass::plan(&self.gi_probes);
        plans.particles = particle_frame.any_gpu.then_some(ParticlesPlan {
            frame: &particle_frame,
            sprites: &gpu_sprites,
        });
        // The frame's passes, compiled into the render graph (cached while
        // the pass list stays the same; pass_list.rs).
        let passes = self.plan_pass_list(PassListInputs {
            scene,
            camera: &camera,
            effects: &effects,
            live_sky: plans.live_sky.is_some(),
            gi_probes: plans.gi_probes.is_some(),
            particles: plans.particles.is_some(),
            light_2d: plans.light_2d.is_some(),
            metaballs: plans.metaballs.is_some(),
            froxels: plans.volumetric.is_some(),
            cascades: cascades.len() as u32,
            shelter_redraw,
            local_shadows: &local_shadows,
            probes: !probe_frame.due.is_empty(),
            reflections: reflections.len() as u8,
            views: view_plans.len() as u32,
            glow,
            taa,
            smaa_t2x,
        });
        // The stylise passes' table, key and uniform (stylise_frame.rs).
        let PrepareFrameStyliseOutputs { stylise_key } =
            self.prepare_frame_stylise(PrepareFrameStyliseInputs {
                scene,
                camera: &camera,
                cam: &cam,
                upscale,
                grid_offset,
                lens,
                prepass,
            });
        // The 2D light map's size, its directions target and the light
        // pass's plan (pass_light_2d.rs).
        let (light_2d, lightmap_size, lightdir_on) =
            self.plan_light_2d(scene, &light_slots, lights_2d);
        plans.light_2d = light_2d;
        // The colour adjustments' table and the transmission's scene copy
        // (post_resources.rs).
        let adjusted = self.prepare_colour_lut(scene);
        let transmission = self.prepare_transmission(&batches, &blended_list, &skinned_visible);
        // The render graph for the frame's passes (graph_prepare.rs).
        self.prepare_graph(PrepareGraphInputs {
            scene,
            effects: &effects,
            passes,
            taa,
            lightmap_size,
            lightdir_on,
            gtao,
            scene_water,
            distortion: particle_frame.distortion,
            transmission,
            fluids,
            reflections: &reflections,
            object_space,
            oit_mode,
            adjusted,
            lut_index,
            stylise_key,
        })?;
        // FSR in TAA's place (fsr_frame.rs).
        let fsr = self.prepare_fsr(PrepareFsrInputs {
            scene,
            cam: &cam,
            taa,
            slot,
        });
        // The UI's tables and groups (ui_upload.rs).
        self.upload_ui(UploadUiInputs {
            scene,
            cam: &cam,
            overlay_frame: &overlay_frame,
            ui_uniform,
            glass_on,
        });
        // Fork s70: a covered surface's frame ends prepared: its passes would
        // only be decoded and validated by the browser's GPU process for a
        // canvas nobody sees.
        if matches!(target, RenderTarget::Surface)
            && COVERED.load(std::sync::atomic::Ordering::Relaxed)
        {
            if let Some(gc) = self.visibility.gpu_cull.as_mut().filter(|_| gpu_cull) {
                gc.dropped();
            }
            return Ok(FrameStats::default());
        }
        let surface_frame = match target {
            RenderTarget::Surface => {
                let (surface, config) = self.surface.as_ref().ok_or_else(|| {
                    RenderError::Unsupported(
                        "RenderTarget::Surface on a renderer created without one".into(),
                    )
                })?;
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                        let view = t.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(config.format),
                            ..Default::default()
                        });
                        let linear = config.view_formats.first().map(|&format| {
                            t.texture.create_view(&wgpu::TextureViewDescriptor {
                                format: Some(format),
                                ..Default::default()
                            })
                        });
                        Some((t, view, linear))
                    }
                    wgpu::CurrentSurfaceTexture::Occluded => {
                        return Err(RenderError::SurfaceOccluded);
                    }
                    other => {
                        if let Some(reason) = self.lost() {
                            return Err(RenderError::DeviceLost(reason));
                        }
                        surface.configure(&self.device, config);
                        return Err(RenderError::Surface(format!("{other:?}")));
                    }
                }
            }
            _ => None,
        };
        // The effect bind groups, rebuilt only when what they read changed
        // (fx_groups.rs).
        self.update_fx_groups(scene, ao_target || contact, reflection.is_some());
        // The water's and the 3D fluids' draws (water_fluids.rs).
        let PrepareWaterAndFluidsOutputs {
            water_draws,
            face_water,
            fluid_bounds,
            fluid_frame,
            face_fluids,
        } = self.prepare_water_and_fluids(PrepareWaterAndFluidsInputs {
            scene,
            cam: &cam,
            scene_water,
            reflections: &reflections,
            ao_target,
            ao,
            probe_frame: &probe_frame,
            probe_faces: &probe_faces,
            fluids_3d,
            face_fluid,
        });
        plans.fluid_3d = fluids_3d.then_some(fluid_frame.as_ref());
        // The frame's encoder and the compute before the graph (pre_graph_stage.rs).
        let EncodePreGraphOutputs {
            mut encoder,
            crowd_cull_queries,
            scatter_queries,
        } = self.encode_pre_graph(EncodePreGraphInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            crowd_gpu,
            crowd_frame: &crowd_frame,
            crowd_previous,
            vis: &vis,
            gpu_cull,
            skinned: &skinned,
            skinned_motion,
            joints_drawn,
            terrain_draws: &terrain_draws,
            grass_frame: &grass_frame,
        });
        let transmission_view = self
            .transmission
            .as_ref()
            .filter(|_| transmission)
            .map(|(_, view, chain)| (view, chain));
        let destination: &wgpu::TextureView = match target {
            RenderTarget::Offscreen => &self.persistent.offscreen_view,
            RenderTarget::View(v) => v,
            RenderTarget::Surface => &surface_frame.as_ref().expect("acquired above").1,
        };
        let output = if self.hdr.is_some() {
            &self.persistent.offscreen_view
        } else {
            destination
        };
        // Where the overlay blends: the output when it is sRGB, else its sRGB
        // view where the backend has one (linear light either way), else the
        // output itself with the shader encoding (a caller's non-sRGB view).
        let overlay_view = match target {
            _ if self.output_format.is_srgb() => None,
            RenderTarget::Offscreen => self.persistent.offscreen_linear.as_ref(),
            RenderTarget::Surface => surface_frame.as_ref().and_then(|f| f.2.as_ref()),
            RenderTarget::View(_) => None,
        };

        // The frame's statistics so far (frame_stats.rs).
        let FrameStatsOutputs { mut stats } = self.frame_stats(FrameStatsInputs {
            scene,
            batches: &batches,
            instance_upload_bytes,
            culled,
            main_list: &main_list,
            blended_list: &blended_list,
            local_shadows: &local_shadows,
            shadowed_dropped,
            sprite_frame: &sprite_frame,
            particle_frame: &particle_frame,
            ribbon_frame: &ribbon_frame,
            line_frame: &line_frame,
            decals,
            decals_dropped,
            lights_2d,
            occluders_2d,
            joints_drawn,
            taa_blend,
            oit_mode,
            gpu_counts,
            overlay_frame: &overlay_frame,
            skinned_visible: &skinned_visible,
            skinned_gpu,
        });
        let (_, frame_bind_group, sky_group, reflection_frame_group) = self
            .lights
            .environment
            .as_ref()
            .expect("environment generated above");
        // The probes' frame groups (`probes.rs`): per active probe, the
        // environment's prefiltered cube and the probe's in place of the
        // irradiance and prefiltered cubes; per due capture face, the
        // frame's own cubes under the face camera's uniform.
        // With the linked lists (oit.rs) the OIT pass binds the frame's and
        // its probes' groups again with the arena at binding 14.
        let oit_arena = self
            .oit
            .frame_list_layout
            .as_ref()
            .zip(self.oit.arena.as_ref())
            .filter(|_| oit_mode.is_some_and(oit::Oit::arena));
        let (probe_groups, face_groups, oit_frame_group, oit_probe_groups) = {
            let ibl = self.lights.ibl.views();
            // The reflections' read (frame.wgsl ssr_tex), whose rows the
            // uniforms switch: on in the main camera's and its probes'.
            let ssr_view = self.ssr.frame_view();
            let group_in = |layout: &wgpu::BindGroupLayout,
                            arena: Option<&wgpu::Buffer>,
                            label: &str,
                            uniform: wgpu::BindingResource,
                            a: &wgpu::TextureView,
                            b: &wgpu::TextureView| {
                let mut entries = vec![
                    uniform,
                    wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
                    wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
                    wgpu::BindingResource::TextureView(a),
                    wgpu::BindingResource::TextureView(b),
                    wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                    wgpu::BindingResource::Sampler(&ibl.sampler),
                    self.lights.light_buffers[0].as_entire_binding(),
                    self.lights.light_buffers[1].as_entire_binding(),
                    self.lights.light_buffers[2].as_entire_binding(),
                    self.lights.lightmaps.probe_buffer.as_entire_binding(),
                    self.lights.ltc.as_entire_binding(),
                    self.lights.caustic_grid.as_entire_binding(),
                    wgpu::BindingResource::TextureView(ssr_view),
                ];
                entries.extend(arena.map(|a| a.as_entire_binding()));
                bind_group(&self.device, label, layout, &entries)
            };
            let oit_frame_group = oit_arena.map(|(layout, arena)| {
                group_in(
                    layout,
                    Some(&arena.buffer),
                    "frame (oit lists)",
                    self.lights.frame_buffer.as_entire_binding(),
                    &ibl.irradiance,
                    &ibl.prefiltered,
                )
            });
            let mut oit_probe_groups: Vec<wgpu::BindGroup> = Vec::new();
            let group = |k: usize, a: &wgpu::TextureView, b: &wgpu::TextureView| {
                bind_group(
                    &self.device,
                    "frame (reflection probe)",
                    &self.frame_layout,
                    &[
                        self.probes.binding(k),
                        wgpu::BindingResource::TextureView(&self.shadows.cascade_map.array_view),
                        wgpu::BindingResource::TextureView(&self.shadows.local_map.array_view),
                        wgpu::BindingResource::TextureView(a),
                        wgpu::BindingResource::TextureView(b),
                        wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                        wgpu::BindingResource::Sampler(&ibl.sampler),
                        self.lights.light_buffers[0].as_entire_binding(),
                        self.lights.light_buffers[1].as_entire_binding(),
                        self.lights.light_buffers[2].as_entire_binding(),
                        self.lights.lightmaps.probe_buffer.as_entire_binding(),
                        self.lights.ltc.as_entire_binding(),
                        self.lights.caustic_grid.as_entire_binding(),
                        wgpu::BindingResource::TextureView(ssr_view),
                    ],
                )
            };
            let probe_groups: Vec<wgpu::BindGroup> = probe_frame.active[..probe_frame.sampled]
                .iter()
                .enumerate()
                .map(|(k, (p, slot))| {
                    let view = match (slot, p.baked) {
                        (Some(s), _) => &self.probes.slots[*s].view,
                        (None, Some(h)) => {
                            &self.cache.environment_maps[h.index()]
                                .as_ref()
                                .expect("a loaded map (Probes::prepare)")
                                .view
                        }
                        (None, None) => unreachable!("a probe without a slot is baked"),
                    };
                    if let Some((layout, arena)) = oit_arena {
                        oit_probe_groups.push(group_in(
                            layout,
                            Some(&arena.buffer),
                            "frame (reflection probe, oit lists)",
                            self.probes.binding(k),
                            &ibl.prefiltered,
                            view,
                        ));
                    }
                    group(k, &ibl.prefiltered, view)
                })
                .collect();
            let first = probe_frame.active.len();
            let face_groups: Vec<wgpu::BindGroup> = (0..6 * probe_frame.due.len())
                .map(|j| group(first + j, &ibl.irradiance, &ibl.prefiltered))
                .collect();
            (probe_groups, face_groups, oit_frame_group, oit_probe_groups)
        };
        let probe_binding = (!probe_groups.is_empty()).then_some(probe_select::ProbeGroups {
            boxes: &probe_boxes,
            groups: &probe_groups,
            frame: frame_bind_group,
        });
        let oit_probe_binding = oit_frame_group
            .as_ref()
            .filter(|_| !oit_probe_groups.is_empty())
            .map(|frame| probe_select::ProbeGroups {
                boxes: &probe_boxes,
                groups: &oit_probe_groups,
                frame,
            });
        let groups = self.lights.fx_groups.as_ref().expect("made above");
        let (fx_group, sprite_fx_group, glow_fx_group) = (&groups.fx, &groups.sprite, &groups.glow);
        // The physical pipelines' effect group: the same plus the scene copy
        // (a placeholder in a frame without transmission).
        let physical_fx_group = self.lit.physical.as_ref().and_then(|p| {
            batches
                .iter()
                .chain(&skinned_visible)
                .any(|b| b.physical)
                .then(|| {
                    let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                        &self.cache.textures[t.index()].view
                    });
                    let ao_view = if ao_target || contact {
                        self.graph.view("ao")
                    } else {
                        &self.post.white
                    };
                    fx::fx_group(
                        &self.device,
                        "physical fx",
                        &p.fx_layout,
                        &[
                            self.lights.fx_buffer.as_entire_binding(),
                            wgpu::BindingResource::TextureView(ao_view),
                            wgpu::BindingResource::TextureView(atlas_view),
                            wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                            wgpu::BindingResource::TextureView(
                                transmission_view.map_or(&self.post.white, |(v, _)| v),
                            ),
                        ],
                        &self.lights.decal_buffers,
                    )
                })
        });
        let scene_depth = self
            .graph
            .try_view("lindepth")
            .unwrap_or(&self.post.far_depth);
        let particle_group = (!particle_frame.systems.is_empty()).then(|| {
            let copy = self
                .graph
                .try_view("scene-copy")
                .unwrap_or(&self.post.white);
            self.particles
                .gpu
                .draw_group(&self.device, scene_depth, copy)
        });
        let metaball_groups: Vec<wgpu::BindGroup> = post::FIELD_NAMES
            .iter()
            .map_while(|name| self.graph.try_view(name))
            .map(|v| self.metaballs.group(&self.device, v))
            .collect();
        if !water_draws.is_empty() {
            (stats.water_submits, stats.foam_steps) = self.water.submitted;
        }
        // The planar reflection's effect group: its decals, no AO.
        let reflection_fx_group: Vec<wgpu::BindGroup> = (0..reflections.len())
            .map(|k| {
                let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                    &self.cache.textures[t.index()].view
                });
                fx::fx_group(
                    &self.device,
                    "fx (planar reflection)",
                    &self.fx_layout,
                    &[
                        self.reflection_fx_buffer[k].as_entire_binding(),
                        wgpu::BindingResource::TextureView(&self.post.white),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        // Not the reflection itself: this pass draws it.
                        wgpu::BindingResource::TextureView(&self.post.white),
                    ][..if self.planar_slot() { 5 } else { 4 }],
                    &self.lights.decal_buffers,
                )
            })
            .collect();
        let view_groups = self.view_groups(scene, &view_plans);
        // Each pass's effect group without the planar reflection, for its
        // lightmapped draws (`fx_lean_group`), by the pass's own group.
        let reflection_fx_lean: Vec<Option<wgpu::BindGroup>> = (0..reflections.len())
            .map(|k| {
                let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                    &self.cache.textures[t.index()].view
                });
                self.fx_lean_group(&self.reflection_fx_buffer[k], &self.post.white, atlas_view)
            })
            .collect();
        let fx_leans: Vec<(&wgpu::BindGroup, &wgpu::BindGroup)> =
            std::iter::once((fx_group, groups.fx_lean.as_ref()))
                .chain(
                    reflection_fx_group
                        .iter()
                        .zip(reflection_fx_lean.iter().map(Option::as_ref)),
                )
                .chain(view_groups.iter().map(|g| (&g.1, g.2.as_ref())))
                .filter_map(|(fx, lean)| Some((fx, lean?)))
                .collect();
        // Each view's metaball field groups (its own fields' textures).
        let view_fields: Vec<Vec<wgpu::BindGroup>> = view_plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                slot.fields
                    .iter()
                    .map(|v| self.metaballs.group(&self.device, v))
                    .collect()
            })
            .collect();
        // Each view's sprite effect group: its 2D light map (white
        // without 2D lighting) and its image's size.
        let view_sprite_fx: Vec<wgpu::BindGroup> = view_plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                bind_group(
                    &self.device,
                    "sprite fx (view)",
                    &self.sprite_fx_layout,
                    &[
                        wgpu::BindingResource::TextureView(
                            slot.light_map
                                .as_ref()
                                .map_or(&self.default_white, |(_, v)| v),
                        ),
                        wgpu::BindingResource::Sampler(&self.sprite_sampler),
                        slot.sprite_fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(
                            slot.light_dir.as_ref().unwrap_or(&self.post.black),
                        ),
                    ],
                )
            })
            .collect();
        // Each view's particle and line groups over its own camera's
        // uniforms (views.rs).
        let view_particle_groups: Vec<Option<wgpu::BindGroup>> = view_plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                // Soft particles fade against the view's own depth.
                let depth = slot
                    .prepass
                    .as_ref()
                    .map_or(&self.post.far_depth, |pre| &pre.lindepth);
                (!particle_frame.systems.is_empty()).then(|| {
                    self.particles.gpu.draw_group_of(
                        &self.device,
                        &slot.particle_buffer,
                        depth,
                        &self.post.white,
                    )
                })
            })
            .collect();
        // Each due probe face's particle and line groups over its own
        // camera's uniforms (probes.rs); soft particles fade against the
        // face's own depth prepass.
        let (face_particle_groups, face_line_groups): (Vec<_>, Vec<_>) = probe_faces
            .iter()
            .enumerate()
            .flat_map(|(d, cams)| {
                let (slot, i) = probe_frame.due[d];
                let size = active_probes[i].face_size();
                let position = active_probes[i].position;
                let depth = if face_depth {
                    &self.probes.targets_of(slot).prepass.lindepth
                } else {
                    &self.post.far_depth
                };
                cams.iter().map(move |f| (f, position, size, depth))
            })
            .map(|(f, position, size, depth)| {
                let (particles, lines) =
                    views::camera_gpu(f, position, (size, size), scene.css_height);
                let buffer = |label, contents: &[u8]| {
                    crate::buffer_init(
                        &self.device,
                        &self.queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(label),
                            contents,
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    )
                };
                let pb = buffer("probe face particles", bytemuck::bytes_of(&particles));
                let lb = buffer("probe face lines", bytemuck::bytes_of(&lines));
                (
                    (!particle_frame.systems.is_empty()).then(|| {
                        self.particles
                            .gpu
                            .draw_group_of(&self.device, &pb, depth, &self.post.white)
                    }),
                    self.lines.view_group(&self.device, &lb),
                )
            })
            .unzip();
        let view_line_groups: Vec<wgpu::BindGroup> = view_plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                self.lines.view_group(&self.device, &slot.line_buffer)
            })
            .collect();
        let palette_group = self.instances.palette.bind_group.as_ref();
        // The main pass draws physical materials with their pipelines, the
        // planar reflection with the standard ones.
        let lit_skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, b.physical))
            .collect();
        let lit_skinned_standard: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, false))
            .collect();
        let shadow_skinned: Vec<&ShadowSet> = skinned_batches
            .iter()
            .map(|b| match b.shader {
                Some(sh) => self.cache.hook_shadow_pipelines[&(sh, b.side, true)].get(),
                None => self.shadows.shadow_pipelines[&(b.side, true, b.dissolve)].get(),
            })
            .collect();
        let prepass_skinned: Vec<&wgpu::RenderPipeline> = if prepass {
            skinned_visible
                .iter()
                .map(|b| {
                    let hooked = b
                        .shader
                        .and_then(|sh| self.cache.hook_prepass_pipelines.get(&(sh, b.side, true)));
                    let array = self
                        .lit
                        .array_maps
                        .as_ref()
                        .filter(|_| b.array)
                        .and_then(|a| a.prepass.get(&(b.side, true)));
                    hooked
                        .or(array)
                        .map(|p| p.get())
                        .or_else(|| {
                            self.prepass
                                .prepass_pipelines
                                .get(&(b.side, true))
                                .map(|p| p.get(self.frame_features))
                        })
                        .unwrap_or(lit_skinned[0])
                })
                .collect()
        } else {
            Vec::new()
        };
        let culled_gpu = self.visibility.gpu_cull.as_ref().filter(|_| gpu_cull);
        let crowd_cull = self.crowd_gpu.as_ref().filter(|_| crowd_gpu);
        // The effect groups of the standard and the physical pipelines, in
        // the passes that draw physical materials with theirs.
        let physical_fx = physical_fx_group.as_ref().map(|p| (fx_group, p));
        // What every pass below draws with (draw_ctx.rs).
        let ctx = DrawCtx {
            r: self,
            scene,
            batches: &batches,
            culled_gpu,
            crowd_cull,
            fx_leans: &fx_leans,
            palette_group,
            skinned_gpu,
            sprite_frame: &sprite_frame,
            blended_list: &blended_list,
            main_list: &main_list,
            skinned_visible: &skinned_visible,
            lit_skinned: &lit_skinned,
            physical_fx,
            fx_group,
            sprite_fx_group,
            glow_fx_group,
            terrain_indices: self.terrain.index_buffer(),
            terrain_draws: &terrain_draws,
            scatter_batches: &scatter_batches,
            grass_frame: &grass_frame,
            shadow_skinned: &shadow_skinned,
            skinned_batches: &skinned_batches,
            terrain_casters,
        };
        let camera_cards = ctx.card_source(
            vis.cull_passes
                .iter()
                .position(|p| p.kind == cull_gpu::PassKind::Main)
                .map(|g| g as u32),
        );
        let terrain_indices = ctx.terrain_indices;

        // Per-pass CPU encode time, with a clock (`Renderer::set_clock`),
        // and GPU time where the device has timestamps (`timing.rs`): each
        // graph pass owns the queries of the render and compute passes it
        // opened.
        let clock = self.clock;
        let mut pass_ms: Vec<PassTiming> = Vec::new();
        let mut pass_queries: Vec<(String, std::ops::Range<u32>)> = crowd_cull_queries
            .map(|q| ("cull: crowds".to_string(), q))
            .into_iter()
            .chain(scatter_queries.into_iter().map(|(n, q)| (n.to_string(), q)))
            .collect();
        plans.shelter = shelter
            .as_ref()
            .zip(shelter_list.as_ref())
            .map(|(s, list)| ShelterPlan {
                frustum: &s.1,
                list,
                view: shelter_view,
            });
        plans.local_face = Some(LocalFacePlan {
            lists: &local_lists,
            plans: &local_plans,
            frustums: &local_frustums,
            views: &local_views,
        });
        plans.cascade = Some(CascadePlan {
            cascades: &cascades,
            lists: &cascade_lists,
            plans: &cascade_plans,
            views: &cascade_views,
        });
        plans.probes = (!probe_frame.due.is_empty()).then(|| ProbesPlan {
            meshlet_views: &probe_views,
            batches: &batches,
            scene,
            probe_frame: &probe_frame,
            face_depth,
            face_water: &face_water,
            face_fluids: &face_fluids,
            probe_faces: &probe_faces,
            active_probes: &active_probes,
            face_groups: &face_groups,
            fx_group,
            probe_lists: &probe_lists,
            probe_skinned: &probe_skinned,
            terrain_indices,
            terrain_draws: &terrain_draws,
            scatter_batches: &scatter_batches,
            probe_blended: &probe_blended,
            face_boards: &face_boards,
            glow_fx_group,
            face_particle_groups: &face_particle_groups,
            particle_frame: &particle_frame,
            line_frame: &line_frame,
            face_line_groups: &face_line_groups,
            face_sprites: &face_sprites,
        });
        plans.reflection = (!reflections.is_empty()).then(|| ReflectionPlan {
            meshlet_views: &reflection_meshlets,
            batches: &batches,
            scene,
            reflection_frame_group,
            reflection_fx_group: &reflection_fx_group,
            reflection_list: &reflection_list,
            skinned_visible: &skinned_visible,
            lit_skinned_standard: &lit_skinned_standard,
            terrain_indices,
            reflections: &reflections,
            terrain_draws: &terrain_draws,
            reflection_blended: &reflection_blended,
        });
        plans.view = (!view_plans.is_empty()).then(|| ViewPlan {
            meshlet_views: &view_views,
            batches: &batches,
            scene,
            view_plans: &view_plans,
            lights_2d,
            view_groups: &view_groups,
            view_lists: &view_lists,
            view_skinned: &view_skinned,
            terrain_indices,
            terrain_draws: &terrain_draws,
            view_blended: &view_blended,
            view_boards: &view_boards,
            view_sprite_fx: &view_sprite_fx,
            view_particle_groups: &view_particle_groups,
            particle_frame: &particle_frame,
            view_ribbons: &view_ribbons,
            line_frame: &line_frame,
            view_line_groups: &view_line_groups,
            view_sprites: &view_sprites,
            view_fields: &view_fields,
            view_glow_runs: &view_glow_runs,
            post_fx,
        });
        plans.prepass = (prepass).then(|| PrepassPlan {
            water_draws: &water_draws,
            object_space,
            frame_bind_group,
            fx_group,
            main_list: &main_list,
            culled_gpu,
            object_motion,
            batches: &batches,
            skinned_visible: &skinned_visible,
            prepass_skinned: &prepass_skinned,
            terrain_indices,
            terrain_draws: &terrain_draws,
            view_frustum: &view_frustum,
            scatter_batches: &scatter_batches,
            cards: camera_cards,
            grass_frame: &grass_frame,
        });
        plans.main = Some(MainPlan {
            scene,
            water_draws: &water_draws,
            fluid_frame: &fluid_frame,
            transmission_view,
            taa: temporal,
            frame_bind_group,
            fx_group,
            main_list: &main_list,
            batches: &batches,
            physical_fx,
            probe_binding,
            skinned_visible: &skinned_visible,
            lit_skinned: &lit_skinned,
            terrain_indices,
            terrain_draws: &terrain_draws,
            view_frustum: &view_frustum,
            scatter_batches: &scatter_batches,
            cards: camera_cards,
            grass_frame: &grass_frame,
            fluid_bounds,
            blended_list: &blended_list,
            sprite_frame: &sprite_frame,
            sprite_fx_group,
            particle_group: &particle_group,
            particle_frame: &particle_frame,
            ribbon_frame: &ribbon_frame,
            line_frame: &line_frame,
            metaball_groups: &metaball_groups,
            volumetric_fog,
            oit: oit_mode,
            oit_frame_group: oit_frame_group.as_ref(),
            oit_probe_binding,
            gi_copy,
        });
        plans.glow = glow.then_some(sprite_frame.glow_runs.as_slice());
        plans.custom_post = (!effects.custom_posts.is_empty()).then(|| CustomPostPlan {
            scene,
            posts: &effects.custom_posts,
        });
        for &pass in self.graph.order() {
            // What the pass needs here: its kind's, and what this frame's
            // instance of it adds (schedule.rs).
            encoder.expect(
                self.info.caps,
                pass.needs()
                    | plans.needs(pass)
                    | match pass {
                        FramePass::Taa | FramePass::TaaSharpen if fsr => Caps::COMPUTE,
                        _ => Caps::NONE,
                    },
            );
            let t0 = clock.map(|c| c());
            let q0 = encoder.used();
            let _span = pass.redrawn().then(FrameSpan::open);
            match pass {
                _ if plans.encode(
                    pass,
                    &mut EncodeCx {
                        ctx: &ctx,
                        encoder: &mut encoder,
                        stats: &mut stats,
                        frame_bind_group,
                        scene_depth,
                        sky_group,
                        slot,
                    },
                ) => {}
                post_pass => {
                    let post_output = match (&self.glass_targets, routed) {
                        (Some(g), true) => &g.source_view,
                        _ => output,
                    };
                    stats.draw_calls += self.graph.encode_post(
                        &mut encoder,
                        &self.post,
                        &self.persistent,
                        post_pass,
                        slot,
                        post_output,
                        gtao,
                    );
                }
            }
            if let (Some(c), Some(t0)) = (clock, t0) {
                pass_ms.push(PassTiming {
                    name: pass.name(),
                    start: t0,
                    ms: (c() - t0).max(0.0),
                });
            }
            if encoder.timed() {
                pass_queries.push((pass.name(), q0..encoder.used()));
            }
        }
        encoder.expect(self.info.caps, self.info.caps);
        for &(slot, _) in &probe_frame.due {
            self.probes.slots[slot].captured = Some(scene.tick);
        }
        // The passes after the graph's are timed as its own are, each listed
        // only in a frame where it drew: the viewport views, the UI layers,
        // and the overlay with the glass and the lines drawn among it, so
        // the UI's budget (`docs/ui.md` §6.3) reads from `gpuPasses`.
        let mut timings = Timings {
            clock,
            pass_ms,
            pass_queries,
        };
        // Everything after the graph is drawn again each frame (FrameSpan).
        let tail = FrameSpan::open();
        let views_at = timings.begin(&encoder, &stats);
        // Viewport views (split-screen, views.rs) over the finished frame,
        // under the lines and the overlay.
        for p in view_plans
            .iter()
            .filter(|p| matches!(p.dest, views::Dest::Viewport(_)))
        {
            // Into the glass source when the frame is routed through it,
            // so the blit at the end keeps them.
            let routed_view = self
                .glass_targets
                .as_ref()
                .filter(|_| routed)
                .and_then(|g| match overlay_view {
                    None => Some(&g.source_view),
                    Some(_) => g.source_linear_view.as_ref(),
                });
            let dest = routed_view.unwrap_or(overlay_view.unwrap_or(output));
            self.views.resolve(
                &self.device,
                &self.queue,
                &mut encoder,
                p,
                post_fx.tonemap,
                self.hdr.as_ref().map_or([0.0; 4], |h| h.curve),
                dest,
                viewport_format,
                false,
            );
            stats.draw_calls += 1;
        }
        timings.end("views", views_at, &encoder, &stats);
        let layers_at = timings.begin(&encoder, &stats);
        // UI layers (src/ui/pass.rs): each drawn into its own target before
        // the overlay that composites it.
        if let Some(of) = &overlay_frame
            && !of.layers.is_empty()
        {
            stats.draw_calls += self.draw_ui_layers(&mut encoder, scene, of);
        }
        timings.end("ui-layers", layers_at, &encoder, &stats);
        let overlay_at = timings.begin(&encoder, &stats);
        // Glass over overlay content (src/glass.rs): when overlay runs come
        // before a glass run (or a UI backdrop), the overlay draws into the
        // glass source, the pyramid is rebuilt before each such run, and the
        // source is copied to the output at the end. Otherwise the source is
        // copied and blurred once, before the overlay, which draws into the
        // output. A UI blend that reads what is under it always draws into
        // the source, copied before it to what it reads, and so does a
        // backdrop layer, which captures the source before its composite.
        let runs: &[overlay::OverlayRun] = overlay_frame.as_ref().map_or(&[], |f| f.main_runs());
        let captures: Vec<(usize, u32)> = overlay_frame.as_ref().map_or(Vec::new(), |f| {
            runs.iter()
                .enumerate()
                .filter_map(|(i, r)| f.capture(r).map(|k| (i, k)))
                .collect()
        });
        let decode = !self.output_format.is_srgb() && self.output_format != HDR_FORMAT;
        let mut rebuilds = Vec::new();
        let mut copies = Vec::new();
        let mut layered = None;
        if let (Some(g), true) = (&self.glass_targets, routed) {
            rebuilds = glass::rebuilds(
                &runs
                    .iter()
                    .map(|r| r.source.needs_pyramid())
                    .collect::<Vec<_>>(),
            );
            copies = glass::rebuilds(
                &runs
                    .iter()
                    .map(|r| r.source.needs_copy())
                    .collect::<Vec<_>>(),
            );
            // A colour filter keeps the whole overlay in the source too.
            let over_content = rebuilds.first().is_some_and(|&i| i > 0)
                || rebuilds.len() > 1
                || !copies.is_empty()
                || !captures.is_empty()
                || color_filter.is_some();
            // The overlay draws into the source through the view it blends in.
            layered = match (over_content, overlay_view) {
                (false, _) => None,
                (true, None) => Some(&g.source_view),
                (true, Some(_)) => g.source_linear_view.as_ref(),
            };
            if layered.is_none() {
                // The source as it is now to the output: filtered (only the
                // scene, where the overlay has no view into the source),
                // and the pyramid where glass reads it.
                stats.draw_calls += match &color_filter {
                    Some(f) => self
                        .glass
                        .filter(&self.queue, &mut encoder, g, output, f, decode),
                    None => self.glass.blit(&mut encoder, g, output),
                };
                if glass_on {
                    stats.draw_calls += self.glass.pyramid(&self.queue, &mut encoder, g, decode);
                }
                rebuilds.clear();
            }
        }
        // Lines over the finished frame (debug draw, screen lines), under
        // the overlay, where the overlay draws (lines.rs).
        if line_frame.overlay.1 > 0 {
            let srgb_view = overlay_view.is_some();
            self.lines.set_overlay_uniform(
                &self.queue,
                &lines::LineUniform::new(
                    cam.view_proj,
                    cam.proj,
                    self.out_width,
                    self.out_height,
                    scene.css_height,
                    !srgb_view && !self.output_format.is_srgb() && self.output_format != HDR_FORMAT,
                ),
            );
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            stats.draw_calls +=
                self.lines
                    .draw_overlay(&mut encoder, target, srgb_view, &line_frame);
        }
        // The overlay: over the finished frame, in the output (or its sRGB
        // view), or in the glass source when what it draws reads what is
        // under it.
        if let (false, Some(group0)) = (runs.is_empty(), self.ui_main_group()) {
            let kind = match overlay_view {
                Some(_) => ui::pass::TargetKind::Linear,
                None => ui::pass::TargetKind::Output,
            };
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            // The runs between the pyramid's builds and the target's copies,
            // each in a pass of its own: before run `at`, what `events`
            // lists (true: a pyramid, false: a copy).
            let mut events: Vec<(usize, bool)> = rebuilds
                .iter()
                .map(|&i| (i, true))
                .chain(copies.iter().map(|&i| (i, false)))
                .collect();
            og_core::sort::sort_by_key(&mut events, |e| *e);
            let mut cuts: Vec<usize> = events
                .iter()
                .map(|e| e.0)
                .chain(captures.iter().map(|c| c.0))
                .collect();
            og_core::sort::sort_by_key(&mut cuts, |c| *c);
            cuts.dedup();
            cuts.push(runs.len());
            let mut from = 0;
            for &to in &cuts {
                if to > from {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("overlay"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    stats.draw_calls +=
                        self.draw_overlay_runs(&mut pass, scene, &runs[from..to], group0, kind);
                }
                // What the runs from `to` read, as the target stands now.
                if let Some(g) = self.glass_targets.as_ref() {
                    for &(_, pyramid) in events.iter().filter(|e| e.0 == to) {
                        stats.draw_calls += if pyramid {
                            self.glass.pyramid(&self.queue, &mut encoder, g, decode)
                        } else {
                            self.glass.copy_behind(&mut encoder, g)
                        };
                    }
                    if layered.is_some() {
                        for &(_, k) in captures.iter().filter(|c| c.0 == to) {
                            stats.draw_calls += self.record_capture(&mut encoder, k);
                        }
                    }
                }
                from = to;
            }
        }
        if let (Some(g), Some(_)) = (&self.glass_targets, layered) {
            stats.draw_calls += match &color_filter {
                Some(f) => self
                    .glass
                    .filter(&self.queue, &mut encoder, g, output, f, decode),
                None => self.glass.blit(&mut encoder, g, output),
            };
        }
        timings.end("overlay", overlay_at, &encoder, &stats);
        if let Some(hdr) = &self.hdr
            && !matches!(target, RenderTarget::Offscreen)
        {
            let at = timings.begin(&encoder, &stats);
            hdr.encode(&self.device, &mut encoder, output, destination);
            stats.draw_calls += 1;
            timings.end("hdr-output", at, &encoder, &stats);
        }
        drop(tail);
        let Timings {
            pass_ms,
            pass_queries,
            ..
        } = timings;
        let gpu_queries = encoder.used();
        self.pass_ms = pass_ms;
        // HDR already rendered scene + overlay into the float offscreen target.
        // Keep that paper-white-linear frame for capture, not the encoded surface.
        if let Some((frame, ..)) = surface_frame
            .as_ref()
            .filter(|_| self.keep_frame && self.hdr.is_none())
        {
            self.keep_presented(&mut encoder, &frame.texture);
        }
        // A bisect's knob (dev builds): the debris and cull command buffers
        // in a submit of their own, before the frame's (settings.rs
        // `debug_cull_apart`).
        #[cfg(not(feature = "publish"))]
        let (debris_commands, cull_commands) =
            if super::settings::DEBUG_CULL_APART.load(std::sync::atomic::Ordering::Relaxed) {
                self.queue
                    .submit(debris_commands.into_iter().chain(cull_commands));
                (Vec::new(), Vec::new())
            } else {
                (
                    debris_commands.into_iter().collect::<Vec<_>>(),
                    cull_commands.into_iter().collect::<Vec<_>>(),
                )
            };
        let _submission = self.queue.submit(
            debris_commands
                .into_iter()
                .chain(cull_commands)
                .chain(Some(encoder.finish())),
        );
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(target, RenderTarget::Offscreen) {
            self.offscreen_submissions.push_back(_submission);
        }
        self.shadows.shadow_cache.submitted();
        if let Some(arena) = self
            .oit
            .arena
            .as_ref()
            .filter(|_| oit_mode == Some(oit::Oit::List))
        {
            arena.after_submit();
        }
        if let Some(gc) = self.visibility.gpu_cull.as_mut().filter(|_| gpu_cull) {
            gc.after_submit();
        }
        if let (Some(t), Some(c)) = (self.gpu_timer.as_mut(), self.clock) {
            t.after_submit(
                &self.queue,
                self.frame_index,
                pass_queries,
                gpu_queries,
                c(),
            );
        }
        if let Some((frame, ..)) = surface_frame {
            self.queue.present(frame);
        }
        if temporal {
            // A history reset since the last frame (a cut, a resize, a
            // look switch) starts the count again.
            self.history.taa_history_frames = match self.history.taa_history {
                Some(_) => self.history.taa_history_frames + 1,
                None => 1,
            };
            self.history.taa_history = Some(cam.view_proj);
        } else {
            self.history.taa_history_frames = 0;
        }
        // The main pass copied its lit image for the next frame's GI.
        self.gi_history = gi_copy.then_some(self.frame_index);
        self.history.prev_view_proj = Some(cam.view_proj);
        self.history.previous_time = Some(t[0]);
        let wind = scene.wind.unwrap_or(Wind {
            strength: 0.0,
            ..Wind::default()
        });
        self.instances.previous_sway.clear();
        self.instances
            .previous_sway
            .extend(scene.instances.iter().map(|i| i.sway));
        self.history.previous_wind = Some((wind, scene.tick, scene.tick_rate));
        self.frame_index += 1;
        // The first frame made the scene's pipelines: keep them now, not
        // only when the renderer is dropped (a killed process never is).
        if self.frame_index == 1
            && let Some(cache) = &mut self.pipeline_disk
        {
            cache.save();
        }
        // The full lit variants this frame asked for compile now
        // (pipeline_features.rs).
        crate::deferred::release_background();
        stats.compile_wait_us = crate::deferred::take_waited_us().min(u64::from(u32::MAX)) as u32;
        Ok(stats)
    }
}

/// The frame's pass timings while it is encoded: each render-graph pass's
/// CPU encoding and the queries of the passes it opened (`timing.rs`).
struct Timings {
    clock: Option<fn() -> f64>,
    pass_ms: Vec<PassTiming>,
    pass_queries: Vec<(String, std::ops::Range<u32>)>,
}

/// Where a pass outside the graph began: the clock, the encoder's queries
/// and the frame's draws.
struct PassStart {
    t0: Option<f64>,
    q0: u32,
    draws: u32,
}

impl Timings {
    fn begin(&self, encoder: &Encoder, stats: &FrameStats) -> PassStart {
        PassStart {
            t0: self.clock.map(|c| c()),
            q0: encoder.used(),
            draws: stats.draw_calls,
        }
    }

    /// Lists `name` from `at` to now when it drew anything.
    fn end(&mut self, name: &str, at: PassStart, encoder: &Encoder, stats: &FrameStats) {
        if stats.draw_calls == at.draws {
            return;
        }
        if let (Some(c), Some(t0)) = (self.clock, at.t0) {
            self.pass_ms.push(PassTiming {
                name: name.into(),
                start: t0,
                ms: (c() - t0).max(0.0),
            });
        }
        if encoder.timed() {
            self.pass_queries.push((name.into(), at.q0..encoder.used()));
        }
    }
}

/// A shadow pass into one layer: cleared (depth 1, and distance / range 1:
/// nothing nearer) or loaded over what the layer holds, its depth kept
/// for a later pass or not.
fn shadow_target<'e>(
    encoder: &'e mut crate::Encoder,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    load: bool,
    keep_depth: bool,
) -> wgpu::RenderPass<'e> {
    let store = |keep| {
        if keep {
            wgpu::StoreOp::Store
        } else {
            wgpu::StoreOp::Discard
        }
    };
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("shadow"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if load {
                    wgpu::LoadOp::Load
                } else {
                    wgpu::LoadOp::Clear(wgpu::Color::WHITE)
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: if load {
                    wgpu::LoadOp::Load
                } else {
                    wgpu::LoadOp::Clear(1.0)
                },
                store: store(keep_depth),
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
    scene.previous_transforms.len() == scene.instances.len()
        && scene
            .previous_transforms
            .iter()
            .zip(&scene.instances)
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
