//! A 2D plant's wind parts: the `sway` line of a texture's import
//! settings (`texture { sway clumps }`) has the cook cut the image into
//! the parts og.Wind moves, cooked as `<id>#sway`: a texture of the parts
//! packed side by side with their table ([`SwayParts`]) in its `sway`
//! section. The runtime loads it beside the texture and an og.Sprite with
//! a `sway` draws the parts, each moved by the wind where it stands
//! (og-runtime's present/feel/sway.rs). Drawn in order at rest the parts
//! are the image, texel for texel.
//!
//! `sway clumps`: a broadleaf tree or a bush. Its wood stands still (the
//! trunk with the line drawn round it) and each bright mass of leaves is
//! a part (the bright spots are their middles, the darker lines between
//! them their borders: a watershed of the blurred luminance), looser the
//! farther it is from the crown's centre. A part holds a rim of its
//! neighbours' texels, so two clumps a texel or two apart still meet;
//! under them lies a darker copy of the crown, which leans with it, so
//! clumps drifting further apart show the inside of the tree and not the
//! ground.
//!
//! `sway tiers`: a conifer. Each tier of branches is a part (the rows
//! where the tree narrows part them), the trunk and the dark copy bend in
//! rows, and a tier rides the trunk at its height.
//!
//! `sway clumps "oak_parts.png"` (or `tiers`) takes the parts from a map
//! beside the texture instead of finding them: an image of the same size
//! whose every colour is one part, clear where the plant stands still.
//!
//! The parts and the motion are the method of SpriteCook's "How to make
//! pixel art trees move in the wind" (2026), written here from its
//! description; the hand-animated loop it reproduces is Raymond
//! Schlitter's (Pixelblog 33, 2021).

use super::texture::Image;
use crate::runtime::{SwayPart, SwayParts};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

/// What kind of plant a `sway` line names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plant {
    /// `sway clumps`.
    Clumps,
    /// `sway tiers`.
    Tiers,
}

/// A texture's `sway` line: the plant, and its parts map's file when it
/// names one (relative to the texture's directory).
#[derive(Clone, Debug, PartialEq)]
pub struct Setting {
    pub plant: Plant,
    pub map: Option<String>,
}

/// The `sway` line of the `texture` block, if any.
pub fn setting(block: Option<&og_core::ogt::Block>) -> Result<Option<Setting>, String> {
    let Some(e) = block.and_then(|b| b.get("sway")) else {
        return Ok(None);
    };
    let usage = || {
        format!(
            "line {}: sway takes clumps (a broadleaf tree, a bush) or tiers (a conifer), then a quoted parts map if it has one (sway clumps \"oak_parts.png\")",
            e.line
        )
    };
    let plant = match e.values.first().and_then(|v| v.as_word()) {
        Some("clumps") => Plant::Clumps,
        Some("tiers") => Plant::Tiers,
        _ => return Err(usage()),
    };
    let map = match e.values.len() {
        1 => None,
        2 => match e.string(1) {
            Ok(s) if !s.is_empty() => Some(s.to_string()),
            _ => return Err(usage()),
        },
        _ => return Err(usage()),
    };
    Ok(Some(Setting { plant, map }))
}

/// The most parts a plant is cut into: past it the clumps are found
/// farther apart.
const MAX_PARTS: usize = 32;
/// How dark the copy of the crown under the clumps is.
const INSIDE_SHADE: f32 = 0.55;
/// How much of the lean and the loop a part at the crown's centre takes
/// (one at its edge takes all of it).
const INNER: f32 = 0.35;
/// How far a broadleaf's dark copy leans, over its loosest clump: with
/// the clump at the middle of its loop, so the copy stays inside the
/// crown's outline on both sides.
const COPY_LEAD: f32 = 1.3;
/// Clear texels between two parts in the `#sway` texture.
const GUTTER: usize = 2;

/// An image's texels as flags and numbers, row by row.
struct Field {
    w: usize,
    h: usize,
}

impl Field {
    /// The texels beside `i`: four, or all eight.
    fn near(&self, i: usize, eight: bool) -> impl Iterator<Item = usize> + '_ {
        let (x, y) = ((i % self.w) as isize, (i / self.w) as isize);
        const D: [(isize, isize); 8] = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];
        D.into_iter()
            .take(if eight { 8 } else { 4 })
            .filter_map(move |(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                (nx >= 0 && ny >= 0 && nx < self.w as isize && ny < self.h as isize)
                    .then(|| ny as usize * self.w + nx as usize)
            })
    }

    /// Steps from each texel to the nearest one `from` holds (four ways or
    /// eight), and which one that is. With `edge`, the image's border
    /// counts as holding: one step past its outermost texels.
    fn steps(&self, from: impl Fn(usize) -> bool, eight: bool, edge: bool) -> (Vec<u32>, Vec<u32>) {
        let n = self.w * self.h;
        let mut d = vec![u32::MAX; n];
        let mut src = vec![u32::MAX; n];
        let mut q = VecDeque::new();
        for i in 0..n {
            if from(i) {
                d[i] = 0;
                src[i] = i as u32;
                q.push_back(i);
            } else if edge {
                let (x, y) = (i % self.w, i / self.w);
                if x == 0 || y == 0 || x == self.w - 1 || y == self.h - 1 {
                    d[i] = 1;
                    q.push_back(i);
                }
            }
        }
        while let Some(i) = q.pop_front() {
            for j in self.near(i, eight) {
                if d[j] == u32::MAX {
                    d[j] = d[i] + 1;
                    src[j] = src[i];
                    q.push_back(j);
                }
            }
        }
        (d, src)
    }
}

/// The plant's outline in rows: where it starts and ends, and the last row
/// of its crown (everything under it is the stem and the ground it stands
/// on, which never part from the trunk).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rows {
    top: usize,
    foot: usize,
    crown: usize,
    /// The stem's narrowest row and the columns it spans there (first,
    /// last); `None` for a plant without one.
    stem: Option<(usize, usize, usize)>,
}

impl Rows {
    fn height(&self) -> usize {
        self.foot - self.top + 1
    }

    /// How high row `y` stands: 0 at the foot, 1 at the top.
    fn up(&self, y: f32) -> f32 {
        ((self.foot as f32 + 1.0 - y) / self.height() as f32).clamp(0.0, 1.0)
    }
}

/// The plant's rows from its opaque texels: the stem is the narrowest row
/// of its lower 45 %, and the crown ends where going up from there the
/// plant is first much wider than its stem. A plant whose narrowest low
/// row is half its width or more has no stem (a bush): its crown is all
/// of it.
fn rows(f: &Field, solid: &[bool]) -> Option<(Rows, Vec<u32>)> {
    let widths: Vec<u32> = (0..f.h)
        .map(|y| solid[y * f.w..(y + 1) * f.w].iter().filter(|s| **s).count() as u32)
        .collect();
    let top = widths.iter().position(|w| *w > 0)?;
    let foot = widths.iter().rposition(|w| *w > 0)?;
    let height = foot - top + 1;
    let widest = *widths.iter().max()?;
    let low = foot - (height * 45 / 100).min(foot - top);
    // The highest of the narrowest rows: the one nearest the crown.
    let stem = (low..=foot)
        .filter(|y| widths[*y] > 0)
        .min_by_key(|y| (widths[*y], *y))?;
    if widths[stem] * 2 >= widest {
        let rows = Rows {
            top,
            foot,
            crown: foot,
            stem: None,
        };
        return Some((rows, widths));
    }
    let wide = (widths[stem] * 8 / 5).max(widest * 35 / 100);
    let crown = (top..stem)
        .rev()
        .find(|y| widths[*y] >= wide)
        .unwrap_or(top);
    let row = &solid[stem * f.w..(stem + 1) * f.w];
    let (first, last) = (row.iter().position(|s| *s)?, row.iter().rposition(|s| *s)?);
    let rows = Rows {
        top,
        foot,
        crown,
        stem: Some((stem, first, last)),
    };
    Some((rows, widths))
}

fn luminance(p: &[u8]) -> f32 {
    0.3 * f32::from(p[0]) + 0.59 * f32::from(p[1]) + 0.11 * f32::from(p[2])
}

/// Whether a texel is the colour of wood (brown, not bright) or of pale
/// bark (grey, not dark).
fn woody(p: &[u8]) -> bool {
    let (r, g, b) = (f32::from(p[0]), f32::from(p[1]), f32::from(p[2]));
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let sat = if max > 0.0 { (max - min) / max } else { 0.0 };
    let hue = if max == min {
        0.0
    } else if max == r {
        (60.0 * (g - b) / (max - min)).rem_euclid(360.0)
    } else if max == g {
        60.0 * (b - r) / (max - min) + 120.0
    } else {
        60.0 * (r - g) / (max - min) + 240.0
    };
    let lum = luminance(p);
    ((5.0..40.0).contains(&hue) && sat > 0.22 && lum < 125.0) || (sat < 0.16 && lum > 95.0)
}

/// `v` blurred by a Gaussian of `sigma` texels (clear past the image).
fn blur(f: &Field, v: &[f32], sigma: f32) -> Vec<f32> {
    let r = (sigma * 3.0).ceil() as isize;
    let k: Vec<f32> = (-r..=r)
        .map(|d| (-(d * d) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let sum: f32 = k.iter().sum();
    let pass = |src: &[f32], dx: isize, dy: isize| -> Vec<f32> {
        (0..f.w * f.h)
            .map(|i| {
                let (x, y) = ((i % f.w) as isize, (i / f.w) as isize);
                let mut a = 0.0;
                for (j, kv) in k.iter().enumerate() {
                    let d = j as isize - r;
                    let (sx, sy) = (x + dx * d, y + dy * d);
                    if sx >= 0 && sy >= 0 && sx < f.w as isize && sy < f.h as isize {
                        a += kv * src[sy as usize * f.w + sx as usize];
                    }
                }
                a / sum
            })
            .collect()
    };
    pass(&pass(v, 1, 0), 0, 1)
}

/// Floods `label` from its marked texels through `mask`, the brightest
/// texels of `bright` first, so two floods meet along the darker line
/// between them (a marker watershed).
fn watershed(f: &Field, bright: &[f32], mask: &[bool], label: &mut [u32]) {
    // The brightest first, then the oldest: (brightness, age, texel).
    let mut heap: BinaryHeap<(u32, Reverse<u64>, usize)> = BinaryHeap::new();
    let key = |v: f32| (v.max(0.0) * 256.0) as u32;
    let mut age = 0u64;
    for i in 0..label.len() {
        if label[i] != 0 {
            heap.push((key(bright[i]), Reverse(age), i));
            age += 1;
        }
    }
    while let Some((_, _, i)) = heap.pop() {
        for j in f.near(i, false) {
            if mask[j] && label[j] == 0 {
                label[j] = label[i];
                heap.push((key(bright[j]), Reverse(age), j));
                age += 1;
            }
        }
    }
}

/// How far apart in texels things of a plant `height` texels tall are,
/// for a measure that is `at_100` on one of 100.
fn scaled(height: usize, at_100: f32, least: f32) -> f32 {
    (at_100 * height as f32 / 100.0).max(least)
}

/// `sway clumps`: each texel's clump (0: none, the wood and the ground).
fn clumps(f: &Field, img: &Image, solid: &[bool], r: Rows, apart: f32) -> Vec<u32> {
    let n = f.w * f.h;
    let px = |i: usize| &img.rgba[i * 4..i * 4 + 4];
    let in_crown = |i: usize| i / f.w <= r.crown;
    // The wood: wood-coloured texels joined to the stem.
    let mut wood = vec![false; n];
    let mut q: VecDeque<usize> = (0..n)
        .filter(|&i| solid[i] && !in_crown(i) && woody(px(i)))
        .collect();
    for &i in &q {
        wood[i] = true;
    }
    while let Some(i) = q.pop_front() {
        for j in f.near(i, true) {
            if solid[j] && !wood[j] && woody(px(j)) {
                wood[j] = true;
                q.push_back(j);
            }
        }
    }
    // The trunk's drawn outline is not its colour, and is the trunk's: a
    // texel under the crown between the wood and the clear, darker than
    // the wood.
    let count = wood.iter().filter(|w| **w).count().max(1);
    let shade = (0..n)
        .filter(|&i| wood[i])
        .map(|i| luminance(px(i)))
        .sum::<f32>()
        / count as f32;
    let outline: Vec<usize> = (0..n)
        .filter(|&i| {
            solid[i]
                && !wood[i]
                && !in_crown(i)
                && luminance(px(i)) < shade * 0.75
                && f.near(i, true).any(|j| wood[j])
                && (f.near(i, true).count() < 8 || f.near(i, true).any(|j| !solid[j]))
        })
        .collect();
    let mut line = vec![false; n];
    for i in outline {
        (wood[i], line[i]) = (true, true);
    }
    let leaves: Vec<bool> = (0..n).map(|i| solid[i] && !wood[i]).collect();
    let lum: Vec<f32> = (0..n)
        .map(|i| if solid[i] { luminance(px(i)) } else { 0.0 })
        .collect();
    let bright = blur(f, &lum, scaled(r.height(), 1.6, 0.8));
    // The clumps' middles: the brightest leaf texel of each neighbourhood,
    // no two nearer than `apart`.
    let d = apart.round().max(2.0) as isize;
    let mut peaks: Vec<usize> = (0..n)
        .filter(|&i| {
            if !leaves[i] {
                return false;
            }
            let (x, y) = ((i % f.w) as isize, (i / f.w) as isize);
            (-d..=d).all(|dy| {
                (-d..=d).all(|dx| {
                    let (sx, sy) = (x + dx, y + dy);
                    sx < 0
                        || sy < 0
                        || sx >= f.w as isize
                        || sy >= f.h as isize
                        || bright[sy as usize * f.w + sx as usize] <= bright[i]
                })
            })
        })
        .collect();
    peaks.sort_by(|a, b| bright[*b].total_cmp(&bright[*a]).then(a.cmp(b)));
    let mut label = vec![0u32; n];
    let mut kept: Vec<usize> = Vec::new();
    for i in peaks {
        let (x, y) = ((i % f.w) as isize, (i / f.w) as isize);
        let far = kept.iter().all(|&k| {
            let (kx, ky) = ((k % f.w) as isize, (k / f.w) as isize);
            (kx - x).abs().max((ky - y).abs()) > d
        });
        if far {
            kept.push(i);
            label[i] = kept.len() as u32;
        }
    }
    watershed(f, &bright, &leaves, &mut label);
    // Crumbs stay on the wood, and so does what grows at its foot (grass,
    // a root's moss): a clump centred under the stem's narrowest row. A
    // clump hanging beside the trunk above it is the crown's.
    let least = scaled(r.height(), 5.0, 2.5).powi(2) as usize;
    let low = r.stem.map_or(usize::MAX, |(row, ..)| row);
    let mut sizes = vec![(0usize, 0usize); kept.len() + 1];
    for (i, &l) in label.iter().enumerate() {
        sizes[l as usize].0 += 1;
        sizes[l as usize].1 += i / f.w;
    }
    for l in &mut label {
        let (count, rows) = sizes[*l as usize];
        if count < least || rows / count.max(1) > low {
            *l = 0;
        }
    }
    // The crown's outline is often the wood's colour and would stay behind
    // when its clump moves: an outline texel goes to the clump beside it,
    // unless the wood's own body is as near.
    let reach = scaled(r.height(), 2.0, 2.0).round() as u32;
    let (edge, _) = f.steps(|i| !solid[i], true, true);
    let (to_clump, nearest) = f.steps(|i| label[i] != 0, true, false);
    let free = |i: usize| solid[i] && label[i] == 0 && !line[i];
    let pale = |i: usize| {
        let p = px(i);
        let (max, min) = (p[0].max(p[1]).max(p[2]), p[0].min(p[1]).min(p[2]));
        max > 0 && f32::from(max - min) / f32::from(max) < 0.16 && luminance(p) > 95.0
    };
    let (to_body, _) = f.steps(|i| free(i) && (edge[i] > reach || pale(i)), true, false);
    let joined: Vec<(usize, u32)> = (0..n)
        .filter(|&i| {
            free(i)
                && edge[i] <= reach
                && to_clump[i] <= reach + 1
                && to_clump[i] < to_body[i]
                && to_body[i] > 1
        })
        .map(|i| (i, label[nearest[i] as usize]))
        .collect();
    for (i, l) in joined {
        label[i] = l;
    }
    label
}

/// `sway tiers`: each texel's tier (0: none), the lowest tier first.
fn tiers(f: &Field, img: &Image, solid: &[bool], r: Rows, widths: &[u32]) -> Vec<u32> {
    let n = f.w * f.h;
    let smooth = |y: usize| -> f32 {
        let (a, b) = (y.saturating_sub(1).max(r.top), (y + 1).min(r.crown));
        (a..=b).map(|y| widths[y] as f32).sum::<f32>() / (b - a + 1) as f32
    };
    // Going down the crown a tier widens to its hem, then the plant
    // narrows to the top of the next: a new tier starts at the narrowest
    // row after each such fall.
    let least = scaled(r.height(), 9.0, 3.0) as usize;
    let mut starts = vec![r.top];
    let (mut peak, mut y) = (0.0f32, r.top);
    while y <= r.crown {
        let w = smooth(y);
        peak = peak.max(w);
        if w < 0.9 * peak && y - starts[starts.len() - 1] >= least {
            let mut m = y;
            while m < r.crown && smooth(m + 1) <= smooth(m) {
                m += 1;
            }
            if r.crown + 1 - m >= least {
                starts.push(m);
                peak = smooth(m);
            }
            y = m;
        }
        y += 1;
    }
    starts.push(r.crown + 1);
    // Each band's middle rows are its tier's; the rows between two bands'
    // middles go to the tier whose flood reaches them first, so the border
    // follows the shadow under a hem and not a straight row.
    // Tips of the lowest hem hang under the crown's last row, beside the
    // stem: they are the tier's too (the flood reaches them from it).
    let mask: Vec<bool> = (0..n)
        .map(|i| {
            let (x, y) = (i % f.w, i / f.w);
            solid[i]
                && (y <= r.crown
                    || r.stem
                        .is_some_and(|(row, a, b)| y < row && (x + 1 < a || x > b + 1)))
        })
        .collect();
    let mut label = vec![0u32; n];
    let bands = starts.len() - 1;
    for (k, band) in starts.windows(2).enumerate() {
        let (a, b) = (band[0], band[1]);
        let third = (b - a) / 3;
        for y in a + third..(b - third).max(a + third + 1) {
            for x in 0..f.w {
                if mask[y * f.w + x] {
                    // The lowest tier is 1: it draws first.
                    label[y * f.w + x] = (bands - k) as u32;
                }
            }
        }
    }
    let lum: Vec<f32> = (0..n)
        .map(|i| {
            if solid[i] {
                luminance(&img.rgba[i * 4..i * 4 + 3])
            } else {
                0.0
            }
        })
        .collect();
    let bright = blur(f, &lum, scaled(r.height(), 1.6, 0.8));
    watershed(f, &bright, &mask, &mut label);
    // A sliver left between two tiers (the shadow under a hem, read as a
    // narrowing) is the hem's: it joins the tier above it.
    let mut span = vec![(usize::MAX, 0usize); bands + 1];
    for (i, &l) in label.iter().enumerate() {
        let (a, b) = &mut span[l as usize];
        (*a, *b) = ((*a).min(i / f.w), (*b).max(i / f.w));
    }
    let mut into: Vec<u32> = (0..=bands as u32).collect();
    for k in (1..bands).rev() {
        let (a, b) = span[k];
        if a != usize::MAX && b - a + 1 < least * 2 / 3 {
            into[k] = into[k + 1];
        }
    }
    for l in &mut label {
        *l = into[*l as usize];
    }
    label
}

/// A parts map's parts: each of its colours one, on the plant's opaque
/// texels, the lowest first.
fn mapped(f: &Field, solid: &[bool], map: &Image) -> Result<Vec<u32>, String> {
    if (map.width as usize, map.height as usize) != (f.w, f.h) {
        return Err(format!(
            "the parts map is {}x{}, the texture {}x{}: a map is the texture's size",
            map.width, map.height, f.w, f.h
        ));
    }
    // Each colour's texels and the sum of their rows.
    let mut colours: BTreeMap<[u8; 3], (u64, u64)> = BTreeMap::new();
    let colour = |i: usize| -> Option<[u8; 3]> {
        let p = &map.rgba[i * 4..i * 4 + 4];
        (solid[i] && p[3] >= 128).then(|| [p[0], p[1], p[2]])
    };
    for i in 0..f.w * f.h {
        if let Some(c) = colour(i) {
            let e = colours.entry(c).or_default();
            e.0 += 1;
            e.1 += (i / f.w) as u64;
        }
    }
    if colours.len() > 2 * MAX_PARTS {
        return Err(format!(
            "the parts map has {} colours: each colour is one part, at most {} (flat colours, no antialiasing)",
            colours.len(),
            2 * MAX_PARTS
        ));
    }
    let mut order: Vec<([u8; 3], f64)> = colours
        .iter()
        .map(|(c, (count, rows))| (*c, *rows as f64 / *count as f64))
        .collect();
    order.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let index: BTreeMap<[u8; 3], u32> = order
        .iter()
        .enumerate()
        .map(|(k, (c, _))| (*c, k as u32 + 1))
        .collect();
    Ok((0..f.w * f.h)
        .map(|i| colour(i).map_or(0, |c| index[&c]))
        .collect())
}

/// One part before it is packed: its texels' box on the canvas (x, y,
/// width, height) and its image.
struct Piece {
    at: [usize; 2],
    size: [usize; 2],
    rgba: Vec<u8>,
    centre: [f32; 2],
    lean: f32,
    flutter: f32,
}

/// The texels `keep` holds of `colour` as a piece trimmed to them; `None`
/// when it holds none.
fn piece(
    f: &Field,
    keep: impl Fn(usize) -> bool,
    colour: impl Fn(usize) -> [u8; 4],
) -> Option<Piece> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    let (mut sx, mut sy, mut count) = (0.0f64, 0.0f64, 0u64);
    for i in 0..f.w * f.h {
        if keep(i) {
            let (x, y) = (i % f.w, i / f.w);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            sx += x as f64 + 0.5;
            sy += y as f64 + 0.5;
            count += 1;
        }
    }
    (count > 0).then(|| {
        let size = [x1 - x0 + 1, y1 - y0 + 1];
        let mut rgba = vec![0u8; size[0] * size[1] * 4];
        for y in 0..size[1] {
            for x in 0..size[0] {
                let i = (y0 + y) * f.w + x0 + x;
                if keep(i) {
                    let o = (y * size[0] + x) * 4;
                    rgba[o..o + 4].copy_from_slice(&colour(i));
                }
            }
        }
        Piece {
            at: [x0, y0],
            size,
            rgba,
            centre: [(sx / count as f64) as f32, (sy / count as f64) as f32],
            lean: 0.0,
            flutter: 0.0,
        }
    })
}

/// The image cut by `label` (0: what stands still; 1 and up: the parts in
/// drawing order) into pieces: the still wood, then the dark copy of the
/// crown (the two together in rows for a plant whose trunk bends), then
/// the parts, each with a rim of the parts beside it. Not `exact` (art
/// that is filtered), a part takes one texel of whatever is beside it
/// too, so filtering leaves no seam between two parts at rest.
fn pieces(
    f: &Field,
    img: &Image,
    solid: &[bool],
    label: &[u32],
    plant: Plant,
    r: Rows,
    exact: bool,
) -> Vec<Piece> {
    let px = |i: usize| -> [u8; 4] { img.rgba[i * 4..i * 4 + 4].try_into().expect("four bytes") };
    let count = label.iter().copied().max().unwrap_or(0) as usize;
    // The crown's dark copy: under the parts, in from the outline.
    let reach = scaled(r.height(), 2.0, 2.0).round() as u32;
    let (edge, _) = f.steps(|i| !solid[i], false, true);
    let base_colour = |i: usize| -> [u8; 4] {
        let p = px(i);
        if label[i] == 0 {
            return p;
        }
        let shade = |c: u8| (f32::from(c) * INSIDE_SHADE) as u8;
        [shade(p[0]), shade(p[1]), shade(p[2]), p[3]]
    };
    let in_base = |i: usize| solid[i] && (label[i] == 0 || edge[i] > reach);
    let mut out = Vec::new();
    // A broadleaf's dark copy goes with its crown (its lean is set below).
    let mut copy = None;
    match plant {
        Plant::Clumps => {
            out.extend(piece(f, |i| solid[i] && label[i] == 0, px));
            if let Some(p) = piece(f, |i| in_base(i) && label[i] != 0, base_colour) {
                copy = Some(out.len());
                out.push(p);
            }
        }
        Plant::Tiers => {
            // Rows of the trunk slide a whole texel apart at most while
            // the bands are this thin.
            let band = (r.height() / 16).max(2);
            let mut y = 0;
            while y < f.h {
                let rows = y..(y + band).min(f.h);
                let p = piece(f, |i| in_base(i) && rows.contains(&(i / f.w)), base_colour);
                if let Some(mut p) = p {
                    p.lean = r.up(p.centre[1]).powi(2);
                    out.extend(Some(p));
                }
                y += band;
            }
        }
    }
    let first = out.len();
    // A part holds a rim of its neighbours' texels too (opaque ones, so
    // two parts never add up where they lie on one another): moved a
    // texel or two apart they still meet, and the dark copy shows only
    // where they part further.
    let rim = scaled(r.height(), 3.0, 2.0).round() as u32;
    for k in 1..=count as u32 {
        let own = |i: usize| label[i] == k;
        let centre = piece(f, own, px).map(|p| p.centre);
        let (to_own, _) = f.steps(own, true, false);
        let p = piece(
            f,
            |i| {
                own(i)
                    || (solid[i] && label[i] != 0 && to_own[i] <= rim && px(i)[3] == 255)
                    || (!exact && solid[i] && to_own[i] <= 1)
            },
            px,
        );
        if let (Some(mut p), Some(c)) = (p, centre) {
            p.centre = c;
            out.push(p);
        }
    }
    // Stiff inside, loose outside: by how far a part's centre is from the
    // middle of them all.
    let parts = &mut out[first..];
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in parts.iter() {
        for a in 0..2 {
            lo[a] = lo[a].min(p.centre[a]);
            hi[a] = hi[a].max(p.centre[a]);
        }
    }
    let alone = parts.len() == 1;
    for p in parts.iter_mut() {
        let d: f32 = (0..2)
            .map(|a| {
                let half = ((hi[a] - lo[a]) / 2.0).max(1.0);
                ((p.centre[a] - (lo[a] + hi[a]) / 2.0) / half).powi(2)
            })
            .sum::<f32>()
            .sqrt();
        let loose = if alone {
            1.0
        } else {
            INNER + (1.0 - INNER) * d.min(1.0)
        };
        p.flutter = loose;
        p.lean = match plant {
            Plant::Clumps => loose,
            Plant::Tiers => r.up(p.centre[1]).powi(2),
        };
    }
    if let Some(k) = copy {
        let most = out[first..].iter().map(|p| p.lean).fold(0.0, f32::max);
        out[k].lean = most * COPY_LEAD;
    }
    out
}

/// The pieces packed in rows into one image, and their table.
fn pack(f: &Field, pieces: Vec<Piece>) -> (Image, SwayParts) {
    let area: usize = pieces
        .iter()
        .map(|p| (p.size[0] + GUTTER) * (p.size[1] + GUTTER))
        .sum();
    let widest = pieces.iter().map(|p| p.size[0]).max().unwrap_or(1) + GUTTER;
    let limit = ((area as f64).sqrt().ceil() as usize).max(widest);
    // The tallest first, row by row.
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by_key(|&k| (Reverse(pieces[k].size[1]), k));
    let mut place = vec![[0usize; 2]; pieces.len()];
    let (mut x, mut y, mut row, mut width) = (0, 0, 0, 0);
    for k in order {
        let s = pieces[k].size;
        if x > 0 && x + s[0] + GUTTER > limit {
            (x, y, row) = (0, y + row, 0);
        }
        place[k] = [x, y];
        x += s[0] + GUTTER;
        row = row.max(s[1] + GUTTER);
        width = width.max(x);
    }
    let (w, h) = (width.max(1), (y + row).max(1));
    let mut rgba = vec![0u8; w * h * 4];
    let mut parts = Vec::with_capacity(pieces.len());
    for (p, at) in pieces.iter().zip(&place) {
        for r in 0..p.size[1] {
            let src = &p.rgba[r * p.size[0] * 4..(r + 1) * p.size[0] * 4];
            let o = ((at[1] + r) * w + at[0]) * 4;
            rgba[o..o + src.len()].copy_from_slice(src);
        }
        parts.push(SwayPart {
            atlas: at.map(|v| v as u32),
            size: p.size.map(|v| v as u32),
            at: p.at.map(|v| v as u32),
            centre: p.centre,
            lean: p.lean,
            flutter: p.flutter,
        });
    }
    (
        Image {
            width: w as u32,
            height: h as u32,
            rgba,
        },
        SwayParts {
            size: [f.w as u32, f.h as u32],
            parts,
        },
    )
}

/// A texture cut into its wind parts: the `#sway` image and its table.
/// `map` is the parts map when the setting names one; `exact` when the
/// texture is pixel art.
pub fn split(
    img: &Image,
    plant: Plant,
    map: Option<&Image>,
    exact: bool,
) -> Result<(Image, SwayParts), String> {
    let f = Field {
        w: img.width as usize,
        h: img.height as usize,
    };
    let solid: Vec<bool> = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| p[3] > 0)
        .collect();
    let (r, widths) = rows(&f, &solid).ok_or("sway: the image has no opaque texel")?;
    let label = match (map, plant) {
        (Some(map), _) => mapped(&f, &solid, map)?,
        (None, Plant::Tiers) => tiers(&f, img, &solid, r, &widths),
        (None, Plant::Clumps) => {
            // Fewer, larger clumps when there are too many to draw.
            let mut apart = scaled(r.height(), 5.0, 2.0);
            loop {
                let label = clumps(&f, img, &solid, r, apart);
                let mut seen: Vec<u32> = label.iter().copied().filter(|l| *l != 0).collect();
                seen.sort_unstable();
                seen.dedup();
                if seen.len() <= MAX_PARTS {
                    // Number the clumps left 1, 2, 3 and on.
                    break label
                        .iter()
                        .map(|l| seen.binary_search(l).map_or(0, |k| k as u32 + 1))
                        .collect();
                }
                apart *= 1.5;
            }
        }
    };
    if label.iter().all(|l| *l == 0) {
        return Err(match (map.is_some(), plant) {
            (true, _) => "sway: the parts map marks none of the texture's opaque texels".into(),
            (false, Plant::Clumps) => "sway clumps: no clump of leaves found over the stem (a conifer is `sway tiers`; a parts map names them by hand: sway clumps \"parts.png\")".into(),
            (false, Plant::Tiers) => "sway tiers: no crown found over the stem (a parts map names the tiers by hand: sway tiers \"parts.png\")".into(),
        });
    }
    Ok(pack(&f, pieces(&f, img, &solid, &label, plant, r, exact)))
}

/// The parts drawn in order where they lie at rest: the texture again.
#[cfg(test)]
fn rest(atlas: &Image, parts: &SwayParts) -> Image {
    let (w, h) = (parts.size[0] as usize, parts.size[1] as usize);
    let mut rgba = vec![0u8; w * h * 4];
    for p in &parts.parts {
        for y in 0..p.size[1] as usize {
            for x in 0..p.size[0] as usize {
                let s =
                    ((p.atlas[1] as usize + y) * atlas.width as usize + p.atlas[0] as usize + x)
                        * 4;
                if atlas.rgba[s + 3] > 0 {
                    let o = ((p.at[1] as usize + y) * w + p.at[0] as usize + x) * 4;
                    rgba[o..o + 4].copy_from_slice(&atlas.rgba[s..s + 4]);
                }
            }
        }
    }
    Image {
        width: w as u32,
        height: h as u32,
        rgba,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture from rows of characters: `.` clear, else `colour`'s.
    fn picture(rows: &[&str], colour: impl Fn(char, usize, usize) -> [u8; 4]) -> Image {
        let (w, h) = (rows[0].len(), rows.len());
        let mut rgba = Vec::with_capacity(w * h * 4);
        for (y, row) in rows.iter().enumerate() {
            assert_eq!(row.len(), w, "row {y}");
            for (x, c) in row.chars().enumerate() {
                rgba.extend(if c == '.' { [0; 4] } else { colour(c, x, y) });
            }
        }
        Image {
            width: w as u32,
            height: h as u32,
            rgba,
        }
    }

    /// A broadleaf tree: a brown trunk drawn with a near-black line down
    /// each side under five round masses of leaves (left, right, top,
    /// bottom and one in the middle), each brightest at its middle, with a
    /// dark brown outline (the bottom one's the trunk's line: one line
    /// round the crown and down the trunk).
    fn oak() -> Image {
        let (w, h) = (48usize, 54usize);
        let blobs = [
            (10.0f32, 16.0f32),
            (38.0, 16.0),
            (24.0, 5.0),
            (24.0, 27.0),
            (24.0, 16.0),
        ];
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let o = (y * w + x) * 4;
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let (near, mass) = blobs
                    .iter()
                    .map(|(bx, by)| ((fx - bx).powi(2) + (fy - by).powi(2)).sqrt())
                    .zip(0..)
                    .fold((f32::MAX, 0), |a, b| if b.0 < a.0 { b } else { a });
                if near < 8.5 {
                    // Leaves: green, darker from the middle out; the rim dark.
                    let c = if near > 7.5 {
                        if mass == 3 {
                            [20, 16, 28, 255]
                        } else {
                            [50, 30, 12, 255]
                        }
                    } else {
                        [30, (220.0 - 16.0 * near) as u8, 40, 255]
                    };
                    rgba[o..o + 4].copy_from_slice(&c);
                } else if (20..28).contains(&x) && (30..54).contains(&y) {
                    let c = if x == 20 || x == 27 {
                        [20, 16, 28, 255]
                    } else {
                        [110, 70, 30, 255]
                    };
                    rgba[o..o + 4].copy_from_slice(&c);
                }
            }
        }
        Image {
            width: w as u32,
            height: h as u32,
            rgba,
        }
    }

    /// A conifer: three tiers, each widening down to its hem, on a stem.
    fn pine() -> Image {
        let mut rows: Vec<String> = Vec::new();
        for tier in 0..3 {
            for k in 0..10 {
                let half = 2 + tier * 3 + k;
                let side = 20 - half;
                // The row under each hem is dark: the shadow of the tier above.
                let c = if k == 0 && tier > 0 { 's' } else { 'g' };
                rows.push(format!(
                    "{}{}{}",
                    ".".repeat(side),
                    c.to_string().repeat(half * 2),
                    ".".repeat(side)
                ));
            }
        }
        for _ in 0..8 {
            rows.push(format!("{}tt{}", ".".repeat(19), ".".repeat(19)));
        }
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        picture(&rows, |c, _, y| match c {
            'g' => [20, 90 + (y % 10) as u8 * 12, 30, 255],
            's' => [8, 30, 12, 255],
            _ => [100, 60, 25, 255],
        })
    }

    #[test]
    fn the_setting_names_a_plant_and_its_map() {
        let parse = |text: &str| {
            let doc = og_core::ogt::Document::parse(&format!(
                "type import\nversion 1\n---\ntexture {{\n  {text}\n}}\n"
            ))
            .unwrap();
            setting(doc.body.get("texture").and_then(|e| e.block.as_ref()))
        };
        assert_eq!(parse("kind pixel"), Ok(None));
        assert_eq!(
            parse("sway clumps"),
            Ok(Some(Setting {
                plant: Plant::Clumps,
                map: None
            }))
        );
        assert_eq!(
            parse("sway tiers \"pine_parts.png\""),
            Ok(Some(Setting {
                plant: Plant::Tiers,
                map: Some("pine_parts.png".into())
            }))
        );
        for bad in ["sway", "sway true", "sway \"parts.png\"", "sway clumps 3"] {
            assert!(parse(bad).unwrap_err().contains("sway takes"), "{bad}");
        }
    }

    /// Ways this could fail: a texel lost or changed (the tree at rest not
    /// the texture), the trunk or its drawn line cut into the clumps (it
    /// would sway), the five masses read as one or as crumbs, the crown's
    /// outline left on the wood (it would stay behind its clump), a mass
    /// cut where the crown narrows to the trunk, the middle clump as loose
    /// as the rim, two clumps meeting edge to edge (a texel apart they
    /// would show a crack), or the dark copy left standing (the crown would
    /// lean off it and show it upwind).
    #[test]
    fn a_broadleaf_tree_parts_into_its_clumps_over_still_wood() {
        let img = oak();
        let (atlas, parts) = split(&img, Plant::Clumps, None, true).unwrap();
        assert_eq!(rest(&atlas, &parts).rgba, img.rgba);
        assert_eq!(parts.size, [48, 54]);
        // The wood, the crown's dark copy, then the five clumps.
        assert_eq!(parts.parts.len(), 7, "{:?}", parts.parts);
        let wood = &parts.parts[0];
        assert_eq!((wood.lean, wood.flutter), (0.0, 0.0));
        assert_eq!(wood.at[1] + wood.size[1], 54);
        // The trunk with the line down each side.
        assert_eq!((wood.at[0], wood.size[0]), (20, 8), "{wood:?}");
        let clumps = &parts.parts[2..];
        // The copy leans with the crown, past its loosest clump, and walks
        // no loop.
        let copy = &parts.parts[1];
        assert_eq!(copy.flutter, 0.0);
        assert!(clumps.iter().all(|c| copy.lean > c.lean), "{copy:?}");
        let shown = |p: &SwayPart| -> u32 {
            (0..p.size[1])
                .flat_map(|y| (0..p.size[0]).map(move |x| (x, y)))
                .filter(|(x, y)| {
                    atlas.rgba[(((p.atlas[1] + y) * atlas.width + p.atlas[0] + x) * 4 + 3) as usize]
                        > 0
                })
                .count() as u32
        };
        let nearest = |x: f32, y: f32| {
            clumps
                .iter()
                .min_by(|a, b| {
                    let d = |p: &SwayPart| (p.centre[0] - x).powi(2) + (p.centre[1] - y).powi(2);
                    d(a).total_cmp(&d(b))
                })
                .unwrap()
        };
        // The left mass is a disc of radius 8.5 less what the middle one
        // covers, and a rim of the middle one two texels deep: 170 to 270
        // texels, its outline with it out to column 2.
        let left = nearest(10.0, 16.0);
        assert!((170..270).contains(&shown(left)), "{}", shown(left));
        assert_eq!(left.at[0], 2, "{left:?}");
        // Its own texels end where the middle mass is as near (column 17);
        // it reaches two more.
        assert!((18..=21).contains(&(left.at[0] + left.size[0])), "{left:?}");
        // The bottom mass whole, down to its last row (34) over the trunk.
        let bottom = nearest(24.0, 27.0);
        assert_eq!(bottom.at[1] + bottom.size[1], 35, "{bottom:?}");
        // Stiff in the middle, loose at the rim.
        let middle = nearest(24.0, 16.0);
        assert!(middle.lean < 0.5 && left.lean > 0.9, "{middle:?} {left:?}");
        for c in clumps {
            assert!(
                c.lean == c.flutter && (INNER..=1.0).contains(&c.lean),
                "{c:?}"
            );
        }
    }

    /// Ways this could fail: the tiers cut at the wrong rows or into one,
    /// drawn top first (a lower tier over the hem above it), the trunk in
    /// one piece (it could not bend), or the lean not growing with height.
    #[test]
    fn a_conifer_parts_into_tiers_on_a_trunk_in_rows() {
        let img = pine();
        let (atlas, parts) = split(&img, Plant::Tiers, None, true).unwrap();
        assert_eq!(rest(&atlas, &parts).rgba, img.rgba);
        let (rows, tiers): (Vec<&SwayPart>, Vec<&SwayPart>) =
            parts.parts.iter().partition(|p| p.flutter == 0.0);
        assert_eq!(tiers.len(), 3, "{tiers:?}");
        // Bottom up, each its own ten rows (cut within a row of each hem:
        // its texels' centre is that of the rows, wider towards the hem)
        // and a rim of two rows of the tiers beside it.
        for (k, t) in tiers.iter().enumerate() {
            let tier = 2 - k;
            let width = |j: usize| (2 + tier * 3 + j) as f32;
            let centre = (tier * 10) as f32
                + (0..10).map(|j| (j as f32 + 0.5) * width(j)).sum::<f32>()
                    / (0..10).map(width).sum::<f32>();
            assert!((t.centre[1] - centre).abs() < 0.8, "{k} {centre} {t:?}");
            let rows = 10 + if tier == 1 { 4 } else { 2 };
            assert!(t.size[1].abs_diff(rows) <= 2, "{k} {t:?}");
        }
        assert!(tiers[0].lean < tiers[1].lean && tiers[1].lean < tiers[2].lean);
        // The trunk and the dark copy in rows, the foot still, the top not.
        assert!(rows.len() >= 8, "{}", rows.len());
        let foot = rows.iter().max_by_key(|p| p.at[1]).unwrap();
        let tip = rows.iter().min_by_key(|p| p.at[1]).unwrap();
        assert!(foot.lean < 0.01 && tip.lean > 0.6, "{foot:?} {tip:?}");
    }

    #[test]
    fn a_parts_map_names_the_parts_by_colour() {
        let img = oak();
        // Two parts by hand: the left half of the crown red, the right blue.
        let mut map = Image {
            width: 48,
            height: 54,
            rgba: vec![0; 48 * 54 * 4],
        };
        for y in 0..26 {
            for x in 0..48 {
                let c = if x < 24 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                };
                map.rgba[(y * 48 + x) * 4..][..4].copy_from_slice(&c);
            }
        }
        let (atlas, parts) = split(&img, Plant::Clumps, Some(&map), true).unwrap();
        assert_eq!(rest(&atlas, &parts).rgba, img.rgba);
        assert_eq!(parts.parts.len(), 4);
        assert!(parts.parts[2].centre[0] < 24.0 || parts.parts[3].centre[0] < 24.0);
        let small = Image {
            width: 4,
            height: 4,
            rgba: vec![255; 64],
        };
        assert!(
            split(&img, Plant::Clumps, Some(&small), true)
                .err()
                .unwrap()
                .contains("a map is the texture's size")
        );
    }

    /// A filtered texture's parts overlap by a texel, so two parts at rest
    /// leave no seam; drawn in order they are still the texture.
    #[test]
    fn filtered_parts_overlap_and_still_rest_as_the_texture() {
        let img = oak();
        let (exact, a) = split(&img, Plant::Clumps, None, true).unwrap();
        let (soft, b) = split(&img, Plant::Clumps, None, false).unwrap();
        assert_eq!(rest(&soft, &b).rgba, img.rgba);
        let texels = |i: &Image| {
            i.rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count()
        };
        assert!(texels(&soft) > texels(&exact));
        assert_eq!(a.parts.len(), b.parts.len());
    }

    #[test]
    fn an_image_with_no_clump_says_what_to_do() {
        // Four texels: a crumb, no clump of leaves.
        let crumb = picture(&["gg", "gg"], |_, _, _| [30, 160, 40, 255]);
        let e = split(&crumb, Plant::Clumps, None, true).err().unwrap();
        assert!(e.contains("no clump of leaves"), "{e}");
    }
}
