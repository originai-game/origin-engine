//! 8-bit 4:2:0 to RGBA8 (sRGB-encoded, as the frame textures are): limited
//! range, BT.709 (what the cook encodes) or BT.601, in 14-bit fixed point,
//! so both hosts' software decoders give the same bytes.

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
    // (Y gain, R from V, G from U, G from V, B from U), each x 2^14.
    let (ky, rv, gu, gv, bu): (i32, i32, i32, i32, i32) = match matrix {
        Matrix::Bt709 => (19077, 29372, 3494, 8731, 34610),
        Matrix::Bt601 => (19077, 26149, 6406, 13320, 33063),
    };
    let clamp = |x: i32| ((x + (1 << 13)) >> 14).clamp(0, 255) as u8;
    for row in 0..height {
        let yr = &y.data[row * y.stride..];
        let ur = &u.data[(row / 2) * u.stride..];
        let vr = &v.data[(row / 2) * v.stride..];
        let o = &mut out[row * width * 4..(row + 1) * width * 4];
        for col in 0..width {
            let l = ky * (i32::from(yr[col]) - 16);
            let cu = i32::from(ur[col / 2]) - 128;
            let cv = i32::from(vr[col / 2]) - 128;
            let p = &mut o[col * 4..col * 4 + 4];
            p[0] = clamp(l + rv * cv);
            p[1] = clamp(l - gu * cu - gv * cv);
            p[2] = clamp(l + bu * cu);
            p[3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_range_bt709_maps_black_white_and_primaries() {
        // 2x2: one chroma sample. Values from BT.709's limited-range table.
        let px = |yv: u8, uv: u8, vv: u8, m| {
            let (yd, ud, vd) = ([yv; 4], [uv], [vv]);
            let mut out = [0u8; 16];
            i420_to_rgba(
                Plane {
                    data: &yd,
                    stride: 2,
                },
                Plane {
                    data: &ud,
                    stride: 1,
                },
                Plane {
                    data: &vd,
                    stride: 1,
                },
                2,
                2,
                m,
                &mut out,
            );
            [out[0], out[1], out[2], out[3]]
        };
        assert_eq!(px(16, 128, 128, Matrix::Bt709), [0, 0, 0, 255]);
        assert_eq!(px(235, 128, 128, Matrix::Bt709), [255, 255, 255, 255]);
        // Pure red in BT.709 limited range: Y 63, U 102, V 240.
        let r = px(63, 102, 240, Matrix::Bt709);
        assert!(r[0] >= 254 && r[1] <= 2 && r[2] <= 2, "{r:?}");
        // The same YUV under BT.601 is a different colour (the matrix matters).
        assert_ne!(px(63, 102, 240, Matrix::Bt601), r);
    }
}
