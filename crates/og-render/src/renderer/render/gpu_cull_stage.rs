//! The frame's GPU culling: the sorted instances' records and previous
//! transforms uploaded and culled for every pass the frame registered
//! (`FrameVisibility`), then the clustered batches' culling after it.

use super::*;

pub(super) struct GpuCullInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub transforms: &'a [instances::GpuTransform],
    pub origins: &'a [u32],
    pub sorted: u32,
    pub vis: &'a FrameVisibility,
    pub max_passes: usize,
    pub auxiliary_instances: bool,
    pub previous_valid: bool,
    pub object_motion: bool,
    pub prepass: bool,
    pub skinned: Option<cull_gpu::SkinnedCull<'a>>,
    pub cam: &'a frame::CameraFrame,
    pub copy_counts: bool,
}

impl Renderer {
    /// Encodes the GPU culling and the meshlets' culling after it, and
    /// answers their command buffers with the counts read back (lagged)
    /// and the occluders drawn.
    #[allow(clippy::type_complexity)]
    pub(super) fn encode_gpu_cull(
        &mut self,
        GpuCullInputs {
            scene,
            batches,
            transforms,
            origins,
            sorted,
            vis,
            max_passes,
            auxiliary_instances,
            previous_valid,
            object_motion,
            prepass,
            skinned,
            cam,
            copy_counts,
        }: GpuCullInputs<'_>,
    ) -> (Vec<wgpu::CommandBuffer>, (Option<cull_gpu::GpuCounts>, u32)) {
        let previous_sway = if self.instances.previous_sway.len() == scene.instances.len() {
            self.instances.previous_sway.as_slice()
        } else {
            &[]
        };
        let gc = self
            .visibility
            .gpu_cull
            .as_mut()
            .expect("made when gpu_cull was decided");
        if gc.in_flight() {
            let _ = self.device.poll(wgpu::PollType::Poll);
        }
        let lagged = gc.poll_counts();
        debug_assert!(vis.cull_passes.len() <= max_passes);
        // The sorted instances' previous transforms (as the CPU path's
        // below), compacted with the camera pass's and, for foliage,
        // every pass that reads its current sway.
        let previous_sorted: Option<Vec<instances::GpuTransform>> =
            auxiliary_instances.then(|| {
                let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
                origins[..sorted as usize]
                    .iter()
                    .map(|&i| {
                        let i = i as usize;
                        let inst = &scene.instances[i];
                        let p = if previous_valid {
                            scene.previous_transforms[i]
                        } else {
                            inst.transform
                        };
                        let d = Self::dequant(
                            meshes,
                            materials,
                            inst.mesh.index(),
                            inst.material.index(),
                        );
                        let mut t = instances::gpu_transform(&d.map_or(p, |d| p * d));
                        // Previous sway uses the layer's shadow-sign encoding;
                        // current sway is raw in the spare fx word. Ordinary
                        // prepass vertices read only the transform columns.
                        let sway = previous_sway.get(i).copied().unwrap_or(inst.sway);
                        t.layer = instances::layer_bits(sway.max(0.0), inst.receive_shadows);
                        t.fx[0] = inst.sway.max(0.0);
                        t
                    })
                    .collect()
            });
        let (commands, occluder_draws) = gc.encode(
            &self.device,
            &self.queue,
            &transforms[..sorted as usize],
            previous_sorted.as_deref(),
            batches,
            skinned.as_ref(),
            &self.cache.meshes,
            &self.cache.materials,
            &vis.cull_passes,
            cam.jittered_view_proj,
            (self.width, self.height, self.samples),
            self.frame_index,
            lagged.unwrap_or_default(),
            copy_counts,
        );
        let mut commands = vec![commands];
        // The clustered batches' culling, after GPU culling's (it
        // reads the camera pass's survivors and the pyramid).
        if let Some(m) = self.visibility.meshlet.as_mut() {
            let gc = self.visibility.gpu_cull.as_ref().expect("made above");
            // Last frame's transforms at the same (sorted) slots: with
            // object motion, region 0 of GPU culling's `previous`.
            let previous = if object_motion {
                &gc.previous
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
                batches,
                &vis.meshlet_views,
                gc,
                cam.jittered_view_proj,
                (self.width, self.height),
                previous,
                &levels,
                {
                    let (w, f) = crate::foliage::foliage_winds(
                        scene,
                        self.history.previous_wind,
                        self.world.wind_field.grid(),
                    );
                    (w, f, self.world.wind_field.lean_cap())
                },
                |mesh| &meshes[mesh].vertices,
                |b| materials[b.material].occluder,
            ));
            if !m.frame.is_empty() {
                m.ensure_pipelines(
                    &self.device,
                    [&self.frame_layout, &self.material_layout, &self.fx_layout],
                    planar_slot(self.info.planar_reflections),
                    self.info.without.features(),
                    batches,
                    prepass,
                    self.samples,
                );
                m.ensure_shadow_pipelines(
                    &self.device,
                    (&self.shadow_bind_layout, &self.material_layout),
                    batches,
                );
            }
        }
        if self
            .visibility
            .meshlet
            .as_ref()
            .is_some_and(|m| !m.frame.is_empty())
            && let Some(gc) = self.visibility.gpu_cull.as_mut()
        {
            gc.counted_by_meshlets(&vis.meshlet_views[0].taken);
        }
        (commands, (lagged, occluder_draws))
    }
}
