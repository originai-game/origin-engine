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
//! transparent, except in a tile, which continues into its neighbours:
//! there only the tile's own texels count, so a full tile takes no line
//! and a seam never shows between two. The cell is the texture the shader saw, so a line never
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
/// (or, inside, the edge when `edge_clear`) is of the other kind.
/// `transparent(x, y)` is `None` outside the cell.
fn contrary(
    x: i64,
    y: i64,
    width: f64,
    brush: Brush,
    inside: bool,
    edge_clear: bool,
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
            // Off the cell: transparent when `edge_clear`, else no texel.
            let other = transparent(x + i as i64, y + j as i64)
                .map_or(edge_clear && inside, |t| t == inside);
            if other {
                return true;
            }
            j += 1.0;
        }
        i += 1.0;
    }
    false
}

/// The texels of a `w` x `h` cell (alpha by `alpha(x, y)`) that take a
/// line of `width` with `brush`, inside or outside the shape; the cell's
/// edge counts as transparent when `edge_clear` (a sprite's frame, not a
/// tile).
pub fn mask(
    w: u32,
    h: u32,
    alpha: &dyn Fn(u32, u32) -> u8,
    width: f64,
    brush: Brush,
    inside: bool,
    edge_clear: bool,
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
                    edge_clear,
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
