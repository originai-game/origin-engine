//! GPU skinning: skinned meshes carry four joints and weights per vertex
//! ([`SkinData`], added with [`crate::RenderScene::add_skinned_mesh`]);
//! [`SkinnedInstance`]s draw them with a [`JointPalette`] of model-space
//! joint matrices the animation system writes every frame. The palette is a
//! storage buffer; the vertex shader blends the four matrices of each
//! vertex.
//!
//! # The palette buffer
//!
//! One `array<mat4x4<f32>>` per frame (skin group, binding 0), read by every
//! skinned vertex stage (`skin_storage.wgsl`). Every other module that reads
//! or extends it follows this layout:
//!
//! 1. **The current section**, `section` slots:
//!    - each [`crate::RenderScene::joint_palettes`] entry's joints, in
//!      palette order (a skinned instance's `palette.x` is its first slot,
//!      `palette.y` its count);
//!    - then the blendshape weights of each skinned instance whose mesh has
//!      morph targets and whose weights are not all zero, in draw order:
//!      [`weight_slots`] of its targets, 16 weights a slot in column-major
//!      order (target `t` at slot `t / 16`, column `(t % 16) / 4`, row
//!      `t % 4`); its `morph0.x` is its first slot plus one (0: none);
//!    - with them, in the same draw order, the vertices of each skinned
//!      instance with [`SkinnedInstance::deformed`] (a simulated garment):
//!      [`deformed_slots`], two vertices a slot (columns 0 and 1 the even
//!      vertex's position and normal in the instance's space, 2 and 3 the
//!      odd one's), which replace the mesh's own and its morphs; its
//!      `morph0.y` is its first slot plus one (0: none).
//! 2. **The previous section** (TAA only, [`append_previous`]): the same
//!    layout again from slot `section`, holding the previous frame's joints
//!    (in the world, [`crate::RenderScene::previous_joint_palettes`]), the
//!    previous frame's weights
//!    ([`crate::RenderScene::previous_morph_weights`]) and the previous
//!    frame's deformed vertices
//!    ([`crate::RenderScene::previous_deformed_vertices`], in this frame's
//!    instance space). The skinned prepass reads `section` at
//!    `Fx::decal_info.w` (0: no previous section) and finds an instance's
//!    previous joints at `palette.x + section`, its previous weights at
//!    `morph0.x - 1 + section` and its previous vertices at
//!    `morph0.y - 1 + section`, so a morphing face and a swinging garment
//!    have motion vectors under TAA.
//!
//! **Baked clips (crowds, [`SkinnedInstance::baked`]).** Each
//! [`crate::RenderScene::baked_clips`] entry a frame's instances use (now
//! or the frame before) lives in the persistent baked buffer (binding 2):
//! its frames one after another,
//! `joints` slots each, shared by every instance that plays it. A baked
//! instance's `palette.x` is the first slot of the frame before its time,
//! `palette.y` the clip's joints, and `palette.w` is `-1 - t` (below
//! -0.5 marks it baked; no blendshapes then): the shader blends joint `j`
//! of that frame with joint `j` of the next, `joints` slots on, at `t`.
//! Under TAA it also has one slot of its own, its previous transform; `morph1` is (that slot plus one, the first slot of its previous frame,
//! that frame's `t`, 0), read in the current section, so its motion
//! vectors need no previous baked palette. Only the transform slot is
//! copied into the dynamic previous section, keeping it aligned.
//!
//! Both sections together hold at most [`MAX_JOINTS`] slots.
//!
//! The morph deltas (binding 1, `array<vec4<f32>>`) are static per mesh:
//! from the mesh's base (`palette.z`), one header per vertex (x: its first
//! entry, y: its entries; exact f32 indices), then each vertex's entries,
//! two slots each, only for the targets that move it: (position delta,
//! target) and (normal delta, 0). `palette.w` is the mesh's target count
//! (0: no morph targets).

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

/// The most blendshape (morph) targets a mesh can have (og-anim's
/// `BLENDSHAPES`): a face rig's 52 ARKit shapes and more.
pub const MAX_MORPH_TARGETS: usize = 64;

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
    /// Its own vertices, an index into
    /// [`crate::RenderScene::deformed_vertices`] (a simulated garment): they
    /// replace the mesh's positions, normals and morphs (`None`: the mesh's).
    /// Give the mesh zero skin weights, so the palette leaves them in place.
    pub deformed: Option<u32>,
    /// Posed by a baked clip on the GPU instead of `palette` (a crowd):
    /// see [`BakedPose`]. No blendshapes or deformed vertices with it.
    pub baked: Option<BakedPose>,
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
            deformed: None,
            baked: None,
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

/// The most slots one frame's palette buffer may hold: joints and
/// blendshape weights, both sections together (see the module's layout).
pub const MAX_JOINTS: usize = 256 * 256;

/// The palette slots of `targets` blendshape weights: 16 to a slot.
pub(crate) fn weight_slots(weights: &[f32], targets: usize) -> Vec<[[f32; 4]; 4]> {
    let mut slots = vec![[[0.0; 4]; 4]; targets.div_ceil(16)];
    for (t, &w) in weights.iter().take(targets).enumerate() {
        slots[t / 16][(t % 16) / 4][t % 4] = w;
    }
    slots
}

/// An animation clip baked into joint palettes for crowds
/// ([`crate::RenderScene::baked_clips`]): `frames` holds its frames one
/// after another, `joints` matrices each (model space, as a
/// [`JointPalette`]), evenly spaced in time; at least two. A clip that
/// loops ends on a copy of its first frame. Kept by the scene and shared
/// by every instance that plays it. It occupies `frames.len()` × 64 bytes
/// in the persistent baked buffer, uploaded when the active bakes change,
/// not when their playback advances. TAA takes only a dynamic previous
/// transform per instance, not a second copy of the clip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedClip {
    pub joints: u32,
    pub frames: Vec<Mat4>,
}

impl BakedClip {
    /// How many frames it holds.
    pub fn frame_count(&self) -> usize {
        if self.joints == 0 {
            0
        } else {
            self.frames.len() / self.joints as usize
        }
    }

    /// The frame before `frame` (a fractional frame index, clamped to the
    /// clip) and how far towards the next one it is, 0 to 1.
    pub fn at(&self, frame: f32) -> (usize, f32) {
        let last = self.frame_count().saturating_sub(2);
        let f = if frame.is_finite() {
            frame.max(0.0)
        } else {
            0.0
        };
        let k = (f as usize).min(last);
        (k, (f - k as f32).clamp(0.0, 1.0))
    }

    /// Its palette at `frame`, blended as the shader blends it.
    pub fn palette(&self, frame: f32) -> Vec<Mat4> {
        let (k, t) = self.at(frame);
        let j = self.joints as usize;
        (0..j)
            .map(|i| {
                let a = self
                    .frames
                    .get(k * j + i)
                    .copied()
                    .unwrap_or(Mat4::IDENTITY);
                let b = self.frames.get((k + 1) * j + i).copied().unwrap_or(a);
                a * (1.0 - t) + b * t
            })
            .collect()
    }
}

/// Where a baked clip poses a [`SkinnedInstance`] ([`SkinnedInstance::baked`]):
/// `clip` indexes [`crate::RenderScene::baked_clips`] and `frame` is a
/// fractional frame of it (frame 1.5: halfway between its second and third
/// frames). The previous frame's clip, frame and transform give its TAA
/// motion vectors (the same as now: it stood still).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BakedPose {
    pub clip: u32,
    pub frame: f32,
    pub previous_clip: u32,
    pub previous_frame: f32,
    pub previous_transform: Mat4,
}

/// A skinned instance's own vertices ([`SkinnedInstance::deformed`]): one
/// position and normal per vertex of its mesh, in the instance's space
/// (before its transform), in the mesh's vertex order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeformedVertices {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Deferred garment draw work; positions/normals remain empty until the
    /// renderer chooses compute or the coarse CPU fallback.
    pub garment: Option<crate::GarmentDeformation>,
}

impl DeformedVertices {
    pub fn vertex_count(&self) -> usize {
        self.garment
            .as_ref()
            .map_or(self.positions.len(), |g| g.topology.vertices.len())
    }

    pub fn evaluated(&self) -> std::borrow::Cow<'_, Self> {
        self.garment.as_ref().map_or_else(
            || std::borrow::Cow::Borrowed(self),
            |g| std::borrow::Cow::Owned(g.evaluate()),
        )
    }
}

/// The palette slots of a deformed instance's vertices: two to a slot
/// (position, normal; position, normal), the module's layout.
pub(crate) fn deformed_slots(d: &DeformedVertices) -> Vec<[[f32; 4]; 4]> {
    let d = d.evaluated();
    let mut slots = vec![[[0.0; 4]; 4]; d.positions.len().div_ceil(2)];
    for (v, (p, n)) in d.positions.iter().zip(&d.normals).enumerate() {
        let s = &mut slots[v / 2];
        let c = (v % 2) * 2;
        s[c] = [p.x, p.y, p.z, 1.0];
        s[c + 1] = [n.x, n.y, n.z, 0.0];
    }
    slots
}

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
/// [`crate::instances::GpuTransform::columns`]), (first joint, joint count,
/// the mesh's first morph header, its morph targets), `morph0.x` its first
/// weight slot plus one (0: no weights; the module's layout), `morph0.y`
/// its first deformed vertex slot plus one (0: none), `morph1` a baked
/// clip's previous frame (the module's layout; 0 otherwise), and the instance's look ([`crate::instances::layer_bits`] of no layer, then
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
/// deltas (binding 1) and immutable bakes (binding 2), read by the vertex shader.
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
        entries: &[entry(0), entry(1), entry(2)],
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

/// Active bakes and their stable offsets until the set or its contents change.
#[derive(Default)]
struct BakeCache {
    clips: Vec<(u32, BakedClip)>,
    offsets: std::collections::BTreeMap<u32, usize>,
    len: usize,
}

impl BakeCache {
    fn update(
        &mut self,
        clips: &[BakedClip],
        used: &std::collections::BTreeSet<u32>,
    ) -> Option<Vec<Mat4>> {
        if self.clips.len() == used.len()
            && self
                .clips
                .iter()
                .zip(used)
                .all(|((id, clip), next)| id == next && clip == &clips[*next as usize])
        {
            return None;
        }
        self.clips.clear();
        self.offsets.clear();
        let mut matrices = Vec::new();
        for &id in used {
            let clip = &clips[id as usize];
            self.offsets.insert(id, matrices.len());
            matrices.extend_from_slice(&clip.frames);
            self.clips.push((id, clip.clone()));
        }
        self.len = matrices.len();
        Some(matrices)
    }
}

/// Dynamic joints, morph deltas and persistent baked clips on the GPU.
pub(crate) struct GpuPalette {
    palette: Vec4Array,
    morphs: Vec4Array,
    baked: Vec4Array,
    bakes: BakeCache,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl GpuPalette {
    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        self.palette.buffer.as_ref().expect("palette uploaded")
    }

    pub fn new() -> GpuPalette {
        GpuPalette {
            palette: Vec4Array::new(),
            morphs: Vec4Array::new(),
            baked: Vec4Array::new(),
            bakes: BakeCache::default(),
            bind_group: None,
        }
    }

    /// Upload immutable data only when its active set changes. Recreated
    /// storage invalidates the group, which `write` binds with the dynamic data.
    pub fn prepare_bakes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clips: &[BakedClip],
        used: &std::collections::BTreeSet<u32>,
    ) {
        if let Some(matrices) = self.bakes.update(clips, used) {
            if self.baked.reserve(device, matrices.len() * 4) {
                self.bind_group = None;
            }
            self.baked.upload(queue, bytemuck::cast_slice(&matrices));
        }
        if self.baked.reserve(device, 0) {
            self.bind_group = None;
        }
    }

    pub fn baked_offset(&self, clip: u32) -> usize {
        self.bakes.offsets[&clip]
    }

    pub fn baked_len(&self) -> usize {
        self.bakes.len
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.baked.resource(),
                    },
                ],
            }));
        }
    }
}

/// The previous section of the palette buffer for TAA's motion vectors
/// (the module's layout), appended after the current section `slots`
/// (at `slots.len()` before the call, which the prepass reads through
/// `Fx::decal_info.w`): each palette's
/// [`crate::RenderScene::previous_joint_palettes`] entry, in the world,
/// or, where it has none of the same length, its joints this frame times
/// the transform its first instance is drawn at (motion with the camera
/// only); then `previous_slots`, the previous frame's weight and deformed vertex slots in
/// the current section's order. `None` without previous palettes or room
/// for a second section; else whether any joint or weight moved since the
/// previous frame.
pub(crate) fn append_previous(
    palettes: &[JointPalette],
    previous: &[JointPalette],
    instances: &[SkinnedInstance],
    previous_slots: &[[[f32; 4]; 4]],
    slots: &mut Vec<[[f32; 4]; 4]>,
) -> Option<bool> {
    if previous.is_empty() || previous.len() != palettes.len() || slots.len() * 2 > MAX_JOINTS {
        return None;
    }
    let joints: usize = palettes.iter().map(|p| p.matrices.len()).sum();
    if slots.len() != joints + previous_slots.len() {
        return None;
    }
    let weights_moved = slots[joints..] != *previous_slots;
    let mut at = vec![None; palettes.len()];
    for s in instances {
        if let Some(t) = at.get_mut(s.palette as usize) {
            t.get_or_insert(s.transform);
        }
    }
    let mut moved = false;
    for ((p, q), t) in palettes.iter().zip(previous).zip(at) {
        let t = t.unwrap_or(Mat4::IDENTITY);
        if q.matrices.len() == p.matrices.len() {
            moved |= p
                .matrices
                .iter()
                .zip(&q.matrices)
                .any(|(m, w)| !(t * *m).abs_diff_eq(*w, 1e-6));
            slots.extend(q.matrices.iter().map(Mat4::to_cols_array_2d));
        } else {
            slots.extend(p.matrices.iter().map(|m| (t * *m).to_cols_array_2d()));
        }
    }
    slots.extend_from_slice(previous_slots);
    Some(moved || weights_moved)
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use crate::scene::{MaterialHandle, MeshHandle};

    #[test]
    fn bakes_survive_playback_but_replacements_and_clip_switches_refresh_them() {
        let mut clips = vec![
            BakedClip {
                joints: 1,
                frames: vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::Y)],
            },
            BakedClip {
                joints: 1,
                frames: vec![Mat4::from_translation(Vec3::X); 3],
            },
        ];
        let used = [0, 1].into();
        let mut cache = BakeCache::default();
        let bytes = cache.update(&clips, &used).unwrap();
        assert_eq!(bytes.len(), 5);
        assert_eq!(cache.offsets[&1], 2);
        // Playback selects different offsets into the same resident bytes.
        for at in [0.0, 0.25, 1.0, 0.0] {
            let (frame, _) = clips[0].at(at);
            assert_eq!(bytes[cache.offsets[&0] + frame], clips[0].frames[frame]);
            assert!(cache.update(&clips, &used).is_none());
        }
        clips[0].frames[1] = Mat4::from_translation(Vec3::Z);
        assert_eq!(cache.update(&clips, &used).unwrap()[1], clips[0].frames[1]);
        assert!(cache.update(&clips, &used).is_none());
        assert_eq!(cache.update(&clips, &[1].into()).unwrap(), clips[1].frames);
        assert_eq!(cache.offsets[&1], 0);
        assert!(cache.update(&clips, &[1].into()).is_none());
        assert!(
            cache
                .update(&clips, &Default::default())
                .unwrap()
                .is_empty()
        );
    }

    fn palette(m: &[Mat4]) -> JointPalette {
        JointPalette::from(m.to_vec())
    }

    /// The second section mirrors the first's layout: a palette with a
    /// previous pose of its own length takes it as given (world space); one
    /// without (a joint count that changed) repeats this frame's joints in
    /// the world at its first instance's transform, which reads as no
    /// motion; a pose that only moved with its instance still counts as
    /// moved.
    #[test]
    fn previous_joints_follow_the_palettes_layout() {
        let t = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let j = Mat4::from_translation(Vec3::Y);
        let instances = [
            SkinnedInstance::new(
                MeshHandle::from_index(0),
                MaterialHandle::from_index(0),
                t,
                1,
            ),
            SkinnedInstance::new(
                MeshHandle::from_index(0),
                MaterialHandle::from_index(0),
                Mat4::IDENTITY,
                0,
            ),
        ];
        let now = [palette(&[j]), palette(&[j, j])];
        let mut joints: Vec<_> = [j, j, j].iter().map(Mat4::to_cols_array_2d).collect();
        // Palette 0 unmoved (its world joints as now), palette 1 lost a joint.
        let before = [palette(&[j]), palette(&[j])];
        assert_eq!(
            append_previous(&now, &before, &instances, &[], &mut joints),
            Some(false)
        );
        assert_eq!(joints.len(), 6);
        assert_eq!(joints[4], (t * j).to_cols_array_2d());
        // The instance under palette 0 moved a metre: moved.
        let mut joints: Vec<_> = [j, j, j].iter().map(Mat4::to_cols_array_2d).collect();
        let before = [
            palette(&[Mat4::from_translation(Vec3::X) * j]),
            palette(&[t * j, t * j]),
        ];
        assert_eq!(
            append_previous(&now, &before, &instances, &[], &mut joints),
            Some(true)
        );
        assert_eq!(
            joints[3],
            (Mat4::from_translation(Vec3::X) * j).to_cols_array_2d()
        );
        // No previous palettes, or a count that does not match: no section.
        assert_eq!(
            append_previous(&now, &before[1..], &instances, &[], &mut joints),
            None
        );
        let mut joints = Vec::new();
        assert_eq!(
            append_previous(&now, &[], &instances, &[], &mut joints),
            None
        );
        assert!(joints.is_empty());
    }

    /// Two vertices a slot, the shader's reading (`morphed`'s `deformed`):
    /// vertex `v` at slot `v / 2`, its position in column `(v % 2) * 2` and
    /// its normal in the next; an odd count leaves the last slot half empty.
    #[test]
    fn a_baked_clip_blends_its_frames_as_the_shader_does() {
        // Two joints, three frames: joint 0 moves up 1 a frame, joint 1
        // stays.
        let up = |y: f32| Mat4::from_translation(Vec3::new(0.0, y, 0.0));
        let clip = BakedClip {
            joints: 2,
            frames: vec![
                up(0.0),
                Mat4::IDENTITY,
                up(1.0),
                Mat4::IDENTITY,
                up(2.0),
                Mat4::IDENTITY,
            ],
        };
        assert_eq!(clip.frame_count(), 3);
        // The frame before and how far on: the last frame is reached
        // from the one before it, never past it.
        assert_eq!(clip.at(0.25), (0, 0.25));
        assert_eq!(clip.at(1.5), (1, 0.5));
        assert_eq!(clip.at(2.0), (1, 1.0));
        assert_eq!(clip.at(7.0), (1, 1.0));
        assert_eq!(clip.at(-1.0), (0, 0.0));
        assert_eq!(clip.at(f32::NAN), (0, 0.0));
        let p = clip.palette(1.25);
        assert!(p[0].abs_diff_eq(up(1.25), 1e-6), "{:?}", p[0]);
        assert_eq!(p[1], Mat4::IDENTITY);
    }

    #[test]
    fn deformed_vertices_pack_two_a_slot() {
        let d = DeformedVertices {
            positions: (0..3)
                .map(|v| Vec3::new(v as f32, 10.0 + v as f32, -1.0))
                .collect(),
            normals: (0..3)
                .map(|v| Vec3::new(0.0, 0.0, v as f32 + 0.5))
                .collect(),
            garment: None,
        };
        let slots = deformed_slots(&d);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0][2], [1.0, 11.0, -1.0, 1.0]);
        assert_eq!(slots[0][3], [0.0, 0.0, 1.5, 0.0]);
        assert_eq!(slots[1][0], [2.0, 12.0, -1.0, 1.0]);
        assert_eq!(slots[1][1], [0.0, 0.0, 2.5, 0.0]);
        assert_eq!(slots[1][2], [0.0; 4]);
    }

    /// The previous section carries the previous frame's weight slots after
    /// its joints, at the same offset as the current ones; a change of
    /// weight alone counts as motion; mismatched slots give no section.
    #[test]
    fn previous_weights_follow_the_joints() {
        let j = Mat4::IDENTITY;
        let inst = [SkinnedInstance::new(
            MeshHandle::from_index(0),
            MaterialHandle::from_index(0),
            Mat4::IDENTITY,
            0,
        )];
        let now = [palette(&[j])];
        let w_now = weight_slots(&[0.5; 20], 20);
        let w_before = weight_slots(&[0.25; 20], 20);
        let mut slots = vec![j.to_cols_array_2d()];
        slots.extend_from_slice(&w_now);
        assert_eq!(
            append_previous(&now, &now, &inst, &w_before, &mut slots),
            Some(true)
        );
        assert_eq!(slots.len(), 6);
        assert_eq!(slots[3..4], [j.to_cols_array_2d()]);
        assert_eq!(slots[4..], w_before[..]);
        // The same weights and joints: nothing moved.
        let mut slots = vec![j.to_cols_array_2d()];
        slots.extend_from_slice(&w_now);
        assert_eq!(
            append_previous(&now, &now, &inst, &w_now, &mut slots),
            Some(false)
        );
        // Weight slots that do not match the current section's: none.
        let mut slots = vec![j.to_cols_array_2d()];
        slots.extend_from_slice(&w_now);
        assert_eq!(append_previous(&now, &now, &inst, &[], &mut slots), None);
    }
}
