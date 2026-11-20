//! Whether a pipeline is made yet, for the passes whose output the
//! renderer keeps.
//!
//! On the web every pipeline a frame makes is asked of the browser
//! asynchronously ([`set_pipeline_hooks`]'s `make` span around
//! [`crate::Renderer::render`]; packages/engine/src/web/pipelines.ts): a cold
//! driver compiles one for seconds, and synchronous ones in one GPU-process
//! task past Chrome's 15 s watchdog lose the device. A pass drawn again every
//! frame skips a pipeline still compiling (the host's frame span), but one
//! whose output is kept would keep that gap, or make the pipeline
//! synchronously at its use. So each such pass waits:
//!
//! - one-shot work (a scatter chunk's placement, a grass field's blades, the
//!   environment's bake, the live sky's step, the clouds' shadow bake) takes
//!   the [`Tickets`] of the pipelines it made and, while they are not
//!   [`Tickets::ready`], leaves its work (and every draw that reads it) for a
//!   later frame;
//! - the compute before the frame's passes waits the same way, its readers
//!   with it: the wind field steps nothing (and catches up its ticks
//!   later), a terrain whose corner cache is not filled is left out, the
//!   scattered copies the GPU culls are left out, the clouds' noise is
//!   drawn by a later frame (and their shadow baked after it), and the
//!   frame culls its instances on the CPU;
//! - the shadow layers, which the static cache keeps, are encoded in a frame
//!   span, and while any pipeline still [`compiling`] the cache forgets its
//!   layers, so a caster skipped meanwhile is drawn again.
//!
//! Either way the host still compiles, so the play gate draws the first
//! frame again until it is whole. Natively, and on a page that makes its
//! pipelines at once, every ticket is ready and nothing compiles.

use std::cell::Cell;

/// The host's `(count, ready, compiling)`: the tickets handed out so far
/// (each a pipeline asked for asynchronously, numbered from 1), whether a
/// ticket's pipeline is made, and whether any is still compiling.
pub type ReadyHooks = (fn() -> u32, fn(u32) -> bool, fn() -> bool);

thread_local! {
    static READY: Cell<Option<ReadyHooks>> = const { Cell::new(None) };
    /// The host's span around the renderer's frame: every pipeline made in
    /// it is asked for asynchronously.
    static MAKE: Cell<Option<fn(bool)>> = const { Cell::new(None) };
    /// Whether the frame drawing now (or drawn last) left kept work for a
    /// later one ([`hold`]).
    static HELD: Cell<bool> = const { Cell::new(false) };
}

/// The web host's pipeline hooks: `make(on)` around each frame the renderer
/// draws, and `ready` for the [`Tickets`] of what it made; `None` (the
/// default) calls nothing, every ticket is ready and nothing compiles.
pub fn set_pipeline_hooks(make: Option<fn(bool)>, ready: Option<ReadyHooks>) {
    MAKE.with(|c| c.set(make));
    READY.with(|c| c.set(ready));
}

fn count() -> u32 {
    READY.with(|c| c.get()).map_or(0, |(count, ..)| count())
}

/// Whether the host has hooks: off, every ticket is ready, and nothing is
/// made only to ask whether it is compiled.
pub(crate) fn hooked() -> bool {
    READY.with(|c| c.get()).is_some()
}

/// A pass whose output is kept left its work for a later frame: the web
/// host draws the first frame again until none did ([`pipelines_held`]).
pub(crate) fn hold() {
    HELD.with(|h| h.set(true));
}

/// Whether the last frame left kept work for a later one, its pipelines
/// still compiling (the web host's play gate).
pub fn pipelines_held() -> bool {
    HELD.with(|h| h.get())
}

/// The pipelines compiled off the frame so far, and how many of them are
/// made: on the web those asked of the browser asynchronously (the host's
/// tickets) and those of them it has made or refused; natively the jobs
/// handed to the compile workers (deferred.rs) and those finished. Both
/// only go up, and are equal when nothing compiles: a loading screen's
/// count (og.loading.status).
pub fn pipeline_counts() -> (u32, u32) {
    if let Some((count, ready, _)) = READY.with(|c| c.get()) {
        let asked = count();
        // A ticket once ready stays ready.
        return (asked, (1..=asked).filter(|&t| ready(t)).count() as u32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    return crate::deferred::worker_counts();
    #[cfg(target_arch = "wasm32")]
    (0, 0)
}

/// Whether the host still compiles any pipeline it was asked for.
pub(crate) fn compiling() -> bool {
    READY
        .with(|c| c.get())
        .is_some_and(|(.., compiling)| compiling())
}

/// The pipelines the host was asked for asynchronously while a value was
/// made ([`Tickets::during`]); none when it made them at once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tickets {
    /// After the tickets handed out before, up to those handed out after.
    from: u32,
    to: u32,
}

impl Tickets {
    /// `make()`, with the tickets of the pipelines it made.
    pub(crate) fn during<T>(make: impl FnOnce() -> T) -> (T, Tickets) {
        let from = count();
        let made = make();
        (made, Tickets { from, to: count() })
    }

    /// The tickets from here on, for a constructor that makes its
    /// pipelines among its other resources: [`Tickets::to_here`] once
    /// they are made.
    pub(crate) fn from_here() -> Tickets {
        let n = count();
        Tickets { from: n, to: n }
    }

    /// The tickets handed out since [`Tickets::from_here`].
    pub(crate) fn to_here(self) -> Tickets {
        Tickets {
            from: self.from,
            to: count(),
        }
    }

    /// Whether each of its pipelines is made (or refused, which its use
    /// reports).
    pub(crate) fn ready(self) -> bool {
        if self.from >= self.to {
            return true;
        }
        READY
            .with(|c| c.get())
            .is_none_or(|(_, ready, _)| (self.from + 1..=self.to).all(ready))
    }
}

/// The host's make span, open while this lives.
pub(crate) struct MakeSpan(Option<fn(bool)>);

impl MakeSpan {
    /// Opened once a frame, before anything is held.
    pub(crate) fn open() -> MakeSpan {
        HELD.with(|h| h.set(false));
        let hook = MAKE.with(|c| c.get());
        if let Some(hook) = hook {
            hook(true);
        }
        MakeSpan(hook)
    }
}

impl Drop for MakeSpan {
    fn drop(&mut self) {
        if let Some(hook) = self.0 {
            hook(false);
        }
    }
}
