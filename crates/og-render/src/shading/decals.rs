//! Decals in the lit passes: each [`crate::effects::Decal`] packed as a GPU
//! record with its bounding sphere ([`pack_decals`]), the records and their
//! clusters in two storage buffers of the effect group ([`DecalBuffers`]:
//! the records, binding 5, and their clusters, binding 6, binned by
//! `clusters::DecalBins`) and those bindings' layout
//! ([`decal_layout_entries`]). pbr.wgsl projects them.

use crate::effects::{Decal, MAX_DECALS};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DecalGpu {
    inv: [[f32; 4]; 4],
    axis: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
    emissive: [f32; 4],
    normal_rect: [f32; 4],
    /// x: the normal map's scale (0 without one).
    normal: [f32; 4],
}

/// Width of the facing fade at a decal's cut-off angle, as a cosine step.
pub(crate) const DECAL_FADE: f32 = 0.25;

/// Packs the first [`MAX_DECALS`] decals into `out` and their bounding
/// spheres (world centre, radius) into `spheres`.
pub(crate) fn pack_decals(
    decals: &[Decal],
    out: &mut Vec<DecalGpu>,
    spheres: &mut Vec<(Vec3, f32)>,
) {
    out.clear();
    spheres.clear();
    for d in decals.iter().take(MAX_DECALS) {
        let axis = d.transform.transform_vector3(Vec3::Z).normalize_or(Vec3::Y);
        let cut = d.angle_fade_degrees.clamp(0.0, 180.0).to_radians().cos();
        let [u0, v0, u1, v1] = d.normal_rect;
        let mapped = u1 > u0 && v1 > v0;
        out.push(DecalGpu {
            inv: d.transform.inverse().to_cols_array_2d(),
            axis: axis.extend(cut - DECAL_FADE).to_array(),
            uv_rect: d.uv_rect,
            color: d.color,
            emissive: d.emissive.extend(d.roughness).to_array(),
            normal_rect: d.normal_rect,
            normal: [if mapped { d.normal_scale } else { 0.0 }, 0.0, 0.0, 0.0],
        });
        // The box's half diagonal around its centre.
        let half = 0.5
            * (d.transform.x_axis.truncate().length_squared()
                + d.transform.y_axis.truncate().length_squared()
                + d.transform.z_axis.truncate().length_squared())
            .sqrt();
        spheres.push((d.transform.w_axis.truncate(), half));
    }
}

/// The fx group's entries for the decal buffers (bindings 5 and 6).
pub(crate) fn decal_layout_entries() -> [wgpu::BindGroupLayoutEntry; 2] {
    [5, 6].map(|binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    })
}

/// The decal records and cells on the GPU, grown (to a power of two) as a
/// frame needs; `generation` moves when a buffer is made again, so the
/// effect groups that bind them are made again too.
pub(crate) struct DecalBuffers {
    pub records: wgpu::Buffer,
    pub cells: wgpu::Buffer,
    pub generation: u32,
}

fn storage(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl DecalBuffers {
    pub fn new(device: &wgpu::Device) -> DecalBuffers {
        DecalBuffers {
            records: storage(device, "decals", 16 * size_of::<DecalGpu>() as u64),
            cells: storage(
                device,
                "decal cells",
                4 * crate::clusters::CLUSTERS as u64 + 1024,
            ),
            generation: 0,
        }
    }

    /// Writes the records and cells, growing either buffer that is short.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        records: &[DecalGpu],
        cells: &[u32],
    ) {
        let grow = |b: &mut wgpu::Buffer, bytes: u64, label: &str| {
            if bytes > b.size() {
                *b = storage(device, label, bytes.next_power_of_two());
                true
            } else {
                false
            }
        };
        let r = bytemuck::cast_slice::<DecalGpu, u8>(records);
        let c = bytemuck::cast_slice::<u32, u8>(cells);
        let a = grow(&mut self.records, r.len() as u64, "decals");
        let b = grow(&mut self.cells, c.len() as u64, "decal cells");
        if a || b {
            self.generation = self.generation.wrapping_add(1);
        }
        if !r.is_empty() {
            queue.write_buffer(&self.records, 0, r);
        }
        if !c.is_empty() {
            queue.write_buffer(&self.cells, 0, c);
        }
    }

    /// The two bind group entries (bindings 5 and 6).
    pub fn entries(&self) -> [wgpu::BindGroupEntry<'_>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: 5,
                resource: self.records.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: self.cells.as_entire_binding(),
            },
        ]
    }
}
