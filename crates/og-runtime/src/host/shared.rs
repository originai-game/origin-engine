//! The shared buffer: fixed memory the engine and the JavaScript runtime
//! both see, for what crosses the host boundary on every system call and
//! every component read or write. The engine writes an answer there and
//! the runtime reads it before its next engine call (and the other way
//! for a component's bytes), so neither side allocates and nothing is
//! turned into text.
//!
//! A host installs the memory once ([`install`]): the native host a buffer
//! JavaScript owns, the web host one in the module's memory. Without a host
//! (Rust tests) the buffer is the runtime's own.
//!
//! What is in it is only ever valid until the next engine call; nothing of
//! the world lives there, so it is in no snapshot and no hash.

use std::cell::Cell;

/// The buffer's size in bytes: what a host installs, and what the runtime
/// (`SHARED_BYTES` in runtime.ts) views.
pub const SHARED_BYTES: usize = 65536;

thread_local! {
    static SHARED: Cell<*mut u8> = const { Cell::new(std::ptr::null_mut()) };
}

/// Installs the buffer both sides use from now on.
///
/// # Safety
/// `ptr` is valid for reads and writes of [`SHARED_BYTES`] bytes for as
/// long as the thread lives (or until the next `install`), and only touched
/// by the JavaScript the engine runs on this thread.
pub unsafe fn install(ptr: *mut u8) {
    SHARED.with(|s| s.set(ptr));
}

/// The buffer's address, made as the runtime's own when no host installed
/// one (the web host hands this to its loader).
pub fn address() -> *mut u8 {
    SHARED.with(|s| {
        if s.get().is_null() {
            let own = vec![0u64; SHARED_BYTES / 8].into_boxed_slice();
            s.set(Box::leak(own).as_mut_ptr().cast());
        }
        s.get()
    })
}

/// Runs `f` on the buffer. `f` never calls into game code.
pub(crate) fn with<T>(f: impl FnOnce(&mut [u8]) -> T) -> T {
    // SAFETY: `install`'s contract; the engine is single-threaded here and
    // JavaScript does not run while `f` does.
    f(unsafe { std::slice::from_raw_parts_mut(address(), SHARED_BYTES) })
}

/// The buffer's 32-bit words (little-endian, as every target the engine
/// runs on reads them), written and read in order.
pub(crate) struct Words<'a> {
    bytes: &'a mut [u8],
    at: usize,
}

impl<'a> Words<'a> {
    /// From word `start`.
    pub fn at(bytes: &'a mut [u8], start: usize) -> Words<'a> {
        Words {
            bytes,
            at: start * 4,
        }
    }
    /// The next word's index.
    pub fn index(&self) -> usize {
        self.at / 4
    }
    /// Whether `n` more words fit.
    pub fn fits(&self, n: usize) -> bool {
        self.at + n * 4 <= self.bytes.len()
    }
    pub fn put(&mut self, v: u32) {
        self.bytes[self.at..self.at + 4].copy_from_slice(&v.to_le_bytes());
        self.at += 4;
    }
    /// Writes word `index`, leaving the position.
    pub fn set(&mut self, index: usize, v: u32) {
        self.bytes[index * 4..index * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// Writes `v` as the buffer's `index`th 64-bit float.
pub(crate) fn put_f64(bytes: &mut [u8], index: usize, v: f64) {
    bytes[index * 8..index * 8 + 8].copy_from_slice(&v.to_le_bytes());
}
