//! GPU skinning: skinned meshes carry four joints and weights per vertex
//! ([`SkinData`], added with [`crate::RenderScene::add_skinned_mesh`]);
//! [`SkinnedInstance`]s draw them with a [`JointPalette`] of model-space
//! joint matrices the animation system writes every frame. The palette is a
//! storage buffer on the full tier and an `Rgba32Float` vertex texture on
//! WebGL2 (which has no storage buffers); the vertex shader blends the four
//! matrices of each vertex.

use crate::scene::{MaterialHandle, MeshHandle};
use glam::{Mat4, Vec3};

/// Per-vertex skin of a mesh: four joint indices into the palette and their
/// weights (normalized by the shader; all-zero weights leave the vertex in
/// its bind pose). One entry per vertex of the mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinData {
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
}

/// The joints of one skeleton this frame: for each joint, the matrix taking
/// the mesh's bind-pose (model) space to the posed model space, i.e. the
/// joint's model-space transform times its inverse bind matrix (glTF's
/// `jointMatrix` without the mesh node's inverse). This is exactly what
/// og-anim's `PaletteSink::joint_palette` hands over (`JointPalette::from(joints)`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JointPalette {
    pub matrices: Vec<Mat4>,
}

impl From<&[Mat4]> for JointPalette {
    fn from(joints: &[Mat4]) -> Self {
        JointPalette {
            matrices: joints.to_vec(),
        }
    }
}

impl From<Vec<Mat4>> for JointPalette {
    fn from(matrices: Vec<Mat4>) -> Self {
        JointPalette { matrices }
    }
}

/// The most blendshape (morph) targets a mesh can have (og-anim's `BLENDSHAPES`).
pub const MAX_MORPH_TARGETS: usize = 8;

/// Blendshape targets of a mesh: per target, per vertex, the position (and
/// optionally normal) delta added at the target's weight before skinning
/// (og-cook's `MorphTargets`). At most [`MAX_MORPH_TARGETS`] targets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MorphTargets {
    pub positions: Vec<Vec<Vec3>>,
    pub normals: Option<Vec<Vec<Vec3>>>,
}

/// A skinned (or morphed) mesh to draw: like [`crate::Instance`], plus the
/// palette (an index into [`crate::RenderScene::joint_palettes`]) that poses
/// it and its blendshape weights. Instances sharing a (mesh, material) draw
/// in one instanced call whatever their palettes. The camera culls each one
/// with its mesh's bind-pose bounding sphere scaled by `bounds_scale` (an
/// animated pose can leave the bind pose's bounds); shadow passes draw
/// every one that casts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkinnedInstance {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub transform: Mat4,
    pub palette: u32,
    /// Blendshape weights in target order (og-anim's `morph_weights`).
    pub morph_weights: [f32; MAX_MORPH_TARGETS],
    /// How much larger than the bind pose's bounds the pose may reach.
    pub bounds_scale: f32,
    /// Its own hit flash and tint ([`crate::InstanceFx`]).
    pub fx: crate::InstanceFx,
    /// Drawn into the shadow maps, as [`crate::Instance::cast_shadows`]:
    /// a skinned (mesh, material)'s non-casters are a batch of their own.
    pub cast_shadows: bool,
    /// Darkened by the shadow maps, as [`crate::Instance::receive_shadows`].
    pub receive_shadows: bool,
}

impl SkinnedInstance {
    /// An instance posed by palette `palette`, no blendshapes, bounds ×2,
    /// casting and receiving shadows.
    pub fn new(mesh: MeshHandle, material: MaterialHandle, transform: Mat4, palette: u32) -> Self {
        SkinnedInstance {
            mesh,
            material,
            transform,
            palette,
            morph_weights: [0.0; MAX_MORPH_TARGETS],
            bounds_scale: 2.0,
            fx: crate::InstanceFx::NONE,
            cast_shadows: true,
            receive_shadows: true,
        }
    }
}

/// The most joints one frame's palettes may hold together.
pub const MAX_JOINTS: usize = 256 * 256;

/// `SkinIn` in pbr.wgsl / shadow.wgsl: joints (Uint16x4) and weights.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkinVertex {
    pub joints: [u16; 4],
    pub weights: [f32; 4],
}

pub(crate) const SKIN_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<SkinVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![9 => Uint16x4, 10 => Float32x4],
};

/// `SkinnedInstanceIn`: the transform (its columns' xyz, as
/// [`crate::instances::GpuTransform::columns`]), (first joint, joint count, first
/// morph element, morph targets), the eight morph weights and the
/// instance's look ([`crate::instances::layer_bits`] of no layer, then
/// [`crate::instances::fx_bits`]).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkinnedInstanceGpu {
    pub transform: [[f32; 3]; 4],
    pub palette: [f32; 4],
    pub morph0: [f32; 4],
    pub morph1: [f32; 4],
    pub look: [f32; 4],
}

pub(crate) const SKINNED_INSTANCE_STRIDE: u64 = std::mem::size_of::<SkinnedInstanceGpu>() as u64;

pub(crate) const SKINNED_INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: SKINNED_INSTANCE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32x4, 11 => Float32x4, 12 => Float32x4, 13 => Float32x4],
    };

/// Joints per row of the palette texture (four texels each).
pub(crate) const TEXTURE_JOINTS_PER_ROW: u32 = 256;

/// How the palette reaches the vertex shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaletteMode {
    Storage,
    Texture,
}

impl PaletteMode {
    /// The WGSL that defines `joint_matrix` for this mode, in bind group `group`.
    pub fn wgsl(self, group: u32) -> String {
        let src = match self {
            PaletteMode::Storage => wgsl!("skin_storage.wgsl"),
            PaletteMode::Texture => wgsl!("skin_texture.wgsl"),
        };
        src.replace("SKIN_GROUP", &group.to_string())
    }

    pub fn layout(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let ty = match self {
            PaletteMode::Storage => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            PaletteMode::Texture => wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        };
        let entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty,
            count: None,
        };
        let entries = match self {
            PaletteMode::Storage => vec![entry(0), entry(1)],
            PaletteMode::Texture => vec![entry(0)],
        };
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("joint palette"),
            entries: &entries,
        })
    }
}

/// Texels per row of the texture form of a vec4 array (the palette's four
/// columns per joint, the morph deltas).
pub(crate) const TEXTURE_ROW: u32 = TEXTURE_JOINTS_PER_ROW * 4;

/// A growable array of vec4s the vertex shader reads: a storage buffer, or
/// an Rgba32Float texture of [`TEXTURE_ROW`] texels per row.
struct Vec4Array {
    capacity: u32,
    buffer: Option<wgpu::Buffer>,
    texture: Option<(wgpu::Texture, wgpu::TextureView)>,
}

impl Vec4Array {
    fn new() -> Vec4Array {
        Vec4Array {
            capacity: 0,
            buffer: None,
            texture: None,
        }
    }

    /// Makes room for `count` vec4s; returns whether the resource was
    /// recreated (its contents are then gone).
    fn reserve(&mut self, mode: PaletteMode, device: &wgpu::Device, count: usize) -> bool {
        let count = (count as u32).max(TEXTURE_ROW);
        if count <= self.capacity {
            return false;
        }
        let mut capacity = count.next_power_of_two();
        if mode == PaletteMode::Texture {
            // Never more rows than the device allows (what is written was
            // checked against it: `GpuPalette::texture_rows`).
            let most = device
                .limits()
                .max_texture_dimension_2d
                .saturating_mul(TEXTURE_ROW);
            capacity = capacity.min(most.max(count));
        }
        match mode {
            PaletteMode::Storage => {
                self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("vertex vec4 array"),
                    size: capacity as u64 * 16,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            PaletteMode::Texture => {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("vertex vec4 array"),
                    size: wgpu::Extent3d {
                        width: TEXTURE_ROW,
                        height: capacity.div_ceil(TEXTURE_ROW),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                self.texture = Some((texture, view));
            }
        }
        self.capacity = capacity;
        true
    }

    /// Writes `data` from row `row` (of [`TEXTURE_ROW`] vec4s) on.
    fn upload(&self, mode: PaletteMode, queue: &wgpu::Queue, row: u32, data: &[[f32; 4]]) {
        if data.is_empty() {
            return;
        }
        match mode {
            PaletteMode::Storage => {
                queue.write_buffer(
                    self.buffer.as_ref().expect("made"),
                    row as u64 * TEXTURE_ROW as u64 * 16,
                    bytemuck::cast_slice(data),
                );
            }
            PaletteMode::Texture => {
                let rows = data.len().div_ceil(TEXTURE_ROW as usize);
                let mut padded = data.to_vec();
                padded.resize(rows * TEXTURE_ROW as usize, [0.0; 4]);
                let (texture, _) = self.texture.as_ref().expect("made");
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        origin: wgpu::Origin3d { x: 0, y: row, z: 0 },
                        ..texture.as_image_copy()
                    },
                    bytemuck::cast_slice(&padded),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(TEXTURE_ROW * 16),
                        rows_per_image: Some(rows as u32),
                    },
                    wgpu::Extent3d {
                        width: TEXTURE_ROW,
                        height: rows as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn resource(&self) -> wgpu::BindingResource<'_> {
        match (&self.buffer, &self.texture) {
            (Some(b), _) => b.as_entire_binding(),
            (_, Some((_, v))) => wgpu::BindingResource::TextureView(v),
            _ => unreachable!("written before binding"),
        }
    }
}

/// Rows of [`TEXTURE_ROW`] vec4s that `texels` vec4s take.
fn rows(texels: usize) -> u32 {
    texels.div_ceil(TEXTURE_ROW as usize) as u32
}

/// The frame's joint palette and the scene's morph deltas on the GPU, and
/// the bind group that reads them: two storage buffers, or on the texture
/// path one texture, the morph deltas in its first rows and the palette
/// after them (wgpu's GL backend has 16 texture slots per pipeline, and the
/// lit skinned pipeline needs every other one).
pub(crate) struct GpuPalette {
    mode: PaletteMode,
    palette: Vec4Array,
    /// Storage only: the morph deltas.
    morphs: Vec4Array,
    /// Texture only: the rows of morph deltas the texture holds before the palette.
    morph_rows: Option<u32>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl GpuPalette {
    pub(crate) fn mode(&self) -> PaletteMode {
        self.mode
    }

    pub fn new(mode: PaletteMode) -> GpuPalette {
        GpuPalette {
            mode,
            palette: Vec4Array::new(),
            morphs: Vec4Array::new(),
            morph_rows: None,
            bind_group: None,
        }
    }

    /// Where the palette's first joint is for a scene with `morph_texels`
    /// morph deltas: what every skinned instance's first joint is offset by.
    pub fn joint_base(&self, morph_texels: usize) -> u32 {
        match self.mode {
            PaletteMode::Storage => 0,
            PaletteMode::Texture => rows(morph_texels) * TEXTURE_JOINTS_PER_ROW,
        }
    }

    /// The texture rows `joints` joint matrices and `morph_texels` morph
    /// deltas take on the texture path (`None` on the storage path), to
    /// check against the device's largest 2D texture before writing.
    pub fn texture_rows(&self, morph_texels: usize, joints: usize) -> Option<u64> {
        match self.mode {
            PaletteMode::Storage => None,
            PaletteMode::Texture => Some(
                (morph_texels as u64).div_ceil(TEXTURE_ROW as u64)
                    + (joints as u64 * 4).div_ceil(TEXTURE_ROW as u64),
            ),
        }
    }

    /// Uploads the joint matrices (four columns each) and, when
    /// `morphs_changed` (or their storage was recreated), the scene's morph
    /// deltas; rebinds when a resource was recreated.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        matrices: &[[[f32; 4]; 4]],
        morphs: &[[f32; 4]],
        morphs_changed: bool,
    ) {
        let columns: &[[f32; 4]] = bytemuck::cast_slice(matrices);
        let mode = self.mode;
        let rebind = match mode {
            PaletteMode::Storage => {
                let mut grew = self.palette.reserve(mode, device, columns.len());
                self.palette.upload(mode, queue, 0, columns);
                let morphs_grew = self.morphs.reserve(mode, device, morphs.len());
                if morphs_changed || morphs_grew {
                    self.morphs.upload(mode, queue, 0, morphs);
                }
                grew |= morphs_grew;
                grew
            }
            PaletteMode::Texture => {
                let morph_rows = rows(morphs.len());
                let size = (morph_rows * TEXTURE_ROW) as usize + columns.len();
                let grew = self.palette.reserve(mode, device, size);
                if morphs_changed || grew || self.morph_rows != Some(morph_rows) {
                    self.palette.upload(mode, queue, 0, morphs);
                    self.morph_rows = Some(morph_rows);
                }
                self.palette.upload(mode, queue, morph_rows, columns);
                grew
            }
        };
        if rebind || self.bind_group.is_none() {
            let mut entries = vec![wgpu::BindGroupEntry {
                binding: 0,
                resource: self.palette.resource(),
            }];
            if mode == PaletteMode::Storage {
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.morphs.resource(),
                });
            }
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("joint palette"),
                layout,
                entries: &entries,
            }));
        }
    }
}
