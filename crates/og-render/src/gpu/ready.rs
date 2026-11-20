//! Whether a pipeline is made yet, for the passes whose output the
//! renderer keeps.
//!
//! On the web every pipeline a frame makes is asked of the browser
//! asynchronously ([`set_pipeline_hooks`]'s `make` span around
//! [`crate::Renderer::render`]; packages/engine/src/web/pipelines.ts): a cold
//! driver compiles one for seconds, and synchronous ones in one GPU-process
//! task past Chrome's 15 s watchdog lose the device. A pass drawn again every
//! frame skips a pipeline still compiling (the host's frame span), but one
//! whose output is kept (a scatter chunk placed once, a grass field's
//! blades) would keep that gap, or make the pipeline synchronously at its
//! use. Such a pass takes the [`Ticket`] of each pipeline it makes and,
//! while one is not [`Ticket::ready`], leaves its work (and every draw that
//! reads it) for a later frame. Natively, and on a page that makes its
//! pipelines at once, every ticket is ready.

use std::cell::Cell;

/// The host's `(last, ready)`: the ticket of the pipeline made last, and
/// whether a ticket's pipeline is made.
pub type ReadyHooks = (fn() -> u32, fn(u32) -> bool);

thread_local! {
    /// The host's `(last, ready)`: the ticket of the pipeline made last, and
    /// whether a ticket's pipeline is made.
    static READY: Cell<Option<ReadyHooks>> = const { Cell::new(None) };
    /// The host's span around the renderer's frame: every pipeline made in
    /// it is asked for asynchronously.
    static MAKE: Cell<Option<fn(bool)>> = const { Cell::new(None) };
}

/// The web host's pipeline hooks: `make(on)` around each frame the renderer
/// draws, and `ready = (last, ready)` for the [`Ticket`]s of what it made;
/// `None` (the default) calls nothing and every ticket is ready.
pub fn set_pipeline_hooks(make: Option<fn(bool)>, ready: Option<ReadyHooks>) {
    MAKE.with(|c| c.set(make));
    READY.with(|c| c.set(ready));
}

/// A pipeline the host may still compile: taken right after making it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Ticket(u32);

impl Ticket {
    /// The ticket of the pipeline made last on this thread.
    pub(crate) fn last() -> Ticket {
        Ticket(READY.with(|c| c.get()).map_or(0, |(last, _)| last()))
    }

    /// Whether its pipeline is made (or refused, which its use reports).
    pub(crate) fn ready(self) -> bool {
        self.0 == 0
            || READY
                .with(|c| c.get())
                .is_none_or(|(_, ready)| ready(self.0))
    }
}

/// Makes `n` pipelines with `make`, each with its ticket.
pub(crate) fn tracked<T, const N: usize>(
    mut make: impl FnMut(usize) -> T,
) -> ([T; N], [Ticket; N]) {
    let mut tickets = [Ticket::default(); N];
    let made = std::array::from_fn(|i| {
        let p = make(i);
        tickets[i] = Ticket::last();
        p
    });
    (made, tickets)
}

/// Whether every one of `tickets` is ready.
pub(crate) fn all(tickets: &[Ticket]) -> bool {
    tickets.iter().all(|t| t.ready())
}

/// The host's make span, open while this lives.
pub(crate) struct MakeSpan(Option<fn(bool)>);

impl MakeSpan {
    pub(crate) fn open() -> MakeSpan {
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
