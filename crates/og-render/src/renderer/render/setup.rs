//! Setup stage.

use super::*;

impl Renderer {
    pub(super) fn prepare_frame_resources(&mut self, scene: &RenderScene) -> (u32, (u32, u32)) {
        self.apply_quality(&scene.quality);
        // A look that switches the stylisation on or off (passes added or
        // removed, the pixel grid, the material style) is a cut: no ghost of
        // the old look in the first frame of the new one.
        let chain = scene.stylise.chain_bits();
        if self.stylise_chain.is_some_and(|c| c != chain) {
            self.taa_history = None;
        }
        self.stylise_chain = Some(chain);
        let (grid, upscale, grid_offset) = self.pixel_grid(scene.stylise.pixel);
        self.scene_size(grid.0, grid.1);
        self.grow_local_shadows(scene);
        self.ensure_environment(scene);
        (upscale, grid_offset)
    }

    pub(super) fn prepare_frame_probes(
        &mut self,
        scene: &RenderScene,
    ) -> (probes::ProbeFrame, Vec<ReflectionProbe>, Vec<(Vec3, Vec3)>) {
        let probe_frame = self.probes.prepare(
            &self.device,
            &self.queue,
            &self.ibl,
            &scene.reflection_probes,
            scene.tick,
            self.samples,
            |h| {
                self.cache
                    .environment_maps
                    .get(h.index())
                    .is_some_and(Option::is_some)
            },
        );
        self.lightmaps
            .prepare(&self.device, &self.queue, &scene.baked);
        // Reflection probes: the frame's active ones, whose boxes key the
        // instances' sort (probe_select.rs: a batch samples one probe).
        let active_probes: Vec<ReflectionProbe> = probe_frame.active.iter().map(|a| a.0).collect();
        let probe_boxes = probe_select::boxes(&active_probes);
        self.sorted.set_probes(probe_boxes.clone());
        (probe_frame, active_probes, probe_boxes)
    }

    pub(super) fn prepare_full_meshes(&mut self, scene: &RenderScene, batches: &[Batch]) {
        // The f32 vertices of compact meshes drawn without their compact
        // ones: by a material that is not plain, skinned or scattered.
        let full: Vec<usize> = batches
            .iter()
            .filter(|b| !b.compact)
            .map(|b| b.mesh)
            .chain(scene.skinned_instances.iter().map(|s| s.mesh.index()))
            .chain(scene.scatters.iter().map(|s| s.mesh.index()))
            .collect();
        for mesh in full {
            self.ensure_full(scene, mesh);
        }
        // Water drawn over a mesh reads its f32 vertices.
        for m in scene.water.iter().filter_map(|w| w.mesh) {
            self.ensure_full(scene, m.index());
        }
    }
}
