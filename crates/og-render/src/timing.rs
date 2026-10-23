//! Frame timing: each render-graph pass's CPU encoding (from the clock the
//! host gives, [`crate::Renderer::set_clock`]) and, where the device has
//! timestamp queries, its GPU time ([`GpuTimer`]). Measurement only:
//! nothing drawn depends on it.
//!
//! The GPU time comes from the timestamps WebGPU writes at a pass's
//! boundaries (`timestamp_writes`, the `timestamp-query` feature). The
//! frame's command encoder is an [`Encoder`]: every render or compute pass
//! it begins gets a pair of queries from the frame's set, one written as
//! the pass begins and one as it ends. This is the one measurement every
//! backend makes: Vulkan writes the pair at the top and the bottom of the
//! pipe; Metal samples the GPU's counters at the encoder's stage
//! boundaries (the vertex stage's start, the fragment stage's end).
//! Nothing else measures a pass there: Apple GPUs run encoders with no
//! data dependency at once, so a timestamp written by a separate marker
//! pass measured only the marker (every pass read a few microseconds on
//! Apple silicon), and for the same reason a pass's own beginning can lie
//! before the end of the pass before it (its vertex stage starts while the
//! previous fragments run). A render-graph pass is therefore reported
//! from the end of the pass before it to the last end of the passes it
//! opened (`ProbeTimings.gpuPasses`): the GPU's time in queue order,
//! which sums to the frame.
//!
//! The timestamps of a frame are resolved into a buffer, copied to a
//! readback and mapped after the submit; a later frame (or
//! [`crate::Renderer::gpu_timings`]) reads them, so the GPU's numbers
//! arrive a frame or two after the frame they measure, tagged with it.

use std::ops::Range;
use std::sync::{Arc, Mutex};

/// One render-graph pass of a frame: when its CPU encoding began (the
/// renderer's clock, [`crate::Renderer::set_clock`]) and how long it took.
#[derive(Clone, Debug, PartialEq)]
pub struct PassTiming {
    pub name: String,
    pub start: f64,
    pub ms: f64,
}

/// One frame's GPU time per render-graph pass, from timestamp queries.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuFrame {
    /// The frame's index ([`crate::FrameStats::frame`]).
    pub frame: u64,
    /// The renderer's clock when the frame was submitted: the GPU's own
    /// clock is not the CPU's, so a trace anchors the frame's first
    /// timestamp here.
    pub submitted: f64,
    /// The passes in execution order: each one's start and end in
    /// milliseconds after the frame's first timestamp, back to back (a
    /// pass starts at the very value the one before it ended, so a trace
    /// rounding both lands them on one microsecond).
    pub passes: Vec<GpuPass>,
}

/// One pass of a [`GpuFrame`].
#[derive(Clone, Debug, PartialEq)]
pub struct GpuPass {
    pub name: String,
    pub start: f64,
    pub end: f64,
}

impl GpuPass {
    /// The pass's GPU milliseconds.
    pub fn ms(&self) -> f64 {
        self.end - self.start
    }
}

impl GpuFrame {
    /// The GPU milliseconds from the first timestamp to the last: the last
    /// pass's end, the same value.
    pub fn ms(&self) -> f64 {
        self.passes.last().map_or(0.0, |p| p.end)
    }
}

/// Why a device has no GPU pass timing, or `None` when it has.
pub fn gpu_timing_absent(features: wgpu::Features) -> Option<&'static str> {
    if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
        None
    } else {
        Some("the adapter has no timestamp queries (wgpu's TIMESTAMP_QUERY)")
    }
}

/// A frame's timestamp queries: its set and the next pair to hand out
/// ([`GpuTimer::begin`]).
pub(crate) struct FrameMarks {
    set: wgpu::QuerySet,
    next: u32,
    count: u32,
}

impl FrameMarks {
    /// The set and the pair of queries the next pass writes, or `None`
    /// once the frame's set is full (the pass goes untimed).
    fn pair(&mut self) -> Option<(&wgpu::QuerySet, u32, u32)> {
        if self.next + 2 > self.count {
            return None;
        }
        let begin = self.next;
        self.next += 2;
        Some((&self.set, begin, begin + 1))
    }
}

/// The frame's command encoder: wgpu's, writing a pair of timestamps
/// around every render or compute pass it begins when the frame is timed
/// (the module's documentation). Everything else of the encoder is reached
/// through `Deref`.
pub(crate) struct Encoder {
    raw: wgpu::CommandEncoder,
    marks: Option<FrameMarks>,
}

impl std::ops::Deref for Encoder {
    type Target = wgpu::CommandEncoder;
    fn deref(&self) -> &wgpu::CommandEncoder {
        &self.raw
    }
}

impl std::ops::DerefMut for Encoder {
    fn deref_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.raw
    }
}

impl Encoder {
    /// An encoder timing its passes with `marks`, or none with `None`.
    pub fn new(raw: wgpu::CommandEncoder, marks: Option<FrameMarks>) -> Encoder {
        Encoder { raw, marks }
    }

    /// An encoder outside a timed frame (uploads, bakes, readbacks).
    pub fn untimed(raw: wgpu::CommandEncoder) -> Encoder {
        Encoder::new(raw, None)
    }

    /// Whether this frame's passes are timed.
    pub fn timed(&self) -> bool {
        self.marks.is_some()
    }

    /// The queries the passes begun so far wrote: pass `k` of the frame
    /// wrote `2k` and `2k + 1`.
    pub fn used(&self) -> u32 {
        self.marks.as_ref().map_or(0, |m| m.next)
    }

    /// `wgpu::CommandEncoder::begin_render_pass`, with the frame's
    /// timestamps around the pass unless `desc` brings its own.
    pub fn begin_render_pass<'e>(
        &'e mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'e> {
        let Encoder { raw, marks } = self;
        let pair = match desc.timestamp_writes {
            Some(_) => None,
            None => marks.as_mut().and_then(FrameMarks::pair),
        };
        let timestamp_writes = match (&desc.timestamp_writes, pair) {
            (Some(w), _) => Some(wgpu::RenderPassTimestampWrites {
                query_set: w.query_set,
                beginning_of_pass_write_index: w.beginning_of_pass_write_index,
                end_of_pass_write_index: w.end_of_pass_write_index,
            }),
            (None, Some((set, begin, end))) => Some(wgpu::RenderPassTimestampWrites {
                query_set: set,
                beginning_of_pass_write_index: Some(begin),
                end_of_pass_write_index: Some(end),
            }),
            (None, None) => None,
        };
        raw.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: desc.label,
            color_attachments: desc.color_attachments,
            depth_stencil_attachment: desc.depth_stencil_attachment.clone(),
            timestamp_writes,
            occlusion_query_set: desc.occlusion_query_set,
            multiview_mask: desc.multiview_mask,
        })
    }

    /// `wgpu::CommandEncoder::begin_compute_pass`, with the frame's
    /// timestamps around the pass unless `desc` brings its own.
    pub fn begin_compute_pass<'e>(
        &'e mut self,
        desc: &wgpu::ComputePassDescriptor<'_>,
    ) -> wgpu::ComputePass<'e> {
        let Encoder { raw, marks } = self;
        let pair = match desc.timestamp_writes {
            Some(_) => None,
            None => marks.as_mut().and_then(FrameMarks::pair),
        };
        let timestamp_writes = match (&desc.timestamp_writes, pair) {
            (Some(w), _) => Some(wgpu::ComputePassTimestampWrites {
                query_set: w.query_set,
                beginning_of_pass_write_index: w.beginning_of_pass_write_index,
                end_of_pass_write_index: w.end_of_pass_write_index,
            }),
            (None, Some((set, begin, end))) => Some(wgpu::ComputePassTimestampWrites {
                query_set: set,
                beginning_of_pass_write_index: Some(begin),
                end_of_pass_write_index: Some(end),
            }),
            (None, None) => None,
        };
        raw.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: desc.label,
            timestamp_writes,
        })
    }

    /// `wgpu::CommandEncoder::finish`.
    pub fn finish(self) -> wgpu::CommandBuffer {
        self.raw.finish()
    }
}

/// Frames whose timestamps are in flight at once. A frame that finds every
/// readback still in flight once the device was polled is not timed on
/// the GPU.
const SLOTS: usize = 8;

/// Queries a frame's set holds: 128 passes (the graph's passes open one
/// render or compute pass each, the post chain and a probe's capture
/// several).
const QUERIES: u32 = 256;

struct Slot {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    /// Set by the queue once the frame's work is done
    /// (`on_submitted_work_done`): its timestamps can be resolved.
    done: Arc<Mutex<bool>>,
    /// Set by `map_async`'s callback: whether the mapping succeeded.
    mapped: Arc<Mutex<Option<bool>>>,
    /// The frame written into it and its passes, until read.
    pending: Option<Pending>,
}

struct Pending {
    frame: u64,
    /// Each render-graph pass and the queries the passes it opened wrote.
    passes: Vec<(String, Range<u32>)>,
    /// Queries written, to resolve into the readback.
    queries: u32,
    submitted: f64,
    /// Whether the resolve and the copy to the readback were submitted.
    resolved: bool,
}

/// The GPU side of the frame's timing: a ring of query sets with their
/// readbacks ([`SLOTS`]).
///
/// A frame's timestamps are resolved in a command buffer of their own,
/// submitted once the queue reports the frame's work done: Metal's
/// counter samples are not complete until the command buffer that took
/// them has, so a resolve encoded at the end of the frame read zeros and
/// the previous frame's values on Apple silicon (Apple's counter-sampling
/// sample resolves from the completion handler for the same reason).
pub(crate) struct GpuTimer {
    /// Nanoseconds per timestamp tick (`Queue::get_timestamp_period`).
    period_ns: f64,
    slots: Vec<Slot>,
    /// The slot this frame writes, between [`GpuTimer::begin`] and
    /// [`GpuTimer::after_submit`].
    current: Option<usize>,
    /// Frames read back and not yet taken.
    done: Vec<GpuFrame>,
}

impl GpuTimer {
    /// A timer where the device has timestamp queries.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<GpuTimer> {
        if gpu_timing_absent(device.features()).is_some() {
            return None;
        }
        Some(GpuTimer {
            period_ns: f64::from(queue.get_timestamp_period()),
            slots: Vec::new(),
            current: None,
            done: Vec::new(),
        })
    }

    fn slot(device: &wgpu::Device) -> Slot {
        let bytes = u64::from(QUERIES) * u64::from(wgpu::QUERY_SIZE);
        Slot {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("pass timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: QUERIES,
            }),
            resolve: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("pass timestamps resolve"),
                size: bytes,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            read: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("pass timestamps readback"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            done: Arc::new(Mutex::new(false)),
            mapped: Arc::new(Mutex::new(None)),
            pending: None,
        }
    }

    /// Starts a frame: the marks its passes write, or `None` when every
    /// readback is still in flight.
    pub fn begin(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<FrameMarks> {
        self.collect(device, queue);
        let free = |slots: &[Slot]| slots.iter().position(|s| s.pending.is_none());
        let mut i = free(&self.slots);
        if i.is_none() {
            // Completions and readbacks land in callbacks the device runs
            // when polled: a frame whose GPU work is done may still hold
            // its slot.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = device.poll(wgpu::PollType::Poll);
            self.collect(device, queue);
            i = free(&self.slots);
        }
        let i = match i {
            Some(i) => i,
            None if self.slots.len() < SLOTS => {
                self.slots.push(Self::slot(device));
                self.slots.len() - 1
            }
            None => return None,
        };
        self.current = Some(i);
        Some(FrameMarks {
            set: self.slots[i].set.clone(),
            next: 0,
            count: QUERIES,
        })
    }

    /// After the submit of frame `frame`, whose render-graph passes were
    /// `passes` (each with the queries it wrote; `queries` in all, the
    /// encoder's [`Encoder::used`]), at `submitted` on the renderer's
    /// clock: asks the queue to say when the frame's work is done, so its
    /// timestamps can be resolved. A frame that failed before its submit
    /// never gets here, and its slot stays free.
    pub fn after_submit(
        &mut self,
        queue: &wgpu::Queue,
        frame: u64,
        passes: Vec<(String, Range<u32>)>,
        queries: u32,
        submitted: f64,
    ) {
        let Some(i) = self.current.take() else {
            return;
        };
        if queries == 0 {
            return;
        }
        let s = &mut self.slots[i];
        s.pending = Some(Pending {
            frame,
            passes,
            queries,
            submitted,
            resolved: false,
        });
        *s.done.lock().unwrap_or_else(|e| e.into_inner()) = false;
        let done = s.done.clone();
        queue.on_submitted_work_done(move || {
            *done.lock().unwrap_or_else(|e| e.into_inner()) = true;
        });
    }

    /// Resolves the timestamps of every frame whose work the queue
    /// reported done and maps the readback, then reads every readback
    /// that has landed into [`GpuTimer::done`].
    fn collect(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for s in &mut self.slots {
            let Some(p) = s.pending.as_mut().filter(|p| !p.resolved) else {
                continue;
            };
            if !*s.done.lock().unwrap_or_else(|e| e.into_inner()) {
                continue;
            }
            p.resolved = true;
            let n = p.queries;
            let bytes = u64::from(n) * u64::from(wgpu::QUERY_SIZE);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pass timestamps resolve"),
            });
            encoder.resolve_query_set(&s.set, 0..n, &s.resolve, 0);
            encoder.copy_buffer_to_buffer(&s.resolve, 0, &s.read, 0, bytes);
            queue.submit(Some(encoder.finish()));
            let mapped = s.mapped.clone();
            s.read
                .slice(..bytes)
                .map_async(wgpu::MapMode::Read, move |result| {
                    *mapped.lock().unwrap_or_else(|e| e.into_inner()) = Some(result.is_ok());
                });
        }
        for s in &mut self.slots {
            let landed = s.mapped.lock().unwrap_or_else(|e| e.into_inner()).take();
            let Some(ok) = landed else {
                continue;
            };
            let pending = s.pending.take();
            if !ok {
                continue;
            }
            if let Some(p) = pending {
                let ticks: Vec<u64> = {
                    let Ok(view) = s
                        .read
                        .slice(..u64::from(p.queries) * u64::from(wgpu::QUERY_SIZE))
                        .get_mapped_range()
                    else {
                        s.read.unmap();
                        continue;
                    };
                    let (words, _) = view.as_chunks::<8>();
                    words.iter().map(|b| u64::from_le_bytes(*b)).collect()
                };
                self.done.push(frame_of(p, &ticks, self.period_ns));
            }
            s.read.unmap();
        }
    }

    /// The frames read back since the last take, oldest first. With `wait`
    /// (native), every frame in flight is waited for first: once for its
    /// work, once for the resolve of its timestamps.
    pub fn take(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        wait: bool,
    ) -> Vec<GpuFrame> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let in_flight = |slots: &[Slot]| slots.iter().any(|s| s.pending.is_some());
            let poll = |wait: bool| {
                let _ = device.poll(if wait {
                    wgpu::PollType::wait_indefinitely()
                } else {
                    wgpu::PollType::Poll
                });
            };
            poll(wait && in_flight(&self.slots));
            self.collect(device, queue);
            poll(wait && in_flight(&self.slots));
        }
        self.collect(device, queue);
        std::mem::take(&mut self.done)
    }
}

/// A frame's passes from its timestamps: pass `k` of a render-graph pass
/// wrote queries `2k` and `2k + 1`, and the graph pass ends at the last
/// end among them. Each graph pass runs from the end of the one before it
/// (the frame's first beginning for the first) to its own end, in queue
/// order (the module's documentation). A query the GPU left at 0 is not
/// read; a pass with none ends where the one before it did, 0 ms long.
fn frame_of(p: Pending, ticks: &[u64], period_ns: f64) -> GpuFrame {
    let span = |r: &Range<u32>| -> Option<(u64, u64)> {
        let mut span: Option<(u64, u64)> = None;
        for q in (r.start..r.end).step_by(2) {
            let (Some(&b), Some(&e)) = (ticks.get(q as usize), ticks.get(q as usize + 1)) else {
                break;
            };
            if b == 0 || e == 0 {
                continue;
            }
            span = Some(span.map_or((b, e), |(sb, se)| (sb.min(b), se.max(e))));
        }
        span
    };
    let spans: Vec<Option<(u64, u64)>> = p.passes.iter().map(|(_, r)| span(r)).collect();
    let first = spans.iter().flatten().map(|s| s.0).min().unwrap_or(0);
    let ms = |ticks: u64| ticks.saturating_sub(first) as f64 * period_ns / 1e6;
    let mut prev_end = 0.0;
    let passes = p
        .passes
        .into_iter()
        .zip(spans)
        .map(|((name, _), span)| {
            let end = span.map_or(prev_end, |(_, e)| ms(e).max(prev_end));
            let pass = GpuPass {
                name,
                start: prev_end,
                end,
            };
            prev_end = end;
            pass
        })
        .collect();
    GpuFrame {
        frame: p.frame,
        submitted: p.submitted,
        passes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_run_back_to_back_to_the_ends_of_the_passes_they_opened() {
        let p = Pending {
            frame: 7,
            passes: vec![
                ("Cascade0".into(), 0..2),
                ("Main".into(), 2..6),
                ("Bloom".into(), 6..6),
                ("Tonemap".into(), 6..8),
            ],
            queries: 8,
            submitted: 100.0,
            resolved: true,
        };
        // 10 ns a tick: 1 ms is 100 000 ticks. Main opened two passes, the
        // second begun before the first ended (an Apple GPU overlapping
        // encoders): it ends at the later end. Bloom opened none: 0 ms
        // where Main ended. Tonemap's end was left at 0 by the GPU: its
        // one pair is not read, so it sits there too.
        let f = frame_of(
            p,
            &[
                5_000, 105_000, 95_000, 305_000, 255_000, 405_000, 405_000, 0,
            ],
            10.0,
        );
        assert_eq!(f.frame, 7);
        let got: Vec<(&str, f64, f64)> = f
            .passes
            .iter()
            .map(|p| (p.name.as_str(), p.start, p.end))
            .collect();
        assert_eq!(
            got,
            [
                ("Cascade0", 0.0, 1.0),
                ("Main", 1.0, 4.0),
                ("Bloom", 4.0, 4.0),
                ("Tonemap", 4.0, 4.0)
            ]
        );
        assert_eq!(f.passes[1].ms(), 3.0);
        assert_eq!(f.ms(), 4.0);
    }
}
