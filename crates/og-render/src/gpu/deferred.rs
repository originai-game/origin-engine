//! GPU objects made the first time a frame uses them.
//!
//! Every pipeline is a program to compile before a pass draws with it:
//! the renderer making all of its fixed pipelines up front cost the web
//! page of the retired WebGL2 tier 400 to 550 ms before its first frame
//! (web finding #11), most of them for passes the first frame
//! does not draw (water, metaballs, fluids, terrain, scatter,
//! particles, 2D lights, ribbons, bloom, depth of field…). A [`Deferred`]
//! pipeline is made from the same descriptor when a pass first sets it, so
//! the first frame compiles what it draws and nothing else, and every frame
//! is the frame it was. [`Renderer::warm`](crate::Renderer::warm) makes
//! the rest one at a time when the host has idle time (behind the web
//! page's play gate), so a pass met later does not stop a frame to compile.
//!
//! **Compiling ahead, natively.** A pipeline compiles on the CPU (DXC and
//! the driver on DX12 take over a second for the terrain's lit stages),
//! and a frame used to make what its passes met one after another. A value
//! made by [`Deferred::anywhere`] (or [`Sampled::anywhere`]) can be started
//! on a worker thread ([`Deferred::start`]) as soon as the frame knows it
//! will draw it, before its passes are encoded; the pass that sets it waits
//! for it there, so the frame's compiles overlap each other and the CPU
//! work between them. wgpu's native device makes pipelines from any thread
//! (Bevy's pipeline cache compiles on a task pool the same way); the
//! pipelines are the same objects from the same descriptors, so nothing a
//! frame draws changes. On the web a value is made where it is first used,
//! as before: the browser's GPU process compiles it, and the host's frame
//! span asks for it asynchronously (`FrameSpan`).

use std::cell::{Cell, OnceCell};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, OnceLock};

/// A closure that makes a value off the renderer's thread (natively; on
/// the web, where nothing is made off it, any closure).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type MakeAnywhere<T> = Box<dyn FnOnce() -> T + Send>;
#[cfg(target_arch = "wasm32")]
pub(crate) type MakeAnywhere<T> = Box<dyn FnOnce() -> T>;

/// What a closure or module needs to be made off the renderer's thread:
/// `Send + Sync` natively, nothing on the web.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait Anywhere: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync> Anywhere for T {}
#[cfg(target_arch = "wasm32")]
pub(crate) trait Anywhere {}
#[cfg(target_arch = "wasm32")]
impl<T> Anywhere for T {}

/// What a closure run once needs to be run off the renderer's thread:
/// `Send` natively, nothing on the web.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait AnywhereOnce: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> AnywhereOnce for T {}
#[cfg(target_arch = "wasm32")]
pub(crate) trait AnywhereOnce {}
#[cfg(target_arch = "wasm32")]
impl<T> AnywhereOnce for T {}

/// How a [`Deferred`] value is made.
enum Make<T> {
    /// On first use, here.
    Here(Box<dyn FnOnce() -> T>),
    /// On first use, or on a worker thread once started.
    Anywhere(MakeAnywhere<T>),
    /// Started on a worker thread.
    #[cfg(not(target_arch = "wasm32"))]
    Started(Pending<T>),
}

impl<T> Make<T> {
    fn make(self) -> T {
        match self {
            Make::Here(make) => timed(make),
            Make::Anywhere(make) => timed(make),
            #[cfg(not(target_arch = "wasm32"))]
            Make::Started(worker) => joined(worker),
        }
    }
}

/// A value being made on a compile worker ([`spawn`]).
///
/// Dropped unreceived, it owns its job still: one not yet started is taken
/// off the queues and dropped here, and one running is waited for. A job's
/// closure owns GPU objects (the device, layouts, modules); detached, it
/// went on compiling in the driver after its renderer was gone, and on
/// NVIDIA's Linux driver a worker still inside the shader compiler when
/// the process exited crashed it (SIGSEGV, heap corruption) after the
/// test had passed.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct Pending<T> {
    answer: std::sync::mpsc::Receiver<std::thread::Result<T>>,
    /// Its job's id in [`Queues`].
    id: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Drop for Pending<T> {
    fn drop(&mut self) {
        if let Some(job) = cancel(self.id) {
            drop(job);
            return;
        }
        // Running, or done (its answer, received or not, is dropped here;
        // a panic of work nobody uses is not raised again).
        let _ = self.answer.recv();
    }
}

/// A worker's value, waited for, its panic passed on.
#[cfg(not(target_arch = "wasm32"))]
fn joined<T>(pending: Pending<T>) -> T {
    if let Ok(answer) = pending.answer.try_recv() {
        return match answer {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
    }
    // A background compile waited for runs now, ahead of the others (they
    // still wait for the frame: [`release_background`]).
    promote(pending.id);
    let t0 = std::time::Instant::now();
    let answer = pending.answer.recv();
    WAITED_US.fetch_add(
        t0.elapsed().as_micros() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
    match answer {
        Ok(Ok(value)) => value,
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => unreachable!("a compile worker always answers"),
    }
}

/// `make()` on the renderer's thread, its time counted as waited
/// ([`take_waited_us`]): what a frame spends making a pipeline it meets is
/// time it does not draw.
fn timed<T>(make: impl FnOnce() -> T) -> T {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let t0 = std::time::Instant::now();
        let v = make();
        WAITED_US.fetch_add(
            t0.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        v
    }
    #[cfg(target_arch = "wasm32")]
    make()
}

/// A job for the compile workers.
#[cfg(not(target_arch = "wasm32"))]
type Job = Box<dyn FnOnce() + Send>;

/// Microseconds the renderer's thread has spent on pipelines: waiting for
/// the compile workers, or making one itself ([`take_waited_us`]).
#[cfg(not(target_arch = "wasm32"))]
static WAITED_US: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The time the renderer's thread spent on compiles since the last call
/// (`FrameStats::compile_wait_us`; 0 on the web, which waits for none).
pub(crate) fn take_waited_us() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    return WAITED_US.swap(0, std::sync::atomic::Ordering::Relaxed);
    #[cfg(target_arch = "wasm32")]
    0
}

/// When a started value compiles.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Priority {
    /// For a frame that is waiting or will wait for it: before any
    /// background work.
    Frame,
    /// Ahead of a need that may come (a pipeline variant with every
    /// feature, [`crate::pipeline_features::Variants`]): held until the
    /// frame that asked has been submitted ([`release_background`]), then
    /// compiled while no frame's work is queued.
    Background,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Queues {
    /// Every job is queued with its id ([`Pending`]).
    frame: std::collections::VecDeque<(u64, Job)>,
    /// Released background jobs, each with its id (a wait for one moves
    /// it to `frame`: [`promote`]).
    background: std::collections::VecDeque<(u64, Job)>,
    held: Vec<(u64, Job)>,
    next: u64,
    /// Jobs a worker is running.
    running: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl Queues {
    fn idle(&self) -> bool {
        self.frame.is_empty()
            && self.background.is_empty()
            && self.held.is_empty()
            && self.running == 0
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct Pool {
    queues: Mutex<Queues>,
    work: std::sync::Condvar,
    /// Notified when the last running job ends with nothing queued
    /// ([`settle`]).
    idle: std::sync::Condvar,
}

/// The compile workers: one a core, started on first use and kept for the
/// process. A bounded pool, not a thread a pipeline: a scene's first frame
/// can ask for dozens at once.
#[cfg(not(target_arch = "wasm32"))]
fn pool() -> &'static Pool {
    static POOL: OnceLock<&'static Pool> = OnceLock::new();
    POOL.get_or_init(|| {
        let pool: &'static Pool = Box::leak(Box::new(Pool {
            queues: Mutex::new(Queues::default()),
            work: std::sync::Condvar::new(),
            idle: std::sync::Condvar::new(),
        }));
        let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
        for i in 0..workers {
            std::thread::Builder::new()
                .name(format!("og-render compile {i}"))
                .spawn(move || {
                    loop {
                        let job = {
                            let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
                            loop {
                                if let Some((_, job)) =
                                    q.frame.pop_front().or_else(|| q.background.pop_front())
                                {
                                    q.running += 1;
                                    break job;
                                }
                                q = pool.work.wait(q).unwrap_or_else(|e| e.into_inner());
                            }
                        };
                        job();
                        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
                        q.running -= 1;
                        if q.idle() {
                            pool.idle.notify_all();
                        }
                    }
                })
                .expect("a pipeline compile worker");
        }
        pool
    })
}

/// Lets the background compiles asked for so far start (the frame that
/// asked for them has been submitted: [`Priority::Background`]).
pub(crate) fn release_background() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let pool = pool();
        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
        if q.held.is_empty() {
            return;
        }
        let held = std::mem::take(&mut q.held);
        q.background.extend(held);
        pool.work.notify_all();
    }
}

/// Waits until every pipeline compile started so far has finished, in
/// every renderer of the process, the full lit variants compiling in the
/// background included (pipeline_features.rs): from then on the frames
/// draw with the full variants of every pipeline met so far. Display
/// frames drawn while a variant compiles may differ from later ones in
/// their last bit (docs/design.md §2.4), so a test that compares display
/// frames byte for byte settles them first. Natively; nothing on the web,
/// which makes the full variants only.
pub fn settle_pipelines() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        release_background();
        let pool = pool();
        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
        while !q.idle() {
            q = pool.idle.wait(q).unwrap_or_else(|e| e.into_inner());
        }
    }
}

/// Runs `make` on the compile workers, in the order started within its
/// priority (a frame starts its longest compiles first).
#[cfg(not(target_arch = "wasm32"))]
fn spawn<T: Send + 'static>(
    priority: Priority,
    make: impl FnOnce() -> T + Send + 'static,
) -> Pending<T> {
    let (answer, pending) = std::sync::mpsc::channel();
    let job: Job = Box::new(move || {
        let made = std::panic::catch_unwind(std::panic::AssertUnwindSafe(make));
        let _ = answer.send(made);
    });
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let id = q.next;
    q.next += 1;
    match priority {
        Priority::Frame => {
            q.frame.push_back((id, job));
            pool.work.notify_one();
        }
        Priority::Background => q.held.push((id, job)),
    }
    Pending {
        answer: pending,
        id,
    }
}

/// Takes the job `id` off the queues if no worker has started it (its
/// [`Pending`] was dropped); the caller drops it, outside the lock.
#[cfg(not(target_arch = "wasm32"))]
fn cancel(id: u64) -> Option<Job> {
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let job = if let Some(i) = q.held.iter().position(|(j, _)| *j == id) {
        Some(q.held.remove(i).1)
    } else if let Some(i) = q.background.iter().position(|(j, _)| *j == id) {
        q.background.remove(i).map(|(_, job)| job)
    } else {
        let i = q.frame.iter().position(|(j, _)| *j == id);
        i.and_then(|i| q.frame.remove(i)).map(|(_, job)| job)
    };
    if job.is_some() && q.idle() {
        pool.idle.notify_all();
    }
    job
}

/// Moves the background job `id`, held or queued, to the front of the
/// frame's queue (the renderer waits for it); a frame's job, or a running
/// or finished one, is in neither.
#[cfg(not(target_arch = "wasm32"))]
fn promote(id: u64) {
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let job = if let Some(i) = q.held.iter().position(|(j, _)| *j == id) {
        Some(q.held.remove(i).1)
    } else {
        let i = q.background.iter().position(|(j, _)| *j == id);
        i.and_then(|i| q.background.remove(i)).map(|(_, job)| job)
    };
    if let Some(job) = job {
        q.frame.push_front((id, job));
        pool.work.notify_one();
    }
}

/// A value made by its closure on first use (deref), at most once.
pub(crate) struct Deferred<T> {
    value: OnceCell<T>,
    make: Cell<Option<Make<T>>>,
}

impl<T> Deferred<T> {
    pub(crate) fn new(make: impl FnOnce() -> T + 'static) -> Deferred<T> {
        Deferred {
            value: OnceCell::new(),
            make: Cell::new(Some(Make::Here(Box::new(make)))),
        }
    }

    /// One that [`Start::start`] can make on a worker thread ahead of its
    /// first use (on the web, made on first use).
    pub(crate) fn anywhere(make: impl FnOnce() -> T + AnywhereOnce + 'static) -> Deferred<T> {
        Deferred {
            value: OnceCell::new(),
            make: Cell::new(Some(Make::Anywhere(Box::new(make)))),
        }
    }

    /// The value if it is made, or was started and has been made since,
    /// without waiting.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn ready(&self) -> Option<&T> {
        if let Some(v) = self.value.get() {
            return Some(v);
        }
        #[cfg(not(target_arch = "wasm32"))]
        match self.make.take() {
            Some(Make::Started(worker)) => match worker.answer.try_recv() {
                Ok(Ok(value)) => return Some(self.value.get_or_init(|| value)),
                Ok(Err(panic)) => std::panic::resume_unwind(panic),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.make.set(Some(Make::Started(worker)));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    unreachable!("a compile worker always answers")
                }
            },
            other => self.make.set(other),
        }
        None
    }

    /// The value, made now if it was not yet (waited for, if started).
    pub(crate) fn get(&self) -> &T {
        self.value.get_or_init(|| {
            self.make
                .take()
                .expect("a deferred value is made once, by its first use")
                .make()
        })
    }

    /// Whether the value has been made.
    pub(crate) fn made(&self) -> bool {
        self.value.get().is_some()
    }
}

/// Something a frame can start making ahead of the pass that uses it.
pub(crate) trait Start {
    /// Starts making it on a worker thread if it can be made there and is
    /// neither made nor started (natively; on the web nothing).
    fn start(&self);
}

impl<T: AnywhereOnce + 'static> Deferred<T> {
    /// [`Start::start`] at `priority`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn start_at(&self, priority: Priority) {
        match self.make.take() {
            Some(Make::Anywhere(make)) => self.make.set(Some(Make::Started(spawn(priority, make)))),
            other => self.make.set(other),
        }
    }
}

impl<T: AnywhereOnce + 'static> Start for Deferred<T> {
    #[cfg(not(target_arch = "wasm32"))]
    fn start(&self) {
        self.start_at(Priority::Frame);
    }

    #[cfg(target_arch = "wasm32")]
    fn start(&self) {}
}

/// Starts every one of `items` (see [`Start`]).
pub(crate) fn start_all(items: &[&dyn Start]) {
    for item in items {
        item.start();
    }
}

impl<T> Deref for Deferred<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for Deferred<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get();
        self.value.get_mut().expect("made just now")
    }
}

/// A pipeline (or anything else) that depends on the renderer's samples
/// per pixel: made on first use at the current count, as a [`Deferred`],
/// and again at the first use after [`Sampled::set_samples`] changed the
/// count (`RenderQuality::msaa_samples`). One is kept per count (one
/// sample, [`crate::MSAA_SAMPLES`]), so going back costs nothing.
pub(crate) struct Sampled<T> {
    samples: Cell<u32>,
    made: [OnceCell<T>; 2],
    /// A count's value started on a worker thread ([`Sampled::anywhere`]).
    #[cfg(not(target_arch = "wasm32"))]
    started: [Cell<Option<Pending<T>>>; 2],
    make: SampledMake<T>,
}

enum SampledMake<T> {
    Here(Box<dyn Fn(u32) -> T>),
    #[cfg(not(target_arch = "wasm32"))]
    Anywhere(Arc<dyn Fn(u32) -> T + Send + Sync>),
}

impl<T> Sampled<T> {
    fn with(samples: u32, make: SampledMake<T>) -> Sampled<T> {
        Sampled {
            samples: Cell::new(samples),
            made: [OnceCell::new(), OnceCell::new()],
            #[cfg(not(target_arch = "wasm32"))]
            started: [Cell::new(None), Cell::new(None)],
            make,
        }
    }

    pub(crate) fn new(samples: u32, make: impl Fn(u32) -> T + 'static) -> Sampled<T> {
        Sampled::with(samples, SampledMake::Here(Box::new(make)))
    }

    /// One whose value at a count [`Start::start`] can make on a worker
    /// thread ahead of its first use (on the web, made on first use).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn anywhere(
        samples: u32,
        make: impl Fn(u32) -> T + Anywhere + 'static,
    ) -> Sampled<T> {
        Sampled::with(samples, SampledMake::Anywhere(Arc::new(make)))
    }

    /// One whose value at a count [`Start::start`] can make on a worker
    /// thread ahead of its first use (on the web, made on first use).
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn anywhere(samples: u32, make: impl Fn(u32) -> T + 'static) -> Sampled<T> {
        Sampled::new(samples, make)
    }

    fn index(&self) -> usize {
        (self.samples.get() > 1) as usize
    }

    /// The value at the current count, made now if it was not yet (waited
    /// for, if started).
    pub(crate) fn get(&self) -> &T {
        let i = self.index();
        self.made[i].get_or_init(|| {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(worker) = self.started[i].take() {
                return joined(worker);
            }
            match &self.make {
                SampledMake::Here(make) => timed(|| make(self.samples.get())),
                #[cfg(not(target_arch = "wasm32"))]
                SampledMake::Anywhere(make) => timed(|| make(self.samples.get())),
            }
        })
    }

    /// The value at the current count if it is made, or was started and
    /// has been made since, without waiting.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn ready(&self) -> Option<&T> {
        let i = self.index();
        if let Some(v) = self.made[i].get() {
            return Some(v);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.started[i].take() {
            match worker.answer.try_recv() {
                Ok(Ok(value)) => return Some(self.made[i].get_or_init(|| value)),
                Ok(Err(panic)) => std::panic::resume_unwind(panic),
                Err(_) => self.started[i].set(Some(worker)),
            }
        }
        None
    }

    /// Draw at `samples` per pixel from now on.
    pub(crate) fn set_samples(&self, samples: u32) {
        self.samples.set(samples);
    }

    /// The samples per pixel it draws at.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn samples(&self) -> u32 {
        self.samples.get()
    }

    /// Whether the value at the current count has been made.
    pub(crate) fn made(&self) -> bool {
        self.made[self.index()].get().is_some()
    }
}

impl<T: AnywhereOnce + 'static> Sampled<T> {
    /// [`Start::start`] at `priority`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn start_at(&self, priority: Priority) {
        let i = self.index();
        let SampledMake::Anywhere(make) = &self.make else {
            return;
        };
        if self.made() {
            return;
        }
        match self.started[i].take() {
            Some(worker) => self.started[i].set(Some(worker)),
            None => {
                let (make, samples) = (make.clone(), self.samples.get());
                self.started[i].set(Some(spawn(priority, move || make(samples))));
            }
        }
    }
}

impl<T: AnywhereOnce + 'static> Start for Sampled<T> {
    #[cfg(not(target_arch = "wasm32"))]
    fn start(&self) {
        self.start_at(Priority::Frame);
    }

    #[cfg(target_arch = "wasm32")]
    fn start(&self) {}
}

impl<T> Deref for Sampled<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> Warm for Sampled<T> {
    fn warm(&self) -> bool {
        if self.made() {
            return false;
        }
        self.get();
        true
    }
}

/// A shader module parsed when the first pipeline that uses it is made,
/// on whichever thread that is ([`Deferred::anywhere`]).
pub(crate) struct ModuleCell {
    /// The optional features whose overrides its source declares.
    pub(crate) declared: crate::pipeline_features::Features,
    value: OnceLock<wgpu::ShaderModule>,
    make: Mutex<Option<MakeAnywhere<wgpu::ShaderModule>>>,
}

impl ModuleCell {
    /// The module, parsed now if it was not yet.
    pub(crate) fn get(&self) -> &wgpu::ShaderModule {
        self.value.get_or_init(|| {
            let make = self
                .make
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take()
                .expect("a module is parsed once, by its first use");
            make()
        })
    }
}

impl Deref for ModuleCell {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &wgpu::ShaderModule {
        self.get()
    }
}

/// A shader module parsed when the first pipeline that uses it is made.
pub(crate) type Module = Arc<ModuleCell>;

/// A WGSL module made on first use ([`Module`]).
pub(crate) fn module(device: &wgpu::Device, label: &'static str, source: String) -> Module {
    module_with(device, label, source, crate::shader)
}

/// [`module`] for a module in [`crate::UNBOUNDED_MODULES`].
pub(crate) fn unbounded_module(
    device: &wgpu::Device,
    label: &'static str,
    source: String,
) -> Module {
    module_with(device, label, source, crate::unbounded_shader)
}

// The web's wgpu objects are neither Send nor Sync; nothing there leaves
// the renderer's thread.
#[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
fn module_with(
    device: &wgpu::Device,
    label: &'static str,
    source: String,
    make: fn(&wgpu::Device, &str, String) -> wgpu::ShaderModule,
) -> Module {
    let device = device.clone();
    Arc::new(ModuleCell {
        declared: crate::pipeline_features::Features::declared(&source),
        value: OnceLock::new(),
        make: Mutex::new(Some(Box::new(move || make(&device, label, source)))),
    })
}

/// A shader module a deferred pipeline is made from: one already made, or a [`Module`].
pub(crate) trait AsModule {
    fn module(&self) -> &wgpu::ShaderModule;
}

impl AsModule for wgpu::ShaderModule {
    fn module(&self) -> &wgpu::ShaderModule {
        self
    }
}

impl AsModule for Module {
    fn module(&self) -> &wgpu::ShaderModule {
        self.get()
    }
}

/// Something [`Renderer::warm`](crate::Renderer::warm) can make ahead of use.
pub(crate) trait Warm {
    /// Make it now if it is not made; true when this call made it.
    fn warm(&self) -> bool;
}

impl<T> Warm for Deferred<T> {
    fn warm(&self) -> bool {
        if self.made() {
            return false;
        }
        self.get();
        true
    }
}

/// Makes the first of `items` not yet made; false when all are.
pub(crate) fn warm_one(items: &[&dyn Warm]) -> bool {
    items.iter().any(|w| w.warm())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn made_once_on_first_use_and_warmed_in_order() {
        let made = std::rc::Rc::new(Cell::new(0));
        let m = made.clone();
        let a = Deferred::new(move || {
            m.set(m.get() + 1);
            7
        });
        assert!(!a.made());
        assert_eq!(made.get(), 0);
        assert_eq!(*a, 7);
        assert_eq!(*a.get(), 7);
        assert_eq!(made.get(), 1);
        let b = Deferred::new(|| "b");
        let c = Deferred::new(|| "c");
        assert!(warm_one(&[&a, &b, &c]));
        assert!(b.made() && !c.made());
        assert!(warm_one(&[&a, &b, &c]));
        assert!(!warm_one(&[&a, &b, &c]));
        let mut d = Deferred::new(|| vec![1]);
        d.push(2);
        assert_eq!(*d, vec![1, 2]);
    }

    /// A started value is made once, on a compile worker, and its first use
    /// waits for it; a worker's panic reaches that first use.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn started_values_are_made_on_a_worker_and_waited_for() {
        let here = std::thread::current().id();
        let made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let m = made.clone();
        let a = Deferred::anywhere(move || {
            m.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::current().id()
        });
        a.start();
        a.start();
        assert_ne!(*a.get(), here);
        assert_eq!(made.load(std::sync::atomic::Ordering::SeqCst), 1);
        let s = Sampled::anywhere(4, |n| (n, std::thread::current().id()));
        s.start();
        assert_eq!(s.get().0, 4);
        assert_ne!(s.get().1, here);
        let p = Deferred::anywhere(|| -> u32 { panic!("compile failed") });
        p.start();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| *p.get()));
        assert_eq!(
            panic.unwrap_err().downcast_ref::<&str>(),
            Some(&"compile failed")
        );
    }

    /// A started value dropped before its first use leaves no work behind:
    /// one still queued is dropped with it, with what its closure owns, and
    /// one a worker is making is waited for.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_dropped_started_value_owns_its_compile() {
        use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
        let resource = Arc::new(());
        let captured = resource.clone();
        let held = Deferred::anywhere(move || captured);
        held.start_at(Priority::Background);
        drop(held);
        assert_eq!(Arc::strong_count(&resource), 1, "a held compile is dropped");
        let captured = resource.clone();
        let sampled = Sampled::anywhere(4, move |_| captured.clone());
        sampled.start_at(Priority::Background);
        drop(sampled);
        assert_eq!(
            Arc::strong_count(&resource),
            1,
            "a held sampled compile is dropped"
        );
        let (began, done) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        );
        let (b, d) = (began.clone(), done.clone());
        let running = Deferred::anywhere(move || {
            b.store(true, SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(200));
            d.store(true, SeqCst);
        });
        running.start();
        while !began.load(SeqCst) {
            std::thread::yield_now();
        }
        drop(running);
        assert!(done.load(SeqCst), "a running compile is waited for");
    }
}
