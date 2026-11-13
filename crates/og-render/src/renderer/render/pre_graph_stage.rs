//! The frame's encoder, and the compute nodes the graph begins with
//! (pre_graph.rs): the crowds' cull, GPU culling's counts' copy, the wind
//! and trample steps, the water's fields, the trails, the aurora's
//! footprint, the visual meshes, the surfaces, the terrain levels'
//! corners, the grass blades and the scattered copies, each encoded when
//! execute.rs reaches it.

use super::*;

/// What the compute before the frame's draws reads.
pub(super) struct PreGraphInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub crowd_frame: &'a crate::crowd::CrowdFrame,
    pub crowd_previous: Option<u32>,
    pub vis: &'a FrameVisibility,
    pub gpu_cull: bool,
    pub skinned: &'a SkinnedFrame,
    pub skinned_motion: Option<bool>,
    pub joints_drawn: u32,
    pub terrain_draws: &'a [terrain::TerrainDraw],
    pub grass_frame: &'a grass::GrassFrame,
}

impl Renderer {
    /// Opens the frame's encoder, with the queries this frame's passes
    /// write their timestamps into (`timing.rs`): the encoder puts a pair
    /// around every pass.
    pub(super) fn open_frame_encoder(&mut self) -> Encoder {
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
        encoder.expect(self.info.caps, self.info.caps);
        encoder
    }

    /// Records the compute node `pass`; answers the GPU queries it lists
    /// under their own names (the crowds' cull's, the water's fields', the
    /// scatter's).
    pub(super) fn encode_pre_graph_pass(
        &mut self,
        pass: FramePass,
        encoder: &mut Encoder,
        &PreGraphInputs {
            scene,
            camera,
            cam,
            view_frustum,
            crowd_frame,
            crowd_previous,
            vis,
            gpu_cull,
            skinned,
            skinned_motion,
            joints_drawn,
            terrain_draws,
            grass_frame,
        }: &PreGraphInputs<'_>,
    ) -> Vec<(String, std::ops::Range<u32>)> {
        match pass {
            // The crowds' cull, before any pass draws them (pre_graph.rs).
            FramePass::CrowdCull => pre_graph::encode_crowd_cull(
                encoder,
                (&self.device, &self.queue, (self.width, self.height)),
                self.visibility.gpu_cull.as_ref().expect("made above"),
                self.world
                    .crowd_gpu
                    .as_mut()
                    .expect("made when crowd_gpu was decided"),
                scene,
                crowd_frame,
                &vis.cull_passes,
                (camera, cam),
                crowd_previous,
            )
            .map(|q| ("cull: crowds".to_string(), q))
            .into_iter()
            .collect(),
            // GPU culling's counts' readback, after every cull that writes them.
            FramePass::CullCounts => {
                if gpu_cull && let Some(gc) = self.visibility.gpu_cull.as_mut() {
                    gc.copy_counts(encoder);
                }
                Vec::new()
            }
            // The wind field and the trample map step before any pass reads them.
            FramePass::Wind => {
                self.world.wind_field.encode(encoder);
                self.world.trample.encode(encoder);
                Vec::new()
            }
            // The water's fields this frame's prepare planned, before any
            // pass draws water or reads the caustics.
            FramePass::WaterFields => self
                .world
                .water
                .encode_fields(encoder)
                .map(|q| ("water fields".to_string(), q))
                .into_iter()
                .collect(),
            // The trails' histories, before the strips are drawn.
            FramePass::Trails => {
                if let Some(trails) = &mut self.strips.gpu_trails {
                    trails.encode(encoder);
                }
                Vec::new()
            }
            // The aurora's footprint, before any pass reads the sky.
            FramePass::Aurora => {
                self.sky.aurora_map.encode(encoder);
                Vec::new()
            }
            FramePass::Visual => {
                if let Some(pipelines) = &self.world.visual {
                    for mesh in &self.cache.meshes {
                        if let Some(v) = mesh.surface.as_ref().and_then(|s| s.visual.as_ref()) {
                            pipelines.encode(encoder, v, false);
                        }
                    }
                }
                if let Some(fluids) = &self.world.visual_fluids {
                    fluids.encode(encoder);
                }
                Vec::new()
            }
            FramePass::Surfaces => {
                if let Some(pipelines) = &self.world.surfaces {
                    for mesh in &self.cache.meshes {
                        if let Some(surface) = &mesh.surface {
                            pipelines.encode(
                                encoder,
                                surface,
                                &mesh.vertices,
                                scene.frame.camera_cut,
                            );
                        }
                    }
                }
                if let Some(pipelines) = &self.world.garments {
                    if !skinned.garments.is_empty() {
                        pipelines.encode(
                            &self.device,
                            &self.queue,
                            encoder,
                            &mut self.cache.garments,
                            scene,
                            &skinned.garments,
                            self.instances.palette.buffer(),
                            skinned_motion.map(|_| joints_drawn),
                            self.world.visual.as_ref(),
                        );
                    } else {
                        self.cache.garments.clear();
                    }
                }
                Vec::new()
            }
            // Every terrain level's corners, before every pass that draws them.
            FramePass::TerrainCorners => {
                terrain::encode_corners(encoder, terrain_draws);
                Vec::new()
            }
            // The grass's blades, generated before every pass that draws them.
            FramePass::Grass => {
                self.world.grass.encode(encoder, grass_frame, terrain_draws);
                Vec::new()
            }
            // The scattered copies placed on the GPU this frame, then those the
            // main camera sees, culled on the GPU against its view and the
            // depth pyramid the culling drew.
            FramePass::Scatter => {
                let textures = &self.cache.textures;
                self.world
                    .scatter
                    .encode(
                        &self.device,
                        &self.queue,
                        encoder,
                        &scatter_cull::CullView {
                            view_proj: cam.jittered_view_proj,
                            planes: view_frustum.planes(),
                            camera: camera.position,
                            size: (self.width, self.height),
                            pyramid: self
                                .visibility
                                .gpu_cull
                                .as_ref()
                                .filter(|_| gpu_cull)
                                .and_then(|gc| gc.pyramid()),
                        },
                        &|h| textures.get(h.index()).map(|t| &t.view),
                        terrain_draws,
                    )
                    .into_iter()
                    .map(|(n, q)| (n.to_string(), q))
                    .collect()
            }
            other => unreachable!("{other:?} is not compute before the graph"),
        }
    }
}
