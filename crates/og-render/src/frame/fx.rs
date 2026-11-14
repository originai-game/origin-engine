//! The per-frame effect uniform of the lit passes (`struct Fx` in
//! pbr.wgsl): unjittered and previous view-projections for motion vectors,
//! the target size, ambient occlusion and the frame's decal count; and the
//! effect group, whose last two bindings are the decals'
//! (`shading::decals`).

use crate::shading::decals::{DECAL_FADE, DecalBuffers};
use glam::Mat4;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FxUniform {
    pub view_proj_unjittered: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub screen: [f32; 4],
    pub ao: [f32; 4],
    /// x: decal records (negated: walk them all, the view has no decal
    /// clusters), y: the atlas (0 none, 1 linear, 2 sRGB), z: the cos fade
    /// width, w: free for the skinned motion vectors.
    pub decal_info: [f32; 4],
    pub planar: [f32; 4],
    /// x: 1 when the frame's motion vectors are read (TAA, motion blur,
    /// the reflections' or screen-space GI's reprojection, a custom post):
    /// the prepass then moves each vertex by its material shader's
    /// `vertex_offset` of the frame before too (pbr.wgsl
    /// `prepass_vertex_from`), 0 when nothing reads them.
    pub motion: [f32; 4],
}

pub(crate) fn fx_uniform(view_proj: Mat4, prev: Mat4, size: (u32, u32)) -> FxUniform {
    let (w, h) = (size.0 as f32, size.1 as f32);
    FxUniform {
        view_proj_unjittered: view_proj.to_cols_array_2d(),
        prev_view_proj: prev.to_cols_array_2d(),
        screen: [w, h, 1.0 / w, 1.0 / h],
        ao: [0.0; 4],
        decal_info: [0.0, 0.0, DECAL_FADE, 0.0],
        planar: [0.0; 4],
        motion: [1.0, 0.0, 0.0, 0.0],
    }
}

/// `decal_info.x` of a view without decal clusters (a planar reflection, a
/// render-to-texture view): every record, walked in order.
pub(crate) fn unclustered(u: &mut FxUniform) {
    u.decal_info[0] = -u.decal_info[0].abs();
}

/// An effect group: `resources` at bindings 0.., then the decal buffers.
pub(crate) fn fx_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    resources: &[wgpu::BindingResource],
    decals: &DecalBuffers,
) -> wgpu::BindGroup {
    let mut entries: Vec<_> = resources
        .iter()
        .enumerate()
        .map(|(i, r)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: r.clone(),
        })
        .collect();
    entries.extend(decals.entries());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}
