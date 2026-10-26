//! The per-frame effect uniform of the lit passes (`struct Fx` in
//! pbr.wgsl): unjittered and previous view-projections for motion vectors,
//! the target size, ambient occlusion and the frame's decals.

use crate::effects::{Decal, MAX_DECALS};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DecalGpu {
    inv: [[f32; 4]; 4],
    axis: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
    emissive: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FxUniform {
    pub view_proj_unjittered: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub screen: [f32; 4],
    pub ao: [f32; 4],
    pub decal_info: [f32; 4],
    pub planar: [f32; 4],
    pub decals: [DecalGpu; MAX_DECALS],
}

/// Width of the facing fade at a decal's cut-off angle, as a cosine step.
const DECAL_FADE: f32 = 0.25;

/// Packs the first [`MAX_DECALS`] decals; returns the count.
pub(crate) fn pack_decals(decals: &[Decal], out: &mut [DecalGpu; MAX_DECALS]) -> u32 {
    let n = decals.len().min(MAX_DECALS);
    for (d, g) in decals.iter().take(n).zip(out.iter_mut()) {
        let axis = d.transform.transform_vector3(Vec3::Z).normalize_or(Vec3::Y);
        let cut = d.angle_fade_degrees.clamp(0.0, 180.0).to_radians().cos();
        *g = DecalGpu {
            inv: d.transform.inverse().to_cols_array_2d(),
            axis: axis.extend(cut - DECAL_FADE).to_array(),
            uv_rect: d.uv_rect,
            color: d.color,
            emissive: d.emissive.extend(d.roughness).to_array(),
        };
    }
    n as u32
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
        decals: [DecalGpu::default(); MAX_DECALS],
    }
}
