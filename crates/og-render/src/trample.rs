//! Grass trample (G3): where [`GrassPusher`]s (characters, horses, carts)
//! press the vegetation, a camera-centred map of [`N`]² cells of [`CELL`]
//! metres records the push away from them, the lowest they reached, and the
//! seconds since each cell was last pressed (`shaders/grass_trample.wgsl`).
//! og.Grass blades bend away and flatten under it and rise again over the
//! field's `recovery`; scattered copies part the same way. After Wohllaib,
//! *Procedural Grass in Ghost of Tsushima* (GDC 2021).
//!
//! Stepped once per tick on the GPU, like the wind field
//! ([`crate::wind_field::plan`]): a frame at a tick already drawn steps
//! nothing. A renderer starting on a later tick knows nothing pressed
//! before it (the map is visual, not world state: the tick never reads it,
//! and nothing reads it back). Without compute it never runs and nothing
//! is pressed.

use crate::renderer::history::GpuCopy;
use glam::{DVec2, IVec2, Vec3};

/// Cells a side, and a cell (m): 64 m across.
pub const N: u32 = 256;
pub const CELL: f64 = 0.25;
/// The most pushers a tick presses with (the nearest the camera).
pub const MAX_PUSHERS: usize = 32;

/// Something that presses the grass (og.GrassPusher): a vertical capsule
/// standing at `position` (its bottom), in the scene's space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GrassPusher {
    pub position: Vec3,
    /// Its reach on the ground (m).
    pub radius: f32,
    /// How tall it is (m): grass under a pusher floating higher than the
    /// blades is not touched.
    pub height: f32,
    /// How hard it presses, 0..1 (1 flattens).
    pub strength: f32,
}

pub(crate) fn source() -> String {
    wgsl3d!("grass_trample.wgsl").to_string()
}

/// `Trample` in grass_trample.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TrampleGpu {
    corner: [f32; 4],
    shift: [f32; 4],
    count: [f32; 4],
    pushers: [[f32; 4]; 2 * MAX_PUSHERS],
}

const SLOT: u64 = 1280;
/// The most ticks one frame steps (a longer gap starts the map again).
const MAX_STEPS: u64 = crate::wind_field::MAX_CATCH_UP;

/// The map's first cell, in whole cells of the absolute world.
fn grid_cell(camera: DVec2) -> IVec2 {
    let c = (camera / CELL).floor();
    IVec2::new(c.x as i32, c.y as i32) - IVec2::splat(N as i32 / 2)
}

struct Gpu {
    press: wgpu::ComputePipeline,
    store: wgpu::ComputePipeline,
    group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    /// The map's state and its step, kept for a presentation checkpoint's copy.
    buffers: [wgpu::Buffer; 2],
}

/// [`Trample::save`].
pub(crate) struct TrampleSave {
    copy: Option<GpuCopy>,
    last: Option<(u64, IVec2)>,
}

impl TrampleSave {
    pub(crate) fn bytes(&self) -> u64 {
        self.copy.as_ref().map_or(0, GpuCopy::bytes)
    }
}

/// The map: its GPU state, and the tick and grid it last stepped at.
pub(crate) struct Trample {
    layout: Option<wgpu::BindGroupLayout>,
    gpu: Option<Gpu>,
    _texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    last: Option<(u64, IVec2)>,
    steps: u32,
    /// This frame's map for the fields: its min corner from the scene's
    /// origin (x, z), cell, cells a side (0: nothing pressed).
    grid: [f32; 4],
}

impl Trample {
    /// The map as the last frame left it, copied for a presentation
    /// checkpoint (`renderer/history.rs`).
    pub(crate) fn save(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> TrampleSave {
        TrampleSave {
            copy: self.gpu.as_ref().map(|g| {
                GpuCopy::of(
                    device,
                    encoder,
                    &[&self._texture],
                    &[&g.buffers[0], &g.buffers[1]],
                )
            }),
            last: self.last,
        }
    }

    /// Brings back [`Trample::save`]'s map; true when it came back (or
    /// neither had one), else the map starts again.
    pub(crate) fn restore(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        saved: &TrampleSave,
    ) -> bool {
        let back = match (&saved.copy, &self.gpu) {
            (Some(c), Some(g)) => {
                c.back(encoder, &[&self._texture], &[&g.buffers[0], &g.buffers[1]])
            }
            (None, None) => true,
            (None, Some(_)) => saved.last.is_none(),
            (Some(_), None) => false,
        };
        self.last = if back { saved.last } else { None };
        back
    }

    pub(crate) fn new(device: &wgpu::Device, compute: bool) -> Trample {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("grass trample"),
            size: wgpu::Extent3d {
                width: N,
                height: N,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: if compute {
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
            } else {
                wgpu::TextureUsages::TEXTURE_BINDING
            },
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let layout = compute.then(|| {
            let c = wgpu::ShaderStages::COMPUTE;
            let storage = |binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: c,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("grass trample"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: c,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<TrampleGpu>() as u64,
                            ),
                        },
                        count: None,
                    },
                    storage(1),
                    storage(2),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: c,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            })
        });
        Trample {
            layout,
            gpu: None,
            _texture: texture,
            view,
            last: None,
            steps: 0,
            grid: [0.0; 4],
        }
    }

    /// This frame's map for the fields (`GrassGen.trample`).
    pub(crate) fn grid(&self) -> [f32; 4] {
        self.grid
    }

    fn gpu(&mut self, device: &wgpu::Device) -> Option<&Gpu> {
        let layout = self.layout.as_ref()?;
        if self.gpu.is_some() {
            return self.gpu.as_ref();
        }
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grass trample"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grass trample"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });
        let pipeline = |entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("grass trample"),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let cells = u64::from(N) * u64::from(N) * 16;
        let buffer = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: cells,
                // Copied for a presentation checkpoint (renderer/history.rs).
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let (state, scratch) = (buffer("grass trample"), buffer("grass trample step"));
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grass trample"),
            size: SLOT * MAX_STEPS,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grass trample"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<TrampleGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: state.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: scratch.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
            ],
        });
        self.gpu = Some(Gpu {
            press: pipeline("press"),
            store: pipeline("store"),
            group,
            uniform,
            buffers: [state, scratch],
        });
        self.gpu.as_ref()
    }

    /// Plans this frame's steps (one per tick since the last stepped) and
    /// writes their uniforms. Runs once a scene has pushers and grass or
    /// scatter to press, and keeps running while that holds (pressed grass
    /// recovers after its pushers leave).
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
    ) {
        self.steps = 0;
        let wanted = !scene.grass_pushers.is_empty() || self.last.is_some();
        let vegetation = !scene.grass.is_empty() || !scene.scatters.is_empty();
        if self.layout.is_none() || !crate::WORLD_3D || !wanted || !vegetation {
            self.grid = [0.0; 4];
            self.last = None;
            return;
        }
        let fresh = self.gpu.is_none();
        let Some(gpu) = self.gpu(device) else {
            self.grid = [0.0; 4];
            return;
        };
        let uniform = gpu.uniform.clone();
        let origin = scene.origin.as_dvec3();
        let eye = scene.camera.position.as_dvec3() + origin;
        let cell = grid_cell(DVec2::new(eye.x, eye.z));
        let corner = cell.as_dvec2() * CELL;
        let rel = corner - DVec2::new(origin.x, origin.z);
        self.grid = [rel.x as f32, rel.y as f32, CELL as f32, N as f32];
        // One step per tick; a first frame, a jump back or a long gap
        // starts from nothing pressed at the tick drawn.
        let last = if fresh { None } else { self.last };
        let (restart, first, count) = match last {
            Some((t, _)) if scene.tick >= t && scene.tick - t <= MAX_STEPS => {
                (false, t + 1, scene.tick - t)
            }
            _ => (true, scene.tick, 1),
        };
        if count == 0 {
            return;
        }
        let previous = last.map_or(cell, |l| l.1);
        let pushers = pushers_gpu(scene, eye);
        let dt = 1.0 / scene.tick_rate.max(1e-3);
        let mut bytes = vec![0u8; (SLOT * count) as usize];
        for k in 0..count {
            let d = if k == 0 { cell - previous } else { IVec2::ZERO };
            let t = TrampleGpu {
                corner: [corner.x as f32, corner.y as f32, CELL as f32, N as f32],
                shift: [
                    d.x as f32,
                    d.y as f32,
                    f32::from(u8::from(restart && k == 0)),
                    dt,
                ],
                count: [pushers.1 as f32, origin.y as f32, 0.0, 0.0],
                pushers: pushers.0,
            };
            let at = (SLOT * k) as usize;
            bytes[at..at + std::mem::size_of::<TrampleGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&t));
        }
        queue.write_buffer(&uniform, 0, &bytes);
        self.steps = count as u32;
        self.last = Some((first + count - 1, cell));
    }

    /// Records this frame's steps, before the grass generates.
    pub(crate) fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        let Some(gpu) = self.gpu.as_ref().filter(|_| self.steps > 0) else {
            return;
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("grass trample"),
            timestamp_writes: None,
        });
        let groups = N.div_ceil(8);
        for k in 0..self.steps {
            pass.set_bind_group(0, &gpu.group, &[(SLOT * u64::from(k)) as u32]);
            for p in [&gpu.press, &gpu.store] {
                pass.set_pipeline(p);
                pass.dispatch_workgroups(groups, groups, 1);
            }
        }
    }
}

/// The scene's pushers nearest the eye (world `eye`), at most
/// [`MAX_PUSHERS`], as grass_trample.wgsl's rows (absolute), and how many.
fn pushers_gpu(
    scene: &crate::RenderScene,
    eye: glam::DVec3,
) -> ([[f32; 4]; 2 * MAX_PUSHERS], usize) {
    let origin = scene.origin.as_dvec3();
    let mut near: Vec<&GrassPusher> = scene
        .grass_pushers
        .iter()
        .filter(|p| p.radius > 0.0 && p.strength > 0.0 && p.position.is_finite())
        .collect();
    let d = |p: &GrassPusher| (p.position.as_dvec3() + origin - eye).length_squared();
    near.sort_by(|a, b| d(a).total_cmp(&d(b)));
    let mut rows = [[0.0; 4]; 2 * MAX_PUSHERS];
    let n = near.len().min(MAX_PUSHERS);
    for (i, p) in near.iter().take(n).enumerate() {
        let at = p.position.as_dvec3() + origin;
        rows[2 * i] = [
            at.x as f32,
            at.z as f32,
            p.radius,
            p.strength.clamp(0.0, 1.0),
        ];
        rows[2 * i + 1] = [at.y as f32, at.y as f32 + p.height.max(0.0), 0.0, 0.0];
    }
    (rows, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The step's uniform has the layout grass_trample.wgsl reads.
    #[test]
    fn the_uniform_reaches_the_shader() {
        use std::mem::{offset_of, size_of};
        let module = naga::front::wgsl::parse_str(&source()).expect("parses");
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("lays out");
        let (handle, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some("Trample"))
            .expect("Trample");
        let naga::TypeInner::Struct { members, .. } = &ty.inner else {
            panic!("Trample is a struct")
        };
        let want = [
            offset_of!(TrampleGpu, corner),
            offset_of!(TrampleGpu, shift),
            offset_of!(TrampleGpu, count),
            offset_of!(TrampleGpu, pushers),
        ];
        for (m, at) in members.iter().zip(want) {
            assert_eq!(m.offset as usize, at, "{:?}", m.name);
        }
        assert_eq!(layouter[handle].size as usize, size_of::<TrampleGpu>());
        assert!(size_of::<TrampleGpu>() as u64 <= SLOT);
        assert_eq!(SLOT % 256, 0);
    }

    /// The map is fixed to the world's cells around the camera.
    #[test]
    fn the_map_is_fixed_to_the_world() {
        let a = grid_cell(DVec2::new(10.1, 3.0));
        let b = grid_cell(DVec2::new(10.2, 3.1));
        assert_eq!(a, b);
        assert_eq!(a, IVec2::new(40, 12) - IVec2::splat(128));
    }
}
