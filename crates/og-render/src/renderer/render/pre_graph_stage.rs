//! The frame's encoder and the compute before the graph (pre_graph.rs):
//! the timestamp queries begun, the crowds' cull, GPU culling's counts'
//! copy, the wind and trample steps, the visual meshes, the surfaces, the
//! terrain levels' corners, the grass blades and the scattered copies, in
//! that order.

use super::*;

pub(super) struct EncodePreGraphInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub crowd_gpu: bool,
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

pub(super) struct EncodePreGraphOutputs {
    pub encoder: Encoder,
    pub crowd_cull_queries: Option<std::ops::Range<u32>>,
    pub scatter_queries: Vec<(&'static str, std::ops::Range<u32>)>,
}

impl Renderer {
    /// Opens the frame's encoder and records the compute before the graph.
    #[allow(clippy::too_many_lines)]
    pub(super) fn encode_pre_graph(
        &mut self,
        EncodePreGraphInputs {
            scene,
            camera,
            cam,
            view_frustum,
            crowd_gpu,
            crowd_frame,
            crowd_previous,
            vis,
            gpu_cull,
            skinned,
            skinned_motion,
            joints_drawn,
            terrain_draws,
            grass_frame,
        }: EncodePreGraphInputs<'_>,
    ) -> EncodePreGraphOutputs {
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
        encoder.expect(self.info.caps, self.info.caps);
        // The crowds' cull, before any pass draws them (pre_graph.rs).
        let crowd_cull_queries = if crowd_gpu {
            pre_graph(&mut encoder, self.info.caps, pre_graph::CROWD_CULL, |e| {
                pre_graph::encode_crowd_cull(
                    e,
                    (&self.device, &self.queue, (self.width, self.height)),
                    self.visibility.gpu_cull.as_ref().expect("made above"),
                    self.crowd_gpu
                        .as_mut()
                        .expect("made when crowd_gpu was decided"),
                    scene,
                    crowd_frame,
                    &vis.cull_passes,
                    (camera, cam),
                    crowd_previous,
                )
            })
            .flatten()
        } else {
            None
        };
        // GPU culling's counts' readback, after every cull that writes them.
        if gpu_cull && let Some(gc) = self.visibility.gpu_cull.as_mut() {
            gc.copy_counts(&mut encoder);
        }
        // The wind field and the trample map step before any pass reads them.
        pre_graph(&mut encoder, self.info.caps, pre_graph::WIND, |e| {
            self.wind_field.encode(e);
            self.trample.encode(e);
        });
        pre_graph(&mut encoder, self.info.caps, pre_graph::VISUAL, |e| {
            if let Some(pipelines) = &self.visual {
                for mesh in &self.cache.meshes {
                    if let Some(v) = mesh.surface.as_ref().and_then(|s| s.visual.as_ref()) {
                        pipelines.encode(e, v, false);
                    }
                }
            }
            if let Some(fluids) = &self.visual_fluids {
                fluids.encode(e);
            }
        });
        pre_graph(&mut encoder, self.info.caps, pre_graph::SURFACES, |e| {
            if let Some(pipelines) = &self.surfaces {
                for mesh in &self.cache.meshes {
                    if let Some(surface) = &mesh.surface {
                        pipelines.encode(e, surface, &mesh.vertices, scene.camera_cut);
                    }
                }
            }
            if let Some(pipelines) = &self.garments {
                if !skinned.garments.is_empty() {
                    pipelines.encode(
                        &self.device,
                        &self.queue,
                        e,
                        &mut self.cache.garments,
                        scene,
                        &skinned.garments,
                        self.instances.palette.buffer(),
                        skinned_motion.map(|_| joints_drawn),
                        self.visual.as_ref(),
                    );
                } else {
                    self.cache.garments.clear();
                }
            }
        });
        // Every terrain level's corners, before every pass that draws them.
        pre_graph(
            &mut encoder,
            self.info.caps,
            pre_graph::TERRAIN_CORNERS,
            |e| terrain::encode_corners(e, terrain_draws),
        );
        // The grass's blades, generated before every pass that draws them.
        pre_graph(&mut encoder, self.info.caps, pre_graph::GRASS, |e| {
            self.grass.encode(e, grass_frame, terrain_draws)
        });
        // The scattered copies placed on the GPU this frame, then those the
        // main camera sees, culled on the GPU against its view and the
        // depth pyramid the culling drew.
        let scatter_queries = pre_graph(&mut encoder, self.info.caps, pre_graph::SCATTER, |e| {
            let textures = &self.cache.textures;
            self.scatter.encode(
                &self.device,
                &self.queue,
                e,
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
        })
        .unwrap_or_default();
        EncodePreGraphOutputs {
            encoder,
            crowd_cull_queries,
            scatter_queries,
        }
    }
}
