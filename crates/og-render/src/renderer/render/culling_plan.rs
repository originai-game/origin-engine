//! The frame's culling decisions: the passes it can cull at most, the
//! batches the meshlet path takes and its work items, whether the GPU
//! culls (and takes the skinned batches), the instance regions' length, the
//! auxiliary instance records, and the meshlet batches the shadow casters
//! and lit views take.

use super::*;

pub(super) struct PlanCullingInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub view_plans: &'a [views::ViewPlan],
    pub reflection: Option<&'a camera::PlanarReflection>,
    pub shelter_redraw: bool,
    pub sorted: u32,
    pub skinned_n: u32,
    pub crowd_gpu: bool,
    pub object_space: bool,
    pub skinned_batches: &'a [Batch],
    pub skinned_visible: &'a mut Vec<Batch>,
    pub skinned_culled: &'a mut u32,
    pub previous_valid: bool,
    pub blur_motion: bool,
    pub temporal: bool,
}

pub(super) struct PlanCullingOutputs {
    pub max_passes: usize,
    pub meshlet_taken: Vec<u32>,
    pub meshlet_max: u64,
    pub meshlet_items: u64,
    pub gpu_cull: bool,
    pub skinned_gpu: bool,
    pub cull_n: u32,
    pub object_motion: bool,
    pub auxiliary_instances: bool,
    pub meshlet_casters: Vec<u32>,
    pub meshlet_all: Vec<u32>,
    pub meshlet_firsts: Vec<u32>,
}

impl Renderer {
    /// Decides how the frame culls: GPU or CPU, and what the meshlet path takes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_culling(
        &mut self,
        PlanCullingInputs {
            scene,
            batches,
            view_plans,
            reflection,
            shelter_redraw,
            sorted,
            skinned_n,
            crowd_gpu,
            object_space,
            skinned_batches,
            skinned_visible,
            skinned_culled,
            previous_valid,
            blur_motion,
            temporal,
        }: PlanCullingInputs<'_>,
    ) -> PlanCullingOutputs {
        // The passes this frame can cull at most: the camera, the
        // occluders, the planar reflection, the cascades and the shadowed
        // local lights' faces. The buffers must fit them; `encode` sizes
        // them for the frame's actual passes and regrows when that grows.
        let max_passes = 2
            + view_plans.len()
            + shelter_redraw as usize
            + reflection.is_some() as usize
            + if scene.lighting.light.cast_shadows {
                self.frame_cascades(scene) as usize
            } else {
                0
            }
            + scene
                .lighting
                .lights
                .iter()
                .filter(|l| l.cast_shadows && self.kept_kind(l))
                .map(|l| match l.kind {
                    LightKind::Point => self.info.point_shadows.layers(),
                    LightKind::Spot { .. } => 1,
                    LightKind::Rect { two_sided, .. } => 1 + two_sided as usize,
                })
                .sum::<usize>()
                .min(shadows::capacity(
                    self.info.shadowed_lights as usize,
                    self.info.point_shadows,
                ));
        // The batches the meshlet path draws in the camera pass. It follows
        // GPU culling, so it runs in a frame that culls on the GPU anyway:
        // by its instances (`Culling::Auto` from GPU_CULLING_MIN_INSTANCES,
        // `Culling::Gpu` always) or for its crowds. Below that a clustered
        // batch draws as any mesh, culled on the CPU, at the level
        // og-runtime picked for its model, as the meshlet path draws it.
        // Never with stylise's object pass, which redraws the camera list's
        // instances. Its cluster pools (meshlet_gpu.rs) are made by the
        // first frame that runs it.
        let meshlet_on = self.visibility.indirect
            && (self.gpu_culls(sorted + skinned_n) || crowd_gpu)
            && !object_space
            && scene.objects.meshlets
            && scene.mesh_clusters_all().iter().any(Option::is_some);
        if meshlet_on {
            let limits = self.device.limits();
            let max = limits
                .max_storage_buffer_binding_size
                .min(limits.max_buffer_size);
            self.visibility
                .meshlet
                .get_or_insert_with(|| {
                    meshlet_gpu::MeshletGpu::new(
                        &self.device,
                        max,
                        self.world.wind_field.lean_view.clone(),
                    )
                })
                .sync(&self.device, &self.queue, scene);
        }
        let mut meshlet_taken: Vec<u32> = match self.visibility.meshlet.as_ref() {
            Some(m) if meshlet_on => batches
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    meshlet_gpu::takes(
                        scene,
                        b,
                        m.clustered(b.mesh),
                        self.lights.lightmaps.group(b.mesh).is_some(),
                    ) && !self.cache.materials[b.material].wire_only
                })
                .map(|(i, _)| i as u32)
                .collect(),
            _ => Vec::new(),
        };
        // The work items the dispatch writes, within the device's storage
        // binding limit: the camera's first, then each shadow view while
        // they fit (a view past it culls and draws its casters as before).
        let meshlet_max = self
            .visibility
            .meshlet
            .as_ref()
            .map_or(0, |m| m.max_items());
        let mut meshlet_items = match self.visibility.meshlet.as_ref() {
            Some(m) => m.items(batches, &meshlet_taken, true),
            None => None,
        }
        .unwrap_or(u64::MAX);
        if meshlet_items > meshlet_max {
            meshlet_taken.clear();
            meshlet_items = 0;
        }
        // Crowds on the GPU path (`crowd_gpu`, checked to fit) turn GPU
        // culling on under `Culling::Auto` however few the instances.
        let gpu_cull = (self.gpu_culls(sorted + skinned_n) || crowd_gpu)
            && (!batches.is_empty() || skinned_n > 0 || crowd_gpu)
            && {
                let limits = self.device.limits();
                let gc = self.visibility.gpu_cull.get_or_insert_with(|| {
                    cull_gpu::GpuCulling::new(
                        &self.device,
                        limits
                            .max_storage_buffer_binding_size
                            .min(limits.max_buffer_size),
                        self.info.indirect_per_draw,
                    )
                });
                gc.fits(sorted, skinned_n, max_passes)
            };
        if !gpu_cull {
            meshlet_taken.clear();
        }
        // Under GPU culling every skinned batch is the camera's list, each
        // drawing what the camera pass kept (`draw_skinned`'s `cull_pass`); the
        // instance regions are as long as both kinds together.
        let skinned_gpu = gpu_cull && skinned_n > 0;
        let cull_n = sorted + if skinned_gpu { skinned_n } else { 0 };
        if skinned_gpu {
            *skinned_visible = skinned_batches.iter().map(|b| Batch { ..*b }).collect();
            *skinned_culled = 0;
        }
        let object_motion = previous_valid && (blur_motion || temporal);
        // Foliage's auxiliary record carries current sway as well as history,
        // including array materials whose current layer word is occupied.
        let foliage_instances = batches.iter().any(|b| b.foliage);
        let auxiliary_instances = object_motion || foliage_instances;
        // The shadow casters among them: every shadow pass takes each one
        // (on any layer) through its own view of the dispatch.
        let meshlet_casters: Vec<u32> = meshlet_taken
            .iter()
            .copied()
            .filter(|&b| batches[b as usize].cast)
            .collect();
        // Every batch the path takes, on any layer: each lit view takes
        // those on its layers. A list leaves out a batch its view draws
        // (`in_meshlet`, by its instances' first slot).
        let meshlet_all = meshlet_taken.clone();
        let meshlet_firsts: Vec<u32> = {
            let mut f: Vec<u32> = meshlet_all
                .iter()
                .map(|&b| &batches[b as usize])
                .filter(|b| b.count > 0)
                .map(|b| b.first)
                .collect();
            f.sort_unstable();
            f
        };
        PlanCullingOutputs {
            max_passes,
            meshlet_taken,
            meshlet_max,
            meshlet_items,
            gpu_cull,
            skinned_gpu,
            cull_n,
            object_motion,
            auxiliary_instances,
            meshlet_casters,
            meshlet_all,
            meshlet_firsts,
        }
    }
}
