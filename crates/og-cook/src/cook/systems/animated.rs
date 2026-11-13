//! Animated image sources of a flipbook: a sheet with `source "hero.gif"`
//! lines instead of `texture` (animated GIF, animated WebP or APNG, the
//! last two with real alpha; a still image is one frame). The cook decodes
//! and composites each source's frames, turns their delays into clip
//! durations, optionally keys out a flat background, aligns the frames to a
//! common anchor, crops them to one size, shares identical frames and
//! packs them into a texture of its own, `<sheet>#sheet`; the sheet's text
//! becomes the canonical one the Aseprite path writes (`frame` lines and
//! clips), so the rest of the cook (the art contract, `array true`, the
//! runtime) sees an ordinary sheet.
//!
//! The grammar (og-anim's README has the whole sheet's):
//!
//! ```text
//! type flipbook
//! version 1
//! ---
//! source "hero_idle.gif"                  # a clip named `hero_idle`
//! source "hero_run.webp" {
//!   clip "run"                            # its clip's name (default: the file stem)
//!   mode loop                             # loop (default), pingPong or once
//!   fps 12                                # every frame 1/12 s instead of the file's delays
//!   event 3 "step"                        # a FlipbookEvent on the clip's frame 3
//!   align feet                            # this source's alignment (default: the sheet's)
//!   scale 0.9                             # times the sheet's scale: a source drawn larger
//! }
//! key auto 0.1 holes                      # off (default), auto or "#rrggbb", a tolerance, and
//!                                         # `holes`: enclosed pockets of the background too
//! align feet                              # feet, bbox, centroid or none (default)
//! padding 1                               # transparent texels around the common crop
//! scale 0.25                              # resample before anything else is measured
//! dedupe true                             # identical frames share one cell (default)
//! pivot 0.5 0                             # instead of the anchor the alignment found
//! crop each                               # each source its own cell size (default `all`: one)
//! filter nearest
//! ```
//!
//! [`inspect`] measures a decoded animation (`og sprite inspect`): frame
//! count, delays, per-frame bounds and centroid and their drift, the loop
//! seam and the alpha edge fringe. Measurements only, never a verdict.

use super::Source;
use og_core::ogt::{Document, Entry, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Alpha at or above which a texel counts as drawn, for bounds, centroids
/// and the crop (a speck of keying noise below it moves nothing).
pub const VISIBLE: u8 = 16;

/// The largest side of the packed texture.
const MAX_SIDE: u32 = 8192;

/// A decoded animation: every frame composited onto the full canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    /// `GIF`, `WebP`, `APNG`, or `still` for an image of one frame.
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Frame>,
    /// Delays the file gave as 10 ms or less, shown as 100 ms (the
    /// browsers' rule for GIF; 0 for the other formats).
    pub clamped: usize,
}

/// One composited frame, RGBA8 (straight alpha), and how long it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub rgba: Vec<u8>,
    /// Seconds.
    pub delay: f32,
}

fn err<E: std::fmt::Display>(what: &'static str) -> impl Fn(E) -> String {
    move |e| format!("{what}: {e}")
}

/// Decodes an animated GIF, WebP or PNG (APNG), or a still image (one
/// frame of 0.1 s), by its bytes.
pub fn decode(bytes: &[u8]) -> Result<Animation, String> {
    use image::AnimationDecoder;
    use std::io::Cursor;
    let frames: (&'static str, image::Frames<'_>) = if bytes.starts_with(b"GIF8") {
        let d = image::codecs::gif::GifDecoder::new(Cursor::new(bytes)).map_err(err("GIF"))?;
        ("GIF", d.into_frames())
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        let d = image::codecs::webp::WebPDecoder::new(Cursor::new(bytes)).map_err(err("WebP"))?;
        if !d.has_animation() {
            return still(bytes);
        }
        ("WebP", d.into_frames())
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        let d = image::codecs::png::PngDecoder::new(Cursor::new(bytes)).map_err(err("PNG"))?;
        if !d.is_apng().map_err(err("PNG"))? {
            return still(bytes);
        }
        ("APNG", d.apng().map_err(err("APNG"))?.into_frames())
    } else {
        return still(bytes);
    };
    let (format, frames) = frames;
    let mut out = Vec::new();
    let mut clamped = 0;
    let (mut width, mut height) = (0, 0);
    for f in frames {
        let f = f.map_err(|e| format!("{format}: frame {}: {e}", out.len()))?;
        let (n, d) = f.delay().numer_denom_ms();
        let mut ms = f64::from(n) / f64::from(d.max(1));
        if ms <= 10.0 && (format == "GIF" || ms == 0.0) {
            ms = 100.0;
            clamped += 1;
        }
        let buf = f.into_buffer();
        if out.is_empty() {
            (width, height) = buf.dimensions();
        } else if buf.dimensions() != (width, height) {
            return Err(format!(
                "{format}: frame {} changes the canvas size",
                out.len()
            ));
        }
        out.push(Frame {
            rgba: buf.into_raw(),
            delay: (ms / 1000.0) as f32,
        });
    }
    if out.is_empty() {
        return Err(format!("{format}: no frames"));
    }
    Ok(Animation {
        format,
        width,
        height,
        frames: out,
        clamped,
    })
}

fn still(bytes: &[u8]) -> Result<Animation, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("not an image: {e}"))?
        .to_rgba8();
    Ok(Animation {
        format: "still",
        width: img.width(),
        height: img.height(),
        frames: vec![Frame {
            rgba: img.into_raw(),
            delay: 0.1,
        }],
        clamped: 0,
    })
}

// ---- background keying ---------------------------------------------------

/// What the border of a frame is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Border {
    /// Most of it is transparent already: nothing to key.
    Transparent,
    /// One colour covers `share` of the border's texels.
    Flat { colour: [u8; 3], share: f32 },
}

/// How far (the largest channel difference, of 255) a border texel may be
/// from the background's colour and still be it: a generated GIF's palette
/// spreads one flat colour over several near ones (#f403f2, #f804f6,
/// #ef03ec), which also straddle the 5-bit buckets the mode is found in.
const FLAT_SPREAD: u8 = 12;

/// The border of an RGBA image: transparent when at least half its texels
/// are, else its most common colour (5 bits a channel, then the exact mean
/// of the opaque border texels within [`FLAT_SPREAD`] of that bucket's
/// mean) and the share of the border those texels cover.
pub fn border(rgba: &[u8], w: u32, h: u32) -> Border {
    let mut texels = Vec::new();
    for x in 0..w {
        texels.push(x as usize);
        texels.push(((h - 1) * w + x) as usize);
    }
    for y in 1..h.saturating_sub(1) {
        texels.push((y * w) as usize);
        texels.push((y * w + w - 1) as usize);
    }
    let clear = texels.iter().filter(|&&i| rgba[i * 4 + 3] == 0).count();
    if clear * 2 >= texels.len() {
        return Border::Transparent;
    }
    let bucket = |i: usize| {
        let p = &rgba[i * 4..i * 4 + 3];
        (u32::from(p[0] >> 3) << 10) | (u32::from(p[1] >> 3) << 5) | u32::from(p[2] >> 3)
    };
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for &i in &texels {
        if rgba[i * 4 + 3] > 0 {
            *counts.entry(bucket(i)).or_default() += 1;
        }
    }
    // The most texels; ties to the lowest bucket (BTreeMap order).
    let (best, _) = counts.iter().fold(
        (0u32, 0usize),
        |(b, n), (&k, &c)| if c > n { (k, c) } else { (b, n) },
    );
    let mean = |pick: &dyn Fn(usize) -> bool| {
        let (mut sum, mut n) = ([0u64; 3], 0u64);
        for &i in texels.iter().filter(|&&i| rgba[i * 4 + 3] > 0 && pick(i)) {
            for (c, s) in sum.iter_mut().enumerate() {
                *s += u64::from(rgba[i * 4 + c]);
            }
            n += 1;
        }
        let d = n.max(1);
        (sum.map(|s| ((s + d / 2) / d) as u8), n as usize)
    };
    // From the mode's mean to the mean of the texels near it, twice: the
    // mode may be one end of the spread, the second pass centres it.
    let (mut colour, mut near_n) = mean(&|i| bucket(i) == best);
    for _ in 0..2 {
        let at = colour;
        (colour, near_n) =
            mean(&|i| (0..3).all(|c| rgba[i * 4 + c].abs_diff(at[c]) <= FLAT_SPREAD));
    }
    Border::Flat {
        colour,
        share: near_n as f32 / texels.len() as f32,
    }
}

/// How a sheet keys its sources' background.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Off,
    /// The flat colour of the first frame's border, if it has one.
    Auto,
    Colour([u8; 3]),
}

/// The background colour a key takes out of an animation, or `None`.
pub fn key_colour(anim: &Animation, key: Key) -> Result<Option<[u8; 3]>, String> {
    match key {
        Key::Off => Ok(None),
        Key::Colour(c) => Ok(Some(c)),
        Key::Auto => match border(&anim.frames[0].rgba, anim.width, anim.height) {
            Border::Transparent => Ok(None),
            Border::Flat { colour, share } if share >= 0.5 => Ok(Some(colour)),
            Border::Flat { colour, share } => Err(format!(
                "key auto: the first frame's border is not one flat colour (#{:02x}{:02x}{:02x} covers {:.0}%); name it, `key \"#rrggbb\"`, or `key off`",
                colour[0],
                colour[1],
                colour[2],
                share * 100.0
            )),
        },
    }
}

fn distance(p: &[u8], k: [u8; 3]) -> f32 {
    let d = (0..3).map(|c| p[c].abs_diff(k[c])).max().unwrap_or(0);
    f32::from(d) / 255.0
}

/// Keys `colour` out of one frame: the texels within `2 tolerance` of it
/// (the largest channel difference, 0 to 1) that the border reaches
/// through such texels (so a white eye inside a character on white stays)
/// lose their alpha, fully within `tolerance` and in a ramp beyond, with
/// the background's colour taken back out of the ramp's texels. With
/// `holes`, every texel within `tolerance` of it reaches too: pockets of
/// background a figure encloses (between an arm and the body, inside a
/// slash's arc) go, and a white eye on white goes with them.
pub fn key_frame(rgba: &mut [u8], w: u32, h: u32, colour: [u8; 3], tolerance: f32, holes: bool) {
    let (w, h) = (w as usize, h as usize);
    let soft = tolerance * 2.0;
    let near =
        |rgba: &[u8], i: usize| rgba[i * 4 + 3] > 0 && distance(&rgba[i * 4..], colour) <= soft;
    let mut seen = vec![false; w * h];
    let mut queue = std::collections::VecDeque::new();
    for y in 0..h {
        for x in 0..w {
            let seed = if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                near(rgba, y * w + x)
            } else {
                holes
                    && rgba[(y * w + x) * 4 + 3] > 0
                    && distance(&rgba[(y * w + x) * 4..], colour) <= tolerance
            };
            if seed {
                seen[y * w + x] = true;
                queue.push_back(y * w + x);
            }
        }
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % w, i / w);
        let mut visit = |j: usize| {
            if !seen[j] && near(rgba, j) {
                seen[j] = true;
                queue.push_back(j);
            }
        };
        if x > 0 {
            visit(i - 1);
        }
        if x + 1 < w {
            visit(i + 1);
        }
        if y > 0 {
            visit(i - w);
        }
        if y + 1 < h {
            visit(i + w);
        }
    }
    // The ring of kept texels beside the keyed ones mixes the figure's edge
    // with the background (antialiasing, a GIF's dither): each is unmixed
    // against the mean of the kept texels two in from it, its alpha the
    // share of that colour along the line from the background to it.
    let ring = |i: usize| {
        let (x, y) = (i % w, i / w);
        !seen[i]
            && ((x > 0 && seen[i - 1])
                || (x + 1 < w && seen[i + 1])
                || (y > 0 && seen[i - w])
                || (y + 1 < h && seen[i + w]))
    };
    let edge: Vec<usize> = (0..w * h)
        .filter(|&i| ring(i) && rgba[i * 4 + 3] > 0)
        .collect();
    let on_ring: BTreeSet<usize> = edge.iter().copied().collect();
    let mut unmixed = Vec::with_capacity(edge.len());
    for &i in &edge {
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        let (mut sum, mut n) = ([0f32; 3], 0f32);
        for dy in -2i64..=2 {
            for dx in -2i64..=2 {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if seen[j] || on_ring.contains(&j) || rgba[j * 4 + 3] < 255 {
                    continue;
                }
                for c in 0..3 {
                    sum[c] += f32::from(rgba[j * 4 + c]);
                }
                n += 1.0;
            }
        }
        if n == 0.0 {
            continue;
        }
        let f = sum.map(|s| s / n);
        let k = colour.map(f32::from);
        let p = [0, 1, 2].map(|c| f32::from(rgba[i * 4 + c]));
        let fk = [0, 1, 2].map(|c| f[c] - k[c]);
        let len = fk.iter().map(|v| v * v).sum::<f32>();
        // A figure the background's colour cannot be told from it.
        if len < (0.1 * 255.0) * (0.1 * 255.0) {
            continue;
        }
        let a = (0..3).map(|c| (p[c] - k[c]) * fk[c]).sum::<f32>() / len;
        // Only a texel on the line between the two is their mix: a thin
        // stroke of another colour (an eye, a hilt) is left alone.
        let off = (0..3)
            .map(|c| {
                let d = p[c] - (k[c] + a * fk[c]);
                d * d
            })
            .sum::<f32>();
        if off > (0.12 * 255.0) * (0.12 * 255.0) {
            continue;
        }
        unmixed.push((i, a.clamp(0.0, 1.0), p));
    }
    for (i, a, p) in unmixed {
        let px = &mut rgba[i * 4..i * 4 + 4];
        if a <= 0.0 {
            px.copy_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        for c in 0..3 {
            let v = (p[c] - f32::from(colour[c]) * (1.0 - a)) / a;
            px[c] = v.round().clamp(0.0, 255.0) as u8;
        }
        px[3] = (f32::from(px[3]) * a).round() as u8;
    }
    for (i, reached) in seen.iter().enumerate() {
        if !reached {
            continue;
        }
        let p = &mut rgba[i * 4..i * 4 + 4];
        let d = distance(p, colour);
        if d <= tolerance || tolerance <= 0.0 {
            p.copy_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        let a = ((d - tolerance) / tolerance).clamp(0.0, 1.0);
        for c in 0..3 {
            let k = f32::from(colour[c]);
            let v = (f32::from(p[c]) - k * (1.0 - a)) / a;
            p[c] = v.round().clamp(0.0, 255.0) as u8;
        }
        p[3] = (f32::from(p[3]) * a).round() as u8;
    }
}

// ---- resampling --------------------------------------------------------------

/// The image resampled to `scale`: a box filter over premultiplied alpha
/// below 1 (each texel the exact average of what it covers), nearest
/// above.
pub fn resample(rgba: &[u8], w: u32, h: u32, scale: f32) -> (Vec<u8>, u32, u32) {
    let ow = ((w as f32 * scale).round() as u32).max(1);
    let oh = ((h as f32 * scale).round() as u32).max(1);
    if (ow, oh) == (w, h) {
        return (rgba.to_vec(), w, h);
    }
    if scale > 1.0 {
        let mut out = vec![0u8; (ow * oh * 4) as usize];
        for y in 0..oh {
            let sy = (u64::from(y) * u64::from(h) / u64::from(oh)) as u32;
            for x in 0..ow {
                let sx = (u64::from(x) * u64::from(w) / u64::from(ow)) as u32;
                let s = ((sy * w + sx) * 4) as usize;
                let d = ((y * ow + x) * 4) as usize;
                out[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
            }
        }
        return (out, ow, oh);
    }
    // Premultiplied, f32, one axis at a time.
    let pre: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            let a = f32::from(p[3]) / 255.0;
            [
                f32::from(p[0]) * a,
                f32::from(p[1]) * a,
                f32::from(p[2]) * a,
                f32::from(p[3]),
            ]
        })
        .collect();
    // The source span [x0, x1) of output texel i along an axis of n -> m.
    let weights = |n: u32, m: u32| -> Vec<Vec<(usize, f32)>> {
        let step = n as f32 / m as f32;
        (0..m)
            .map(|i| {
                let (a, b) = (i as f32 * step, (i + 1) as f32 * step);
                let mut ws = Vec::new();
                let mut s = a.floor() as u32;
                while (s as f32) < b && s < n {
                    let cover = (b.min(s as f32 + 1.0) - a.max(s as f32)).max(0.0);
                    if cover > 0.0 {
                        ws.push((s as usize, cover / step));
                    }
                    s += 1;
                }
                ws
            })
            .collect()
    };
    let (wx, wy) = (weights(w, ow), weights(h, oh));
    let mut rows = vec![0f32; (ow * h * 4) as usize];
    for y in 0..h as usize {
        for (x, ws) in wx.iter().enumerate() {
            for c in 0..4 {
                rows[(y * ow as usize + x) * 4 + c] = ws
                    .iter()
                    .map(|&(s, k)| pre[(y * w as usize + s) * 4 + c] * k)
                    .sum();
            }
        }
    }
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for (y, ws) in wy.iter().enumerate() {
        for x in 0..ow as usize {
            let mut v = [0f32; 4];
            for (c, vc) in v.iter_mut().enumerate() {
                *vc = ws
                    .iter()
                    .map(|&(s, k)| rows[(s * ow as usize + x) * 4 + c] * k)
                    .sum();
            }
            let a = v[3] / 255.0;
            let d = (y * ow as usize + x) * 4;
            for c in 0..3 {
                out[d + c] = if a > 0.0 {
                    (v[c] / a).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                };
            }
            out[d + 3] = v[3].round().clamp(0.0, 255.0) as u8;
        }
    }
    (out, ow, oh)
}

// ---- measuring ---------------------------------------------------------------

/// Where a frame's drawn texels (alpha at least [`VISIBLE`]) lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    /// `[x0, y0, x1, y1)`, from the top left.
    pub bounds: [u32; 4],
    /// The alpha-weighted centre of the drawn texels (texel centres).
    pub centroid: [f32; 2],
    /// The drawn texels' share of the canvas.
    pub coverage: f32,
}

/// The shape of a frame, `None` when nothing in it is drawn.
pub fn shape(rgba: &[u8], w: u32, h: u32) -> Option<Shape> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    let (mut sx, mut sy, mut sa) = (0f64, 0f64, 0f64);
    let mut drawn = 0usize;
    for y in 0..h {
        for x in 0..w {
            let a = rgba[((y * w + x) * 4 + 3) as usize];
            if a < VISIBLE {
                continue;
            }
            drawn += 1;
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
            let a = f64::from(a);
            sx += (f64::from(x) + 0.5) * a;
            sy += (f64::from(y) + 0.5) * a;
            sa += a;
        }
    }
    (drawn > 0).then(|| Shape {
        bounds: [x0, y0, x1, y1],
        centroid: [(sx / sa) as f32, (sy / sa) as f32],
        coverage: drawn as f32 / (w * h) as f32,
    })
}

/// What a frame is aligned by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// The frames keep their places; the source as a whole is placed by its
    /// first frame's feet.
    None,
    /// The bottom of the drawn bounds, at the centroid's x.
    Feet,
    /// The centre of the drawn bounds.
    Bbox,
    Centroid,
}

impl Align {
    fn parse(w: &str) -> Option<Align> {
        match w {
            "none" => Some(Align::None),
            "feet" => Some(Align::Feet),
            "bbox" => Some(Align::Bbox),
            "centroid" => Some(Align::Centroid),
            _ => None,
        }
    }

    /// The anchor of a shape, in texels from the top left.
    fn anchor(self, s: &Shape) -> [f32; 2] {
        let [x0, y0, x1, y1] = s.bounds.map(|v| v as f32);
        match self {
            Align::None | Align::Feet => [s.centroid[0], y1],
            Align::Bbox => [(x0 + x1) / 2.0, (y0 + y1) / 2.0],
            Align::Centroid => s.centroid,
        }
    }
}

/// The mean absolute difference of two frames over premultiplied RGBA, 0
/// (equal) to 1.
pub fn difference(a: &[u8], b: &[u8]) -> f32 {
    let mut sum = 0u64;
    for (p, q) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let (pa, qa) = (u32::from(p[3]), u32::from(q[3]));
        for c in 0..3 {
            let x = u32::from(p[c]) * pa;
            let y = u32::from(q[c]) * qa;
            sum += u64::from(x.abs_diff(y)) / 255;
        }
        sum += u64::from(pa.abs_diff(qa));
    }
    let n = (a.len().min(b.len()) as u64).max(1);
    (sum as f64 / (n as f64 * 255.0)) as f32
}

/// The texels at the edge of a frame's alpha: drawn, with a transparent
/// texel beside them (4-neighbours).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fringe {
    /// Edge texels.
    pub edge: usize,
    /// Edge texels with an alpha strictly between 0 and 255.
    pub partial: usize,
    /// The mean colour step (largest channel difference, 0 to 1) from an
    /// edge texel to the mean of the opaque, non-edge texels within two
    /// texels of it (edge texels without such neighbours are left out).
    pub step: f32,
    /// The mean distance of the edge texels from the background colour
    /// (when there is one): small is a halo of the background.
    pub to_background: Option<f32>,
}

pub fn fringe(rgba: &[u8], w: u32, h: u32, background: Option<[u8; 3]>) -> Fringe {
    let (w, h) = (w as i64, h as i64);
    let alpha = |x: i64, y: i64| -> u8 {
        if x < 0 || y < 0 || x >= w || y >= h {
            0
        } else {
            rgba[((y * w + x) * 4 + 3) as usize]
        }
    };
    let is_edge = |x: i64, y: i64| {
        alpha(x, y) > 0
            && [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| alpha(x + dx, y + dy) == 0)
    };
    let mut f = Fringe::default();
    let (mut steps, mut stepped) = (0f64, 0usize);
    let mut to_bg = 0f64;
    for y in 0..h {
        for x in 0..w {
            if !is_edge(x, y) {
                continue;
            }
            f.edge += 1;
            let i = ((y * w + x) * 4) as usize;
            if rgba[i + 3] < 255 {
                f.partial += 1;
            }
            if let Some(k) = background {
                to_bg += f64::from(distance(&rgba[i..], k));
            }
            let mut sum = [0u32; 3];
            let mut n = 0u32;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let (nx, ny) = (x + dx, y + dy);
                    if alpha(nx, ny) == 255 && !is_edge(nx, ny) {
                        let j = ((ny * w + nx) * 4) as usize;
                        for c in 0..3 {
                            sum[c] += u32::from(rgba[j + c]);
                        }
                        n += 1;
                    }
                }
            }
            if n > 0 {
                let mean = sum.map(|s| ((s + n / 2) / n) as u8);
                steps += f64::from(distance(&rgba[i..], mean));
                stepped += 1;
            }
        }
    }
    if stepped > 0 {
        f.step = (steps / stepped as f64) as f32;
    }
    if background.is_some() && f.edge > 0 {
        f.to_background = Some((to_bg / f.edge as f64) as f32);
    }
    f
}

/// What [`inspect`] measured of one frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameReport {
    pub delay: f32,
    pub shape: Option<Shape>,
    /// The centroid's offset from the first frame's, in texels.
    pub centroid_drift: Option<[f32; 2]>,
    /// The bounds' edges' offsets from the first frame's: left, top, right, bottom.
    pub bounds_drift: Option<[i64; 4]>,
    /// The first earlier frame it is identical to.
    pub duplicate_of: Option<usize>,
}

/// What `og sprite inspect` reports of an animation.
#[derive(Clone, Debug, PartialEq)]
pub struct Inspection {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub clamped: usize,
    /// The first frame's border.
    pub border: Border,
    /// The colour keyed out before measuring, when one was.
    pub keyed: Option<[u8; 3]>,
    /// The distinct alpha values across the frames (2: one-bit alpha).
    pub alpha_levels: usize,
    pub frames: Vec<FrameReport>,
    /// The largest centroid drift, in texels (x, y).
    pub max_centroid_drift: [f32; 2],
    /// The largest drift of each bounds edge: left, top, right, bottom.
    pub max_bounds_drift: [u64; 4],
    /// The last frame's difference from the first ([`difference`]).
    pub seam: f32,
    /// The mean and largest difference between consecutive frames.
    pub step_mean: f32,
    pub step_max: f32,
    /// The fringe summed over every frame (step and distance means over all edges).
    pub fringe: Fringe,
    /// With a key: the texels still drawn within its tolerance of it, over
    /// every frame (pockets of background the border does not reach, or a
    /// figure's own colour that close to the key).
    pub near_key: Option<usize>,
}

impl Inspection {
    pub fn duration(&self) -> f32 {
        self.frames.iter().map(|f| f.delay).sum()
    }
}

/// Measures an animation after keying `key` out of it (at `tolerance`,
/// with `holes` as [`key_frame`]).
pub fn inspect(
    anim: &Animation,
    key: Key,
    tolerance: f32,
    holes: bool,
) -> Result<Inspection, String> {
    let border0 = border(&anim.frames[0].rgba, anim.width, anim.height);
    let keyed = key_colour(anim, key)?;
    let frames: Vec<Vec<u8>> = anim
        .frames
        .iter()
        .map(|f| {
            let mut rgba = f.rgba.clone();
            if let Some(k) = keyed {
                key_frame(&mut rgba, anim.width, anim.height, k, tolerance, holes);
            }
            rgba
        })
        .collect();
    let mut report = measure(anim, border0, keyed, &frames);
    report.near_key = keyed.map(|k| {
        frames
            .iter()
            .flat_map(|f| f.as_chunks::<4>().0)
            .filter(|p| p[3] > 0 && distance(&p[..], k) <= tolerance)
            .count()
    });
    Ok(report)
}

fn measure(
    anim: &Animation,
    border0: Border,
    keyed: Option<[u8; 3]>,
    frames: &[Vec<u8>],
) -> Inspection {
    let (w, h) = (anim.width, anim.height);
    let shapes: Vec<Option<Shape>> = frames.iter().map(|f| shape(f, w, h)).collect();
    let first = shapes[0];
    let mut levels = [false; 256];
    let mut seen: BTreeMap<u64, usize> = BTreeMap::new();
    let mut reports = Vec::new();
    let mut max_c = [0f32; 2];
    let mut max_b = [0u64; 4];
    for (i, f) in frames.iter().enumerate() {
        for p in f.as_chunks::<4>().0 {
            levels[p[3] as usize] = true;
        }
        let s = shapes[i];
        let cd = s.zip(first).map(|(s, f0)| {
            [
                s.centroid[0] - f0.centroid[0],
                s.centroid[1] - f0.centroid[1],
            ]
        });
        let bd = s.zip(first).map(|(s, f0)| {
            let d = |k: usize| i64::from(s.bounds[k]) - i64::from(f0.bounds[k]);
            [d(0), d(1), d(2), d(3)]
        });
        if let Some(c) = cd {
            max_c = [max_c[0].max(c[0].abs()), max_c[1].max(c[1].abs())];
        }
        if let Some(b) = bd {
            for k in 0..4 {
                max_b[k] = max_b[k].max(b[k].unsigned_abs());
            }
        }
        let hash = og_core::hash::hash_bytes(f);
        let dup = match seen.get(&hash) {
            Some(&j) if frames[j] == *f => Some(j),
            _ => {
                seen.entry(hash).or_insert(i);
                None
            }
        };
        reports.push(FrameReport {
            delay: anim.frames[i].delay,
            shape: s,
            centroid_drift: cd,
            bounds_drift: bd,
            duplicate_of: dup,
        });
    }
    let steps: Vec<f32> = frames
        .windows(2)
        .map(|p| difference(&p[0], &p[1]))
        .collect();
    let seam = difference(frames.last().expect("a frame"), &frames[0]);
    let mut total = Fringe::default();
    let (mut step_sum, mut bg_sum) = (0f64, 0f64);
    for f in frames {
        let one = fringe(f, w, h, keyed);
        total.edge += one.edge;
        total.partial += one.partial;
        step_sum += f64::from(one.step) * one.edge as f64;
        bg_sum += f64::from(one.to_background.unwrap_or(0.0)) * one.edge as f64;
    }
    if total.edge > 0 {
        total.step = (step_sum / total.edge as f64) as f32;
        if keyed.is_some() {
            total.to_background = Some((bg_sum / total.edge as f64) as f32);
        }
    }
    Inspection {
        format: anim.format,
        width: w,
        height: h,
        clamped: anim.clamped,
        border: border0,
        keyed,
        alpha_levels: levels.iter().filter(|l| **l).count(),
        frames: reports,
        max_centroid_drift: max_c,
        max_bounds_drift: max_b,
        seam,
        step_mean: if steps.is_empty() {
            0.0
        } else {
            steps.iter().sum::<f32>() / steps.len() as f32
        },
        step_max: steps.iter().copied().fold(0.0, f32::max),
        fringe: total,
        near_key: None,
    }
}

// ---- the sheet ---------------------------------------------------------------

/// One `source` line of a sheet.
#[derive(Clone, Debug, PartialEq)]
struct SourceLine {
    /// The asset id of the file.
    id: String,
    clip: String,
    mode: Option<String>,
    fps: Option<f32>,
    events: Vec<(u32, String)>,
    align: Option<Align>,
    /// Times the sheet's `scale`.
    scale: f32,
}

/// The sheet-wide import settings.
#[derive(Clone, Debug, PartialEq)]
struct Settings {
    key: Key,
    tolerance: f32,
    holes: bool,
    align: Align,
    padding: u32,
    scale: f32,
    dedupe: bool,
    pivot: Option<[f32; 2]>,
    /// `crop each`: every source cut to its own frames' bounds, the cells
    /// shelf-packed (loose props and crops of many sizes in one texture).
    crop_each: bool,
}

const IMPORT_KEYS: [&str; 8] = [
    "source", "key", "align", "padding", "scale", "dedupe", "pivot", "crop",
];

fn line_err(e: &Entry, what: impl std::fmt::Display) -> String {
    format!("line {}: {what}", e.line)
}

/// A `"#rrggbb"` colour.
pub fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([b(0)?, b(2)?, b(4)?])
}

/// Whether a parsed flipbook sheet takes animated sources.
pub(crate) fn is_animated(doc: &Document) -> bool {
    doc.ty == "flipbook" && doc.body.get("source").is_some()
}

fn settings(doc: &Document) -> Result<Settings, String> {
    let mut s = Settings {
        key: Key::Off,
        tolerance: 0.1,
        holes: false,
        align: Align::None,
        padding: 1,
        scale: 1.0,
        dedupe: true,
        pivot: None,
        crop_each: false,
    };
    for e in &doc.body.entries {
        match e.key.as_str() {
            "key" => {
                let v = e
                    .values
                    .first()
                    .ok_or_else(|| line_err(e, "key off|auto|\"#rrggbb\" [tolerance] [holes]"))?;
                s.key = match (v.as_word(), v.as_str()) {
                    (Some("off"), _) => Key::Off,
                    (Some("auto"), _) => Key::Auto,
                    (_, Some(h)) => Key::Colour(parse_hex(h).ok_or_else(|| {
                        line_err(e, format!("key \"{h}\": a colour is \"#rrggbb\""))
                    })?),
                    _ => return Err(line_err(e, "key off|auto|\"#rrggbb\" [tolerance] [holes]")),
                };
                for (i, v) in e.values.iter().enumerate().skip(1) {
                    if v.as_word() == Some("holes") {
                        s.holes = true;
                        continue;
                    }
                    s.tolerance = e
                        .f32(i)
                        .ok()
                        .filter(|t| (0.0..=0.5).contains(t))
                        .ok_or_else(|| line_err(e, "the key's tolerance is 0 to 0.5"))?;
                }
            }
            "align" => {
                s.align = e
                    .word(0)
                    .ok()
                    .and_then(Align::parse)
                    .ok_or_else(|| line_err(e, "align feet|bbox|centroid|none"))?
            }
            "padding" => {
                s.padding = e
                    .f32(0)
                    .ok()
                    .filter(|p| *p >= 0.0 && p.fract() == 0.0 && *p <= 64.0)
                    .ok_or_else(|| line_err(e, "padding is a whole number of texels, 0 to 64"))?
                    as u32
            }
            "scale" => {
                s.scale = e
                    .f32(0)
                    .ok()
                    .filter(|v| *v > 0.0 && *v <= 8.0)
                    .ok_or_else(|| line_err(e, "scale is above 0, at most 8"))?
            }
            "dedupe" => {
                s.dedupe = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| line_err(e, "dedupe true|false"))?
            }
            "crop" => {
                s.crop_each = match e.word(0) {
                    Ok("each") => true,
                    Ok("all") => false,
                    _ => return Err(line_err(e, "crop all|each")),
                }
            }
            "pivot" => {
                s.pivot = Some([
                    e.f32(0).map_err(|_| line_err(e, "pivot x y"))?,
                    e.f32(1).map_err(|_| line_err(e, "pivot x y"))?,
                ])
            }
            "texture" | "atlas" | "grid" | "frame" | "margin" | "spacing" => {
                return Err(line_err(
                    e,
                    format!(
                        "`{}` and `source` do not mix: a sheet takes its frames from a texture or from animated sources",
                        e.key
                    ),
                ));
            }
            _ => {}
        }
    }
    Ok(s)
}

fn source_lines(
    doc: &Document,
    resolve: &dyn Fn(&str) -> String,
) -> Result<Vec<SourceLine>, String> {
    let mut out: Vec<SourceLine> = Vec::new();
    for e in doc.body.get_all("source") {
        let path = e
            .string(0)
            .map_err(|_| line_err(e, "source \"path\" [{ ... }]"))?;
        let id = resolve(path);
        let stem = id
            .rsplit('/')
            .next()
            .unwrap_or(&id)
            .split('.')
            .next()
            .unwrap_or("clip")
            .to_string();
        let mut line = SourceLine {
            id,
            clip: stem,
            mode: None,
            fps: None,
            events: Vec::new(),
            align: None,
            scale: 1.0,
        };
        for b in e.block.iter().flat_map(|b| &b.entries) {
            match b.key.as_str() {
                "clip" => {
                    line.clip = b
                        .values
                        .first()
                        .and_then(|v| v.as_str().or(v.as_word()))
                        .ok_or_else(|| line_err(b, "clip \"name\""))?
                        .to_string()
                }
                "mode" => {
                    let m = b.word(0).unwrap_or("");
                    if !matches!(m, "loop" | "pingPong" | "pingpong" | "once") {
                        return Err(line_err(b, "mode is loop, pingPong or once"));
                    }
                    line.mode = Some(m.to_string());
                }
                "fps" => {
                    line.fps = Some(
                        b.f32(0)
                            .ok()
                            .filter(|f| *f > 0.0 && f.is_finite())
                            .ok_or_else(|| line_err(b, "fps is above 0"))?,
                    )
                }
                "event" => line.events.push((
                    b.f32(0)
                        .ok()
                        .filter(|f| *f >= 0.0 && f.fract() == 0.0)
                        .ok_or_else(|| line_err(b, "event <frame> \"name\""))?
                        as u32,
                    b.string(1)
                        .map_err(|_| line_err(b, "event <frame> \"name\""))?
                        .to_string(),
                )),
                "align" => {
                    line.align = Some(
                        b.word(0)
                            .ok()
                            .and_then(Align::parse)
                            .ok_or_else(|| line_err(b, "align feet|bbox|centroid|none"))?,
                    )
                }
                "scale" => {
                    line.scale = b
                        .f32(0)
                        .ok()
                        .filter(|v| *v > 0.0 && *v <= 8.0)
                        .ok_or_else(|| line_err(b, "scale is above 0, at most 8"))?
                }
                k => {
                    return Err(line_err(
                        b,
                        format!(
                            "unknown key `{k}` in a source block (clip, mode, fps, event, align, scale)"
                        ),
                    ));
                }
            }
        }
        if out.iter().any(|o| o.clip == line.clip) {
            return Err(line_err(
                e,
                format!(
                    "two sources make clip `{}`; name one with `clip \"...\"`",
                    line.clip
                ),
            ));
        }
        out.push(line);
    }
    Ok(out)
}

/// A sheet built from its animated sources.
#[derive(Clone, Debug, PartialEq)]
pub struct Built {
    /// The canonical sheet text (its `texture` the packed image's id).
    pub text: String,
    /// The packed texture, a PNG.
    pub png: Vec<u8>,
    /// The common frame size.
    pub size: [u32; 2],
    /// Every source's frames, cropped to the common size, in source order.
    pub frames: Vec<Vec<u8>>,
    /// Each frame's delay (seconds), parallel to `frames`.
    pub delays: Vec<f32>,
    /// Frames that share another's cell.
    pub shared: usize,
    /// The asset ids of the files it read.
    pub read: Vec<String>,
    /// What the import did, for the cook report.
    pub notes: Vec<String>,
}

/// The texture id of a sheet's packed frames.
pub fn sheet_texture_id(rel: &str) -> String {
    format!("{rel}#sheet")
}

/// Builds sheet `rel` (its text `doc`) from its animated sources; `read`
/// returns a file's bytes by asset id and `resolve` turns a path the sheet
/// names into one.
pub fn build(
    rel: &str,
    doc: &Document,
    resolve: &dyn Fn(&str) -> String,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Built, String> {
    let set = settings(doc)?;
    let lines = source_lines(doc, resolve)?;
    let mut notes = Vec::new();
    // Decode, key, resample.
    struct Clip {
        line: SourceLine,
        frames: Vec<(Vec<u8>, f32)>,
        w: u32,
        h: u32,
    }
    let mut clips = Vec::new();
    for line in &lines {
        let bytes = read(&line.id)?;
        let anim = decode(&bytes).map_err(|e| format!("source `{}`: {e}", line.id))?;
        let keyed = key_colour(&anim, set.key).map_err(|e| format!("source `{}`: {e}", line.id))?;
        let mut what = format!(
            "{}: {} {} frame(s) of {}x{}",
            line.clip,
            anim.format,
            anim.frames.len(),
            anim.width,
            anim.height
        );
        if anim.clamped > 0 {
            what += &format!(
                ", {} delay(s) of 10 ms or less shown as 100 ms",
                anim.clamped
            );
        }
        if let Some(k) = keyed {
            what += &format!(
                ", keyed #{:02x}{:02x}{:02x} at {}{}",
                k[0],
                k[1],
                k[2],
                set.tolerance,
                if set.holes { " with its holes" } else { "" }
            );
        }
        notes.push(what);
        let mut frames = Vec::new();
        let (mut w, mut h) = (anim.width, anim.height);
        for f in &anim.frames {
            let mut rgba = f.rgba.clone();
            if let Some(k) = keyed {
                key_frame(
                    &mut rgba,
                    anim.width,
                    anim.height,
                    k,
                    set.tolerance,
                    set.holes,
                );
            }
            let (r, rw, rh) = resample(&rgba, anim.width, anim.height, set.scale * line.scale);
            (w, h) = (rw, rh);
            let delay = line.fps.map_or(f.delay, |fps| 1.0 / fps);
            frames.push((r, delay));
        }
        clips.push(Clip {
            line: line.clone(),
            frames,
            w,
            h,
        });
    }
    // Every frame's offset, in texels, onto the common canvas whose origin
    // is the reference anchor.
    let mut anchor_ref: Option<[f32; 2]> = None;
    let mut placed: Vec<Vec<(i64, i64, Option<Shape>)>> = Vec::new();
    for c in &clips {
        let align = c.line.align.unwrap_or(set.align);
        let shapes: Vec<Option<Shape>> = c.frames.iter().map(|(f, _)| shape(f, c.w, c.h)).collect();
        // A source with nothing drawn in its first frame is placed by its canvas's bottom centre.
        let canvas = [c.w as f32 / 2.0, c.h as f32];
        let first = shapes
            .iter()
            .flatten()
            .next()
            .map(|s| Align::Feet.anchor(s))
            .unwrap_or(canvas);
        let reference = *anchor_ref.get_or_insert(match (align, shapes.iter().flatten().next()) {
            (Align::None, _) => first,
            (a, Some(s)) => a.anchor(s),
            (_, None) => canvas,
        });
        let mut row = Vec::new();
        for s in &shapes {
            let a = match (align, s) {
                (Align::None, _) => first,
                (a, Some(s)) => a.anchor(s),
                (_, None) => first,
            };
            // Whole texels, so pixel art stays on its grid.
            let dx = (a[0] - reference[0]).round() as i64;
            let dy = (a[1] - reference[1]).round() as i64;
            row.push((dx, dy, *s));
        }
        placed.push(row);
    }
    let reference = anchor_ref.unwrap_or([0.0, 0.0]);
    // The common crop: the union of the drawn bounds, moved, plus padding.
    let (mut x0, mut y0, mut x1, mut y1) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for row in &placed {
        for (dx, dy, s) in row {
            if let Some(s) = s {
                let [a, b, c, d] = s.bounds.map(i64::from);
                (x0, y0, x1, y1) = (
                    x0.min(a - dx),
                    y0.min(b - dy),
                    x1.max(c - dx),
                    y1.max(d - dy),
                );
            }
        }
    }
    if x0 == i64::MAX {
        return Err(
            "every frame of every source is transparent (keyed away?): nothing to draw".into(),
        );
    }
    let pad = i64::from(set.padding);
    (x0, y0, x1, y1) = (x0 - pad, y0 - pad, x1 + pad, y1 + pad);
    let (fw, fh) = ((x1 - x0) as u32, (y1 - y0) as u32);
    // Cut every frame to the crop.
    let mut frames = Vec::new();
    let mut delays = Vec::new();
    for (c, row) in clips.iter().zip(&placed) {
        for ((rgba, delay), (dx, dy, _)) in c.frames.iter().zip(row) {
            let mut out = vec![0u8; (fw * fh * 4) as usize];
            for y in 0..fh as i64 {
                let sy = y + y0 + dy;
                if sy < 0 || sy >= i64::from(c.h) {
                    continue;
                }
                for x in 0..fw as i64 {
                    let sx = x + x0 + dx;
                    if sx < 0 || sx >= i64::from(c.w) {
                        continue;
                    }
                    let s = ((sy * i64::from(c.w) + sx) * 4) as usize;
                    let d = ((y * i64::from(fw) + x) * 4) as usize;
                    out[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
                }
            }
            frames.push(out);
            delays.push(*delay);
        }
    }
    // Each frame's cell within the crop: the whole crop, or with `crop
    // each` its source's frames' drawn bounds and the padding.
    let mut rects: Vec<[u32; 4]> = Vec::new();
    for c in &clips {
        let first = rects.len();
        let mut b = [u32::MAX, u32::MAX, 0u32, 0u32];
        for f in &frames[first..first + c.frames.len()] {
            if let Some(sh) = shape(f, fw, fh) {
                let [a, bb, cc, d] = sh.bounds;
                b = [b[0].min(a), b[1].min(bb), b[2].max(cc), b[3].max(d)];
            }
        }
        let rect = if set.crop_each && b[0] != u32::MAX {
            let p = set.padding;
            let (x0, y0) = (b[0].saturating_sub(p), b[1].saturating_sub(p));
            let (x1, y1) = ((b[2] + p).min(fw), (b[3] + p).min(fh));
            [x0, y0, x1 - x0, y1 - y0]
        } else {
            [0, 0, fw, fh]
        };
        rects.extend(std::iter::repeat_n(rect, c.frames.len()));
    }
    let cut = |i: usize| -> Vec<u8> {
        let [rx, ry, rw, rh] = rects[i];
        let mut out = Vec::with_capacity((rw * rh * 4) as usize);
        for y in ry..ry + rh {
            let at = ((y * fw + rx) * 4) as usize;
            out.extend_from_slice(&frames[i][at..at + (rw * 4) as usize]);
        }
        out
    };
    // Cells: one per distinct frame (of one size and the same texels).
    let images: Vec<Vec<u8>> = (0..frames.len()).map(cut).collect();
    let mut cell_of: Vec<usize> = Vec::new();
    let mut cells: Vec<usize> = Vec::new();
    let mut by_hash: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (i, f) in images.iter().enumerate() {
        let same = |j: usize| images[j] == *f && rects[j][2..] == rects[i][2..];
        let hit = set
            .dedupe
            .then(|| og_core::hash::hash_bytes(f))
            .and_then(|h| {
                let found = by_hash
                    .get(&h)
                    .and_then(|cs| cs.iter().copied().find(|&c| same(cells[c])));
                if found.is_none() {
                    by_hash.entry(h).or_default().push(cells.len());
                }
                found
            });
        match hit {
            Some(c) => cell_of.push(c),
            None => {
                cell_of.push(cells.len());
                cells.push(i);
            }
        }
    }
    let shared = frames.len() - cells.len();
    if shared > 0 {
        notes.push(format!("{shared} identical frame(s) share a cell"));
    }
    // Pack the cells one texel apart: in a grid near a square when they are
    // one size, else in shelves, tallest first, about as wide as tall.
    const GAP: u32 = 1;
    let n = cells.len() as u32;
    let size = |c: usize| [rects[cells[c]][2], rects[cells[c]][3]];
    let mut origin: Vec<(u32, u32)> = vec![(0, 0); cells.len()];
    let (tw, th) = if set.crop_each {
        let area: f64 = (0..cells.len())
            .map(|c| f64::from((size(c)[0] + GAP) * (size(c)[1] + GAP)))
            .sum();
        let widest = (0..cells.len()).map(|c| size(c)[0]).max().unwrap_or(1);
        let limit = (area.sqrt().ceil() as u32).max(widest).min(MAX_SIDE);
        let mut order: Vec<usize> = (0..cells.len()).collect();
        order.sort_by_key(|&c| {
            (
                std::cmp::Reverse(size(c)[1]),
                std::cmp::Reverse(size(c)[0]),
                c,
            )
        });
        let (mut x, mut y, mut shelf, mut tw) = (0u32, 0u32, 0u32, 0u32);
        for c in order {
            let [w, h] = size(c);
            if x > 0 && x + w > limit {
                (x, y, shelf) = (0, y + shelf + GAP, 0);
            }
            origin[c] = (x, y);
            tw = tw.max(x + w);
            shelf = shelf.max(h);
            x += w + GAP;
        }
        (tw, y + shelf)
    } else {
        let mut cols =
            ((n as f64 * f64::from(fh) / f64::from(fw)).sqrt().ceil() as u32).clamp(1, n);
        while cols > 1 && cols * (fw + GAP) - GAP > MAX_SIDE {
            cols -= 1;
        }
        for (c, o) in origin.iter_mut().enumerate() {
            let c = c as u32;
            *o = ((c % cols) * (fw + GAP), (c / cols) * (fh + GAP));
        }
        let rows = n.div_ceil(cols);
        (cols * (fw + GAP) - GAP, rows * (fh + GAP) - GAP)
    };
    if tw > MAX_SIDE || th > MAX_SIDE {
        return Err(format!(
            "{n} frames of {fw}x{fh} need a {tw}x{th} texture, past {MAX_SIDE}; `scale` them down"
        ));
    }
    let mut atlas = image::RgbaImage::new(tw, th);
    for (c, &i) in cells.iter().enumerate() {
        let (ox, oy) = origin[c];
        let [w, h] = size(c);
        for y in 0..h {
            for x in 0..w {
                let p = &images[i][((y * w + x) * 4) as usize..][..4];
                atlas.put_pixel(ox + x, oy + y, image::Rgba([p[0], p[1], p[2], p[3]]));
            }
        }
    }
    let mut png = Vec::new();
    {
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(atlas.as_raw(), tw, th, image::ExtendedColorType::Rgba8)
            .map_err(|e| format!("the packed sheet: {e}"))?;
    }
    // The pivot: the reference anchor in the frame's cell, from its bottom
    // left (the crop's, when the cell is the whole crop).
    let pivot = |r: [u32; 4]| {
        let (ax, ay) = (reference[0] - x0 as f32, reference[1] - y0 as f32);
        set.pivot.unwrap_or([
            (ax - r[0] as f32) / r[2] as f32,
            ((r[1] + r[3]) as f32 - ay) / r[3] as f32,
        ])
    };
    // The canonical text: the sheet's own lines but the import's, the
    // texture, a frame line per source frame, a clip per source.
    let num = |v: f64| Value::Number(og_core::ogt::Number::from_f64(v).expect("finite"));
    let mut out = Document::new("flipbook", 1);
    out.body
        .push(Entry::new("texture").with(Value::String(sheet_texture_id(rel))));
    for e in &doc.body.entries {
        if !IMPORT_KEYS.contains(&e.key.as_str()) && e.key != "clip" {
            out.body.push(e.clone());
        }
    }
    let mut k = 0usize;
    let mut clip_entries = Vec::new();
    for c in &clips {
        let mut names = Entry::new("frames");
        let mut durations = Entry::new("durations");
        for i in 0..c.frames.len() {
            let name = format!("{}_{i}", c.line.clip);
            let (ox, oy) = origin[cell_of[k]];
            let r = rects[k];
            let mut e = Entry::new("frame").with(Value::String(name.clone()));
            for v in [ox, oy, r[2], r[3]] {
                e = e.with(num(f64::from(v)));
            }
            let pv = pivot(r);
            e = e.with(num(f64::from(pv[0]))).with(num(f64::from(pv[1])));
            out.body.push(e);
            names = names.with(Value::String(name));
            durations = durations.with(num(f64::from(delays[k])));
            k += 1;
        }
        let mut block = og_core::ogt::Block::new();
        block.push(names);
        block.push(durations);
        block.push(Entry::new("mode").with(Value::Word(
            c.line.mode.clone().unwrap_or_else(|| "loop".into()),
        )));
        for (f, name) in &c.line.events {
            if *f as usize >= c.frames.len() {
                return Err(format!(
                    "source `{}`: event `{name}` on frame {f} of {}",
                    c.line.id,
                    c.frames.len()
                ));
            }
            block.push(
                Entry::new("event")
                    .with(num(f64::from(*f)))
                    .with(Value::String(name.clone())),
            );
        }
        clip_entries.push(
            Entry::new("clip")
                .with(Value::String(c.line.clip.clone()))
                .with_block(block),
        );
    }
    // The sheet's own clips come after (they name the frames above) and
    // win over a source's clip of the same name.
    let own: BTreeSet<String> = doc
        .body
        .get_all("clip")
        .filter_map(|e| {
            e.values
                .first()
                .and_then(|v| v.as_word().or(v.as_str()))
                .map(str::to_string)
        })
        .collect();
    for e in clip_entries {
        let name = e.values[0].as_str().unwrap_or_default().to_string();
        if !own.contains(&name) {
            out.body.push(e);
        }
    }
    for e in doc.body.get_all("clip") {
        out.body.push(e.clone());
    }
    let text = out.to_text().map_err(|e| e.reason().to_string())?;
    let drawn = cells
        .iter()
        .map(|&i| {
            images[i]
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] >= VISIBLE)
                .count()
        })
        .sum::<usize>();
    let first = pivot(rects[0]);
    notes.push(format!(
        "{} frame(s) of {fw}x{fh}{} in {} cell(s), packed {tw}x{th} ({:.1}% of its texels drawn), pivot {} {}",
        frames.len(),
        if set.crop_each { " cut to their sources' bounds" } else { "" },
        cells.len(),
        100.0 * drawn as f64 / f64::from(tw * th),
        first[0],
        first[1]
    ));
    Ok(Built {
        text,
        png,
        size: [fw, fh],
        frames,
        delays,
        shared,
        read: lines.iter().map(|l| l.id.clone()).collect(),
        notes,
    })
}

/// Builds the flipbook sheet at `path` (under a project's `assets/`) from
/// its animated sources, as the cook would.
pub fn build_file(path: &std::path::Path) -> Result<Built, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc = Document::parse(&text).map_err(|e| format!("{}: {}", path.display(), e.reason()))?;
    if !is_animated(&doc) {
        return Err(format!(
            "{}: not a flipbook with `source` lines; inspect the image itself",
            path.display()
        ));
    }
    let path = std::path::absolute(path).map_err(|e| e.to_string())?;
    let root = path
        .ancestors()
        .find(|a| a.file_name().is_some_and(|n| n == "assets"))
        .ok_or_else(|| format!("{}: not under a project's assets/", path.display()))?;
    let rel = path
        .strip_prefix(root)
        .expect("an ancestor")
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    let resolve = |p: &str| super::flipbook::resolve_in(root, &rel, p);
    let read = |id: &str| {
        std::fs::read(root.join(id)).map_err(|_| format!("source `{id}` is not in assets/"))
    };
    build(&rel, &doc, &resolve, &read)
}

/// Measures a built sheet's frames as [`inspect`] measures a file.
pub fn inspect_built(b: &Built) -> Inspection {
    let anim = Animation {
        format: "sheet",
        width: b.size[0],
        height: b.size[1],
        frames: b
            .frames
            .iter()
            .zip(&b.delays)
            .map(|(f, d)| Frame {
                rgba: f.clone(),
                delay: *d,
            })
            .collect(),
        clamped: 0,
    };
    let border0 = border(&b.frames[0], b.size[0], b.size[1]);
    measure(&anim, border0, None, &b.frames)
}

/// Expands every flipbook with animated sources before the cook hashes
/// anything: its text becomes the canonical sheet (with the packed image
/// as its texture, `Source::conformed`, so the sheet reads its size), the
/// packed image a texture source of its own (`<sheet>#sheet`, pixel art
/// under `filter nearest`), and the files it read are consumed.
pub(crate) fn expand(sources: &mut Vec<Source>, consumed: &mut BTreeSet<String>) {
    let mut extra = Vec::new();
    for s in sources.iter_mut() {
        if s.kind != &crate::cook::flipbook::FLIPBOOK || s.error.is_some() {
            continue;
        }
        let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
            continue;
        };
        if !is_animated(&doc) {
            continue;
        }
        let root = crate::cook::assets_root(s);
        let resolve = |p: &str| super::flipbook::resolve(s, p);
        let read = |id: &str| {
            std::fs::read(root.join(id)).map_err(|_| format!("source `{id}` is not in assets/"))
        };
        // The files it names are part of it, built or not.
        if let Ok(lines) = source_lines(&doc, &resolve) {
            consumed.extend(lines.into_iter().map(|l| l.id));
        }
        let built = match build(&s.rel, &doc, &resolve, &read) {
            Ok(b) => b,
            Err(e) => {
                s.error = Some(e);
                continue;
            }
        };
        let nearest = !doc
            .body
            .get("filter")
            .is_some_and(|e| e.word(0) == Ok("linear"));
        let settings = nearest.then(|| {
            let t = "type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".to_string();
            let d = Document::parse(&t).expect("the import text");
            (t, d)
        });
        s.bytes = built.text.into_bytes();
        s.conformed = Some(Arc::new(built.png.clone()));
        extra.push(Source {
            rel: sheet_texture_id(&s.rel),
            path: s.path.clone(),
            kind: &crate::cook::texture::TEXTURE,
            bytes: built.png,
            settings,
            settings_files: Vec::new(),
            hash: 0,
            gltf: None,
            error: None,
            encoded: Default::default(),
            whole: Default::default(),
            conformed: None,
            fracture: None,
        });
    }
    sources.extend(extra);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keying_takes_only_the_background_the_border_reaches() {
        // A white hole inside the block's ring stays.
        let mut px = vec![255u8; 8 * 8 * 4];
        for y in 2..6 {
            for x in 2..6 {
                if !(x == 3 || x == 4) || !(y == 3 || y == 4) {
                    let i = (y * 8 + x) * 4;
                    px[i..i + 4].copy_from_slice(&[0, 0, 200, 255]);
                }
            }
        }
        key_frame(&mut px, 8, 8, [255, 255, 255], 0.1, false);
        assert_eq!(px[3], 0, "the corner is keyed");
        assert_eq!(px[(3 * 8 + 3) * 4 + 3], 255, "the enclosed white stays");
        assert_eq!(px[(2 * 8 + 2) * 4 + 3], 255);
    }

    /// A generated GIF's magenta, quantised to near colours on both sides
    /// of a 5-bit bucket boundary (none of them half the border), is one
    /// flat background, keyed by `key auto`; a figure's colour is not.
    #[test]
    fn a_quantised_flat_border_is_one_colour() {
        let (w, h) = (16u32, 16u32);
        let shades = [[244u8, 3, 242], [248, 4, 246], [239, 3, 236], [251, 6, 250]];
        let mut px = Vec::new();
        for i in 0..w * h {
            px.extend_from_slice(&shades[i as usize % 4]);
            px.push(255);
        }
        // A figure touches the border on one side: 4 of its 60 texels.
        for y in 6..10 {
            let i = (y * w * 4) as usize;
            px[i..i + 4].copy_from_slice(&[40, 30, 20, 255]);
        }
        let Border::Flat { colour, share } = border(&px, w, h) else {
            panic!("an opaque border is flat or not");
        };
        assert!((share - 56.0 / 60.0).abs() < 1e-6, "share {share}");
        assert!(
            colour
                .iter()
                .zip([245, 4, 243])
                .all(|(a, b)| a.abs_diff(b) <= 2),
            "{colour:?}"
        );
        let anim = Animation {
            format: "GIF",
            width: w,
            height: h,
            frames: vec![Frame {
                rgba: px,
                delay: 0.1,
            }],
            clamped: 0,
        };
        assert_eq!(key_colour(&anim, Key::Auto), Ok(Some(colour)));
    }

    /// A ring on magenta enclosing a pocket of it (inside a slash's arc):
    /// the border's flood cannot reach the pocket, so it stays drawn and
    /// inspect counts it; `holes` keys it, the ring stays.
    #[test]
    fn holes_key_the_pockets_the_border_does_not_reach() {
        let (w, h) = (12u32, 12u32);
        let mut px = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let ring = (3..9).contains(&x)
                    && (3..9).contains(&y)
                    && !((4..8).contains(&x) && (4..8).contains(&y));
                px.extend_from_slice(if ring {
                    &[200, 200, 210, 255]
                } else {
                    &[248, 4, 246, 255]
                });
            }
        }
        let anim = Animation {
            format: "GIF",
            width: w,
            height: h,
            frames: vec![Frame {
                rgba: px,
                delay: 0.1,
            }],
            clamped: 0,
        };
        let plain = inspect(&anim, Key::Auto, 0.1, false).unwrap();
        assert_eq!(plain.near_key, Some(16), "the 4x4 pocket stays");
        let holes = inspect(&anim, Key::Auto, 0.1, true).unwrap();
        assert_eq!(holes.near_key, Some(0));
        let mut rgba = anim.frames[0].rgba.clone();
        key_frame(&mut rgba, w, h, [248, 4, 246], 0.1, true);
        assert_eq!(
            rgba[((5 * w + 5) * 4 + 3) as usize],
            0,
            "the pocket is keyed"
        );
        assert_eq!(rgba[((3 * w + 5) * 4 + 3) as usize], 255, "the ring stays");
    }

    #[test]
    fn the_ramp_takes_the_background_out_of_its_colour() {
        // Half way from white to a red: alpha about half, colour red again.
        let mut px = vec![255u8; 4 * 4 * 4];
        // Distance 0.15 from white at tolerance 0.1: alpha 0.5.
        let v = 255 - (0.15f32 * 255.0).round() as u8;
        px[0..4].copy_from_slice(&[255, v, v, 255]);
        key_frame(&mut px, 4, 4, [255, 255, 255], 0.1, false);
        assert!((120..=135).contains(&px[3]), "alpha {}", px[3]);
        assert_eq!(px[0], 255);
        assert!(px[1] < 200, "the white taken out: {}", px[1]);
    }

    /// The kept ring beside the keyed background: a texel half way from
    /// white to the red figure behind it becomes red at half alpha; a blue
    /// stroke there is not a mix of the two and stays.
    #[test]
    fn the_edge_ring_is_unmixed_from_the_background() {
        // 10x5: white, a red block at x 4..10, its first column the mix.
        let mut px = vec![255u8; 10 * 5 * 4];
        for y in 0..5 {
            for x in 4..10 {
                let i = (y * 10 + x) * 4;
                let c = if x == 4 {
                    [228, 128, 128, 255]
                } else {
                    [200, 0, 0, 255]
                };
                px[i..i + 4].copy_from_slice(&c);
            }
        }
        // Row 2's edge texel a blue stroke.
        let blue = (2 * 10 + 4) * 4;
        px[blue..blue + 4].copy_from_slice(&[0, 0, 220, 255]);
        key_frame(&mut px, 10, 5, [255, 255, 255], 0.1, false);
        let mixed = &px[(10 + 4) * 4..(10 + 4) * 4 + 4];
        assert!((125..=131).contains(&mixed[3]), "half alpha: {mixed:?}");
        assert!(mixed[0] >= 195 && mixed[1] <= 6, "red again: {mixed:?}");
        assert_eq!(&px[blue..blue + 4], &[0, 0, 220, 255], "the stroke stays");
        assert_eq!(px[3], 0, "the background is keyed");
    }

    #[test]
    fn box_resampling_averages_premultiplied() {
        // An opaque red texel beside a transparent one: half alpha, still red.
        let px = [255, 0, 0, 255, 0, 255, 0, 0];
        let (out, w, h) = resample(&px, 2, 1, 0.5);
        assert_eq!((w, h), (1, 1));
        assert_eq!(out, vec![255, 0, 0, 128]);
    }
}
