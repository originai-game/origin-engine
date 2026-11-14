//! Terrain renderer initialization, tile uploads and frame preparation.

use super::*;

mod data;
mod draw;
mod pipelines;
mod prepare;
mod stamps;

pub(crate) use draw::*;
pub(crate) use prepare::*;

impl TerrainRenderer {
    /// Draw the lit pass into a target of `samples` per pixel from now on
    /// (the layer-array variant, made later, too).
    pub(crate) fn set_samples(&mut self, samples: u32) {
        self.samples = samples;
        let all = std::iter::once(&self.passes)
            .chain(self.arrays.as_ref().map(|(_, p)| p))
            .chain(self.holed.iter().flatten());
        for passes in all {
            for lit in passes.lit.iter().chain(&passes.stamp_lit) {
                for lit in lit.each() {
                    lit.set_samples(samples);
                }
            }
        }
    }

    /// Starts compiling the pipelines `scene`'s visible terrains draw with,
    /// as far as they are known before [`TerrainRenderer::prepare`] (which
    /// makes the hole and layer-array variants; the frame starts those
    /// after it, from its draws): the frame's longest compiles, on worker
    /// threads while the rest of the frame is prepared (deferred.rs).
    pub(crate) fn start_passes(&self, scene: &crate::RenderScene, prepass: bool) {
        for t in scene.world.terrains.iter().filter(|t| t.visible) {
            let holes = self
                .datas
                .get(t.terrain.index())
                .and_then(Option::as_ref)
                .is_some_and(|d| d.layout.holes);
            let passes = match (holes, t.layer_arrays.any(), &self.arrays) {
                (true, arrayed, _) => self.holed[usize::from(arrayed)].as_ref(),
                (false, true, arrays) => arrays.as_ref().map(|(_, p)| p),
                (false, false, _) => Some(&self.passes),
            };
            if let Some(passes) = passes {
                passes.start(
                    self.features,
                    crate::mirrors(&t.transform),
                    prepass,
                    t.cast_shadows,
                    !scene.world.terrain_stamps.is_empty(),
                );
            }
        }
    }

    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        let p = &self.passes;
        crate::deferred::warm_one(&[
            p.lit[0].full(),
            p.prepass[0].full(),
            &*p.shadow,
            &*p.corners,
        ])
    }

    /// Drops every terrain's bind group (a layer texture was replaced):
    /// the next prepare makes them again from the current views.
    pub(crate) fn forget_groups(&mut self) {
        for slot in &mut self.slots {
            slot.group = None;
        }
    }

    /// The index buffer (bound by the passes that draw terrain).
    pub(crate) fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref().map(|(b, _)| b)
    }

    /// The world bounds of the terrains drawn this frame (centre, radius),
    /// for the shadow fit.
    pub(crate) fn bounds(&self, scene: &crate::RenderScene) -> Vec<(Vec3, f32)> {
        scene
            .world
            .terrains
            .iter()
            .filter_map(|t| {
                let d = self.datas.get(t.terrain.index())?.as_ref()?;
                let s = d.layout.size;
                let lo = Vec3::new(-s.x * 0.5, d.range.0, -s.y * 0.5);
                let hi = Vec3::new(s.x * 0.5, d.range.1, s.y * 0.5);
                Some(world_sphere(t.transform, lo, hi))
            })
            .collect()
    }
}

impl GroundGpu for TerrainRenderer {
    fn group_layout(&self, arrayed: bool) -> Option<&wgpu::BindGroupLayout> {
        if arrayed {
            self.arrays.as_ref().map(|(l, _)| l)
        } else {
            Some(&self.layout)
        }
    }

    fn level0_slots(&self, data: usize, tiles: &[u32]) -> Option<Vec<usize>> {
        let d = self.datas.get(data)?.as_ref()?;
        tiles
            .iter()
            .map(|t| {
                let st = d.slots[*d.by_tile.get(t)?].as_ref()?;
                Some(Arc::as_ptr(&st.data) as usize)
            })
            .collect()
    }

    fn extent_of(&self, data: usize) -> Option<(Vec2, (f32, f32))> {
        let d = self.datas.get(data)?.as_ref()?;
        Some((d.layout.size, d.range))
    }
}

impl GroundDraw for TerrainDraw {
    fn group(&self) -> &wgpu::BindGroup {
        &self.group
    }

    fn terrain(&self) -> usize {
        self.terrain
    }

    fn arrayed(&self) -> bool {
        self.arrayed
    }
}
