//! The world's prepared draws: the terrains (new or edited data uploaded,
//! this frame's clipmap levels and their pipeline variants), the scattered
//! copies' chunks, and the grass fields over the drawn terrains, in that
//! order.

use super::*;

pub(super) struct PrepareWorldInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub prepass: bool,
    pub crowd_frame: &'a crate::crowd::CrowdFrame,
    pub cpu_cards: &'a (Vec<(u32, u32, u32)>, Vec<crate::skin::SkinnedInstanceGpu>),
}

pub(super) struct PrepareWorldOutputs {
    pub prev_view_proj: Mat4,
    pub terrain_draws: Vec<terrain::TerrainDraw>,
    pub terrain_casters: bool,
    pub scatter_batches: scatter::ScatterBatches,
    pub grass_frame: grass::GrassFrame,
}

impl Renderer {
    /// Prepares the frame's terrains, scattered copies and grass.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_world(
        &mut self,
        PrepareWorldInputs {
            scene,
            camera,
            cam,
            view_frustum,
            prepass,
            crowd_frame,
            cpu_cards,
        }: PrepareWorldInputs<'_>,
    ) -> Result<PrepareWorldOutputs, RenderError> {
        // Terrains: new or edited data uploaded, this frame's clipmap levels.
        let prev_view_proj = match self.history.prev_view_proj {
            Some(vp) if !scene.frame.camera_cut => vp,
            _ => cam.view_proj,
        };
        let textures = &self.cache.textures;
        let arrays = &self.cache.texture_arrays;
        let terrain_draws = if !WORLD_3D {
            Vec::new()
        } else {
            self.world.terrain.prepare(
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
        // The variants the prepare made (holes, layer arrays), and stamps.
        for d in &terrain_draws {
            d.passes.start(
                d.features,
                d.mirrored,
                prepass,
                d.cast_shadows,
                !d.stamps.is_empty(),
            );
        }
        // Scattered copies: the chunks this frame needs built, what to draw.
        let scatter_batches = if !WORLD_3D {
            scatter::ScatterBatches::default()
        } else {
            let materials = &self.cache.materials;
            let meshes = &self.cache.meshes;
            self.world.scatter.prepare(
                &self.device,
                &self.queue,
                scene,
                camera.position,
                (cam.view_proj, self.height, scene.objects.lod_bias),
                prev_view_proj,
                self.phase(),
                (
                    self.world.wind_field.grid(),
                    self.world.wind_field.lean_cap(),
                    self.world.trample.grid(),
                ),
                |m| materials.get(m).map(|g| g.side),
                |m| {
                    meshes.get(m).filter(|g| g.index_count > 0).map(|g| {
                        let (r, top) = g
                            .bounds
                            .map_or((1.0, 1.0), |(c, e)| (c.length() + e.length(), c.y + e.y));
                        (r, top, g.index_count)
                    })
                },
                |m| materials.get(m).and_then(|g| g.shader),
                |sh| scene.shaders()[sh].hooks_wgsl(),
                (
                    &Self::crowd_card_inputs(crowd_frame, &cpu_cards.0),
                    &cpu_cards.1,
                ),
                (&self.world.terrain, &terrain_draws),
            )
        };
        // Grass: this frame's fields planned over the drawn terrains (grass.rs).
        let grass_frame = if !WORLD_3D {
            grass::GrassFrame::default()
        } else {
            self.world.grass.prepare(
                &self.device,
                &self.queue,
                scene,
                &self.world.terrain,
                &terrain_draws,
                grass::GrassView {
                    camera: camera.position,
                    frame: cam,
                    prev_view_proj,
                    height: self.height,
                    wind_grid: self.world.wind_field.grid(),
                    trample: self.world.trample.grid(),
                },
                view_frustum,
                &|h| textures.get(h.index()).map(|t| &t.view),
            )
        };
        Ok(PrepareWorldOutputs {
            prev_view_proj,
            terrain_draws,
            terrain_casters,
            scatter_batches,
            grass_frame,
        })
    }
}
