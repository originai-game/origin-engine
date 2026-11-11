//! The passes the frame encodes before its render graph's: the compute
//! steps the graph's passes read (the crowds' cull, the wind, the
//! simulated meshes, the terrain's corners, the grass, the scatter). They
//! are not graph nodes. [`PRE_GRAPH`] lists them in the frame encoder's
//! order with what each needs; [`pre_graph`] encodes one where the device
//! has its needs, checked against them as it encodes.

use super::*;

/// What a pass before the graph needs of the device.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreGraphDesc {
    /// Its name, as [`Renderer::absent_passes`] reports it.
    pub name: &'static str,
    pub needs: Caps,
}

/// The crowds' cull (crowd_gpu.rs): their members culled and their
/// draws' arguments written on the GPU, drawn indirect with a first
/// instance.
pub(crate) const CROWD_CULL: PreGraphDesc = PreGraphDesc {
    name: "cull: crowds",
    needs: Caps::COMPUTE
        .union(Caps::INDIRECT)
        .union(Caps::INDIRECT_FIRST_INSTANCE),
};

/// The wind field's and the trample map's steps (wind_field.rs,
/// trample.rs), before every pass that reads them.
pub(crate) const WIND: PreGraphDesc = PreGraphDesc {
    name: "wind",
    needs: Caps::COMPUTE,
};

/// The visual bodies' and the visual fluids' steps (visual_sim.rs,
/// visual_fluid.rs): simulated in compute, drawn from storage.
pub(crate) const VISUAL: PreGraphDesc = PreGraphDesc {
    name: "visual simulation",
    needs: Caps::COMPUTE.union(Caps::VERTEX_STORAGE),
};

/// The deforming surfaces' and the garments' vertices (surface.rs,
/// garment_gpu.rs), written in compute before every pass that draws them.
pub(crate) const SURFACES: PreGraphDesc = PreGraphDesc {
    name: "surfaces",
    needs: Caps::COMPUTE.union(Caps::VERTEX_STORAGE),
};

/// Every terrain level's corners (terrain/renderer.rs), before every
/// pass that draws them. Terrain has no path without compute.
pub(crate) const TERRAIN_CORNERS: PreGraphDesc = PreGraphDesc {
    name: "terrain corners",
    needs: Caps::COMPUTE,
};

/// The grass's blades (grass.rs), generated in compute before every
/// pass that draws them.
pub(crate) const GRASS: PreGraphDesc = PreGraphDesc {
    name: "grass",
    needs: Caps::COMPUTE,
};

/// The scattered copies placed on the GPU this frame, then those the
/// main camera sees, culled on the GPU (scatter/draw.rs): nothing
/// without compute.
pub(crate) const SCATTER: PreGraphDesc = PreGraphDesc {
    name: "scatter",
    needs: Caps::COMPUTE,
};

/// The passes before the graph, in the frame encoder's order.
pub(crate) const PRE_GRAPH: &[PreGraphDesc] = &[
    CROWD_CULL,
    WIND,
    VISUAL,
    SURFACES,
    TERRAIN_CORNERS,
    GRASS,
    SCATTER,
];

/// Encodes the pass `desc` with `encode` where `caps` has its needs, the
/// encoder checking what it opens against them (`Encoder::expect`), and
/// against the device alone again after it.
pub(super) fn pre_graph<R>(
    encoder: &mut Encoder,
    caps: Caps,
    desc: PreGraphDesc,
    encode: impl FnOnce(&mut Encoder) -> R,
) -> Option<R> {
    if !caps.contains(desc.needs) {
        return None;
    }
    encoder.expect(caps, desc.needs);
    let r = encode(encoder);
    encoder.expect(caps, caps);
    Some(r)
}

/// The crowds' cull, in the frame's (timed) encoder: after GPU
/// culling's commands and the pyramid's last build (the meshlets'
/// first phase draws into its depth), before any pass draws them.
/// It opens no graph pass, so its queries are listed as their own
/// pass ("cull: crowds", the first of the frame's GPU passes).
#[allow(clippy::too_many_arguments)]
pub(super) fn encode_crowd_cull(
    encoder: &mut Encoder,
    (device, queue, (width, height)): (&wgpu::Device, &wgpu::Queue, (u32, u32)),
    gc: &cull_gpu::GpuCulling,
    cc: &mut crowd_gpu::CrowdCulling,
    scene: &RenderScene,
    crowd_frame: &crate::crowd::CrowdFrame,
    cull_passes: &[cull_gpu::CullPass],
    (camera, cam): (&Camera, &frame::CameraFrame),
    crowd_previous: Option<u32>,
) -> Option<std::ops::Range<u32>> {
    let q0 = encoder.used();
    let (half_tan, ortho_height) = match scene.camera.projection {
        Projection::Perspective { fov_y_degrees } => {
            (Some((fov_y_degrees.to_radians() * 0.5).tan()), 0.0)
        }
        Projection::Orthographic { height } => (None, height),
    };
    let index_counts = crowd_frame.index_counts();
    cc.encode(
        encoder,
        device,
        queue,
        crowd_frame,
        &index_counts,
        cull_passes,
        &crowd_gpu::CrowdView {
            position: camera.position,
            half_tan,
            ortho_height,
            view_height: height as f32,
            view_proj: cam.jittered_view_proj,
            size: (width, height),
            pyramid: gc.pyramid(),
        },
        crowd_previous.map_or(0, |at| at + 1),
        (&gc.args, gc.crowd_counts_word()),
    );
    (encoder.timed() && encoder.used() > q0).then(|| q0..encoder.used())
}
