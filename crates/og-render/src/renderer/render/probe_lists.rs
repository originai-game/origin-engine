//! The reflection probes' lists: each due capture's six faces culled (a
//! face with a depth prepass keeps its whole list), and their transparent
//! meshes back to front from the probe.

use super::*;

pub(super) struct PlanProbeListsInputs<'a> {
    pub batches: &'a [Batch],
    pub vis: &'a mut FrameVisibility,
    pub gpu_cull: bool,
    pub spheres: &'a [(Vec3, f32)],
    pub transforms: &'a mut Vec<instances::GpuTransform>,
    pub origins: &'a mut Vec<u32>,
    pub cull_inside: &'a mut CullScratch,
    pub submission: &'a [u32],
    pub in_meshlet: &'a dyn Fn(&Batch) -> bool,
    pub probe_faces: &'a [[frame::CameraFrame; 6]],
    pub active_probes: &'a [probes::ReflectionProbe],
    pub probe_frame: &'a probes::ProbeFrame,
    pub face_depth: bool,
    pub meshlet_all: &'a [u32],
    pub face_boards: &'a [sprites::Billboards],
}

pub(super) struct PlanProbeListsOutputs {
    pub probe_lists: Vec<DrawList>,
    pub probe_blended: Vec<DrawList>,
    pub probe_views: Vec<u32>,
}

impl Renderer {
    /// Culls and sorts each due probe face's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_probe_lists(
        &self,
        PlanProbeListsInputs {
            batches,
            vis,
            gpu_cull,
            spheres,
            transforms,
            origins,
            cull_inside,
            submission,
            in_meshlet,
            probe_faces,
            active_probes,
            probe_frame,
            face_depth,
            meshlet_all,
            face_boards,
        }: PlanProbeListsInputs<'_>,
    ) -> PlanProbeListsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        // Reflection probes: each due capture's faces culled, their
        // transparent meshes sorted back to front from the probe (probes.rs).
        let mut probe_lists: Vec<DrawList> = Vec::new();
        let mut probe_blended: Vec<DrawList> = Vec::new();
        let mut probe_views: Vec<u32> = Vec::new();
        for (d, cams) in probe_faces.iter().enumerate() {
            let probe = &active_probes[probe_frame.due[d].1];
            let eye = probe.position;
            for (face, f) in cams.iter().enumerate() {
                let frustum = Frustum::from_view_proj(f.view_proj);
                // A face with a depth prepass keeps its list (the prepass
                // draws it).
                let view = if face_depth {
                    0
                } else {
                    vis.meshlet_view(
                        meshlet_gpu,
                        batches,
                        meshlet_gpu::MeshletView::seen(
                            frustum.planes(),
                            meshlet_all.to_vec(),
                            eye,
                            f.view_proj,
                            probe.face_size(),
                        ),
                    )
                };
                probe_views.push(view);
                probe_lists.push(
                    self.cull(
                        batches,
                        spheres,
                        transforms,
                        origins,
                        cull_inside,
                        &frustum,
                        !gpu_cull,
                        |b| !b.transparent && !(view > 0 && in_meshlet(b)),
                    )
                    .0,
                );
                probe_blended.push(
                    self.sort_transparent(
                        batches,
                        spheres,
                        submission,
                        transforms,
                        origins,
                        &frustum,
                        eye,
                        f.forward,
                        u32::MAX,
                        &sprites::depths(&face_boards[6 * d + face].0),
                    )
                    .0,
                );
            }
        }
        PlanProbeListsOutputs {
            probe_lists,
            probe_blended,
            probe_views,
        }
    }
}
