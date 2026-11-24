//! `og render --contact-sheet <cols>`: the render's review surface
//! (docs/design.md §2.9). A grid of `cols` columns and `ceil(cols / 2)` rows
//! of frames spaced evenly over the range (every frame when there are
//! fewer), each labelled with its time in seconds and its tick, drawn in a
//! 5x7 pixel font of its own so the sheet needs no font file and is the same
//! bytes on every machine. An agent reads one image instead of a video.

use image::{Rgba, RgbaImage, imageops};

/// Width of one cell's thumbnail (the frame's own when narrower).
pub const CELL_WIDTH: u32 = 320;
/// Pixels between cells and around the grid.
const GAP: u32 = 6;
/// The label's band under each thumbnail.
const LABEL: u32 = 20;
const BACKGROUND: Rgba<u8> = Rgba([18, 18, 22, 255]);
const INK: Rgba<u8> = Rgba([235, 235, 240, 255]);

/// The frames (indices into the render's `total`) a sheet of `cols` columns shows.
pub fn picks(total: usize, cols: u32) -> Vec<usize> {
    let cols = cols.max(1) as usize;
    let cells = cols * cols.div_ceil(2);
    if total <= cells {
        return (0..total).collect();
    }
    // Evenly spaced, first and last included.
    (0..cells)
        .map(|i| (i * (total - 1) + (cells - 1) / 2) / (cells - 1))
        .collect()
}

/// A frame shrunk to a cell's width (aspect kept), as the sheet draws it.
pub fn thumbnail(width: u32, height: u32, rgba: Vec<u8>) -> RgbaImage {
    let image = RgbaImage::from_raw(width, height, rgba).expect("width * height * 4 bytes");
    if width <= CELL_WIDTH {
        return image;
    }
    let h = (u64::from(height) * u64::from(CELL_WIDTH) / u64::from(width)).max(1) as u32;
    imageops::resize(&image, CELL_WIDTH, h, imageops::FilterType::Triangle)
}

/// The sheet: `cells` (thumbnail, label) in order, `cols` to a row.
pub fn sheet(cells: &[(RgbaImage, String)], cols: u32) -> RgbaImage {
    let cols = cols.max(1).min(cells.len().max(1) as u32);
    let rows = (cells.len() as u32).div_ceil(cols).max(1);
    let (cw, ch) = cells.iter().fold((1, 1), |(w, h), (t, _)| {
        (w.max(t.width()), h.max(t.height()))
    });
    let width = GAP + cols * (cw + GAP);
    let height = GAP + rows * (ch + LABEL + GAP);
    let mut out = RgbaImage::from_pixel(width, height, BACKGROUND);
    for (i, (thumb, label)) in cells.iter().enumerate() {
        let (c, r) = (i as u32 % cols, i as u32 / cols);
        let x = GAP + c * (cw + GAP);
        let y = GAP + r * (ch + LABEL + GAP);
        imageops::overlay(&mut out, thumb, i64::from(x), i64::from(y));
        text(&mut out, x + 2, y + ch + 4, label, 2);
    }
    out
}

/// A cell's label: `12.345s  t740`.
/// The characters of a label at `scale` that fit `width` pixels.
pub fn fits(width: u32, scale: u32) -> usize {
    (width / (6 * scale.max(1))) as usize
}

pub fn label(time: f64, tick: u64) -> String {
    format!("{time:.3}s  t{tick}")
}

/// Draws `s` at (x, y) in the 5x7 font, each font pixel `scale` pixels
/// (`og compare`'s sheet labels with it too).
pub(crate) fn text(img: &mut RgbaImage, x: u32, y: u32, s: &str, scale: u32) {
    let mut cx = x;
    for ch in s.chars() {
        if let Some(rows) = glyph(ch) {
            for (gy, bits) in rows.iter().enumerate() {
                for gx in 0..5u32 {
                    if bits & (0x10 >> gx) == 0 {
                        continue;
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let (px, py) = (cx + gx * scale + dx, y + gy as u32 * scale + dy);
                            if px < img.width() && py < img.height() {
                                img.put_pixel(px, py, INK);
                            }
                        }
                    }
                }
            }
        }
        cx += 6 * scale;
    }
}

/// The 5x7 rows of the characters labels use (top row first, bit 4 left):
/// the digits, `.`, `s`, `t` and the capitals A to Z.
fn glyph(c: char) -> Option<[u8; 7]> {
    Some(match c {
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1f, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x0c],
        's' => [0x00, 0x00, 0x0e, 0x10, 0x0e, 0x01, 0x1e],
        't' => [0x08, 0x08, 0x1c, 0x08, 0x08, 0x09, 0x06],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x11, 0x0a, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        ' ' => [0; 7],
        _ => return None,
    })
}
