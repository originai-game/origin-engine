//! The AV1 decoder: rav1d (BSD-2-Clause) through its dav1d C ABI, one
//! temporal unit in, one RGBA8 picture out.

use crate::yuv::{self, Matrix, Plane};
use rav1d::include::dav1d::data::Dav1dData;
use rav1d::include::dav1d::dav1d::{Dav1dContext, Dav1dSettings};
use rav1d::include::dav1d::headers::{DAV1D_MC_BT470BG, DAV1D_MC_BT601, DAV1D_PIXEL_LAYOUT_I420};
use rav1d::include::dav1d::picture::Dav1dPicture;
use rav1d::src::lib::{
    dav1d_close, dav1d_data_create, dav1d_data_unref, dav1d_default_settings, dav1d_flush,
    dav1d_get_picture, dav1d_open, dav1d_picture_unref, dav1d_send_data,
};
use std::mem::MaybeUninit;
use std::ptr::NonNull;

/// A decoded picture as RGBA8 rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    /// Shared with the texture it is written to, not copied.
    pub rgba: std::sync::Arc<Vec<u8>>,
}

/// A decoded picture as packed 8-bit 4:2:0 planes: Y (`width` x
/// `height`), then U and V (each half the size, rounded up), BT.709
/// limited range as the cook encodes it. The web fallback hands these to
/// the browser (a `VideoFrame` of format I420), which converts on the GPU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planes {
    pub width: u32,
    pub height: u32,
    pub i420: Vec<u8>,
}

/// One rav1d context.
pub struct Decoder {
    ctx: Option<Dav1dContext>,
}

// SAFETY: a dav1d context is used from one thread at a time (it is owned
// by `&mut self` here); rav1d's own threads synchronise inside it.
unsafe impl Send for Decoder {}

/// rav1d's "send or take again" answer (dav1d's `DAV1D_ERR(EAGAIN)`).
fn again(r: i32) -> bool {
    r == -libc::EAGAIN
}

impl Decoder {
    /// A decoder on `threads` threads (0: one per core, at most 8), with
    /// one frame of delay, so each unit sent gives its picture back at once.
    pub fn new(threads: u32) -> Result<Decoder, String> {
        let mut s = MaybeUninit::<Dav1dSettings>::uninit();
        // SAFETY: `s` is valid to write; dav1d_default_settings fills it.
        let mut s = unsafe {
            dav1d_default_settings(NonNull::new_unchecked(s.as_mut_ptr()));
            s.assume_init()
        };
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get() as u32);
        s.n_threads = if threads == 0 { cores.min(8) } else { threads } as i32;
        s.max_frame_delay = 1;
        let mut ctx: Option<Dav1dContext> = None;
        // SAFETY: both pointers are valid for the call.
        let r = unsafe { dav1d_open(Some(NonNull::from(&mut ctx)), Some(NonNull::from(&mut s))) };
        if r.0 != 0 || ctx.is_none() {
            return Err(format!("rav1d refused to open a decoder ({})", r.0));
        }
        Ok(Decoder { ctx })
    }

    /// Forgets every frame it holds (before decoding from another key frame).
    pub fn flush(&mut self) {
        if let Some(c) = self.ctx {
            // SAFETY: `c` is open.
            unsafe { dav1d_flush(c) };
        }
    }

    /// Decodes one temporal unit: the picture it shows as RGBA8, or `None`
    /// for a unit that shows none.
    pub fn decode(&mut self, unit: &[u8]) -> Result<Option<Picture>, String> {
        self.decode_with(unit, convert)
    }

    /// [`Decoder::decode`] as packed I420 planes, unconverted.
    pub fn decode_i420(&mut self, unit: &[u8]) -> Result<Option<Planes>, String> {
        self.decode_with(unit, pack)
    }

    fn decode_with<T>(
        &mut self,
        unit: &[u8],
        out: fn(&Dav1dPicture) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        let c = self.ctx.ok_or("the decoder is closed")?;
        if unit.is_empty() {
            return Ok(None);
        }
        let mut data = MaybeUninit::<Dav1dData>::zeroed();
        // SAFETY: `data` is valid to write; the buffer it answers holds
        // `unit.len()` bytes, written before it is sent.
        let mut data = unsafe {
            let p = dav1d_data_create(Some(NonNull::new_unchecked(data.as_mut_ptr())), unit.len());
            if p.is_null() {
                return Err("rav1d could not allocate a unit".into());
            }
            std::ptr::copy_nonoverlapping(unit.as_ptr(), p, unit.len());
            data.assume_init()
        };
        let mut shown = None;
        loop {
            // SAFETY: `c` is open and `data` a valid Dav1dData.
            let r = unsafe { dav1d_send_data(Some(c), Some(NonNull::from(&mut data))) };
            if r.0 != 0 && !again(r.0) {
                // SAFETY: `data` is ours to release.
                unsafe { dav1d_data_unref(Some(NonNull::from(&mut data))) };
                return Err(format!("rav1d refused a unit ({})", r.0));
            }
            if let Some(p) = self.picture(c, out)? {
                shown = Some(p);
            }
            if data.sz == 0 {
                break;
            }
        }
        Ok(shown)
    }

    fn picture<T>(
        &mut self,
        c: Dav1dContext,
        out: fn(&Dav1dPicture) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        let mut pic = MaybeUninit::<Dav1dPicture>::zeroed();
        // SAFETY: `c` is open; `pic` is valid to write.
        let r =
            unsafe { dav1d_get_picture(Some(c), Some(NonNull::new_unchecked(pic.as_mut_ptr()))) };
        if again(r.0) {
            return Ok(None);
        }
        if r.0 != 0 {
            return Err(format!("rav1d could not decode a picture ({})", r.0));
        }
        // SAFETY: get_picture succeeded and wrote `pic`.
        let mut pic = unsafe { pic.assume_init() };
        let result = out(&pic);
        // SAFETY: `pic` came from get_picture.
        unsafe { dav1d_picture_unref(Some(NonNull::from(&mut pic))) };
        result.map(Some)
    }
}

/// A picture's three planes and their strides, checked to be 8-bit 4:2:0.
fn planes(pic: &Dav1dPicture) -> Result<[Plane<'_>; 3], String> {
    let (w, h) = (pic.p.w as usize, pic.p.h as usize);
    if pic.p.bpc != 8 || pic.p.layout != DAV1D_PIXEL_LAYOUT_I420 {
        return Err(format!(
            "a {}-bit picture of layout {} (the engine plays 8-bit 4:2:0; cook it again)",
            pic.p.bpc, pic.p.layout
        ));
    }
    let (ys, uvs) = (pic.stride[0] as usize, pic.stride[1] as usize);
    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
    let plane = |i: usize, stride: usize, rows: usize, cols: usize| -> Result<Plane<'_>, String> {
        let p = pic.data[i].ok_or("a picture without its planes")?;
        // SAFETY: rav1d allocates `stride` bytes for each of a plane's rows,
        // alive as long as the picture.
        let data = unsafe {
            std::slice::from_raw_parts(p.as_ptr() as *const u8, stride * (rows - 1) + cols)
        };
        Ok(Plane { data, stride })
    };
    Ok([
        plane(0, ys, h, w)?,
        plane(1, uvs, ch, cw)?,
        plane(2, uvs, ch, cw)?,
    ])
}

/// Packs a picture's planes row after row, without their strides' padding.
fn pack(pic: &Dav1dPicture) -> Result<Planes, String> {
    let [y, u, v] = planes(pic)?;
    let (w, h) = (pic.p.w as usize, pic.p.h as usize);
    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
    let mut i420 = Vec::with_capacity(w * h + 2 * cw * ch);
    for (p, cols, rows) in [(y, w, h), (u, cw, ch), (v, cw, ch)] {
        for r in 0..rows {
            i420.extend_from_slice(&p.data[r * p.stride..r * p.stride + cols]);
        }
    }
    Ok(Planes {
        width: w as u32,
        height: h as u32,
        i420,
    })
}

fn convert(pic: &Dav1dPicture) -> Result<Picture, String> {
    let (w, h) = (pic.p.w as usize, pic.p.h as usize);
    let [y, u, v] = planes(pic)?;
    let matrix = pic
        .seq_hdr
        // SAFETY: a picture's sequence header lives as long as the picture.
        .map(|s| unsafe { s.as_ref().mtrx })
        .map_or(Matrix::Bt709, |m| {
            if m == DAV1D_MC_BT601 || m == DAV1D_MC_BT470BG {
                Matrix::Bt601
            } else {
                Matrix::Bt709
            }
        });
    let mut rgba = vec![0u8; w * h * 4];
    yuv::i420_to_rgba(y, u, v, w, h, matrix, &mut rgba);
    Ok(Picture {
        width: w as u32,
        height: h as u32,
        rgba: std::sync::Arc::new(rgba),
    })
}

impl Drop for Decoder {
    fn drop(&mut self) {
        if self.ctx.is_some() {
            // SAFETY: the context is open and closed once.
            unsafe { dav1d_close(Some(NonNull::from(&mut self.ctx))) };
        }
    }
}
