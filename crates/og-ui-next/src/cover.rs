//! A screen transition's cover: the whole surface covered and revealed by
//! a fade, a wipe, a slide, an iris or a dissolve, drawn over everything
//! the tree paints. A timeline's transition track times it, a function of
//! its time; the tree only draws it (presentation, never the simulation).

use og_draw::paint::{PaintKind, PaintQuad};

/// How a cover covers and reveals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Wipe {
    #[default]
    Fade,
    /// An edge sweeps across from `from`.
    Wipe,
    /// A circle closes on the centre and opens again.
    Iris,
    /// Cells appear in a fixed scattered order.
    Dissolve,
    /// A sheet slides over and away.
    Slide,
}

/// Where a wipe or slide comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

/// One frame's cover.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cover {
    pub kind: Wipe,
    pub from: Side,
    /// How much is covered, 0 clear to 1 covered (eased here).
    pub amount: f32,
    /// The second half: a wipe leaves past the far side.
    pub revealing: bool,
    /// Its colour, linear RGBA.
    pub color: [f32; 4],
}

fn smooth(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    c * c * (3.0 - 2.0 * c)
}

/// A fixed scatter in 0–1 for cell `(x, y)` (an integer hash).
fn scatter(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9e37_79b9) ^ y.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.0
}

impl Cover {
    /// Its quads over a `w` by `h` CSS px surface; reduced motion fades.
    pub fn paint(&self, w: f32, h: f32, reduced_motion: bool, out: &mut Vec<PaintQuad>) {
        let amount = if self.amount.is_nan() {
            0.0
        } else {
            self.amount.clamp(0.0, 1.0)
        };
        if amount <= 0.0 {
            return;
        }
        let e = smooth(amount);
        let screen = [0.0, 0.0, w, h];
        let mut piece = |[x, y, pw, ph]: [f32; 4], alpha: f32| {
            let (x0, y0) = (x.max(0.0), y.max(0.0));
            let (x1, y1) = ((x + pw).min(w), (y + ph).min(h));
            if x1 <= x0 || y1 <= y0 || alpha <= 0.0 {
                return;
            }
            let mut color = self.color;
            color[3] *= alpha;
            out.push(PaintQuad {
                rect: [x0, y0, x1 - x0, y1 - y0],
                clip: screen,
                color,
                radius: [0.0; 4],
                p: [0.0; 4],
                kind: PaintKind::Solid as u32,
                a: 0,
                b: 0,
                xf: 0,
                uv: [0.0; 4],
                wipe: [0.0; 4],
                mask: 0,
                filter: 0,
                round: 0,
            });
        };
        let kind = if reduced_motion {
            Wipe::Fade
        } else {
            self.kind
        };
        match kind {
            Wipe::Fade => piece(screen, e),
            Wipe::Wipe | Wipe::Slide => {
                // Covering grows from the side; revealing leaves past the
                // far side. A slide moves a whole sheet; a wipe an edge.
                let (along, total) = match self.from {
                    Side::Left | Side::Right => (true, w),
                    Side::Top | Side::Bottom => (false, h),
                };
                let size = total * e;
                let start = match (kind, self.revealing) {
                    (Wipe::Wipe, false) => 0.0,
                    (Wipe::Wipe, true) => total - size,
                    (_, false) => size - total,
                    (_, true) => total - size,
                };
                let len = if kind == Wipe::Wipe { size } else { total };
                let flip = matches!(self.from, Side::Right | Side::Bottom);
                let s0 = if flip { total - start - len } else { start };
                piece(
                    if along {
                        [s0, 0.0, len, h]
                    } else {
                        [0.0, s0, w, len]
                    },
                    1.0,
                );
            }
            Wipe::Iris => {
                let (cx, cy) = (w / 2.0, h / 2.0);
                let big = (cx * cx + cy * cy).sqrt() + 2.0;
                let r = big * (1.0 - e);
                let band = 2.0;
                let mut y = 0.0;
                while y < h {
                    let dy = (y + band / 2.0 - cy).abs();
                    if dy >= r {
                        piece([0.0, y, w, band], 1.0);
                    } else {
                        let half = (r * r - dy * dy).sqrt();
                        piece([0.0, y, cx - half, band], 1.0);
                        piece([cx + half, y, w - cx - half, band], 1.0);
                    }
                    y += band;
                }
            }
            Wipe::Dissolve => {
                let cell = 24.0;
                let (cols, rows) = ((w / cell).ceil() as u32, (h / cell).ceil() as u32);
                for y in 0..rows {
                    for x in 0..cols {
                        let a = ((e * 1.25 - scatter(x, y)) * 6.0).clamp(0.0, 1.0);
                        piece([x as f32 * cell, y as f32 * cell, cell, cell], a);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wipe from the right covers the right side first and reveals by
    /// leaving past the left; an iris keeps the centre clear until the end.
    #[test]
    fn a_wipe_and_an_iris_cover_where_they_should() {
        let at = |kind, from, amount, revealing| {
            let mut q = Vec::new();
            let c = Cover {
                kind,
                from,
                amount,
                revealing,
                color: [0.0, 0.0, 0.0, 1.0],
            };
            c.paint(200.0, 100.0, false, &mut q);
            q
        };
        let q = at(Wipe::Wipe, Side::Right, 0.5, false);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].rect, [100.0, 0.0, 100.0, 100.0], "the right half");
        let q = at(Wipe::Wipe, Side::Right, 0.5, true);
        assert_eq!(q[0].rect, [0.0, 0.0, 100.0, 100.0], "leaving past the left");
        let covers = |q: &[PaintQuad], x: f32, y: f32| {
            q.iter().any(|q| {
                let [qx, qy, qw, qh] = q.rect;
                x >= qx && x < qx + qw && y >= qy && y < qy + qh
            })
        };
        let q = at(Wipe::Iris, Side::Left, 0.5, false);
        assert!(covers(&q, 1.0, 1.0) && !covers(&q, 100.0, 50.0));
        assert!(covers(&at(Wipe::Iris, Side::Left, 1.0, false), 100.0, 50.0));
        assert!(at(Wipe::Dissolve, Side::Left, 0.0, false).is_empty());
    }
}
