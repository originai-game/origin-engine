//! The views' lists: each view's transparent meshes on its layers back to
//! front from its eye, and its opaque meshes on its layers (under GPU culling
//! a pass of its own, drawn indirect; on the CPU path culled here).

use super::*;

pub(super) struct PlanViewListsInputs<'a> {
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
    pub view_plans: &'a [views::ViewPlan],
    pub view_boards: &'a [sprites::Billboards],
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub cull_n: u32,
}

pub(super) struct PlanViewListsOutputs {
    pub view_blended: Vec<DrawList>,
    pub view_views: Vec<u32>,
    pub view_lists: Vec<DrawList>,
}

impl Renderer {
    /// Sorts and culls each view's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_view_lists(
        &self,
        PlanViewListsInputs {
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
            view_plans,
            view_boards,
            particle_frame,
            cull_n,
        }: PlanViewListsInputs<'_>,
    ) -> PlanViewListsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        // Each view's transparent meshes on its layers, back to front from
        // its eye.
        let view_blended: Vec<DrawList> = view_plans
            .iter()
            .zip(view_boards)
            .map(|(p, boards)| {
                self.sort_transparent(
                    batches,
                    spheres,
                    submission,
                    transforms,
                    origins,
                    &Frustum::from_view_proj(p.frame.view_proj),
                    p.position,
                    p.frame.forward,
                    p.layers,
                    &sprites::depths(&boards.0),
                )
                .0
            })
            .collect();
        // Each view's opaque meshes on its layers: under GPU culling a pass
        // of its own (its planes and layers, as the planar reflection's),
        // drawn indirect; on the CPU path culled here.
        let mut view_views: Vec<u32> = Vec::new();
        let view_lists: Vec<DrawList> = view_plans
            .iter()
            .map(|p| {
                // A view with a depth prepass (a depth texture, soft
                // particles: views.rs `write`) keeps its list.
                let depth = particle_frame.soft
                    || matches!(p.dest, views::Dest::Texture { depth: Some(_), .. });
                let view = if depth {
                    0
                } else {
                    vis.meshlet_view(
                        meshlet_gpu,
                        batches,
                        meshlet_gpu::MeshletView::seen(
                            Frustum::from_view_proj(p.frame.view_proj).planes(),
                            on_layers(p.layers),
                            p.position,
                            p.frame.view_proj,
                            p.size.1,
                        ),
                    )
                };
                view_views.push(view);
                let frustum = Frustum::from_view_proj(p.frame.view_proj);
                let include = |b: &Batch| {
                    !b.transparent && b.layers & p.layers != 0 && !(view > 0 && in_meshlet(b))
                };
                if gpu_cull {
                    let g = vis.gpu_pass(
                        cull_gpu::PassKind::Reflection,
                        &frustum,
                        None,
                        Some(p.layers),
                    );
                    return indirect_list(batches, cull_n, g, include);
                }
                self.cull(
                    batches,
                    spheres,
                    transforms,
                    origins,
                    cull_inside,
                    &frustum,
                    true,
                    include,
                )
                .0
            })
            .collect();
        PlanViewListsOutputs {
            view_blended,
            view_views,
            view_lists,
        }
    }
}
