//! A texture budget: a map over a size shrunk by a box filter in linear light.

use super::*;

/// `t` shrunk so its longest side is at most `d.max`, as PNG (as
/// lossless WebP where `t` is WebP); `None` when it already fits (or
/// `d.max` is 0), so the original bytes stay.
pub fn downsize(t: &Texture, d: &Downsize) -> Result<Option<Texture>, TextureError> {
    if d.max == 0 {
        return Ok(None);
    }
    let image = decode(&t.bytes)?;
    let longest = image.width.max(image.height);
    if longest <= d.max {
        return Ok(None);
    }
    let scale = |side: u32| {
        let v = (u64::from(side) * u64::from(d.max) + u64::from(longest) / 2) / u64::from(longest);
        (v as u32).max(1)
    };
    let small = resize(&image, scale(image.width), scale(image.height), d.color);
    if t.mime == "image/webp" {
        return Ok(Some(webp_of(&t.name, &small)));
    }
    let name = match t.name.rsplit_once('.') {
        Some((stem, _)) => format!("{stem}.png"),
        None => t.name.clone(),
    };
    Ok(Some(Texture {
        name,
        mime: "image/png".into(),
        bytes: crate::tools::png(&small),
    }))
}

/// Fixed-point scale of linear values.
pub(crate) const LIN_ONE: u32 = 1 << 20;

/// sRGB byte to linear, × [`LIN_ONE`], rounded.
pub(crate) fn lin_table() -> [u32; 256] {
    core::array::from_fn(|i| {
        let l = crate::tools::export::linear(i as f64 / 255.0);
        (l * f64::from(LIN_ONE)).round() as u32
    })
}

/// The sRGB byte whose linear value is nearest `v` (ties to the lower).
pub(crate) fn to_srgb(table: &[u32; 256], v: u32) -> u8 {
    let i = table.partition_point(|&x| x < v);
    if i == 0 {
        return 0;
    }
    if i == 256 {
        return 255;
    }
    if v - table[i - 1] <= table[i] - v {
        (i - 1) as u8
    } else {
        i as u8
    }
}

/// The weights of one axis: for each output texel, the input texels it
/// covers and how much of each (integer lengths, summing to `from`).
pub(crate) fn weights(from: u32, to: u32) -> Vec<Vec<(usize, u64)>> {
    let (from, to) = (u64::from(from), u64::from(to));
    (0..to)
        .map(|i| {
            // Output i spans [i·from, (i+1)·from); input j spans
            // [j·to, (j+1)·to), in the same units.
            let (lo, hi) = (i * from, (i + 1) * from);
            let mut w = Vec::new();
            let mut j = lo / to;
            while j * to < hi {
                let overlap = hi.min((j + 1) * to) - lo.max(j * to);
                if overlap > 0 {
                    w.push((j as usize, overlap));
                }
                j += 1;
            }
            w
        })
        .collect()
}

pub(crate) fn resize(img: &Image, tw: u32, th: u32, color: bool) -> Image {
    let (w, h) = (img.width as usize, img.height as usize);
    let table = lin_table();
    // Each channel in fixed point: linear × 2²⁰ for colour, × 2¹² else.
    let plain_shift = 12;
    let value = |c: usize, b: u8| -> u64 {
        if color && c < 3 {
            u64::from(table[b as usize])
        } else {
            u64::from(b) << plain_shift
        }
    };
    let wx = weights(img.width, tw);
    let wy = weights(img.height, th);
    let (fx, fy) = (u64::from(img.width), u64::from(img.height));
    // Horizontal pass: h rows of tw texels.
    let mut mid = vec![0u64; h * tw as usize * 4];
    for y in 0..h {
        let row = &img.rgba[y * w * 4..(y + 1) * w * 4];
        for (x, ws) in wx.iter().enumerate() {
            for c in 0..4 {
                let sum: u64 = ws.iter().map(|&(j, k)| value(c, row[j * 4 + c]) * k).sum();
                mid[(y * tw as usize + x) * 4 + c] = (sum + fx / 2) / fx;
            }
        }
    }
    // Vertical pass.
    let mut rgba = vec![0u8; tw as usize * th as usize * 4];
    for (y, ws) in wy.iter().enumerate() {
        for x in 0..tw as usize {
            for c in 0..4 {
                let sum: u64 = ws
                    .iter()
                    .map(|&(j, k)| mid[(j * tw as usize + x) * 4 + c] * k)
                    .sum();
                let v = (sum + fy / 2) / fy;
                rgba[(y * tw as usize + x) * 4 + c] = if color && c < 3 {
                    to_srgb(&table, v as u32)
                } else {
                    ((v + (1 << (plain_shift - 1))) >> plain_shift).min(255) as u8
                };
            }
        }
    }
    Image {
        width: tw,
        height: th,
        rgba,
    }
}
