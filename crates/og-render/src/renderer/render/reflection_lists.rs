//! The planar reflections' lists: each mirror's opaque meshes, the first
//! through the GPU cull's reflection pass, the rest culled on the CPU, and
//! its transparent meshes back to front from the mirrored eye.

use super::*;

pub(super) struct PlanReflectionListsInputs<'a> {
    pub batches: &'a [Batch],
    pub vis: &'a mut FrameVisibility,
    pub gpu_cull: bool,
    pub spheres: &'a [(Vec3, f32)],
    pub transforms: &'a mut Vec<instances::GpuTransform>,
    pub origins: &'a mut Vec<u32>,
    pub cull_inside: &'a mut CullScratch,
    pub submission: &'a [u32],
    pub in_meshlet: &'a dyn Fn(&Batch) -> bool,
    pub on_layers: &'a dyn Fn(u32) -> Vec<u32>,
    pub reflections: &'a [camera::PlanarReflection],
    pub mask: u32,
    pub cull_n: u32,
}

pub(super) struct PlanReflectionListsOutputs {
    pub reflection_list: Vec<DrawList>,
    pub reflection_meshlets: Vec<u32>,
    pub reflection_blended: Vec<DrawList>,
}

impl Renderer {
    /// Culls and sorts each planar reflection's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_reflection_lists(
        &self,
        PlanReflectionListsInputs {
            batches,
            vis,
            gpu_cull,
            spheres,
            transforms,
            origins,
            cull_inside,
            submission,
            in_meshlet,
            on_layers,
            reflections,
            mask,
            cull_n,
        }: PlanReflectionListsInputs<'_>,
    ) -> PlanReflectionListsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        // (The GPU culling has one reflection pass: the second slot is culled here.)
        let mut reflection_list = Vec::new();
        let mut reflection_meshlets: Vec<u32> = Vec::new();
        for (k, r) in reflections.iter().enumerate() {
            let frustum = Frustum::from_view_proj(r.3);
            let view = vis.meshlet_view(
                meshlet_gpu,
                batches,
                meshlet_gpu::MeshletView::seen(
                    frustum.planes(),
                    on_layers(mask),
                    r.1.position,
                    r.3,
                    r.4.1,
                ),
            );
            reflection_meshlets.push(view);
            let include =
                |b: &Batch| !b.transparent && b.layers & mask != 0 && !(view > 0 && in_meshlet(b));
            reflection_list.push(if gpu_cull && k == 0 {
                let g = vis.gpu_pass(cull_gpu::PassKind::Reflection, &frustum, None, None);
                indirect_list(batches, cull_n, g, include)
            } else {
                self.cull(
                    batches,
                    spheres,
                    transforms,
                    origins,
                    cull_inside,
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
        for r in reflections {
            let frustum = Frustum::from_view_proj(r.3);
            reflection_blended.push(
                self.sort_transparent(
                    batches,
                    spheres,
                    submission,
                    transforms,
                    origins,
                    &frustum,
                    r.1.position,
                    r.2.forward,
                    mask,
                    &[],
                )
                .0,
            );
        }
        PlanReflectionListsOutputs {
            reflection_list,
            reflection_meshlets,
            reflection_blended,
        }
    }
}
