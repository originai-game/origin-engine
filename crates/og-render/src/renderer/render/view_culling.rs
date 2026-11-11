//! View culling stage.

use super::*;

impl Renderer {
    pub(super) fn prepare_view_culling(
        &mut self,
        scene: &RenderScene,
        &camera: &Camera,
        skinned: &mut SkinnedFrame,
    ) -> (Vec<views::ViewPlan>, Vec<Vec<Batch>>) {
        // Secondary cameras (views.rs): the views due this frame, each
        // with its skinned instances in view on its layers.
        let view_plans = views::plan(scene, &camera, (self.out_width, self.out_height), |i| {
            self.views.stale || self.views.slots.get(i).is_none_or(|s| s.is_none())
        });
        self.views.stale = false;
        self.views.retain(scene.frame.views.len());
        let view_skinned: Vec<Vec<Batch>> = view_plans
            .iter()
            .map(|p| {
                let f = Frustum::from_view_proj(p.frame.view_proj);
                Self::cull_skinned(skinned, &f, p.layers).0
            })
            .collect();
        (view_plans, view_skinned)
    }

    pub(super) fn prepare_probe_culling(
        &self,
        probe_frame: &probes::ProbeFrame,
        active_probes: &[ReflectionProbe],
        &camera: &Camera,
        skinned: &mut SkinnedFrame,
    ) -> (Vec<[frame::CameraFrame; 6]>, Vec<Vec<Batch>>) {
        // Reflection probes: the captures due this frame, each face's
        // skinned instances in view in their pose (probes.rs).
        let probe_faces: Vec<[frame::CameraFrame; 6]> = probe_frame
            .due
            .iter()
            .map(|&(_, i)| probes::face_frames(&camera, active_probes[i].position, camera.far))
            .collect();
        let probe_skinned: Vec<Vec<Batch>> = probe_faces
            .iter()
            .flatten()
            .map(|f| {
                let frustum = Frustum::from_view_proj(f.view_proj);
                Self::cull_skinned(skinned, &frustum, u32::MAX).0
            })
            .collect();
        (probe_faces, probe_skinned)
    }
}
