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
//! Apple silicon).
//!
//! A render-graph pass is reported by its own time: when each pass it
//! opened ran on the GPU, from its beginning to its end
//! ([`GpuPass::busy`], overlapping ones merged), and their length
//! ([`GpuPass::ms`]). Passes the GPU ran at once each report their whole
//! time, so a frame's passes may sum past it; the frame's span is its own
//! ([`GpuFrame::ms`], the first beginning to the last end). Laid back to
//! back in queue order instead (each pass from the end of the one before
//! it), a pass that overlapped an earlier, longer one read 0 and the one
//! it overlapped took its time.
//!
//! A pass that cannot be timed on its own says why ([`GpuUntimed`]) and
//! has no time, never 0 ms: it opened no render or compute pass (copies
//! only), the frame's queries ran out before it, or the GPU wrote none of
//! its timestamps (a query never written resolves to 0, WebGPU's
//! unwritten query). A frame with no query read is not reported
//! ([`frame_of`]).
//!
//! The timestamps of a frame are resolved into a buffer, copied to a
//! readback and mapped after the submit; a later frame (or
//! [`crate::Renderer::gpu_timings`]) reads them, so the GPU's numbers
//! arrive a frame or two after the frame they measure, tagged with it.
//!
//! A `publish` build times no pass on the GPU: [`GpuTimer`] and
//! [`FrameMarks`] are types without values there, so the query sets, their
//! resolves and readbacks and the encoder's pairs are not in the module
//! ([`gpu_timing_absent`] says why; the CPU's pass timings stay).

#[cfg(not(feature = "publish"))]
use std::ops::Range;
#[cfg(not(feature = "publish"))]
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
    /// The render-graph passes, in queue order.
    pub passes: Vec<GpuPass>,
}

/// One render-graph pass of a [`GpuFrame`]: when it ran on the GPU, or
/// why it was not timed.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuPass {
    pub name: String,
    /// When the passes it opened ran: each from its beginning to its end,
    /// in milliseconds after the frame's first timestamp, in order, those
    /// that overlapped merged. Empty when it is [`GpuPass::untimed`].
    pub busy: Vec<(f64, f64)>,
    /// Why it has no time of its own, or `None` when it has.
    pub untimed: Option<GpuUntimed>,
}

/// Why a render-graph pass has no GPU time of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuUntimed {
    /// It opened no render or compute pass for timestamps to bracket
    /// (copies or clears only, or nothing this frame).
    NoGpuPass,
    /// The frame's timestamp queries ran out before it.
    QueriesFull,
    /// The GPU wrote none of its timestamps.
    NotWritten,
}

impl GpuUntimed {
    /// The reason, in words, for a trace or a report.
    pub fn reason(self) -> &'static str {
        match self {
            GpuUntimed::NoGpuPass => "it opened no render or compute pass to time",
            GpuUntimed::QueriesFull => "the frame's timestamp queries ran out before it",
            GpuUntimed::NotWritten => "the GPU wrote none of its timestamps",
        }
    }
}

impl GpuPass {
    /// The pass's own GPU milliseconds (the length of [`GpuPass::busy`]),
    /// or `None` when it is untimed.
    pub fn ms(&self) -> Option<f64> {
        self.untimed
            .is_none()
            .then(|| self.busy.iter().map(|(b, e)| e - b).sum())
    }
}

impl GpuFrame {
    /// The frame's span on the GPU: from its first timestamp to its last.
    pub fn ms(&self) -> f64 {
        self.passes
            .iter()
            .filter_map(|p| p.busy.last())
            .map(|b| b.1)
            .fold(0.0, f64::max)
    }
}

/// Why a device has no GPU pass timing, or `None` when it has.
pub fn gpu_timing_absent(features: wgpu::Features) -> Option<&'static str> {
    if cfg!(feature = "publish") {
        Some("a publish build times no pass on the GPU")
    } else if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
        None
    } else {
        Some("the adapter has no timestamp queries (wgpu's TIMESTAMP_QUERY)")
    }
}

/// A frame's timestamp queries: its set and the next pair to hand out
/// ([`GpuTimer::begin`]).
#[cfg(not(feature = "publish"))]
pub(crate) struct FrameMarks {
    set: wgpu::QuerySet,
    next: u32,
    count: u32,
}

/// No frame is timed in a `publish` build (the module's documentation).
#[cfg(feature = "publish")]
pub(crate) enum FrameMarks {}

impl FrameMarks {
    /// The set and the pair of queries the next pass writes, or `None`
    /// once the frame's set is full (the pass goes untimed).
    #[cfg(feature = "publish")]
    fn pair(&mut self) -> Option<(&wgpu::QuerySet, u32, u32)> {
        match *self {}
    }

    /// The set and the pair of queries the next pass writes, or `None`
    /// once the frame's set is full (the pass goes untimed).
    #[cfg(not(feature = "publish"))]
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
    /// Debug builds: the device's capabilities and what the pass being
    /// encoded declared it needs ([`Encoder::expect`]).
    #[cfg(all(debug_assertions, not(feature = "publish")))]
    expected: Option<(crate::Caps, crate::Caps)>,
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
        Encoder {
            raw,
            marks,
            #[cfg(all(debug_assertions, not(feature = "publish")))]
            expected: None,
        }
    }

    /// From here on, what the encoder opens is checked (debug builds
    /// only, never `publish`): against `device`, the device's capabilities,
    /// and `declared`, what the pass being encoded declared it needs
    /// (`renderer/render/schedule.rs`; `device` again outside a declared
    /// pass). A compute pass needs [`crate::Caps::COMPUTE`], a render pass
    /// with more than one colour target [`crate::Caps::MRT`], a timestamp
    /// write [`crate::Caps::TIMESTAMPS`] (of the device alone: the frame's
    /// timing is not a pass's). A use outside either panics, naming the
    /// pass, so a declaration that is wrong fails every test that encodes
    /// the pass, on any device.
    #[cfg_attr(
        not(all(debug_assertions, not(feature = "publish"))),
        allow(unused_variables)
    )]
    pub fn expect(&mut self, device: crate::Caps, declared: crate::Caps) {
        #[cfg(all(debug_assertions, not(feature = "publish")))]
        {
            self.expected = Some((device, declared));
        }
    }

    /// Checks `uses` for the pass `label` (see [`Encoder::expect`]).
    #[cfg_attr(
        not(all(debug_assertions, not(feature = "publish"))),
        allow(unused_variables)
    )]
    fn uses(&self, label: Option<&str>, uses: crate::Caps, timestamps: bool) {
        #[cfg(all(debug_assertions, not(feature = "publish")))]
        if let Some((device, declared)) = self.expected {
            let label = label.unwrap_or("(unlabelled)");
            let lacking = device.missing(
                uses | if timestamps {
                    crate::Caps::TIMESTAMPS
                } else {
                    crate::Caps::NONE
                },
            );
            assert!(
                lacking == crate::Caps::NONE,
                "og-render: pass '{label}' uses {lacking:?}, which the device's capabilities lack"
            );
            let undeclared = declared.missing(uses);
            assert!(
                undeclared == crate::Caps::NONE,
                "og-render: pass '{label}' uses {undeclared:?}, which its declaration leaves out (renderer/render/schedule.rs)"
            );
        }
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
    #[cfg(not(feature = "publish"))]
    pub fn used(&self) -> u32 {
        self.marks.as_ref().map_or(0, |m| m.next)
    }

    /// The queries the passes begun so far wrote: none in a `publish` build.
    #[cfg(feature = "publish")]
    pub fn used(&self) -> u32 {
        0
    }

    /// `wgpu::CommandEncoder::begin_render_pass`, with the frame's
    /// timestamps around the pass unless `desc` brings its own.
    pub fn begin_render_pass<'e>(
        &'e mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'e> {
        let targets = desc.color_attachments.iter().flatten().count();
        let mrt = if targets > 1 {
            crate::Caps::MRT
        } else {
            crate::Caps::NONE
        };
        self.uses(desc.label, mrt, desc.timestamp_writes.is_some());
        let Encoder { raw, marks, .. } = self;
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
        self.uses(
            desc.label,
            crate::Caps::COMPUTE,
            desc.timestamp_writes.is_some(),
        );
        let Encoder { raw, marks, .. } = self;
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

#[cfg(not(feature = "publish"))]
/// Frames whose timestamps are in flight at once. A frame that finds every
/// readback still in flight once the device was polled is not timed on
/// the GPU.
const SLOTS: usize = 8;

#[cfg(not(feature = "publish"))]
/// Queries a frame's set holds: 128 passes (the graph's passes open one
/// render or compute pass each, the post chain and a probe's capture
/// several).
const QUERIES: u32 = 256;

#[cfg(not(feature = "publish"))]
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

#[cfg(not(feature = "publish"))]
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

#[cfg(not(feature = "publish"))]
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

#[cfg(not(feature = "publish"))]
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
                self.done.extend(frame_of(p, &ticks, self.period_ns));
            }
            s.read.unmap();
        }
    }

    /// The frames read back since the last take, oldest first. With `wait`
    /// (native), every frame in flight is waited for first: once for its
    /// work, once for the resolve of its timestamps. On the web nothing
    /// waits: a frame is read back when its map has resolved.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
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

/// No GPU timing in a `publish` build: no timer is ever made, so every
/// method below is unreachable and nothing of the timing is linked.
#[cfg(feature = "publish")]
pub(crate) enum GpuTimer {}

#[cfg(feature = "publish")]
impl GpuTimer {
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> Option<GpuTimer> {
        None
    }

    pub fn begin(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) -> Option<FrameMarks> {
        match *self {}
    }

    pub fn after_submit(
        &mut self,
        _queue: &wgpu::Queue,
        _frame: u64,
        _passes: Vec<(String, std::ops::Range<u32>)>,
        _queries: u32,
        _submitted: f64,
    ) {
        match *self {}
    }

    pub fn take(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _wait: bool,
    ) -> Vec<GpuFrame> {
        match *self {}
    }
}

#[cfg(not(feature = "publish"))]
/// A frame's passes from its timestamps: pass `k` the frame's encoder
/// began wrote queries `2k` and `2k + 1`, and a render-graph pass lists
/// the queries of the passes it opened. Each graph pass is busy over its
/// own passes' beginnings and ends, overlaps merged (the module's
/// documentation); a pair with a query the GPU left at 0 is not read. A
/// graph pass with no pair read is untimed, with why. A frame with no
/// query read is no frame (`None`).
fn frame_of(p: Pending, ticks: &[u64], period_ns: f64) -> Option<GpuFrame> {
    let pairs = |r: &Range<u32>| -> Vec<(u64, u64)> {
        (r.start..r.end)
            .step_by(2)
            .map_while(|q| Some((*ticks.get(q as usize)?, *ticks.get(q as usize + 1)?)))
            .filter(|&(b, e)| b != 0 && e != 0)
            .map(|(b, e)| (b, e.max(b)))
            .collect()
    };
    let read: Vec<Vec<(u64, u64)>> = p.passes.iter().map(|(_, r)| pairs(r)).collect();
    let first = read.iter().flatten().map(|s| s.0).min()?;
    let ms = |ticks: u64| ticks.saturating_sub(first) as f64 * period_ns / 1e6;
    let passes = p
        .passes
        .into_iter()
        .zip(read)
        .map(|((name, r), mut pairs)| {
            pairs.sort_unstable();
            let mut busy: Vec<(u64, u64)> = Vec::with_capacity(pairs.len());
            for (b, e) in pairs {
                match busy.last_mut() {
                    Some(last) if b <= last.1 => last.1 = last.1.max(e),
                    _ => busy.push((b, e)),
                }
            }
            let untimed = if !busy.is_empty() {
                None
            } else if !r.is_empty() {
                Some(GpuUntimed::NotWritten)
            } else if r.start + 2 > QUERIES {
                Some(GpuUntimed::QueriesFull)
            } else {
                Some(GpuUntimed::NoGpuPass)
            };
            GpuPass {
                name,
                busy: busy.into_iter().map(|(b, e)| (ms(b), ms(e))).collect(),
                untimed,
            }
        })
        .collect();
    Some(GpuFrame {
        frame: p.frame,
        submitted: p.submitted,
        passes,
    })
}

#[cfg(all(test, not(feature = "publish")))]
mod tests {
    use super::*;

    #[test]
    fn each_pass_reports_its_own_time_and_overlapping_passes_keep_theirs() {
        let p = || Pending {
            frame: 7,
            passes: vec![
                ("Cull".into(), 0..2),
                ("Cascade0".into(), 2..4),
                ("Main".into(), 4..8),
                ("Bloom".into(), 8..8),
                ("Tonemap".into(), 8..10),
                ("Late".into(), QUERIES..QUERIES),
            ],
            queries: 10,
            submitted: 100.0,
            resolved: true,
        };
        // 10 ns a tick: 1 ms is 100 000 ticks. The cull runs 0 to 8 ms
        // and the cascade, which needs nothing of it, runs at once, 1 to
        // 4 ms: laid back to back in queue order the cascade read 0 ms.
        // Main opened two passes, the second begun before the first ended
        // (an Apple GPU overlapping encoders): one stretch. Bloom opened
        // none; Tonemap's end was left at 0 by the GPU; Late came after
        // the frame's queries ran out: untimed, each with why.
        let f = frame_of(
            p(),
            &[
                1, 800_001, 100_001, 400_001, 800_001, 1_000_001, 950_001, 1_100_001, 1_100_001, 0,
            ],
            10.0,
        )
        .expect("a frame with queries read");
        assert_eq!(f.frame, 7);
        let got: Vec<(&str, Option<f64>, Option<GpuUntimed>)> = f
            .passes
            .iter()
            .map(|p| (p.name.as_str(), p.ms(), p.untimed))
            .collect();
        assert_eq!(
            got,
            [
                ("Cull", Some(8.0), None),
                ("Cascade0", Some(3.0), None),
                ("Main", Some(3.0), None),
                ("Bloom", None, Some(GpuUntimed::NoGpuPass)),
                ("Tonemap", None, Some(GpuUntimed::NotWritten)),
                ("Late", None, Some(GpuUntimed::QueriesFull)),
            ]
        );
        assert_eq!(f.passes[1].busy, [(1.0, 4.0)]);
        assert_eq!(f.passes[2].busy, [(8.0, 11.0)]);
        assert_eq!(
            f.ms(),
            11.0,
            "the frame's span, first beginning to last end"
        );
        // Every query left at 0 (none written): no frame, not one of
        // untimed passes.
        assert_eq!(frame_of(p(), &[0; 10], 10.0), None);
    }
}

#[cfg(all(test, feature = "publish"))]
mod publish_tests {
    #[test]
    fn a_publish_build_reports_no_gpu_timing_even_where_the_adapter_has_timestamps() {
        assert_eq!(
            super::gpu_timing_absent(wgpu::Features::TIMESTAMP_QUERY),
            Some("a publish build times no pass on the GPU")
        );
    }
}
