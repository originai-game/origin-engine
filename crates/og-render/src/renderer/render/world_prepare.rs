//! The world's prepared draws: the terrains (new or edited data uploaded,
//! this frame's clipmap levels and their pipeline variants), the scattered
//! copies' chunks, and the grass fields over the drawn terrains, in that
//! order.

use super::*;
use crate::ground::GroundGpu;

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
        let card_inputs = Self::crowd_card_inputs(crowd_frame, &cpu_cards.0);
        self.make_world(scene, !card_inputs.is_empty());
        let textures = &self.cache.textures;
        let arrays = &self.cache.texture_arrays;
        let terrain_draws = match self.world.terrain.get_mut() {
            Some(terrain) if WORLD_3D => terrain.prepare(
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
                // The far colour of what grows on each (the grass).
                &|terrain| crate::grass::far_gpu(scene, terrain),
            )?,
            _ => Vec::new(),
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
        let ground: &dyn GroundGpu = match self.world.terrain.get() {
            Some(t) => t,
            None => &NoGround,
        };
        let scatter_batches = match self.world.scatter.get_mut() {
            Some(scatter) if WORLD_3D => {
                let materials = &self.cache.materials;
                let meshes = &self.cache.meshes;
                scatter.prepare(
                    &self.device,
                    &self.queue,
                    scene,
                    camera.position,
                    (cam.view_proj, self.height, scene.objects.lod_bias),
                    prev_view_proj,
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
                    (&card_inputs, &cpu_cards.1),
                    (ground, &terrain_draws),
                )
            }
            _ => scatter::ScatterBatches::default(),
        };
        // Grass: this frame's fields planned over the drawn terrains (grass.rs).
        let grass_frame = match (self.world.grass.get_mut(), self.world.terrain.get()) {
            (Some(grass), Some(terrain)) if WORLD_3D => grass.prepare(
                &self.device,
                &self.queue,
                scene,
                terrain,
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
            ),
            _ => grass::GrassFrame::default(),
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

/// The ground of a frame that draws no terrain, for what grows on one: none.
struct NoGround;

impl GroundGpu for NoGround {
    fn group_layout(&self, _arrayed: bool) -> Option<&wgpu::BindGroupLayout> {
        None
    }

    fn level0_slots(&self, _data: usize, _tiles: &[u32]) -> Option<Vec<usize>> {
        None
    }

    fn extent_of(&self, _data: usize) -> Option<(Vec2, (f32, f32))> {
        None
    }
}

impl Renderer {
    /// Makes the 3D world's renderers this frame draws with ([`OnDemand`]):
    /// the terrain with a terrain, the scattered copies with a scatter, a
    /// model impostor or a crowd's cards (`cards`), the grass with a grass
    /// field on a terrain, the water with a water and the 3D fluids with
    /// one; each at the frame's sample count and features.
    pub(super) fn make_world(&mut self, scene: &RenderScene, cards: bool) {
        if !WORLD_3D {
            return;
        }
        let samples = self.samples;
        let features = self.frame_features;
        let capturing = self.capturing;
        if !scene.world.terrains.is_empty() {
            let (t, made) = self.world.terrain.make();
            if made {
                t.set_samples(samples);
                t.features = features;
                t.settled = capturing;
            }
        }
        if !scene.world.scatters.is_empty() || !scene.objects.impostors.is_empty() || cards {
            let (s, made) = self.world.scatter.make();
            if made {
                s.set_samples(samples);
                s.features = features;
            }
        }
        if !scene.world.grass.is_empty() && !scene.world.terrains.is_empty() {
            let (g, made) = self.world.grass.make();
            if made {
                g.set_samples(samples);
            }
        }
        if !scene.world.water.is_empty() {
            let (w, made) = self.world.water.make();
            if made {
                for lit in w.pipeline.each().chain(w.mesh_pipeline.each()) {
                    lit.set_samples(samples);
                }
            }
        }
        if scene
            .world
            .fluids_3d
            .iter()
            .any(|f| !f.particles.is_empty() || f.visual.is_some())
        {
            let (f, made) = self.world.fluid3d.make();
            if made {
                f.set_samples(samples);
            }
        }
    }
}
