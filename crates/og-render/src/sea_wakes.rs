//! W3: the wakes of the hulls in a sea (`shaders/sea_wakes.wgsl`): a
//! camera-centred square of dynamic waves (eWave, linear deep water solved
//! per Fourier mode) over the FFT sea, stirred by the og.Buoyancy hulls
//! that cut it ([`WaterWakes`]: their footprints, drafts and motion), drawn
//! on top of the sea's waves with its foam. Picture only: nothing reads it
//! back but the height probe, and the tick floats the hulls on its own
//! waves.
//!
//! It steps once per tick of the wakes, never by the wall clock, as
//! [`crate::water_ripples`]: a frame steps the ticks the world took since the
//! last; a world that went back, was restored or jumped further starts from
//! calm water. The square follows the camera in whole cells, moving its
//! state with it at the next step.

use crate::water::{WaterCut, WaterWakes};
use glam::{DVec2, Vec2};

/// Points along the square's side (the transform's length).
pub(crate) const SIDE: u32 = 256;
/// Its cell (m): a 128 m square, the Kelvin wake of a hull at 5 m/s (16 m
/// waves) at 32 cells a wavelength.
pub(crate) const CELL: f32 = 0.5;
/// The hulls one tick stirs it with at most.
const MAX_CUTS: usize = 64;
/// One uniform slot (the dynamic offset's alignment).
const SLOT: u64 = 256;
/// The waves' own damping (1/s): a wake fades over some 15 s.
const DAMPING: f32 = 0.07;
/// The deepest dent a hull presses (m), and the highest crest drawn.
const MAX_DRAFT: f32 = 1.5;
const CREST: f32 = 1.2;
/// How long the wakes' foam takes to thin to half (s).
const FOAM_HALF_LIFE: f32 = 2.0;
/// The fraction of the square over which the waves fade out at its edge.
const EDGE_FADE: f32 = 0.15;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    size: [f32; 4],
    shift: [f32; 4],
    centre: [f32; 4],
    foam: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CutGpu {
    frame: [f32; 4],
    extra: [f32; 4],
}

/// The state's format: (height, vertical velocity, foam, the hulls' dent).
const STATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
/// What the sea draws: (height, its slopes along x and z, foam), filterable.
const OUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// A sea's wakes: their pipelines, state and the square's place.
pub(crate) struct SeaWakes {
    press: wgpu::ComputePipeline,
    rows: wgpu::ComputePipeline,
    columns: wgpu::ComputePipeline,
    unrows: wgpu::ComputePipeline,
    compose: wgpu::ComputePipeline,
    calm: wgpu::ComputePipeline,
    /// `groups[0]` reads state A and writes B, `groups[1]` the other way.
    groups: [wgpu::BindGroup; 2],
    _state: [wgpu::Texture; 2],
    _spectrum: wgpu::Buffer,
    _out: wgpu::Texture,
    /// What the sea samples (water binding 11).
    pub(crate) view: wgpu::TextureView,
    uniform: wgpu::Buffer,
    cuts: wgpu::Buffer,
    /// The wakes' epoch and the last tick stepped.
    at: Option<(u64, u64)>,
    /// The square's centre cell (absolute world cells: its centre is
    /// `cell * CELL`).
    cell: Option<glam::I64Vec2>,
    /// Whether both states were set calm since the last reset.
    cleared: bool,
}

/// The square's cell nearest `eye` (absolute world xz).
fn centre_cell(eye: DVec2) -> glam::I64Vec2 {
    let c = (eye / f64::from(CELL)).round();
    glam::I64Vec2::new(c.x as i64, c.y as i64)
}

/// The hulls of `now` as the shader reads them, at the frame's `at` (the
/// surface's centre, frame xz): each one's speed from where it was the tick
/// `before`.
fn cuts_gpu(before: Option<&[WaterCut]>, now: &[WaterCut], at: Vec2, dt: f32) -> Vec<CutGpu> {
    now.iter()
        .take(MAX_CUTS)
        .map(|c| {
            let speed = before
                .and_then(|b| b.iter().find(|p| p.entity == c.entity))
                .map_or(0.0, |p| (c.centre - p.centre).length() / dt.max(1e-6));
            let centre = at + c.centre;
            CutGpu {
                frame: [centre.x, centre.y, c.axis.x, c.axis.y],
                extra: [c.half.x, c.half.y, c.depth.clamp(0.0, MAX_DRAFT), speed],
            }
        })
        .collect()
}

impl SeaWakes {
    pub(crate) fn new(device: &wgpu::Device) -> SeaWakes {
        let compute = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sea wakes"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: compute,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Params>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: compute,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: compute,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: STATE_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: compute,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: compute,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: compute,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: OUT_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sea wakes"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sea wakes"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("sea wakes"),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let texture = |label, format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: SIDE,
                    height: SIDE,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let both = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;
        let state = [
            texture("sea wakes state", STATE_FORMAT, both),
            texture("sea wakes state", STATE_FORMAT, both),
        ];
        let out = texture("sea wakes", OUT_FORMAT, both);
        let out_view = out.create_view(&Default::default());
        let view = out.create_view(&Default::default());
        let states = state.each_ref().map(|t| t.create_view(&Default::default()));
        let spectrum = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sea wakes spectrum"),
            size: 16 * u64::from(SIDE * SIDE),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sea wakes"),
            size: SLOT * (crate::water::WAKE_TICKS as u64 + 1),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cuts = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sea wakes cuts"),
            size: (std::mem::size_of::<CutGpu>() * MAX_CUTS * crate::water::WAKE_TICKS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = |from: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sea wakes"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&states[from]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&states[1 - from]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: cuts.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: spectrum.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&out_view),
                    },
                ],
            })
        };
        SeaWakes {
            press: pipeline("press"),
            rows: pipeline("rows"),
            columns: pipeline("columns"),
            unrows: pipeline("unrows"),
            compose: pipeline("compose"),
            calm: pipeline("calm"),
            groups: [group(0), group(1)],
            _state: state,
            _spectrum: spectrum,
            _out: out,
            view,
            uniform,
            cuts,
            at: None,
            cell: None,
            cleared: false,
        }
    }

    /// The square's centre in the frame (xz) at the scene's `origin`
    /// (absolute world xz), for the sea's uniform (`WaterGpu::wake`).
    pub(crate) fn centre(&self, origin: DVec2) -> Vec2 {
        let c = self.cell.unwrap_or_default().as_dvec2() * f64::from(CELL) - origin;
        c.as_vec2()
    }

    /// Steps the ticks of `wakes` since the last frame into `encoder` (the
    /// sea's, submitted with its fields), the square moved to `eye`
    /// (absolute world xz) at the first; `at` is the surface's centre (frame
    /// xz), `origin` the frame's (absolute world xz) and `scale` the
    /// surface's dynamicWaves.
    pub(crate) fn update(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        wakes: &WaterWakes,
        eye: DVec2,
        at: Vec2,
        origin: DVec2,
        scale: f32,
    ) {
        let ticks = crate::water_ripples::plan(self.at, wakes);
        if ticks.reset {
            self.cleared = false;
        }
        self.at = wakes.ticks.last().map(|(t, _)| (wakes.epoch, *t));
        let target = centre_cell(eye);
        // The cells the square moves at the first step (none without one:
        // it stays where the state is until the next).
        let mut shift = match self.cell {
            Some(c) if !ticks.steps.is_empty() => target - c,
            Some(_) => glam::I64Vec2::ZERO,
            None => {
                self.cell = Some(target);
                glam::I64Vec2::ZERO
            }
        };
        if shift.abs().max_element() >= i64::from(SIDE) {
            self.cleared = false;
            shift = glam::I64Vec2::ZERO;
        }
        if !ticks.steps.is_empty() {
            self.cell = Some(target);
        }
        let dt = wakes.dt.max(1e-6);
        let centre = self.centre(origin);
        let base = Params {
            size: [SIDE as f32, CELL, dt, DAMPING],
            shift: [0.0; 4],
            centre: [
                centre.x,
                centre.y,
                scale.max(0.0),
                0.5f32.powf(dt / FOAM_HALF_LIFE),
            ],
            foam: [0.25, EDGE_FADE, 0.5, CREST],
        };
        let mut params: Vec<u8> = Vec::new();
        let mut cuts: Vec<CutGpu> = Vec::new();
        for (k, (before, now)) in ticks.steps.iter().enumerate() {
            let first = cuts.len();
            cuts.extend(cuts_gpu(*before, now, at, dt));
            let moved = if k == 0 { shift } else { glam::I64Vec2::ZERO };
            let p = Params {
                shift: [
                    moved.x as f32,
                    moved.y as f32,
                    first as f32,
                    (cuts.len() - first) as f32,
                ],
                ..base
            };
            params.extend_from_slice(bytemuck::bytes_of(&p));
            params.resize(params.len().next_multiple_of(SLOT as usize), 0);
        }
        // The compose step's slot (no cuts).
        params.extend_from_slice(bytemuck::bytes_of(&base));
        queue.write_buffer(&self.uniform, 0, &params);
        if !cuts.is_empty() {
            queue.write_buffer(&self.cuts, 0, bytemuck::cast_slice(&cuts));
        }
        let groups = SIDE.div_ceil(8);
        let compose = (ticks.steps.len() as u64 * SLOT) as u32;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("sea wakes"),
            timestamp_writes: None,
        });
        if !self.cleared {
            pass.set_pipeline(&self.calm);
            for group in &self.groups {
                pass.set_bind_group(0, group, &[compose]);
                pass.dispatch_workgroups(groups, groups, 1);
            }
            self.cleared = true;
        }
        for k in 0..ticks.steps.len() {
            let slot = (k as u64 * SLOT) as u32;
            pass.set_bind_group(0, &self.groups[0], &[slot]);
            pass.set_pipeline(&self.press);
            pass.dispatch_workgroups(groups, groups, 1);
            pass.set_bind_group(0, &self.groups[1], &[slot]);
            pass.set_pipeline(&self.rows);
            pass.dispatch_workgroups(SIDE, 1, 1);
            pass.set_pipeline(&self.columns);
            pass.dispatch_workgroups(SIDE, 1, 1);
            pass.set_pipeline(&self.unrows);
            pass.dispatch_workgroups(SIDE, 1, 1);
        }
        pass.set_bind_group(0, &self.groups[0], &[compose]);
        pass.set_pipeline(&self.compose);
        pass.dispatch_workgroups(groups, groups, 1);
    }
}

/// The wakes' compute shader.
pub(crate) const WGSL: &str = wgsl3d!("sea_wakes.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(entity: u64, centre: Vec2) -> WaterCut {
        WaterCut {
            entity,
            min: centre - Vec2::new(1.2, 3.0),
            max: centre + Vec2::new(1.2, 3.0),
            centre,
            half: Vec2::new(1.2, 3.0),
            axis: Vec2::X,
            depth: 0.4,
            speed: 0.0,
        }
    }

    /// A hull's speed is how far its centre moved since the tick before (a
    /// new one stands still), its footprint in the frame, its draft held
    /// under the deepest dent.
    #[test]
    fn a_hulls_speed_and_place_come_from_its_two_ticks() {
        let before = [cut(1, Vec2::new(10.0, 0.0))];
        let mut now = vec![cut(1, Vec2::new(10.0, -0.1)), cut(2, Vec2::ZERO)];
        now[0].depth = 4.0;
        let g = cuts_gpu(Some(&before), &now, Vec2::new(100.0, 50.0), 1.0 / 60.0);
        assert!((g[0].extra[3] - 6.0).abs() < 1e-3, "{:?}", g[0].extra);
        assert_eq!(g[0].extra[2], MAX_DRAFT);
        assert_eq!(&g[0].frame[..2], &[110.0, 49.9]);
        assert_eq!(g[1].extra[3], 0.0);
    }

    /// The square's centre is a whole cell of the absolute world, so the
    /// state moves by whole cells and a far origin places it to the cell.
    #[test]
    fn the_square_stands_on_whole_cells_of_the_world() {
        assert_eq!(
            centre_cell(DVec2::new(10.26, -0.24)),
            glam::I64Vec2::new(21, 0)
        );
        let mut w = DVec2::new(1.0e7 + 0.3, 2.0e6);
        let c = centre_cell(w);
        w.x += 0.1;
        assert_eq!(centre_cell(w), c);
    }
}
