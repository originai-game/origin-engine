//! The line drawn into a sprite's cells (`ArtLine`): a port of
//! Pixelorama's OutlineInline effect
//! (src/Shaders/Effects/OutlineInline.gdshaderinc at 4ed451de, itself
//! based on godotshaders.com's 2D outline/inline), from a fragment shader
//! to texels. MIT, Copyright (c) 2019-present Orama Interactive and
//! contributors: the notice is third_party/pixelorama/LICENSE.
//!
//! As there: a texel takes the line when it is transparent (outside) or
//! opaque (inside) and a texel of the other kind lies under the brush
//! round it, the brush being a diamond, a circle or a square of the
//! line's width; for the inside line the cell's edge counts as
//! transparent. The cell is the texture the shader saw, so a line never
//! crosses into a neighbouring frame. The line's colour is opaque (the
//! shader's `color.a` 1): the texel becomes that colour, alpha 1.

/// `ArtBrush`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brush {
    Diamond,
    Circle,
    Square,
}

impl Brush {
    pub fn from_word(w: &str) -> Brush {
        match w {
            "circle" => Brush::Circle,
            "square" => Brush::Square,
            _ => Brush::Diamond,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Brush::Diamond => "diamond",
            Brush::Circle => "circle",
            Brush::Square => "square",
        }
    }
}

/// `has_contrary_neighbour`: whether under the brush round (x, y) a texel
/// (or, inside, the edge) is of the other kind. `transparent(x, y)` is
/// `None` outside the cell.
fn contrary(
    x: i64,
    y: i64,
    width: f64,
    brush: Brush,
    inside: bool,
    transparent: &dyn Fn(i64, i64) -> Option<bool>,
) -> bool {
    let reach = width.ceil();
    let mut i = -reach;
    while i <= reach {
        let offset = match brush {
            Brush::Diamond => width - i.abs(),
            Brush::Circle => ((width + 0.5).powi(2) - i * i).sqrt().floor(),
            Brush::Square => width,
        };
        let mut j = -offset.ceil();
        while j <= offset.ceil() {
            let other = transparent(x + i as i64, y + j as i64).unwrap_or(true);
            if other == inside {
                return true;
            }
            j += 1.0;
        }
        i += 1.0;
    }
    false
}

/// The texels of a `w` x `h` cell (alpha by `alpha(x, y)`) that take a
/// line of `width` with `brush`, inside or outside the shape.
pub fn mask(
    w: u32,
    h: u32,
    alpha: &dyn Fn(u32, u32) -> u8,
    width: f64,
    brush: Brush,
    inside: bool,
) -> Vec<bool> {
    let mut out = vec![false; (w * h) as usize];
    if width <= 0.0 {
        return out;
    }
    // `is_zero_approx(a)`: under 0.0001 of 1, so only alpha 0.
    let transparent = |x: i64, y: i64| -> Option<bool> {
        (x >= 0 && y >= 0 && x < i64::from(w) && y < i64::from(h))
            .then(|| alpha(x as u32, y as u32) == 0)
    };
    for y in 0..h {
        for x in 0..w {
            let opaque = alpha(x, y) > 0;
            if opaque == inside
                && contrary(
                    i64::from(x),
                    i64::from(y),
                    width,
                    brush,
                    inside,
                    &transparent,
                )
            {
                out[(y * w + x) as usize] = true;
            }
        }
    }
    out
}

/// Where a line of `width` at `place` lies: the inside and outside widths
/// (`both` puts the larger half inside, so a 1-pixel line keeps the
/// shape's size).
pub fn split(width: f64, place: &str) -> (f64, f64) {
    match place {
        "inside" => (width, 0.0),
        "both" => {
            let inner = (width / 2.0).ceil();
            (inner, (width - inner).max(0.0))
        }
        _ => (0.0, width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 7 x 5 cell with an L-shaped 3-texel shape: (2,1), (2,2), (3,2).
    fn shape() -> impl Fn(u32, u32) -> u8 {
        |x, y| {
            if matches!((x, y), (2, 1) | (2, 2) | (3, 2)) {
                255
            } else {
                0
            }
        }
    }

    fn marked(m: &[bool], w: u32) -> Vec<(u32, u32)> {
        m.iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| (i as u32 % w, i as u32 / w))
            .collect()
    }

    #[test]
    fn a_diamond_outline_takes_the_four_sides_only() {
        let m = mask(7, 5, &shape(), 1.0, Brush::Diamond, false);
        // Each transparent texel with a shape texel up, down, left or right.
        assert_eq!(
            marked(&m, 7),
            vec![(2, 0), (1, 1), (3, 1), (1, 2), (4, 2), (2, 3), (3, 3)]
        );
    }

    #[test]
    fn a_square_outline_fills_the_corners() {
        let m = mask(7, 5, &shape(), 1.0, Brush::Square, false);
        // Every transparent texel of the 3x3 neighbourhoods round the shape.
        assert_eq!(
            marked(&m, 7),
            vec![
                (1, 0),
                (2, 0),
                (3, 0),
                (1, 1),
                (3, 1),
                (4, 1),
                (1, 2),
                (4, 2),
                (1, 3),
                (2, 3),
                (3, 3),
                (4, 3)
            ]
        );
    }

    #[test]
    fn a_circle_of_width_two_rounds_off() {
        // One texel at (3, 3) of a 7 x 7 cell: a disc of radius 2.5 takes
        // offsets with i^2 + j^2 <= 6.25 column by column (floor(sqrt)).
        let one = |x: u32, y: u32| if (x, y) == (3, 3) { 255 } else { 0 };
        let m = mask(7, 7, &one, 2.0, Brush::Circle, false);
        let got = marked(&m, 7);
        assert_eq!(got.len(), 20, "{got:?}");
        assert!(got.contains(&(1, 2)) && got.contains(&(5, 4)));
        assert!(!got.contains(&(1, 1)) && !got.contains(&(3, 3)));
    }

    #[test]
    fn an_inline_counts_the_cells_edge_as_outside() {
        // A shape filling a 3 x 2 cell: every texel is on the edge.
        let full = |_: u32, _: u32| 255u8;
        let m = mask(3, 2, &full, 1.0, Brush::Diamond, true);
        assert!(m.iter().all(|b| *b));
        // In a 4 x 4 cell with a 2 x 2 block in the middle, inside: the
        // block itself, outside: its 8 side neighbours.
        let block = |x: u32, y: u32| {
            if (1..3).contains(&x) && (1..3).contains(&y) {
                255
            } else {
                0
            }
        };
        let m = mask(4, 4, &block, 1.0, Brush::Diamond, true);
        assert_eq!(marked(&m, 4), vec![(1, 1), (2, 1), (1, 2), (2, 2)]);
        let m = mask(4, 4, &block, 1.0, Brush::Diamond, false);
        assert_eq!(marked(&m, 4).len(), 8);
        assert_eq!(split(1.0, "both"), (1.0, 0.0));
        assert_eq!(split(3.0, "both"), (2.0, 1.0));
        assert_eq!(split(2.0, "outside"), (0.0, 2.0));
    }
}
