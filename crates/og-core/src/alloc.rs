//! The engine's heap, counted: a host installs [`Counting`] as its global
//! allocator (`#[global_allocator]` in og-host-web and og-host-node), and
//! the probe's `memory` member reads [`heap`] on either host: the bytes
//! live now, the most ever live at once and the allocations made.

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);

std::thread_local! {
    /// This thread's allocations: a span on one thread (a UI frame) counts
    /// its own, not the render or audio threads' beside it.
    static MINE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn counted() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    // A thread being torn down has no counter left; its count is moot.
    let _ = MINE.try_with(|c| c.set(c.get() + 1));
}

/// A global allocator that counts what `A` hands out ([`heap`]).
pub struct Counting<A>(pub A);

fn grew(by: usize) {
    let live = LIVE.fetch_add(by, Ordering::Relaxed) + by;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

// SAFETY: every call is forwarded to `A` unchanged; the counters are atomics
// that never allocate.
unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract, passed on.
        let p = unsafe { self.0.alloc(layout) };
        if !p.is_null() {
            counted();
            grew(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract, passed on.
        let p = unsafe { self.0.alloc_zeroed(layout) };
        if !p.is_null() {
            counted();
            grew(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller's contract, passed on.
        unsafe { self.0.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller's contract, passed on.
        let p = unsafe { self.0.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                grew(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        p
    }
}

/// What [`Counting`] counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heap {
    /// Bytes allocated and not freed.
    pub live: usize,
    /// The most bytes live at once.
    pub peak: usize,
    /// Allocations made (reallocations not counted).
    pub allocations: usize,
}

/// The heap as [`Counting`] counted it; `None` when no counting allocator
/// is installed (nothing was counted).
pub fn heap() -> Option<Heap> {
    let allocations = COUNT.load(Ordering::Relaxed);
    (allocations > 0).then(|| Heap {
        live: LIVE.load(Ordering::Relaxed),
        peak: PEAK.load(Ordering::Relaxed),
        allocations,
    })
}

/// The allocations [`Counting`] made on this thread so far.
pub fn thread_allocations() -> usize {
    MINE.try_with(std::cell::Cell::get).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::System;

    #[test]
    fn counts_what_it_hands_out() {
        let a = Counting(System);
        let before = heap().unwrap_or_default();
        let layout = Layout::from_size_align(1000, 8).unwrap();
        // SAFETY: a valid layout; the pointer is freed below with it.
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        let during = heap().expect("counted");
        assert_eq!(during.allocations, before.allocations + 1);
        let mine = thread_allocations();
        assert!(mine >= 1, "this thread's allocation counted");
        let elsewhere = std::thread::spawn(move || {
            let b = Counting(System);
            // SAFETY: a valid layout; freed at once with it.
            unsafe { b.dealloc(b.alloc(layout), layout) };
        });
        elsewhere.join().unwrap();
        assert_eq!(
            thread_allocations(),
            mine,
            "another thread's allocation is its own"
        );
        assert!(during.peak >= during.live);
        // SAFETY: grown from the same layout, then freed with its new size.
        let q = unsafe { a.realloc(p, layout, 3000) };
        let grown = heap().unwrap();
        assert!(grown.peak >= grown.live);
        // SAFETY: allocated above with this size and alignment.
        unsafe { a.dealloc(q, Layout::from_size_align(3000, 8).unwrap()) };
        let after = heap().unwrap();
        assert_eq!(after.live, before.live);
        assert!(after.peak >= before.live + 3000);
    }
}
