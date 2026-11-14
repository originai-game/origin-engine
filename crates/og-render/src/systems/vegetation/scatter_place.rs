//! Scatter placement on the GPU (`shaders/scatter_place.wgsl`): the visual
//! mode of [`crate::TerrainScatter::gpu`]. A chunk of such a scatter is
//! placed by a compute job instead of `Terrain::copy_ruled` on the CPU,
//! once, when it comes into range, into the chunk's instance buffer, with
//! its count and draws after its copies (nothing is read back): the frame
//! that needs it records the job, the chunk is drawn from the same frame
//! on, and scatter_cull culls it as it culls the CPU's chunks. The rules
//! are the CPU's ([`crate::Terrain::scatter_among`] stays the answer for
//! gameplay), read from the level-0 tiles the GPU holds; the copies match
//! the CPU's but for the last bits of the arithmetic.
//!
//! **Footprints.** A copy keeps clear of the copies of the scatters
//! placed before it, which kept clear of theirs: the CPU recurses. A job
//! instead evaluates the earlier scatters in order, each over the cells
//! of its grid the later ones can reach (its *region*: the chunk grown by
//! the reaches down the chain), into one bit per cell, and the chunk's
//! placement reads those bits. The layers of van Muijden's *GPU-Based
//! Procedural Placement in Horizon Zero Dawn* (GDC 2017), which resolves
//! collisions between layers of different footprints by evaluating them
//! in dependency order, with the exact distances of `copy_ruled` instead
//! of a density map. A chunk whose chain is too long, too wide or near
//! too many waters is placed on the CPU.

use crate::scatter::TerrainScatter;
use crate::scatter_cull::{COPY_BYTES, MAX_CHUNK_COPIES};
use glam::{Mat3, Mat4, Vec2};

/// Scatters one job evaluates at most: the chunk's and those before it.
const MAX_LAYERS: usize = 8;
/// Water surfaces a job reads at most.
const MAX_WATERS: usize = 8;
/// Bits a job's earlier scatters evaluate at most.
const MAX_STAGE_BITS: u64 = 1 << 22;
/// Where a placed chunk's draws start in its buffer, past its copies
/// (scatter_place.wgsl `PLACE_TAIL`): an indexed indirect draw, then a
/// plain one (a card's).
pub(crate) const TAIL: u64 = MAX_CHUNK_COPIES as u64 * COPY_BYTES;
/// A placed chunk's buffer: its copies and its draws.
pub(crate) const CHUNK_BYTES: u64 = TAIL + 48;
/// A job row's slot (the dynamic offset's alignment).
const ROW_SLOT: u64 = std::mem::size_of::<JobGpu>().next_multiple_of(256) as u64;

/// `PlaceLayer` in scatter_place.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerGpu {
    shape: [f32; 4],
    rules: [f32; 4],
    grid: [u32; 4],
    region: [u32; 4],
    bits: [u32; 4],
}

/// `PlaceJob` in scatter_place.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct JobGpu {
    model: [[f32; 4]; 4],
    normal: [[f32; 4]; 4],
    ground: [f32; 4],
    counts: [u32; 4],
    layers: [LayerGpu; MAX_LAYERS],
    water: [[[f32; 4]; 4]; MAX_WATERS],
    water_half: [[f32; 4]; MAX_WATERS],
}

/// One chunk to place: its terrain, its scatter and those before it.
pub(crate) struct PlaceInput<'a> {
    /// The terrain's space to the frame's, and its size along x and z.
    pub model: Mat4,
    pub size: Vec2,
    /// The scatters placed before it, first placed first, then its own.
    pub layers: Vec<&'a TerrainScatter>,
    /// The chunk's first cell and cells along x and z.
    pub cells: (u32, u32, u32, u32),
    /// The waters from the frame (each to its surface) and their half extents.
    pub waters: &'a [(Mat4, Vec2)],
    pub index_count: u32,
}

/// A job: its rows (one per dispatch: each earlier scatter it evaluates,
/// then the chunk), the bits its stages take, and the rectangle of the
/// terrain's space whose ground it reads.
pub(crate) struct Plan {
    rows: Vec<JobGpu>,
    words: u32,
    pub reads: (Vec2, Vec2),
}

/// The grid's cells along x and z of a scatter over a terrain of `size`.
fn cells_of(s: &TerrainScatter, size: Vec2) -> (u32, u32) {
    let c = (size / s.cell()).ceil();
    (c.x as u32, c.y as u32)
}

/// Plans `input`'s job (`None`: the CPU places the chunk).
pub(crate) fn plan(input: &PlaceInput) -> Option<Plan> {
    let n = input.layers.len();
    let reads_water = input.layers.iter().any(|s| s.avoid_water > 0.0);
    if n > MAX_LAYERS || (reads_water && input.waters.len() > MAX_WATERS) {
        return None;
    }
    let along = crate::ground::horizontal_scale(input.model).max(1e-6);
    let half = input.size * 0.5;
    let me = input.layers[n - 1];
    let cell = me.cell();
    let (x0, z0, w, h) = input.cells;
    let a = Vec2::new(x0 as f32, z0 as f32) * cell - half;
    let b = Vec2::new((x0 + w) as f32, (z0 + h) as f32) * cell - half;
    // How far past the chunk each layer's copies are read (`None`: not at
    // all), the chunk's own at 0, then down the chain: a consumer whose
    // copies reach `e` past the chunk (plus a cell of its own grid, where
    // its region's cells overhang) reads an earlier layer's out to the
    // larger footprint past that.
    let mut grow: Vec<Option<f32>> = vec![None; n];
    grow[n - 1] = Some(0.0);
    for j in (0..n - 1).rev() {
        for k in j + 1..n {
            let Some(e) = grow[k] else { continue };
            let reach = input.layers[k].footprint.max(input.layers[j].footprint) / along;
            if reach <= 0.0 {
                continue;
            }
            let overhang = if k == n - 1 {
                0.0
            } else {
                input.layers[k].cell()
            };
            let e = e + overhang + reach;
            grow[j] = Some(grow[j].map_or(e, |g: f32| g.max(e)));
        }
    }
    let n3 = Mat3::from_mat4(input.model).inverse().transpose();
    let mut base = JobGpu {
        model: input.model.to_cols_array_2d(),
        normal: Mat4::from_mat3(n3).to_cols_array_2d(),
        ground: [along, 0.0, 0.0, 0.0],
        counts: [n as u32, 0, input.index_count, 0],
        layers: [LayerGpu::default(); MAX_LAYERS],
        water: [[[0.0; 4]; 4]; MAX_WATERS],
        water_half: [[0.0; 4]; MAX_WATERS],
    };
    if reads_water {
        base.ground[1] = input.waters.len() as f32;
        for (i, (m, half)) in input.waters.iter().enumerate() {
            base.water[i] = m.to_cols_array_2d();
            let h = half.min(Vec2::splat(1e30));
            base.water_half[i] = [h.x, h.y, 0.0, 0.0];
        }
    }
    let mut words = 0u32;
    let mut bits = 0u64;
    let mut reads = (a, b);
    for (i, s) in input.layers.iter().enumerate() {
        let (cx, cz) = cells_of(s, input.size);
        let (region, evaluated) = if i == n - 1 {
            ([x0, z0, w, h], 0)
        } else if let Some(e) = grow[i] {
            let c = s.cell();
            let lo = ((a - e + half) / c).floor().max(Vec2::ZERO);
            let hi = ((b + e + half) / c)
                .floor()
                .min(Vec2::new(cx as f32 - 1.0, cz as f32 - 1.0));
            if lo.x > hi.x || lo.y > hi.y {
                ([0, 0, 0, 0], 1)
            } else {
                let r = [
                    lo.x as u32,
                    lo.y as u32,
                    (hi.x - lo.x) as u32 + 1,
                    (hi.y - lo.y) as u32 + 1,
                ];
                reads.0 = reads.0.min(lo * c - half);
                reads.1 = reads.1.max((hi + 1.0) * c - half);
                (r, 1)
            }
        } else {
            ([0, 0, 0, 0], 0)
        };
        let cells = u64::from(region[2]) * u64::from(region[3]);
        let first = words;
        if evaluated == 1 {
            bits += cells;
            if bits > MAX_STAGE_BITS {
                return None;
            }
            words += cells.div_ceil(32) as u32;
        }
        base.layers[i] = LayerGpu {
            shape: [s.cell(), s.scale[0], s.scale[1], s.align],
            rules: [s.noise_scale, s.noise_threshold, s.footprint, s.avoid_water],
            grid: [crate::scatter::seed_hash(s.seed), cx, cz, s.layer],
            region,
            bits: [first, evaluated, 0, 0],
        };
    }
    let rows = (0..n)
        .filter(|&i| i == n - 1 || base.layers[i].bits[1] == 1)
        .map(|i| {
            let mut row = base;
            row.counts[1] = i as u32;
            row
        })
        .collect();
    Some(Plan { rows, words, reads })
}

/// The source of the placement module over a terrain's group
/// (`arrayed`: the layer-array variant's).
pub(crate) fn source(arrayed: bool) -> String {
    let s = format!(
        "{}{}{}{}{}",
        crate::LIBRARY_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::ground::GRASS_COMMON_WGSL,
        crate::ground::TERRAIN_WGSL,
        wgsl3d!("scatter_place.wgsl"),
    );
    if arrayed {
        crate::ground::terrain_array_declarations(&s)
    } else {
        s
    }
}

/// A job recorded this frame: its terrain draw, its first row and rows.
struct Job {
    terrain_draw: usize,
    arrayed: bool,
    first: u32,
    rows: u32,
}

/// Places chunks on the GPU.
pub(crate) struct ScatterPlace {
    shared_layout: wgpu::BindGroupLayout,
    pub chunk_layout: wgpu::BindGroupLayout,
    /// The evaluate and place pipelines, per terrain layout (standard,
    /// layer arrays), made when first needed.
    pipelines: [Option<[wgpu::ComputePipeline; 2]>; 2],
    rows: wgpu::Buffer,
    row_cap: u64,
    bits: wgpu::Buffer,
    bit_words: u64,
    /// This frame's jobs and their rows and words.
    jobs: Vec<Job>,
    frame_rows: Vec<u8>,
    words: u32,
}

impl ScatterPlace {
    pub(crate) fn new(device: &wgpu::Device) -> ScatterPlace {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let shared_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter place"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<JobGpu>() as u64
                        ),
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let chunk_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter place chunk"),
            entries: &[entry(
                0,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )],
        });
        let make = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        ScatterPlace {
            rows: make(
                "scatter place rows",
                ROW_SLOT * 16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            row_cap: 16,
            bits: make("scatter place bits", 4 * 1024, Self::bits_usage()),
            bit_words: 1024,
            shared_layout,
            chunk_layout,
            pipelines: [None, None],
            jobs: Vec::new(),
            frame_rows: Vec::new(),
            words: 0,
        }
    }

    fn bits_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
    }

    /// A placed chunk's buffer: its copies (vertex and storage) and its
    /// draws (indirect).
    pub(crate) fn chunk_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scatter chunk (placed)"),
            size: CHUNK_BYTES,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// The chunk's buffer as a job writes it.
    pub(crate) fn chunk_group(
        &self,
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter place chunk"),
            layout: &self.chunk_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// Starts a frame's jobs.
    pub(crate) fn begin(&mut self) {
        self.jobs.clear();
        self.frame_rows.clear();
        self.words = 0;
    }

    /// Records `plan` for the chunk drawn over terrain draw `terrain_draw`
    /// (whose group is the layer-array variant's with `arrayed`), its
    /// pipelines made over `terrain_layout` when first needed. Jobs are
    /// encoded in the order recorded.
    pub(crate) fn push(
        &mut self,
        device: &wgpu::Device,
        terrain_layout: &wgpu::BindGroupLayout,
        (terrain_draw, arrayed): (usize, bool),
        mut plan: Plan,
    ) {
        let a = usize::from(arrayed);
        if self.pipelines[a].is_none() {
            self.pipelines[a] = Some(self.make(device, terrain_layout, arrayed));
        }
        let first = (self.frame_rows.len() as u64 / ROW_SLOT) as u32;
        for row in &mut plan.rows {
            for l in &mut row.layers {
                l.bits[0] += self.words;
            }
            let at = self.frame_rows.len();
            self.frame_rows.resize(at + ROW_SLOT as usize, 0);
            self.frame_rows[at..at + std::mem::size_of::<JobGpu>()]
                .copy_from_slice(bytemuck::bytes_of(row));
        }
        self.words += plan.words;
        self.jobs.push(Job {
            terrain_draw,
            arrayed,
            first,
            rows: plan.rows.len() as u32,
        });
    }

    fn make(
        &self,
        device: &wgpu::Device,
        terrain_layout: &wgpu::BindGroupLayout,
        arrayed: bool,
    ) -> [wgpu::ComputePipeline; 2] {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter place"),
            source: wgpu::ShaderSource::Wgsl(source(arrayed).into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scatter place"),
            bind_group_layouts: &[
                Some(&self.shared_layout),
                Some(terrain_layout),
                Some(&self.chunk_layout),
            ],
            immediate_size: 0,
        });
        ["evaluate", "place"].map(|entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("scatter place"),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        })
    }

    /// Writes this frame's rows (after every [`Self::push`]).
    pub(crate) fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.jobs.is_empty() {
            return;
        }
        let rows = self.frame_rows.len() as u64 / ROW_SLOT;
        if self.row_cap < rows {
            self.row_cap = rows.next_power_of_two();
            self.rows = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("scatter place rows"),
                size: ROW_SLOT * self.row_cap,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if self.bit_words < u64::from(self.words) {
            self.bit_words = u64::from(self.words).next_power_of_two();
            self.bits = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("scatter place bits"),
                size: 4 * self.bit_words,
                usage: Self::bits_usage(),
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.rows, 0, &self.frame_rows);
    }

    /// Whether any job waits to be encoded.
    pub(crate) fn pending(&self) -> bool {
        !self.jobs.is_empty()
    }

    /// Places this frame's chunks (`chunks`: each job's chunk group, in the
    /// order recorded), before their cull and every pass that draws them.
    pub(crate) fn encode(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        terrain_draws: &[impl crate::ground::GroundDraw],
        chunks: &[&wgpu::BindGroup],
    ) {
        if self.jobs.is_empty() {
            return;
        }
        debug_assert_eq!(chunks.len(), self.jobs.len());
        if self.words > 0 {
            encoder.clear_buffer(&self.bits, 0, Some(4 * u64::from(self.words)));
        }
        let shared = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter place"),
            layout: &self.shared_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.rows,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<JobGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.bits.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scatter place"),
            timestamp_writes: None,
        });
        for (job, chunk) in self.jobs.iter().zip(chunks) {
            let Some([evaluate, place]) = &self.pipelines[usize::from(job.arrayed)] else {
                continue;
            };
            let Some(td) = terrain_draws.get(job.terrain_draw) else {
                continue;
            };
            pass.set_bind_group(1, td.group(), &[0]);
            pass.set_bind_group(2, *chunk, &[]);
            for r in 0..job.rows {
                let row = job.first + r;
                pass.set_bind_group(0, &shared, &[(u64::from(row) * ROW_SLOT) as u32]);
                if r + 1 < job.rows {
                    // The stage's cells, read back from its row's region.
                    let cells = self.stage_cells(row);
                    pass.set_pipeline(evaluate);
                    pass.dispatch_workgroups(cells.div_ceil(64).max(1), 1, 1);
                } else {
                    pass.set_pipeline(place);
                    pass.dispatch_workgroups(1, 1, 1);
                }
            }
        }
        drop(pass);
        self.jobs.clear();
    }

    /// The cells row `row`'s stage evaluates.
    fn stage_cells(&self, row: u32) -> u32 {
        let at = (u64::from(row) * ROW_SLOT) as usize;
        let job: &JobGpu =
            bytemuck::from_bytes(&self.frame_rows[at..at + std::mem::size_of::<JobGpu>()]);
        let r = job.layers[job.counts[1] as usize].region;
        r[2] * r[3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The job's uniform has the layout its shader reads, and the module
    /// parses over both terrain layouts.
    #[test]
    fn the_job_reaches_the_shader() {
        for arrayed in [false, true] {
            let module = naga::front::wgsl::parse_str(&source(arrayed)).expect("parses");
            let mut layouter = naga::proc::Layouter::default();
            layouter.update(module.to_ctx()).expect("lays out");
            let (h, _) = module
                .types
                .iter()
                .find(|(_, t)| t.name.as_deref() == Some("PlaceJob"))
                .expect("in the shader");
            assert_eq!(layouter[h].size as usize, std::mem::size_of::<JobGpu>());
        }
        assert_eq!(TAIL, 4096 * 16, "PLACE_TAIL vectors of copies");
    }

    /// The chain's regions: a chunk of flowers after trees with a 3 m
    /// footprint reads the trees' cells out to 3 m past it, and none of an
    /// earlier scatter without a footprint on either side.
    #[test]
    fn a_footprint_grows_the_earlier_scatters_region() {
        let trees = TerrainScatter {
            density: 0.04,
            footprint: 3.0,
            priority: 2,
            seed: 3,
            ..TerrainScatter::default()
        };
        let stones = TerrainScatter {
            density: 0.5,
            priority: 1,
            seed: 4,
            ..TerrainScatter::default()
        };
        let flowers = TerrainScatter {
            density: 4.0,
            seed: 5,
            ..TerrainScatter::default()
        };
        let size = Vec2::splat(256.0);
        let plan = plan(&PlaceInput {
            model: Mat4::IDENTITY,
            size,
            layers: vec![&trees, &stones, &flowers],
            cells: (64, 64, 32, 32),
            waters: &[],
            index_count: 36,
        })
        .expect("on the GPU");
        // The trees and the flowers: stones are read by nothing.
        assert_eq!(plan.rows.len(), 2);
        let row = &plan.rows[1];
        assert_eq!(row.layers[1].bits[1], 0, "the stones are not evaluated");
        let r = row.layers[0].region;
        let cell = trees.cell();
        // The chunk: cells 64..96 of a 0.5 m grid, -96 m to -80 m.
        let lo = ((-96.0 - 3.0 + 128.0) / cell).floor() as u32;
        let hi = ((-80.0 + 3.0 + 128.0) / cell).floor() as u32;
        assert_eq!([r[0], r[1]], [lo, lo]);
        assert_eq!([r[2], r[3]], [hi - lo + 1, hi - lo + 1]);
        assert_eq!(row.layers[2].region, [64, 64, 32, 32]);
        assert!(plan.reads.0.x <= -99.0 && plan.reads.1.x >= -77.0);
    }
}
