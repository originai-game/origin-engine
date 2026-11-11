//! The water's and the 3D fluids' frame: the water's draws through the
//! scene copy and over the scene's depth, and as each due capture face draws
//! it; the fluids' bounding sphere, their frame and visual simulation, and
//! their draws in each due capture face, in that order.

use super::*;

pub(super) struct PrepareWaterAndFluidsInputs<'a> {
    pub scene: &'a RenderScene,
    pub cam: &'a frame::CameraFrame,
    pub scene_water: &'a [Water],
    pub reflections: &'a [camera::PlanarReflection],
    pub ao_target: bool,
    pub ao: Option<AmbientOcclusion>,
    pub probe_frame: &'a probes::ProbeFrame,
    pub probe_faces: &'a [[frame::CameraFrame; 6]],
    pub fluids_3d: bool,
    pub face_fluid: bool,
}

pub(super) struct PrepareWaterAndFluidsOutputs {
    pub water_draws: Vec<water::WaterDraw>,
    pub face_water: Vec<Vec<water::WaterDraw>>,
    pub fluid_bounds: Option<(Vec3, f32)>,
    pub fluid_frame: Option<fluid3d::Fluid3dFrame>,
    pub face_fluids: Vec<Option<fluid3d::Fluid3dFrame>>,
}

impl Renderer {
    /// Prepares the frame's water and 3D fluid draws.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_water_and_fluids(
        &mut self,
        PrepareWaterAndFluidsInputs {
            scene,
            cam,
            scene_water,
            reflections,
            ao_target,
            ao,
            probe_frame,
            probe_faces,
            fluids_3d,
            face_fluid,
        }: PrepareWaterAndFluidsInputs<'_>,
    ) -> PrepareWaterAndFluidsOutputs {
        let scene_depth = self
            .graph
            .try_view("lindepth")
            .unwrap_or(&self.post.far_depth);
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
            Some(copy) if !scene_water.is_empty() => self.world.water.prepare(
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
                scene.atmosphere.adaptation,
                scene.origin,
                water::seconds(scene),
                &scene.camera,
                &scene.quality,
                &self.post.white,
                &self.lights.caustic_grid,
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
                    self.world.water.face_draws(
                        &self.device,
                        &self.queue,
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
            scene.world.fluids_3d.iter().flat_map(|f| {
                // A visual fluid: its spawn box's corners.
                let corners = f
                    .visual
                    .iter()
                    .flat_map(|v| [v.centre - v.size * 0.5, v.centre + v.size * 0.5]);
                f.particles
                    .iter()
                    .copied()
                    .chain(corners)
                    .map(move |p| (p, f.radius))
            }),
        )
        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo).length() * 0.5));
        let fluid_frame = match fluid3d::views(&self.graph, scene_depth) {
            Some(views) if fluids_3d => self.world.fluid3d.prepare(
                &self.device,
                &self.queue,
                &scene.world.fluids_3d,
                &fluid3d::Fluid3dCamera {
                    view: cam.view,
                    proj: cam.proj,
                    orthographic: cam.orthographic,
                },
                (self.width, self.height),
                &views,
                self.world.visual_fluids.is_some(),
            ),
            _ => None,
        };
        if let Some(fluids) = self.world.visual_fluids.as_mut() {
            let slots = match fluid_frame {
                Some(_) => self.world.fluid3d.visual.as_slice(),
                None => &[],
            };
            fluids.prepare(
                &self.device,
                &self.queue,
                slots,
                &scene.world.colliders,
                scene.tick,
                scene.tick_rate,
                self.world.fluid3d.instances(),
            );
        }
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
                    self.world.fluid3d.camera_frame(
                        &self.device,
                        &self.queue,
                        &scene.world.fluids_3d,
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
        PrepareWaterAndFluidsOutputs {
            water_draws,
            face_water,
            fluid_bounds,
            fluid_frame,
            face_fluids,
        }
    }
}
