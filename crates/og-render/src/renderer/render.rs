//! Frame rendering.

use super::*;

mod overlay_setup;

mod prepare_pipelines;
use prepare_pipelines::PrepareFramePipelinesInputs;

mod draw_skinned;
use draw_skinned::DrawSkinnedInputs;

mod draw_list;
use draw_list::DrawListInputs;

mod draw_sprites;

mod draw_strips;

mod draw_glow;

mod stylise_uniforms;

mod post_uniforms;
use post_uniforms::WriteFramePostUniformsInputs;

mod pass_probes;
use pass_probes::ProbesPassInputs;

mod pass_view;
use pass_view::ViewPassInputs;

mod pass_prepass;
use pass_prepass::PrepassPassInputs;

mod pass_reflection;
use pass_reflection::ReflectionPassInputs;

mod pass_main;
use pass_main::MainPassInputs;

mod view_culling;

mod camera;

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
        self.sync(scene)?;
        let (upscale, grid_offset) = self.prepare_frame_resources(scene);
        let (probe_frame, active_probes, probe_boxes) = self.prepare_frame_probes(scene);
        let (mut batches, sorted_instances) = self.build_batches(scene)?;
        let instances::Sorted {
            mut transforms,
            spheres,
            submission,
            sphere_box,
            ..
        } = sorted_instances;
        let mut origins = submission.clone();
        self.prepare_full_meshes(scene, &batches);
        // The meshlet path's cluster pools (meshlet_gpu.rs), once the
        // scene has a clustered mesh.
        if scene.meshlets && scene.mesh_clusters_all().iter().any(Option::is_some) {
            let limits = self.device.limits();
            let max = limits
                .max_storage_buffer_binding_size
                .min(limits.max_buffer_size);
            self.meshlet
                .get_or_insert_with(|| meshlet_gpu::MeshletGpu::new(&self.device, max))
                .sync(&self.device, &self.queue, scene);
        }
        let mut skinned = self.build_skinned(scene, &probe_boxes)?;
        // The planar water this frame draws: none where the 3D world is
        // not built in (WORLD_3D).
        let scene_water: &[Water] = if WORLD_3D { &scene.water } else { &[] };

        let (aspect, taa, camera, cam, view_frustum) = self.prepare_frame_camera(scene);
        let (volumetric_fog, frame_fog, reflections) = self.prepare_frame_reflections(
            scene,
            scene_water,
            &camera,
            &cam,
            &view_frustum,
            &aspect,
        );
        let reflection = reflections.first();
        let (view_plans, mut view_skinned) =
            self.prepare_view_culling(scene, &camera, &mut skinned);
        let (probe_faces, mut probe_skinned) =
            self.prepare_probe_culling(&probe_frame, &active_probes, &camera, &mut skinned);
        let (mut skinned_visible, skinned_culled) =
            Self::cull_skinned(&mut skinned, &view_frustum, scene.camera_layers);
        let mut skinned_batches = std::mem::take(&mut skinned.batches);
        let skinned_data = std::mem::take(&mut skinned.data);
        let mut joint_matrices = std::mem::take(&mut skinned.matrices);
        let joints_drawn = joint_matrices.len() as u32;
        // TAA's skinned motion vectors: the previous frame's joints and
        // blendshape weights in a second section of the palette (skin.rs,
        // its layout), read by the skinned prepass at `Fx::decal_info.w`.
        let taa_on = self.info.antialias == Antialias::Taa;
        let skinned_motion = if taa_on {
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

        // Effects this frame.
        let post_fx = &scene.post;
        let particle_frame = self.particles.prepare(
            &self.device,
            &self.queue,
            scene,
            self.desc_particles == ParticleSimulation::Auto,
            self.cache.textures.len(),
            camera.position,
            cam.forward,
        )?;
        // Mesh particles read the f32 vertices of a compact mesh too.
        for &m in &particle_frame.meshes {
            self.ensure_full(scene, m);
        }
        // The quality's resolution, where it sets one, over the scene's.
        let ao = post_fx
            .ao
            .filter(|a| scene.quality.ambient_occlusion && a.intensity > 0.0 && a.radius > 0.0)
            .map(|a| AmbientOcclusion {
                resolution: scene.quality.ao_resolution.unwrap_or(a.resolution),
                ..a
            });
        // Screen-space GI (ssgi.rs), in the AO passes' place: its targets
        // hold the AO too, at the GI's resolution when both are on.
        let gi = post_fx
            .gi
            .filter(|g| scene.quality.global_illumination && g.intensity > 0.0)
            .map(|g| ScreenSpaceGlobalIllumination {
                resolution: scene.quality.ao_resolution.unwrap_or(g.resolution),
                ..g
            });
        let ao = ao.map(|a| AmbientOcclusion {
            resolution: gi.map_or(a.resolution, |g| g.resolution),
            ..a
        });
        // The AO passes run (for the AO, the GI or both).
        let ao_target = ao.is_some() || gi.is_some();
        let outline = post_fx.outline.is_some()
            || batches
                .iter()
                .chain(&skinned_batches)
                .any(|b| !b.transparent && self.cache.materials[b.material].outline);
        let dof = post_fx.depth_of_field;
        let motion_blur = post_fx.motion_blur;
        let screen_blur = post_fx.screen_blur.filter(|b| b.strength != 0.0);
        let god_rays = post_fx.god_rays;
        let fluids_3d = WORLD_3D && scene.fluids_3d.iter().any(|f| !f.particles.is_empty());
        let depth_bands = scene.stylise.depth_bands.filter(|b| b.count >= 2);
        // Ink or hatching on depth edges reads the prepass in the lines
        // pass, a canvas, hatching or halftone on surfaces in the paint or
        // lines pass.
        let ink_depth = scene.stylise.depth_edges();
        // Screen-space reflections, read from the prepass (ssr.rs).
        let ssr = post_fx
            .ssr
            .filter(|s| scene.quality.ssr && s.intensity > 0.0)
            .map(|s| ssr::ScreenSpaceReflections {
                steps: scene.quality.ssr_steps(s.steps),
                ..s
            });
        // Custom post passes (post_custom.rs) that compiled; they read the
        // prepass's depth and normals.
        let custom_posts = self.custom_posts.prepare(&self.device, scene);
        let ui_depth = scene.overlay.as_ref().is_some_and(|o| o.ui.needs_depth());
        let prepass = !custom_posts.is_empty()
            || ao_target
            || ssr.is_some()
            // Volumetric fog reads each pixel's depth.
            || volumetric_fog.is_some()
            || depth_bands.is_some()
            || ink_depth
            || scene.stylise.on_surfaces()
            || outline
            || dof.is_some()
            || motion_blur.is_some()
            || god_rays.is_some()
            || particle_frame.needs_depth
            // Water and 3D fluids read the depth below them.
            || !scene_water.is_empty()
            || fluids_3d
            // UI occluders and mass elements the scene hides (§14).
            || ui_depth
            // TAA's motion vectors of what moved besides the camera.
            // (Instances only where they are culled on the CPU: GPU culling
            // draws them without their previous transforms.)
            || (taa_on
                && (skinned_motion == Some(true)
                    || skinned.baked_moved
                    || (!self.gpu_culls(transforms.len() as u32) && objects_moved(scene))));
        let (face_fluid, face_depth, object_space) =
            self.prepare_frame_pipelines(PrepareFramePipelinesInputs {
                scene,
                ao: ao_target,
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
        self.particles
            .ensure_pipelines(&self.device, &particle_frame, self.samples);
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
            )?),
            None => None,
        };
        let glass_on = self.prepare_frame_overlay(scene, &overlay_frame);
        let line_frame = self.prepare_frame_lines(scene, &cam);
        let mut ribbon_frame = self
            .trails
            .build(
                scene.id(),
                scene.trail_styles(),
                &scene.trails,
                &scene.trail_layers,
                &scene.ribbons,
                &scene.beams,
                scene.tick,
                scene.tick_rate,
                camera.position,
                cam.forward,
                cam.orthographic,
            )
            .map_err(RenderError::InvalidHandle)?;
        for run in &ribbon_frame.runs {
            if let Some(t) = run.texture
                && t.index() >= self.cache.textures.len()
            {
                return Err(RenderError::InvalidHandle(format!(
                    "strip texture {}",
                    t.index()
                )));
            }
            self.ribbon_group(run.texture);
        }
        let lit_range = scene.lighting_2d.map(|l| (l.min_layer, l.max_layer));
        // The light map's strips, one per light mask of the lit receivers.
        let light_slots = light2d::LightSlots::of(scene);
        let lit = |layer: i32, mask: u32| light_slots.lit(lit_range, layer, mask);
        let view = match camera.projection {
            Projection::Orthographic { height } => {
                let half = Vec2::new(height * 0.5 * aspect, height * 0.5);
                let c = camera.position.truncate();
                Some(sprites::ViewRect {
                    min: c - half,
                    max: c + half,
                })
            }
            Projection::Perspective { .. } => None,
        };
        water::water_2d_strips(&scene.water_2d, &mut ribbon_frame);
        // Each view's strips facing its own eye (views.rs), after the main
        // camera's in the strip buffer.
        let view_ribbons: Vec<Vec<ribbons::RibbonRun>> = view_plans
            .iter()
            .map(|p| {
                let mut f = self
                    .trails
                    .emit(p.position, p.frame.forward, p.frame.orthographic);
                water::water_2d_strips(&scene.water_2d, &mut f);
                let base = ribbon_frame.vertices.len() as u32;
                ribbon_frame.vertices.extend_from_slice(&f.vertices);
                f.runs
                    .into_iter()
                    .map(|mut r| {
                        r.first += base;
                        r
                    })
                    .collect()
            })
            .collect();
        let mut layered: Vec<sprites::LayeredStrip> = ribbon_frame
            .runs
            .iter()
            .filter_map(|r| {
                Some(sprites::LayeredStrip {
                    layer: r.layer?,
                    source: RunSource::Strip(r.blend as u8, r.texture),
                    first: r.first,
                    count: r.count,
                })
            })
            .collect();
        let fluids = self.metaballs.prepare(
            &self.device,
            &self.queue,
            &scene.metaballs,
            (self.width, self.height),
        );
        layered.extend(
            metaballs::layers(&scene.metaballs)
                .into_iter()
                .map(|(i, layer)| sprites::LayeredStrip {
                    layer,
                    source: RunSource::Metaballs(i as u8),
                    first: 0,
                    count: 1,
                }),
        );
        let fonts = &self.cache.fonts;
        let arrays = &self.cache.texture_arrays;
        let font = |i: usize| fonts.get(i).map(|f| &f.layout);
        let array_layers = |a: TextureArrayHandle| {
            scene
                .contains_texture_array(a)
                .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                .flatten()
                .map(|g| g.info.layers)
        };
        let mut sprite_frame = sprites::build(
            scene,
            font,
            self.cache.textures.len(),
            array_layers,
            &particle_frame.sprites,
            lit,
            view,
            &layered,
            scene.camera_layers,
            &sprites::SpriteEye {
                position: camera.position,
                forward: Some(cam.forward),
                frustum: Some(&view_frustum),
            },
        )?;
        // Each view's world sprites, 2D particles and text on its layers
        // (views.rs), their instances after the main camera's in the
        // sprite buffer, lit by the view's own 2D light map, its 2D fluids
        // composited from its own fields, its glowing sprites into its own
        // glow buffer.
        let mut view_sprites: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        let mut view_glow_runs: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        // Each view's billboards among its transparent meshes and their
        // drop shadows (sprites.rs `SpriteFrame::billboards`).
        let mut view_boards: Vec<sprites::Billboards> = Vec::new();
        for (p, runs) in view_plans.iter().zip(&view_ribbons) {
            let view_strips: Vec<sprites::LayeredStrip> = runs
                .iter()
                .filter_map(|r| {
                    Some(sprites::LayeredStrip {
                        layer: r.layer?,
                        source: RunSource::Strip(r.blend as u8, r.texture),
                        first: r.first,
                        count: r.count,
                    })
                })
                .chain(
                    layered
                        .iter()
                        .filter(|s| matches!(s.source, RunSource::Metaballs(_)))
                        .map(|s| sprites::LayeredStrip {
                            layer: s.layer,
                            source: s.source,
                            first: s.first,
                            count: s.count,
                        }),
                )
                .collect();
            let f = sprites::build(
                scene,
                font,
                self.cache.textures.len(),
                array_layers,
                &particle_frame.sprites,
                lit,
                None,
                &view_strips,
                p.layers,
                &sprites::SpriteEye {
                    position: p.position,
                    forward: Some(p.frame.forward),
                    frustum: Some(&Frustum::from_view_proj(p.frame.view_proj)),
                },
            )?;
            let in_buffer = |r: &sprites::SpriteRun| {
                !matches!(
                    r.source,
                    RunSource::Strip(..) | RunSource::Mesh(_) | RunSource::Metaballs(_)
                )
            };
            // Its glow instances follow every run's (its glow pass reads them).
            let base = sprite_frame.instances.len() as u32;
            sprite_frame.instances.extend_from_slice(&f.instances);
            view_boards.push(sprites::billboards_at(&f, base));
            view_glow_runs.push(
                f.glow_runs
                    .into_iter()
                    .map(|mut r| {
                        r.first += base;
                        r
                    })
                    .collect(),
            );
            view_sprites.push(
                f.runs
                    .into_iter()
                    .map(|mut r| {
                        if in_buffer(&r) {
                            r.first += base;
                        }
                        r
                    })
                    .collect(),
            );
        }
        // The due probe faces' world sprites, 2D particles and text (every
        // layer, unlit, without glow: probes.rs), after the views' in the
        // sprite buffer; built for each face, its billboards sorted by the
        // face camera's depth from the capture point among the face's
        // transparent meshes and culled to the face.
        let mut face_sprites: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        let mut face_boards: Vec<sprites::Billboards> = Vec::new();
        for (d, &(_, i)) in probe_frame.due.iter().enumerate() {
            for cam in &probe_faces[d] {
                let frustum = Frustum::from_view_proj(cam.view_proj);
                let f = sprites::build(
                    scene,
                    font,
                    self.cache.textures.len(),
                    array_layers,
                    &particle_frame.sprites,
                    |_, _| 0.0,
                    None,
                    &[],
                    u32::MAX,
                    &sprites::SpriteEye {
                        position: active_probes[i].position,
                        forward: Some(cam.forward),
                        frustum: Some(&frustum),
                    },
                )?;
                let base = sprite_frame.instances.len() as u32;
                sprite_frame.instances.extend_from_slice(&f.instances);
                face_boards.push(sprites::billboards_at(&f, base));
                face_sprites.push(
                    f.runs
                        .into_iter()
                        .map(|mut r| {
                            if !matches!(
                                r.source,
                                RunSource::Strip(..) | RunSource::Mesh(_) | RunSource::Metaballs(_)
                            ) {
                                r.first += base;
                            }
                            r
                        })
                        .collect(),
                );
            }
        }
        let sprite_frame = sprite_frame;
        for run in sprite_frame
            .runs
            .iter()
            .chain(&sprite_frame.glow_runs)
            .chain(view_sprites.iter().flatten())
            .chain(view_glow_runs.iter().flatten())
            .chain(face_sprites.iter().flatten())
            .chain(sprite_frame.billboards.iter().map(|b| &b.1))
            .chain(&sprite_frame.billboard_shadows)
            .chain(
                view_boards
                    .iter()
                    .chain(&face_boards)
                    .flat_map(|(b, s)| b.iter().map(|b| &b.1).chain(s)),
            )
        {
            match run.source {
                RunSource::Sprite(_, t) => {
                    self.sprite_group(t);
                    if let Some(t) = t {
                        self.sprite_normal_group(scene, t);
                    }
                }
                RunSource::Array(_, a) => self.sprite_array_group(scene, a),
                RunSource::Mesh(i) => {
                    let t = scene.sprite_meshes[i as usize].texture;
                    self.sprite_group(t);
                    if let Some(t) = t {
                        self.sprite_normal_group(scene, t);
                    }
                }
                _ => {}
            }
        }
        // The sprite meshes' corners (sprite_mesh.rs), lit as their layers' sprites.
        if !scene.sprite_meshes.is_empty() {
            let corners = sprite_mesh::vertices(scene, lit);
            let meshes = self.sprite_meshes.get_or_insert_with(|| {
                sprite_mesh::SpriteMeshRenderer::new(
                    &self.device,
                    [
                        &self.frame_layout,
                        &self.sprite_layout,
                        &self.sprite_fx_layout,
                    ],
                    self.samples,
                )
            });
            grow_and_write(
                &self.device,
                &self.queue,
                &mut meshes.buffer,
                "sprite meshes",
                bytemuck::cast_slice(&corners),
            );
        }
        for s in &particle_frame.systems {
            self.sprite_group(s.texture);
        }
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
        // Per-object motion for motion blur, and for TAA's motion vectors
        // where the instances are culled on the CPU (GPU culling compacts
        // them without their previous transforms, so TAA there reprojects
        // the camera's motion alone).
        let previous_valid = !scene.previous_transforms.is_empty()
            && scene.previous_transforms.len() == scene.instances.len();
        let blur_motion = motion_blur.is_some() && previous_valid;
        let sorted = transforms.len() as u32;
        // The weather occlusion map (weather.rs `shelter_view`), in the
        // cascade array's last layer (`fit_shelter_layer` gave it one):
        // drawn again only when what it was drawn from changes.
        let shelter = scene
            .surface_weather
            .as_ref()
            .filter(|_| self.cascade_map.layers.len() as u32 > self.info.shadow_cascades)
            .and_then(|w| {
                let (view_proj, cell) = weather::shelter_view(w, camera.position, scene.origin)?;
                let frustum = Frustum::from_view_proj(view_proj);
                let mut inside = vec![false; spheres.len()];
                for b in batches.iter().filter(|b| b.cast) {
                    inside[b.first as usize..(b.first + b.count) as usize].fill(true);
                }
                frustum.test_spheres(&mut inside, &spheres);
                let key = weather::shelter_key(
                    w,
                    cell,
                    spheres
                        .iter()
                        .zip(&inside)
                        .filter(|(_, i)| **i)
                        .map(|(s, _)| *s),
                    scene.terrains.len(),
                );
                Some((view_proj, frustum, key, self.shelter_key != Some(key)))
            });
        let shelter_redraw = shelter.as_ref().is_some_and(|s| s.3);
        // The passes this frame can cull at most: the camera, the
        // occluders, the planar reflection, the cascades and the shadowed
        // local lights' faces. The buffers must fit them; `encode` sizes
        // them for the frame's actual passes and regrows when that grows.
        let max_passes = 2
            + shelter_redraw as usize
            + reflection.is_some() as usize
            + if scene.light.cast_shadows {
                (self.info.shadow_cascades as usize).clamp(1, CASCADES)
            } else {
                0
            }
            + scene
                .lights
                .iter()
                .filter(|l| l.cast_shadows)
                .map(|l| match l.kind {
                    LightKind::Point => self.info.point_shadows.layers(),
                    LightKind::Spot { .. } => 1,
                    LightKind::Rect { two_sided, .. } => 1 + two_sided as usize,
                })
                .sum::<usize>()
                .min(shadows::capacity(
                    self.info.shadowed_lights as usize,
                    self.info.point_shadows,
                ));
        // The batches the meshlet path draws in the camera pass (under GPU
        // culling, which it follows; never with stylise's object pass,
        // which redraws the camera list's instances, nor with motion blur,
        // which culls on the CPU).
        let mut meshlet_taken: Vec<u32> = match self.meshlet.as_ref() {
            Some(m)
                if self.indirect
                    && self.desc_culling != Culling::Cpu
                    && !blur_motion
                    && !object_space =>
            {
                batches
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| {
                        meshlet_gpu::takes(
                            scene,
                            b,
                            m.clustered(b.mesh),
                            self.lightmaps.group(b.mesh).is_some(),
                        ) && self.cache.materials[b.material].wireframe.is_none()
                    })
                    .map(|(i, _)| i as u32)
                    .collect()
            }
            _ => Vec::new(),
        };
        // The work items the dispatch writes, within the device's storage
        // binding limit: the camera's first, then each shadow view while
        // they fit (a view past it culls and draws its casters as before).
        let meshlet_max = self.meshlet.as_ref().map_or(0, |m| m.max_items());
        let mut meshlet_items = match self.meshlet.as_ref() {
            Some(m) => m.items(&batches, &meshlet_taken, true),
            None => None,
        }
        .unwrap_or(u64::MAX);
        if meshlet_items > meshlet_max {
            meshlet_taken.clear();
            meshlet_items = 0;
        }
        // Clustered batches draw only under GPU culling: they turn it on
        // under `Culling::Auto` however few the instances.
        let gpu_cull = (self.gpu_culls(sorted) || !meshlet_taken.is_empty())
            && !blur_motion
            && !batches.is_empty()
            && {
                let limits = self.device.limits();
                let gc = self.gpu_cull.get_or_insert_with(|| {
                    cull_gpu::GpuCulling::new(
                        &self.device,
                        limits
                            .max_storage_buffer_binding_size
                            .min(limits.max_buffer_size),
                    )
                });
                gc.fits(sorted, max_passes)
            };
        if !gpu_cull {
            meshlet_taken.clear();
        }
        // Under GPU culling (the meshlet path's batches included) TAA
        // reprojects the camera's motion alone.
        let object_motion = previous_valid && (blur_motion || (taa && !gpu_cull));
        // The shadow casters among them: every shadow pass takes each one
        // (on any layer) through its own view of the dispatch.
        let meshlet_casters: Vec<u32> = meshlet_taken
            .iter()
            .copied()
            .filter(|&b| batches[b as usize].cast)
            .collect();
        // Every batch the path takes, on any layer: each lit view takes
        // those on its layers. A list leaves out a batch its view draws
        // (`in_meshlet`, by its instances' first slot).
        let meshlet_all = meshlet_taken.clone();
        let meshlet_firsts: Vec<u32> = {
            let mut f: Vec<u32> = meshlet_all
                .iter()
                .map(|&b| &batches[b as usize])
                .filter(|b| b.count > 0)
                .map(|b| b.first)
                .collect();
            f.sort_unstable();
            f
        };
        let in_meshlet = |b: &Batch| b.count > 0 && meshlet_firsts.binary_search(&b.first).is_ok();
        let on_layers = |mask: u32| -> Vec<u32> {
            meshlet_all
                .iter()
                .copied()
                .filter(|&b| batches[b as usize].layers & mask != 0)
                .collect()
        };
        let mut cull_passes: Vec<cull_gpu::CullPass> = Vec::new();
        let mut gpu_pass = |kind, frustum: &Frustum| {
            cull_passes.push(cull_gpu::CullPass {
                kind,
                planes: frustum.planes().to_vec(),
                // Shadow maps take every caster; the camera's passes its
                // layers (views.rs), as the CPU lists' `include`.
                layers: match kind {
                    cull_gpu::PassKind::Shadow => u32::MAX,
                    _ => scene.camera_layers,
                },
            });
            cull_passes.len() as u32 - 1
        };
        // Lent to every CPU cull of this frame, given back after the last.
        let mut cull_inside = std::mem::take(&mut self.cull_inside);
        cull_inside.whole.clear();
        // Occlusion needs a batch that can occlude and an earlier frame's
        // camera visibility (its visible occluders); without them the
        // occluders' depth, the pyramid and the occlusion test are skipped.
        let occlusion = gpu_cull
            && self.gpu_cull.as_ref().is_some_and(|gc| gc.primed())
            && batches
                .iter()
                .any(|b| self.cache.materials[b.material].occluder);
        // The main camera's layers (views.rs).
        let mask = scene.camera_layers;
        let (main_list, mut culled) = if gpu_cull {
            let g = gpu_pass(cull_gpu::PassKind::Main, &view_frustum);
            if occlusion {
                gpu_pass(cull_gpu::PassKind::Occluders, &view_frustum);
            }
            let mut list = indirect_list(&batches, sorted, g, |b| {
                !b.transparent && b.layers & mask != 0
            });
            // The meshlet path draws its batches itself; the camera's
            // layers still decide (a batch off them draws nothing there).
            meshlet_taken.retain(|&b| batches[b as usize].layers & mask != 0);
            list.ranges
                .retain(|r| meshlet_taken.binary_search(&r.0).is_err());
            (list, 0)
        } else {
            self.cull(
                &batches,
                &spheres,
                &mut transforms,
                &mut origins,
                &mut cull_inside,
                &view_frustum,
                true,
                |b| !b.transparent && b.layers & mask != 0,
            )
        };
        // The meshlet dispatch's views: the camera's, then one per
        // reflection, view, probe face and shadow pass that fits
        // (`meshlet_view`: its index, 0 none: that pass keeps its list).
        let mut meshlet_views = vec![meshlet_gpu::MeshletView::camera(
            &scene.camera,
            view_frustum.planes(),
            std::mem::take(&mut meshlet_taken),
            self.height,
        )];
        let camera_seen = meshlet_gpu::MeshletView {
            taken: Vec::new(),
            ..meshlet_views[0].clone()
        };
        let meshlet_gpu = self.meshlet.as_ref();
        let mut meshlet_view = |v: meshlet_gpu::MeshletView| -> u32 {
            if v.taken.is_empty() {
                return 0;
            }
            match meshlet_gpu.and_then(|m| m.items(&batches, &v.taken, false)) {
                Some(n) if meshlet_items + n <= meshlet_max => {
                    meshlet_items += n;
                    meshlet_views.push(v);
                    meshlet_views.len() as u32 - 1
                }
                _ => 0,
            }
        };
        let shadow_view = |meshlet_view: &mut dyn FnMut(meshlet_gpu::MeshletView) -> u32,
                           f: &Frustum| {
            meshlet_view(meshlet_gpu::MeshletView::shadow(
                &camera_seen,
                f.planes(),
                meshlet_casters.clone(),
            ))
        };
        let (mut blended_list, blended_culled) = self.sort_transparent(
            &batches,
            &spheres,
            &submission,
            &mut transforms,
            &mut origins,
            &view_frustum,
            camera.position,
            cam.forward,
            mask,
            &sprites::depths(&sprite_frame.billboards),
        );
        culled += blended_culled + skinned_culled;
        // (The GPU culling has one reflection pass: the second slot is culled here.)
        let mut reflection_list = Vec::new();
        let mut reflection_meshlets: Vec<u32> = Vec::new();
        for (k, r) in reflections.iter().enumerate() {
            let frustum = Frustum::from_view_proj(r.3);
            let view = meshlet_view(meshlet_gpu::MeshletView::seen(
                frustum.planes(),
                on_layers(mask),
                r.1.position,
                r.3,
                r.4.1,
            ));
            reflection_meshlets.push(view);
            let include =
                |b: &Batch| !b.transparent && b.layers & mask != 0 && !(view > 0 && in_meshlet(b));
            reflection_list.push(if gpu_cull && k == 0 {
                let g = gpu_pass(cull_gpu::PassKind::Reflection, &frustum);
                indirect_list(&batches, sorted, g, include)
            } else {
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &mut cull_inside,
                    &frustum,
                    // Under GPU culling the sorted instances are the
                    // culling's: append, never draw a batch whole (the
                    // shift below would wrap).
                    !gpu_cull,
                    include,
                )
                .0
            });
        }
        // The transparent meshes in each mirror (three.js's Reflector renders the whole scene: the sky dome's clouds,
        // the falls' curtains), back to front from the mirrored eye.
        let mut reflection_blended = Vec::new();
        for r in &reflections {
            let frustum = Frustum::from_view_proj(r.3);
            reflection_blended.push(
                self.sort_transparent(
                    &batches,
                    &spheres,
                    &submission,
                    &mut transforms,
                    &mut origins,
                    &frustum,
                    r.1.position,
                    r.2.forward,
                    mask,
                    &[],
                )
                .0,
            );
        }
        // Reflection probes: each due capture's faces culled, their
        // transparent meshes sorted back to front from the probe (probes.rs).
        let mut probe_lists: Vec<DrawList> = Vec::new();
        let mut probe_blended: Vec<DrawList> = Vec::new();
        let mut probe_views: Vec<u32> = Vec::new();
        for (d, cams) in probe_faces.iter().enumerate() {
            let probe = &active_probes[probe_frame.due[d].1];
            let eye = probe.position;
            for (face, f) in cams.iter().enumerate() {
                let frustum = Frustum::from_view_proj(f.view_proj);
                // A face with a depth prepass keeps its list (the prepass
                // draws it).
                let view = if face_depth {
                    0
                } else {
                    meshlet_view(meshlet_gpu::MeshletView::seen(
                        frustum.planes(),
                        meshlet_all.clone(),
                        eye,
                        f.view_proj,
                        probe.face_size(),
                    ))
                };
                probe_views.push(view);
                probe_lists.push(
                    self.cull(
                        &batches,
                        &spheres,
                        &mut transforms,
                        &mut origins,
                        &mut cull_inside,
                        &frustum,
                        !gpu_cull,
                        |b| !b.transparent && !(view > 0 && in_meshlet(b)),
                    )
                    .0,
                );
                probe_blended.push(
                    self.sort_transparent(
                        &batches,
                        &spheres,
                        &submission,
                        &mut transforms,
                        &mut origins,
                        &frustum,
                        eye,
                        f.forward,
                        u32::MAX,
                        &sprites::depths(&face_boards[6 * d + face].0),
                    )
                    .0,
                );
            }
        }

        // Each view's transparent meshes on its layers, back to front from
        // its eye.
        let mut view_blended: Vec<DrawList> = view_plans
            .iter()
            .zip(&view_boards)
            .map(|(p, boards)| {
                self.sort_transparent(
                    &batches,
                    &spheres,
                    &submission,
                    &mut transforms,
                    &mut origins,
                    &Frustum::from_view_proj(p.frame.view_proj),
                    p.position,
                    p.frame.forward,
                    p.layers,
                    &sprites::depths(&boards.0),
                )
                .0
            })
            .collect();
        // Each view's opaque meshes on its layers, culled on the CPU (the
        // main camera, its shadows and reflection keep GPU culling: under
        // it these lists append every instance they draw).
        let mut view_views: Vec<u32> = Vec::new();
        let mut view_lists: Vec<DrawList> = view_plans
            .iter()
            .map(|p| {
                // A view with a depth prepass (a depth texture, soft
                // particles: views.rs `write`) keeps its list.
                let depth = particle_frame.soft
                    || matches!(p.dest, views::Dest::Texture { depth: Some(_), .. });
                let view = if depth {
                    0
                } else {
                    meshlet_view(meshlet_gpu::MeshletView::seen(
                        Frustum::from_view_proj(p.frame.view_proj).planes(),
                        on_layers(p.layers),
                        p.position,
                        p.frame.view_proj,
                        p.size.1,
                    ))
                };
                view_views.push(view);
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &mut cull_inside,
                    &Frustum::from_view_proj(p.frame.view_proj),
                    !gpu_cull,
                    |b| !b.transparent && b.layers & p.layers != 0 && !(view > 0 && in_meshlet(b)),
                )
                .0
            })
            .collect();

        // Terrains: new or edited data uploaded, this frame's clipmap levels.
        let prev_view_proj = match self.prev_view_proj {
            Some(vp) if !scene.camera_cut => vp,
            _ => cam.view_proj,
        };
        let textures = &self.cache.textures;
        let arrays = &self.cache.texture_arrays;
        let terrain_draws = if !WORLD_3D {
            Vec::new()
        } else {
            self.terrain.prepare(
                &self.device,
                &self.queue,
                scene,
                camera.position,
                camera.far,
                self.height,
                cam.view_proj,
                prev_view_proj,
                |h| textures.get(h.index()).map(|t| &t.view),
                |h| {
                    arrays
                        .get(h.index())
                        .and_then(Option::as_ref)
                        .filter(|a| a.generation == h.generation())
                        .map(|a| &a.texture)
                },
            )?
        };
        let terrain_casters = terrain_draws.iter().any(|d| d.cast_shadows);
        // Scattered copies: the chunks this frame needs built, what to draw.
        let scatter_batches = if !WORLD_3D {
            scatter::ScatterBatches::default()
        } else {
            let materials = &self.cache.materials;
            let meshes = &self.cache.meshes;
            self.scatter.prepare(
                &self.device,
                &self.queue,
                scene,
                camera.position,
                cam.view_proj,
                prev_view_proj,
                |m| materials.get(m).map(|g| g.side),
                |m| {
                    meshes.get(m).filter(|g| g.index_count > 0).map(|g| {
                        g.bounds
                            .map_or((1.0, 1.0), |(c, e)| (c.length() + e.length(), c.y + e.y))
                    })
                },
                |m| materials.get(m).and_then(|g| g.shader),
                |sh| scene.shaders()[sh].hooks_wgsl(),
            )
        };
        let scatter_material = |m: usize| self.cache.materials.get(m).map(|g| &g.bind_group);
        let scatter_mesh = |m: usize| {
            self.cache
                .meshes
                .get(m)
                .map(|g| (&g.vertices, &g.indices, g.index_count))
        };

        // The directional light's cascades.
        let light = &scene.light;
        let to_light = light.direction_to_light.normalize_or(Vec3::Y);
        let shadow_size = self.info.shadow_map_size;
        let has_casters = batches.iter().chain(&skinned_batches).any(|b| b.cast) || terrain_casters;
        let cascades: Vec<Cascade> = if light.cast_shadows && has_casters {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => {
                    // The instances' box (kept while they stand still), then
                    // the skinned instances and the terrains.
                    let skinned = scene.skinned_instances.iter().map(|s| {
                        let r = self.cache.meshes[s.mesh.index()]
                            .bounds
                            .map_or(1.0, |(_, e)| e.length() * 2.0);
                        (s.transform.w_axis.truncate(), r)
                    });
                    let all = instances::fold_box(sphere_box, skinned);
                    Self::instance_bounds(instances::fold_box(
                        all,
                        if WORLD_3D {
                            self.terrain.bounds(scene)
                        } else {
                            Vec::new()
                        },
                    ))
                }
            };
            if light.shadow_box > 0.0 {
                let level = Vec3::new(cam.forward.x, 0.0, cam.forward.z).normalize_or(Vec3::NEG_Z);
                let focus = camera.position + level * light.shadow_focus
                    - Vec3::Y * light.shadow_focus_drop;
                frame::fit_box(
                    to_light,
                    bounds,
                    focus,
                    light.shadow_box,
                    shadow_size,
                    light.shadow_bias,
                    light.shadow_normal_bias,
                    scene.origin,
                )
            } else {
                frame::fit_cascades(
                    &camera,
                    &cam,
                    aspect,
                    to_light,
                    bounds,
                    light.max_shadow_distance,
                    shadow_size,
                    self.info.shadow_cascades as usize,
                    light.cascade_blend,
                    scene.origin,
                )
            }
        } else {
            Vec::new()
        };
        // Each shadow pass's meshlet view (0: none), by cascade and by
        // local layer; its clustered casters leave the pass's list.
        let mut cascade_views: Vec<u32> = Vec::new();
        let mut local_views = [0u32; LOCAL_SHADOW_LAYERS];
        let caster_list = |g: u32, view: u32| {
            let mut list = indirect_list(&batches, sorted, g, |b| b.cast);
            if view > 0 {
                list.ranges
                    .retain(|r| meshlet_casters.binary_search(&r.0).is_err());
            }
            list
        };
        let cascade_lists: Vec<DrawList> = cascades
            .iter()
            .map(|c| {
                if gpu_cull {
                    let g = gpu_pass(cull_gpu::PassKind::Shadow, &c.frustum);
                    let view = shadow_view(&mut meshlet_view, &c.frustum);
                    cascade_views.push(view);
                    caster_list(g, view)
                } else {
                    self.cull(
                        &batches,
                        &spheres,
                        &mut transforms,
                        &mut origins,
                        &mut cull_inside,
                        &c.frustum,
                        true,
                        |b| b.cast,
                    )
                    .0
                }
            })
            .collect();
        // The weather occlusion map's casters (not the skinned or the
        // scattered copies: see `shadow_pass`), when it is drawn again: its
        // clustered casters through a shadow view of the meshlet dispatch,
        // as a cascade's (its own planes, no cone or occlusion, the
        // camera's level per model), and its meshlet view (0: none).
        let mut shelter_view = 0;
        let shelter_list: Option<DrawList> = shelter.as_ref().filter(|s| s.3).map(|s| {
            if gpu_cull {
                let g = gpu_pass(cull_gpu::PassKind::Shadow, &s.1);
                shelter_view = shadow_view(&mut meshlet_view, &s.1);
                caster_list(g, shelter_view)
            } else {
                self.cull(
                    &batches,
                    &spheres,
                    &mut transforms,
                    &mut origins,
                    &mut cull_inside,
                    &s.1,
                    true,
                    |b| b.cast,
                )
                .0
            }
        });

        // Local lights: the clusters they reach, their records, their shadow faces.
        self.clusters.build(
            &scene.lights,
            &clusters::ClusterView {
                camera: &cam,
                frustum: &view_frustum,
                position: camera.position,
                near: camera.near,
                far: camera.far,
                width: self.width,
                height: self.height,
            },
        );
        let (mut lights_gpu, local_shadows, shadowed_dropped) = frame::local_lights(
            &scene.lights,
            &self.clusters.kept,
            camera.position,
            frame::LocalShadowBudget {
                max_shadowed: self.info.shadowed_lights.min(scene.quality.shadowed_lights) as usize,
                rect_shadows: scene.quality.rect_shadows,
                point: self.info.point_shadows,
                shadow_size: self.local_shadow_size,
            },
        );
        debug_assert!(
            local_shadows
                .last()
                .is_none_or(|ls| ls.layer + ls.faces.len() <= self.local_map.layers.len())
        );
        // Cookies in the local shadow array past its shadow layers (area.rs).
        let cookie_base =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows) as u32;
        if !scene.light_cookies.is_empty() && self.local_map.layers.len() as u32 > cookie_base {
            let textures = &self.cache.textures;
            self.cookies.prepare(
                &self.device,
                &self.queue,
                scene,
                (&self.local_map.texture, &self.local_map.layers),
                cookie_base,
                |h| textures.get(h.index()).map(|t| &t.view),
            );
            self.cookies
                .patch(scene, &self.clusters.kept, &mut lights_gpu, cookie_base);
        }
        if let Some(fog) = &volumetric_fog {
            self.volumetric.prepare(
                &self.device,
                &self.queue,
                &self.frame_layout,
                fog,
                (self.width, self.height),
                scene.tick,
                &scene.quality,
            );
        }
        if !lights_gpu.is_empty() {
            self.queue
                .write_buffer(&self.light_buffers[0], 0, bytemuck::cast_slice(&lights_gpu));
            self.queue.write_buffer(
                &self.light_buffers[1],
                0,
                bytemuck::cast_slice(&self.clusters.table),
            );
            self.queue.write_buffer(
                &self.light_buffers[2],
                0,
                bytemuck::cast_slice(&self.clusters.ref_words()),
            );
        }
        // The planar reflection's clusters: the same light records seen by
        // the mirrored camera (its oblique projection moves only depth).
        for (k, (_, mirrored, f, _, (rw, rh))) in reflections.iter().enumerate() {
            self.reflection_clusters[k].build_kept(
                &scene.lights,
                &self.clusters.kept,
                &clusters::ClusterView {
                    camera: f,
                    frustum: &Frustum::from_view_proj(f.view_proj),
                    position: mirrored.position,
                    near: mirrored.near,
                    far: mirrored.far,
                    width: *rw,
                    height: *rh,
                },
            );
            if !lights_gpu.is_empty() {
                self.queue.write_buffer(
                    &self.reflection_light_buffers[k][1],
                    0,
                    bytemuck::cast_slice(&self.reflection_clusters[k].table),
                );
                self.queue.write_buffer(
                    &self.reflection_light_buffers[k][2],
                    0,
                    bytemuck::cast_slice(&self.reflection_clusters[k].ref_words()),
                );
            }
        }
        let mut local_lists: Vec<Option<DrawList>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        let mut local_frustums: Vec<Option<Frustum>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        for ls in &local_shadows {
            for (face, (_, f)) in ls.faces.iter().enumerate() {
                local_frustums[ls.layer + face] = Some(f.clone());
                let list = if gpu_cull {
                    let g = gpu_pass(cull_gpu::PassKind::Shadow, f);
                    let view = shadow_view(&mut meshlet_view, f);
                    local_views[ls.layer + face] = view;
                    caster_list(g, view)
                } else {
                    self.cull(
                        &batches,
                        &spheres,
                        &mut transforms,
                        &mut origins,
                        &mut cull_inside,
                        f,
                        true,
                        |b| b.cast,
                    )
                    .0
                };
                local_lists[ls.layer + face] = Some(list);
            }
        }
        self.cull_inside = cull_inside;

        // Upload instances, previous transforms, skinned data, strips and
        // sprites. With GPU culling the sorted instances go to its buffer and
        // this one holds only the transparent draws.
        let mut gpu_counts = None;
        let copy_counts = std::mem::take(&mut self.culling_counts);
        let cull_commands = if gpu_cull {
            let gc = self
                .gpu_cull
                .as_mut()
                .expect("made when gpu_cull was decided");
            if gc.in_flight() {
                let _ = self.device.poll(wgpu::PollType::Poll);
            }
            let lagged = gc.poll_counts();
            debug_assert!(cull_passes.len() <= max_passes);
            let (commands, occluder_draws) = gc.encode(
                &self.device,
                &self.queue,
                &transforms[..sorted as usize],
                &batches,
                &self.cache.meshes,
                &self.cache.materials,
                &cull_passes,
                cam.jittered_view_proj,
                (self.width, self.height, self.samples),
                self.frame_index,
                lagged.unwrap_or_default(),
                copy_counts,
            );
            gpu_counts = Some((lagged, occluder_draws));
            // The CPU-culled lists sit after the sorted instances, in this buffer.
            for list in reflection_list
                .iter_mut()
                .filter(|l| l.indirect.is_none())
                .chain(&mut reflection_blended)
                .chain(std::iter::once(&mut blended_list))
            {
                for r in &mut list.ranges {
                    r.1 -= sorted;
                }
            }
            for r in probe_lists
                .iter_mut()
                .chain(probe_blended.iter_mut())
                .chain(view_lists.iter_mut())
                .chain(view_blended.iter_mut())
                .flat_map(|l| &mut l.ranges)
            {
                r.1 -= sorted;
            }
            let mut commands = vec![commands];
            // The clustered batches' culling, after GPU culling's (it
            // reads the camera pass's survivors and the pyramid).
            if let Some(m) = self.meshlet.as_mut() {
                let gc = self.gpu_cull.as_ref().expect("made above");
                // Last frame's transforms at the same slots: with object
                // motion, the previous-transform buffer (the regular
                // prepass's rule), once it can be bound as storage.
                let previous = if object_motion
                    && self
                        .prev_instance_buffer
                        .usage()
                        .contains(wgpu::BufferUsages::STORAGE)
                {
                    &self.prev_instance_buffer
                } else {
                    &gc.instances
                };
                // The levels callers picked, at the sorted slots.
                let levels: Vec<u8> = if scene.instance_levels.is_empty() {
                    Vec::new()
                } else {
                    origins[..sorted as usize]
                        .iter()
                        .map(|&i| {
                            let i = i as usize;
                            scene.instance_levels.get(i).copied().unwrap_or(u8::MAX)
                        })
                        .collect()
                };
                let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
                commands.extend(m.encode(
                    &self.device,
                    &self.queue,
                    scene,
                    &batches,
                    &meshlet_views,
                    gc,
                    cam.jittered_view_proj,
                    (self.width, self.height),
                    previous,
                    &levels,
                    |mesh| &meshes[mesh].vertices,
                    |b| materials[b.material].occluder,
                ));
                if !m.frame.is_empty() {
                    m.ensure_pipelines(
                        &self.device,
                        [&self.frame_layout, &self.material_layout, &self.fx_layout],
                        planar_slot(self.info.planar_reflections),
                        &batches,
                        prepass,
                        self.samples,
                    );
                    m.ensure_shadow_pipelines(
                        &self.device,
                        (&self.shadow_bind_layout, &self.material_layout),
                        &batches,
                    );
                }
            }
            commands
        } else {
            if let Some(m) = self.meshlet.as_mut() {
                m.frame.clear();
            }
            Vec::new()
        };
        let cpu_instances = if gpu_cull {
            &transforms[sorted as usize..]
        } else {
            &transforms[..]
        };
        // The records a draw reads: under GPU culling every one here (the
        // lists' own copies); on the CPU path the batches a list draws
        // whole and the copies appended after the sorted instances.
        let mut drawn: Vec<std::ops::Range<usize>> = Vec::new();
        let mut draw = |r: std::ops::Range<usize>| match drawn.last_mut() {
            Some(last) if last.end == r.start => last.end = r.end,
            _ if r.is_empty() => {}
            _ => drawn.push(r),
        };
        if !gpu_cull {
            for (b, _) in batches
                .iter()
                .zip(&self.cull_inside.whole)
                .filter(|(_, w)| **w)
            {
                draw(b.first as usize..(b.first + b.count) as usize);
            }
            draw(sorted as usize..cpu_instances.len());
        } else {
            draw(0..cpu_instances.len());
        }
        let stride = INSTANCE_STRIDE as usize;
        let drawn_bytes: Vec<std::ops::Range<usize>> = drawn
            .iter()
            .map(|r| r.start * stride..r.end * stride)
            .collect();
        // Of those, only the blocks that changed since the last frame are
        // written (a buffer just replaced is written whole).
        let instance_bytes: &[u8] = bytemuck::cast_slice(cpu_instances);
        if grow(
            &self.device,
            &mut self.instance_buffer,
            "instances",
            instance_bytes.len() as u64,
        ) {
            self.instance_upload.invalidate();
        }
        let mut instance_upload_bytes = self.instance_upload.write(
            &self.queue,
            &self.instance_buffer,
            instance_bytes,
            &drawn_bytes,
        );
        if gpu_cull && let Some(gc) = &self.gpu_cull {
            instance_upload_bytes += gc.uploaded;
        }
        if object_motion {
            // As the instances' own: with a compact run's dequantisation,
            // for the records a draw reads.
            let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
            let mut prev = vec![instances::GpuTransform::default(); origins.len()];
            for k in drawn.iter().flat_map(|r| r.clone()) {
                let i = origins[k] as usize;
                let (inst, p) = (&scene.instances[i], scene.previous_transforms[i]);
                let d = Self::dequant(meshes, materials, inst.mesh.index(), inst.material.index());
                prev[k] = instances::gpu_transform(&d.map_or(p, |d| p * d));
            }
            let bytes: &[u8] = bytemuck::cast_slice(&prev);
            if grow(
                &self.device,
                &mut self.prev_instance_buffer,
                "previous instances",
                bytes.len() as u64,
            ) {
                self.prev_instance_upload.invalidate();
            }
            instance_upload_bytes += self.prev_instance_upload.write(
                &self.queue,
                &self.prev_instance_buffer,
                bytes,
                &drawn_bytes,
            );
        } else {
            // Not written this frame: what it holds is not known next time.
            self.prev_instance_upload.invalidate();
        }
        self.sorted.restore(transforms, spheres);
        if !skinned_data.is_empty() {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.skinned_buffer,
                "skinned instances",
                bytemuck::cast_slice(&skinned_data),
            );
            let morphs_changed = std::mem::take(&mut self.cache.morphs_dirty);
            self.palette.write(
                &self.device,
                &self.queue,
                &self.palette_layout,
                &joint_matrices,
                &self.cache.morph_data,
                morphs_changed,
            );
        }
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.ribbon_buffer,
            "strips",
            bytemuck::cast_slice(&ribbon_frame.vertices),
        );
        grow_and_write(
            &self.device,
            &self.queue,
            &mut self.sprite_buffer,
            "sprites",
            bytemuck::cast_slice(&sprite_frame.instances),
        );
        let mut ui_uniform = None;
        if let (Some(of), Some(o)) = (&overlay_frame, &scene.overlay) {
            grow_and_write(
                &self.device,
                &self.queue,
                &mut self.overlay.buffer,
                "overlay instances",
                bytemuck::cast_slice(&of.instances),
            );
            let u = overlay::OverlayRenderer::uniform(
                self.out_width,
                self.out_height,
                o,
                !self.output_format.is_srgb() && !self.has_linear_view(target),
            );
            ui_uniform = Some(u);
        }
        // The sprite shaders' light map strips and ambient
        // (`SpriteFxFrame::light`).
        let ambient_2d = scene.lighting_2d.map_or(Vec3::ONE, |l| l.ambient);
        self.queue.write_buffer(
            &self.sprite_fx_buffer,
            16,
            bytemuck::bytes_of(&[
                light_slots.count() as f32,
                ambient_2d.x,
                ambient_2d.y,
                ambient_2d.z,
            ]),
        );
        let (lights_2d, occluders_2d) = match scene.lighting_2d {
            Some(l) => self.lights_2d.write(
                &self.device,
                &self.queue,
                cam.view_proj,
                &scene.lights_2d,
                &scene.occluders_2d,
                &light_slots,
                l.shadow_angles,
            ),
            None => (0, 0),
        };

        // Uniforms.
        let mut uniform = frame::FrameUniform {
            view_proj: cam.jittered_view_proj.to_cols_array_2d(),
            inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
            view_pos: camera
                .position
                .extend(if cam.orthographic { 1.0 } else { 0.0 })
                .to_array(),
            light_dir: to_light.extend(cascades.len() as f32).to_array(),
            light_color: (light.color * light.intensity)
                .extend(scene.debug.lit_code())
                .to_array(),
            params: [
                scene.environment.intensity,
                (ibl::PREFILTER_MIPS - 1) as f32,
                lights_gpu.len() as f32,
                1.0 / shadow_size as f32,
            ],
            view_forward: cam
                .forward
                .extend(1.0 / self.local_shadow_size as f32)
                .to_array(),
            cascade_view_proj: [[[0.0; 4]; 4]; CASCADES],
            cascade_bias: [[0.0; 4]; CASCADES],
            cluster: self.clusters.params,
            local_view_proj: [[[0.0; 4]; 4]; LOCAL_SHADOW_LAYERS],
            atmosphere: atmosphere::AtmosphereGpu::new(
                frame_fog.as_ref(),
                match scene.environment.background {
                    Background::Procedural {
                        sun_direction,
                        sun_color,
                    } => Some((
                        scene
                            .environment
                            .sky
                            .look
                            .as_ref()
                            .unwrap_or(&SkyLook::DEFAULT),
                        sun_direction,
                        sun_color,
                    )),
                    _ => None,
                },
                scene.clouds.as_ref(),
                scene.surface_weather.as_ref(),
                scene.tick,
                scene.tick_rate,
            ),
            caustics: water::caustics(scene_water, scene.origin),
            shadow: [
                if cascades.is_empty() {
                    0.0
                } else {
                    light.shadow_softness.max(0.0)
                },
                (self.info.point_shadows == PointShadows::Paraboloid) as u32 as f32,
                0.0,
                0.0,
            ],
            stylise: match scene.stylise.retro {
                Some(r) if r.vertex_snap > 0.0 || r.affine > 0.0 || r.depth_bits > 0 => [
                    if r.vertex_snap > 0.0 {
                        self.width as f32 / r.vertex_snap
                    } else {
                        0.0
                    },
                    if r.vertex_snap > 0.0 {
                        self.height as f32 / r.vertex_snap
                    } else {
                        0.0
                    },
                    r.affine.clamp(0.0, 1.0),
                    r.depth_bits.min(24) as f32,
                ],
                _ => [0.0; 4],
            },
            shadow_tint: {
                let m = &scene.stylise.material;
                let a = m.shadow_tint_amount.clamp(0.0, 1.0);
                [m.shadow_tint[0], m.shadow_tint[1], m.shadow_tint[2], a]
            },
            outline_style: [
                scene.stylise.material.outline_distance.max(0.0),
                scene.stylise.material.shadow_darkness.clamp(0.0, 1.0),
                // The terrain's toon (terrain_lit.wgsl): it has no material to carry it.
                f32::from(u8::from(scene.stylise.material.toon)),
                0.0,
            ],
            probe: [[0.0; 4]; 3],
            view_right: cam.axes().0,
            view_up: cam.axes().1,
            hemi_sky: {
                let (s, _, i) = scene.hemisphere;
                // Irradiance in the IBL's units (the lit shaders add it beside the environment's
                // irradiance, at its own strength: an environment at intensity 0 leaves it).
                [s[0], s[1], s[2], i.max(0.0) / std::f32::consts::PI]
            },
            hemi_ground: {
                let g = scene.hemisphere.1;
                [g[0], g[1], g[2], 0.0]
            },
            material_surface: scene.stylise.material.surface_uniform(),
            origin: scene.origin.extend(0.0).to_array(),
        };
        // The shadow passes' clock, and the scene's origin for the hooks
        // (`h.origin`, `origin.rs`).
        let t = uniform.atmosphere.time;
        let shadow_time = [t[0], scene.origin.x, scene.origin.y, scene.origin.z];
        let mut shadow_slots = vec![bytemuck::Zeroable::zeroed(); SHADOW_SLOTS];
        for (i, c) in cascades.iter().enumerate() {
            uniform.cascade_view_proj[i] = c.view_proj.to_cols_array_2d();
            uniform.cascade_bias[i] = [c.normal_offset, c.depth_bias, c.split_far, c.blend_start];
            shadow_slots[i] = ShadowPassUniform {
                view_proj: c.view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                time: shadow_time,
                paraboloid: [0.0; 4],
                pad: [0.0; 36],
            };
        }
        for ls in &local_shadows {
            for (face, (vp, _)) in ls.faces.iter().enumerate() {
                let layer = ls.layer + face;
                uniform.local_view_proj[layer] = vp.to_cols_array_2d();
                shadow_slots[CASCADES + layer] = ShadowPassUniform {
                    view_proj: vp.to_cols_array_2d(),
                    light: ls.position.extend(ls.range).to_array(),
                    time: shadow_time,
                    // Hemisphere 0 below the light, 1 above (frame.rs).
                    paraboloid: if ls.paraboloid {
                        [
                            1.0,
                            if face == 0 { 1.0 } else { -1.0 },
                            self.local_shadow_size as f32,
                            shadows::PARABOLOID_SCALE,
                        ]
                    } else {
                        [0.0; 4]
                    },
                    pad: [0.0; 36],
                };
            }
        }
        // The weather occlusion map: its view and its layer, the array's
        // last (never a constant: cascades or cached layers may come first).
        if let Some((view_proj, ..)) = &shelter {
            let side = self.info.shadow_map_size as f32;
            let w = scene.surface_weather.as_ref().expect("filtered above");
            let texel = w.shelter.clamp(4.0, 1024.0) / side;
            uniform.atmosphere.shelter = [
                view_proj.x_axis.to_array(),
                view_proj.y_axis.to_array(),
                view_proj.z_axis.to_array(),
                view_proj.w_axis.to_array(),
                [
                    self.cascade_map.layers.len() as f32,
                    weather::DRIP_EDGE / texel,
                    side,
                    // 5 cm along the fall, in the map's depth.
                    0.05 / (2.0 * weather::SHELTER_REACH),
                ],
            ];
            shadow_slots[SHELTER_SLOT] = ShadowPassUniform {
                view_proj: view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                time: shadow_time,
                paraboloid: [0.0; 4],
                pad: [0.0; 36],
            };
        }
        self.queue
            .write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&uniform));
        let probe_positions: Vec<Vec3> = probe_frame
            .due
            .iter()
            .map(|&(_, i)| active_probes[i].position)
            .collect();
        self.probes.write(
            &self.queue,
            &uniform,
            &probe_frame,
            &probe_faces,
            &probe_positions,
        );
        for (k, (_, mirrored, f, view_proj, _)) in reflections.iter().enumerate() {
            let mut u = uniform;
            u.view_proj = view_proj.to_cols_array_2d();
            // The sky's view rays: the mirrored camera's own projection
            // (the oblique one tilts its far plane through the sky).
            u.inv_view_proj = f.view_proj.inverse().to_cols_array_2d();
            u.view_pos = mirrored.position.extend(0.0).to_array();
            u.view_forward = f.forward.extend(u.view_forward[3]).to_array();
            (u.view_right, u.view_up) = f.axes();
            u.caustics = [[0.0; 4]; 4];
            u.cluster = self.reflection_clusters[k].params;
            self.queue
                .write_buffer(&self.reflection_buffer[k], 0, bytemuck::bytes_of(&u));
        }
        self.queue
            .write_buffer(&self.shadow_buffer, 0, bytemuck::cast_slice(&shadow_slots));

        // The sun on screen, for god rays.
        let sun_clip = cam.view_proj * (camera.position + to_light * camera.far * 0.5).extend(1.0);
        let sun_ndc = sun_clip.truncate().truncate() / sun_clip.w.max(1e-6);
        let sun_front = sun_clip.w > 0.0 && to_light.dot(cam.forward) > 0.0;
        let sun_fade = (2.0 - sun_ndc.abs().max_element()).clamp(0.0, 1.0);
        let god_visible = god_rays.is_some() && sun_front && sun_fade > 0.0;
        self.post.write_uniform(
            &self.queue,
            camera.exposure,
            post_fx,
            glow,
            god_rays.is_some(),
            sprite_frame.untonemapped,
            scene.tick,
            self.history_size().0,
        );
        if camera.exposure.adaptation().is_some() {
            self.post.advance_exposure(
                &self.queue,
                camera.exposure,
                scene.tick,
                scene.tick_rate,
                (self.frame_index % 2) as usize,
            );
        } else {
            self.post.forget_exposure();
        }
        let mut fx_uniform =
            fx::fx_uniform(cam.view_proj, prev_view_proj, (self.width, self.height));
        // The decals: their records, and the main view's clusters they reach.
        fx::pack_decals(
            &scene.decals,
            &mut self.decal_records,
            &mut self.decal_spheres,
        );
        let decals = self.decal_records.len() as u32;
        if decals > 0 {
            self.clusters.bin_decals(
                &self.decal_spheres,
                &clusters::ClusterView {
                    camera: &cam,
                    frustum: &view_frustum,
                    position: camera.position,
                    near: camera.near,
                    far: camera.far,
                    width: self.width,
                    height: self.height,
                },
                &mut self.decal_bins,
            );
            self.decal_buffers.write(
                &self.device,
                &self.queue,
                &self.decal_records,
                &self.decal_bins.cells,
            );
        }
        let decals_dropped = scene.decals.len() as u32 - decals
            + if decals > 0 {
                self.decal_bins.refs_dropped
            } else {
                0
            };
        // The previous joints' section (`skin::append_previous`).
        if skinned_motion.is_some() {
            fx_uniform.decal_info[3] = joints_drawn as f32;
        }
        fx_uniform.decal_info[0] = decals as f32;
        fx_uniform.decal_info[1] = match scene.decal_atlas {
            None => 0.0,
            Some(t) if scene.texture_color_space(t) == Some(ColorSpace::Srgb) => 2.0,
            Some(_) => 1.0,
        };
        // w: 1 with the GI's bounce in the AO texture's gba.
        if ao_target {
            fx_uniform.ao = [
                1.0,
                1.0,
                ao.map_or(0.0, |a| a.direct),
                gi.is_some() as u32 as f32,
            ];
        }
        // The planar reflection's plane (n, n·at) for the material shaders
        // that read it (pbr.wgsl `planar_reflection`); 0 without one.
        if let Some((i, ..)) = &reflection {
            let w = &scene.water[*i];
            let (n, at, ..) = water::surface(w, w.mesh.and_then(|m| scene.mesh(m)));
            fx_uniform.planar = n.extend(n.dot(at)).to_array();
        }
        self.queue
            .write_buffer(&self.fx_buffer, 0, bytemuck::bytes_of(&fx_uniform));
        for (k, (_, _, _, view_proj, (rw, rh))) in reflections.iter().enumerate() {
            let mut u = fx_uniform;
            u.ao = [0.0; 4];
            // Drawing the reflection itself: no plane, w -1 (a surface that
            // reads it may leave itself out, as three.js's readers of the
            // mirror do).
            u.planar = [0.0, 0.0, 0.0, -1.0];
            u.view_proj_unjittered = view_proj.to_cols_array_2d();
            u.prev_view_proj = view_proj.to_cols_array_2d();
            u.screen = [*rw as f32, *rh as f32, 1.0 / *rw as f32, 1.0 / *rh as f32];
            fx::unclustered(&mut u);
            self.queue
                .write_buffer(&self.reflection_fx_buffer[k], 0, bytemuck::bytes_of(&u));
        }
        let viewport_format = self.viewport_format(target);
        self.views.write(
            &self.device,
            &self.queue,
            self.samples,
            &self.clusters.kept,
            scene,
            &view_plans,
            &uniform,
            &fx_uniform,
            !lights_gpu.is_empty(),
            viewport_format,
            scene
                .lighting_2d
                .map(|l| (l.resolution.clamp(0.25, 1.0), light_slots.count())),
            fluids,
            particle_frame.soft,
            // The weight the frame's tonemap gives its glow (post.rs
            // `write_uniform`): the bloom's intensity when the lit image
            // blooms too, else the glow's own.
            post_fx
                .glow
                .filter(|_| view_glow_runs.iter().any(|r| !r.is_empty()))
                .map(|g| {
                    post_fx
                        .bloom
                        .filter(|b| b.intensity > 0.0)
                        .map_or(g.intensity, |b| b.intensity)
                }),
        );
        if scene.lighting_2d.is_some() {
            for (k, p) in view_plans.iter().enumerate() {
                self.lights_2d
                    .write_view(&self.device, &self.queue, k, p.frame.view_proj);
            }
        }
        // Screen-space GI reads the previous frame's lit image: TAA's
        // history, or without TAA its own copy of the main pass's image.
        let gi_copy = gi.is_some() && !taa;
        if gi_copy
            && self
                .persistent
                .ensure_gi(&self.device, self.width, self.height)
        {
            self.gi_history = None;
        }
        let gi_history_valid = if taa {
            self.taa_history.is_some()
        } else {
            gi_copy && self.gi_history.is_some_and(|f| f + 1 == self.frame_index)
        };
        self.write_frame_post_uniforms(WriteFramePostUniformsInputs {
            cam: &cam,
            camera: &camera,
            post_fx,
            scene,
            ao,
            gi,
            gi_history: gi_history_valid,
            dof,
            motion_blur,
            god_rays,
            sun_ndc,
            god_visible,
            sun_fade,
            depth_bands,
            ssr,
            screen_blur,
        });
        let slot = (self.frame_index % 2) as usize;
        let taa_blend = taa && self.taa_history.is_some();
        self.write_frame_taa_uniform(taa, prepass, &cam);
        // The frame's passes, compiled into the render graph (cached while
        // the pass list stays the same).
        let auto_exposure = matches!(camera.exposure, Exposure::Auto { .. });
        let bloom = post_fx.bloom.is_some_and(|b| b.intensity > 0.0) || glow;
        let gtao = match ao.map(|a| a.method) {
            Some(AoMethod::Gtao) => true,
            Some(AoMethod::Ssao | AoMethod::Screen) => false,
            _ => true,
        };
        let mut passes: Vec<FramePass> =
            (0..cascades.len() as u32).map(FramePass::Cascade).collect();
        if shelter_redraw {
            passes.push(FramePass::Shelter);
        }
        self.shelter_key = shelter.as_ref().map(|s| s.2);
        for ls in &local_shadows {
            for face in 0..ls.faces.len() {
                passes.push(FramePass::LocalFace((ls.layer + face) as u32));
            }
        }
        if prepass {
            passes.push(FramePass::Prepass);
        }
        let ao_resolution = gi.map(|g| g.resolution).or(ao.map(|a| a.resolution));
        if let Some(resolution) = ao_resolution {
            passes.extend([FramePass::Ao, FramePass::AoBlur]);
            if resolution == AoResolution::Half {
                passes.push(FramePass::AoUpsample);
            }
        }
        if particle_frame.any_gpu {
            passes.push(FramePass::Particles);
        }
        if scene.lighting_2d.is_some() {
            passes.push(FramePass::Light2d);
        }
        if fluids > 0 {
            passes.push(FramePass::Metaballs);
        }
        if fluids_3d {
            passes.push(FramePass::Fluid3d);
        }
        if !probe_frame.due.is_empty() {
            passes.push(FramePass::Probes);
        }
        for k in 0..reflections.len() {
            passes.push(FramePass::Reflection(k as u8));
        }
        passes.extend((0..view_plans.len() as u32).map(FramePass::View));
        if volumetric_fog.is_some() && self.volumetric.method == VolumetricMethod::Froxel {
            passes.push(FramePass::Volumetric);
        }
        passes.push(FramePass::Main);
        if ssr.is_some() {
            passes.push(FramePass::Ssr);
        }
        if depth_bands.is_some() {
            passes.push(FramePass::DepthBands);
        }
        if glow {
            passes.push(FramePass::Glow);
        }
        if outline {
            passes.push(FramePass::Outline);
        }
        if taa {
            passes.push(FramePass::Taa);
            if self.post.taa_sharpen > 0.0 {
                passes.push(FramePass::TaaSharpen);
            }
        }
        if dof.is_some() {
            passes.extend([
                FramePass::DofCoc,
                FramePass::DofBlur,
                FramePass::DofComposite,
            ]);
        }
        if motion_blur.is_some() {
            passes.push(FramePass::MotionBlur);
        }
        if screen_blur.is_some() {
            passes.push(FramePass::ScreenBlur);
        }
        passes.extend((0..custom_posts.len() as u32).map(FramePass::CustomPost));
        if god_rays.is_some() {
            passes.extend([FramePass::GodRaysMask, FramePass::GodRaysBlur]);
        }
        if auto_exposure {
            passes.push(FramePass::Meter);
            passes.extend((0..post::meter_levels() - 1).map(FramePass::Reduce));
            if camera.exposure.adaptation().is_some() {
                passes.push(FramePass::Adapt);
            }
        }
        if bloom {
            let (pw, ph) = self.history_size();
            let levels = post::bloom_levels(pw, ph);
            passes.push(FramePass::BloomPrefilter);
            passes.extend((0..levels - 1).map(FramePass::BloomDown));
            passes.extend((0..levels - 1).rev().map(FramePass::BloomUp));
        }
        passes.push(FramePass::Tonemap);
        // The stylisation chain (post.rs `chain_passes`), then FXAA, then
        // the pixel grid scaled up to the output.
        let st = &scene.stylise;
        if st.paint.is_some() {
            passes.push(FramePass::Paint);
        }
        if st.hatching.is_some() || st.halftone.is_some() || st.ink.is_some() {
            passes.push(FramePass::Lines);
        }
        let lens = post_fx.lens_distortion.filter(|l| l.amount != 0.0);
        if lens.is_some() {
            passes.push(FramePass::Lens);
        }
        if st.quantize.is_some() {
            passes.push(FramePass::Quantize);
        }
        if self.info.antialias == Antialias::Fxaa {
            passes.push(FramePass::Fxaa);
        }
        // Under TAAU the frame after TAA is already the output's size.
        if self.history_size() != (self.out_width, self.out_height) {
            passes.push(FramePass::Upscale);
        }
        self.post.ensure_chain(&self.device, &passes);
        // The stylise passes' parameters, palette and canvas.
        let palette_view = st
            .quantize
            .and_then(|q| q.palette)
            .and_then(|p| self.cache.palettes.get(p.index()))
            .and_then(Option::as_ref)
            .unwrap_or(&self.default_white);
        let canvas_view = st
            .paint
            .and_then(|p| p.canvas)
            .and_then(|t| self.cache.textures.get(t.index()))
            .map_or(&self.default_white, |g| &g.view);
        let stylise_key = {
            // The slot's revision too: a palette added in a freed slot is
            // another texture under the same index.
            let p = st.quantize.and_then(|q| q.palette).map_or(u64::MAX, |p| {
                let revision = self.cache.palette_revisions.get(p.index()).copied();
                p.index() as u64 ^ revision.unwrap_or(0).rotate_left(24)
            });
            let c = st
                .paint
                .and_then(|p| p.canvas)
                .map_or(u64::MAX, |t| t.index() as u64);
            scene.id() ^ p.rotate_left(16) ^ c.rotate_left(40)
        };
        self.write_frame_stylise_uniform(st, scene, (&camera, &cam), upscale, grid_offset, lens);
        let lightmap_scale = scene
            .lighting_2d
            .map_or(0.5, |l| l.resolution.clamp(0.25, 1.0));
        // The lights' directions beside the light map, for normal-mapped
        // receivers (sprite_normals.rs).
        let lightdir_on = scene.lighting_2d.is_some() && scene.has_texture_normals();
        // The light map: its strips side by side (light2d.rs).
        let lightmap_size = light2d::strips_size(
            (
                ((self.width as f32 * lightmap_scale) as u32).max(1),
                ((self.height as f32 * lightmap_scale) as u32).max(1),
            ),
            light_slots.count(),
            self.device.limits().max_texture_dimension_2d,
        );
        // Colour adjustments: the grading table and the adjustments baked
        // into one (when either changed), bound under scene id 0, which no
        // scene has, so the bindings are made again when it comes or goes.
        let adjust = post_fx.adjust();
        if let Some(a) = &adjust {
            let grade = post_fx.color_grading.map(|g| {
                let i = g.lut.index();
                (
                    &scene.luts()[i],
                    (scene.id(), i, scene.lut_revisions()[i]),
                    g.intensity,
                )
            });
            self.post
                .bake_adjusted_lut(&self.device, &self.queue, a, grade);
        }
        let (lut_key, lut_view) = match (adjust.and(self.post.adjusted_lut()), lut_index) {
            (Some(view), _) => (Some((0, 0)), view),
            (None, Some(i)) => (
                Some((scene.id(), i)),
                &self.cache.luts[i].as_ref().expect("checked above").0,
            ),
            (None, None) => (None, &self.post.identity_lut),
        };
        // Transmissive draws refract a copy of the opaque scene with its
        // mip chain: the main pass splits for it, as for water.
        let transmission = blended_list
            .ranges
            .iter()
            .any(|r| batches[r.0 as usize].transmissive)
            || skinned_visible.iter().any(|b| b.transmissive);
        if transmission {
            self.mips.prepare(&self.device, HDR_FORMAT);
            let stale = self
                .transmission
                .as_ref()
                .is_none_or(|(t, ..)| (t.width(), t.height()) != (self.width, self.height));
            if stale {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("transmission"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: mips::mip_count(self.width, self.height),
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                let chain = self.mips.chain(&self.device, &texture);
                self.transmission = Some((texture, view, chain));
            }
        }
        let transmission_view = self
            .transmission
            .as_ref()
            .filter(|_| transmission)
            .map(|(_, view, chain)| (view, chain));
        self.graph
            .prepare(
                &self.device,
                passes,
                &post::GraphConfig {
                    width: self.width,
                    height: self.height,
                    post: self.history_size(),
                    samples: self.samples,
                    taa,
                    lightmap: lightmap_size,
                    lightdir: lightdir_on,
                    gtao,
                    gi: gi.is_some(),
                    water: !scene_water.is_empty() || fluids_3d || particle_frame.distortion,
                    water_ao: !scene_water.is_empty() && ao.is_some_and(|a| a.direct > 0.0),
                    transmission,
                    fluid_fields: fluids as u32,
                    reflection: [
                        reflections.first().map(|r| r.4),
                        reflections.get(1).map(|r| r.4),
                    ],
                    object_space,
                    ui_depth: ui_depth && prepass,
                },
                &self.post,
                &self.persistent,
                (lut_key, lut_view),
                (palette_view, canvas_view, stylise_key),
            )
            .map_err(|e| RenderError::Graph(e.to_string()))?;
        // The UI's tables and groups: after the graph, whose view depth
        // occluders and mass elements read.
        if let (Some(of), Some(o), Some(u)) = (&overlay_frame, &scene.overlay, ui_uniform) {
            let textures = &self.cache.textures;
            let backdrop = self.glass_targets.as_ref().filter(|_| glass_on).map(|g| {
                (
                    &g.pyramid_view,
                    &g.behind_view,
                    !self.output_format.is_srgb(),
                )
            });
            let depth = self
                .graph
                .try_view("lindepth")
                .unwrap_or(&self.post.far_depth);
            let scale = overlay::OverlayRenderer::scale(self.out_height, o).max(1e-6);
            let camera = ui::mass::MassCameraGpu {
                view_proj: cam.view_proj.to_cols_array_2d(),
                eye: scene
                    .camera
                    .position
                    .extend(f32::from(u8::from(cam.orthographic)))
                    .to_array(),
                forward: cam.forward.extend(0.0).to_array(),
                css: [
                    self.out_width as f32 / scale,
                    self.out_height as f32 / scale,
                    o.ui.tick_fraction.clamp(0.0, 1.0),
                    0.0,
                ],
                tick: [o.ui.tick, 0, 0, 0],
            };
            self.ui.upload(
                &self.device,
                &self.queue,
                &self.overlay.layout,
                scene,
                of,
                u,
                backdrop,
                |t| textures.get(t.index()).map(|g| g.view.clone()),
                depth,
                camera,
                &self.mips,
            );
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
        let output: &wgpu::TextureView = match target {
            RenderTarget::Offscreen => &self.persistent.offscreen_view,
            RenderTarget::View(v) => v,
            RenderTarget::Surface => &surface_frame.as_ref().expect("acquired above").1,
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

        let cluster_stats = &self.clusters.stats;
        let mut stats = FrameStats {
            instance_upload_bytes,
            terrain_tiles: self.terrain.resident,
            terrain_uploads: self.terrain.uploads,
            scatter_built: self.scatter.built,
            batches: batches.len() as u32,
            instances: scene.instances.len() as u32,
            culled,
            triangles: main_list.triangles + blended_list.triangles,
            in_view_triangles: main_list.in_view_triangles,
            lights: self.clusters.kept.len() as u32,
            shadowed_lights: local_shadows.len() as u32,
            shadowed_lights_dropped: shadowed_dropped,
            lights_dropped: cluster_stats.dropped,
            lights_culled: cluster_stats.culled,
            cluster_lights: cluster_stats.refs,
            cluster_lights_dropped: cluster_stats.refs_dropped,
            texture_bytes: self.cache.textures.iter().map(|t| t.bytes).sum::<u64>()
                + self.asset_bytes(),
            sprites: sprite_frame.sprites,
            sprites_culled: sprite_frame.culled,
            tiles: sprite_frame.tiles,
            tile_chunks: sprite_frame.tile_chunks,
            glyphs: sprite_frame.glyphs,
            sprite_draws: (sprite_frame.runs.len()
                + sprite_frame.billboard_shadows.len()
                + sprites::draw_runs_of(&sprite_frame.billboards).len())
                as u32,
            missing_glyphs: sprite_frame.missing_glyphs,
            transparent: blended_list.instances,
            transparent_draws: blended_list.ranges.len() as u32,
            graph_passes: self.graph.order().len() as u32,
            transient_textures: self.graph.transient_textures() as u32,
            frame: self.frame_index,
            particle_systems: scene.particles.len() as u32,
            particle_capacity: particle_frame.capacity,
            particles_cpu: particle_frame.cpu_alive,
            particle_ticks: particle_frame.ticks,
            particle_collision_skipped: particle_frame.collision_skipped,
            strips: ribbon_frame.strips,
            strip_segments: ribbon_frame.segments,
            line_instances: line_frame.instances,
            decals,
            decals_dropped,
            lights_2d,
            occluders_2d,
            skinned_instances: scene.skinned_instances.len() as u32,
            joints: joints_drawn,
            taa_history: taa_blend,
            ..Default::default()
        };
        if let Some((lagged, occluder_draws)) = gpu_counts {
            let c = lagged.unwrap_or_default();
            stats.gpu_culling = true;
            stats.culled += c.culled;
            stats.occluded = c.occluded;
            stats.occluded_triangles = c.occluded_triangles;
            stats.in_view_triangles = c.triangles + c.occluded_triangles;
            stats.triangles += c.triangles;
            stats.shadow_casters += c.shadow_casters;
            stats.counts_lag = lagged.map_or(u32::MAX, |l| (self.frame_index - l.frame) as u32);
            stats.occluder_draws = occluder_draws;
            stats.draw_calls += occluder_draws;
        }
        if let Some(of) = &overlay_frame {
            stats.overlay_sprites = of.sprites;
            stats.overlay_glyphs = of.glyphs;
            stats.overlay_draws = of.runs.len() as u32;
            stats.sprites += of.sprites;
            stats.glyphs += of.glyphs;
            stats.sprite_draws += of.runs.len() as u32;
            stats.missing_glyphs += of.missing_glyphs;
        }
        for b in &skinned_visible {
            stats.triangles += (self.cache.meshes[b.mesh].index_count / 3) as u64 * b.count as u64;
        }

        // Per-frame bind groups of the effects.
        // The effect bind groups, rebuilt only when what they read changed.
        let fx_key = (
            self.graph.generation,
            scene.id(),
            scene.decal_atlas,
            self.decal_buffers.generation,
            ao_target,
            scene.lighting_2d.is_some(),
            reflection.is_some(),
        );
        if self.fx_groups.as_ref().map(|g| g.key) != Some(fx_key) {
            let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                &self.cache.textures[t.index()].view
            });
            let ao_view = if ao_target {
                self.graph.view("ao")
            } else {
                &self.post.white
            };
            // This frame's planar reflection, for the material shaders that
            // read it (pbr.wgsl `planar_reflection`; fx `reserved` says
            // whether it is there).
            let planar_view = if reflection.is_some() {
                self.graph.view("reflection")
            } else {
                &self.post.white
            };
            let lightmap_view = if scene.lighting_2d.is_some() {
                self.graph.view("lightmap")
            } else {
                &self.default_white
            };
            // The lights' directions (normal-mapped receivers), black
            // (none) without.
            let lightdir_view = self.graph.try_view("lightdir").unwrap_or(&self.post.black);
            let sprite_group = |light: &wgpu::TextureView, dir, label: &str| {
                bind_group(
                    &self.device,
                    label,
                    &self.sprite_fx_layout,
                    &[
                        wgpu::BindingResource::TextureView(light),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        self.sprite_fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(dir),
                    ],
                )
            };
            let groups = FxGroups {
                key: fx_key,
                fx: fx::fx_group(
                    &self.device,
                    "fx",
                    &self.fx_layout,
                    &[
                        self.fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(ao_view),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        wgpu::BindingResource::TextureView(planar_view),
                    ][..if self.planar_slot() { 5 } else { 4 }],
                    &self.decal_buffers,
                ),
                fx_lean: self.fx_lean_group(&self.fx_buffer, ao_view, atlas_view),
                sprite: sprite_group(lightmap_view, lightdir_view, "sprite fx"),
                // The glow pass draws into a target that may alias the light
                // map: its sprites read a placeholder instead.
                glow: sprite_group(&self.default_white, &self.post.black, "sprite fx (glow)"),
            };
            self.fx_groups = Some(groups);
        }
        let (_, frame_bind_group, sky_group, reflection_frame_group) = self
            .environment
            .as_ref()
            .expect("environment generated above");
        // The probes' frame groups (`probes.rs`): per active probe, the
        // environment's prefiltered cube and the probe's in place of the
        // irradiance and prefiltered cubes; per due capture face, the
        // frame's own cubes under the face camera's uniform.
        let (probe_groups, face_groups) = {
            let ibl = self.ibl.views();
            let group = |k: usize, a: &wgpu::TextureView, b: &wgpu::TextureView| {
                bind_group(
                    &self.device,
                    "frame (reflection probe)",
                    &self.frame_layout,
                    &[
                        self.probes.binding(k),
                        wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
                        wgpu::BindingResource::TextureView(&self.local_map.array_view),
                        wgpu::BindingResource::TextureView(a),
                        wgpu::BindingResource::TextureView(b),
                        wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                        wgpu::BindingResource::Sampler(&ibl.sampler),
                        self.light_buffers[0].as_entire_binding(),
                        self.light_buffers[1].as_entire_binding(),
                        self.light_buffers[2].as_entire_binding(),
                        self.lightmaps.probe_buffer.as_entire_binding(),
                        self.ltc.as_entire_binding(),
                    ],
                )
            };
            let probe_groups: Vec<wgpu::BindGroup> = probe_frame
                .active
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
                    group(k, &ibl.prefiltered, view)
                })
                .collect();
            let first = probe_frame.active.len();
            let face_groups: Vec<wgpu::BindGroup> = (0..6 * probe_frame.due.len())
                .map(|j| group(first + j, &ibl.irradiance, &ibl.prefiltered))
                .collect();
            (probe_groups, face_groups)
        };
        let probe_binding = (!probe_groups.is_empty()).then_some(probe_select::ProbeGroups {
            boxes: &probe_boxes,
            groups: &probe_groups,
            frame: frame_bind_group,
        });
        let groups = self.fx_groups.as_ref().expect("made above");
        let (fx_group, sprite_fx_group, glow_fx_group) = (&groups.fx, &groups.sprite, &groups.glow);
        // The physical pipelines' effect group: the same plus the scene copy
        // (a placeholder in a frame without transmission).
        let physical_fx_group = self.physical.as_ref().and_then(|p| {
            batches
                .iter()
                .chain(&skinned_visible)
                .any(|b| b.physical)
                .then(|| {
                    let atlas_view = scene.decal_atlas.map_or(&self.default_white, |t| {
                        &self.cache.textures[t.index()].view
                    });
                    let ao_view = if ao_target {
                        self.graph.view("ao")
                    } else {
                        &self.post.white
                    };
                    fx::fx_group(
                        &self.device,
                        "physical fx",
                        &p.fx_layout,
                        &[
                            self.fx_buffer.as_entire_binding(),
                            wgpu::BindingResource::TextureView(ao_view),
                            wgpu::BindingResource::TextureView(atlas_view),
                            wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                            wgpu::BindingResource::TextureView(
                                transmission_view.map_or(&self.post.white, |(v, _)| v),
                            ),
                        ],
                        &self.decal_buffers,
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
            self.particles.draw_group(&self.device, scene_depth, copy)
        });
        let metaball_groups: Vec<wgpu::BindGroup> = post::FIELD_NAMES
            .iter()
            .map_while(|name| self.graph.try_view(name))
            .map(|v| self.metaballs.group(&self.device, v))
            .collect();
        let reflection_views: Vec<(usize, &wgpu::TextureView)> = reflections
            .iter()
            .enumerate()
            .filter_map(|(k, r)| {
                Some((
                    r.0,
                    self.graph.try_view(post::reflection_target(k as u8, ""))?,
                ))
            })
            .collect();
        let water_draws = match self.graph.try_view("scene-copy") {
            Some(copy) if !scene_water.is_empty() => self.water.prepare(
                &self.device,
                &self.queue,
                scene_water,
                (self.width, self.height),
                copy,
                scene_depth,
                &reflection_views,
                if ao_target {
                    self.graph.view("ao")
                } else {
                    &self.post.white
                },
                ao.map_or(0.0, |a| a.direct),
                scene.origin,
                water::seconds(scene),
                &self.post.white,
                &|h| {
                    self.cache
                        .meshes
                        .get(h.index())
                        .filter(|m| m.full)
                        .map(|m| (m.vertices.clone(), m.indices.clone(), m.index_count))
                },
                &|h| self.cache.textures.get(h.index()).map(|t| &t.view),
            ),
            _ => Vec::new(),
        };
        // The water as each due capture's faces draw it, through the face's
        // copy and over its depth (water.rs `face_draws`).
        let face_water: Vec<Vec<water::WaterDraw>> = if water_draws.is_empty() {
            Vec::new()
        } else {
            probe_frame
                .due
                .iter()
                .map(|&(slot, _)| {
                    let t = self.probes.targets_of(slot);
                    self.water.face_draws(
                        &self.device,
                        scene_water,
                        t.size,
                        t.copy.as_ref().expect("made with the water"),
                        &t.prepass.lindepth,
                        &self.post.white,
                        &|h| self.cache.textures.get(h.index()).map(|t| &t.view),
                    )
                })
                .collect()
        };
        // The 3D fluids' bounding sphere: the reflection probe their one
        // composite draw samples.
        let fluid_bounds = instances::fold_box(
            None,
            scene
                .fluids_3d
                .iter()
                .flat_map(|f| f.particles.iter().map(move |&p| (p, f.radius))),
        )
        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo).length() * 0.5));
        let fluid_frame = match fluid3d::views(&self.graph, scene_depth) {
            Some(views) if fluids_3d => self.fluid3d.prepare(
                &self.device,
                &self.queue,
                &scene.fluids_3d,
                &fluid3d::Fluid3dCamera {
                    view: cam.view,
                    proj: cam.proj,
                    orthographic: cam.orthographic,
                },
                (self.width, self.height),
                &views,
            ),
            _ => None,
        };
        // The 3D fluids as each due capture face draws them: the frame's
        // particles through the face camera into the face's own targets,
        // over its depth and through its copy (fluid3d.rs `camera_frame`).
        let face_fluids: Vec<Option<fluid3d::Fluid3dFrame>> = match &fluid_frame {
            Some(main) if face_fluid => probe_frame
                .due
                .iter()
                .enumerate()
                .flat_map(|(d, &(slot, _))| {
                    let t = self.probes.targets_of(slot);
                    probe_faces[d].iter().map(move |f| (t, f))
                })
                .map(|(t, f)| {
                    let views = t.fluid.as_ref().expect("made with the fluids").views(
                        &t.prepass.lindepth,
                        t.copy.as_ref().expect("made with the fluids"),
                    );
                    self.fluid3d.camera_frame(
                        &self.device,
                        &scene.fluids_3d,
                        main,
                        &fluid3d::Fluid3dCamera {
                            view: f.view,
                            proj: f.proj,
                            orthographic: f.orthographic,
                        },
                        (t.size, t.size),
                        &views,
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
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
                    &self.decal_buffers,
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
        let fx_lean_of = |fx: &wgpu::BindGroup| {
            fx_leans
                .iter()
                .find(|(f, _)| std::ptr::eq(*f, fx))
                .map(|(_, l)| *l)
        };
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
                    self.particles.draw_group_of(
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
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(label),
                            contents,
                            usage: wgpu::BufferUsages::UNIFORM,
                        })
                };
                let pb = buffer("probe face particles", bytemuck::bytes_of(&particles));
                let lb = buffer("probe face lines", bytemuck::bytes_of(&lines));
                (
                    (!particle_frame.systems.is_empty()).then(|| {
                        self.particles
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
        // The queries this frame's passes write their timestamps into
        // (`timing.rs`): the encoder puts a pair around every pass.
        let gpu_set = match (&mut self.gpu_timer, self.clock) {
            (Some(t), Some(_)) => t.begin(&self.device, &self.queue),
            _ => None,
        };
        let mut encoder = Encoder::new(
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame"),
                }),
            gpu_set,
        );
        let palette_group = self.palette.bind_group.as_ref();
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
        let shadow_skinned: Vec<&wgpu::RenderPipeline> = skinned_batches
            .iter()
            .map(|b| match b.shader {
                Some(sh) => &self.cache.hook_shadow_pipelines[&(sh, b.side, true)],
                None => &self.shadow_pipelines[&(b.side, true, b.dissolve)],
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
                        .array_maps
                        .as_ref()
                        .filter(|_| b.array)
                        .and_then(|a| a.prepass.get(&(b.side, true)));
                    hooked
                        .or(array)
                        .or_else(|| self.prepass_pipelines.get(&(b.side, true)))
                        .unwrap_or(lit_skinned[0])
                })
                .collect()
        } else {
            Vec::new()
        };
        let culled_gpu = self.gpu_cull.as_ref().filter(|_| gpu_cull);
        // The effect groups of the standard and the physical pipelines, in
        // the passes that draw physical materials with theirs.
        let physical_fx = physical_fx_group.as_ref().map(|p| (fx_group, p));
        // Draws a list's ranges. Lit passes switch the pipeline when the
        // batch's (side, blending, effects, shader, physical, lean) changes and
        // bind each batch's material (and, between standard and physical,
        // the effect group: `physical_fx`); shadow passes switch on the
        // side and dissolve (with the material).
        let draw_list = |pass: &mut wgpu::RenderPass,
                         list: &DrawList,
                         lit: bool,
                         current: &mut Option<LitDrawKey>,
                         physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
                         probes: Option<probe_select::ProbeGroups>,
                         fx: Option<&wgpu::BindGroup>| {
            self.frame_draw_list(DrawListInputs {
                pass,
                list,
                lit,
                current,
                physical_fx,
                probes,
                fx,
                batches: &batches,
                culled_gpu,
                fx_lean_of: &fx_lean_of,
            })
        };
        // The skinned batches (`transparent`: which of them) with the
        // pipelines `pipeline` picks; materials at group `material_group`.
        // `physical_fx` as in draw_list: the main pass's physical batches
        // bind their physical groups (their pipelines are in `pipelines`).
        let draw_skinned = |pass: &mut wgpu::RenderPass,
                            list: &[Batch],
                            transparent: bool,
                            material_group: u32,
                            palette_at: u32,
                            pipelines: &[&wgpu::RenderPipeline],
                            physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
                            probes: Option<probe_select::ProbeGroups>|
         -> u32 {
            self.frame_draw_skinned(DrawSkinnedInputs {
                pass,
                list,
                transparent,
                material_group,
                palette_at,
                pipelines,
                physical_fx,
                probes,
                palette_group,
            })
        };
        // Glowing sprites into a glow buffer (group 0 set: the camera's).
        let draw_glow = |pass: &mut wgpu::RenderPass, runs: &[sprites::SpriteRun]| -> u32 {
            self.frame_draw_glow(pass, runs, glow_fx_group)
        };
        // Trails, ribbons and beams drawn with the 3D scene (layered 2D
        // trails go with the sprites): the main camera's or a view's runs.
        let draw_strips = |pass: &mut wgpu::RenderPass, runs: &[ribbons::RibbonRun]| -> u32 {
            self.frame_draw_strips(pass, runs)
        };
        // Sprites and text over a camera's scene, in sorted order: the main
        // camera's runs with its effect group and metaball fields, or a
        // view's (`view`: every run tone mapped, no HDR alpha cover).
        // Answers the draws made.
        let draw_sprites =
            |pass: &mut wgpu::RenderPass,
             runs: &[sprites::SpriteRun],
             fx: &wgpu::BindGroup,
             fields: &[wgpu::BindGroup],
             view: bool|
             -> u32 { self.frame_draw_sprites(pass, runs, fx, fields, view, scene) };
        // The transparent meshes of `list` and the billboards sorted among
        // them (`sprites::SpriteFrame::billboards`): the billboards' drop
        // shadows first, then each billboard before the first run nearer
        // than it (`DrawList::before`), the rest after the last. `fx`: the
        // pass's effects group and its sprites' (each pass's group 2).
        // Returns the draw calls.
        #[allow(clippy::type_complexity)]
        let draw_blended =
            |pass: &mut wgpu::RenderPass,
             list: &DrawList,
             (boards, shadows): (&[(f32, sprites::SpriteRun)], &[sprites::SpriteRun]),
             (fx, sprite_fx): (&wgpu::BindGroup, &wgpu::BindGroup),
             physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
             probes: Option<probe_select::ProbeGroups>,
             view: bool|
             -> u32 {
                if boards.is_empty() && shadows.is_empty() {
                    draw_list(pass, list, true, &mut None, physical_fx, probes, Some(fx));
                    return list.ranges.len() as u32;
                }
                let billboards = |pass: &mut wgpu::RenderPass, runs: &[sprites::SpriteRun]| {
                    let n = draw_sprites(pass, runs, sprite_fx, &[], view);
                    pass.set_bind_group(2, fx, &[]);
                    n
                };
                let meshes = |pass: &mut wgpu::RenderPass, range: std::ops::Range<usize>| {
                    let part = DrawList {
                        ranges: list.ranges[range.clone()].to_vec(),
                        ..DrawList::default()
                    };
                    draw_list(pass, &part, true, &mut None, physical_fx, probes, Some(fx));
                    range.len() as u32
                };
                let mut draws = billboards(pass, shadows);
                let (mut drawn, mut start) = (0usize, 0usize);
                for r in 0..=list.ranges.len() {
                    let before = list.before.get(r).map_or(boards.len(), |&b| b as usize);
                    if before > drawn {
                        draws += meshes(pass, start..r);
                        draws += billboards(pass, &sprites::draw_runs_of(&boards[drawn..before]));
                        (drawn, start) = (before, r);
                    }
                }
                draws + meshes(pass, start..list.ranges.len())
            };
        let terrain_indices = self.terrain.index_buffer();
        // The depth prepass of a camera other than the main one (a view's,
        // a probe face's: views.rs, probes.rs): its opaque `list`, its
        // opaque skinned batches, the terrain in `frustum` and the
        // scattered copies `scatter` keeps, through the main prepass's
        // pipelines (group 0 and 2: `groups`) into `pre`. Returns the draws.
        let depth_prepass = |encoder: &mut crate::Encoder,
                             label: &str,
                             pre: &views::DepthPrepass,
                             groups: (&wgpu::BindGroup, &wgpu::BindGroup),
                             list: &DrawList,
                             skinned: &[Batch],
                             frustum: &Frustum,
                             scatter: Option<&dyn Fn(Vec3, f32) -> bool>|
         -> u32 {
            let attachment = |view, clear: f64| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[
                    attachment(&pre.gbuffer, 0.0),
                    attachment(&pre.lindepth, post::FAR_DEPTH as f64),
                    attachment(&pre.outline, 0.0),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &pre.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, groups.0, &[]);
            pass.set_bind_group(2, groups.1, &[]);
            let mut draws = 0;
            for &(b, first, count) in &list.ranges {
                let batch = &batches[b as usize];
                let Some(pipeline) = self.prepass_pipeline_of(batch, false) else {
                    continue;
                };
                pass.set_pipeline(pipeline);
                let m = &self.cache.materials[batch.material];
                pass.set_bind_group(1, m.array.as_ref().unwrap_or(&m.bind_group), &[]);
                let mesh = &self.cache.meshes[batch.mesh];
                let start = first as u64 * INSTANCE_STRIDE;
                let range = start..start + count as u64 * INSTANCE_STRIDE;
                pass.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.slice(range.clone()));
                pass.set_vertex_buffer(2, self.instance_buffer.slice(range));
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..count);
                draws += 1;
            }
            let opaque: Vec<Batch> = skinned
                .iter()
                .filter(|b| !b.transparent && self.prepass_pipeline_of(b, true).is_some())
                .map(|b| Batch { ..*b })
                .collect();
            let pipelines: Vec<&wgpu::RenderPipeline> = opaque
                .iter()
                .filter_map(|b| self.prepass_pipeline_of(b, true))
                .collect();
            draws += draw_skinned(&mut pass, &opaque, false, 1, 3, &pipelines, None, None);
            if let Some(indices) = terrain_indices {
                draws += terrain::draw_levels(
                    &mut pass,
                    &terrain_draws,
                    indices,
                    terrain::TerrainPass::Prepass,
                    |c, rad| frustum.intersects_sphere(c, rad),
                    None,
                )
                .0;
            }
            if let Some(keep) = scatter {
                draws += self
                    .scatter
                    .draw(
                        &mut pass,
                        scatter::ScatterPass::Prepass,
                        &scatter_batches,
                        scatter_material,
                        scatter_mesh,
                        keep,
                        None,
                        None,
                    )
                    .0;
            }
            draws
        };
        // One shadow pass: a cascade or a local light's face.
        let shadow_pass = |encoder: &mut crate::Encoder,
                           color: &wgpu::TextureView,
                           depth: &wgpu::TextureView,
                           slot: usize,
                           list: &DrawList,
                           frustum: &Frustum,
                           view: u32,
                           stats: &mut FrameStats,
                           // The weather occlusion map takes only what stands
                           // still for its key (weather::shelter_key): no
                           // skinned casters, no scattered copies (grass must
                           // not keep the ground under it dry).
                           all_casters: bool| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Depth 1 and distance / range 1: nothing nearer.
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipelines[&(Side::Front, false, false)]);
            pass.set_bind_group(
                0,
                &self.shadow_bind_group,
                &[(slot as u64 * SHADOW_SLOT_STRIDE) as u32],
            );
            draw_list(&mut pass, list, false, &mut None, None, None, None);
            stats.draw_calls += list.ranges.len() as u32;
            stats.shadow_casters += list.instances;
            stats.shadow_passes += 1;
            // The clustered casters (meshlet_gpu.rs), each one draw of the
            // clusters the pass's view kept.
            if view > 0
                && let Some(m) = self.meshlet.as_ref()
            {
                stats.draw_calls += m.draw_shadow(&mut pass, &batches, view, |b| {
                    &self.cache.materials[b.material].bind_group
                });
            }
            if !all_casters {
                if let Some(indices) = terrain_indices
                    && terrain_casters
                {
                    let (n, _) = terrain::draw_levels(
                        &mut pass,
                        &terrain_draws,
                        indices,
                        terrain::TerrainPass::Shadow,
                        |c, r| frustum.intersects_sphere(c, r),
                        None,
                    );
                    stats.draw_calls += n;
                    stats.terrain_shadow_draws += n;
                }
                return;
            }
            // Skinned casters, unculled.
            let n = draw_skinned(
                &mut pass,
                &skinned_batches,
                false,
                1,
                2,
                &shadow_skinned,
                None,
                None,
            );
            stats.draw_calls += n;
            stats.shadow_casters += skinned_batches
                .iter()
                .filter(|b| !b.transparent && b.cast)
                .map(|b| b.count)
                .sum::<u32>();
            // Terrain levels that reach into the pass's volume.
            if let Some(indices) = terrain_indices
                && terrain_casters
            {
                let shadow = terrain::TerrainPass::Shadow;
                let (n, _) = terrain::draw_levels(
                    &mut pass,
                    &terrain_draws,
                    indices,
                    shadow,
                    |c, r| frustum.intersects_sphere(c, r),
                    None,
                );
                stats.draw_calls += n;
                stats.terrain_shadow_draws += n;
            }
            let (n, _) = self.scatter.draw(
                &mut pass,
                scatter::ScatterPass::Shadow,
                &scatter_batches,
                scatter_material,
                scatter_mesh,
                |c, r| frustum.intersects_sphere(c, r),
                None,
                None,
            );
            stats.draw_calls += n;
        };
        // Per-pass CPU encode time, with a clock (`Renderer::set_clock`),
        // and GPU time where the device has timestamps (`timing.rs`): each
        // graph pass owns the queries of the render and compute passes it
        // opened.
        let clock = self.clock;
        let mut pass_ms: Vec<PassTiming> = Vec::new();
        let mut pass_queries: Vec<(String, std::ops::Range<u32>)> = Vec::new();
        for &pass in self.graph.order() {
            let t0 = clock.map(|c| c());
            let q0 = encoder.used();
            match pass {
                FramePass::Cascade(i) => shadow_pass(
                    &mut encoder,
                    &self.cascade_map.layers[i as usize],
                    &self.cascade_map.depth,
                    i as usize,
                    &cascade_lists[i as usize],
                    &cascades[i as usize].frustum,
                    cascade_views.get(i as usize).copied().unwrap_or(0),
                    &mut stats,
                    true,
                ),
                FramePass::Shelter => {
                    let s = shelter.as_ref().expect("a shelter pass has its view");
                    shadow_pass(
                        &mut encoder,
                        self.cascade_map.layers.last().expect("fit_shelter_layer"),
                        &self.cascade_map.depth,
                        SHELTER_SLOT,
                        shelter_list.as_ref().expect("a list when redrawn"),
                        &s.1,
                        shelter_view,
                        &mut stats,
                        false,
                    )
                }
                FramePass::LocalFace(layer) => shadow_pass(
                    &mut encoder,
                    &self.local_map.layers[layer as usize],
                    &self.local_map.depth,
                    CASCADES + layer as usize,
                    local_lists[layer as usize]
                        .as_ref()
                        .expect("a list per shadowed face"),
                    local_frustums[layer as usize]
                        .as_ref()
                        .expect("a frustum per shadowed face"),
                    local_views[layer as usize],
                    &mut stats,
                    true,
                ),
                FramePass::Prepass => self.encode_prepass(PrepassPassInputs {
                    water_draws: &water_draws,
                    encoder: &mut encoder,
                    object_space,
                    frame_bind_group,
                    fx_group,
                    main_list: &main_list,
                    culled_gpu,
                    object_motion,
                    batches: &batches,
                    stats: &mut stats,
                    draw_skinned: &draw_skinned,
                    skinned_visible: &skinned_visible,
                    prepass_skinned: &prepass_skinned,
                    terrain_indices,
                    terrain_draws: &terrain_draws,
                    view_frustum: &view_frustum,
                    scatter_batches: &scatter_batches,
                    scatter_material: &scatter_material,
                    scatter_mesh: &scatter_mesh,
                }),
                FramePass::Particles => {
                    self.particles.simulate(
                        &self.device,
                        &mut encoder,
                        &particle_frame,
                        scene_depth,
                        self.graph.try_view("gbuffer").unwrap_or(&self.post.white),
                    );
                }
                FramePass::Metaballs => {
                    let fields: Vec<&wgpu::TextureView> = post::FIELD_NAMES
                        .iter()
                        .map_while(|name| self.graph.try_view(name))
                        .collect();
                    self.metaballs
                        .encode_field(&mut encoder, &fields, frame_bind_group);
                    stats.draw_calls += fields.len() as u32;
                }
                FramePass::Fluid3d => {
                    if let (Some(f), Some(views)) =
                        (&fluid_frame, fluid3d::views(&self.graph, scene_depth))
                    {
                        stats.draw_calls +=
                            self.fluid3d
                                .encode(&mut encoder, f, frame_bind_group, &views);
                    }
                }
                FramePass::Light2d => {
                    let ambient = scene.lighting_2d.map_or(Vec3::ONE, |l| l.ambient);
                    self.lights_2d.encode(
                        &mut encoder,
                        self.graph.view("lightmap"),
                        self.graph.try_view("lightdir"),
                        lightmap_size,
                        ambient,
                    );
                    stats.draw_calls += (lights_2d > 0) as u32;
                }
                FramePass::Probes => self.encode_probes(ProbesPassInputs {
                    meshlet_views: &probe_views,
                    batches: &batches,
                    scene,
                    probe_frame: &probe_frame,
                    face_depth,
                    face_water: &face_water,
                    face_fluids: &face_fluids,
                    probe_faces: &probe_faces,
                    active_probes: &active_probes,
                    stats: &mut stats,
                    depth_prepass: &depth_prepass,
                    encoder: &mut encoder,
                    face_groups: &face_groups,
                    fx_group,
                    probe_lists: &probe_lists,
                    probe_skinned: &probe_skinned,
                    draw_list: &draw_list,
                    draw_skinned: &draw_skinned,
                    terrain_indices,
                    terrain_draws: &terrain_draws,
                    scatter_batches: &scatter_batches,
                    scatter_material: &scatter_material,
                    scatter_mesh: &scatter_mesh,
                    sky_group,
                    draw_blended: &draw_blended,
                    probe_blended: &probe_blended,
                    face_boards: &face_boards,
                    glow_fx_group,
                    face_particle_groups: &face_particle_groups,
                    particle_frame: &particle_frame,
                    line_frame: &line_frame,
                    face_line_groups: &face_line_groups,
                    draw_sprites: &draw_sprites,
                    face_sprites: &face_sprites,
                }),
                FramePass::Reflection(k) => self.encode_reflection(ReflectionPassInputs {
                    meshlet_views: &reflection_meshlets,
                    batches: &batches,
                    k,
                    scene,
                    encoder: &mut encoder,
                    reflection_frame_group,
                    reflection_fx_group: &reflection_fx_group,
                    reflection_list: &reflection_list,
                    draw_list: &draw_list,
                    stats: &mut stats,
                    draw_skinned: &draw_skinned,
                    skinned_visible: &skinned_visible,
                    lit_skinned_standard: &lit_skinned_standard,
                    terrain_indices,
                    reflections: &reflections,
                    terrain_draws: &terrain_draws,
                    sky_group,
                    reflection_blended: &reflection_blended,
                }),
                FramePass::View(i) => self.encode_view(ViewPassInputs {
                    meshlet_views: &view_views,
                    batches: &batches,
                    i,
                    scene,
                    view_plans: &view_plans,
                    encoder: &mut encoder,
                    stats: &mut stats,
                    lights_2d,
                    view_groups: &view_groups,
                    depth_prepass: &depth_prepass,
                    view_lists: &view_lists,
                    view_skinned: &view_skinned,
                    draw_list: &draw_list,
                    draw_skinned: &draw_skinned,
                    terrain_indices,
                    terrain_draws: &terrain_draws,
                    sky_group,
                    draw_blended: &draw_blended,
                    view_blended: &view_blended,
                    view_boards: &view_boards,
                    view_sprite_fx: &view_sprite_fx,
                    view_particle_groups: &view_particle_groups,
                    particle_frame: &particle_frame,
                    draw_strips: &draw_strips,
                    view_ribbons: &view_ribbons,
                    line_frame: &line_frame,
                    view_line_groups: &view_line_groups,
                    draw_sprites: &draw_sprites,
                    view_sprites: &view_sprites,
                    view_fields: &view_fields,
                    draw_glow: &draw_glow,
                    view_glow_runs: &view_glow_runs,
                    post_fx,
                }),
                FramePass::Volumetric => self.volumetric.encode_froxels(
                    &self.device,
                    &mut encoder,
                    &self.frame_buffer,
                    &self.cascade_map.array_view,
                    &self.local_map.array_view,
                    [
                        &self.light_buffers[0],
                        &self.light_buffers[1],
                        &self.light_buffers[2],
                    ],
                ),
                FramePass::Main => {
                    self.encode_main(MainPassInputs {
                        scene,
                        water_draws: &water_draws,
                        fluid_frame: &fluid_frame,
                        transmission_view,
                        taa,
                        encoder: &mut encoder,
                        frame_bind_group,
                        fx_group,
                        draw_list: &draw_list,
                        main_list: &main_list,
                        batches: &batches,
                        physical_fx,
                        probe_binding,
                        stats: &mut stats,
                        draw_skinned: &draw_skinned,
                        skinned_visible: &skinned_visible,
                        lit_skinned: &lit_skinned,
                        terrain_indices,
                        terrain_draws: &terrain_draws,
                        view_frustum: &view_frustum,
                        scatter_batches: &scatter_batches,
                        scatter_material: &scatter_material,
                        scatter_mesh: &scatter_mesh,
                        sky_group,
                        fluid_bounds,
                        draw_blended: &draw_blended,
                        blended_list: &blended_list,
                        sprite_frame: &sprite_frame,
                        sprite_fx_group,
                        particle_group: &particle_group,
                        particle_frame: &particle_frame,
                        draw_strips: &draw_strips,
                        ribbon_frame: &ribbon_frame,
                        line_frame: &line_frame,
                        draw_sprites: &draw_sprites,
                        metaball_groups: &metaball_groups,
                        volumetric_fog,
                    });
                    // The lit image for the next frame's GI (`gi-write`).
                    if let (true, Some((_, copies))) = (gi_copy, &self.persistent.gi) {
                        encoder.copy_texture_to_texture(
                            self.graph.view("hdr").texture().as_image_copy(),
                            copies[slot].texture().as_image_copy(),
                            self.graph.view("hdr").texture().size(),
                        );
                    }
                }
                FramePass::Glow => {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("glow"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: self.graph.view("glow"),
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_bind_group(0, frame_bind_group, &[]);
                    stats.draw_calls += draw_glow(&mut pass, &sprite_frame.glow_runs);
                }
                FramePass::CustomPost(i) => {
                    let (read, write) = self.graph.custom_post_views(&self.persistent, i, slot);
                    let lit = self.graph.custom_post_views(&self.persistent, 0, slot).0;
                    stats.draw_calls += self.custom_posts.encode(
                        &self.device,
                        &self.queue,
                        &mut encoder,
                        scene,
                        custom_posts[i as usize],
                        [
                            read,
                            lit,
                            self.graph.view("lindepth"),
                            self.graph.view("gbuffer"),
                            write,
                        ],
                        self.history_size(),
                    );
                }
                post_pass => {
                    let post_output = match (&self.glass_targets, glass_on) {
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
        for &(slot, _) in &probe_frame.due {
            self.probes.slots[slot].captured = Some(scene.tick);
        }
        // The passes after the graph's are timed as its own are, each listed
        // only in a frame where it drew: the viewport views, the UI layers,
        // and the overlay with the glass and the lines drawn among it, so
        // the UI's budget (`docs/ui.md` §15.1) reads from `gpuPasses`.
        let mut timings = Timings {
            clock,
            pass_ms,
            pass_queries,
        };
        let views_at = timings.begin(&encoder, &stats);
        // Viewport views (split-screen, views.rs) over the finished frame,
        // under the lines and the overlay.
        for p in view_plans
            .iter()
            .filter(|p| matches!(p.dest, views::Dest::Viewport(_)))
        {
            let dest = overlay_view.unwrap_or(output);
            self.views.resolve(
                &self.device,
                &self.queue,
                &mut encoder,
                p,
                post_fx.tonemap,
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
        // the source, copied before it to what it reads.
        let runs: &[overlay::OverlayRun] = overlay_frame.as_ref().map_or(&[], |f| f.main_runs());
        let decode = !self.output_format.is_srgb();
        let mut rebuilds = Vec::new();
        let mut copies = Vec::new();
        let mut layered = None;
        if let (Some(g), true) = (&self.glass_targets, glass_on) {
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
            let over_content = rebuilds.first().is_some_and(|&i| i > 0)
                || rebuilds.len() > 1
                || !copies.is_empty();
            // The overlay draws into the source through the view it blends in.
            layered = match (over_content, overlay_view) {
                (false, _) => None,
                (true, None) => Some(&g.source_view),
                (true, Some(_)) => g.source_linear_view.as_ref(),
            };
            if layered.is_none() {
                stats.draw_calls += self
                    .glass
                    .encode(&self.queue, &mut encoder, g, output, decode);
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
                    !srgb_view && !self.output_format.is_srgb(),
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
            let mut cuts: Vec<usize> = events.iter().map(|e| e.0).collect();
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
                }
                from = to;
            }
        }
        if let (Some(g), Some(_)) = (&self.glass_targets, layered) {
            stats.draw_calls += self.glass.blit(&mut encoder, g, output);
        }
        timings.end("overlay", overlay_at, &encoder, &stats);
        let Timings {
            pass_ms,
            pass_queries,
            ..
        } = timings;
        let gpu_queries = encoder.used();
        self.pass_ms = pass_ms;
        if let Some((frame, ..)) = surface_frame.as_ref().filter(|_| self.keep_frame) {
            self.keep_presented(&mut encoder, &frame.texture);
        }
        self.queue
            .submit(cull_commands.into_iter().chain(Some(encoder.finish())));
        if let Some(gc) = self.gpu_cull.as_mut().filter(|_| gpu_cull) {
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
        if taa {
            self.taa_history = Some(cam.view_proj);
        }
        // The main pass copied its lit image for the next frame's GI.
        self.gi_history = gi_copy.then_some(self.frame_index);
        self.prev_view_proj = Some(cam.view_proj);
        self.frame_index += 1;
        // The first frame made the scene's pipelines: keep them now, not
        // only when the renderer is dropped (a killed process never is).
        if self.frame_index == 1
            && let Some(cache) = &mut self.pipeline_disk
        {
            cache.save();
        }
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
        self.indirect
            && match self.desc_culling {
                Culling::Auto => sorted >= GPU_CULLING_MIN_INSTANCES,
                Culling::Gpu => true,
                Culling::Cpu => false,
            }
    }
}
