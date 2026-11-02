//! The parts packed into one texture: rows of parts, tallest first, two
//! pixels apart, each part's colour bled into the clear pixels around it
//! so linear filtering at its edge draws no dark fringe.

use super::super::texture::Image;
use super::Part;

/// The space between parts and around the atlas.
const PAD: u32 = 2;
/// The largest atlas side.
const MAX: u32 = 4096;

pub struct Packed {
    pub image: Image,
    /// Each part's rectangle (x y w h, pixels from the top left), in the parts' order.
    pub rects: Vec<[u32; 4]>,
}

/// Rows of `sizes` (tallest first) in a `width` wide atlas: each
/// rectangle's place and the height used.
fn shelves(sizes: &[(u32, u32)], width: u32) -> Option<(Vec<[u32; 2]>, u32)> {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(sizes[i].1), i));
    let mut at = vec![[0, 0]; sizes.len()];
    let (mut x, mut y, mut row) = (PAD, PAD, 0);
    for i in order {
        let (w, h) = sizes[i];
        if w + 2 * PAD > width {
            return None;
        }
        if x + w + PAD > width {
            x = PAD;
            y += row + PAD;
            row = 0;
        }
        at[i] = [x, y];
        x += w + PAD;
        row = row.max(h);
    }
    Some((at, y + row + PAD))
}

pub fn pack(parts: &[Part]) -> Result<Packed, String> {
    let sizes: Vec<(u32, u32)> = parts.iter().map(|p| (p.width, p.height)).collect();
    let area: u64 = sizes
        .iter()
        .map(|&(w, h)| (w + PAD) as u64 * (h + PAD) as u64)
        .sum();
    let widest = sizes.iter().map(|s| s.0).max().unwrap_or(1) + 2 * PAD;
    // The narrowest power-of-two width at least the widest part and about
    // square, then the height it needs.
    let mut width = widest
        .next_power_of_two()
        .max(((area as f64).sqrt() as u32).next_power_of_two());
    let (at, height) = loop {
        if width > MAX {
            return Err(format!(
                "the parts do not fit a {MAX}x{MAX} atlas; draw the character smaller (about 1024 pixels tall is plenty)"
            ));
        }
        match shelves(&sizes, width) {
            // At most twice as tall as wide, else a wider atlas.
            Some((at, h)) if h <= (width * 2).min(MAX) => break (at, h),
            _ => width *= 2,
        }
    };
    let height = height.next_power_of_two().max(4);
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let mut rects = Vec::with_capacity(parts.len());
    for (p, [x, y]) in parts.iter().zip(&at) {
        for row in 0..p.height {
            let src = &p.rgba[(row * p.width * 4) as usize..][..(p.width * 4) as usize];
            let o = (((y + row) * width + x) * 4) as usize;
            rgba[o..o + src.len()].copy_from_slice(src);
        }
        rects.push([*x, *y, p.width, p.height]);
    }
    bleed(&mut rgba, width, height, PAD);
    Ok(Packed {
        image: Image {
            width,
            height,
            rgba,
        },
        rects,
    })
}

/// Gives clear pixels within `steps` of a drawn one the average colour of
/// their drawn neighbours, alpha left at 0.
fn bleed(rgba: &mut [u8], w: u32, h: u32, steps: u32) {
    let (w, h) = (w as i32, h as i32);
    let mut known: Vec<bool> = rgba.chunks(4).map(|p| p[3] > 0).collect();
    for _ in 0..steps {
        let mut next = known.clone();
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if known[i] {
                    continue;
                }
                let (mut sum, mut n) = ([0u32; 3], 0);
                for (dx, dy) in [
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (1, 1),
                    (-1, 1),
                    (1, -1),
                ] {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let j = (ny * w + nx) as usize;
                    if known[j] {
                        for c in 0..3 {
                            sum[c] += rgba[j * 4 + c] as u32;
                        }
                        n += 1;
                    }
                }
                if let (Some(r), Some(g), Some(b)) = (
                    sum[0].checked_div(n),
                    sum[1].checked_div(n),
                    sum[2].checked_div(n),
                ) {
                    rgba[i * 4..i * 4 + 3].copy_from_slice(&[r as u8, g as u8, b as u8]);
                    next[i] = true;
                }
            }
        }
        known = next;
    }
}
