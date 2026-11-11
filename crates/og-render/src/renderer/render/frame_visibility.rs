//! The frame's visibility stage: the GPU cull's passes and the meshlet
//! dispatch's views, registered by every view that culls in the frame's
//! order. The GPU cull is one dispatch for them all, and the order of
//! registration is the layout of its buffers, so the views register here
//! in table order rather than each keeping its own.

use super::*;

pub(super) struct FrameVisibility {
    /// The main camera's layers (views.rs).
    camera_layers: u32,
    /// The GPU cull's passes, in registration order.
    pub cull_passes: Vec<cull_gpu::CullPass>,
    /// The meshlet dispatch's views: the camera's, then one per
    /// reflection, view, probe face and shadow pass that fits.
    pub meshlet_views: Vec<meshlet_gpu::MeshletView>,
    /// The work items the dispatch writes so far, and at most.
    meshlet_items: u64,
    meshlet_max: u64,
    /// The camera's view without its batches, which a shadow view starts
    /// from.
    camera_seen: Option<meshlet_gpu::MeshletView>,
}

impl FrameVisibility {
    pub(super) fn new(
        camera_layers: u32,
        (meshlet_items, meshlet_max): (u64, u64),
    ) -> FrameVisibility {
        FrameVisibility {
            camera_layers,
            cull_passes: Vec::new(),
            meshlet_views: Vec::new(),
            meshlet_items,
            meshlet_max,
            camera_seen: None,
        }
    }

    /// Registers a GPU culling pass and answers its index. `layers`: the
    /// pass's three.js layer mask where it is not its kind's (a secondary
    /// view's own).
    pub(super) fn gpu_pass(
        &mut self,
        kind: cull_gpu::PassKind,
        frustum: &Frustum,
        counted: Option<Vec<u32>>,
        layers: Option<u32>,
    ) -> u32 {
        self.cull_passes.push(cull_gpu::CullPass {
            counted,
            kind,
            planes: frustum.planes().to_vec(),
            // Shadow maps take every caster; the camera's passes its
            // layers (views.rs), as the CPU lists' `include`.
            layers: layers.unwrap_or(match kind {
                cull_gpu::PassKind::Shadow => u32::MAX,
                _ => self.camera_layers,
            }),
        });
        self.cull_passes.len() as u32 - 1
    }

    /// The camera's meshlet view, the dispatch's first.
    pub(super) fn camera(&mut self, view: meshlet_gpu::MeshletView) {
        self.camera_seen = Some(meshlet_gpu::MeshletView {
            taken: Vec::new(),
            ..view.clone()
        });
        self.meshlet_views.push(view);
    }

    /// Registers a meshlet view while its items fit, and answers its index
    /// (0: none, that pass keeps its list).
    pub(super) fn meshlet_view(
        &mut self,
        gpu: Option<&meshlet_gpu::MeshletGpu>,
        batches: &[Batch],
        v: meshlet_gpu::MeshletView,
    ) -> u32 {
        if v.taken.is_empty() {
            return 0;
        }
        match gpu.and_then(|m| m.items(batches, &v.taken, false)) {
            Some(n) if self.meshlet_items + n <= self.meshlet_max => {
                self.meshlet_items += n;
                self.meshlet_views.push(v);
                self.meshlet_views.len() as u32 - 1
            }
            _ => 0,
        }
    }

    /// A shadow pass's meshlet view: `casters`, the shadow casters the
    /// path takes, through `f`.
    pub(super) fn shadow_view(
        &mut self,
        gpu: Option<&meshlet_gpu::MeshletGpu>,
        batches: &[Batch],
        casters: &[u32],
        f: &Frustum,
    ) -> u32 {
        let v = meshlet_gpu::MeshletView::shadow(
            self.camera_seen.as_ref().expect("the camera's view first"),
            f.planes(),
            casters.to_vec(),
        );
        self.meshlet_view(gpu, batches, v)
    }
}
