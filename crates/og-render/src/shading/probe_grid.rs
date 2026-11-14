//! Realtime light volumes (R3, `docs/design.md` §2.6): boxes of irradiance
//! probes relit while the game runs, sampled where baked light volumes are
//! and over them.
//!
//! A probe is relit by capturing it: six 16² views of the scene from its
//! point, drawn as a realtime reflection probe's capture is (`probes.rs`:
//! the lit pipelines with the sun and its shadows, every local light, the
//! sky, emissive surfaces, lightmapped models with their lightmaps), then
//! projected onto first-order spherical harmonics by `probe_grid.wgsl` and
//! written over the probe in the light volumes' buffer (frame binding 10,
//! `lightmap.rs`), as the baked probes are. The captures sample the probes
//! too, so light bounces once more each time the grid is swept.
//!
//! The probes are relit a few a frame ([`RealtimeLightVolume::updates_per_frame`],
//! at most [`MAX_GRID_CAPTURES`] a frame over every volume), in an order
//! that spreads them over the box, so a moving light's bounce follows it
//! over a sweep. A volume is sampled from the frame after its first sweep
//! (drawn [`FIRST_SWEEP`] probes a frame); before that its box takes the
//! baked volumes' light or the environment's, as if it were not there.
//!
//! Sampling costs no binding: the probes share the baked volumes' buffer
//! and their header (bit 21 of a header's packed counts marks a realtime
//! volume), and `lightmap.rs`'s `ambient_irradiance` blends the realtime
//! volumes over the baked ones by the same edge weight.

use crate::lightmap::{PROBE_CAPACITY, PROBE_VOLUMES, VOLUME_ROWS, fit_counts, header};
use glam::Vec3;

/// The most probes relit in one frame, over every realtime volume.
pub const MAX_GRID_CAPTURES: usize = 8;
/// The probes a frame relights of a volume not yet swept once.
pub(crate) const FIRST_SWEEP: u32 = MAX_GRID_CAPTURES as u32;
/// The face size of a probe's capture.
pub(crate) const GRID_FACE: u32 = crate::PROBE_MIN_RESOLUTION;
/// The capture slots' ids (`probes.rs` keys a slot by id): one per capture
/// of a frame, above every entity id.
pub(crate) const GRID_SLOT_IDS: u64 = u64::MAX - MAX_GRID_CAPTURES as u64;
/// The sweep's stride through a volume's probes: a prime above 64, so
/// coprime with every grid's probe count (a product of counts of 2 to 64).
const STRIDE: u64 = 7919;

/// A box of probes relit at run time ([`crate::RenderScene::light_volumes`]):
/// axis-aligned in world space, probes on its faces and evenly between.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealtimeLightVolume {
    /// Who it is across frames (an entity id): its probes are kept by it.
    pub id: u64,
    pub min: Vec3,
    pub max: Vec3,
    /// How far past the box's faces its light fades out (m; 0: a hard
    /// edge).
    pub fade: f32,
    /// The most distance between neighbouring probes (at most 64 a side).
    pub spacing: f32,
    /// Probes relit a frame once swept (0: none, the first sweep kept).
    pub updates_per_frame: u32,
    /// Scales the light its probes capture.
    pub intensity: f32,
}

impl RealtimeLightVolume {
    /// The probes along each axis: at least two, no further apart than
    /// `spacing` (`og_bake::volume::VolumeDesc::counts`, which bakes a
    /// LightVolume's).
    pub fn counts(&self) -> [u32; 3] {
        let size = (self.max - self.min).max(Vec3::ZERO);
        let s = self.spacing.max(1e-3);
        [0, 1, 2].map(|k| ((size[k] / s).ceil() as u32 + 1).clamp(2, 64))
    }
}

/// One probe relit this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridCapture {
    pub position: Vec3,
    /// Its first vec4 in the light volumes' buffer.
    pub at: u32,
    pub intensity: f32,
    /// The distance between its volume's probes (the capture's box, which
    /// bounds what scattered copies it draws).
    pub cell: Vec3,
}

/// A realtime volume laid out in the buffer.
#[derive(Clone, Debug, PartialEq)]
struct GridVolume {
    volume: RealtimeLightVolume,
    /// Its grid (resampled to fit the buffer).
    counts: [u32; 3],
    /// Its first probe's vec4.
    first: usize,
    /// Probes relit in its current sweep.
    cursor: u64,
    /// Every probe relit at least once: it is sampled.
    swept: bool,
}

impl GridVolume {
    fn probes(&self) -> u64 {
        self.counts.iter().map(|&c| c as u64).product()
    }
}

/// The realtime volumes' layout and sweeps.
#[derive(Default)]
pub(crate) struct ProbeGrid {
    volumes: Vec<GridVolume>,
    /// The volume whose turn is first this frame (each frame the next one).
    turn: usize,
}

impl ProbeGrid {
    /// The frame's realtime volumes, laid out after the baked probes (which
    /// end at vec4 `end` and take `baked` header entries): the header rows
    /// of those swept, and the probes relit this frame. A volume keeps its
    /// probes and sweep while its grid and place in the buffer stay (a
    /// moved box keeps them until its next sweep relights them); one that
    /// does not fit is left out.
    pub fn plan(
        &mut self,
        volumes: &[RealtimeLightVolume],
        end: usize,
        baked: usize,
    ) -> (Vec<[[f32; 4]; VOLUME_ROWS]>, Vec<GridCapture>) {
        let mut next = Vec::new();
        let mut at = end;
        for v in volumes.iter().take(PROBE_VOLUMES.saturating_sub(baked)) {
            let room = PROBE_CAPACITY.saturating_sub(at) / 3;
            let Some(counts) = fit_counts(v.counts(), room) else {
                break;
            };
            let (cursor, swept) = self
                .volumes
                .iter()
                .find(|g| g.volume.id == v.id && g.counts == counts && g.first == at)
                .map_or((0, false), |g| (g.cursor, g.swept));
            let g = GridVolume {
                volume: *v,
                counts,
                first: at,
                cursor,
                swept,
            };
            at += 3 * g.probes() as usize;
            next.push(g);
        }
        self.volumes = next;
        let rows = self
            .volumes
            .iter()
            .filter(|g| g.swept)
            .map(|g| {
                let v = &g.volume;
                header(v.min, v.max, v.fade, g.first, g.counts, true)
            })
            .collect();
        let mut captures = Vec::new();
        let count = self.volumes.len();
        self.turn = if count == 0 {
            0
        } else {
            (self.turn + 1) % count
        };
        for k in 0..count {
            let g = &mut self.volumes[(self.turn + k) % count];
            let want = if g.swept {
                g.volume.updates_per_frame
            } else {
                FIRST_SWEEP
            };
            let n = g.probes();
            let c = g.counts;
            let cell = (g.volume.max - g.volume.min)
                / Vec3::new(c[0] as f32 - 1.0, c[1] as f32 - 1.0, c[2] as f32 - 1.0);
            for _ in 0..want {
                if captures.len() >= MAX_GRID_CAPTURES {
                    break;
                }
                let i = (g.cursor * STRIDE % n) as u32;
                let (x, y, z) = (i % c[0], i / c[0] % c[1], i / (c[0] * c[1]));
                captures.push(GridCapture {
                    position: g.volume.min + cell * Vec3::new(x as f32, y as f32, z as f32),
                    at: (g.first + 3 * i as usize) as u32,
                    intensity: g.volume.intensity.max(0.0),
                    cell,
                });
                g.cursor += 1;
                if g.cursor == n {
                    g.cursor = 0;
                    g.swept = true;
                    // A probe is relit at most once a frame.
                    break;
                }
            }
        }
        (rows, captures)
    }
}

/// What projects the frame's captures onto the probes.
pub(crate) struct GridProjector {
    layout: wgpu::BindGroupLayout,
    /// Compiled on the compile workers once the first frame is submitted
    /// (natively; deferred.rs): only a realtime light volume uses it, so
    /// no first frame waits for it.
    pipeline: crate::deferred::Deferred<wgpu::ComputePipeline>,
    /// Each capture's [`GridCapture::at`] and intensity, 256 bytes apart.
    params: wgpu::Buffer,
}

/// The module's source (`crate::shaders` validates it).
pub(crate) const PROBE_GRID_WGSL: &str = wgsl!("probe_grid.wgsl");

const PARAMS_STRIDE: u64 = 256;

impl GridProjector {
    pub fn new(device: &wgpu::Device) -> GridProjector {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("probe grid"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
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
                entry(
                    2,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("probe grid"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = crate::deferred::Deferred::anywhere({
            let device = device.clone();
            move || {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("probe grid"),
                    source: wgpu::ShaderSource::Wgsl(PROBE_GRID_WGSL.into()),
                });
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("probe grid"),
                    layout: Some(&pipeline_layout),
                    module: &module,
                    entry_point: Some("cs_project"),
                    compilation_options: Default::default(),
                    cache: None,
                })
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        pipeline.start_at(crate::deferred::Priority::Background);
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("probe grid params"),
            size: PARAMS_STRIDE * MAX_GRID_CAPTURES as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        GridProjector {
            layout,
            pipeline,
            params,
        }
    }

    /// Writes each capture's parameters (before the frame is submitted).
    pub fn write(&self, queue: &wgpu::Queue, captures: &[GridCapture]) {
        if captures.is_empty() {
            return;
        }
        let mut bytes = vec![0u8; PARAMS_STRIDE as usize * captures.len()];
        for (k, c) in captures.iter().enumerate() {
            let o = k * PARAMS_STRIDE as usize;
            bytes[o..o + 4].copy_from_slice(&c.at.to_le_bytes());
            bytes[o + 4..o + 8].copy_from_slice(&c.intensity.to_le_bytes());
        }
        queue.write_buffer(&self.params, 0, &bytes);
    }

    /// Projects capture `k` of the frame (its six faces in `capture`) onto
    /// its probe in `probes`.
    pub fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        k: usize,
        capture: &wgpu::Texture,
        probes: &wgpu::Buffer,
    ) {
        let view = capture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("probe grid capture"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let group = crate::volumetric::bind_at(
            device,
            "probe grid",
            &self.layout,
            &[
                (0, wgpu::BindingResource::TextureView(&view)),
                (1, probes.as_entire_binding()),
                (
                    2,
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.params,
                        offset: PARAMS_STRIDE * k as u64,
                        size: wgpu::BufferSize::new(16),
                    }),
                ),
            ],
        );
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("probe grid"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.pipeline.get());
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(id: u64, spacing: f32, per_frame: u32) -> RealtimeLightVolume {
        RealtimeLightVolume {
            id,
            min: Vec3::ZERO,
            max: Vec3::new(4.0, 2.0, 4.0),
            fade: 0.0,
            spacing,
            updates_per_frame: per_frame,
            intensity: 1.0,
        }
    }

    /// A first sweep relights every probe once, FIRST_SWEEP a frame, and
    /// only then is the volume in the header; after it, its own budget a
    /// frame, in the same order again.
    #[test]
    fn a_volume_is_sampled_after_its_first_sweep_and_then_relit_at_its_rate() {
        let mut grid = ProbeGrid::default();
        let v = volume(7, 2.0, 2);
        assert_eq!(v.counts(), [3, 2, 3]);
        let mut seen = std::collections::BTreeSet::new();
        let mut frames = 0;
        loop {
            let (rows, captures) = grid.plan(&[v], 100, 1);
            if !rows.is_empty() {
                break;
            }
            frames += 1;
            assert!(captures.len() <= FIRST_SWEEP as usize);
            for c in &captures {
                assert!((c.at - 100) % 3 == 0 && c.at < 100 + 18 * 3);
                assert!(seen.insert(c.at), "a probe relit twice in one sweep");
            }
        }
        assert_eq!(seen.len(), 18, "every probe relit before it is sampled");
        assert_eq!(frames, 3, "18 probes at 8 a frame");
        let (rows, captures) = grid.plan(&[v], 100, 1);
        assert_eq!(captures.len(), 2);
        let packed = rows[0][1][3] as u32;
        assert_eq!(packed >> 21, 1, "marked realtime");
        assert_eq!(
            [packed & 127, (packed >> 7) & 127, (packed >> 14) & 127],
            [3, 2, 3]
        );
        assert_eq!(rows[0][0], [0.0, 0.0, 0.0, 100.0]);
    }

    /// Probes sit on the box's corners and evenly between them.
    #[test]
    fn captures_sit_on_the_grid() {
        let mut grid = ProbeGrid::default();
        let v = volume(1, 2.0, 0);
        let mut points = Vec::new();
        for _ in 0..3 {
            points.extend(grid.plan(&[v], 0, 0).1.into_iter().map(|c| c.position));
        }
        assert_eq!(points.len(), 18);
        assert!(points.contains(&Vec3::new(4.0, 2.0, 4.0)));
        assert!(points.contains(&Vec3::new(2.0, 0.0, 2.0)));
        assert!(
            points
                .iter()
                .all(|p| p.cmpge(Vec3::ZERO).all() && p.cmple(v.max).all())
        );
        // Swept, a volume relit at none a frame keeps its probes.
        assert!(grid.plan(&[v], 0, 0).1.is_empty());
    }

    /// A volume's sweep is kept while its layout stays (moved or not), and
    /// starts again when the probes before it change size or its grid does.
    #[test]
    fn a_new_layout_starts_the_sweep_again() {
        let mut grid = ProbeGrid::default();
        let v = volume(1, 2.0, 1);
        for _ in 0..3 {
            grid.plan(&[v], 0, 0);
        }
        let moved = RealtimeLightVolume {
            min: v.min + Vec3::X,
            max: v.max + Vec3::X,
            ..v
        };
        assert_eq!(grid.plan(&[moved], 0, 0).0.len(), 1, "moved, still swept");
        assert!(
            grid.plan(&[moved], 30, 1).0.is_empty(),
            "after more baked probes"
        );
        let mut grid = ProbeGrid::default();
        for _ in 0..3 {
            grid.plan(&[v], 0, 0);
        }
        assert!(
            grid.plan(&[volume(1, 1.0, 1)], 0, 0).0.is_empty(),
            "a new grid"
        );
    }

    /// Every volume gets a share of a frame's captures when they are fewer
    /// than asked for, and no more than PROBE_VOLUMES volumes are laid out.
    #[test]
    fn volumes_share_the_frame_s_captures() {
        let mut grid = ProbeGrid::default();
        let volumes: Vec<_> = (0..3).map(|id| volume(id, 0.5, 8)).collect();
        let mut firsts = std::collections::BTreeSet::new();
        for _ in 0..3 {
            let (_, captures) = grid.plan(&volumes, 0, 0);
            assert_eq!(captures.len(), MAX_GRID_CAPTURES);
            firsts.insert(captures[0].at / (3 * 9 * 5 * 9));
        }
        assert_eq!(firsts.len(), 3, "each volume first in turn");
        let many: Vec<_> = (0..12).map(|id| volume(id, 4.0, 1)).collect();
        grid.plan(&many, 0, 6);
        assert_eq!(grid.volumes.len(), 2);
    }
}
