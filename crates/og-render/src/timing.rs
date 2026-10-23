//! Frame timing: each render-graph pass's CPU encoding (from the clock the
//! host gives, [`crate::Renderer::set_clock`]) and, where the device has
//! timestamp queries, its GPU time ([`GpuTimer`]). Measurement only:
//! nothing drawn depends on it.
//!
//! The GPU time of a pass is the time between two timestamps the GPU
//! writes into the frame's encoder, one before the graph's first pass and
//! one after each pass. Each is written by an empty compute pass whose
//! only content is its beginning timestamp, which needs nothing past
//! `Features::TIMESTAMP_QUERY` (WebGPU's `timestamp-query`); arbitrary
//! timestamps inside an encoder (`TIMESTAMP_QUERY_INSIDE_ENCODERS`) are a
//! native extension WebGPU lacks, and timestamps on every pass descriptor
//! would reach into every pass's code. A timestamp lands once the work
//! before it is done (Vulkan writes it at the bottom of the pipe), so the
//! difference of two is the GPU's time for the pass between them,
//! including its wait for the previous one.
//!
//! The timestamps of a frame are resolved into a buffer, copied to a
//! readback and mapped after the submit; a later frame (or
//! [`crate::Renderer::gpu_timings`]) reads them, so the GPU's numbers
//! arrive a frame or two after the frame they measure, tagged with it.

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
    /// The passes in execution order: each one's start in milliseconds
    /// after the frame's first timestamp, and its GPU milliseconds.
    pub passes: Vec<GpuPass>,
}

/// One pass of a [`GpuFrame`].
#[derive(Clone, Debug, PartialEq)]
pub struct GpuPass {
    pub name: String,
    pub start: f64,
    pub ms: f64,
}

impl GpuFrame {
    /// The GPU milliseconds from the first timestamp to the last.
    pub fn ms(&self) -> f64 {
        self.passes.last().map_or(0.0, |p| p.start + p.ms)
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

/// Frames whose timestamps are in flight at once: a fourth frame finds no
/// readback free and is not timed on the GPU.
const SLOTS: usize = 3;

struct Slot {
    set: wgpu::QuerySet,
    /// Queries the set and buffers hold.
    count: u32,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    /// Set by `map_async`'s callback: whether the mapping succeeded.
    mapped: Arc<Mutex<Option<bool>>>,
    /// The frame written into it and its passes, until read.
    pending: Option<Pending>,
}

struct Pending {
    frame: u64,
    names: Vec<String>,
    submitted: f64,
}

/// The GPU side of the frame's timing: a ring of query sets with their
/// readbacks ([`SLOTS`]).
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
        gpu_timing_absent(device.features())
            .is_none()
            .then(|| GpuTimer {
                period_ns: f64::from(queue.get_timestamp_period()),
                slots: Vec::new(),
                current: None,
                done: Vec::new(),
            })
    }

    fn slot(device: &wgpu::Device, count: u32) -> Slot {
        let bytes = u64::from(count) * u64::from(wgpu::QUERY_SIZE);
        Slot {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("pass timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count,
            }),
            count,
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
            mapped: Arc::new(Mutex::new(None)),
            pending: None,
        }
    }

    /// Starts a frame of `passes` graph passes: the query set its
    /// timestamps go into (`passes + 1` of them), or `None` when every
    /// readback is still in flight.
    pub fn begin(&mut self, device: &wgpu::Device, passes: usize) -> Option<wgpu::QuerySet> {
        self.collect();
        let count = (passes as u32 + 1).min(wgpu::QUERY_SET_MAX_QUERIES);
        let free = self
            .slots
            .iter()
            .position(|s| s.pending.is_none() && s.count >= count);
        let i = match free {
            Some(i) => i,
            None if self.slots.len() < SLOTS => {
                self.slots.push(Self::slot(device, count.max(16)));
                self.slots.len() - 1
            }
            None => {
                // A free slot too small for this graph is made again.
                let i = self.slots.iter().position(|s| s.pending.is_none())?;
                self.slots[i] = Self::slot(device, count.max(16));
                i
            }
        };
        self.current = Some(i);
        Some(self.slots[i].set.clone())
    }

    /// Writes timestamp `index` of the frame's set: an empty compute pass
    /// that writes it as it begins (see the module's documentation).
    pub fn mark(encoder: &mut wgpu::CommandEncoder, set: &wgpu::QuerySet, index: u32) {
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass timestamp"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: set,
                beginning_of_pass_write_index: Some(index),
                end_of_pass_write_index: None,
            }),
        });
    }

    /// After the frame's last mark: resolves its `passes + 1` timestamps
    /// and copies them to the slot's readback.
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder, passes: usize) {
        let Some(i) = self.current else {
            return;
        };
        let s = &self.slots[i];
        let n = passes as u32 + 1;
        let bytes = u64::from(n) * u64::from(wgpu::QUERY_SIZE);
        encoder.resolve_query_set(&s.set, 0..n, &s.resolve, 0);
        encoder.copy_buffer_to_buffer(&s.resolve, 0, &s.read, 0, bytes);
    }

    /// After the submit of frame `frame`, whose passes were `names`, at
    /// `submitted` on the renderer's clock: maps the readback it copied
    /// into. A frame that failed before its submit never gets here, and
    /// its slot stays free.
    pub fn after_submit(&mut self, frame: u64, names: Vec<String>, submitted: f64) {
        let Some(i) = self.current.take() else {
            return;
        };
        let s = &mut self.slots[i];
        s.pending = Some(Pending {
            frame,
            names,
            submitted,
        });
        let mapped = s.mapped.clone();
        s.read
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                *mapped.lock().unwrap_or_else(|e| e.into_inner()) = Some(result.is_ok());
            });
    }

    /// Reads every readback that has landed into [`GpuTimer::done`].
    fn collect(&mut self) {
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
                let n = p.names.len() + 1;
                let ticks: Vec<u64> = {
                    let Ok(view) = s.read.slice(..).get_mapped_range() else {
                        s.read.unmap();
                        continue;
                    };
                    let (words, _) = view.as_chunks::<8>();
                    words
                        .iter()
                        .take(n)
                        .map(|b| u64::from_le_bytes(*b))
                        .collect()
                };
                self.done.push(frame_of(p, &ticks, self.period_ns));
            }
            s.read.unmap();
        }
    }

    /// The frames read back since the last take, oldest first. With `wait`
    /// (native), every frame in flight is waited for first.
    pub fn take(&mut self, device: &wgpu::Device, wait: bool) -> Vec<GpuFrame> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let poll = if wait && self.slots.iter().any(|s| s.pending.is_some()) {
                wgpu::PollType::wait_indefinitely()
            } else {
                wgpu::PollType::Poll
            };
            let _ = device.poll(poll);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = (device, wait);
        self.collect();
        std::mem::take(&mut self.done)
    }
}

/// A frame's passes from its timestamps: pass `k` runs from timestamp `k`
/// to `k + 1`. A timestamp before the one ahead of it (a GPU whose
/// counter is not monotonic across passes) gives the pass 0 ms.
fn frame_of(p: Pending, ticks: &[u64], period_ns: f64) -> GpuFrame {
    let first = ticks.first().copied().unwrap_or(0);
    let at = |t: u64| t.saturating_sub(first) as f64 * period_ns / 1e6;
    let passes = p
        .names
        .into_iter()
        .enumerate()
        .map(|(k, name)| {
            let (a, b) = (ticks.get(k).copied(), ticks.get(k + 1).copied());
            let start = a.map_or(0.0, at);
            let end = b.map_or(start, at).max(start);
            GpuPass {
                name,
                start,
                ms: end - start,
            }
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
    fn passes_lie_between_their_timestamps() {
        let p = Pending {
            frame: 7,
            names: vec!["Cascade0".into(), "Main".into(), "Bloom".into()],
            submitted: 100.0,
        };
        // 10 ns a tick: 1 ms is 100 000 ticks. Bloom's end is earlier than
        // its start (a counter that went back): 0 ms, never negative.
        let f = frame_of(p, &[5_000, 105_000, 405_000, 305_000], 10.0);
        assert_eq!(f.frame, 7);
        let got: Vec<(&str, f64, f64)> = f
            .passes
            .iter()
            .map(|p| (p.name.as_str(), p.start, p.ms))
            .collect();
        assert_eq!(
            got,
            [
                ("Cascade0", 0.0, 1.0),
                ("Main", 1.0, 3.0),
                ("Bloom", 4.0, 0.0)
            ]
        );
        assert_eq!(f.ms(), 4.0);
    }
}
