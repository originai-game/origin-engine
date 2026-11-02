//! The web fallback's AV1 decoder (G3-2): og-video's rav1d as a WASI
//! reactor with a C ABI the decoder Worker calls
//! (`packages/engine/src/web/video-decoder.ts`). One thread: a page that is
//! not cross-origin isolated has no shared memory for more.
//!
//! The Worker allocates a unit in the module's memory (`og_av1_alloc`),
//! decodes it (`og_av1_decode`), and copies the picture out of
//! `og_av1_picture` (packed I420: Y, then U and V at half size, BT.709
//! limited range), which it wraps in a `VideoFrame` the browser converts
//! and copies to the stream texture on the GPU.

use og_video::decode::{Decoder, Planes};

/// The module's decoders, by handle (index + 1).
struct State {
    decoders: Vec<Option<(Decoder, Option<Planes>)>>,
}

static mut STATE: State = State {
    decoders: Vec::new(),
};

#[allow(static_mut_refs)]
fn state() -> &'static mut State {
    // SAFETY: the module runs on one thread (the decoder Worker) and no
    // export re-enters another.
    unsafe { &mut STATE }
}

fn slot(h: u32) -> Option<&'static mut (Decoder, Option<Planes>)> {
    state()
        .decoders
        .get_mut((h as usize).checked_sub(1)?)
        .and_then(Option::as_mut)
}

/// `len` bytes for a unit, freed with [`og_av1_free`].
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_alloc(len: usize) -> *mut u8 {
    let mut v = vec![0u8; len].into_boxed_slice();
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Frees what [`og_av1_alloc`] answered.
///
/// # Safety
/// `ptr` and `len` are an allocation of [`og_av1_alloc`], freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_av1_free(ptr: *mut u8, len: usize) {
    // SAFETY: the caller's contract.
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) });
}

/// A new decoder's handle, 0 when rav1d refuses.
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_open() -> u32 {
    match Decoder::new(1) {
        Ok(d) => {
            let s = state();
            s.decoders.push(Some((d, None)));
            s.decoders.len() as u32
        }
        Err(_) => 0,
    }
}

/// Closes a decoder.
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_close(h: u32) {
    if let Some(d) = state().decoders.get_mut((h as usize).wrapping_sub(1)) {
        *d = None;
    }
}

/// Forgets the frames a decoder holds (before a key frame elsewhere).
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_flush(h: u32) {
    if let Some((d, _)) = slot(h) {
        d.flush();
    }
}

/// Decodes one temporal unit: 1 when it shows a picture (read it with
/// [`og_av1_picture`]), 0 when it shows none, -1 on an error.
///
/// # Safety
/// `ptr` and `len` are bytes of this module's memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_av1_decode(h: u32, ptr: *const u8, len: usize) -> i32 {
    let Some((d, out)) = slot(h) else { return -1 };
    // SAFETY: the caller's contract.
    let unit = unsafe { std::slice::from_raw_parts(ptr, len) };
    match d.decode_i420(unit) {
        Ok(Some(p)) => {
            *out = Some(p);
            1
        }
        Ok(None) => 0,
        Err(_) => -1,
    }
}

/// The last picture's packed I420 bytes (null before one).
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_picture(h: u32) -> *const u8 {
    slot(h)
        .and_then(|(_, p)| p.as_ref())
        .map_or(std::ptr::null(), |p| p.i420.as_ptr())
}

/// The last picture's width.
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_width(h: u32) -> u32 {
    slot(h).and_then(|(_, p)| p.as_ref()).map_or(0, |p| p.width)
}

/// The last picture's height.
#[unsafe(no_mangle)]
pub extern "C" fn og_av1_height(h: u32) -> u32 {
    slot(h)
        .and_then(|(_, p)| p.as_ref())
        .map_or(0, |p| p.height)
}
