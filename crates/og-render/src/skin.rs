//! GPU skinning: skinned meshes carry four joints and weights per vertex
//! ([`SkinData`], added with [`crate::RenderScene::add_skinned_mesh`]);
//! [`SkinnedInstance`]s draw them with a [`JointPalette`] of model-space
//! joint matrices the animation system writes every frame. The palette is a
//! storage buffer; the vertex shader blends the four matrices of each
//! vertex.

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

/// The bind-pose box (min, max) of the vertices one joint moves (any
/// weight above 0), in the mesh's space: a skinned mesh's posed bounds are
/// these boxes through their joints' matrices ([`posed_centre`]).
pub(crate) type JointBox = (u16, Vec3, Vec3);

/// Each joint's [`JointBox`] over `positions` and `skin` (vertices with no
/// weight stay in the bind pose: they are joint `u16::MAX`'s, unmoved).
pub(crate) fn joint_boxes(positions: &[[f32; 3]], skin: &SkinData) -> Vec<JointBox> {
    let mut boxes: Vec<JointBox> = Vec::new();
    let mut grow = |j: u16, p: Vec3| match boxes.iter_mut().find(|b| b.0 == j) {
        Some(b) => {
            b.1 = b.1.min(p);
            b.2 = b.2.max(p);
        }
        None => boxes.push((j, p, p)),
    };
    for ((p, joints), weights) in positions.iter().zip(&skin.joints).zip(&skin.weights) {
        let p = Vec3::from_array(*p);
        let mut any = false;
        for k in 0..4 {
            if weights[k] > 0.0 {
                grow(joints[k], p);
                any = true;
            }
        }
        if !any {
            grow(u16::MAX, p);
        }
    }
    boxes
}

/// The centre of a skinned instance's posed bounds in the world: the union
/// of its joints' boxes moved by `palette` (a missing joint stays put),
/// through `transform`. `None` without boxes (a morphed mesh with no skin).
pub(crate) fn posed_centre(boxes: &[JointBox], palette: &[Mat4], transform: &Mat4) -> Option<Vec3> {
    let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for &(j, a, b) in boxes {
        let m = palette.get(j as usize).copied().unwrap_or(Mat4::IDENTITY);
        for k in 0..8 {
            let c = Vec3::new(
                if k & 1 == 0 { a.x } else { b.x },
                if k & 2 == 0 { a.y } else { b.y },
                if k & 4 == 0 { a.z } else { b.z },
            );
            let w = transform.transform_point3(m.transform_point3(c));
            lo = lo.min(w);
            hi = hi.max(w);
        }
    }
    lo.is_finite().then(|| (lo + hi) * 0.5)
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

/// The WGSL that defines `joint_matrix` and `morph_delta` (the palette and
/// the morph deltas as read-only storage buffers), in bind group `group`.
pub(crate) fn palette_wgsl(group: u32) -> String {
    wgsl!("skin_storage.wgsl").replace("SKIN_GROUP", &group.to_string())
}

/// The palette group's layout: the joint palette (binding 0) and the morph
/// deltas (binding 1), read by the vertex shader.
pub(crate) fn palette_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("joint palette"),
        entries: &[entry(0), entry(1)],
    })
}

/// The fewest vec4s a [`Vec4Array`] holds.
const MIN_VEC4S: u32 = 1024;

/// A growable storage buffer of vec4s the vertex shader reads.
struct Vec4Array {
    capacity: u32,
    buffer: Option<wgpu::Buffer>,
}

impl Vec4Array {
    fn new() -> Vec4Array {
        Vec4Array {
            capacity: 0,
            buffer: None,
        }
    }

    /// Makes room for `count` vec4s; returns whether the buffer was
    /// recreated (its contents are then gone).
    fn reserve(&mut self, device: &wgpu::Device, count: usize) -> bool {
        let count = (count as u32).max(MIN_VEC4S);
        if count <= self.capacity {
            return false;
        }
        let capacity = count.next_power_of_two();
        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex vec4 array"),
            size: capacity as u64 * 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.capacity = capacity;
        true
    }

    /// Writes `data` from the start.
    fn upload(&self, queue: &wgpu::Queue, data: &[[f32; 4]]) {
        if data.is_empty() {
            return;
        }
        queue.write_buffer(
            self.buffer.as_ref().expect("made"),
            0,
            bytemuck::cast_slice(data),
        );
    }

    fn resource(&self) -> wgpu::BindingResource<'_> {
        self.buffer
            .as_ref()
            .expect("written before binding")
            .as_entire_binding()
    }
}

/// The frame's joint palette and the scene's morph deltas on the GPU, in
/// two storage buffers, and the bind group that reads them.
pub(crate) struct GpuPalette {
    palette: Vec4Array,
    morphs: Vec4Array,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl GpuPalette {
    pub fn new() -> GpuPalette {
        GpuPalette {
            palette: Vec4Array::new(),
            morphs: Vec4Array::new(),
            bind_group: None,
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
        let mut rebind = self.palette.reserve(device, columns.len());
        self.palette.upload(queue, columns);
        let morphs_grew = self.morphs.reserve(device, morphs.len());
        if morphs_changed || morphs_grew {
            self.morphs.upload(queue, morphs);
        }
        rebind |= morphs_grew;
        if rebind || self.bind_group.is_none() {
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("joint palette"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.palette.resource(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.morphs.resource(),
                    },
                ],
            }));
        }
    }
}
