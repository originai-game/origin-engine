//! The simulated wind field every vegetation tier reads (G2): grass blades,
//! scattered copies and their impostor cards, meshlet trees and swaying
//! models lean by one camera-centred field on the GPU instead of each
//! evaluating its own analytic wind (`shaders/wind_field.wgsl`).
//!
//! Two levels of [`N`]² cells, [`FINE_CELL`] and [`COARSE_CELL`] metres,
//! snapped to their cells around the camera, hold a velocity advanced by a
//! stable-fluids step (Stam 1999) towards og.Wind's travelling gusts,
//! stirred by curl-noise turbulence (og.Wind.turbulence) and pushed by
//! [`WindMotor`]s (Rockenbeck, *Blowing from the West*, GDC 2021), and a
//! lean driven by it through a damped spring per cell, so gusts arrive and
//! settle. Grass blades are generated afresh each frame and keep no state,
//! so the spring lives in the field's cells rather than on each blade;
//! each blade still sways on its own on top of it.
//!
//! The field steps once per tick, never per frame: a frame at the tick
//! already drawn steps nothing, and each tick's step reads that tick's
//! wind, so the same ticks draw the same field on every run. A first frame,
//! a jump back or one of more than [`crate::fields::MAX_CATCH_UP`] ticks starts the field
//! from the wind's target and runs [`WARM_STEPS`] ticks up to the tick
//! drawn. Visual only: nothing reads it back, and the tick never sees it
//! (og.Wind's analytic lean stays what gameplay may read).
//!
//! Without compute (`RenderCapabilities.compute` false) the field never
//! runs and every tier keeps og.Wind's analytic lean, as before.

use crate::fields::catch_up::catch_up;
use crate::fields::window;
use crate::gpu::copy::GpuCopy;
use glam::{DVec2, IVec2, Vec3};

/// wind_field.wgsl, after wind_common.wgsl.
pub(crate) fn source() -> String {
    format!(
        "{}{}{}",
        crate::LIBRARY_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        wgsl!("wind_field.wgsl")
    )
}

/// Cells a side of each level.
pub const N: u32 = 128;
/// The fine level's cell (m): 256 m across.
pub const FINE_CELL: f64 = 2.0;
/// The coarse level's cell (m): 2 km across.
pub const COARSE_CELL: f64 = 16.0;
/// The most motors a frame drives the field with (the nearest first).
pub const MAX_MOTORS: usize = 16;
/// Jacobi iterations of the pressure per step (even: it ends in `pressure`).
/// Pressure is warm-started and scrolled across camera shifts, so 10
/// iterations per tick converges cleanly without excess compute cost.
const JACOBI: u32 = 10;
/// Ticks run from the wind's target when the field starts (bounded so a
/// first frame or jump stays well under TDR budgets).
pub const WARM_STEPS: u64 = 24;
/// Ticks the field keeps stepping after its last driver (og.Wind or a
/// motor) went, so vegetation springs back and the air calms rather than
/// snapping still: 3 s at 60 Hz.
pub const SETTLE_TICKS: u64 = 180;
/// What a cell leans by per m/s without og.Wind (motors in still air).
const CALM_LEAN: f32 = crate::Wind::LEAN_PER_MPS;
/// One step's uniform, padded to the dynamic offset's alignment.
const SLOT: u64 = 768;

/// The most a cell of the field leans (wind_field.wgsl WIND_MAX_LEAN).
pub const MAX_LEAN: f32 = 1.5;

/// The most anything leans in og.Wind `wind` (strength above 0), the field
/// running or not, with motors in the scene or not: og.Wind's gusts, plus
/// the turbulence's curl (at most about 2.1 times its share of the speed,
/// the value noise's steepest slope on both axes) and the spring's
/// overshoot (under 20 % at its damping), capped as the field caps it;
/// motors can push a cell to the cap. What culling bounds widen by.
pub fn lean_cap(wind: Option<crate::Wind>, motors: bool) -> f32 {
    let analytic = wind.map_or(0.0, |w| {
        w.strength.max(0.0) * (1.0 + w.gust.clamp(0.0, 1.0))
    });
    let field = wind.map_or(0.0, |w| {
        w.strength.max(0.0)
            * (1.0 + w.gust.clamp(0.0, 1.0) + 2.1 * w.turbulence.clamp(0.0, 1.0))
            * 1.2
    });
    analytic
        .max(field.min(MAX_LEAN))
        .max(if motors { MAX_LEAN } else { 0.0 })
}

/// A local push on the wind field (og.WindMotor), in the scene's space,
/// and how it pushes: the air's own motor (`og_env::air`), the one the
/// tick moves ropes, cloths and bodies with. The field is a sheet over the
/// ground, so it takes a motor's horizontal part.
pub use og_env::air::{Motor as WindMotor, MotorKind as WindMotorKind};

/// `Sim` in wind_field.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SimGpu {
    corner: [[f32; 4]; 2],
    shift: [[i32; 4]; 2],
    wind: crate::wind::WindFieldGpu,
    drive: [f32; 4],
    step: [f32; 4],
    motors: [[f32; 4]; 2 * MAX_MOTORS],
}

/// The field's grid as the tiers read it (`WindField.grid`, `grid_outer`):
/// the fine level's corner from the scene's origin, its cell and 1 when
/// it runs; the coarse level's corner, cell and cells a side.
pub type WindGrid = [[f32; 4]; 2];

/// The grid of a field that does not run: every tier leans analytically.
pub const NO_GRID: WindGrid = [[0.0; 4], [0.0, 0.0, 0.0, N as f32]];

/// The field's level cells: each level's first cell, in whole cells of
/// the world (absolute, so the origin moving does not move them).
fn level_cells(camera: DVec2) -> [IVec2; 2] {
    [FINE_CELL, COARSE_CELL].map(|cell| window::first_cell(camera, cell, N))
}

/// The tiers' [`WindGrid`] for level cells `cells`, the scene's origin at
/// `origin` (world).
fn grid_of(cells: [IVec2; 2], origin: DVec2) -> WindGrid {
    let fine = window::corner_from(cells[0], FINE_CELL, origin);
    let coarse = window::corner_from(cells[1], COARSE_CELL, origin);
    [
        [fine.x, fine.y, FINE_CELL as f32, 1.0],
        [coarse.x, coarse.y, COARSE_CELL as f32, N as f32],
    ]
}

struct Gpu {
    /// Asked for when the field is made; nothing steps until they are
    /// ready (gpu/deferred.rs).
    pipelines: crate::deferred::Deferred<[wgpu::ComputePipeline; 5]>,
    group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    slots: u64,
    /// The field's state (velocity, lean, pressure and their steps), kept
    /// for a presentation checkpoint's copy.
    buffers: [wgpu::Buffer; 7],
}

/// [`WindFieldSim::save`].
pub(crate) struct WindSave {
    copy: Option<GpuCopy>,
    last: Option<(u64, [IVec2; 2])>,
    drive: Option<Drive>,
}

impl WindSave {
    pub(crate) fn bytes(&self) -> u64 {
        self.copy.as_ref().map_or(0, GpuCopy::bytes)
    }
}

/// The field: its GPU state, and what it last stepped.
pub(crate) struct WindFieldSim {
    gpu: Option<Gpu>,
    layout: Option<wgpu::BindGroupLayout>,
    _lean: wgpu::Texture,
    pub(crate) lean_view: wgpu::TextureView,
    /// The tick last stepped and the level cells it stepped at.
    last: Option<(u64, [IVec2; 2])>,
    /// This frame's steps (uniform slots) to encode.
    steps: u32,
    /// This frame's grid.
    grid: WindGrid,
    /// The last tick something drove the field, the most it could lean
    /// then and its lean per m/s: what it settles with after.
    drive: Option<Drive>,
    /// The most anything leans this frame ([`lean_cap`]; 0: the field is off).
    cap: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Drive {
    tick: u64,
    cap: f32,
    lean_per_ms: f32,
}

/// Whether the field runs at `tick`: driven now (`driven`), or settling
/// within [`SETTLE_TICKS`] after the last tick it was driven.
fn runs(driven: bool, last_driven: Option<u64>, tick: u64) -> bool {
    driven || last_driven.is_some_and(|t| tick >= t && tick - t <= SETTLE_TICKS)
}

impl WindFieldSim {
    /// The field, run on the GPU when `compute` (else only its texture, which
    /// no tier reads: their grid says it does not run).
    pub(crate) fn new(device: &wgpu::Device, compute: bool) -> WindFieldSim {
        let lean = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wind field"),
            size: wgpu::Extent3d {
                width: N,
                height: 2 * N,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: if compute {
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
            } else {
                wgpu::TextureUsages::TEXTURE_BINDING
            },
            view_formats: &[],
            texture_binding_view_dimension: None,
        });
        let lean_view = lean.create_view(&Default::default());
        let layout = compute.then(|| {
            let storage = |binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            let mut entries = vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<SimGpu>() as u64),
                },
                count: None,
            }];
            entries.extend((1..8).map(storage));
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            });
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("wind field"),
                entries: &entries,
            })
        });
        WindFieldSim {
            gpu: None,
            layout,
            _lean: lean,
            lean_view,
            last: None,
            steps: 0,
            grid: NO_GRID,
            drive: None,
            cap: 0.0,
        }
    }

    /// This frame's grid (`WindField.grid`, `grid_outer`).
    pub(crate) fn grid(&self) -> WindGrid {
        self.grid
    }

    /// The most anything leans this frame, the field driven or settling
    /// ([`lean_cap`]): what culling bounds widen by.
    pub(crate) fn lean_cap(&self) -> f32 {
        self.cap
    }

    /// The pipelines and buffers, made on the first frame that runs the
    /// field, with `slots` uniform slots at least.
    fn gpu(&mut self, device: &wgpu::Device, slots: u64) -> Option<&Gpu> {
        let layout = self.layout.as_ref()?;
        if self.gpu.as_ref().is_some_and(|g| g.slots >= slots) {
            return self.gpu.as_ref();
        }
        let slots = slots.next_power_of_two().max(16);
        let module = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("wind field"),
                source: wgpu::ShaderSource::Wgsl(source().into()),
            },
        );
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wind field"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });
        let device_ = device.clone();
        let pipelines = crate::deferred::Deferred::anywhere(move || {
            let device = &device_;
            [
                "advect",
                "divergence_pass",
                "jacobi_a",
                "jacobi_b",
                "project",
            ]
            .map(|entry| {
                crate::traffic::create_compute_pipeline(
                    device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("wind field"),
                        layout: Some(&pl),
                        module: &module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(device).as_ref(),
                    },
                )
            })
        });
        crate::deferred::Start::start(&pipelines);
        let cells = 2 * u64::from(N) * u64::from(N);
        let buffer = |label, size| {
            crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    // Copied for a presentation checkpoint (renderer/history.rs).
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_SRC
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            )
        };
        let buffers = [
            buffer("wind velocity", cells * 16),
            buffer("wind lean", cells * 16),
            buffer("wind velocity step", cells * 16),
            buffer("wind lean step", cells * 16),
            buffer("wind pressure", cells * 4),
            buffer("wind pressure step", cells * 4),
            buffer("wind divergence", cells * 4),
        ];
        let uniform = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("wind field"),
                size: SLOT * slots,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &uniform,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<SimGpu>() as u64),
            }),
        }];
        entries.extend(
            buffers
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32 + 1,
                    resource: b.as_entire_binding(),
                }),
        );
        entries.push(wgpu::BindGroupEntry {
            binding: 8,
            resource: wgpu::BindingResource::TextureView(&self.lean_view),
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wind field"),
            layout,
            entries: &entries,
        });
        self.gpu = Some(Gpu {
            pipelines,
            group,
            uniform,
            slots,
            buffers,
        });
        self.gpu.as_ref()
    }

    /// Plans this frame's steps and writes their uniforms: the ticks since
    /// the last one stepped, each with that tick's wind. Nothing to encode
    /// when the field does not run (no compute, or neither wind nor
    /// motors), and then its grid says so.
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
    ) {
        self.steps = 0;
        let wind = scene.atmosphere.wind.filter(|w| w.strength > 0.0);
        let motors_on = !scene.atmosphere.wind_motors.is_empty();
        let driven = wind.is_some() || motors_on;
        let off = |me: &mut Self| {
            me.grid = NO_GRID;
            me.last = None;
            me.drive = None;
            me.cap = lean_cap(wind, motors_on);
        };
        if self.layout.is_none() {
            return off(self);
        }
        let sref = |w: crate::Wind| w.speed.max(1.0);
        if driven {
            self.drive = Some(Drive {
                tick: scene.frame.tick,
                cap: lean_cap(wind, motors_on),
                lean_per_ms: wind.map_or(CALM_LEAN, |w| w.strength / sref(w)),
            });
        }
        if !runs(driven, self.drive.map(|d| d.tick), scene.frame.tick) {
            return off(self);
        }
        self.step(device, queue, scene, wind, motors_on);
    }

    /// [`WindFieldSim::prepare`]'s steps once the field runs: its pipelines
    /// and buffers, made the first time, and the steps' uniforms.
    // A guarded entry into the engine's `render3d` module (wasm-modules.json).
    #[inline(never)]
    fn step(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        wind: Option<crate::Wind>,
        motors_on: bool,
    ) {
        let driven = wind.is_some() || motors_on;
        let sref = |w: crate::Wind| w.speed.max(1.0);
        let drive = self.drive.expect("runs only after a drive");
        self.cap = if driven {
            lean_cap(wind, motors_on)
        } else {
            drive.cap
        };
        let origin = scene.frame.origin.as_dvec3();
        let eye = scene.frame.camera.position.as_dvec3() + origin;
        let cells = level_cells(DVec2::new(eye.x, eye.z));
        // The buffers are made once, with a slot for every warm-up tick;
        // new buffers hold nothing, so the field starts again.
        let fresh = self.gpu.is_none();
        let Some(gpu) = self.gpu(device, WARM_STEPS) else {
            self.grid = NO_GRID;
            return;
        };
        // Its state is kept from step to step: while its pipelines are not
        // ready nothing steps (and nothing leans by it), and the first
        // frame that runs catches up the ticks since.
        if gpu.pipelines.ready().is_none() {
            self.grid = NO_GRID;
            return;
        }
        let uniform = gpu.uniform.clone();
        let last = if fresh { None } else { self.last.map(|l| l.0) };
        let steps = catch_up(last, scene.frame.tick, WARM_STEPS);
        let (restart, first, count) = (steps.restart, steps.first, steps.count);
        self.grid = grid_of(cells, DVec2::new(origin.x, origin.z));
        if count == 0 {
            return;
        }
        let previous_cells = self.last.map_or(cells, |l| l.1);
        // A restart after drawn frames keeps the lean they last drew.
        let keep = restart && last.is_some();
        let motors = motors_gpu(scene, eye);
        let mut bytes = vec![0u8; (SLOT * count) as usize];
        for k in 0..count {
            let tick = first + k;
            let w = wind.unwrap_or(crate::Wind {
                strength: 0.0,
                ..crate::Wind::default()
            });
            let sref = sref(w);
            let shift = |level: usize| {
                let d = if k == 0 {
                    cells[level] - previous_cells[level]
                } else {
                    IVec2::ZERO
                };
                [
                    d.x,
                    d.y,
                    i32::from(restart && k == 0),
                    i32::from(keep && k == 0),
                ]
            };
            let corner = |level: usize, cell: f64| {
                let c = window::corner(cells[level], cell);
                [c.x as f32, c.y as f32, cell as f32, 0.0]
            };
            let sim = SimGpu {
                corner: [corner(0, FINE_CELL), corner(1, COARSE_CELL)],
                shift: [shift(0), shift(1)],
                wind: crate::wind::field_gpu(
                    wind,
                    tick,
                    tick,
                    scene.frame.tick_rate,
                    Vec3::ZERO,
                    NO_GRID,
                ),
                drive: [
                    sref,
                    drive.lean_per_ms,
                    w.turbulence.clamp(0.0, 1.0),
                    w.gust_size.max(1.0),
                ],
                step: [
                    1.0 / scene.frame.tick_rate.max(1e-3),
                    // A start with no frame drawn before it moves nothing
                    // on screen: its last step is what it last drew.
                    if restart && !keep && k + 1 == count {
                        2.0
                    } else {
                        f32::from(u8::from(k == 0))
                    },
                    motors.1 as f32,
                    N as f32,
                ],
                motors: motors.0,
            };
            let at = (SLOT * k) as usize;
            bytes[at..at + std::mem::size_of::<SimGpu>()].copy_from_slice(bytemuck::bytes_of(&sim));
        }
        crate::traffic::write_buffer(queue, &uniform, 0, &bytes);
        self.steps = count as u32;
        self.last = Some((scene.frame.tick, cells));
    }

    /// The field as the last frame left it, copied for a presentation
    /// checkpoint (`renderer/history.rs`): its buffers and lean (when
    /// made), the tick and cells it last stepped and its drive.
    pub(crate) fn save(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> WindSave {
        WindSave {
            copy: self.gpu.as_ref().map(|g| {
                let buffers: Vec<&wgpu::Buffer> = g.buffers.iter().collect();
                GpuCopy::of(device, encoder, &[&self._lean], &buffers)
            }),
            last: self.last,
            drive: self.drive,
        }
    }

    /// Brings back [`WindFieldSim::save`]'s field; true when it came back
    /// (or neither had one). Otherwise the field starts again, as after a
    /// jump back.
    pub(crate) fn restore(&mut self, encoder: &mut wgpu::CommandEncoder, saved: &WindSave) -> bool {
        let back = match (&saved.copy, &self.gpu) {
            (Some(c), Some(g)) => {
                let buffers: Vec<&wgpu::Buffer> = g.buffers.iter().collect();
                c.back(encoder, &[&self._lean], &buffers)
            }
            (None, None) => true,
            (None, Some(_)) => saved.last.is_none(),
            (Some(_), None) => false,
        };
        if back {
            self.last = saved.last;
            self.drive = saved.drive;
        } else {
            self.last = None;
            self.drive = None;
        }
        back
    }

    /// Records this frame's steps, before any pass that reads the field.
    pub(crate) fn encode(&self, encoder: &mut crate::Encoder) {
        let Some(gpu) = self.gpu.as_ref().filter(|_| self.steps > 0) else {
            return;
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("wind field"),
            timestamp_writes: None,
        });
        let groups = N.div_ceil(8);
        let [advect, divergence, jacobi_a, jacobi_b, project] = gpu.pipelines.wait();
        for k in 0..self.steps {
            pass.set_bind_group(0, &gpu.group, &[(SLOT * u64::from(k)) as u32]);
            let mut run = |p: &wgpu::ComputePipeline| {
                pass.set_pipeline(p);
                pass.dispatch_workgroups(groups, groups, 2);
            };
            run(advect);
            run(divergence);
            for _ in 0..JACOBI / 2 {
                run(jacobi_a);
                run(jacobi_b);
            }
            run(project);
        }
    }
}

/// The scene's motors nearest the eye (world `eye`), at most
/// [`MAX_MOTORS`], as wind_field.wgsl's rows (absolute x, z), and how many.
fn motors_gpu(scene: &crate::RenderScene, eye: glam::DVec3) -> ([[f32; 4]; 2 * MAX_MOTORS], usize) {
    let origin = scene.frame.origin.as_dvec3();
    let mut near: Vec<&WindMotor> = scene
        .atmosphere
        .wind_motors
        .iter()
        .filter(|m| m.radius > 0.0 && m.strength.is_finite() && m.strength != 0.0)
        .collect();
    let d = |m: &WindMotor| (m.position.as_dvec3() + origin - eye).length_squared();
    near.sort_by(|a, b| d(a).total_cmp(&d(b)));
    let mut rows = [[0.0; 4]; 2 * MAX_MOTORS];
    let n = near.len().min(MAX_MOTORS);
    for (i, m) in near.iter().take(n).enumerate() {
        let at = m.position.as_dvec3() + origin;
        let dir = glam::Vec2::new(m.direction.x, m.direction.z).normalize_or(glam::Vec2::X);
        let kind = match m.kind {
            WindMotorKind::Omni => 0.0,
            WindMotorKind::Directional => 1.0,
            WindMotorKind::Vortex => 2.0,
        };
        rows[2 * i] = [at.x as f32, at.z as f32, m.radius, m.strength];
        rows[2 * i + 1] = [dir.x, dir.y, kind, 0.0];
    }
    (rows, n)
}
