//! The debris layer: fine pieces of broken props as rigid bodies the GPU
//! simulates and draws, never read back. The simulation (Rapier, in
//! og-physics) stays the authority for every piece a game can touch; a
//! piece given here is visual only, the same on every device that draws it
//! and absent on one that does not ([`crate::RenderScene::debris`]).
//!
//! **Drawn as instances.** Each piece is one or more ordinary instances of
//! the scene (its mesh and materials, shadows, outlines, looks, motion
//! vectors) whose transform the CPU gives as the pose it broke off at. The
//! renderer culls it by the sphere it can reach ([`DebrisPiece::reach`]),
//! and after the frame's instance uploads `shaders/debris.wgsl` steps the
//! bodies and writes each instance's transform (and last frame's, for
//! motion vectors) into the instance buffers in place, before any pass
//! reads them.
//!
//! **Simulated by ticks.** A piece steps once per tick of the scene from
//! the tick it broke off ([`DebrisPiece::tick`]), with the substeps of
//! XPBD's rigid bodies (Müller et al. 2020; `third_party/gpu-debris.md`):
//! its hull's vertices against the scene's colliders
//! ([`crate::scene_colliders`]: planes, boxes, spheres, height fields),
//! with static and dynamic friction and restitution; it sleeps once
//! still. A piece first seen after the tick it broke off (a restored
//! world, a renderer made late) catches up on the GPU, so a frame shows
//! the same pile whenever it was first drawn. Pieces do not collide with
//! each other or with the simulation's convex chunks.

use crate::RenderScene;
use crate::particles::SceneCollider;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

/// The most pieces at once (the bodies buffer's size).
pub const MAX_DEBRIS: usize = 2048;
/// The most hull points a piece collides at.
pub const DEBRIS_POINTS: usize = 16;
/// Ticks after it broke off a piece is taken as still moving (the shadow
/// cache and motion vectors treat it as moving until then).
pub const DEBRIS_SETTLE_TICKS: u64 = 600;
/// Substeps a tick.
const SUBSTEPS: u32 = 4;
/// The most ticks a frame steps; a longer gap restarts every piece from
/// where it broke off.
const MAX_STEPS: u64 = 8;
/// Bytes between the two params slots (the dynamic offset's alignment).
const SLOT: u64 = 256;
const SOURCE: &str = wgsl!("debris.wgsl");
const COLLIDERS: &str = wgsl!("scene_colliders.wgsl");

/// The debris kernel's module: the scene colliders' tests, then its own.
pub(crate) fn source() -> String {
    format!("{COLLIDERS}\n{SOURCE}")
}

/// A fine piece of a broken prop, as [`crate::RenderScene::debris`] holds
/// it: where and when it broke off, how it was moving, what it collides
/// at and the instances that draw it.
#[derive(Clone, Debug, PartialEq)]
pub struct DebrisPiece {
    /// Stable while the piece lives (a new id is a new piece).
    pub id: u64,
    /// The tick it broke off at, in [`crate::RenderScene::tick`]'s count.
    pub tick: u64,
    /// Its centre of mass, rotation and scale at `tick`.
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    /// Its velocity (m/s) and angular velocity (rad/s) at `tick`.
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    /// The points it collides at, in its own frame about its centre of
    /// mass before `scale` (its hull's vertices); at most
    /// [`DEBRIS_POINTS`] are read.
    pub points: Vec<Vec3>,
    pub friction: f32,
    pub restitution: f32,
    /// How far from where it broke off it may go (m): its culling sphere,
    /// and past it the piece is no longer drawn.
    pub reach: f32,
    /// The instances that draw it: each an index into
    /// [`crate::RenderScene::instances`] and the transform from the
    /// piece's frame (centre of mass, rotation, scale) to the instance's
    /// mesh.
    pub instances: Vec<(u32, Mat4)>,
}

impl DebrisPiece {
    /// The world sphere it can reach: where the renderer culls it.
    pub(crate) fn sphere(&self) -> (Vec3, f32) {
        let r = self
            .points
            .iter()
            .map(|p| (*p * self.scale).length())
            .fold(0.0, f32::max);
        (self.position, self.reach + r)
    }
}

/// How many [`chip`] shapes there are.
pub const CHIP_VARIANTS: u32 = 4;

/// A rubble chip about 1 across, centred on its centre of mass, as the
/// debris layer throws them off a fine piece: one of [`CHIP_VARIANTS`]
/// convex lumps (two octahedra of uneven axes, a wedge, a slab),
/// flat-shaded so each facet catches the light on its own. Its vertices are
/// its hull's points ([`DebrisPiece::points`]); its uvs a planar map a
/// twelfth of a unit across, so a cut-face material's texture shows as a
/// grain's worth of its colour rather than squeezed whole onto a chip.
pub fn chip(variant: u32) -> crate::scene::MeshData {
    use crate::scene::{MeshData, Vertex};
    // A fixed jitter in -1..1 per (variant, k): the shapes are data.
    let j = |k: u32| {
        let x = og_core::hash::lowbias32(
            variant.wrapping_mul(0x9e37_79b9) ^ k.wrapping_mul(0x85eb_ca6b),
        );
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let (points, faces): (Vec<Vec3>, Vec<[usize; 3]>) = match variant % CHIP_VARIANTS {
        // Octahedra with each pole at its own distance: convex whatever
        // the distances, and lumpy where they differ.
        0 | 1 => {
            // Each pole half a unit or less out, the flatter axis at
            // least half the longest.
            let r = |k: u32| 0.5 * (0.7 + 0.3 * (0.5 + 0.5 * j(k)));
            let flat = if variant.is_multiple_of(CHIP_VARIANTS) {
                0.72
            } else {
                0.9
            };
            (
                vec![
                    v(r(0), 0.0, 0.0),
                    v(-r(1), 0.0, 0.0),
                    v(0.0, r(2) * flat, 0.0),
                    v(0.0, -r(3) * flat, 0.0),
                    v(0.0, 0.0, r(4) * 0.85),
                    v(0.0, 0.0, -r(5) * 0.85),
                ],
                vec![
                    [0, 2, 4],
                    [2, 1, 4],
                    [1, 3, 4],
                    [3, 0, 4],
                    [2, 0, 5],
                    [1, 2, 5],
                    [3, 1, 5],
                    [0, 3, 5],
                ],
            )
        }
        // A wedge: a triangle extruded, its two ends parallel.
        2 => {
            let t = [
                v(0.55, 0.0, 0.0),
                v(-0.4, 0.0, 0.5 + 0.1 * j(0)),
                v(-0.35, 0.0, -0.5),
            ];
            let h = 0.32 + 0.06 * j(1);
            let mut p: Vec<Vec3> = t.iter().map(|q| *q + Vec3::Y * h).collect();
            p.extend(t.iter().map(|q| *q - Vec3::Y * h));
            (
                p,
                vec![
                    [0, 1, 2],
                    [3, 5, 4],
                    [0, 3, 4],
                    [0, 4, 1],
                    [1, 4, 5],
                    [1, 5, 2],
                    [2, 5, 3],
                    [2, 3, 0],
                ],
            )
        }
        // A slab of uneven sides.
        _ => {
            let (x, y, z) = (0.5, 0.36 + 0.04 * j(0), 0.42);
            let p = vec![
                v(-x, -y, -z),
                v(x, -y, -z),
                v(x, y, -z),
                v(-x, y, -z),
                v(-x, -y, z),
                v(x, -y, z),
                v(x, y, z),
                v(-x, y, z),
            ];
            (
                p,
                vec![
                    [0, 2, 1],
                    [0, 3, 2],
                    [4, 5, 6],
                    [4, 6, 7],
                    [0, 1, 5],
                    [0, 5, 4],
                    [3, 7, 6],
                    [3, 6, 2],
                    [0, 4, 7],
                    [0, 7, 3],
                    [1, 2, 6],
                    [1, 6, 5],
                ],
            )
        }
    };
    let centre = points.iter().copied().sum::<Vec3>() / points.len() as f32;
    let mut vertices = Vec::with_capacity(faces.len() * 3);
    let mut indices = Vec::with_capacity(faces.len() * 3);
    let shift = Vec3::new(j(7), j(8), j(9)) * 0.4;
    for f in faces {
        let (a, mut b, mut c) = (
            points[f[0]] - centre,
            points[f[1]] - centre,
            points[f[2]] - centre,
        );
        let mut n = (b - a).cross(c - a).normalize_or(Vec3::Y);
        // Counter-clockwise seen from outside.
        if n.dot(a + b + c) < 0.0 {
            n = -n;
            std::mem::swap(&mut b, &mut c);
        }
        // The face's plane, mapped along its dominant axis.
        let axis = n.abs();
        let uv = |q: Vec3| {
            let q = (q + shift) / 12.0;
            if axis.x >= axis.y && axis.x >= axis.z {
                [q.z, q.y]
            } else if axis.y >= axis.z {
                [q.x, q.z]
            } else {
                [q.x, q.y]
            }
        };
        let base = vertices.len() as u32;
        for q in [a, b, c] {
            vertices.push(Vertex {
                position: q.to_array(),
                normal: n.to_array(),
                uv: uv(q),
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
    MeshData { vertices, indices }
}

/// Whether some piece of `scene` may still be moving.
pub(crate) fn moving(scene: &RenderScene) -> bool {
    scene
        .objects
        .debris
        .iter()
        .any(|d| scene.frame.tick < d.tick.saturating_add(DEBRIS_SETTLE_TICKS))
}

/// Each debris instance of `scene` by its index: its piece's index in
/// [`RenderScene::debris`] and the piece-to-mesh transform.
pub(crate) fn instances(scene: &RenderScene) -> HashMap<u32, (usize, Mat4)> {
    let mut out = HashMap::new();
    for (p, d) in scene.objects.debris.iter().enumerate() {
        for &(i, local) in &d.instances {
            out.insert(i, (p, local));
        }
    }
    out
}

/// A body as debris.wgsl `Body` holds it.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BodyGpu {
    pos: [f32; 4],
    rot: [f32; 4],
    vel: [f32; 4],
    ang: [f32; 4],
    prev_pos: [f32; 4],
    prev_rot: [f32; 4],
    scale: [f32; 4],
    inv_inertia: [f32; 4],
    home: [f32; 4],
    info: [u32; 4],
}

/// debris.wgsl `Params`.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ParamsGpu {
    counts: [u32; 4],
    step: [f32; 4],
    shift: [f32; 4],
    writes_at: [u32; 4],
}

/// debris.wgsl `Write`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WriteGpu {
    at: [u32; 4],
    local: [[f32; 4]; 4],
}

/// A buffer of instance records the frame draws from, the previous
/// transforms' at the same records (when the frame keeps them), and the
/// records that draw debris: each (its record, the piece's index in
/// [`RenderScene::debris`], the piece-to-mesh transform with the record's
/// dequantisation).
pub(crate) struct Target<'a> {
    pub instances: &'a wgpu::Buffer,
    pub previous: Option<&'a wgpu::Buffer>,
    pub records: Vec<(u32, usize, Mat4)>,
}

struct Gpu {
    layout: wgpu::BindGroupLayout,
    step: wgpu::ComputePipeline,
    write: wgpu::ComputePipeline,
    params: wgpu::Buffer,
    bodies: wgpu::Buffer,
    hull: wgpu::Buffer,
    lists: wgpu::Buffer,
    colliders: wgpu::Buffer,
    writes: wgpu::Buffer,
    /// Bound where a target keeps no previous transforms.
    empty: wgpu::Buffer,
}

/// The renderer's side: the pieces' slots in the bodies buffer and the
/// clock of their simulation.
#[derive(Default)]
pub(crate) struct DebrisLayer {
    gpu: Option<Gpu>,
    /// By piece id.
    slots: HashMap<u64, u32>,
    free: Vec<u32>,
    /// Slots in use or freed: the bodies the kernel steps.
    count: u32,
    /// The tick last stepped to and the origin it was drawn at.
    last: Option<(u64, Vec3)>,
}

impl DebrisLayer {
    fn gpu(&mut self, device: &wgpu::Device) -> &Gpu {
        self.gpu.get_or_insert_with(|| {
            let c = wgpu::ShaderStages::COMPUTE;
            let storage = |binding, read_only| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: c,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("debris"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: c,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<ParamsGpu>() as u64,
                            ),
                        },
                        count: None,
                    },
                    storage(1, false),
                    storage(2, true),
                    storage(3, true),
                    storage(4, true),
                    storage(5, true),
                    storage(6, false),
                    storage(7, false),
                ],
            });
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("debris"),
                source: wgpu::ShaderSource::Wgsl(source().into()),
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("debris"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let pipeline = |entry| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("debris"),
                    layout: Some(&pl),
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(device).as_ref(),
                })
            };
            let buffer = |label, size: u64, usage| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage,
                    mapped_at_creation: false,
                })
            };
            let s = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
            let n = MAX_DEBRIS as u64;
            Gpu {
                step: pipeline("step"),
                write: pipeline("write"),
                params: buffer(
                    "debris params",
                    2 * SLOT,
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                ),
                bodies: buffer(
                    "debris bodies",
                    n * std::mem::size_of::<BodyGpu>() as u64,
                    s,
                ),
                hull: buffer("debris hulls", n * DEBRIS_POINTS as u64 * 16, s),
                lists: buffer("debris colliders' lists", n * 4, s),
                colliders: buffer("debris colliders", 16 * 64, s),
                writes: buffer(
                    "debris writes",
                    std::mem::size_of::<WriteGpu>() as u64 * 64,
                    s,
                ),
                empty: buffer("debris no previous", 64, wgpu::BufferUsages::STORAGE),
                layout,
            }
        })
    }

    /// Steps the scene's pieces to its tick and writes their transforms
    /// into `targets`' records: one command buffer, to run after the
    /// frame's instance uploads and before anything reads them. `None`
    /// when the scene has no debris (its slots are let go).
    pub(crate) fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        targets: &[Target<'_>],
    ) -> Option<wgpu::CommandBuffer> {
        if scene.objects.debris.is_empty() {
            self.slots.clear();
            self.free.clear();
            self.count = 0;
            self.last = None;
            return None;
        }
        // The clock: one step a tick since the last frame; a jump back or
        // a long gap restarts every piece from where it broke off.
        let (steps, shift, restart) = match self.last {
            Some((t, o)) if scene.frame.tick >= t && scene.frame.tick - t <= MAX_STEPS => {
                ((scene.frame.tick - t) as u32, o - scene.frame.origin, false)
            }
            _ => (0, Vec3::ZERO, true),
        };
        if restart {
            self.slots.clear();
            self.free.clear();
            self.count = 0;
        }
        self.last = Some((scene.frame.tick, scene.frame.origin));
        // Slots: the pieces gone give theirs back, new ones take one.
        let live: HashMap<u64, usize> = scene
            .objects
            .debris
            .iter()
            .enumerate()
            .map(|(i, d)| (d.id, i))
            .collect();
        let gone: Vec<u64> = self
            .slots
            .keys()
            .filter(|id| !live.contains_key(id))
            .copied()
            .collect();
        for id in gone {
            if let Some(s) = self.slots.remove(&id) {
                self.free.push(s);
            }
        }
        self.free.sort_unstable_by(|a, b| b.cmp(a));
        let mut fresh: Vec<(u32, usize)> = Vec::new();
        for (i, d) in scene.objects.debris.iter().enumerate() {
            if self.slots.contains_key(&d.id) {
                continue;
            }
            let slot = match self.free.pop() {
                Some(s) => s,
                None if (self.count as usize) < MAX_DEBRIS => {
                    self.count += 1;
                    self.count - 1
                }
                None => continue,
            };
            self.slots.insert(d.id, slot);
            fresh.push((slot, i));
        }
        let gpu = self.gpu(device);
        // New bodies: their state where they broke off, catching up from
        // there to this tick.
        for &(slot, i) in &fresh {
            let d = &scene.objects.debris[i];
            let points: Vec<Vec3> = d.points.iter().take(DEBRIS_POINTS).copied().collect();
            let (lo, hi) = points.iter().fold(
                (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
                |(lo, hi), p| (lo.min(*p * d.scale), hi.max(*p * d.scale)),
            );
            let half = ((hi - lo) * 0.5).max(Vec3::splat(1e-3));
            // A unit-mass box's inertia about its centre.
            let inertia = Vec3::new(
                half.y * half.y + half.z * half.z,
                half.x * half.x + half.z * half.z,
                half.x * half.x + half.y * half.y,
            ) / 3.0;
            let catch_up = scene
                .frame
                .tick
                .saturating_sub(d.tick)
                .min(u64::from(u32::MAX)) as u32;
            let pos = d.position;
            let body = BodyGpu {
                pos: pos.extend(d.friction).to_array(),
                rot: d.rotation.normalize().to_array(),
                vel: d.velocity.extend(d.restitution).to_array(),
                ang: d.angular_velocity.extend(0.0).to_array(),
                prev_pos: pos.extend(0.0).to_array(),
                prev_rot: d.rotation.normalize().to_array(),
                scale: d.scale.extend(d.reach).to_array(),
                inv_inertia: inertia.recip().extend(0.0).to_array(),
                home: pos.extend(0.0).to_array(),
                info: [points.len() as u32, catch_up, 0, 1],
            };
            queue.write_buffer(
                &gpu.bodies,
                u64::from(slot) * std::mem::size_of::<BodyGpu>() as u64,
                bytemuck::bytes_of(&body),
            );
            let mut hull = [[0.0f32; 4]; DEBRIS_POINTS];
            for (k, p) in points.iter().enumerate() {
                hull[k] = p.extend(0.0).to_array();
            }
            queue.write_buffer(
                &gpu.hull,
                u64::from(slot) * DEBRIS_POINTS as u64 * 16,
                bytemuck::cast_slice(&hull),
            );
        }
        // The colliders near each body's reach, and each body's list.
        let mut bounds = Vec::new();
        let mut owners = Vec::new();
        for d in &scene.objects.debris {
            if let Some(&s) = self.slots.get(&d.id) {
                let (c, r) = d.sphere();
                bounds.push((c - Vec3::splat(r), c + Vec3::splat(r)));
                owners.push(s);
            }
        }
        let colliders: &[SceneCollider] = &scene.world.colliders;
        let (words, lists) = crate::scene_colliders::pack(colliders, &bounds);
        let mut by_slot = vec![u32::MAX; self.count as usize];
        for (s, l) in owners.iter().zip(lists) {
            by_slot[*s as usize] = l;
        }
        let gpu = self.gpu.as_mut().expect("made above");
        let bytes: &[u8] = bytemuck::cast_slice(&words);
        if bytes.len() as u64 > gpu.colliders.size() {
            gpu.colliders = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("debris colliders"),
                size: (bytes.len() as u64).next_power_of_two(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&gpu.colliders, 0, bytes);
        if !by_slot.is_empty() {
            queue.write_buffer(&gpu.lists, 0, bytemuck::cast_slice(&by_slot));
        }
        // The writes, each target's after the one before.
        let mut writes: Vec<WriteGpu> = Vec::new();
        let mut ranges = Vec::new();
        for t in targets {
            let first = writes.len() as u32;
            for &(record, piece, local) in &t.records {
                let Some(&slot) = self.slots.get(&scene.objects.debris[piece].id) else {
                    continue;
                };
                writes.push(WriteGpu {
                    at: [
                        record,
                        slot,
                        if t.previous.is_some() {
                            record
                        } else {
                            u32::MAX
                        },
                        0,
                    ],
                    local: local.to_cols_array_2d(),
                });
            }
            ranges.push((first, writes.len() as u32 - first));
        }
        let bytes: &[u8] = bytemuck::cast_slice(&writes);
        if bytes.len() as u64 > gpu.writes.size() {
            gpu.writes = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("debris writes"),
                size: (bytes.len() as u64).next_power_of_two(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !bytes.is_empty() {
            queue.write_buffer(&gpu.writes, 0, bytes);
        }
        let params = |first: u32, n: u32| ParamsGpu {
            counts: [self.count, steps, SUBSTEPS, n],
            step: [9.81, 1.0 / scene.frame.tick_rate.max(1e-3), 0.0, 0.0],
            shift: shift.extend(0.0).to_array(),
            writes_at: [first, 0, 0, 0],
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("debris"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("debris"),
                timestamp_writes: None,
            });
            // The renderer gives at most two targets (the sorted records'
            // buffer and the CPU path's): one params slot each.
            debug_assert!(targets.len() <= 2, "at most two debris targets");
            for (k, t) in targets.iter().enumerate().take(2) {
                let (first, n) = ranges[k];
                // Slot 0 steps (with the first target); slot 1 only writes.
                let slot = k as u64 * SLOT;
                let p = params(first, n);
                queue.write_buffer(&gpu.params, slot, bytemuck::bytes_of(&p));
                let group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("debris"),
                        layout: &gpu.layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &gpu.params,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<ParamsGpu>() as u64
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: gpu.bodies.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: gpu.hull.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: gpu.colliders.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: gpu.lists.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: gpu.writes.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: t.instances.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: t.previous.unwrap_or(&gpu.empty).as_entire_binding(),
                            },
                        ],
                    });
                pass.set_bind_group(0, &group, &[slot as u32]);
                if k == 0 {
                    pass.set_pipeline(&gpu.step);
                    pass.dispatch_workgroups(self.count.div_ceil(64), 1, 1);
                }
                if n > 0 {
                    pass.set_pipeline(&gpu.write);
                    pass.dispatch_workgroups(n.div_ceil(64), 1, 1);
                }
            }
        }
        Some(encoder.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One record's translation, read back from `buffer`.
    fn read_translation(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec3 {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut e = device.create_command_encoder(&Default::default());
        e.copy_buffer_to_buffer(buffer, 0, &staging, 0, 64);
        queue.submit([e.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let words: Vec<f32> =
            bytemuck::cast_slice(&staging.slice(..).get_mapped_range().unwrap()).to_vec();
        // Columns' x y z, four of them: the translation is words 9..12.
        Vec3::new(words[9], words[10], words[11])
    }

    #[test]
    fn a_piece_dropped_on_the_ground_falls_and_comes_to_rest_on_it() {
        let instance = crate::RenderDesc::default().backend.instance();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .expect("a GPU adapter (og-render's tests draw on the GPU)");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).expect("a device");
        let records = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let mut scene = RenderScene::default();
        scene.world.colliders.push(SceneCollider::Plane {
            normal: Vec3::Y,
            offset: 0.0,
        });
        // A 10 cm cube, its lowest corner 1 m up, thrown sideways.
        let h = 0.05;
        let corners = (0..8)
            .map(|k| {
                Vec3::new(
                    if k & 1 == 0 { -h } else { h },
                    if k & 2 == 0 { -h } else { h },
                    if k & 4 == 0 { -h } else { h },
                )
            })
            .collect();
        scene.objects.debris.push(DebrisPiece {
            id: 7,
            tick: 0,
            position: Vec3::new(0.0, 1.0 + h, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            velocity: Vec3::new(1.0, 0.0, 0.0),
            angular_velocity: Vec3::ZERO,
            points: corners,
            friction: 0.6,
            restitution: 0.1,
            reach: 12.0,
            instances: vec![(0, Mat4::IDENTITY)],
        });
        let mut layer = DebrisLayer::default();
        let mut at = Vec::new();
        for tick in 0..=180u64 {
            scene.frame.tick = tick;
            let targets = [Target {
                instances: &records,
                previous: None,
                records: vec![(0, 0, Mat4::IDENTITY)],
            }];
            let commands = layer
                .encode(&device, &queue, &scene, &targets)
                .expect("debris");
            queue.submit([commands]);
            if tick == 15 || tick == 180 {
                at.push(read_translation(&device, &queue, &records));
            }
        }
        // A quarter second: about 0.3 m fallen, and moving on sideways.
        assert!(
            (1.05 - at[0].y - 0.5 * 9.81 * 0.25 * 0.25).abs() < 0.03,
            "{:?}",
            at[0]
        );
        assert!((at[0].x - 0.25).abs() < 0.02, "{:?}", at[0]);
        // Three seconds: resting on the plane (its centre a half side up,
        // give or take the tilt of a corner), slid to a stop.
        assert!(at[1].y > 0.03 && at[1].y < 0.1, "{:?}", at[1]);
        assert!(at[1].x > 0.3 && at[1].x < 2.5, "{:?}", at[1]);
    }

    /// Every chip is a closed lump facing out, about 1 across and at least
    /// about half as thick as it is wide: rubble, not a card.
    #[test]
    fn chips_are_closed_outward_lumps_not_cards() {
        for v in 0..CHIP_VARIANTS {
            let m = chip(v);
            let p: Vec<Vec3> = m.vertices.iter().map(|x| Vec3::from(x.position)).collect();
            let key = |q: Vec3| q.to_array().map(|x| (x + 0.0).to_bits());
            let mut edges = HashMap::new();
            for t in m.indices.chunks(3) {
                let [a, b, c] = [t[0], t[1], t[2]].map(|i| p[i as usize]);
                let n = (b - a).cross(c - a);
                assert!(n.dot(a + b + c) > 0.0, "chip {v}: a face turned in");
                for (x, y) in [(a, b), (b, c), (c, a)] {
                    *edges.entry((key(x), key(y))).or_insert(0) += 1;
                }
            }
            for (&(x, y), &n) in &edges {
                assert_eq!(edges.get(&(y, x)), Some(&n), "chip {v}: open");
            }
            let (lo, hi) = p
                .iter()
                .fold((Vec3::MAX, Vec3::MIN), |(l, h), q| (l.min(*q), h.max(*q)));
            let size = hi - lo;
            assert!(
                size.max_element() > 0.6 && size.max_element() < 1.2,
                "chip {v}: {size}"
            );
            assert!(
                size.min_element() / size.max_element() > 0.45,
                "chip {v}: {size}"
            );
        }
    }

    #[test]
    fn a_piece_is_culled_by_everywhere_it_can_reach() {
        let d = DebrisPiece {
            id: 1,
            tick: 0,
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            points: vec![Vec3::new(0.1, 0.0, 0.0), Vec3::new(0.0, -0.3, 0.0)],
            friction: 0.6,
            restitution: 0.1,
            reach: 5.0,
            instances: Vec::new(),
        };
        let (c, r) = d.sphere();
        assert_eq!(c, Vec3::new(1.0, 2.0, 3.0));
        // The farthest point, scaled, beyond the reach.
        assert!((r - 5.6).abs() < 1e-5, "{r}");
    }
}
