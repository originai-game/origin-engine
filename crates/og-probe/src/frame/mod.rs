//! Frames as pixels: the RGBA8 image the probe's captures, marks, key
//! frames and perceptual diffs work on, its PNG reading and writing, and the
//! few operations an agent's view of a frame needs: fitting it to a long
//! side, cropping a region at full resolution, enlarging a crop, and drawing
//! numbered boxes (Set-of-Mark). Nothing here reads a clock or a random
//! number: the same input gives the same bytes on every host.

pub mod flip;

/// The long side a capture for a model is fitted to unless the caller asks
/// otherwise (the agents guide, `og api show captures`).
pub const MODEL_LONG_SIDE: u32 = 1280;

/// A decoded RGBA8 image, rows top to bottom.
#[derive(Clone, Debug, PartialEq)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// A box in an image's pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    /// The rectangle `x, y, width, height` (in any unit, scaled by `scale`
    /// into pixels) clipped to a `width`×`height` image; `None` when nothing
    /// of it is inside.
    pub fn clipped(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        scale: f32,
        width: u32,
        height: u32,
    ) -> Option<PixelRect> {
        if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
            return None;
        }
        let x0 = (x * scale).floor().clamp(0.0, width as f32) as u32;
        let y0 = (y * scale).floor().clamp(0.0, height as f32) as u32;
        let x1 = ((x + w) * scale).ceil().clamp(0.0, width as f32) as u32;
        let y1 = ((y + h) * scale).ceil().clamp(0.0, height as f32) as u32;
        (x1 > x0 && y1 > y0).then_some(PixelRect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        })
    }
}

impl Rgba {
    /// A `width`×`height` image of one colour.
    pub fn filled(width: u32, height: u32, rgba: [u8; 4]) -> Rgba {
        Rgba {
            width,
            height,
            pixels: rgba.repeat(width as usize * height as usize),
        }
    }

    fn at(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 4
    }

    /// The scale [`Rgba::fit`] applies for `long`: 1 when the image already
    /// fits or `long` is 0.
    pub fn fit_scale(width: u32, height: u32, long: u32) -> f32 {
        let side = width.max(height);
        if long == 0 || side <= long {
            1.0
        } else {
            long as f32 / side as f32
        }
    }

    /// The image with its long side at most `long` (0 keeps it), the aspect
    /// kept, each pixel the area-weighted mean of the pixels it covers
    /// (box filtering: no ringing, text stays legible). An image that
    /// already fits is returned as is.
    pub fn fit(&self, long: u32) -> Rgba {
        let scale = Rgba::fit_scale(self.width, self.height, long);
        if scale >= 1.0 {
            return self.clone();
        }
        let w = ((self.width as f32 * scale).round() as u32).max(1);
        let h = ((self.height as f32 * scale).round() as u32).max(1);
        self.resample(w, h)
    }

    /// Area-weighted downsampling to `w`×`h` (each no larger than the image's).
    fn resample(&self, w: u32, h: u32) -> Rgba {
        let (sx, sy) = (
            self.width as f64 / f64::from(w),
            self.height as f64 / f64::from(h),
        );
        // Each output column's source span and weights, once.
        let spans = |n: u32, s: f64, limit: u32| -> Vec<Vec<(u32, f64)>> {
            (0..n)
                .map(|i| {
                    let (a, b) = (f64::from(i) * s, (f64::from(i) + 1.0) * s);
                    let mut out = Vec::new();
                    let mut p = a.floor() as u32;
                    while f64::from(p) < b && p < limit {
                        let lo = a.max(f64::from(p));
                        let hi = b.min(f64::from(p) + 1.0);
                        if hi > lo {
                            out.push((p, hi - lo));
                        }
                        p += 1;
                    }
                    out
                })
                .collect()
        };
        let cols = spans(w, sx, self.width);
        let rows = spans(h, sy, self.height);
        let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
        for row in &rows {
            for col in &cols {
                let mut acc = [0f64; 4];
                let mut total = 0f64;
                for &(y, wy) in row {
                    for &(x, wx) in col {
                        let k = wx * wy;
                        let i = self.at(x, y);
                        // Premultiplied, so a transparent pixel adds no colour.
                        let a = f64::from(self.pixels[i + 3]) / 255.0;
                        for (c, v) in acc.iter_mut().take(3).enumerate() {
                            *v += f64::from(self.pixels[i + c]) * a * k;
                        }
                        acc[3] += a * k;
                        total += k;
                    }
                }
                let alpha = acc[3] / total.max(1e-12);
                for v in acc.iter().take(3) {
                    let c = if acc[3] > 0.0 { v / acc[3] } else { 0.0 };
                    pixels.push(c.round().clamp(0.0, 255.0) as u8);
                }
                pixels.push((alpha * 255.0).round().clamp(0.0, 255.0) as u8);
            }
        }
        Rgba {
            width: w,
            height: h,
            pixels,
        }
    }

    /// The pixels of `r` (clipped to the image) as their own image.
    pub fn crop(&self, r: PixelRect) -> Rgba {
        let x1 = (r.x + r.width).min(self.width);
        let y1 = (r.y + r.height).min(self.height);
        let (x0, y0) = (r.x.min(x1), r.y.min(y1));
        let mut pixels = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
        for y in y0..y1 {
            pixels.extend_from_slice(&self.pixels[self.at(x0, y)..self.at(x1 - 1, y) + 4]);
        }
        Rgba {
            width: x1 - x0,
            height: y1 - y0,
            pixels,
        }
    }

    /// Each pixel as a `factor`×`factor` block (nearest neighbour: the
    /// pixels a reviewer counts stay pixels).
    pub fn enlarge(&self, factor: u32) -> Rgba {
        let f = factor.max(1);
        let (w, h) = (self.width * f, self.height * f);
        let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                let i = self.at(x / f, y / f);
                pixels.extend_from_slice(&self.pixels[i..i + 4]);
            }
        }
        Rgba {
            width: w,
            height: h,
            pixels,
        }
    }

    /// Fills the part of the rectangle inside the image.
    pub fn fill(&mut self, x: i64, y: i64, w: i64, h: i64, rgba: [u8; 4]) {
        let x0 = x.clamp(0, i64::from(self.width)) as u32;
        let y0 = y.clamp(0, i64::from(self.height)) as u32;
        let x1 = (x + w).clamp(0, i64::from(self.width)) as u32;
        let y1 = (y + h).clamp(0, i64::from(self.height)) as u32;
        for yy in y0..y1 {
            for xx in x0..x1 {
                let i = self.at(xx, yy);
                self.pixels[i..i + 4].copy_from_slice(&rgba);
            }
        }
    }

    /// A `thickness`-pixel outline just inside `r`.
    pub fn stroke(&mut self, r: PixelRect, thickness: u32, rgba: [u8; 4]) {
        let (x, y, w, h) = (
            i64::from(r.x),
            i64::from(r.y),
            i64::from(r.width),
            i64::from(r.height),
        );
        let t = i64::from(thickness.max(1)).min(w.min(h));
        self.fill(x, y, w, t, rgba);
        self.fill(x, y + h - t, w, t, rgba);
        self.fill(x, y, t, h, rgba);
        self.fill(x + w - t, y, t, h, rgba);
    }

    /// `digits` (0 to 9 only) in a 3×5 pixel font scaled by `scale`, its
    /// top-left at `x, y`; answers the width drawn.
    pub fn digits(&mut self, x: i64, y: i64, digits: &str, scale: u32, rgba: [u8; 4]) -> i64 {
        let s = i64::from(scale.max(1));
        let mut at = x;
        for d in digits.bytes().filter(u8::is_ascii_digit) {
            let glyph = DIGITS[usize::from(d - b'0')];
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..3 {
                    if bits & (0b100 >> col) != 0 {
                        self.fill(at + col * s, y + row as i64 * s, s, s, rgba);
                    }
                }
            }
            at += 4 * s;
        }
        at - x - s
    }

    /// The PNG of the image: RGB when every pixel is opaque, else RGBA.
    pub fn png(&self) -> Vec<u8> {
        encode_png(self)
    }
}

/// The 3×5 digits, one row a byte (bit 2 the left column).
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b011, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b010, 0b010, 0b010],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

/// A number tag's size at `scale`: the digits with one scaled pixel of
/// padding around them.
pub fn tag_size(number: u32, scale: u32) -> (u32, u32) {
    let s = scale.max(1);
    let n = number.to_string().len() as u32;
    (n * 4 * s - s + 2 * s + 2, 5 * s + 2 * s + 2)
}

/// Draws mark `number` on `image`: a two-pixel outline in `colour` with a
/// dark edge outside it (legible on any ground), and the number in black on
/// a `colour` tag at the box's top-left corner, above the box when there is
/// room and inside it otherwise. Answers the tag's rectangle.
pub fn draw_mark(image: &mut Rgba, r: PixelRect, number: u32, colour: [u8; 3]) -> PixelRect {
    const DARK: [u8; 4] = [16, 16, 16, 255];
    let fill = [colour[0], colour[1], colour[2], 255];
    let (x, y, w, h) = (
        i64::from(r.x),
        i64::from(r.y),
        i64::from(r.width),
        i64::from(r.height),
    );
    // The dark edge, then the colour inside it.
    for (dx, dy, dw, dh) in [
        (x - 1, y - 1, w + 2, 1),
        (x - 1, y + h, w + 2, 1),
        (x - 1, y - 1, 1, h + 2),
        (x + w, y - 1, 1, h + 2),
    ] {
        image.fill(dx, dy, dw, dh, DARK);
    }
    image.stroke(r, 2, fill);
    let scale = 2;
    let (tw, th) = tag_size(number, scale);
    let ty = if y >= i64::from(th) {
        y - i64::from(th)
    } else {
        y
    };
    let tx = x.min(i64::from(image.width) - i64::from(tw)).max(0);
    image.fill(tx, ty, i64::from(tw), i64::from(th), fill);
    image.digits(
        tx + 1 + i64::from(scale),
        ty + 1 + i64::from(scale),
        &number.to_string(),
        scale,
        DARK,
    );
    PixelRect {
        x: tx as u32,
        y: ty as u32,
        width: tw,
        height: th,
    }
}

/// Decodes an 8-bit RGB or RGBA, non-interlaced PNG (the frames og writes
/// and most references).
pub fn decode_png(bytes: &[u8]) -> Result<Rgba, String> {
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("not a PNG".into());
    }
    let mut at = 8;
    let (mut width, mut height, mut channels) = (0u32, 0u32, 0usize);
    let mut data = Vec::new();
    while at + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[at..at + 4].try_into().expect("4 bytes")) as usize;
        let kind = &bytes[at + 4..at + 8];
        let body = bytes
            .get(at + 8..at + 8 + len)
            .ok_or("a truncated PNG chunk")?;
        match kind {
            b"IHDR" => {
                if body.len() < 13 {
                    return Err("a short IHDR".into());
                }
                width = u32::from_be_bytes(body[0..4].try_into().expect("4"));
                height = u32::from_be_bytes(body[4..8].try_into().expect("4"));
                let (depth, color, interlace) = (body[8], body[9], body[12]);
                channels = match color {
                    2 => 3,
                    6 => 4,
                    _ => return Err(format!("PNG colour type {color} (RGB or RGBA only)")),
                };
                if depth != 8 || interlace != 0 {
                    return Err("only 8-bit, non-interlaced PNGs".into());
                }
            }
            b"IDAT" => data.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + len;
    }
    if width == 0 || height == 0 {
        return Err("a PNG without an image".into());
    }
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&data)
        .map_err(|e| format!("a PNG whose data does not inflate ({e:?})"))?;
    let stride = width as usize * channels;
    if raw.len() < (stride + 1) * height as usize {
        return Err("a PNG with too little data".into());
    }
    let mut out = vec![0u8; width as usize * height as usize * 4];
    let mut prev = vec![0u8; stride];
    let mut cur = vec![0u8; stride];
    for y in 0..height as usize {
        let row = &raw[y * (stride + 1)..(y + 1) * (stride + 1)];
        let filter = row[0];
        for x in 0..stride {
            let a = if x >= channels { cur[x - channels] } else { 0 };
            let b = prev[x];
            let c = if x >= channels { prev[x - channels] } else { 0 };
            let v = row[x + 1];
            cur[x] = match filter {
                0 => v,
                1 => v.wrapping_add(a),
                2 => v.wrapping_add(b),
                3 => v.wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8),
                4 => v.wrapping_add(paeth(a, b, c)),
                f => return Err(format!("PNG filter {f}")),
            };
        }
        for x in 0..width as usize {
            let o = (y * width as usize + x) * 4;
            out[o..o + 3].copy_from_slice(&cur[x * channels..x * channels + 3]);
            out[o + 3] = if channels == 4 {
                cur[x * channels + 3]
            } else {
                255
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    Ok(Rgba {
        width,
        height,
        pixels: out,
    })
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let (pa, pb, pc) = (
        (p - i16::from(a)).abs(),
        (p - i16::from(b)).abs(),
        (p - i16::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// The PNG of `image`: RGB when every pixel is opaque, else RGBA; each row
/// with the Paeth filter (the best of the five for rendered frames, which
/// keeps writing one pass), deflated at level 6.
pub fn encode_png(image: &Rgba) -> Vec<u8> {
    let opaque = image.pixels.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
    let channels = if opaque { 3 } else { 4 };
    let stride = image.width as usize * channels;
    let mut raw = Vec::with_capacity((stride + 1) * image.height as usize);
    let mut prev = vec![0u8; stride];
    let mut cur = vec![0u8; stride];
    for y in 0..image.height as usize {
        for x in 0..image.width as usize {
            let i = (y * image.width as usize + x) * 4;
            cur[x * channels..x * channels + channels]
                .copy_from_slice(&image.pixels[i..i + channels]);
        }
        raw.push(4);
        for x in 0..stride {
            let a = if x >= channels { cur[x - channels] } else { 0 };
            let c = if x >= channels { prev[x - channels] } else { 0 };
            raw.push(cur[x].wrapping_sub(paeth(a, prev[x], c)));
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = [0u8; 13];
    ihdr[..4].copy_from_slice(&image.width.to_be_bytes());
    ihdr[4..8].copy_from_slice(&image.height.to_be_bytes());
    ihdr[8..].copy_from_slice(&[8, if opaque { 2 } else { 6 }, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let at = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[at..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_round_trips_opaque_and_transparent() {
        let mut img = Rgba::filled(5, 3, [10, 200, 30, 255]);
        img.fill(1, 1, 2, 1, [250, 0, 7, 255]);
        assert_eq!(decode_png(&img.png()).unwrap(), img);
        img.fill(0, 0, 1, 1, [1, 2, 3, 4]);
        assert_eq!(decode_png(&img.png()).unwrap(), img);
    }

    #[test]
    fn fit_keeps_the_aspect_and_averages_the_area() {
        // A 2560x1440 frame fits to 1280x720; a 4:1 strip keeps 4:1.
        let big = Rgba::filled(2560, 1440, [0, 0, 0, 255]);
        let f = big.fit(MODEL_LONG_SIDE);
        assert_eq!((f.width, f.height), (1280, 720));
        let strip = Rgba::filled(400, 100, [0, 0, 0, 255]).fit(200);
        assert_eq!((strip.width, strip.height), (200, 50));
        // Halving a black and white checker of 1 px gives mid grey, not one
        // of the two (a nearest-neighbour or point-sampled fit would).
        let mut checker = Rgba::filled(4, 2, [0, 0, 0, 255]);
        for (x, y) in [(0, 0), (2, 0), (1, 1), (3, 1)] {
            checker.fill(x, y, 1, 1, [255, 255, 255, 255]);
        }
        let half = checker.fit(2);
        assert_eq!((half.width, half.height), (2, 1));
        assert!(half.pixels.chunks(4).all(|p| p[0] == 128 && p[3] == 255));
        // An image that fits, or a long side of 0, is untouched.
        assert_eq!(checker.fit(0), checker);
        assert_eq!(checker.fit(4), checker);
    }

    #[test]
    fn crop_and_enlarge_keep_the_pixels() {
        let mut img = Rgba::filled(6, 4, [0, 0, 0, 255]);
        img.fill(4, 3, 1, 1, [9, 8, 7, 255]);
        let c = img.crop(PixelRect {
            x: 3,
            y: 2,
            width: 9,
            height: 9,
        });
        assert_eq!((c.width, c.height), (3, 2));
        assert_eq!(&c.pixels[(3 + 1) * 4..(3 + 1) * 4 + 3], &[9, 8, 7]);
        let e = c.enlarge(3);
        assert_eq!((e.width, e.height), (9, 6));
        assert_eq!(&e.pixels[(3 * 9 + 3) * 4..(3 * 9 + 3) * 4 + 3], &[9, 8, 7]);
        assert_eq!(&e.pixels[(5 * 9 + 5) * 4..(5 * 9 + 5) * 4 + 3], &[9, 8, 7]);
    }

    #[test]
    fn a_mark_is_drawn_around_its_box_with_its_number() {
        let mut img = Rgba::filled(64, 64, [255, 255, 255, 255]);
        let r = PixelRect {
            x: 20,
            y: 30,
            width: 20,
            height: 10,
        };
        let tag = draw_mark(&mut img, r, 7, [255, 196, 0]);
        // The tag sits above the box; the box's edge is the colour, and its
        // inside is untouched.
        assert_eq!(tag.y + tag.height, 30);
        assert_eq!(
            &img.pixels[img.at(30, 30)..img.at(30, 30) + 3],
            &[255, 196, 0]
        );
        assert_eq!(
            &img.pixels[img.at(30, 35)..img.at(30, 35) + 3],
            &[255, 255, 255]
        );
        // The digit 7's top row is dark inside the tag.
        let (dx, dy) = (tag.x + 3, tag.y + 3);
        assert_eq!(img.pixels[img.at(dx, dy)], 16);
        // A box at the top edge carries its tag inside.
        let top = draw_mark(
            &mut img,
            PixelRect {
                x: 0,
                y: 0,
                width: 30,
                height: 30,
            },
            12,
            [0, 220, 255],
        );
        assert_eq!(top.y, 0);
    }
}
