//! The frame: its stages in order.

use super::*;

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
        // A display frame draws without what still compiles; every other
        // frame, a capture's among them, waits for it (gpu/deferred.rs).
        // So does a renderer's first frame natively: the host shows it,
        // and every pipeline in it is cold, the sky's and the post passes'
        // too, so it would show their ground black (a cold NVIDIA driver,
        // for seconds). The web cannot wait (the browser would make them
        // at once): its Worker draws the first frame again until it is
        // whole.
        let first = cfg!(not(target_arch = "wasm32")) && self.frame_index == 0;
        let _policy = crate::deferred::FramePolicy::open(
            !scene.frame.display || self.capturing || self.uploads_whole || first,
        );
        // Where the call's time goes, with a clock (frame_stages.rs).
        let mut stages = Stages::start(self.clock);
        // What this frame waits for compiles (FrameStats::compile_wait_us).
        crate::deferred::take_waited_us();
        // A capture draws with the full variants (pipeline_features.rs),
        // the static lights' far field's where the scene draws it: what a
        // display frame draws with depends on what has compiled.
        self.frame_features = if self.capturing {
            use crate::pipeline_features::Features;
            Features::ALL | (super::setup::scene_features(scene) & Features::STATIC_FAR)
        } else {
            super::setup::scene_features(scene)
        };
        if let Some(t) = self.world.terrain.get_mut() {
            t.features = self.frame_features;
        }
        self.reach = super::readback::Reach::of(scene);
        // Match the surface's two-frame latency. Wait for the oldest frame,
        // not the entire queue: motion and readbacks share this device.
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(target, RenderTarget::Offscreen) && self.offscreen_submissions.len() == 2 {
            crate::traffic::poll(
                &self.device,
                wgpu::PollType::Wait {
                    submission_index: self.offscreen_submissions.front().cloned(),
                    timeout: Some(crate::capture::BACKLOG_TIMEOUT),
                },
            )
            .map_err(|e| RenderError::Readback(format!("waiting for an offscreen frame: {e}")))?;
            self.offscreen_submissions.pop_front();
        }
        stages.end("wait for frame N-2");
        self.sync(scene)?;
        stages.end("sync");
        // With its materials' (toon, the dissolve), as the material pass
        // left them (a capture's are all of them already).
        self.frame_features = self.frame_features | self.cache.material_features;
        if let Some(s) = self.world.scatter.get_mut() {
            s.features = self.frame_features;
        }
        // The 3D world's renderers this frame draws with, made by the first
        // that does (resources.rs OnDemand); the scattered copies' again
        // with the crowds' cards (world_prepare.rs).
        self.make_world(scene, false);
        // What the frame keeps for each of its passes from their plan
        // steps to their encode, and what they encode from (pass.rs).
        let mut frames = PassFrames::default();
        let mut plans = FramePlans::default();
        // The wind field's steps since the last tick drawn (wind_field.rs):
        // its grid is what every vegetation pass below reads it by, its
        // lean cap what their culling bounds widen by.
        self.world
            .wind_field
            .prepare(&self.device, &self.queue, scene);
        self.world.trample.prepare(&self.device, &self.queue, scene);
        self.refresh_foliage_bounds(scene);
        let (upscale, grid_offset) = self.prepare_frame_resources(scene);
        let (probe_frame, active_probes, probe_boxes) = self.prepare_frame_probes(scene);
        let (mut batches, sorted_instances) = self.build_batches(scene)?;
        let mut ahead = self.ahead_batches(scene, &batches, &sorted_instances.ahead);
        // The pairs of models decoded before anything draws them
        // (`SceneObjects::prepare`), as instances still hidden.
        if !scene.objects.prepare.is_empty() {
            let pairs: Vec<instances::Ahead> = scene
                .objects
                .prepare
                .iter()
                .filter(|(mesh, material)| {
                    mesh.index() < self.cache.meshes.len()
                        && material.index() < self.cache.materials.len()
                })
                .map(|&(mesh, material)| instances::Ahead {
                    mesh: mesh.index(),
                    material: material.index(),
                    cast: true,
                    mirrored: false,
                })
                .collect();
            ahead.extend(self.ahead_batches(scene, &[], &pairs));
        }
        let instances::Sorted {
            mut transforms,
            mut spheres,
            submission,
            mut sphere_box,
            serial: sorted_serial,
            changed: sorted_changed,
            still: sorted_still,
            objects_moved,
            content: sorted_content,
            ..
        } = sorted_instances;
        // The debris layer's instances are culled by everywhere their
        // pieces can reach: the GPU moves them (debris.rs).
        let debris_instances = crate::debris::instances(scene);
        if !debris_instances.is_empty() {
            for (s, &i) in submission.iter().enumerate().take(spheres.len()) {
                if let Some(&(p, _)) = debris_instances.get(&i) {
                    spheres[s] = scene.objects.debris[p].sphere();
                }
            }
            sphere_box = instances::fold_box(None, spheres.iter().copied());
            for b in &mut batches {
                let members = b.first as usize..(b.first + b.count) as usize;
                if submission[members.clone()]
                    .iter()
                    .any(|i| debris_instances.contains_key(i))
                {
                    b.reach = instances::fold_box(None, spheres[members].iter().copied());
                }
            }
        }
        let mut origins = submission.clone();
        // The main camera's cull record (cull_record.rs): what the sort
        // left out now, what each cull below decides as it does.
        if let Some(records) = self.visibility.records.as_mut() {
            let f = records.frame.get_or_insert_default();
            f.begin(
                self.frame_index,
                scene.objects.instances.len(),
                scene.objects.skinned_instances.len(),
                scene.sprite.sprites.len(),
            );
            self.instances
                .sorted
                .record_left_out(scene, &mut f.record.instances);
        }
        self.prepare_full_meshes(scene, &batches);
        stages.end("batches");
        // Crowds (crowd.rs): their members as the crowd cull's tables, the
        // one pass over them, before the skinned palette, which holds the
        // clips they play. Their previous transforms go with them under
        // TAA or motion blur.
        let crowd_motion = self.info.antialias.temporal() || scene.look.post.motion_blur.is_some();
        let mut crowd_frame = self.build_crowds(scene, crowd_motion)?;
        stages.end("crowd tables");
        let mut skinned = self.build_skinned(scene, &probe_boxes, &crowd_frame.played)?;
        let palette = &self.instances.palette;
        crowd_frame.place_clips(&|c| palette.baked_offset(c));
        stages.end("skinned");
        // The planar water this frame draws.
        let scene_water: &[Water] = &scene.world.water;

        let (aspect, temporal, camera, cam, view_frustum) = self.prepare_frame_camera(scene);
        let taa = self.info.antialias == Antialias::Taa;
        let smaa_t2x = self.info.antialias == Antialias::SmaaT2x;
        let (volumetric_fog, frame_fog, underwater, reflections) = self.prepare_frame_reflections(
            scene,
            scene_water,
            &camera,
            &cam,
            &view_frustum,
            &aspect,
        );
        let reflection = reflections.first();
        // The crowds' batches, and the path that draws them.
        let batches::CrowdPlan {
            frame: crowd_frame,
            batches: crowd_batches,
            previous: crowd_previous,
            gpu: crowd_gpu,
            cpu_cards,
        } = self.plan_crowds(
            scene,
            crowd_frame,
            crowd_motion,
            probe_frame.due.is_empty(),
            &mut skinned,
        )?;
        skinned.baked_moved |= crowd_frame.moved;
        let (view_plans, mut view_skinned) =
            self.prepare_view_culling(scene, &camera, &mut skinned);
        let (probe_faces, mut probe_skinned) =
            self.prepare_probe_culling(&probe_frame, &active_probes, &camera, &mut skinned);
        let (mut skinned_visible, mut skinned_culled) = Self::cull_skinned(
            &mut skinned,
            &view_frustum,
            scene.frame.camera_layers,
            self.visibility
                .records
                .as_mut()
                .and_then(|r| r.frame.as_mut())
                .map(|f| &mut f.record.skinned[..]),
        );
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
        let skinned_motion = if taa_on || scene.look.post.motion_blur.is_some() {
            skin::append_previous(
                &scene.objects.joint_palettes,
                &scene.objects.previous_joint_palettes,
                &scene.objects.skinned_instances,
                &skinned.previous_slots,
                &mut joint_matrices,
            )
        } else {
            None
        };
        // The crowd members' previous transforms follow every other slot:
        // uploaded from their table (`upload_frame`), not copied here.
        debug_assert!(crowd_previous.is_none_or(|at| at as usize == joint_matrices.len()));

        stages.end("camera, crowd plan, skinned culling");
        // Effects this frame.
        let post_fx = &scene.look.post;
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
            objects_moved,
        });
        // Transmission reads the stored opaque main depth at its split;
        // it does not require a separate geometry prepass.
        plans.ssr = SsrPass::bind(self, &mut frames.ssr, effects.ssr);
        // What the frame will draw with beyond its batches starts compiling
        // on worker threads before the batches' pipelines (deferred.rs):
        // the terrains', the frame's longest compiles, the sky's and the
        // post chain's.
        if let Some(t) = self.world.terrain.get() {
            t.start_passes(scene, effects.prepass);
        }
        {
            use crate::deferred::Start;
            match scene.lighting.environment.background {
                Background::Sky
                    if self.sky.ground.maps.is_some() && ground_heights::wanted(scene) =>
                {
                    self.sky.ground.pipeline.start()
                }
                Background::Sky => self.sky.sky_pipeline.start(),
                Background::Procedural { .. } if scene.atmosphere.clouds.is_some() => {
                    if effects.prepass {
                        self.sky.clouds_low.march_depth.start();
                    } else {
                        self.sky.clouds_low.march.start();
                    }
                    self.sky.clouds_low.resolve.start();
                    self.sky.clouds_low.sky.start();
                }
                Background::Procedural { .. } => self.sky.sky_look_pipeline.start(),
                Background::Color(_) => {}
            }
            if scene.frame.plate.is_some() {
                self.sprite.plate_pipeline.start();
            }
        }
        self.post.start(self.history.taa_history.is_some());
        if let Some(w) = self.world.water.get_mut() {
            w.start(scene_water, self.frame_features);
        }
        let (face_fluid, face_depth, object_space) =
            self.prepare_frame_pipelines(PrepareFramePipelinesInputs {
                scene,
                ao: effects.ao_target || effects.contact,
                batches: &mut batches,
                ahead: &mut ahead,
                skinned_batches: &mut skinned_batches,
                skinned_visible: &mut skinned_visible,
                view_skinned: &mut view_skinned,
                probe_skinned: &mut probe_skinned,
                particle_frame: &particle_frame,
                view_plans: &view_plans,
                probe_frame: &probe_frame,
                fluids_3d: effects.fluids_3d,
                scene_water,
                prepass: effects.prepass,
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
        self.particles.gpu.ensure_pipelines(
            &self.device,
            &particle_frame,
            self.samples,
            scene.shaders(),
        );
        let arrays = &self.cache.texture_arrays;
        let array_layers = |a: TextureArrayHandle| {
            scene
                .contains_texture_array(a)
                .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                .flatten()
                .map(|g| g.info.layers)
        };
        // What the overlay is built from: kept as last frame's when its key
        // and target are (its tables are not uploaded again either).
        let kept_key = scene.ui.overlay.as_ref().and_then(|o| {
            Some(overlay::KeptOverlay {
                key: o.key?,
                width: self.out_width,
                height: self.out_height,
                scale: overlay::OverlayRenderer::scale(self.out_height, o).to_bits(),
                revisions: og_core::hash::hash_words(
                    scene
                        .texture_revisions()
                        .iter()
                        .chain(scene.texture_array_revisions())
                        .copied(),
                ),
                textures: self.cache.textures.len(),
                vectors: scene.ui.ui_vectors.len(),
            })
        });
        let kept_frame = match self.overlay.kept.take() {
            Some((key, frame)) if Some(key) == kept_key => Some(frame),
            _ => None,
        };
        let overlay_fresh = kept_frame.is_none();
        let overlay_frame = match (&scene.ui.overlay, kept_frame) {
            (Some(_), Some(frame)) => Some(frame),
            (Some(o), None) => Some(overlay::build(
                o,
                self.cache.textures.len(),
                array_layers,
                |v| scene.ui.ui_vectors.get(v.index()),
                |t| scene.target_texture(t),
                |t| scene.texture_orientation(t),
                overlay::OverlayTarget {
                    scale: overlay::OverlayRenderer::scale(self.out_height, o),
                    width: self.out_width,
                    height: self.out_height,
                },
                {
                    let atlas = scene.ui.glyphs.lock().unwrap_or_else(|e| e.into_inner());
                    og_draw::glyphs::GlyphTier::ALL.map(|t| atlas.pages(t).1.max(1) as f32)
                },
                &mut self.overlay.mass_eases,
            )?),
            (None, _) => None,
        };
        let glass_on = self.prepare_frame_overlay(scene, &overlay_frame);
        // The frame goes through the glass source when glass reads it or a
        // colour filter recolours it on its way to the output.
        let color_filter = scene
            .look
            .color_filter
            .filter(|_| self.glass_targets.is_some());
        let routed = glass_on || color_filter.is_some();
        let line_frame = self.prepare_frame_lines(scene, &cam);
        // The strips and sprites of the camera, the views and the probe faces,
        // appended in that order to the frame's strip and sprite buffers
        // (sprite_arenas.rs).
        let arenas = self.build_sprite_arenas(SpriteArenaInputs {
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
        plans.metaballs = MetaballsPass::bind(self, &mut frames.metaballs, arenas.fluids);
        // Order-independent transparency's frame mode (oit_frame.rs).
        let oit = self.plan_oit_frame(PlanOitFrameInputs {
            scene,
            batches: &batches,
            skinned_batches: &skinned_batches,
            sprite_frame: &arenas.sprite_frame,
            ribbon_frame: &arenas.ribbon_frame,
            particle_frame: &particle_frame,
        });
        // The sprites' groups, tiles and sprite meshes (sprite_groups.rs).
        self.prepare_sprite_groups(PrepareSpriteGroupsInputs {
            scene,
            sprite_frame: &arenas.sprite_frame,
            view_sprites: &arenas.view_sprites,
            view_glow_runs: &arenas.view_glow_runs,
            face_sprites: &arenas.face_sprites,
            view_boards: &arenas.view_boards,
            face_boards: &arenas.face_boards,
            gpu_sprites: &arenas.gpu_sprites,
            light_slots: &arenas.light_slots,
            lit_range: arenas.lit_range,
            particle_frame: &particle_frame,
        });
        if let Some(t) = scene.effects.decal_atlas
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
        let glow = post_fx.glow.is_some() && !arenas.sprite_frame.glow_runs.is_empty();

        stages.end("effects, pipelines, sprites");
        // GPU-driven culling (cull_gpu.rs): each pass's list is indirect
        // draws the compute fills; the CPU sorts the transparent instances
        // and culls the skinned ones as before.
        // Per-object motion for motion blur and TAA's motion vectors: the
        // previous transforms at the same slots as the instances (GPU
        // culling compacts its camera pass's beside its own). A frame where
        // no instance moved draws its motion from the instances themselves
        // (pass_prepass.rs), so their previous records are neither built
        // nor uploaded; foliage's carry its previous sway as well, so a
        // frame with foliage builds them whatever moved.
        let previous_valid = (effects.instances_moved || batches.iter().any(|b| b.foliage))
            && !scene.objects.previous_transforms.is_empty()
            && scene.objects.previous_transforms.len() == scene.objects.instances.len();
        let blur_motion = effects.motion_blur.is_some() && previous_valid;
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
        let shadow_inputs = self.prepare_shadow_inputs(PrepareShadowInputsInputs {
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
        let cull = self.plan_culling(PlanCullingInputs {
            scene,
            batches: &batches,
            view_plans: &view_plans,
            reflection,
            shelter_redraw: shadow_inputs.shelter_redraw,
            ground_redraw: shadow_inputs.ground_redraw,
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
        let in_meshlet =
            |b: &Batch| b.count > 0 && cull.meshlet_firsts.binary_search(&b.first).is_ok();
        let on_layers = |mask: u32| -> Vec<u32> {
            cull.meshlet_all
                .iter()
                .copied()
                .filter(|&b| batches[b as usize].layers & mask != 0)
                .collect()
        };
        // The main camera's lists (camera_lists.rs).
        let mut camera_lists = self.plan_camera_lists(PlanCameraListsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            batches: &batches,
            gpu_cull: cull.gpu_cull,
            cull_n: cull.cull_n,
            meshlet_items: cull.meshlet_items,
            meshlet_max: cull.meshlet_max,
            meshlet_taken: cull.meshlet_taken,
            spheres: &spheres,
            transforms: &mut transforms,
            origins: &mut origins,
            submission: &submission,
            sprite_frame: &arenas.sprite_frame,
            skinned_visible: &skinned_visible,
            skinned_culled,
            oit_frame: oit.oit_frame,
            oit_candidates: oit.oit_candidates,
        });
        // Thin walls need the scene copy, but only drawn volumes prepare
        // exit targets and pipelines.
        let transmission =
            self.prepare_transmission(&batches, &camera_lists.blended_list, &skinned_visible);
        // The frame's culling as the passes after the main camera extend
        // it, and what they cull against (pass.rs).
        let mut frame_cull = FrameCulling {
            vis: camera_lists.vis,
            transforms,
            origins,
            cull_inside: camera_lists.cull_inside,
        };
        let seen = CullFrame {
            batches: &batches,
            spheres: &spheres,
            submission: &submission,
            gpu_cull: cull.gpu_cull,
            cull_n: cull.cull_n,
            in_meshlet: &in_meshlet,
            on_layers: &on_layers,
        };
        // The planar reflections', the probe faces' and the views' plan
        // steps: their lists, in that order (pass_reflection.rs,
        // pass_probes.rs, pass_view.rs).
        frames.reflection = ReflectionPass::plan(
            self,
            ReflectionIn {
                cull: &mut frame_cull,
                seen,
                reflections: &reflections,
                mask: camera_lists.mask,
            },
        );
        frames.probes = ProbesPass::plan(
            self,
            ProbesIn {
                cull: &mut frame_cull,
                seen,
                probe_faces: &probe_faces,
                active_probes: &active_probes,
                probe_frame: &probe_frame,
                face_depth,
                meshlet_all: &cull.meshlet_all,
                face_boards: &arenas.face_boards,
            },
        );
        frames.view = ViewPass::plan(
            self,
            ViewIn {
                cull: &mut frame_cull,
                seen,
                view_plans: &view_plans,
                view_boards: &arenas.view_boards,
                particle_frame: &particle_frame,
            },
        );
        stages.end("culling plans");
        // Terrains, scattered copies and grass (world_prepare.rs).
        let mut world_frame = self.prepare_world(PrepareWorldInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            prepass: effects.prepass,
            crowd_frame: &crowd_frame,
            cpu_cards: &cpu_cards,
        })?;
        // The directional light's cascades.
        let light = &scene.lighting.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let shadow_size = self.info.shadow_map_size;
        // Every kind of caster: grass and scattered copies on a terrain
        // that casts nothing still need the cascades (their shadows, and
        // the grass canopy's own shading, read them).
        let has_casters = batches.iter().chain(&skinned_batches).any(|b| b.cast)
            || world_frame.terrain_casters
            || world_frame.grass_frame.casts()
            || world_frame.scatter_batches.casts();
        // The local lights (lights_stage.rs).
        let lights_frame = self.prepare_lights(PrepareLightsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            contact: effects.contact,
            reflections: &reflections,
        });
        // The volumetric fog's plan step (pass_volumetric.rs).
        frames.volumetric = VolumetricPass::plan(self, (volumetric_fog.as_ref(), scene));
        plans.volumetric = VolumetricPass::bind(self, &mut frames.volumetric, ());
        // The shadow passes' plan steps: the cascades, the shelter map and
        // the local faces, in that order (pass_shadows.rs).
        let casters = Casters::new(
            seen,
            &cull.meshlet_casters,
            &shadow_inputs.shadow_split,
            &skinned_batches,
            &world_frame.scatter_batches,
            &world_frame.grass_frame,
            world_frame.terrain_casters,
            &world_frame.terrain_draws,
        );
        frames.cascade = CascadePass::plan(
            self,
            CascadeIn {
                cull: &mut frame_cull,
                casters: &casters,
                scene,
                camera: &camera,
                cam: &cam,
                light,
                has_casters,
                sphere_box,
                to_light,
                shadow_size,
                aspect,
            },
        );
        frames.shelter =
            ShelterPass::plan(self, (&mut frame_cull, &casters, &shadow_inputs.shelter));
        frames.ground_heights =
            GroundHeightsPass::plan(self, (&mut frame_cull, &casters, &shadow_inputs.ground));
        frames.local_face = LocalFacePass::plan(
            self,
            (
                &mut frame_cull,
                &casters,
                scene,
                &lights_frame.local_shadows,
            ),
        );
        self.visibility.cull_inside = std::mem::take(&mut frame_cull.cull_inside);
        // The shadow passes' upload steps: the cache textures their plans
        // copy through.
        CascadePass::upload(self, &mut frames.cascade, shadow_size);
        LocalFacePass::upload(self, &mut frames.local_face, ());
        stages.end("world, lights, shadows");
        // The GPU culling's encode and the instance records (instances_stage.rs).
        let mut instance_uploads = self.upload_instances(UploadInstancesInputs {
            scene,
            cam: &cam,
            batches: &batches,
            transforms: frame_cull.transforms,
            spheres,
            origins: &frame_cull.origins,
            debris_instances: &debris_instances,
            sorted,
            vis: &frame_cull.vis,
            gpu_cull: cull.gpu_cull,
            max_passes: cull.max_passes,
            auxiliary_instances: cull.auxiliary_instances,
            previous_valid,
            object_motion: cull.object_motion,
            prepass: effects.prepass,
            skinned_gpu: cull.skinned_gpu,
            skinned_batches: &skinned_batches,
            skinned_plain,
            skinned_data: &skinned_data,
            skinned: &skinned,
            blended_list: &mut camera_lists.blended_list,
            content: sorted_content,
        });
        stages.end("instance records");
        // The CPU-culled lists of the passes after the camera, moved where
        // the records put their instances (their upload steps).
        let moved = ListsMoved {
            gpu_cull: cull.gpu_cull,
            sorted,
        };
        ReflectionPass::upload(self, &mut frames.reflection, moved);
        ProbesPass::upload(self, &mut frames.probes, moved);
        ViewPass::upload(self, &mut frames.view, moved);
        // The skinned, sprite, particle, overlay and 2D light uploads (uploads_stage.rs).
        let uploads = self.upload_frame(UploadFrameInputs {
            scene,
            target,
            crowd_gpu,
            skinned_data: &skinned_data,
            joint_matrices: &joint_matrices,
            crowd_frame: &crowd_frame,
            scatter_batches: &mut world_frame.scatter_batches,
            ribbon_frame: &arenas.ribbon_frame,
            sprite_frame: &arenas.sprite_frame,
            overlay_frame: &overlay_frame,
            overlay_fresh,
            light_slots: &arenas.light_slots,
        });
        stages.end("skinned and sprite uploads");
        // The particles' and the 2D lights' upload steps
        // (pass_particles.rs, pass_light_2d.rs).
        ParticlesPass::upload(
            self,
            &mut frames.particles,
            ParticlesPlan {
                frame: &particle_frame,
                sprites: &arenas.gpu_sprites,
            },
        );
        frames.light_2d = Light2dPass::plan(self, (scene, &arenas.light_slots));
        Light2dPass::upload(
            self,
            &mut frames.light_2d,
            (scene, &cam, &arenas.light_slots),
        );
        // The frame uniform, its shadow slots and its copies (frame_uniform.rs).
        let frame_uniforms = self.write_frame_uniforms(WriteFrameUniformsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            light,
            to_light,
            shadow_size,
            taa,
            frame_fog: &frame_fog,
            underwater,
            scene_water,
            lights_gpu: &lights_frame.lights_gpu,
            cascades: &frames.cascade.cascades,
            cascade_anchor: frames.cascade.anchor,
            local_shadows: &lights_frame.local_shadows,
            shelter: &shadow_inputs.shelter,
            ground: &shadow_inputs.ground,
            probe_frame: &probe_frame,
            active_probes: &active_probes,
            probe_faces: &probe_faces,
            reflections: &reflections,
            ssr: effects.ssr.as_ref(),
        });
        // The sun on screen, the flare's and the post chain's uniforms (sun_and_post.rs).
        let sun = self.write_sun_and_post(WriteSunAndPostInputs {
            scene,
            camera: &camera,
            cam: &cam,
            aspect,
            light,
            to_light,
            uniform: &frame_uniforms.uniform,
            post_fx,
            god_rays: effects.god_rays,
            lens_flare: effects.lens_flare,
            flare_sources_on: effects.flare_sources_on,
            glow,
            sprite_frame: &arenas.sprite_frame,
        });
        // The effects frame_uniforms.uniform and the decals (fx_uniform.rs).
        let fx_frame = self.write_fx_uniforms(WriteFxUniformsInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            prev_view_proj: world_frame.prev_view_proj,
            motion_read: effects.motion_read,
            skinned_motion,
            joints_drawn,
            ao_target: effects.ao_target,
            ao: effects.ao,
            gi: effects.gi.is_some(),
            bent: effects.bent,
            reflection,
            reflections: &reflections,
        });
        // The views' uniforms (view_uniforms.rs).
        let WriteViewUniformsOutputs { viewport_format } =
            self.write_view_uniforms(WriteViewUniformsInputs {
                scene,
                target,
                view_plans: &view_plans,
                uniform: &frame_uniforms.uniform,
                fx_uniform: &fx_frame.fx_uniform,
                lights_on: !lights_frame.lights_gpu.is_empty(),
                light_slots: &arenas.light_slots,
                fluids: arenas.fluids,
                particle_soft: particle_frame.soft,
                post_fx,
                view_glow_runs: &arenas.view_glow_runs,
            });
        // The post chain's uniforms, the GI's history, SSR and TAA (post_uniforms.rs).
        let FramePostHistory { gi_copy, taa_blend } =
            self.write_frame_post_uniforms(WriteFramePostUniformsInputs {
                cam: &cam,
                camera: &camera,
                post_fx,
                scene,
                untonemapped: arenas.sprite_frame.untonemapped
                    || self.plate_group(scene).is_some()
                    || self.authored_environment(scene),
                ao: effects.ao,
                gi: effects.gi,
                dof: effects.dof,
                motion_blur: effects.motion_blur,
                god_rays: effects.god_rays,
                sun_ndc: sun.sun_ndc,
                god_visible: sun.god_visible,
                sun_fade: sun.sun_fade,
                god_sun: sun.god_sun,
                depth_bands: effects.depth_bands,
                screen_blur: effects.screen_blur,
                contact: lights_frame.contact_rows,
                ssr: effects.ssr.as_ref(),
                taa,
                temporal,
                prepass: effects.prepass,
            });
        // The clouds' history step: which texel of each two by two this frame marches (clouds.rs).
        if CloudsPass::runs(scene) {
            let phase = self.phase();
            self.sky.clouds_low.temporal.prepare(
                &self.device,
                &self.queue,
                (self.width, self.height),
                phase,
                cam.jittered_view_proj,
                scene.frame.camera_cut,
            );
        }
        let slot = (self.frame_index % 2) as usize;
        let gtao = match effects.ao.map(|a| a.method) {
            Some(AoMethod::Gtao) => true,
            Some(AoMethod::Ssao | AoMethod::Screen) => false,
            _ => true,
        };
        self.shadows.shelter_key = shadow_inputs.shelter.as_ref().map(|s| s.2);
        if let Some(m) = &mut self.sky.ground.maps {
            m.key = shadow_inputs.ground.as_ref().map(|s| s.2);
        }
        let lens = post_fx.lens_distortion.filter(|l| l.amount != 0.0);
        // The bind steps of the passes whose plans hold nothing of the
        // frame's: the pass list reads them.
        plans.live_sky = LiveSkyPass::bind(self, &mut frames.live_sky, ());
        plans.clouds = CloudsPass::bind(self, &mut frames.clouds, scene);
        plans.gi_probes = GiProbesPass::bind(self, &mut frames.gi_probes, ());
        plans.light_2d = Light2dPass::bind(self, &mut frames.light_2d, scene);
        let Light2dFrame {
            size: lightmap_size,
            lightdir: lightdir_on,
            lights: lights_2d,
            occluders: occluders_2d,
        } = frames.light_2d;
        // The frame's passes, compiled into the render graph (cached while
        // the pass list stays the same; pass_list.rs). It selects passes by
        // their plans, so every plan it reads is made above.
        let passes = self.plan_pass_list(PassListInputs {
            scene,
            camera: &camera,
            effects: &effects,
            crowd_cull: crowd_gpu,
            cull_counts: cull.gpu_cull,
            live_sky: plans.live_sky.is_some(),
            gi_probes: plans.gi_probes.is_some(),
            particles: particle_frame.any_gpu,
            light_2d: plans.light_2d.is_some(),
            metaballs: plans.metaballs.is_some(),
            froxels: plans.volumetric.is_some(),
            cascades: frames.cascade.cascades.len() as u32,
            shelter_redraw: shadow_inputs.shelter_redraw,
            ground_redraw: shadow_inputs.ground_redraw,
            local_shadows: &lights_frame.local_shadows,
            probes: !probe_frame.due.is_empty(),
            reflections: reflections.len() as u8,
            views: view_plans.len() as u32,
            glow,
            clouds: plans.clouds.is_some(),
            taa,
            smaa_t2x,
            // What the frame draws over its finished image (after_graph.rs).
            after: schedule::AfterGraph {
                viewport_views: view_plans
                    .iter()
                    .any(|p| matches!(p.dest, views::Dest::Viewport(_))),
                ui_layers: overlay_frame
                    .as_ref()
                    .is_some_and(|of| !of.layers.is_empty()),
                overlay: (routed && self.glass_targets.is_some())
                    || line_frame.overlay.1 > 0
                    || !scene.ui.paint.is_empty()
                    || overlay_frame
                        .as_ref()
                        .is_some_and(|of| !of.main_runs().is_empty()),
                hdr_output: self.display.is_some() && matches!(target, RenderTarget::Surface),
            },
        });
        // The stylise passes' table, key and frame_uniforms.uniform (stylise_frame.rs).
        let PrepareFrameStyliseOutputs { stylise_key } =
            self.prepare_frame_stylise(PrepareFrameStyliseInputs {
                scene,
                camera: &camera,
                cam: &cam,
                upscale,
                grid_offset,
                lens,
                prepass: effects.prepass,
            });
        // The colour adjustments' table and the transmission's scene copy
        // (post_resources.rs).
        let adjusted = self.prepare_colour_lut(scene);
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
            volumetric_fog: volumetric_fog.is_some(),
            distortion: particle_frame.distortion,
            transmission,
            fluids: arenas.fluids,
            reflections: &reflections,
            object_space,
            oit_mode: camera_lists.oit_mode,
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
            overlay_fresh,
            ui_uniform: uploads.ui_uniform,
            glass_on,
        });
        // Fork s70: a covered surface's frame ends prepared: its passes would
        // only be decoded and validated by the browser's GPU process for a
        // canvas nobody sees.
        if matches!(target, RenderTarget::Surface)
            && COVERED.load(std::sync::atomic::Ordering::Relaxed)
        {
            if let Some(gc) = self.visibility.gpu_cull.as_mut().filter(|_| cull.gpu_cull) {
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
                        // In its other encoding: sRGB of a plain surface,
                        // plain of an sRGB one.
                        let twin = config.view_formats.first().map(|&format| {
                            t.texture.create_view(&wgpu::TextureViewDescriptor {
                                format: Some(format),
                                ..Default::default()
                            })
                        });
                        Some((t, view, twin))
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
        self.update_fx_groups(
            scene,
            effects.ao_target || effects.contact,
            reflection.is_some(),
        );
        // The water's and the 3D arenas.fluids' draws (water_fluids.rs).
        let water_fluids = self.prepare_water_and_fluids(PrepareWaterAndFluidsInputs {
            scene,
            cam: &cam,
            scene_water,
            reflections: &reflections,
            ao_target: effects.ao_target,
            ao: effects.ao,
            probe_frame: &probe_frame,
            probe_faces: &probe_faces,
            fluids_3d: effects.fluids_3d,
            face_fluid,
        });
        // The frame's encoder, and the compute nodes the graph begins with
        // (pre_graph_stage.rs, execute.rs). Each graph pass owns the GPU
        // queries of the render and compute passes it opened (`timing.rs`).
        stages.end("uniforms and binds");
        // The clouds' noise, the first frame with clouds: its pipeline
        // has compiled beside all of the above (clouds.rs `CloudNoise`).
        self.sky.cloud_noise.submit(&self.device, &self.queue);
        let mut encoder = self.open_frame_encoder();
        let mut pass_queries: Vec<(String, std::ops::Range<u32>)> = Vec::new();
        let pre_graph = PreGraphInputs {
            scene,
            camera: &camera,
            cam: &cam,
            view_frustum: &view_frustum,
            crowd_frame: &crowd_frame,
            crowd_previous,
            vis: &frame_cull.vis,
            gpu_cull: cull.gpu_cull,
            skinned: &skinned,
            skinned_motion,
            joints_drawn,
            terrain_draws: &world_frame.terrain_draws,
            grass_frame: &world_frame.grass_frame,
        };
        // Every pass waiting for the one before it (`set_gpu_serial`): the
        // debris' and GPU culling's command buffers, which the frame's
        // follow, run first, in a submit of their own.
        if super::settings::gpu_serial(self.frame_index) {
            self.queue.submit(
                instance_uploads
                    .debris_commands
                    .take()
                    .into_iter()
                    .chain(std::mem::take(&mut instance_uploads.cull_commands)),
            );
        }
        self.execute_pre_graph(&mut encoder, &pre_graph, &mut pass_queries);
        stages.end("compute before the graph");
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
        // A display output composes in the float offscreen target and its
        // pass writes the surface; a caller's view takes the composition.
        let composed = self.display.is_some() && matches!(target, RenderTarget::Surface);
        let output = if composed {
            &self.persistent.offscreen_view
        } else {
            destination
        };
        // Its texture, where it is the renderer's own.
        let output_texture = (composed || matches!(target, RenderTarget::Offscreen))
            .then_some(&self.persistent.offscreen);
        // Where the overlay blends: the output when it is sRGB, else its sRGB
        // view where the backend has one (linear light either way), else the
        // output itself with the shader encoding (a caller's non-sRGB view).
        let overlay_view = match target {
            _ if self.output_format.is_srgb() || self.display.is_some() => None,
            RenderTarget::Offscreen => self.persistent.offscreen_linear.as_ref(),
            RenderTarget::Surface => surface_frame.as_ref().and_then(|f| f.2.as_ref()),
            RenderTarget::View(_) => None,
        };
        // Where the paint pass blends, in encoded values as CSS does: the
        // output when it holds them, else its plain view where the backend
        // has one, else (`None`) a layer of the pass's own (ui/paint.rs).
        let paint_view = match target {
            _ if ui::paint::encoded_format(self.output_format) == self.output_format => {
                Some(output)
            }
            _ if self.display.is_some() => None,
            RenderTarget::Offscreen => self.persistent.offscreen_encoded.as_ref(),
            RenderTarget::Surface => surface_frame.as_ref().and_then(|f| f.2.as_ref()),
            RenderTarget::View(_) => None,
        };

        // The scattered copies' counts landed since the last frame.
        if let Some(s) = self.world.scatter.get_mut() {
            s.poll_counts();
        }
        // The frame's statistics so far (frame_stats.rs).
        let FrameStatsOutputs { mut stats } = self.frame_stats(FrameStatsInputs {
            scene,
            batches: &batches,
            instance_upload_bytes: instance_uploads.instance_upload_bytes,
            culled: camera_lists.culled,
            main_list: &camera_lists.main_list,
            blended_list: &camera_lists.blended_list,
            local_shadows: &lights_frame.local_shadows,
            shadowed_dropped: lights_frame.shadowed_dropped,
            sprite_frame: &arenas.sprite_frame,
            particle_frame: &particle_frame,
            ribbon_frame: &arenas.ribbon_frame,
            line_frame: &line_frame,
            decals: fx_frame.decals,
            decals_dropped: fx_frame.decals_dropped,
            lights_2d,
            occluders_2d,
            joints_drawn,
            taa_blend,
            oit_mode: camera_lists.oit_mode,
            gpu_counts: instance_uploads.gpu_counts,
            overlay_frame: &overlay_frame,
            skinned_visible: &skinned_visible,
            skinned_gpu: cull.skinned_gpu,
        });
        if let Some(w) = self
            .world
            .water
            .get()
            .filter(|_| !water_fluids.water_draws.is_empty())
        {
            stats.foam_steps = w.foam_steps;
        }
        // The frame's bind groups (frame_binds.rs).
        let binds = self.bind_frame(BindFrameInputs {
            scene,
            batches: &batches,
            oit_mode: camera_lists.oit_mode,
            probe_frame: &probe_frame,
            skinned_visible: &skinned_visible,
            ao_target: effects.ao_target,
            contact: effects.contact,
            transmission_view,
            particle_frame: &particle_frame,
        });
        // The groups that select among them: the probes' (per active probe,
        // and with OIT's arena), and the effect groups' lean twins.
        let probe_binding = (!binds.probe_groups.is_empty()).then_some(probe_select::ProbeGroups {
            boxes: &probe_boxes,
            groups: &binds.probe_groups,
            frame: binds.frame_bind_group,
        });
        let oit_probe_binding = binds
            .oit_frame_group
            .as_ref()
            .filter(|_| !binds.oit_probe_groups.is_empty())
            .map(|frame| probe_select::ProbeGroups {
                boxes: &probe_boxes,
                groups: &binds.oit_probe_groups,
                frame,
            });
        // The bind steps of the passes the frame's culling lists for:
        // their groups and their plans (pass_reflection.rs,
        // pass_probes.rs, pass_view.rs).
        // The skinned batches' pipelines by pass (skinned_pipelines.rs).
        let SkinnedPipelines {
            lit: lit_skinned,
            lit_standard: lit_skinned_standard,
            shadow: shadow_skinned,
            prepass: prepass_skinned,
        } = self.skinned_pipelines(&skinned_visible, &skinned_batches, effects.prepass);
        let culled_gpu = self.visibility.gpu_cull.as_ref().filter(|_| cull.gpu_cull);
        let crowd_cull = self.world.crowd_gpu.as_ref().filter(|_| crowd_gpu);
        // The effect groups of the standard and the physical pipelines, in
        // the passes that draw physical materials with theirs.
        let physical_fx = binds
            .physical_fx_group
            .as_ref()
            .map(|p| (binds.fx_group, p));
        plans.reflection = ReflectionPass::bind(
            self,
            &mut frames.reflection,
            ReflectionBindIn {
                scene,
                reflections: &reflections,
                batches: &batches,
                skinned_visible: &skinned_visible,
                lit_skinned_standard: &lit_skinned_standard,
                terrain_draws: &world_frame.terrain_draws,
                scatter_batches: &world_frame.scatter_batches,
                grass_frame: &world_frame.grass_frame,
            },
        );
        plans.probes = ProbesPass::bind(
            self,
            &mut frames.probes,
            ProbesBindIn {
                scene,
                batches: &batches,
                probe_frame: &probe_frame,
                face_depth,
                face_water: &water_fluids.face_water,
                face_fluids: &water_fluids.face_fluids,
                probe_faces: &probe_faces,
                active_probes: &active_probes,
                face_groups: &binds.face_groups,
                fx_group: binds.fx_group,
                glow_fx_group: binds.glow_fx_group,
                probe_skinned: &probe_skinned,
                terrain_draws: &world_frame.terrain_draws,
                scatter_batches: &world_frame.scatter_batches,
                face_boards: &arenas.face_boards,
                face_sprites: &arenas.face_sprites,
                particle_frame: &particle_frame,
                line_frame: &line_frame,
            },
        );
        plans.view = ViewPass::bind(
            self,
            &mut frames.view,
            ViewBindIn {
                scene,
                batches: &batches,
                view_plans: &view_plans,
                lights_2d,
                view_skinned: &view_skinned,
                terrain_draws: &world_frame.terrain_draws,
                view_boards: &arenas.view_boards,
                particle_frame: &particle_frame,
                view_ribbons: &arenas.view_ribbons,
                line_frame: &line_frame,
                view_sprites: &arenas.view_sprites,
                view_glow_runs: &arenas.view_glow_runs,
                post_fx,
            },
        );
        // Each effect group's lean twin (`fx_lean_group`): the camera's,
        // the planar reflections' and the views'.
        let fx_leans: Vec<(&wgpu::BindGroup, &wgpu::BindGroup)> =
            std::iter::once((binds.fx_group, binds.fx_lean))
                .chain(plans.reflection.iter().flat_map(|p| {
                    p.reflection_fx_group
                        .iter()
                        .zip(p.reflection_fx_lean.iter().map(Option::as_ref))
                }))
                .chain(
                    plans
                        .view
                        .iter()
                        .flat_map(|p| p.view_groups.iter().map(|g| (&g.1, g.2.as_ref()))),
                )
                .filter_map(|(fx, lean)| Some((fx, lean?)))
                .collect();
        let palette_group = self.instances.palette.bind_group.as_ref();
        // What every pass below draws with (draw_ctx.rs).
        let ctx = DrawCtx {
            r: self,
            scene,
            batches: &batches,
            culled_gpu,
            crowd_cull,
            fx_leans: &fx_leans,
            palette_group,
            skinned_gpu: cull.skinned_gpu,
            sprite_frame: &arenas.sprite_frame,
            blended_list: &camera_lists.blended_list,
            main_list: &camera_lists.main_list,
            skinned_visible: &skinned_visible,
            lit_skinned: &lit_skinned,
            physical_fx,
            fx_group: binds.fx_group,
            sprite_fx_group: binds.sprite_fx_group,
            glow_fx_group: binds.glow_fx_group,
            terrain_indices: self.world.terrain.get().and_then(|t| t.index_buffer()),
            terrain_draws: &world_frame.terrain_draws,
            scatter_batches: &world_frame.scatter_batches,
            grass_frame: &world_frame.grass_frame,
            shadow_skinned: &shadow_skinned,
            skinned_batches: &skinned_batches,
            terrain_casters: world_frame.terrain_casters,
        };
        let camera_cards = ctx.card_source(
            frame_cull
                .vis
                .cull_passes
                .iter()
                .position(|p| p.kind == cull_gpu::PassKind::Main)
                .map(|g| g as u32),
        );
        let terrain_indices = ctx.terrain_indices;

        stages.end("draw lists and binds");
        // Per-pass CPU encode time, with a clock (`Renderer::set_clock`),
        // and GPU time where the device has timestamps (`timing.rs`).
        let mut pass_ms: Vec<PassTiming> = Vec::new();
        // The bind steps of the other passes, in frame order.
        plans.cascade = CascadePass::bind(self, &mut frames.cascade, ());
        plans.shelter = ShelterPass::bind(self, &mut frames.shelter, &shadow_inputs.shelter);
        plans.ground_heights =
            GroundHeightsPass::bind(self, &mut frames.ground_heights, &shadow_inputs.ground);
        plans.local_face = LocalFacePass::bind(self, &mut frames.local_face, ());
        plans.particles = ParticlesPass::bind(
            self,
            &mut frames.particles,
            ParticlesPlan {
                frame: &particle_frame,
                sprites: &arenas.gpu_sprites,
            },
        );
        plans.fluid_3d = Fluid3dPass::bind(
            self,
            &mut frames.fluid_3d,
            (effects.fluids_3d, water_fluids.fluid_frame.as_ref()),
        );
        plans.prepass = PrepassPass::bind(
            self,
            &mut frames.prepass,
            effects.prepass.then_some(PrepassPlan {
                water_draws: &water_fluids.water_draws,
                object_space,
                frame_bind_group: binds.frame_bind_group,
                fx_group: binds.fx_group,
                main_list: &camera_lists.main_list,
                culled_gpu,
                object_motion: cull.object_motion,
                batches: &batches,
                skinned_visible: &skinned_visible,
                prepass_skinned: &prepass_skinned,
                terrain_indices,
                terrain_draws: &world_frame.terrain_draws,
                view_frustum: &view_frustum,
                scatter_batches: &world_frame.scatter_batches,
                cards: camera_cards,
                grass_frame: &world_frame.grass_frame,
            }),
        );
        plans.main = MainPass::bind(
            self,
            &mut frames.main,
            MainPlan {
                scene,
                water_draws: &water_fluids.water_draws,
                fluid_frame: &water_fluids.fluid_frame,
                transmission_view,
                taa: temporal,
                frame_bind_group: binds.frame_bind_group,
                fx_group: binds.fx_group,
                main_list: &camera_lists.main_list,
                batches: &batches,
                physical_fx,
                backface_fx: binds.backface_fx_group.as_ref(),
                probe_binding,
                skinned_visible: &skinned_visible,
                lit_skinned: &lit_skinned,
                terrain_indices,
                terrain_draws: &world_frame.terrain_draws,
                view_frustum: &view_frustum,
                scatter_batches: &world_frame.scatter_batches,
                cards: camera_cards,
                grass_frame: &world_frame.grass_frame,
                fluid_bounds: water_fluids.fluid_bounds,
                blended_list: &camera_lists.blended_list,
                sprite_frame: &arenas.sprite_frame,
                sprite_fx_group: binds.sprite_fx_group,
                particle_group: &binds.particle_group,
                particle_frame: &particle_frame,
                ribbon_frame: &arenas.ribbon_frame,
                line_frame: &line_frame,
                metaball_groups: &binds.metaball_groups,
                volumetric_fog,
                oit: camera_lists.oit_mode,
                oit_frame_group: binds.oit_frame_group.as_ref(),
                oit_probe_binding,
                gi_copy,
            },
        );
        plans.glow = GlowPass::bind(
            self,
            &mut frames.glow,
            (glow, arenas.sprite_frame.glow_runs.as_slice()),
        );
        plans.custom_post = CustomPostPass::bind(
            self,
            &mut frames.custom_post,
            CustomPostPlan {
                scene,
                posts: &effects.custom_posts,
            },
        );
        // The bind steps of the passes over the finished frame: into the
        // output, or its sRGB view, by way of the glass source when the frame
        // is routed through it.
        let finished = FinishedFrame {
            routed,
            overlay_view,
            paint_view,
            output,
            output_texture,
        };
        plans.viewport_views = ViewportViewsPass::bind(
            self,
            &mut frames.viewport_views,
            ViewportViewsPlan {
                scene,
                view_plans: &view_plans,
                viewport_format,
                to: finished,
            },
        );
        plans.ui_layers =
            UiLayersPass::bind(self, &mut frames.ui_layers, (scene, overlay_frame.as_ref()));
        plans.overlay = OverlayPass::bind(
            self,
            &mut frames.overlay,
            OverlayPlan {
                scene,
                cam: &cam,
                overlay_frame: overlay_frame.as_ref(),
                line_frame: &line_frame,
                glass_on,
                color_filter,
                to: finished,
            },
        );
        plans.hdr_output = HdrOutputPass::bind(
            self,
            &mut frames.hdr_output,
            (output, destination, scene.look.range_limit),
        );
        // The graph's passes in order, the passes over the finished frame
        // last (execute.rs).
        self.execute_graph(GraphRun {
            ctx: &ctx,
            plans: &plans,
            encoder: &mut encoder,
            stats: &mut stats,
            frame_bind_group: binds.frame_bind_group,
            scene_depth: binds.scene_depth,
            sky_group: binds.sky_group,
            slot,
            fsr,
            routed,
            output,
            gtao,
            pass_ms: &mut pass_ms,
            pass_queries: &mut pass_queries,
        });
        stages.end("graph encode");
        encoder.expect(self.info.caps, self.info.caps);
        for &(slot, _) in &probe_frame.due {
            self.probes.slots[slot].captured = Some(scene.frame.tick);
        }
        // Submit, present, and the history the next frame reads (submit_frame.rs).
        self.submit_frame(SubmitFrameInputs {
            scene,
            target,
            cam: &cam,
            encoder,
            surface_frame,
            debris_commands: instance_uploads.debris_commands,
            cull_commands: instance_uploads.cull_commands,
            pass_ms,
            pass_queries,
            oit_mode: camera_lists.oit_mode,
            gpu_cull: cull.gpu_cull,
            temporal,
            gi_copy,
            t: frame_uniforms.time,
            stats: &mut stats,
            sorted: (sorted_serial, sorted_still),
        });
        // Its tables' memory for the next frame's crowds.
        self.world.crowd_tables = crowd_frame;
        // The overlay as built, for a next frame that draws the same one.
        self.overlay.kept = kept_key.zip(overlay_frame);
        stages.end("submit");
        self.stage_ms = stages.done;
        Ok(stats)
    }
}
