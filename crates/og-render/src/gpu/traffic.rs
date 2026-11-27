//! What the renderer moves between the CPU and the GPU, counted where it
//! moves (`og run --perf`): every upload (`write_buffer`,
//! `write_texture`), every buffer made, every readback started
//! (`map_async`), every blocking wait for the GPU, and every pipeline
//! made, with its time. The crate makes each through this module only
//! (og-render's clippy.toml bans the wgpu calls), so nothing moves
//! uncounted.
//!
//! Off by default: each call checks one flag and goes straight to wgpu.
//! On ([`set_clock`]), each event adds to the frame's totals
//! ([`take_frame`], one [`Traffic`] per frame) and to its call site's
//! totals in a fixed table ([`each_site`]), found by the address of its
//! `#[track_caller]` location: no strings, maps or allocation per event.
//! The renderer's thread and the compile workers share the counts, which
//! belong to the process (a process drawing two renderers sums them).

use std::panic::Location;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// One frame's movement between the CPU and the GPU.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Traffic {
    /// Bytes written by `write_buffer` and `write_texture`, and the calls.
    pub upload_bytes: u64,
    pub uploads: u32,
    /// Buffers made, and their bytes.
    pub buffers: u32,
    pub buffer_bytes: u64,
    /// Readbacks started (`map_async`).
    pub readbacks: u32,
    /// Blocking waits for the GPU on the renderer's thread, and their time
    /// (the frame stood still for each).
    pub waits: u32,
    pub wait_us: u32,
    /// Pipelines made, and the time making them took on the thread that
    /// made each (a compile worker's is not the frame's).
    pub pipelines: u32,
    pub pipeline_us: u32,
}

/// What a call site does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Upload,
    Buffer,
    Readback,
    Wait,
    Pipeline,
}

/// A call site's totals over the run.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub at: &'static Location<'static>,
    pub kind: Kind,
    pub count: u64,
    /// Bytes uploaded or made; 0 for the others.
    pub bytes: u64,
    /// Milliseconds waited or compiling; 0 for the others.
    pub ms: f64,
    /// The frames ([`take_frame`]'s count) of its first and last event.
    pub first_frame: u64,
    pub last_frame: u64,
    /// A pipeline's label (its descriptor's, cut to fit): one builder
    /// makes many pipelines, so a pipeline's site is its builder and label.
    name: [u8; NAME],
}

const NAME: usize = 40;

impl Site {
    /// The pipeline's label; empty for the other kinds.
    pub fn name(&self) -> &str {
        let n = self.name.iter().position(|&b| b == 0).unwrap_or(NAME);
        std::str::from_utf8(&self.name[..n]).unwrap_or("")
    }
}

fn name_of(label: Option<&str>) -> [u8; NAME] {
    let mut out = [0u8; NAME];
    let s = label.unwrap_or("");
    let mut n = s.len().min(NAME);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    out[..n].copy_from_slice(&s.as_bytes()[..n]);
    out
}

const SITES: usize = 512;

struct State {
    clock: Option<fn() -> f64>,
    frame: Traffic,
    frames: u64,
    sites: [Option<Site>; SITES],
    /// Events whose site found the table full.
    lost: u64,
}

static ON: AtomicBool = AtomicBool::new(false);
static STATE: Mutex<State> = Mutex::new(State {
    clock: None,
    frame: EMPTY,
    frames: 0,
    sites: [None; SITES],
    lost: 0,
});

const EMPTY: Traffic = Traffic {
    upload_bytes: 0,
    uploads: 0,
    buffers: 0,
    buffer_bytes: 0,
    readbacks: 0,
    waits: 0,
    wait_us: 0,
    pipelines: 0,
    pipeline_us: 0,
};

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Turns the counting on with the clock it times waits and pipelines with
/// (ms), from zero, or off (`None`).
pub fn set_clock(clock: Option<fn() -> f64>) {
    let mut s = state();
    s.clock = clock;
    s.frame = EMPTY;
    s.frames = 0;
    s.sites = [None; SITES];
    s.lost = 0;
    ON.store(clock.is_some(), Ordering::Relaxed);
}

/// The movement since the last call, which closes a frame.
pub fn take_frame() -> Traffic {
    if !ON.load(Ordering::Relaxed) {
        return EMPTY;
    }
    let mut s = state();
    s.frames += 1;
    std::mem::replace(&mut s.frame, EMPTY)
}

/// Every call site seen since the counting turned on, and the events lost
/// to a full table.
pub fn each_site(mut f: impl FnMut(&Site)) -> u64 {
    let s = state();
    s.sites.iter().flatten().for_each(&mut f);
    s.lost
}

fn now() -> Option<f64> {
    if ON.load(Ordering::Relaxed) {
        state().clock.map(|c| c())
    } else {
        None
    }
}

fn count(at: &'static Location<'static>, kind: Kind, bytes: u64, ms: f64) {
    count_named(at, kind, bytes, ms, [0; NAME]);
}

fn count_named(at: &'static Location<'static>, kind: Kind, bytes: u64, ms: f64, name: [u8; NAME]) {
    let mut s = state();
    let t = &mut s.frame;
    match kind {
        Kind::Upload => {
            t.uploads += 1;
            t.upload_bytes += bytes;
        }
        Kind::Buffer => {
            t.buffers += 1;
            t.buffer_bytes += bytes;
        }
        Kind::Readback => t.readbacks += 1,
        Kind::Wait => {
            t.waits += 1;
            t.wait_us = t.wait_us.saturating_add((ms * 1000.0) as u32);
        }
        Kind::Pipeline => {
            t.pipelines += 1;
            t.pipeline_us = t.pipeline_us.saturating_add((ms * 1000.0) as u32);
        }
    }
    let frame = s.frames;
    let start = (std::ptr::from_ref(at) as usize >> 3).wrapping_add(
        name.iter()
            .fold(0usize, |h, &b| h.wrapping_mul(31).wrapping_add(b as usize)),
    ) % SITES;
    for i in (start..SITES).chain(0..start) {
        match &mut s.sites[i] {
            Some(site) if std::ptr::eq(site.at, at) && site.kind == kind && site.name == name => {
                site.count += 1;
                site.bytes += bytes;
                site.ms += ms;
                site.last_frame = frame;
                return;
            }
            Some(_) => {}
            slot @ None => {
                *slot = Some(Site {
                    at,
                    kind,
                    count: 1,
                    bytes,
                    ms,
                    first_frame: frame,
                    last_frame: frame,
                    name,
                });
                return;
            }
        }
    }
    s.lost += 1;
}

/// A buffer made with its contents (`crate::buffer_init`'s native
/// `create_buffer_init`, which makes and fills it in one call), counted as
/// the buffer made and the bytes uploaded at its caller.
#[track_caller]
pub(crate) fn made_with_contents(size: u64, bytes: u64) {
    if ON.load(Ordering::Relaxed) {
        let at = Location::caller();
        count(at, Kind::Buffer, size, 0.0);
        if bytes > 0 {
            count(at, Kind::Upload, bytes, 0.0);
        }
    }
}

/// `Queue::write_buffer`, counted.
#[track_caller]
pub fn write_buffer(queue: &wgpu::Queue, buffer: &wgpu::Buffer, offset: u64, data: &[u8]) {
    if ON.load(Ordering::Relaxed) {
        count(Location::caller(), Kind::Upload, data.len() as u64, 0.0);
    }
    #[allow(clippy::disallowed_methods)]
    queue.write_buffer(buffer, offset, data);
}

/// `Queue::write_texture`, counted.
#[track_caller]
pub fn write_texture(
    queue: &wgpu::Queue,
    texture: wgpu::TexelCopyTextureInfo<'_>,
    data: &[u8],
    layout: wgpu::TexelCopyBufferLayout,
    size: wgpu::Extent3d,
) {
    if ON.load(Ordering::Relaxed) {
        count(Location::caller(), Kind::Upload, data.len() as u64, 0.0);
    }
    #[allow(clippy::disallowed_methods)]
    queue.write_texture(texture, data, layout, size);
}

/// `Device::create_buffer`, counted.
#[track_caller]
pub fn create_buffer(device: &wgpu::Device, desc: &wgpu::BufferDescriptor<'_>) -> wgpu::Buffer {
    if ON.load(Ordering::Relaxed) {
        count(Location::caller(), Kind::Buffer, desc.size, 0.0);
    }
    #[allow(clippy::disallowed_methods)]
    device.create_buffer(desc)
}

/// `BufferSlice::map_async`, counted as a readback.
#[track_caller]
pub fn map_async(
    slice: &wgpu::BufferSlice<'_>,
    mode: wgpu::MapMode,
    callback: impl FnOnce(Result<(), wgpu::BufferAsyncError>) + wgpu::WasmNotSend + 'static,
) {
    if ON.load(Ordering::Relaxed) {
        count(Location::caller(), Kind::Readback, 0, 0.0);
    }
    #[allow(clippy::disallowed_methods)]
    slice.map_async(mode, callback);
}

/// `Device::poll`: a blocking one (`PollType::Wait`) is counted and timed
/// as a wait; a plain `Poll` goes uncounted.
#[track_caller]
pub fn poll(
    device: &wgpu::Device,
    poll: wgpu::PollType,
) -> Result<wgpu::PollStatus, wgpu::PollError> {
    let at = Location::caller();
    let t0 = if poll.is_wait() { now() } else { None };
    #[allow(clippy::disallowed_methods)]
    let r = device.poll(poll);
    if let Some(t0) = t0 {
        let ms = now().map_or(0.0, |t| (t - t0).max(0.0));
        count(at, Kind::Wait, 0, ms);
    }
    r
}

/// `Device::create_render_pipeline`, counted and timed.
#[track_caller]
pub fn create_render_pipeline(
    device: &wgpu::Device,
    desc: &wgpu::RenderPipelineDescriptor<'_>,
) -> wgpu::RenderPipeline {
    let at = Location::caller();
    // On the web, an entry of the pipeline cache asks for it asynchronously
    // (gpu/deferred.rs).
    #[cfg(target_arch = "wasm32")]
    if let Some(p) = crate::deferred::render_pipeline(device, desc) {
        return p;
    }
    let t0 = now();
    #[allow(clippy::disallowed_methods)]
    let p = device.create_render_pipeline(desc);
    if let Some(t0) = t0 {
        count_named(
            at,
            Kind::Pipeline,
            0,
            now().map_or(0.0, |t| (t - t0).max(0.0)),
            name_of(desc.label),
        );
    }
    p
}

/// `Device::create_compute_pipeline`, counted and timed.
#[track_caller]
pub fn create_compute_pipeline(
    device: &wgpu::Device,
    desc: &wgpu::ComputePipelineDescriptor<'_>,
) -> wgpu::ComputePipeline {
    let at = Location::caller();
    #[cfg(target_arch = "wasm32")]
    if let Some(p) = crate::deferred::compute_pipeline(device, desc) {
        return p;
    }
    let t0 = now();
    #[allow(clippy::disallowed_methods)]
    let p = device.create_compute_pipeline(desc);
    if let Some(t0) = t0 {
        count_named(
            at,
            Kind::Pipeline,
            0,
            now().map_or(0.0, |t| (t - t0).max(0.0)),
            name_of(desc.label),
        );
    }
    p
}
