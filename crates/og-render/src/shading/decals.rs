//! Decals in the lit passes: each [`crate::effects::Decal`] packed as a GPU
//! record with its bounding sphere ([`pack_decals`]), the records and their
//! clusters (binned by `clusters::DecalBins`) in one storage buffer of the
//! effect group ([`DecalBuffers`], binding 5) and its layout
//! ([`decal_layout_entries`]). pbr.wgsl projects them. Their textures are
//! in the atlas `shading::decal_atlas` keeps.

use crate::decal_atlas::{DecalAtlas, Key};
use crate::effects::{Decal, MAX_DECALS};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DecalGpu {
    inv: [[f32; 4]; 4],
    axis: [f32; 4],
    /// The image's region of the atlas; below 0 without an image.
    uv_rect: [f32; 4],
    color: [f32; 4],
    emissive: [f32; 4],
    /// The normal map's region of the atlas.
    normal_rect: [f32; 4],
    /// x: the normal map's scale (0 without one), y: wet coverage mode.
    normal: [f32; 4],
}

/// Width of the facing fade at a decal's cut-off angle, as a cosine step.
pub(crate) const DECAL_FADE: f32 = 0.25;

/// What a record without an image carries as its region (decals.wgsl
/// samples nothing for it).
const NO_IMAGE: [f32; 4] = [-1.0; 4];

/// Packs the first [`MAX_DECALS`] decals into `out` and their bounding
/// spheres (world centre, radius) into `spheres`, each texture's region
/// brought into `atlas` (a texture it does not hold is left out).
pub(crate) fn pack_decals(
    decals: &[Decal],
    atlas: Option<&DecalAtlas>,
    out: &mut Vec<DecalGpu>,
    spheres: &mut Vec<(Vec3, f32)>,
) {
    out.clear();
    spheres.clear();
    for d in decals.iter().take(MAX_DECALS) {
        let axis = d.transform.transform_vector3(Vec3::Z).normalize_or(Vec3::Y);
        let cut = d.angle_fade_degrees.clamp(0.0, 180.0).to_radians().cos();
        let region = |texture: Option<crate::TextureHandle>, normal| {
            let index = texture?.index();
            atlas?.region(Key { index, normal }, d.uv_rect)
        };
        let normal_rect = region(d.normal_map, true);
        out.push(DecalGpu {
            inv: d.transform.inverse().to_cols_array_2d(),
            axis: axis.extend(cut - DECAL_FADE).to_array(),
            uv_rect: region(d.texture, false).unwrap_or(NO_IMAGE),
            color: d.color,
            emissive: d.emissive.extend(d.roughness).to_array(),
            normal_rect: normal_rect.unwrap_or_default(),
            normal: [
                if normal_rect.is_some() {
                    d.normal_scale
                } else {
                    0.0
                },
                f32::from(d.wet),
                d.metallic,
                0.0,
            ],
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

// decals.wgsl reads a record as ten vec4s (`DECAL_VEC4S`).
const _: () = assert!(size_of::<DecalGpu>() == 10 * 16);

/// The fx group's entry for the decals' buffer (binding 5).
pub(crate) fn decal_layout_entries() -> [wgpu::BindGroupLayoutEntry; 1] {
    [wgpu::BindGroupLayoutEntry {
        binding: 5,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }]
}

/// The decals' clusters and records in one buffer on the GPU (decals.wgsl
/// `decal_data`: a header, the cells, then the records), grown (to a power
/// of two) as a frame needs; `generation` moves when it is made again, so
/// the effect groups that bind it are made again too.
pub(crate) struct DecalBuffers {
    pub data: wgpu::Buffer,
    bytes: Vec<u8>,
    prev_bytes: Vec<u8>,
    pub generation: u32,
}

fn storage(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some("decals"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    )
}

impl DecalBuffers {
    pub fn new(device: &wgpu::Device) -> DecalBuffers {
        DecalBuffers {
            data: storage(
                device,
                16 + 4 * crate::clusters::CLUSTERS as u64
                    + 1024
                    + 16 * size_of::<DecalGpu>() as u64,
            ),
            bytes: Vec::new(),
            prev_bytes: Vec::new(),
            generation: 0,
        }
    }

    /// Writes the cells and records, growing the buffer when it is short and
    /// skipping the GPU upload when the packed payload is unchanged.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        records: &[DecalGpu],
        cells: &[u32],
    ) {
        let first = (4 + cells.len()).div_ceil(4) as u32;
        let wet_count = records.iter().filter(|r| r.normal[1] > 0.5).count() as u32;
        self.bytes.clear();
        self.bytes
            .extend_from_slice(bytemuck::bytes_of(&[first, wet_count, 0, 0]));
        self.bytes.extend_from_slice(bytemuck::cast_slice(cells));
        self.bytes.resize(first as usize * 16, 0);
        self.bytes.extend_from_slice(bytemuck::cast_slice(records));
        let len = self.bytes.len() as u64;
        let resized = len > self.data.size();
        if resized {
            self.data = storage(device, len.next_power_of_two());
            self.generation = self.generation.wrapping_add(1);
        } else if self.bytes == self.prev_bytes {
            return;
        }
        crate::traffic::write_buffer(queue, &self.data, 0, &self.bytes);
        std::mem::swap(&mut self.bytes, &mut self.prev_bytes);
    }

    /// The bind group entry (binding 5).
    pub fn entries(&self) -> [wgpu::BindGroupEntry<'_>; 1] {
        [wgpu::BindGroupEntry {
            binding: 5,
            resource: self.data.as_entire_binding(),
        }]
    }
}
