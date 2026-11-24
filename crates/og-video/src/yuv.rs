//! 8-bit 4:2:0 to RGBA8 (sRGB-encoded, as the frame textures are): limited
//! range, BT.709 (what the cook encodes) or BT.601, in 14-bit fixed point,
//! so both hosts' software decoders give the same bytes.
//!
//! It runs on a stream's decode thread for every frame shown, and an
//! executor's tick waits for that frame (`stream::Player::take`), so its
//! speed is how fast footage plays headless: rows go by pairs (each chroma
//! sample's terms computed once for its four pixels), the loops are
//! iterators the compiler vectorises, and a large picture's row pairs are
//! split across threads.

/// The colour matrix of a picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matrix {
    Bt709,
    Bt601,
}

/// One plane: its bytes and the bytes from one row to the next.
#[derive(Clone, Copy)]
pub struct Plane<'a> {
    pub data: &'a [u8],
    pub stride: usize,
}

/// (Y gain, R from V, G from U, G from V, B from U), each x 2^14.
#[derive(Clone, Copy)]
struct Gains {
    ky: i32,
    rv: i32,
    gu: i32,
    gv: i32,
    bu: i32,
}

/// Pixels below which a picture is converted on the calling thread alone
/// (a thread's start costs more than converting this many).
const SPLIT_FROM: usize = 256 * 256;

/// Threads a large picture's rows are split across, at most.
const THREADS: usize = 4;

/// Writes `width` x `height` RGBA8 into `out` (`width * height * 4` bytes)
/// from Y at full size and U and V at half size (rounded up).
pub fn i420_to_rgba(
    y: Plane<'_>,
    u: Plane<'_>,
    v: Plane<'_>,
    width: usize,
    height: usize,
    matrix: Matrix,
    out: &mut [u8],
) {
    let g = match matrix {
        Matrix::Bt709 => Gains {
            ky: 19077,
            rv: 29372,
            gu: 3494,
            gv: 8731,
            bu: 34610,
        },
        Matrix::Bt601 => Gains {
            ky: 19077,
            rv: 26149,
            gu: 6406,
            gv: 13320,
            bu: 33063,
        },
    };
    let out = &mut out[..width * height * 4];
    if width == 0 || height == 0 {
        return;
    }
    let pair = width * 8;
    let pairs = height.div_ceil(2);
    let threads = if cfg!(target_arch = "wasm32") || width * height < SPLIT_FROM {
        1
    } else {
        std::thread::available_parallelism().map_or(1, |n| n.get().min(THREADS))
    };
    let band = pairs.div_ceil(threads);
    let convert = |first: usize, rows: &mut [u8]| {
        for (i, o) in rows.chunks_mut(pair).enumerate() {
            let c = first + i;
            let ur = &u.data[c * u.stride..][..width.div_ceil(2)];
            let vr = &v.data[c * v.stride..][..width.div_ceil(2)];
            for (j, o) in o.chunks_exact_mut(width * 4).enumerate() {
                row(&y.data[(c * 2 + j) * y.stride..][..width], ur, vr, o, g);
            }
        }
    };
    if threads == 1 {
        convert(0, out);
        return;
    }
    std::thread::scope(|s| {
        let mut bands = out.chunks_mut(band * pair).enumerate();
        let first = bands.next();
        for (b, rows) in bands {
            s.spawn(move || convert(b * band, rows));
        }
        if let Some((_, rows)) = first {
            convert(0, rows);
        }
    });
}

/// Writes the alpha of `width` x `height` RGBA8 `out` from a packed
/// picture's lower half: `y` the luma rows below the colour (the matte,
/// limited range: 16 transparent, 235 opaque), as the cook packs a stream
/// with `alpha true`.
pub fn alpha_from_luma(y: Plane<'_>, width: usize, height: usize, out: &mut [u8]) {
    for (r, o) in out[..width * height * 4]
        .chunks_exact_mut(width * 4)
        .enumerate()
    {
        let row = &y.data[r * y.stride..][..width];
        for (px, &l) in o.as_chunks_mut::<4>().0.iter_mut().zip(row) {
            px[3] = ((i32::from(l) - 16) * 255 + 109)
                .div_euclid(219)
                .clamp(0, 255) as u8;
        }
    }
}

/// One row: `y` its luma, `u` and `v` its chroma row (half as many,
/// rounded up), `o` its RGBA.
fn row(y: &[u8], u: &[u8], v: &[u8], o: &mut [u8], g: Gains) {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        // SAFETY: the CPU has AVX2.
        return unsafe { row_avx2(y, u, v, o, g) };
    }
    row_body(y, u, v, o, g)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn row_avx2(y: &[u8], u: &[u8], v: &[u8], o: &mut [u8], g: Gains) {
    row_body(y, u, v, o, g)
}

#[inline(always)]
fn row_body(y: &[u8], u: &[u8], v: &[u8], o: &mut [u8], g: Gains) {
    // The rounding of the 14-bit shift folded into the luma term.
    let luma = |y: u8| g.ky * (i32::from(y) - 16) + (1 << 13);
    let chroma = |u: u8, v: u8| {
        let (cu, cv) = (i32::from(u) - 128, i32::from(v) - 128);
        (g.rv * cv, -(g.gu * cu) - g.gv * cv, g.bu * cu)
    };
    let c = |x: i32| (x >> 14).clamp(0, 255) as u8;
    let px = |l: i32, (r, gr, b): (i32, i32, i32)| [c(l + r), c(l + gr), c(l + b), 255];
    let (quads, last) = o.as_chunks_mut::<8>();
    let (pairs, odd) = y.as_chunks::<2>();
    for (((p, &[y0, y1]), &u1), &v1) in quads.iter_mut().zip(pairs).zip(u).zip(v) {
        let k = chroma(u1, v1);
        let ([r0, g0, b0, a0], [r1, g1, b1, a1]) = (px(luma(y0), k), px(luma(y1), k));
        *p = [r0, g0, b0, a0, r1, g1, b1, a1];
    }
    if let (&[y1], Some(&u1), Some(&v1)) = (odd, u.last(), v.last()) {
        last.copy_from_slice(&px(luma(y1), chroma(u1, v1)));
    }
}
