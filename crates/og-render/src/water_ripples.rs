//! The ripples over an og.WaterSurface's heights (`shaders/water_ripples.wgsl`):
//! the CPU's height field (og-physics, world state) holds the waves the
//! tick needs at its own coarse grid; this layer runs the same wave
//! equation on the GPU on a grid four times finer, stirred by the bodies
//! that cut the surface ([`WaterWakes`]: their wakes as they move, their
//! splashes as they fall), and the water draws the CPU's heights plus it.
//! Picture only: nothing reads it back, and the tick never sees it.
//!
//! It steps once per tick of the wakes, never by the wall clock: a frame
//! steps the ticks the world took since the last (at most
//! [`WaterWakes`]' window); a world that went back, was restored or jumped
//! further starts it from calm water, the same on every run.

use crate::water::{WaterHeights, WaterWakes};

/// The ripples' grid: this many times the CPU field's points a side, at
/// most [`MAX_SIDE`].
const DETAIL: u32 = 4;
const MAX_SIDE: u32 = 1024;
/// Substeps a tick at most (the waves' speed is held under the finer
/// grid's limit past them).
const MAX_SUBSTEPS: u32 = 16;
/// The bodies one tick stirs it with at most.
const MAX_CUTS: usize = 64;
/// One uniform slot (the dynamic offset's alignment).
const SLOT: u64 = 256;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    size: [f32; 4],
    wave: [f32; 4],
    force: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CutGpu {
    prev: [f32; 4],
    cur: [f32; 4],
    extra: [f32; 4],
}

/// The compute pipelines, made with the first surface that ripples.
pub(crate) struct RipplePipelines {
    layout: wgpu::BindGroupLayout,
    advance: wgpu::ComputePipeline,
    compose: wgpu::ComputePipeline,
    calm: wgpu::ComputePipeline,
}

impl RipplePipelines {
    pub(crate) fn new(device: &wgpu::Device) -> RipplePipelines {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let storage = |binding, format| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water ripples"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Params>() as u64
                        ),
                    },
                    count: None,
                },
                texture(1),
                storage(2, STATE_FORMAT),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture(4),
                storage(5, crate::water::HEIGHT_FORMAT),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("water ripples"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water ripples"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("water ripples"),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        RipplePipelines {
            advance: pipeline("advance"),
            compose: pipeline("compose"),
            calm: pipeline("calm"),
            layout,
        }
    }
}

/// The ripples' state: (height, vertical velocity) a point.
const STATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;

/// One surface's ripples: its state (two textures, stepped from one into
/// the other), the heights it draws and the tick it stands at.
pub(crate) struct Ripples {
    side: u32,
    coarse: u32,
    size: f32,
    /// The state textures' views and, per direction, a bind group reading
    /// one and writing the other (`groups[k]` reads `k`).
    _state: [wgpu::Texture; 2],
    groups: [wgpu::BindGroup; 2],
    /// Which state is the newest.
    current: usize,
    _heights: wgpu::Texture,
    /// What the water draws: the CPU's heights plus the ripples.
    pub(crate) view: wgpu::TextureView,
    uniform: wgpu::Buffer,
    cuts: wgpu::Buffer,
    /// The wakes' epoch and the last tick stepped.
    at: Option<(u64, u64)>,
    /// Whether both states were set calm since the last reset.
    cleared: bool,
}

impl Ripples {
    /// The grid side for a CPU field of `coarse` points a side.
    pub(crate) fn side(coarse: u32) -> u32 {
        (coarse * DETAIL).min(MAX_SIDE)
    }

    /// Whether these ripples still fit `field` (its resolution and size).
    pub(crate) fn fits(&self, field: &WaterHeights) -> bool {
        self.coarse == field.resolution && self.size == field.size
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        pipes: &RipplePipelines,
        field: &WaterHeights,
        coarse: &wgpu::TextureView,
    ) -> Ripples {
        let side = Ripples::side(field.resolution);
        let texture = |label, format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            })
        };
        let state = [
            texture("water ripples state", STATE_FORMAT),
            texture("water ripples state", STATE_FORMAT),
        ];
        let heights = texture("water ripples heights", crate::water::HEIGHT_FORMAT);
        // Written as a 2D storage texture, read by the water pass as a 1-layer array (water binding 1).
        let storage_view = heights.create_view(&Default::default());
        let view = heights.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let views = state.each_ref().map(|t| t.create_view(&Default::default()));
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water ripples"),
            size: SLOT * (crate::water::WAKE_TICKS as u64 + 1),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cuts = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water ripples cuts"),
            size: (std::mem::size_of::<CutGpu>() * MAX_CUTS * crate::water::WAKE_TICKS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = |from: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("water ripples"),
                layout: &pipes.layout,
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
                        resource: wgpu::BindingResource::TextureView(&views[from]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&views[1 - from]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: cuts.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(coarse),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&storage_view),
                    },
                ],
            })
        };
        Ripples {
            side,
            coarse: field.resolution,
            size: field.size,
            groups: [group(0), group(1)],
            _state: state,
            current: 0,
            _heights: heights,
            view,
            uniform,
            cuts,
            at: None,
            cleared: false,
        }
    }

    /// Steps the ticks of `wakes` since the last frame and draws the
    /// heights (after the CPU's this frame were written to the coarse
    /// texture): its own submission, before the frame's.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipes: &RipplePipelines,
        wakes: &WaterWakes,
    ) {
        let ticks = plan(self.at, wakes);
        if ticks.reset {
            self.cleared = false;
        }
        self.at = wakes.ticks.last().map(|(t, _)| (wakes.epoch, *t));
        let side = self.side as f32;
        let cell = self.size / (side - 1.0).max(1.0);
        let dt = wakes.dt.max(1e-6);
        // The finer grid needs more substeps for the same waves: c dt' under
        // half a cell, at most MAX_SUBSTEPS (the speed held under that).
        let substeps =
            ((wakes.wave_speed.max(0.0) * dt / (0.5 * cell)).ceil() as u32).clamp(1, MAX_SUBSTEPS);
        let dts = dt / substeps as f32;
        let c = wakes.wave_speed.max(0.0).min(0.5 * cell / dts);
        // The ripples die twice as fast as the CPU's waves: they are the
        // short-lived detail over them.
        let keep = wakes.damping.clamp(0.0, 1.0).powf(2.0 * dts);
        let mut params: Vec<u8> = Vec::new();
        let mut cuts: Vec<CutGpu> = Vec::new();
        let base = Params {
            size: [side, self.coarse as f32, self.size, cell],
            wave: [c * c * dts / (cell * cell), keep, dts, substeps as f32],
            force: [0.0, 0.0, 1.0 / dt, wakes.splash],
        };
        for (before, now) in &ticks.steps {
            let first = cuts.len();
            for cur in now.iter().take(MAX_CUTS) {
                // The same body the tick before; one that only now cuts the
                // surface was above it, at no depth.
                let prev = before.and_then(|b| b.iter().find(|p| p.entity == cur.entity));
                let boxed = |w: &crate::water::WaterCut| [w.min.x, w.min.y, w.max.x, w.max.y];
                cuts.push(CutGpu {
                    prev: prev.map_or(boxed(cur), boxed),
                    cur: boxed(cur),
                    extra: [prev.map_or(0.0, |p| p.depth), cur.depth, cur.speed, 0.0],
                });
            }
            let p = Params {
                force: [
                    first as f32,
                    (cuts.len() - first) as f32,
                    1.0 / dt,
                    wakes.splash,
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
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("water ripples"),
        });
        let groups = self.side.div_ceil(8);
        let compose = (ticks.steps.len() as u64 * SLOT) as u32;
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("water ripples"),
                timestamp_writes: None,
            });
            if !self.cleared {
                // Calm water: both states zero.
                pass.set_pipeline(&pipes.calm);
                for group in &self.groups {
                    pass.set_bind_group(0, group, &[compose]);
                    pass.dispatch_workgroups(groups, groups, 1);
                }
                self.cleared = true;
            }
            pass.set_pipeline(&pipes.advance);
            for k in 0..ticks.steps.len() {
                for _ in 0..substeps {
                    pass.set_bind_group(0, &self.groups[self.current], &[(k as u64 * SLOT) as u32]);
                    pass.dispatch_workgroups(groups, groups, 1);
                    self.current = 1 - self.current;
                }
            }
            pass.set_pipeline(&pipes.compose);
            pass.set_bind_group(0, &self.groups[self.current], &[compose]);
            pass.dispatch_workgroups(groups, groups, 1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

/// What a frame steps: each new tick's cuts with the tick before's.
struct Plan<'a> {
    reset: bool,
    steps: Vec<(
        Option<&'a [crate::water::WaterCut]>,
        &'a [crate::water::WaterCut],
    )>,
}

/// The ticks of `wakes` after `at` (the epoch and tick last stepped): all
/// of them when they follow it, else none and a reset (a new epoch, a world
/// that went back, or more ticks than the wakes keep).
fn plan(at: Option<(u64, u64)>, wakes: &WaterWakes) -> Plan<'_> {
    let calm = Plan {
        reset: true,
        steps: Vec::new(),
    };
    let Some((epoch, last)) = at else {
        return calm;
    };
    let Some(&(newest, _)) = wakes.ticks.last() else {
        return calm;
    };
    if epoch != wakes.epoch || newest < last {
        return calm;
    }
    if newest == last {
        return Plan {
            reset: false,
            steps: Vec::new(),
        };
    }
    // The tick last stepped must still be in the window, so every tick
    // after it is.
    let Some(from) = wakes.ticks.iter().position(|(t, _)| *t == last) else {
        return calm;
    };
    let steps = (from + 1..wakes.ticks.len())
        .map(|i| {
            (
                Some(wakes.ticks[i - 1].1.as_slice()),
                wakes.ticks[i].1.as_slice(),
            )
        })
        .collect();
    Plan {
        reset: false,
        steps,
    }
}

/// The ripples' compute shader.
pub(crate) const WGSL: &str = wgsl3d!("water_ripples.wgsl");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::water::WaterCut;
    use glam::Vec2;

    fn wakes(epoch: u64, ticks: &[u64]) -> WaterWakes {
        WaterWakes {
            epoch,
            wave_speed: 2.0,
            damping: 0.5,
            splash: 1.0,
            dt: 1.0 / 60.0,
            ticks: ticks
                .iter()
                .map(|&t| {
                    (
                        t,
                        vec![WaterCut {
                            entity: 1,
                            min: Vec2::splat(t as f32),
                            max: Vec2::splat(t as f32 + 1.0),
                            centre: Vec2::splat(t as f32 + 0.5),
                            half: Vec2::splat(0.5),
                            axis: Vec2::X,
                            depth: 0.1,
                            speed: 0.0,
                        }],
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn steps_only_the_ticks_after_the_last_and_starts_calm_otherwise() {
        let w = wakes(3, &[10, 11, 12, 13]);
        // First frame: calm, nothing stepped.
        let p = plan(None, &w);
        assert!(p.reset && p.steps.is_empty());
        // Two ticks on: 12 (after 11) and 13 (after 12).
        let p = plan(Some((3, 11)), &w);
        assert!(!p.reset);
        let firsts: Vec<f32> = p.steps.iter().map(|(_, now)| now[0].min.x).collect();
        assert_eq!(firsts, vec![12.0, 13.0]);
        assert_eq!(p.steps[0].0.unwrap()[0].min.x, 11.0);
        // The same tick again (a second frame between ticks): nothing.
        let p = plan(Some((3, 13)), &w);
        assert!(!p.reset && p.steps.is_empty());
        // A restored world (new epoch), a world that went back, or a jump
        // past the window: calm again.
        assert!(plan(Some((2, 12)), &w).reset);
        assert!(plan(Some((3, 20)), &w).reset);
        assert!(plan(Some((3, 5)), &w).reset);
    }
}
