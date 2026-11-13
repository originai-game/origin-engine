//! The compute the frame encodes before its draws, which the graph's
//! passes read (the crowds' cull, the culling's counts, the wind, the
//! water's fields, the simulated meshes, the terrain's corners, the grass, the scatter): the
//! first nodes of the render graph, in the frame encoder's order. Each
//! kind's needs are its entry in the capability table (schedule.rs
//! `PASSES`); execute.rs encodes them first (`execute_pre_graph`), each
//! where the device has its needs, checked against them as it encodes.

use super::*;

/// The crowds' cull (crowd_gpu.rs): their members culled and their
/// draws' arguments written on the GPU, drawn indirect with a first
/// instance.
pub(crate) const CROWD_CULL: PassDesc = PassDesc {
    pass: FramePass::CrowdCull,
    name: "cull: crowds",
    needs: Caps::COMPUTE
        .union(Caps::INDIRECT)
        .union(Caps::INDIRECT_FIRST_INSTANCE),
};

/// The wind field's and the trample map's steps (wind_field.rs,
/// trample.rs), before every pass that reads them.
pub(crate) const WIND: PassDesc = PassDesc {
    pass: FramePass::Wind,
    name: "wind",
    needs: Caps::COMPUTE,
};

/// The water's fields (water/fields.rs `encode_fields`): the sea's
/// spectrum and foam, its caustics, the wakes and the ripples, before
/// every pass that draws water or reads the caustics.
pub(crate) const WATER_FIELDS: PassDesc = PassDesc {
    pass: FramePass::WaterFields,
    name: "water fields",
    needs: Caps::COMPUTE,
};

/// The visual bodies' and the visual fluids' steps (visual_sim.rs,
/// visual_fluid.rs): simulated in compute, drawn from storage.
pub(crate) const VISUAL: PassDesc = PassDesc {
    pass: FramePass::Visual,
    name: "visual simulation",
    needs: Caps::COMPUTE.union(Caps::VERTEX_STORAGE),
};

/// The deforming surfaces' and the garments' vertices (surface.rs,
/// garment_gpu.rs), written in compute before every pass that draws them.
pub(crate) const SURFACES: PassDesc = PassDesc {
    pass: FramePass::Surfaces,
    name: "surfaces",
    needs: Caps::COMPUTE.union(Caps::VERTEX_STORAGE),
};

/// Every terrain level's corners (terrain/renderer.rs), before every
/// pass that draws them. Terrain has no path without compute.
pub(crate) const TERRAIN_CORNERS: PassDesc = PassDesc {
    pass: FramePass::TerrainCorners,
    name: "terrain corners",
    needs: Caps::COMPUTE,
};

/// The grass's blades (grass.rs), generated in compute before every
/// pass that draws them.
pub(crate) const GRASS: PassDesc = PassDesc {
    pass: FramePass::Grass,
    name: "grass",
    needs: Caps::COMPUTE,
};

/// The scattered copies placed on the GPU this frame, then those the
/// main camera sees, culled on the GPU (scatter/draw.rs): nothing
/// without compute.
pub(crate) const SCATTER: PassDesc = PassDesc {
    pass: FramePass::Scatter,
    name: "scatter",
    needs: Caps::COMPUTE,
};

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
    let (half_tan, ortho_height) = match scene.frame.camera.projection {
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
