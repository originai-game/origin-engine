//! A pixel font's glyphs on its grid (`font { pixel <grid> }`): the cells
//! its outline fills, and lengths snapped to whole cells, for the raster
//! to draw a pixel face at whole multiples of its grid.

/// Line segments of a glyph's outline in font units (y up), curves
/// flattened into a fixed number of pieces.
#[derive(Default)]
struct Flatten {
    scale: f32,
    tx: f32,
    ty: f32,
    start: (f32, f32),
    at: (f32, f32),
    segments: Vec<[f32; 4]>,
}

/// Pieces per quadratic and cubic curve: under a tenth of a pixel from the
/// curve at 32 px per em.
const QUAD_STEPS: u32 = 8;
const CUBIC_STEPS: u32 = 12;

impl Flatten {
    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.scale + self.tx, y * self.scale + self.ty)
    }

    fn line(&mut self, to: (f32, f32)) {
        if to != self.at {
            self.segments.push([self.at.0, self.at.1, to.0, to.1]);
        }
        self.at = to;
    }
}

impl ttf_parser::OutlineBuilder for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.start = p;
        self.at = p;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        self.line(p);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0, p1, p2) = (self.at, self.point(x1, y1), self.point(x, y));
        for i in 1..=QUAD_STEPS {
            let t = i as f32 / QUAD_STEPS as f32;
            let u = 1.0 - t;
            let q = (
                u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
            );
            self.line(if i == QUAD_STEPS { p2 } else { q });
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0, p1, p2, p3) = (
            self.at,
            self.point(x1, y1),
            self.point(x2, y2),
            self.point(x, y),
        );
        for i in 1..=CUBIC_STEPS {
            let t = i as f32 / CUBIC_STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            let q = (
                a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
                a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
            );
            self.line(if i == CUBIC_STEPS { p3 } else { q });
        }
    }

    fn close(&mut self) {
        let s = self.start;
        self.line(s);
    }
}

/// An em length snapped to whole cells of a `grid`-cell em (a pixel font's
/// advance or kerning); a non-zero length keeps at least a cell.
pub fn snap_to_grid(em: f32, grid: u32) -> f32 {
    let g = grid as f32;
    let cells = (em * g).round();
    let cells = if cells == 0.0 && em.abs() * g >= 0.25 {
        em.signum()
    } else {
        cells
    };
    cells / g
}

/// A pixel font's glyph on its grid: the cells (`grid` to the em, cell
/// (0, 0) right of the pen and above the baseline) whose centre its
/// outline fills by the nonzero rule. Sampling at the centre puts a face
/// drawn on a pitch a little off its nominal grid (DotGothic16's dots)
/// on whole cells, and a face drawn on the grid exactly as it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelCells {
    /// The first column and row (cells; y up).
    pub x0: i32,
    pub y0: i32,
    pub width: u32,
    pub height: u32,
    /// Row by row from the bottom: filled.
    pub filled: Vec<bool>,
}

/// The cells of glyph `gid` of `face` on a `grid`-cell em ([`PixelCells`]).
pub fn pixel_cells(face: &ttf_parser::Face, gid: ttf_parser::GlyphId, grid: u32) -> PixelCells {
    let empty = PixelCells {
        x0: 0,
        y0: 0,
        width: 0,
        height: 0,
        filled: Vec::new(),
    };
    let Some(bbox) = face.glyph_bounding_box(gid) else {
        return empty;
    };
    let cell = f32::from(face.units_per_em()) / grid.max(1) as f32;
    let mut outline = Flatten {
        scale: 1.0,
        tx: 0.0,
        ty: 0.0,
        start: (0.0, 0.0),
        at: (0.0, 0.0),
        segments: Vec::new(),
    };
    if face.outline_glyph(gid, &mut outline).is_none() {
        return empty;
    }
    let x0 = (f32::from(bbox.x_min) / cell).floor() as i32;
    let y0 = (f32::from(bbox.y_min) / cell).floor() as i32;
    let x1 = (f32::from(bbox.x_max) / cell).ceil() as i32;
    let y1 = (f32::from(bbox.y_max) / cell).ceil() as i32;
    let (width, height) = ((x1 - x0).max(0) as u32, (y1 - y0).max(0) as u32);
    let mut filled = vec![false; (width * height) as usize];
    for j in 0..height {
        let py = (y0 + j as i32) as f32 * cell + cell / 2.0;
        for i in 0..width {
            let px = (x0 + i as i32) as f32 * cell + cell / 2.0;
            let mut winding = 0i32;
            for s in &outline.segments {
                let [ax, ay, bx, by] = *s;
                if (ay <= py) != (by <= py) {
                    let x = ax + (py - ay) / (by - ay) * (bx - ax);
                    if x > px {
                        winding += if by > ay { 1 } else { -1 };
                    }
                }
            }
            filled[(j * width + i) as usize] = winding != 0;
        }
    }
    PixelCells {
        x0,
        y0,
        width,
        height,
        filled,
    }
}

impl PixelCells {
    fn at(&self, i: i32, j: i32) -> bool {
        i >= 0
            && j >= 0
            && (i as u32) < self.width
            && (j as u32) < self.height
            && self.filled[(j as u32 * self.width + i as u32) as usize]
    }

    /// The box of the filled cells, in cells: left, bottom, right, top
    /// (`None`: no cell filled).
    pub fn bounds(&self) -> Option<[i32; 4]> {
        let mut b: Option<(i32, i32, i32, i32)> = None;
        for j in 0..self.height as i32 {
            for i in 0..self.width as i32 {
                if self.at(i, j) {
                    let (x, y) = (self.x0 + i, self.y0 + j);
                    b = Some(match b {
                        None => (x, y, x + 1, y + 1),
                        Some((a, c, d, e)) => (a.min(x), c.min(y), d.max(x + 1), e.max(y + 1)),
                    });
                }
            }
        }
        let (a, c, d, e) = b?;
        Some([a, c, d, e])
    }

    /// The outline of the filled cells: one closed contour per boundary,
    /// counter-clockwise (y up) around filled cells and clockwise around
    /// holes, so the nonzero rule fills the cells and only them. Corners in
    /// cell units; straight runs are one segment. Cells that meet at a
    /// corner only are separate contours.
    pub fn contours(&self) -> Vec<Vec<(i32, i32)>> {
        use std::collections::BTreeMap;
        // Directed unit edges with the filled cell on their left.
        let mut out: BTreeMap<(i32, i32), Vec<(i32, i32)>> = BTreeMap::new();
        for j in 0..self.height as i32 {
            for i in 0..self.width as i32 {
                if !self.at(i, j) {
                    continue;
                }
                let (x, y) = (self.x0 + i, self.y0 + j);
                let mut edge = |a: (i32, i32), b: (i32, i32)| {
                    out.entry(a).or_default().push(b);
                };
                if !self.at(i, j - 1) {
                    edge((x, y), (x + 1, y));
                }
                if !self.at(i + 1, j) {
                    edge((x + 1, y), (x + 1, y + 1));
                }
                if !self.at(i, j + 1) {
                    edge((x + 1, y + 1), (x, y + 1));
                }
                if !self.at(i - 1, j) {
                    edge((x, y + 1), (x, y));
                }
            }
        }
        let mut contours = Vec::new();
        while let Some((&start, _)) = out.iter().find(|(_, v)| !v.is_empty()) {
            let mut pts = vec![start];
            let mut at = start;
            let mut dir = (0, 0);
            loop {
                let next = out
                    .get_mut(&at)
                    .expect("an edge leaves every corner reached");
                // Where two leave one corner (cells meeting at it), the left
                // turn: each cell's boundary closes on itself.
                let k = if next.len() > 1 {
                    let left = (-dir.1, dir.0);
                    next.iter()
                        .position(|&b| (b.0 - at.0, b.1 - at.1) == left)
                        .unwrap_or(0)
                } else {
                    0
                };
                let to = next.swap_remove(k);
                dir = (to.0 - at.0, to.1 - at.1);
                at = to;
                if at == start {
                    break;
                }
                pts.push(at);
            }
            // Corners only: drop the points inside a straight run.
            let n = pts.len();
            let corners: Vec<(i32, i32)> = (0..n)
                .filter(|&k| {
                    let (a, b, c) = (pts[(k + n - 1) % n], pts[k], pts[(k + 1) % n]);
                    (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0) != 0
                })
                .map(|k| pts[k])
                .collect();
            contours.push(corners);
        }
        contours
    }

    /// Feeds [`PixelCells::contours`] to `b` in font units of `cell` per
    /// cell.
    pub fn outline(&self, cell: f32, b: &mut impl ttf_parser::OutlineBuilder) {
        for c in self.contours() {
            let p = |(x, y): (i32, i32)| (x as f32 * cell, y as f32 * cell);
            let (sx, sy) = p(c[0]);
            b.move_to(sx, sy);
            for &q in &c[1..] {
                let (x, y) = p(q);
                b.line_to(x, y);
            }
            b.close();
        }
    }
}
