//! Views laid side by side in one image.

use super::*;

/// Images of one size side by side, `columns` to a row, left to right
/// then top to bottom; empty cells are the background colour.
pub fn sheet(images: &[Image], columns: u32) -> Image {
    let Some(first) = images.first() else {
        return Image::default();
    };
    let (w, h) = (first.width, first.height);
    let columns = columns.clamp(1, images.len() as u32);
    let rows = (images.len() as u32).div_ceil(columns);
    let (sw, sh) = (w * columns, h * rows);
    let mut rgba = Vec::with_capacity((sw * sh * 4) as usize);
    for _ in 0..sw * sh {
        rgba.extend_from_slice(&BACKGROUND);
        rgba.push(255);
    }
    for (i, im) in images.iter().enumerate() {
        if im.width != w || im.height != h {
            continue;
        }
        let (cx, cy) = (i as u32 % columns, i as u32 / columns);
        for y in 0..h {
            let src = (y * w * 4) as usize;
            let dst = (((cy * h + y) * sw + cx * w) * 4) as usize;
            rgba[dst..dst + (w * 4) as usize]
                .copy_from_slice(&im.rgba[src..src + (w * 4) as usize]);
        }
    }
    Image {
        width: sw,
        height: sh,
        rgba,
    }
}
