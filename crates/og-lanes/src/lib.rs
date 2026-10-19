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
//! The probe's `lanes:<on|off>` switches them at run time ([`set_enabled`])
//! and `dump_lanes` reports what the engine got ([`report`]), with the
//! parallel calls counted per [`Site`].

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
    /// og-physics: the rapier step ([`install`]).
    Physics,
}

impl Site {
    /// The number of sites.
    pub const COUNT: usize = 4;
    /// Every site, in report order.
    pub const ALL: [Site; Site::COUNT] = [Site::Anim, Site::Extract, Site::Cull, Site::Physics];

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
    /// counts one call of one item, and only natively, where it hands the
    /// work to the pool; in the wasm it runs it on the calling thread).
    pub parallel_calls: u64,
    /// The items those calls ran.
    pub parallel_items: u64,
    /// The same per [`Site`], indexed by `site as usize` (the totals above
    /// are their sums).
    pub sites: [SiteCount; Site::COUNT],
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

fn parallel(site: Site, len: usize, grain: usize) -> bool {
    if len < grain.max(1) * 2 || !SWITCH.load(Ordering::Relaxed) {
        return false;
    }
    let (threads, _, why) = imp::pool();
    if threads <= 1 || !why.is_empty() {
        return false;
    }
    count(site, len);
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
    if parallel(site, items.len(), grain) {
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
    if parallel(site, items.len(), grain) {
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
    if parallel(site, items.len(), grain) {
        imp::for_each_indexed(items, grain.max(1), f)
    } else {
        items.iter_mut().enumerate().for_each(|(i, t)| f(i, t))
    }
}

/// Runs `f` where a library built on rayon (rapier's `parallel` build, in
/// og-physics natively) finds exactly the threads lanes allow: natively,
/// inside the lanes pool when lanes are on and inside a one-thread pool when
/// they are off; elsewhere on the calling thread. Such a library must give
/// the same results on any number of threads (rapier's does under
/// `enhanced-determinism`; the lanes determinism test checks it).
pub fn install<R: Send>(site: Site, f: impl FnOnce() -> R + Send) -> R {
    imp::install(site, report().on, f)
}

#[cfg(any(not(target_arch = "wasm32"), target_feature = "atomics"))]
mod par {
    use rayon::prelude::*;

    pub fn map<T: Sync, R: Send, F: Fn(&T) -> R + Sync + Send>(
        items: &[T],
        grain: usize,
        f: F,
    ) -> Vec<R> {
        items.par_iter().with_min_len(grain).map(f).collect()
    }

    pub fn for_each_mut<T: Send, F: Fn(&mut T) + Sync + Send>(items: &mut [T], grain: usize, f: F) {
        items.par_iter_mut().with_min_len(grain).for_each(f)
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
            .for_each(|(i, t)| f(i, t))
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

    thread_local! {
        /// One thread for a rayon library when lanes are off: the calling
        /// thread itself (`use_current_thread`), so the work runs inline.
        /// A pool of its own thread would cost a wake there and back on
        /// every call, 0.1 ms at the median and milliseconds at the tail on
        /// a loaded machine. Rayon leaks such a pool's registry, one per
        /// thread that ever calls with lanes off. Where the thread already
        /// belongs to a pool (a lane calling in), there is none and the
        /// work runs on that pool, with the same results.
        static SERIAL: Option<rayon::ThreadPool> = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .use_current_thread()
            .build()
            .ok();
    }

    pub fn install<R: Send>(site: super::Site, on: bool, f: impl FnOnce() -> R + Send) -> R {
        if on && let Some(p) = &pool_of().0 {
            // One call of one item handed to the pool.
            super::count(site, 1);
            return p.install(f);
        }
        SERIAL.with(|serial| match serial {
            Some(p) => p.install(f),
            None => f(),
        })
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
    use std::sync::atomic::{AtomicU32, Ordering};
    use wasm_bindgen::prelude::*;

    /// The Worker pool's threads once it is running, 0 before.
    static READY: AtomicU32 = AtomicU32::new(0);

    pub use wasm_bindgen_rayon::init_thread_pool;

    /// Called by the engine Worker once `initThreadPool(threads)` resolved:
    /// from then on lanes run on the pool.
    #[wasm_bindgen]
    pub fn og_lanes_ready(threads: u32) {
        READY.store(threads, Ordering::Relaxed);
    }

    pub fn pool() -> (u32, &'static str, &'static str) {
        match READY.load(Ordering::Relaxed) {
            0 => (1, "none", "the Worker pool has not started"),
            n => (n, "web-workers", ""),
        }
    }

    pub use super::par::{for_each_indexed, for_each_mut, map};

    pub fn install<R: Send>(_site: super::Site, _on: bool, f: impl FnOnce() -> R + Send) -> R {
        f()
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

    pub fn install<R>(_site: super::Site, _on: bool, f: impl FnOnce() -> R) -> R {
        f()
    }
}
