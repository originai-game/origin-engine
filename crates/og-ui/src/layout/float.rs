//! CSS 2 §9.5 floats and CSS Shapes 1 contours: what Taffy's float
//! context does not hold. Taffy places floats and the boxes beside them
//! (`layout::css`); a float's `shape-outside` narrows the lines beside it
//! by its contour, as dilated by `shape-margin` and clipped to its margin
//! box, while float placement, clearance and boxes that start a context of
//! their own keep its margin box (§2). A rich text beside floats breaks
//! one line at a time at each line's room.

use super::*;
use crate::rich::{RichContent, RichLine, RichSpan};

/// `clear: left` (and `both`).
pub const LEFT: u8 = 1;
/// `clear: right` (and `both`).
pub const RIGHT: u8 = 2;

/// A block container: its children lie in a block flow, or in lines, and
/// floats leave it (in a flex, grid, table or multi-column container
/// `float` does not apply).
pub(crate) fn container(n: &Node) -> bool {
    n.layout.used.block
        && n.layout_mode() == LayoutMode::Flex
        && matches!(*n.content, Content::None)
        && n.layout.used.multicol.is_none()
}

/// An image's alpha channel on the CPU: one byte per pixel, row-major,
/// `width` x `height` ([`crate::Ui::define_texture_alpha`]). A
/// `shape-outside: url()` float's contour is read from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageAlpha {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
}

/// `shape-outside: <image>` with its `shape-image-threshold`.
#[derive(Clone, Debug)]
pub struct ShapeImage {
    pub alpha: std::sync::Arc<ImageAlpha>,
    pub threshold: f32,
}

impl PartialEq for ShapeImage {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.alpha, &other.alpha) && self.threshold == other.threshold
    }
}

impl ShapeImage {
    /// The area of the pixels whose alpha is greater than the threshold
    /// (CSS Shapes 1 §6.2), the image stretched over the `w` x `h` box at
    /// `at`: each run of them in a row as its two vertical sides, runs
    /// repeated by the rows below merged into one.
    fn segments(&self, at: [f32; 2], w: f32, h: f32, out: &mut Vec<[[f32; 2]; 2]>) {
        let a = &*self.alpha;
        let (iw, ih) = (a.width as usize, a.height as usize);
        if iw == 0 || ih == 0 || a.alpha.len() < iw * ih {
            return;
        }
        let cut = self.threshold * 255.0;
        let runs = |j: usize| {
            let row = &a.alpha[j * iw..(j + 1) * iw];
            let mut runs = Vec::new();
            let mut k = 0;
            while k < iw {
                if f32::from(row[k]) > cut {
                    let start = k;
                    while k < iw && f32::from(row[k]) > cut {
                        k += 1;
                    }
                    runs.push((start, k));
                } else {
                    k += 1;
                }
            }
            runs
        };
        let (sx, sy) = (w / iw as f32, h / ih as f32);
        let mut j = 0;
        let mut row = runs(0);
        while j < ih {
            let mut end = j + 1;
            let mut next = Vec::new();
            while end < ih {
                next = runs(end);
                if next != row {
                    break;
                }
                end += 1;
            }
            // The rows cover [y0, y1): a line starting at y1 is below them.
            let y0 = at[1] + j as f32 * sy;
            let y1 = (at[1] + end as f32 * sy - 1.0 / 256.0).max(y0);
            for &(s, e) in &row {
                for x in [at[0] + s as f32 * sx, at[0] + e as f32 * sx] {
                    out.push([[x, y0], [x, y1]]);
                }
            }
            row = next;
            j = end;
        }
    }
}

/// Unresolved CSS shape geometry; layout resolves percentages after sizing.
#[derive(Clone, Debug, PartialEq)]
pub struct Outside {
    pub shape: crate::css::paint_css::CssClip,
    pub reference: crate::css::paint_css::GeoBox,
    /// An image's contour, in place of `shape`, over the content box.
    pub image: Option<ShapeImage>,
    pub margin: crate::css::paint_css::Pos,
    pub rtl: bool,
}

impl Outside {
    /// Its contour's edges for float `n`, its border box `size` (in its
    /// frame) with `margins` (top, right, bottom, left) at `offset`.
    pub(crate) fn edges(
        &self,
        n: &Node,
        size: [f32; 2],
        margins: [f32; 4],
        offset: [f32; 2],
        margin: f32,
    ) -> Vec<[[f32; 2]; 2]> {
        use crate::css::paint_css::GeoBox;
        use crate::paint::ShapeKind;
        use og_core::glam::Vec2;
        let vertical = n.layout.used.vertical;
        let physical = |values: [f32; 4]| {
            let mut out = values;
            if let Some(sides) = vertical {
                for k in 0..4 {
                    out[sides[k] as usize] = values[k];
                }
            }
            out
        };
        let p = n.layout.used.padding;
        // An image lies over the content box as a replaced element's
        // picture would, stretched to it (CSS Shapes 1 §6.1).
        let reference = if self.image.is_some() {
            GeoBox::Content
        } else {
            self.reference
        };
        let insets = match reference {
            GeoBox::Border => [0.0; 4],
            GeoBox::Padding => n
                .decor
                .as_ref()
                .map_or([0.0; 4], |d| d.borders.map(|b| b.map_or(0.0, |b| b.width))),
            GeoBox::Content => physical([p.top, p.right, p.bottom, p.left]),
            GeoBox::Margin => margins.map(|m| -m),
        };
        let radius = if n.layout.used.radius > 0.0 {
            (n.layout.used.radius - insets.iter().copied().fold(f32::INFINITY, f32::min)).max(0.0)
        } else {
            0.0
        };
        let (w, h) = if vertical.is_some() {
            (size[1], size[0])
        } else {
            (size[0], size[1])
        };
        let map = |p: Vec2| {
            let (mut x, y) = match vertical {
                Some([1, _, _, _]) => (p.y, w - p.x),
                Some([3, 0, _, _]) => (h - p.y, p.x),
                Some(_) => (p.y, p.x),
                None => (p.x, p.y),
            };
            if self.rtl {
                x = size[0] - x;
            }
            [x + offset[0], y + offset[1]]
        };
        let mut edges = Vec::new();
        if let Some(image) = &self.image {
            let [t, r, b, l] = insets;
            let mut sides = Vec::new();
            image.segments(
                [l, t],
                (w - l - r).max(0.0),
                (h - t - b).max(0.0),
                &mut sides,
            );
            edges.extend(
                sides
                    .into_iter()
                    .map(|[a, b]| [map(Vec2::from(a)), map(Vec2::from(b))]),
            );
        }
        let (points, contours) = match self.image {
            Some(_) => (Vec::new(), Vec::new()),
            None => match self.shape.resolve(w, h, (insets, radius)) {
                ShapeKind::Path {
                    points, contours, ..
                } => (points, contours),
                _ => return Vec::new(),
            },
        };
        let mut start = 0;
        for count in contours {
            let contour = &points[start..start + count as usize];
            start += count as usize;
            if contour.is_empty() {
                continue;
            }
            let mut at = Vec2::from(contour[0]);
            for c in contour[1..].as_chunks::<3>().0 {
                let line = og_draw::flatten_cubic(at, c[0].into(), c[1].into(), c[2].into(), 0.01);
                edges.extend(line.windows(2).map(|p| [map(p[0]), map(p[1])]));
                at = c[2].into();
            }
            edges.push([map(at), map(contour[0].into())]);
        }
        let (top, bottom) = edges
            .iter()
            .flatten()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(top, bottom), p| {
                (top.min(p[1]), bottom.max(p[1]))
            });
        if top == bottom && margin == 0.0 {
            edges.clear();
        }
        edges
    }
}

/// A placed float's margin box, in the box its lines lie in.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placed {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub left: bool,
    pub contour: Option<Vec<[[f32; 2]; 2]>>,
    pub margin: f32,
}

/// The horizontal extent of a segment swept by a disk, intersected with
/// a line band. Endpoints, band crossings and circle tangencies are the
/// only extrema; expanding its bounding box would give square joins.
fn expanded_edge(a: [f32; 2], b: [f32; 2], lo: f32, hi: f32, margin: f32) -> (f32, f32) {
    let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut include = |p: [f32; 2]| {
        let distance = (lo - p[1]).max(p[1] - hi).max(0.0);
        if distance <= margin {
            let radius = (margin * margin - distance * distance).max(0.0).sqrt();
            left = left.min(p[0] - radius);
            right = right.max(p[0] + radius);
        }
    };
    include(a);
    include(b);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    if dy != 0.0 {
        let tangent = margin * dx * dy.signum() / dx.hypot(dy);
        for y in [
            lo - margin,
            lo,
            lo + margin,
            hi - margin,
            hi,
            hi + margin,
            lo - tangent,
            lo + tangent,
            hi - tangent,
            hi + tangent,
        ] {
            let t = (y - a[1]) / dy;
            if (0.0..=1.0).contains(&t) {
                include([a[0] + t * dx, y]);
            }
        }
    }
    (left, right)
}

/// Only inline content uses the contour. Float placement, BFC avoidance
/// and clearance keep the rectangular margin box (CSS Shapes 1 §2).
pub(crate) fn line_room(
    floats: &[Placed],
    y0: f32,
    y1: f32,
    x0: f32,
    x1: f32,
) -> (f32, f32, Option<f32>) {
    let (mut l, mut r, mut next) = (x0, x1, None::<f32>);
    for f in floats {
        let (lo, hi) = (y0.max(f.y), y1.min(f.y + f.h));
        if hi <= lo {
            continue;
        }
        let (mut left, mut right) = (f.x, f.x + f.w);
        if let Some(edges) = &f.contour {
            (left, right) = (f32::INFINITY, f32::NEG_INFINITY);
            for &[a, b] in edges {
                if a[1].max(b[1]) + f.margin < lo || a[1].min(b[1]) - f.margin > hi {
                    continue;
                }
                let (a, b) = expanded_edge(a, b, lo, hi, f.margin);
                left = left.min(a);
                right = right.max(b);
            }
        }
        if left > right {
            continue;
        }
        let left = left.clamp(f.x, f.x + f.w.max(0.0));
        let right = right.clamp(f.x, f.x + f.w.max(0.0));
        if !(if f.left { right > x0 } else { left < x1 }) {
            continue;
        }
        if f.left {
            l = l.max(right);
        } else {
            r = r.min(left);
        }
        let bottom = f.y + f.h;
        let b = if f.contour.is_some() {
            (y0 + 1.0 / 64.0).min(bottom)
        } else {
            bottom
        };
        next = Some(next.map_or(b, |n| n.min(b)));
    }
    (l, r, next)
}

/// A rich text's lines beside floats from its content box's top
/// (`room_at`: a line's left and width from its content box's left, and
/// the floats' next bottom; `lowest`, the floats' bottom): its paragraph
/// shaped once where it can be, else broken one span run at a time.
pub(crate) fn rich_lines_around(
    rc: &RichContent,
    fonts: &dyn Metrics,
    lowest: f32,
    v: Option<crate::layout::flow::VerticalText>,
    room_at: impl Fn(f32, f32) -> (f32, f32, Option<f32>),
) -> Broken {
    #[cfg(feature = "shaping")]
    if v.is_none()
        && let Some(b) = shaped_lines(rc, fonts, 0.0, &room_at)
    {
        return b;
    }
    span_lines(rc, fonts, 0.0, lowest, room_at, v)
}

/// A rich text's lines beside floats from its top `y` (`room_at`: a
/// line's left, width and the floats' next bottom): the lines, each one's
/// left, drop and width, whether a float narrowed one, and the drop.
pub(crate) type Broken = (Vec<RichLine>, Vec<[f32; 3]>, bool, f32);

/// [`rich`]'s lines from its paragraph shaped once
/// ([`crate::text::paragraph::Breaker`]): bidi and soft hyphens see the
/// whole paragraph. `None` when it cannot be shaped.
#[cfg(feature = "shaping")]
fn shaped_lines(
    rc: &RichContent,
    fonts: &dyn Metrics,
    mut y: f32,
    room_at: impl Fn(f32, f32) -> (f32, f32, Option<f32>),
) -> Option<Broken> {
    let mut breaker = crate::text::paragraph::Breaker::new(rc, fonts)?;
    let guess = rc.base.0.line_height;
    let (mut lines, mut shifts) = (Vec::new(), Vec::new());
    let (mut drop, mut narrowed) = (0.0f32, false);
    loop {
        // The room beside the line as tall as the line it holds.
        let mut h = guess;
        let (step, got, l, avail, next) = loop {
            let (l, avail, next) = room_at(y, h);
            let step = breaker.next(avail);
            let got = breaker.lines(&step, fonts)?;
            let height: f32 = got.iter().map(|l| l.height).sum();
            if height <= h + 0.01 || next.is_none() {
                break (step, got, l, avail, next);
            }
            h = height;
        };
        let width = got.iter().map(|l| l.width).fold(0.0, f32::max);
        if let Some(b) = next
            && width > avail + 0.5
        {
            drop += b - y;
            y = b;
            continue;
        }
        narrowed |= next.is_some();
        for line in got {
            y += line.height;
            shifts.push([l, drop, avail]);
            lines.push(line);
        }
        if breaker.take(&step) {
            break;
        }
    }
    Some((lines, shifts, narrowed, drop))
}

/// [`rich`]'s lines where its faces cannot be shaped (fixed or atlas-only
/// metrics) or run down a vertical line `v`: each line broken from the
/// spans left; below `lowest` (the floats' bottom) the rest at once.
fn span_lines(
    rc: &RichContent,
    fonts: &dyn Metrics,
    mut y: f32,
    lowest: f32,
    room_at: impl Fn(f32, f32) -> (f32, f32, Option<f32>),
    v: Option<crate::layout::flow::VerticalText>,
) -> Broken {
    let break_at = |rest: &[(usize, RichSpan)], w: f32| {
        let part = RichContent {
            spans: rest.iter().map(|s| s.1.clone()).collect(),
            ..rc.clone()
        };
        crate::rich::rich_lines(&part, fonts, Some(w + 0.5), v)
    };
    // A line of the spans left, its spans renumbered to the paragraph's.
    let renumber = |rest: &[(usize, RichSpan)], mut l: RichLine| {
        for p in &mut l.pieces {
            p.span = rest[p.span].0;
        }
        for s in l.shaped.iter_mut().flatten() {
            s.span = rest[s.span].0;
        }
        l
    };
    let guess = rc.base.0.line_height;
    let mut rest: Vec<(usize, RichSpan)> = rc.spans.iter().cloned().enumerate().collect();
    let (mut lines, mut shifts) = (Vec::new(), Vec::new());
    let (mut drop, mut narrowed) = (0.0f32, false);
    while !rest.is_empty() {
        if y >= lowest {
            let (l0, full, _) = room_at(y, guess);
            for l in break_at(&rest, full) {
                shifts.push([l0, drop, full]);
                lines.push(renumber(&rest, l));
            }
            break;
        }
        // The room beside the line as tall as the line it holds.
        let mut h = guess;
        let (line, l, avail, next) = loop {
            let (l, avail, next) = room_at(y, h);
            let line = break_at(&rest, avail)
                .into_iter()
                .next()
                .unwrap_or_default();
            if line.height <= h + 0.01 {
                break (line, l, avail, next);
            }
            h = line.height;
        };
        if let Some(b) = next
            && line.width > avail + 0.5
        {
            drop += b - y;
            y = b;
            continue;
        }
        narrowed |= next.is_some();
        // What the line took from each span (an atom whole).
        let mut used = vec![(0usize, false); rest.len()];
        let mut newline = false;
        for p in &line.pieces {
            used[p.span].0 += p.text.chars().count();
            used[p.span].1 = true;
            newline |= p.text.ends_with('\n');
        }
        let lh = line.height;
        lines.push(renumber(&rest, line));
        shifts.push([l, drop, avail]);
        let mut moved = false;
        let mut left = Vec::with_capacity(rest.len());
        for ((id, mut s), (chars, took)) in rest.into_iter().zip(used) {
            if s.atom.is_some() {
                moved |= took;
                if !took {
                    left.push((id, s));
                }
                continue;
            }
            let cut = s
                .text
                .char_indices()
                .nth(chars)
                .map_or(s.text.len(), |(b, _)| b);
            moved |= cut > 0;
            s.text.drain(..cut);
            if !s.text.is_empty() {
                left.push((id, s));
            }
        }
        // The spaces at the break go, and a forced break with them.
        while let Some((_, s)) = left.first_mut()
            && s.atom.is_none()
        {
            let t = s.text.trim_start_matches([' ', '\t']);
            let t = match t.strip_prefix("\r\n").or_else(|| t.strip_prefix('\n')) {
                Some(t) if !newline => {
                    newline = true;
                    t
                }
                _ => t,
            };
            moved |= t.len() != s.text.len();
            s.text = t.to_owned();
            if !s.text.is_empty() {
                break;
            }
            left.remove(0);
        }
        rest = left;
        y += lh;
        if !moved {
            // Nothing more fits a line (cannot happen with text left).
            break;
        }
        if rest.is_empty() && newline {
            let (l, w, _) = room_at(y, guess);
            shifts.push([l, drop, w]);
            lines.push(RichLine {
                height: guess,
                ..RichLine::default()
            });
        }
    }
    (lines, shifts, narrowed, drop)
}
