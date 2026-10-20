//! Lanes: data-parallel work inside one system or one frame step, with the
//! same results as running it on one thread (docs/design.md §2.2).
//!
//! The engine's world, its tables and the TypeScript systems stay on one
//! thread; a lane is a pure function of one item and of state shared read
//! only, run for every item of a slice. The helpers return their results in
//! item order, so whatever a caller does with them (insert into the world,
//! reduce, write a buffer) happens serially in the order a single thread
//! would have used. Nothing depends on which thread ran an item or when, so
//! world hashes and replays are identical with lanes on and off.
//!
//! Where the threads come from:
//! - natively (the addon, `og`), a rayon pool of `OG_LANES` threads
//!   (`off`, `0` or `1` for none; unset, the cores up to 8);
//! - in the lanes wasm (built with `+atomics,+bulk-memory`), rayon on a pool
//!   of Workers sharing the wasm memory through wasm-bindgen-rayon, which
//!   the engine Worker starts when the page is cross-origin isolated and
//!   then announces with [`og_lanes_ready`];
//! - in the single-threaded wasm, nowhere: every helper runs serially.
//!
//! A library built on rayon (rapier's `parallel` build, in og-physics) runs
//! through [`install`]: on the pool when lanes are on and the call has two
//! grains of items, otherwise on the calling thread inside a one-thread pool,
//! so the library finds one thread and never reaches rayon's global pool.
//!
//! The probe's `lanes:<on|off>` switches them at run time ([`set_enabled`])
//! and `dump_lanes` reports what the engine got ([`report`]), with the
//! parallel calls counted per [`Site`].
//!
//! Panics: a host's panic hook hands every Rust panic to [`record_panic`],
//! which keeps the first one with the tick and, when it happened inside a
//! lane, the lane's [`Site`] ([`panic_report`]). Natively a lane's panic
//! reaches the calling thread through rayon. In the lanes wasm a panic
//! aborts the Worker it happens on, so the engine thread never waits on
//! rayon's own latch: it hands the call to the pool and waits on one of its
//! own, which a recorded panic also wakes, and then panics itself with the
//! lane's message. The engine Worker therefore traps and reports the crash
//! instead of sleeping in `Atomics.wait` for a lane that will never end
//! (docs/web-export.md §Lanes). The call goes to the pool through a relay,
//! one pool Worker that runs whatever the engine thread sends it, since
//! `rayon::spawn` reaches the pool its caller belongs to, and the engine
//! thread belongs to its own one-thread pool once a call ran serially.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

static SWITCH: AtomicBool = AtomicBool::new(true);
static CALLS: [AtomicU64; Site::COUNT] = [const { AtomicU64::new(0) }; Site::COUNT];
static ITEMS: [AtomicU64; Site::COUNT] = [const { AtomicU64::new(0) }; Site::COUNT];

/// Where a helper call comes from: each caller names its site, so the
/// report says which systems split, not only that something did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    /// og-anim: clip and graph evaluation, the joint palettes.
    Anim,
    /// og-runtime's extraction: decoding a component's rows.
    Extract,
    /// og-render's CPU frustum cull.
    Cull,
    /// og-physics: the step (rapier's `parallel` build, [`install`]) and
    /// the contact pairs it reads back.
    Physics,
}

impl Site {
    /// The number of sites.
    pub const COUNT: usize = 4;
    /// Every site, in report order.
    pub const ALL: [Site; Site::COUNT] = [Site::Anim, Site::Extract, Site::Cull, Site::Physics];

    fn from_index(i: usize) -> Option<Site> {
        Site::ALL.get(i).copied()
    }

    /// The site of a report name (`anim`, `extract`, `cull`, `physics`).
    pub fn from_name(name: &str) -> Option<Site> {
        Site::ALL.into_iter().find(|s| s.name() == name)
    }

    /// The site's name in the report (`anim`, `extract`, `cull`, `physics`).
    pub fn name(self) -> &'static str {
        match self {
            Site::Anim => "anim",
            Site::Extract => "extract",
            Site::Cull => "cull",
            Site::Physics => "physics",
        }
    }
}

/// The parallel calls of one [`Site`] since the start and the items they ran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SiteCount {
    pub calls: u64,
    pub items: u64,
}

fn count(site: Site, items: usize) {
    CALLS[site as usize].fetch_add(1, Ordering::Relaxed);
    ITEMS[site as usize].fetch_add(items as u64, Ordering::Relaxed);
}

/// What the engine got: whether lanes run, on how many threads, and why
/// they do not when they do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    /// Lanes run in parallel now.
    pub on: bool,
    /// The pool's threads (1 when there is none).
    pub threads: u32,
    /// Where the threads come from: `native` (rayon), `web-workers`
    /// (wasm-bindgen-rayon) or `none`.
    pub pool: &'static str,
    /// Why lanes are serial, empty when they are on.
    pub why: &'static str,
    /// The helper calls that ran in parallel since the start ([`install`]
    /// counts one call of the items it was given, og-physics's bodies).
    pub parallel_calls: u64,
    /// The items those calls ran.
    pub parallel_items: u64,
    /// The same per [`Site`], indexed by `site as usize` (the totals above
    /// are their sums).
    pub sites: [SiteCount; Site::COUNT],
}

// ── panics ──────────────────────────────────────────────────────────────────

/// The site of the last call that went parallel, plus one (0: none yet).
static SITE: AtomicUsize = AtomicUsize::new(0);
static PANICKED: AtomicBool = AtomicBool::new(false);
static PANIC: Mutex<Option<PanicReport>> = Mutex::new(None);
/// Rung when a job handed to the pool ends or a panic is recorded.
static GATE: Mutex<()> = Mutex::new(());
static BELL: Condvar = Condvar::new();

thread_local! {
    /// How many lane items this thread is inside (a pool thread running a
    /// helper's items).
    static IN_LANE: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A Rust panic the engine recorded ([`record_panic`]): what it said, where,
/// at which tick, and the lane's site when it happened inside one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanicReport {
    /// The panic's message.
    pub message: String,
    /// `file:line:column` of the panic, empty when unknown.
    pub location: String,
    /// The tick the world was running (the host passes it; 0 before the first).
    pub tick: u64,
    /// The site of the lane it happened in; `None` on the engine's own thread.
    pub lane: Option<Site>,
}

impl PanicReport {
    /// One line: `panic at tick N in a lane (site): message (location)`.
    pub fn describe(&self) -> String {
        let lane = self
            .lane
            .map_or(String::new(), |s| format!(" in a lane ({})", s.name()));
        let at = if self.location.is_empty() {
            String::new()
        } else {
            format!(" ({})", self.location)
        };
        format!("panic at tick {}{lane}: {}{at}", self.tick, self.message)
    }
}

/// An injected fault's site, plus one (0: none).
static FAULT: AtomicUsize = AtomicUsize::new(0);

/// A test hook (the web host's `og_web_lanes_fault`; the browser test of a
/// lane's panic): the next item a helper runs at `site` panics, inside a
/// lane when the call splits and on the calling thread when it runs
/// serially. Once.
pub fn inject_fault(site: Site) {
    FAULT.store(site as usize + 1, Ordering::Relaxed);
}

#[inline]
fn fault(site: Site) {
    let armed = site as usize + 1;
    if FAULT.load(Ordering::Relaxed) == armed
        && FAULT
            .compare_exchange(armed, 0, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        panic!(
            "a fault injected at the {} site (og_lanes::inject_fault)",
            site.name()
        );
    }
}

/// Records a panic (a host's panic hook calls it, on whichever thread
/// panicked): the first one is kept, since a lane's panic makes the engine
/// thread panic in turn with the lane's message. Wakes an engine thread
/// waiting for the pool.
pub fn record_panic(message: &str, location: &str, tick: u64) {
    let lane = IN_LANE
        .try_with(|d| d.get() > 0)
        .unwrap_or(false)
        .then(|| Site::from_index(SITE.load(Ordering::Relaxed).wrapping_sub(1)))
        .flatten();
    if let Ok(mut slot) = PANIC.lock()
        && slot.is_none()
    {
        *slot = Some(PanicReport {
            message: message.to_string(),
            location: location.to_string(),
            tick,
            lane,
        });
    }
    PANICKED.store(true, Ordering::Release);
    wake();
}

/// [`record_panic`] from a panic hook's info.
pub fn record_panic_info(info: &std::panic::PanicHookInfo<'_>, tick: u64) {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".to_string());
    let location = info.location().map_or(String::new(), |l| {
        format!("{}:{}:{}", l.file(), l.line(), l.column())
    });
    record_panic(&message, &location, tick);
}

/// The first panic recorded, if any.
pub fn panic_report() -> Option<PanicReport> {
    if !PANICKED.load(Ordering::Acquire) {
        return None;
    }
    PANIC.lock().ok().and_then(|p| p.clone())
}

/// Forgets the recorded panic (a test, or a host that dropped the engine).
pub fn clear_panic() {
    if let Ok(mut p) = PANIC.lock() {
        *p = None;
    }
    PANICKED.store(false, Ordering::Release);
}

fn wake() {
    let _held = GATE.lock().unwrap_or_else(|e| e.into_inner());
    BELL.notify_all();
}

/// How long the waiting thread sleeps between looks when nothing rings.
#[cfg(any(test, all(target_arch = "wasm32", target_feature = "atomics")))]
const WAIT_SLICE: std::time::Duration = std::time::Duration::from_millis(250);

/// Waits until `done` or until a panic is recorded.
#[cfg(any(test, all(target_arch = "wasm32", target_feature = "atomics")))]
fn wait_for(done: &AtomicBool) -> Result<(), PanicReport> {
    let mut held = GATE.lock().unwrap_or_else(|e| e.into_inner());
    loop {
        if done.load(Ordering::Acquire) {
            return Ok(());
        }
        if let Some(p) = panic_report() {
            return Err(p);
        }
        held = match BELL.wait_timeout(held, WAIT_SLICE) {
            Ok((h, _)) => h,
            Err(e) => e.into_inner().0,
        };
    }
}

/// Runs one lane item, marked as inside a lane for [`record_panic`].
#[cfg(any(not(target_arch = "wasm32"), target_feature = "atomics"))]
fn lane<R>(f: impl FnOnce() -> R) -> R {
    struct Leave;
    impl Drop for Leave {
        fn drop(&mut self) {
            let _ = IN_LANE.try_with(|d| d.set(d.get().saturating_sub(1)));
        }
    }
    let _ = IN_LANE.try_with(|d| d.set(d.get() + 1));
    let _leave = Leave;
    f()
}

/// The lane boundary of the lanes wasm: a helper call runs on the pool while
/// the calling thread waits on [`wait_for`], so a lane that panics (and
/// aborts its Worker) wakes it instead of leaving it waiting forever.
#[cfg(any(test, all(target_arch = "wasm32", target_feature = "atomics")))]
mod guard {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    /// Runs `f` on the thread `spawn` hands the job to and returns its
    /// result, or panics with the recorded [`super::PanicReport`] when a
    /// panic is recorded first (on any thread).
    ///
    /// # Safety
    ///
    /// On the panic path `f` may still be running elsewhere with what it
    /// borrows: the caller must make sure this thread's panic never returns
    /// to the frame that lent it (panic = "abort", as in the wasm), or that
    /// `f` borrows nothing (the tests).
    pub unsafe fn run<R: Send>(
        spawn: impl FnOnce(Box<dyn FnOnce() + Send + 'static>),
        f: impl FnOnce() -> R + Send,
    ) -> R {
        let shared: Arc<(Mutex<Option<R>>, AtomicBool)> =
            Arc::new((Mutex::new(None), AtomicBool::new(false)));
        let theirs = shared.clone();
        let job: Box<dyn FnOnce() + Send + '_> = Box::new(move || {
            let r = f();
            *theirs.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(r);
            theirs.1.store(true, Ordering::Release);
            super::wake();
        });
        // SAFETY: the job ends before this returns (wait_for saw `done`), or
        // this thread panics and, by the caller's promise, never returns to
        // the frame `f` borrows from.
        let job: Box<dyn FnOnce() + Send + 'static> = unsafe { std::mem::transmute(job) };
        spawn(job);
        if let Err(p) = super::wait_for(&shared.1) {
            panic!("{}", p.describe());
        }
        let out = shared.0.lock().unwrap_or_else(|e| e.into_inner()).take();
        out.expect("the job ended with its result")
    }
}

/// Switches lanes on or off at run time (the probe's `lanes:<on|off>`).
/// Results never change; only the threads that compute them do.
pub fn set_enabled(on: bool) {
    SWITCH.store(on, Ordering::Relaxed);
}

/// What the engine got (the probe's `dump_lanes`).
pub fn report() -> Report {
    let (threads, pool, why) = imp::pool();
    let switched = SWITCH.load(Ordering::Relaxed);
    let on = threads > 1 && switched && why.is_empty();
    let sites = Site::ALL.map(|s| SiteCount {
        calls: CALLS[s as usize].load(Ordering::Relaxed),
        items: ITEMS[s as usize].load(Ordering::Relaxed),
    });
    Report {
        on,
        threads,
        pool,
        why: if !why.is_empty() {
            why
        } else if threads <= 1 {
            "one thread: the pool has no second thread"
        } else if !switched {
            "switched off (the probe's lanes:off)"
        } else {
            ""
        },
        parallel_calls: sites.iter().map(|s| s.calls).sum(),
        parallel_items: sites.iter().map(|s| s.items).sum(),
        sites,
    }
}

/// How many items a call needs to split, given the pool's threads.
#[derive(Clone, Copy)]
enum Least {
    /// At least this many grains of items (at least two).
    Grains(usize, usize),
    /// A grain for every pool thread.
    GrainEach(usize),
    /// At least this many items.
    Items(usize),
}

/// Whether a call of `len` items splits: lanes on, and at least `from`.
fn parallel(site: Site, len: usize, from: Least) -> bool {
    if !SWITCH.load(Ordering::Relaxed) {
        return false;
    }
    let (threads, _, why) = imp::pool();
    if threads <= 1 || !why.is_empty() {
        return false;
    }
    let least = match from {
        Least::Grains(grain, tasks) => grain.max(1).saturating_mul(tasks.max(2)),
        Least::GrainEach(grain) => grain.max(1).saturating_mul(threads.max(2) as usize),
        Least::Items(n) => n.max(2),
    };
    if len < least {
        return false;
    }
    count(site, len);
    SITE.store(site as usize + 1, Ordering::Relaxed);
    true
}

/// `f` for every item, the results in item order. Runs in parallel when lanes
/// are on and there are at least two `grain`s of items, each task taking at
/// least `grain` of them; serially otherwise. `site` names the caller in the
/// report.
pub fn map<T, R, F>(site: Site, items: &[T], grain: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    let f = move |t: &T| {
        fault(site);
        f(t)
    };
    if parallel(site, items.len(), Least::Grains(grain, 2)) {
        imp::map(items, grain.max(1), f)
    } else {
        items.iter().map(f).collect()
    }
}

/// `f` on every item in place, as [`map`] decides.
pub fn for_each_mut<T, F>(site: Site, items: &mut [T], grain: usize, f: F)
where
    T: Send,
    F: Fn(&mut T) + Sync + Send,
{
    let f = move |t: &mut T| {
        fault(site);
        f(t)
    };
    if parallel(site, items.len(), Least::Grains(grain, 2)) {
        imp::for_each_mut(items, grain.max(1), f)
    } else {
        items.iter_mut().for_each(f)
    }
}

/// `f` on every item in place with its index, as [`map`] decides.
pub fn for_each_indexed<T, F>(site: Site, items: &mut [T], grain: usize, f: F)
where
    T: Send,
    F: Fn(usize, &mut T) + Sync + Send,
{
    let f = move |i: usize, t: &mut T| {
        fault(site);
        f(i, t)
    };
    if parallel(site, items.len(), Least::Grains(grain, 2)) {
        imp::for_each_indexed(items, grain.max(1), f)
    } else {
        items.iter_mut().enumerate().for_each(|(i, t)| f(i, t))
    }
}

/// Runs `f` where a library built on rayon (rapier's `parallel` build, in
/// og-physics natively and in the lanes wasm) finds exactly the threads
/// lanes allow: inside the lanes pool when lanes are on and `items` (what
/// `f` works on: og-physics's bodies) gives every pool thread a `grain`,
/// since such a library splits its work across all of them; otherwise on
/// the calling thread inside a one-thread pool, so the library sees one
/// thread and runs its serial paths without waking the pool. In the
/// single-threaded wasm, on the calling thread. `f` is told whether it runs
/// on the pool, so the helpers it calls can split along. Such a library
/// must give the same results on any number of threads (rapier's does under
/// `enhanced-determinism`; the lanes determinism test checks it).
pub fn install<R: Send>(
    site: Site,
    items: usize,
    grain: usize,
    f: impl FnOnce(bool) -> R + Send,
) -> R {
    install_if(site, items, Least::GrainEach(grain), f)
}

/// [`install`] from `from` items whatever the pool's threads (at least
/// two): for a caller that measured where its library pays for each
/// thread count (og-physics's `pool_bodies`, read with [`threads`]).
pub fn install_from<R: Send>(
    site: Site,
    items: usize,
    from: usize,
    f: impl FnOnce(bool) -> R + Send,
) -> R {
    install_if(site, items, Least::Items(from), f)
}

fn install_if<R: Send>(
    site: Site,
    items: usize,
    from: Least,
    f: impl FnOnce(bool) -> R + Send,
) -> R {
    if parallel(site, items, from) {
        imp::install(move || {
            fault(site);
            f(true)
        })
    } else {
        imp::serial(move || {
            fault(site);
            f(false)
        })
    }
}

/// Runs `f`, work outside the helpers that may reach rayon (a physics
/// query: parry splits some of its traversals with `rayon::join`), on the
/// calling thread inside its one-thread pool, as [`install`] runs a call
/// below its grain: rayon reached from `f` runs inline, never on rayon's
/// global pool and never behind a wait on another thread, so on the web a
/// panic in it traps the engine thread itself (the structured crash of
/// [`panic_report`]) instead of leaving it asleep on rayon's own latch.
/// Called from inside a lane, `f` runs there, on that lane's pool. A fault
/// injected at `site` ([`inject_fault`]) fires here, on the calling thread.
/// Nothing is counted: the call does not go parallel.
pub fn serial<R: Send>(site: Site, f: impl FnOnce() -> R + Send) -> R {
    imp::serial(move || {
        fault(site);
        f()
    })
}

/// The pool's threads lanes would use now: 1 when lanes are off, switched
/// off or there is no pool (what [`report`] says, without its counters).
/// A caller that sizes its grain by the thread count (og-physics's body
/// grain) reads it before calling [`install`].
pub fn threads() -> u32 {
    if !SWITCH.load(Ordering::Relaxed) {
        return 1;
    }
    match imp::pool() {
        (n, _, "") => n.max(1),
        _ => 1,
    }
}

/// The calling thread is running a lane's item: it is a pool thread, and
/// rayon called from it reaches the pool.
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn in_lane() -> bool {
    IN_LANE.try_with(|d| d.get() > 0).unwrap_or(false)
}

/// The one-thread pool of [`install`]'s serial path, natively and in the
/// lanes wasm.
#[cfg(any(not(target_arch = "wasm32"), target_feature = "atomics"))]
mod one {
    thread_local! {
        /// One thread for a rayon library when it runs serially: the calling
        /// thread itself (`use_current_thread`), so the work runs inline.
        /// A pool of its own thread would cost a wake there and back on
        /// every call, 0.1 ms at the median and milliseconds at the tail on
        /// a loaded machine (and the wasm cannot start a thread outside
        /// wasm-bindgen-rayon's pool). Rayon leaks such a pool's registry,
        /// one per thread that ever calls serially; the thread belongs to it
        /// from then on, so rayon called from it outside `install` (a query
        /// parry splits) runs inline too, never on rayon's global pool.
        /// Where the thread already belongs to a pool (a lane calling in),
        /// there is none and the work runs on that pool, with the same
        /// results.
        static SERIAL: Option<rayon::ThreadPool> = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .use_current_thread()
            .build()
            .ok();
    }

    pub fn serial<R: Send>(f: impl FnOnce() -> R + Send) -> R {
        SERIAL.with(|serial| match serial {
            Some(p) => p.install(f),
            None => f(),
        })
    }
}

/// The lanes wasm's way onto the pool: one pool thread runs every job sent
/// to it, so rayon inside a job reaches that pool whichever thread sent it,
/// the engine thread included once it belongs to its one-thread pool
/// (`rayon::spawn` would queue the job there, behind the engine thread's own
/// wait). The relay thread runs the jobs it is sent itself and takes part in
/// the rayon work they start, so it costs the pool no thread.
#[cfg(any(test, all(target_arch = "wasm32", target_feature = "atomics")))]
mod relay {
    use std::sync::mpsc;

    pub type Job = Box<dyn FnOnce() + Send + 'static>;

    pub struct Relay(mpsc::Sender<Job>);

    impl Relay {
        /// Starts the relay on the thread `spawn` hands its loop to.
        pub fn start(spawn: impl FnOnce(Job)) -> Relay {
            let (tx, rx) = mpsc::channel::<Job>();
            spawn(Box::new(move || {
                for job in rx {
                    job();
                }
            }));
            Relay(tx)
        }

        pub fn send(&self, job: Job) {
            self.0
                .send(job)
                .expect("the relay runs as long as the pool does");
        }
    }
}

#[cfg(any(not(target_arch = "wasm32"), target_feature = "atomics"))]
mod par {
    use rayon::prelude::*;

    pub fn map<T: Sync, R: Send, F: Fn(&T) -> R + Sync + Send>(
        items: &[T],
        grain: usize,
        f: F,
    ) -> Vec<R> {
        items
            .par_iter()
            .with_min_len(grain)
            .map(|t| super::lane(|| f(t)))
            .collect()
    }

    pub fn for_each_mut<T: Send, F: Fn(&mut T) + Sync + Send>(items: &mut [T], grain: usize, f: F) {
        items
            .par_iter_mut()
            .with_min_len(grain)
            .for_each(|t| super::lane(|| f(t)))
    }

    pub fn for_each_indexed<T: Send, F: Fn(usize, &mut T) + Sync + Send>(
        items: &mut [T],
        grain: usize,
        f: F,
    ) {
        items
            .par_iter_mut()
            .enumerate()
            .with_min_len(grain)
            .for_each(|(i, t)| super::lane(|| f(i, t)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::sync::OnceLock;

    static POOL: OnceLock<(Option<rayon::ThreadPool>, &'static str)> = OnceLock::new();

    fn pool_of() -> &'static (Option<rayon::ThreadPool>, &'static str) {
        POOL.get_or_init(|| {
            let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
            let threads = match std::env::var("OG_LANES").as_deref().map(str::trim) {
                Ok("off") | Ok("0") | Ok("1") => return (None, "off by OG_LANES"),
                Ok(n) if !n.is_empty() => match n.parse::<usize>() {
                    Ok(n) => n.min(64),
                    Err(_) => return (None, "OG_LANES is not off or a number of threads"),
                },
                _ => cores.min(8),
            };
            if threads <= 1 {
                return (None, "one core");
            }
            match rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .thread_name(|i| format!("og-lane-{i}"))
                .build()
            {
                Ok(p) => (Some(p), ""),
                Err(_) => (None, "the thread pool could not start"),
            }
        })
    }

    pub fn pool() -> (u32, &'static str, &'static str) {
        match pool_of() {
            (Some(p), _) => (p.current_num_threads() as u32, "native", ""),
            (None, why) => (1, "none", why),
        }
    }

    pub use super::one::serial;

    pub fn install<R: Send>(f: impl FnOnce() -> R + Send) -> R {
        match &pool_of().0 {
            Some(p) => p.install(|| super::lane(f)),
            None => serial(f),
        }
    }

    fn pooled<R: Send>(f: impl FnOnce() -> R + Send) -> R {
        match &pool_of().0 {
            Some(p) => p.install(f),
            None => f(),
        }
    }

    pub fn map<T: Sync, R: Send, F: Fn(&T) -> R + Sync + Send>(
        items: &[T],
        grain: usize,
        f: F,
    ) -> Vec<R> {
        pooled(|| super::par::map(items, grain, f))
    }

    pub fn for_each_mut<T: Send, F: Fn(&mut T) + Sync + Send>(items: &mut [T], grain: usize, f: F) {
        pooled(|| super::par::for_each_mut(items, grain, f))
    }

    pub fn for_each_indexed<T: Send, F: Fn(usize, &mut T) + Sync + Send>(
        items: &mut [T],
        grain: usize,
        f: F,
    ) {
        pooled(|| super::par::for_each_indexed(items, grain, f))
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
mod imp {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};
    use wasm_bindgen::prelude::*;

    /// The Worker pool's threads once it is running, 0 before.
    static READY: AtomicU32 = AtomicU32::new(0);
    static RELAY: OnceLock<super::relay::Relay> = OnceLock::new();

    pub use super::one::serial;
    pub use wasm_bindgen_rayon::init_thread_pool;

    /// Called by the engine Worker once `initThreadPool(threads)` resolved:
    /// starts the relay on the pool (`rayon::spawn` from the engine thread,
    /// which belongs to no pool yet) and from then on lanes run there.
    #[wasm_bindgen]
    pub fn og_lanes_ready(threads: u32) {
        if rayon::current_thread_index().is_some() {
            // The engine thread already ran rayon work serially, so
            // `rayon::spawn` would not reach the pool: lanes stay off.
            return;
        }
        RELAY.get_or_init(|| super::relay::Relay::start(rayon::spawn));
        // From now on the engine thread belongs to its one-thread pool, so
        // rayon reached from it outside every helper (a library's own
        // `rayon::join`) runs inline instead of entering the Worker pool
        // and waiting on rayon's latch, which no panic rings: the only way
        // from the engine thread onto the pool is the relay and its latch.
        super::one::serial(|| ());
        READY.store(threads, Ordering::Relaxed);
    }

    pub fn pool() -> (u32, &'static str, &'static str) {
        match READY.load(Ordering::Relaxed) {
            0 => (1, "none", "the Worker pool has not started"),
            n => (n, "web-workers", ""),
        }
    }

    /// On the pool through the relay, the calling thread waiting at the
    /// lane boundary (`super::guard`): a lane's panic aborts its Worker, and
    /// the engine thread then panics with its message instead of waiting
    /// forever. A call from inside a lane (a pool Worker: og-physics's step
    /// reading its contacts back) is already on the pool and runs there.
    fn guarded<R: Send>(f: impl FnOnce() -> R + Send) -> R {
        if super::in_lane() {
            return f();
        }
        let relay = RELAY.get().expect("lanes split only once the relay runs");
        // SAFETY: the wasm aborts on a panic, so the engine thread never
        // returns to the frame `f` borrows from once it panicked.
        unsafe { super::guard::run(|job| relay.send(job), f) }
    }

    pub fn map<T: Sync, R: Send, F: Fn(&T) -> R + Sync + Send>(
        items: &[T],
        grain: usize,
        f: F,
    ) -> Vec<R> {
        guarded(|| super::par::map(items, grain, f))
    }

    pub fn for_each_mut<T: Send, F: Fn(&mut T) + Sync + Send>(items: &mut [T], grain: usize, f: F) {
        guarded(|| super::par::for_each_mut(items, grain, f))
    }

    pub fn for_each_indexed<T: Send, F: Fn(usize, &mut T) + Sync + Send>(
        items: &mut [T],
        grain: usize,
        f: F,
    ) {
        guarded(|| super::par::for_each_indexed(items, grain, f))
    }

    pub fn install<R: Send>(f: impl FnOnce() -> R + Send) -> R {
        guarded(|| super::lane(f))
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
pub use imp::{init_thread_pool, og_lanes_ready};

#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
mod imp {
    pub fn pool() -> (u32, &'static str, &'static str) {
        (
            1,
            "none",
            "the single-threaded wasm: the page is not cross-origin isolated or the build has no lanes variant",
        )
    }

    pub fn map<T, R, F: Fn(&T) -> R>(items: &[T], _grain: usize, f: F) -> Vec<R> {
        items.iter().map(f).collect()
    }

    pub fn for_each_mut<T, F: Fn(&mut T)>(items: &mut [T], _grain: usize, f: F) {
        items.iter_mut().for_each(f)
    }

    pub fn for_each_indexed<T, F: Fn(usize, &mut T)>(items: &mut [T], _grain: usize, f: F) {
        items.iter_mut().enumerate().for_each(|(i, t)| f(i, t))
    }

    pub fn install<R>(f: impl FnOnce() -> R) -> R {
        f()
    }

    pub fn serial<R>(f: impl FnOnce() -> R) -> R {
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// One test owns the process's panic hook and the recorded panic.
    static HOOK: Mutex<()> = Mutex::new(());

    fn hooked<R>(tick: u64, f: impl FnOnce() -> R) -> R {
        let _one = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        clear_panic();
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| record_panic_info(info, tick)));
        let r = f();
        std::panic::set_hook(previous);
        r
    }

    fn message(e: Box<dyn std::any::Any + Send>) -> String {
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    /// The lanes wasm's boundary: a lane that panics and never finishes (an
    /// aborted Worker) wakes the waiting thread, which panics in turn with
    /// the lane's message, its site and the tick, at once rather than never.
    #[test]
    fn a_lane_that_panics_wakes_the_waiting_thread_with_its_message() {
        hooked(42, || {
            let t0 = Instant::now();
            let caught = std::panic::catch_unwind(|| {
                // SAFETY: the job borrows nothing.
                unsafe {
                    guard::run(
                        |job| {
                            std::thread::spawn(job);
                        },
                        || -> u32 {
                            SITE.store(Site::Cull as usize + 1, Ordering::Relaxed);
                            lane(|| panic!("bounds of item 7 are NaN"))
                        },
                    )
                }
            });
            let waited = t0.elapsed();
            let text = message(caught.expect_err("the waiting thread panics"));
            assert!(
                text.starts_with("panic at tick 42 in a lane (cull): bounds of item 7 are NaN"),
                "{text}"
            );
            assert!(waited < Duration::from_secs(2), "woken after {waited:?}");
            let report = panic_report().expect("recorded");
            assert_eq!(report.lane, Some(Site::Cull));
            assert_eq!(report.tick, 42);
            assert_eq!(report.message, "bounds of item 7 are NaN");
            assert!(
                report.location.contains("og-lanes/src/lib.rs"),
                "{report:?}"
            );
        });
    }

    /// A job that ends hands its result back, and nothing is recorded.
    #[test]
    fn a_job_that_ends_returns_its_result() {
        hooked(0, || {
            // SAFETY: the job borrows nothing.
            let v = unsafe {
                guard::run(
                    |job| {
                        std::thread::spawn(job);
                    },
                    || 6 * 7,
                )
            };
            assert_eq!(v, 42);
            assert_eq!(panic_report(), None);
        });
    }

    /// Natively a lane's panic reaches the calling thread through rayon, and
    /// the hook records it as the lane's, with its site; the engine thread's
    /// own panic is recorded without one.
    #[test]
    fn a_native_lane_panic_reaches_the_caller_and_names_its_site() {
        hooked(9, || {
            let items: Vec<u32> = (0..4096).collect();
            let caught = std::panic::catch_unwind(|| {
                map(Site::Extract, &items, 512, |&i| {
                    if i == 3000 {
                        panic!("row 3000 does not decode");
                    }
                    i
                })
            });
            assert_eq!(
                message(caught.expect_err("the caller panics")),
                "row 3000 does not decode"
            );
            let report = panic_report().expect("recorded");
            assert_eq!(report.message, "row 3000 does not decode");
            assert_eq!(report.tick, 9);
            // Serial when this machine gives the pool one thread (OG_LANES=off).
            let on = report_on();
            assert_eq!(report.lane, on.then_some(Site::Extract), "lanes on: {on}");
            clear_panic();
            let _ = std::panic::catch_unwind(|| panic!("a system failed"));
            assert_eq!(panic_report().map(|p| p.lane), Some(None));
        });
    }

    /// An injected fault panics once, at its site only.
    #[test]
    fn an_injected_fault_panics_once_at_its_site() {
        hooked(3, || {
            let items: Vec<u32> = (0..4096).collect();
            inject_fault(Site::Cull);
            assert_eq!(map(Site::Anim, &items, 512, |&i| i).len(), 4096);
            let caught = std::panic::catch_unwind(|| map(Site::Cull, &items, 512, |&i| i));
            assert_eq!(
                message(caught.expect_err("the cull's call panics")),
                "a fault injected at the cull site (og_lanes::inject_fault)"
            );
            assert_eq!(map(Site::Cull, &items, 512, |&i| i).len(), 4096);
            assert_eq!(
                panic_report().map(|p| p.lane),
                Some(report_on().then_some(Site::Cull))
            );
        });
    }

    fn report_on() -> bool {
        report().on
    }

    fn physics_calls() -> SiteCount {
        report().sites[Site::Physics as usize]
    }

    /// Below a grain of items for every pool thread `install` never wakes
    /// the pool: the library finds one thread (its serial paths) on the
    /// calling thread; from there, with lanes on, it finds the pool and the
    /// call is counted with its items. Only this test uses the physics site.
    #[test]
    fn install_splits_from_two_grains_and_runs_inline_below() {
        // The last parallel site names a lane's panic: none of the panic tests' meanwhile.
        let _one = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        let caller = std::thread::current().id();
        let enough = 128 * report().threads.max(2) as usize;
        let before = physics_calls();
        let (threads, thread, pooled) = install(Site::Physics, enough - 1, 128, |pooled| {
            (
                rayon::current_num_threads(),
                std::thread::current().id(),
                pooled,
            )
        });
        assert_eq!((threads, thread, pooled), (1, caller, false));
        assert_eq!(physics_calls(), before);
        // Rayon called on this thread outside `install` stays on it too.
        assert_eq!(rayon::current_num_threads(), 1);

        let on = report_on();
        let (threads, sum, pooled) = install(Site::Physics, enough, 128, |pooled| {
            use rayon::prelude::*;
            let sum: u64 = (0..10_000u64).into_par_iter().map(|i| i * i).sum();
            (rayon::current_num_threads(), sum, pooled)
        });
        assert_eq!(sum, (0..10_000u64).map(|i| i * i).sum::<u64>());
        assert_eq!(pooled, on);
        let after = physics_calls();
        if on {
            assert_eq!(threads as u32, report().threads);
            assert_eq!(after.calls, before.calls + 1);
            assert_eq!(after.items, before.items + enough as u64);
        } else {
            assert_eq!((threads, after), (1, before));
        }

        // `install_from`: from that many items exactly, whatever the threads.
        let at = |items: usize| install_from(Site::Physics, items, 777, |pooled| pooled);
        assert!(!at(776));
        assert_eq!(at(777), on);
        assert_eq!(physics_calls().calls, after.calls + u64::from(on));
    }

    /// `serial`: work outside the helpers runs on the calling thread, rayon
    /// inside it included (both halves of a `join` on the caller, one
    /// thread seen), and leaves the thread in its one-thread pool, so rayon
    /// reached from it later never finds the global pool; a fault injected
    /// at the site fires there, on the calling thread, recorded without a
    /// lane, and a query after it runs normally. On a fresh thread, as the
    /// engine thread is before its first step.
    #[test]
    fn serial_runs_rayon_inline_and_its_fault_is_the_callers() {
        hooked(17, || {
            std::thread::spawn(|| {
                let caller = std::thread::current().id();
                assert_eq!(rayon::current_thread_index(), None, "a fresh thread");
                let (a, b, threads) = serial(Site::Physics, || {
                    let (a, b) = rayon::join(
                        || std::thread::current().id(),
                        || std::thread::current().id(),
                    );
                    (a, b, rayon::current_num_threads())
                });
                assert_eq!((a, b, threads), (caller, caller, 1));
                assert_eq!(rayon::current_num_threads(), 1, "still in its own pool");
                assert_eq!(rayon::current_thread_index(), Some(0));

                inject_fault(Site::Physics);
                let caught = std::panic::catch_unwind(|| serial(Site::Physics, || 1));
                assert_eq!(
                    message(caught.expect_err("the query panics")),
                    "a fault injected at the physics site (og_lanes::inject_fault)"
                );
                let report = panic_report().expect("recorded");
                assert_eq!((report.tick, report.lane), (17, None));
                assert!(
                    report.location.contains("og-lanes/src/lib.rs"),
                    "{report:?}"
                );
                assert_eq!(serial(Site::Physics, || 2), 2, "once");
            })
            .join()
            .unwrap();
        });
    }

    /// `threads` is the pool's when lanes are on and 1 when switched off.
    #[test]
    fn threads_follows_the_switch() {
        // The switch is the process's: no other test reads it meanwhile.
        let _one = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        let r = report();
        assert_eq!(threads(), if r.on { r.threads } else { 1 });
        set_enabled(false);
        assert_eq!(threads(), 1);
        set_enabled(true);
        assert_eq!(threads(), if r.on { r.threads } else { 1 });
    }

    /// The lanes wasm's path onto the pool, on a native pool: a caller that
    /// belongs to its own one-thread pool (as the engine thread does once a
    /// call ran serially) sends a job through the relay, and rayon inside
    /// the job runs on the relay's pool, whose threads all take part.
    #[test]
    fn the_relay_reaches_its_pool_from_a_thread_in_another() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        let relay = relay::Relay::start(|job| pool.spawn(job));
        // The latch also wakes on a recorded panic: none of another test's.
        let _one = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        clear_panic();
        std::thread::spawn(move || {
            assert_eq!(one::serial(rayon::current_num_threads), 1);
            assert_eq!(rayon::current_num_threads(), 1, "in the one-thread pool");
            for _ in 0..3 {
                // SAFETY: nothing here panics, so the job ends before this returns.
                let (threads, workers, sum) = unsafe {
                    guard::run(
                        |job| relay.send(job),
                        || {
                            use rayon::prelude::*;
                            let seen = std::sync::Mutex::new(std::collections::BTreeSet::new());
                            let sum: u64 = (0..4096u32)
                                .into_par_iter()
                                .with_max_len(16)
                                .map(|i| {
                                    seen.lock().unwrap().insert(rayon::current_thread_index());
                                    std::thread::sleep(Duration::from_micros(20));
                                    u64::from(i)
                                })
                                .sum();
                            let workers = seen.into_inner().unwrap().len();
                            (rayon::current_num_threads(), workers, sum)
                        },
                    )
                };
                assert_eq!(threads, 4);
                assert!(workers > 1, "{workers} pool thread(s) took part");
                assert_eq!(sum, 4095 * 4096 / 2);
            }
        })
        .join()
        .unwrap();
    }
}
