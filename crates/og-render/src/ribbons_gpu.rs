//! Trails, ribbons and beams on the GPU (`shaders/trails.wgsl`,
//! `ribbon.wgsl` `vs_strip`): every strip is a run of [`StripPoint`]s in
//! one buffer, which the vertex shader turns into segments facing the
//! bound camera (a view's camera draws the same points facing its own eye),
//! so the CPU builds no vertices and nothing per camera.
//!
//! A ribbon's or beam's points are written by the CPU as they are given. A
//! trail's history lives on the GPU: a ring of the positions its head
//! passed, one per elapsed tick for its style's lifetime, which a compute
//! pass steps once a frame from the head and the tick and turns into the
//! trail's points. The CPU keeps only where each ring is and when it must
//! start again (a new or restyled trail, a tick that goes back, another
//! scene). Picture only: nothing reads it back.
//!
//! Used where the device has compute and storage buffers in the vertex
//! stage; elsewhere [`crate::ribbons::Trails`] builds the strips on the CPU.
//!
//! The approach is the GPU ribbon of Niagara and of Unity's VFX Graph
//! strips (a per-strip point history in a buffer, expanded into camera
//! facing quads in the vertex shader); written here from that design, no
//! code taken.

use crate::deferred::{Deferred, Sampled};
use crate::particles::{MAX_CURVE_KEYS, ParticleBlend};
use crate::ribbons::{
    Beam, MAX_TRAIL_POINTS, Ribbon, RibbonFrame, RibbonRun, StripPoint, Trail, TrailStyle,
    fixed_strips, life_ticks, strip_points, trail_layer,
};
use crate::scene::TextureHandle;
use glam::Vec3;
use std::collections::HashMap;

const POINT_BYTES: u64 = std::mem::size_of::<StripPoint>() as u64;
/// One element of the rings (`vec4<u32>`).
const RING_BYTES: u64 = 16;

/// `struct Job` in trails.wgsl: one trail this frame.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct JobGpu {
    head: [f32; 4],
    shift: [f32; 4],
    base: u32,
    cap: u32,
    tick: u32,
    flags: u32,
    style: u32,
    out_first: u32,
    out_count: u32,
    pad: u32,
}

/// `struct Style` in trails.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct StyleGpu {
    life_ticks: u32,
    min_distance: f32,
    width_keys: u32,
    color_keys: u32,
    tile: f32,
    textured: u32,
    pad: [u32; 2],
    width: [[f32; 4]; MAX_CURVE_KEYS],
    color_t: [[f32; 4]; MAX_CURVE_KEYS / 4],
    color: [[f32; 4]; MAX_CURVE_KEYS],
}

impl StyleGpu {
    fn of(style: &TrailStyle, tick_rate: f32) -> StyleGpu {
        let mut s = StyleGpu {
            life_ticks: life_ticks(style, tick_rate).min(u64::from(u32::MAX)) as u32,
            min_distance: style.min_distance,
            width_keys: style.width.keys.len().min(MAX_CURVE_KEYS) as u32,
            color_keys: style.color.keys.len().min(MAX_CURVE_KEYS) as u32,
            tile: match style.uv {
                crate::ribbons::RibbonUv::Stretch => 0.0,
                crate::ribbons::RibbonUv::Tile { length } => length.max(1e-6),
            },
            textured: u32::from(style.texture.is_some()),
            pad: [0; 2],
            width: [[0.0; 4]; MAX_CURVE_KEYS],
            color_t: [[0.0; 4]; MAX_CURVE_KEYS / 4],
            color: [[0.0; 4]; MAX_CURVE_KEYS],
        };
        for (i, &(t, w)) in style.width.keys.iter().take(MAX_CURVE_KEYS).enumerate() {
            s.width[i] = [t, w, 0.0, 0.0];
        }
        for (i, &(t, c)) in style.color.keys.iter().take(MAX_CURVE_KEYS).enumerate() {
            s.color_t[i / 4][i % 4] = t;
            s.color[i] = c;
        }
        s
    }
}

/// Ranges of the ring buffer, first fit; a freed range joins its
/// neighbours, and the end shrinks back when the last range is freed.
#[derive(Default)]
struct Ranges {
    free: Vec<(u32, u32)>,
    end: u32,
}

impl Ranges {
    fn alloc(&mut self, len: u32) -> u32 {
        if let Some(i) = self.free.iter().position(|f| f.1 >= len) {
            let (base, l) = self.free[i];
            if l == len {
                self.free.remove(i);
            } else {
                self.free[i] = (base + len, l - len);
            }
            return base;
        }
        let base = self.end;
        self.end += len;
        base
    }

    fn release(&mut self, base: u32, len: u32) {
        let mut i = self.free.partition_point(|f| f.0 < base);
        self.free.insert(i, (base, len));
        if i + 1 < self.free.len() && base + self.free[i].1 == self.free[i + 1].0 {
            self.free[i].1 += self.free[i + 1].1;
            self.free.remove(i + 1);
        }
        if i > 0 && self.free[i - 1].0 + self.free[i - 1].1 == self.free[i].0 {
            self.free[i - 1].1 += self.free[i].1;
            self.free.remove(i);
            i -= 1;
        }
        if i + 1 == self.free.len() && self.free[i].0 + self.free[i].1 == self.end {
            self.end = self.free[i].0;
            self.free.pop();
        }
    }
}

/// A trail's ring: `cap` points after a header at `base`.
struct Slot {
    base: u32,
    cap: u32,
    style: usize,
    last_tick: u64,
    seen: u64,
}

pub(crate) struct GpuTrails {
    slots: HashMap<u64, Slot>,
    ranges: Ranges,
    ring: wgpu::Buffer,
    jobs: wgpu::Buffer,
    styles: wgpu::Buffer,
    count: wgpu::Buffer,
    points: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    /// Made on first use (deferred.rs).
    pub(crate) pipeline: Deferred<wgpu::ComputePipeline>,
    draw_layout: wgpu::BindGroupLayout,
    /// The strip points for `vs_strip` (group 2).
    pub(crate) draw_group: wgpu::BindGroup,
    /// The strip pipelines: additive, alpha.
    pub(crate) pipelines: [Sampled<wgpu::RenderPipeline>; 2],
    scene_id: u64,
    frame: u64,
    /// Origin moves since the last frame, added to every remembered point.
    shift: Vec3,
}

fn storage_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    read_only: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

const STORAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);

impl GpuTrails {
    /// Whether the device runs this path: compute, and storage buffers
    /// read in the vertex stage.
    pub(crate) fn supported(flags: wgpu::DownlevelFlags) -> bool {
        flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS | wgpu::DownlevelFlags::VERTEX_STORAGE)
    }

    /// The layout of `vs_strip`'s group 2 (the strip points), for the
    /// strip pipelines' layout.
    pub(crate) fn draw_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("strip points"),
            entries: &[storage_entry(0, wgpu::ShaderStages::VERTEX, true)],
        })
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        draw_layout: wgpu::BindGroupLayout,
        pipelines: [Sampled<wgpu::RenderPipeline>; 2],
    ) -> GpuTrails {
        let compute = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("trails"),
            entries: &[
                storage_entry(0, compute, true),
                storage_entry(1, compute, true),
                storage_entry(2, compute, false),
                storage_entry(3, compute, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: compute,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline = {
            let (device, layout) = (device.clone(), layout.clone());
            Deferred::new(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("trails"),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("trails"),
                            bind_group_layouts: &[Some(&layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("trails"),
                        source: wgpu::ShaderSource::Wgsl(WGSL.into()),
                    }),
                    entry_point: Some("advance"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let points = buffer(device, "strip points", POINT_BYTES * 256, STORAGE);
        let draw_group = Self::group(device, &draw_layout, &points);
        GpuTrails {
            slots: HashMap::new(),
            ranges: Ranges::default(),
            ring: buffer(
                device,
                "trail rings",
                RING_BYTES * 4096,
                STORAGE.union(wgpu::BufferUsages::COPY_SRC),
            ),
            jobs: buffer(device, "trail jobs", 64 * 64, STORAGE),
            styles: buffer(device, "trail styles", 320 * 16, STORAGE),
            count: buffer(
                device,
                "trail count",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            points,
            layout,
            pipeline,
            draw_layout,
            draw_group,
            pipelines,
            scene_id: 0,
            frame: 0,
            shift: Vec3::ZERO,
        }
    }

    fn group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        points: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("strip points"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: points.as_entire_binding(),
            }],
        })
    }

    /// Carries the histories across a move of the render origin
    /// (`origin.rs`): `d`, the old origin minus the new, is added to every
    /// point at the next step.
    pub(crate) fn shift(&mut self, d: Vec3) {
        self.shift += d;
    }

    /// Steps the trails' histories to `tick` and lays out every strip of
    /// the frame as runs of the strip point buffer (`vs_strip`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_id: u64,
        styles: &[TrailStyle],
        trails: &[Trail],
        trail_layers: &[(u64, i32)],
        ribbons: &[Ribbon],
        beams: &[Beam],
        tick: u64,
        tick_rate: f32,
    ) -> Result<RibbonFrame, String> {
        self.frame += 1;
        if self.scene_id != scene_id {
            self.scene_id = scene_id;
            self.slots.clear();
            self.ranges = Ranges::default();
        }
        let tick_rate = tick_rate.max(1e-3);
        let shift = std::mem::take(&mut self.shift);
        // What each strip draws: a trail's job, or the points of a ribbon
        // or beam.
        enum Body {
            Trail(usize),
            Points(crate::ribbons::Strip),
        }
        let mut entries: Vec<(Option<i32>, Option<TextureHandle>, ParticleBlend, Body)> =
            Vec::new();
        let mut jobs: Vec<JobGpu> = Vec::new();
        for t in trails {
            let style = styles
                .get(t.style.index())
                .ok_or_else(|| format!("trail {}: style {}", t.id, t.style.index()))?;
            // An id twice in a frame draws its first: one step a ring a frame.
            if self.slots.get(&t.id).is_some_and(|s| s.seen == self.frame) {
                continue;
            }
            let cap = (life_ticks(style, tick_rate) + 2).min(u64::from(MAX_TRAIL_POINTS)) as u32;
            // A new trail, or one whose lifetime changed its ring's size,
            // starts again in a ring of its own.
            let mut reset = false;
            if self.slots.get(&t.id).is_none_or(|s| s.cap != cap) {
                if let Some(old) = self.slots.remove(&t.id) {
                    self.ranges.release(old.base, old.cap + 1);
                }
                reset = true;
                let base = self.ranges.alloc(cap + 1);
                self.slots.insert(
                    t.id,
                    Slot {
                        base,
                        cap,
                        style: t.style.index(),
                        last_tick: t.tick,
                        seen: 0,
                    },
                );
            }
            let slot = self.slots.get_mut(&t.id).expect("made above");
            reset |= slot.style != t.style.index() || t.tick < slot.last_tick;
            slot.style = t.style.index();
            slot.last_tick = t.tick;
            slot.seen = self.frame;
            entries.push((
                trail_layer(trail_layers, t.id),
                style.texture,
                style.blend,
                Body::Trail(jobs.len()),
            ));
            jobs.push(JobGpu {
                head: t.position.extend(0.0).to_array(),
                shift: shift.extend(0.0).to_array(),
                base: slot.base,
                cap,
                tick: t.tick as u32,
                flags: u32::from(reset),
                style: t.style.index() as u32,
                out_first: 0,
                out_count: cap + 1,
                pad: 0,
            });
        }
        let frame = self.frame;
        let ranges = &mut self.ranges;
        self.slots.retain(|_, s| {
            let keep = s.seen == frame;
            if !keep {
                ranges.release(s.base, s.cap + 1);
            }
            keep
        });
        for s in fixed_strips(ribbons, beams, tick, tick_rate) {
            entries.push((s.layer, s.texture, s.blend, Body::Points(s)));
        }
        // Grouped by (texture, blend) for one draw each, layered strips
        // after the 3D ones by layer (as `Trails::build` sorts them).
        og_core::sort::sort_by_key(&mut entries, |e| (e.0, e.1, e.2 as u8));
        let mut out = RibbonFrame::default();
        let mut points: Vec<StripPoint> = Vec::new();
        for (layer, texture, blend, body) in &entries {
            let first = points.len() as u32;
            match body {
                Body::Trail(j) => {
                    jobs[*j].out_first = first;
                    let n = jobs[*j].out_count as usize;
                    points.resize(points.len() + n, StripPoint::default());
                }
                Body::Points(s) => strip_points(s, &mut points),
            }
            let count = points.len() as u32 - first;
            out.strips += 1;
            out.segments += count - 1;
            match out.runs.last_mut() {
                Some(r) if r.texture == *texture && r.blend == *blend && r.layer == *layer => {
                    r.count += count
                }
                _ => out.runs.push(RibbonRun {
                    texture: *texture,
                    blend: *blend,
                    layer: *layer,
                    points: true,
                    first,
                    count,
                }),
            }
        }
        if points.is_empty() {
            return Ok(out);
        }
        let bytes = points.len() as u64 * POINT_BYTES;
        if bytes > self.points.size() {
            self.points = buffer(device, "strip points", bytes.next_power_of_two(), STORAGE);
            self.draw_group = Self::group(device, &self.draw_layout, &self.points);
        }
        queue.write_buffer(&self.points, 0, bytemuck::cast_slice(&points));
        if jobs.is_empty() {
            return Ok(out);
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("trails"),
        });
        let ring_bytes = u64::from(self.ranges.end) * RING_BYTES;
        if ring_bytes > self.ring.size() {
            let grown = buffer(
                device,
                "trail rings",
                ring_bytes.next_power_of_two(),
                STORAGE.union(wgpu::BufferUsages::COPY_SRC),
            );
            encoder.copy_buffer_to_buffer(&self.ring, 0, &grown, 0, self.ring.size());
            self.ring = grown;
        }
        let style_data: Vec<StyleGpu> = styles.iter().map(|s| StyleGpu::of(s, tick_rate)).collect();
        let style_bytes: &[u8] = bytemuck::cast_slice(&style_data);
        if style_bytes.len() as u64 > self.styles.size() {
            self.styles = buffer(
                device,
                "trail styles",
                (style_bytes.len() as u64).next_power_of_two(),
                STORAGE,
            );
        }
        queue.write_buffer(&self.styles, 0, style_bytes);
        let job_bytes: &[u8] = bytemuck::cast_slice(&jobs);
        if job_bytes.len() as u64 > self.jobs.size() {
            self.jobs = buffer(
                device,
                "trail jobs",
                (job_bytes.len() as u64).next_power_of_two(),
                STORAGE,
            );
        }
        queue.write_buffer(&self.jobs, 0, job_bytes);
        queue.write_buffer(
            &self.count,
            0,
            bytemuck::cast_slice(&[jobs.len() as u32, 0, 0, 0]),
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("trails"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.jobs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.styles.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.ring.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.points.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.count.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("trails"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups((jobs.len() as u32).div_ceil(64), 1, 1);
        }
        queue.submit(Some(encoder.finish()));
        Ok(out)
    }
}

/// The trail step's module (compute).
pub(crate) const WGSL: &str = wgsl!("trails.wgsl");

#[cfg(test)]
mod tests {
    use super::Ranges;

    /// Freed rings are reused, join their neighbours whichever order they
    /// are freed in, and give the end back when they were last.
    #[test]
    fn ring_ranges_are_reused_and_joined() {
        let mut r = Ranges::default();
        let a = r.alloc(10);
        let b = r.alloc(20);
        let c = r.alloc(5);
        assert_eq!((a, b, c, r.end), (0, 10, 30, 35));
        r.release(b, 20);
        // First fit: a smaller ring takes the front of the hole.
        assert_eq!(r.alloc(4), 10);
        r.release(10, 4);
        r.release(a, 10);
        // a, the 4 and the rest of b's hole are one range now.
        assert_eq!(r.free, vec![(0, 30)]);
        assert_eq!(r.alloc(30), 0);
        r.release(0, 30);
        // The last ring freed: everything is free and the end is back at 0.
        r.release(c, 5);
        assert_eq!((r.free.len(), r.end), (0, 0));
    }
}
