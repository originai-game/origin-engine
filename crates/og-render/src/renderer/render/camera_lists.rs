//! The main camera's lists: the frame's visibility stage opened, the
//! camera's opaque list (the GPU cull's main and occluder passes, or culled
//! on the CPU), its meshlet view, its transparent meshes back to front, and
//! whether its OIT pass runs.

use super::*;

pub(super) struct PlanCameraListsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub batches: &'a [Batch],
    pub gpu_cull: bool,
    pub cull_n: u32,
    pub meshlet_items: u64,
    pub meshlet_max: u64,
    pub meshlet_taken: Vec<u32>,
    pub spheres: &'a [(Vec3, f32)],
    pub transforms: &'a mut Vec<instances::GpuTransform>,
    pub origins: &'a mut Vec<u32>,
    pub submission: &'a [u32],
    pub sprite_frame: &'a sprites::SpriteFrame,
    pub skinned_visible: &'a [Batch],
    pub skinned_culled: u32,
    pub oit_frame: Option<oit::Oit>,
    pub oit_candidates: bool,
}

pub(super) struct PlanCameraListsOutputs {
    pub vis: FrameVisibility,
    pub cull_inside: CullScratch,
    pub mask: u32,
    pub oit_mode: Option<oit::Oit>,
    pub main_list: DrawList,
    pub culled: u32,
    pub blended_list: DrawList,
}

impl Renderer {
    /// Culls and sorts the main camera's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_camera_lists(
        &mut self,
        PlanCameraListsInputs {
            scene,
            camera,
            cam,
            view_frustum,
            batches,
            gpu_cull,
            cull_n,
            meshlet_items,
            meshlet_max,
            mut meshlet_taken,
            spheres,
            transforms,
            origins,
            submission,
            sprite_frame,
            skinned_visible,
            skinned_culled,
            oit_frame,
            oit_candidates,
        }: PlanCameraListsInputs<'_>,
    ) -> PlanCameraListsOutputs {
        // The GPU cull's passes and the meshlet dispatch's views
        // (frame_visibility.rs), registered by every view below in order.
        let mut vis = FrameVisibility::new(scene.frame.camera_layers, (meshlet_items, meshlet_max));
        // Lent to every CPU cull of this frame, given back after the last.
        let mut cull_inside = std::mem::take(&mut self.visibility.cull_inside);
        cull_inside.whole.clear();
        // Occlusion needs a batch that can occlude and an earlier frame's
        // camera visibility (its visible occluders); without them the
        // occluders' depth, the pyramid and the occlusion test are skipped.
        let occlusion = gpu_cull
            && !self.visibility.lean
            && self
                .visibility
                .gpu_cull
                .as_ref()
                .is_some_and(|gc| gc.primed())
            && batches
                .iter()
                .any(|b| self.cache.materials[b.material].occluder);
        // The main camera's layers (views.rs).
        let mask = scene.frame.camera_layers;
        // The cull record's verdicts on the sorted instances: the CPU
        // cull's and the transparent sort's below (the GPU's land later).
        let mut record = self
            .visibility
            .records
            .as_mut()
            .and_then(|r| r.frame.take());
        let mut verdict = |k: usize, c: Culled| {
            if let Some(v) = record
                .as_mut()
                .and_then(|f| f.record.instances.get_mut(submission[k] as usize))
            {
                *v = c;
            }
        };
        let (main_list, mut culled) = if gpu_cull {
            let g = vis.gpu_pass(cull_gpu::PassKind::Main, view_frustum, None, None);
            if occlusion {
                vis.gpu_pass(cull_gpu::PassKind::Occluders, view_frustum, None, None);
            }
            let mut list = indirect_list(batches, cull_n, g, view_frustum, |b| {
                !b.transparent && b.layers & mask != 0
            });
            // The meshlet path draws its batches itself; the camera's
            // layers still decide (a batch off them draws nothing there).
            meshlet_taken.retain(|&b| batches[b as usize].layers & mask != 0);
            list.ranges
                .retain(|r| meshlet_taken.binary_search(&r.0).is_err());
            (list, 0)
        } else {
            let culled = self.cull(
                batches,
                spheres,
                transforms,
                origins,
                &mut cull_inside,
                view_frustum,
                true,
                |b| !b.transparent && b.layers & mask != 0,
            );
            for b in batches.iter().filter(|b| !b.transparent) {
                let first = b.first as usize;
                let members = &cull_inside.inside[first..first + b.count as usize];
                for (j, &inside) in members.iter().enumerate() {
                    verdict(
                        first + j,
                        match (b.layers & mask != 0, inside) {
                            (false, _) => Culled::Layer,
                            (true, true) => Culled::Drawn,
                            (true, false) => Culled::OutsideView,
                        },
                    );
                }
            }
            culled
        };
        for b in batches
            .iter()
            .filter(|b| b.transparent && b.layers & mask == 0)
        {
            for k in b.first as usize..(b.first + b.count) as usize {
                verdict(k, Culled::Layer);
            }
        }
        // The meshlet dispatch's views: the camera's, then one per
        // reflection, view, probe face and shadow pass that fits
        // (`meshlet_view`: its index, 0 none: that pass keeps its list).
        vis.camera(meshlet_gpu::MeshletView::camera(
            &scene.frame.camera,
            view_frustum.planes(),
            std::mem::take(&mut meshlet_taken),
            self.height,
        ));
        let (blended_list, blended_culled) = self.sort_transparent(
            batches,
            spheres,
            submission,
            transforms,
            origins,
            view_frustum,
            camera.position,
            cam.forward,
            mask,
            &sprites::depths(&sprite_frame.billboards),
            |k, inside| {
                verdict(
                    k,
                    match inside {
                        true => Culled::Drawn,
                        false => Culled::OutsideView,
                    },
                )
            },
        );
        if let (Some(records), Some(f)) = (self.visibility.records.as_mut(), record) {
            records.frame = Some(f);
        }
        culled += blended_culled + skinned_culled;
        // The OIT pass runs when the main camera draws a blended surface
        // after culling (`oit_frame` says the frame may).
        let oit_mode = oit_frame.filter(|_| {
            blended_list
                .ranges
                .iter()
                .any(|r| batches[r.0 as usize].blend)
                || skinned_visible.iter().any(|b| b.transparent && b.blend)
                || oit_candidates
        });
        PlanCameraListsOutputs {
            vis,
            cull_inside,
            mask,
            oit_mode,
            main_list,
            culled,
            blended_list,
        }
    }
}
