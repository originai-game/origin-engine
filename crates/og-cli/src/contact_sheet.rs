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
pub fn label(time: f64, tick: u64) -> String {
    format!("{time:.3}s  t{tick}")
}

/// Draws `s` at (x, y) in the 5x7 font, each font pixel `scale` pixels.
fn text(img: &mut RgbaImage, x: u32, y: u32, s: &str, scale: u32) {
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

/// The 5x7 rows of the characters labels use (top row first, bit 4 left).
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
        ' ' => [0; 7],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_spread_over_the_range_with_both_ends() {
        assert_eq!(
            picks(4, 6),
            vec![0, 1, 2, 3],
            "fewer frames than cells: all"
        );
        // 240 frames (8 s at 30 fps), 6 columns: 18 cells.
        let p = picks(240, 6);
        assert_eq!(p.len(), 18);
        assert_eq!((p[0], p[17]), (0, 239));
        assert!(
            p.windows(2)
                .all(|w| w[1] > w[0] && w[1] - w[0] >= 13 && w[1] - w[0] <= 15)
        );
    }

    #[test]
    fn the_sheet_lays_cells_out_in_rows_and_labels_them() {
        let cell = |v: u8| {
            let rgba = [v, v, v, 255].repeat(640 * 360);
            (thumbnail(640, 360, rgba), label(1.5, 90))
        };
        let cells: Vec<_> = (0..5).map(|i| cell(40 * i as u8)).collect();
        let s = sheet(&cells, 3);
        assert_eq!(s.width(), GAP + 3 * (CELL_WIDTH + GAP));
        assert_eq!(s.height(), GAP + 2 * (180 + LABEL + GAP));
        // Cell 4 (row 1, column 1) holds its frame's value; the sixth is empty.
        let (x, y) = (
            GAP + (CELL_WIDTH + GAP) + 10,
            GAP + (180 + LABEL + GAP) + 10,
        );
        assert_eq!(s.get_pixel(x, y).0[0], 160);
        assert_eq!(
            *s.get_pixel(GAP + 2 * (CELL_WIDTH + GAP) + 10, y),
            BACKGROUND
        );
        // The label band of cell 0 has ink in it.
        let inked = (0..120).any(|dx| *s.get_pixel(GAP + 2 + dx, GAP + 180 + 8) == INK);
        assert!(inked, "the label is drawn");
        assert_eq!(label(2.0, 120), "2.000s  t120");
    }
}
