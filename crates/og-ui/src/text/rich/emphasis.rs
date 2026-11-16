//! Emphasis marks (CSS Text Decoration 4 §3): one mark for each
//! typographic character unit that takes one (`og_i18n::takes_emphasis`),
//! centred on it at half its size, set as its ruby would be: over or
//! under it across a line, on its right or left down one (§3.4). The
//! shapes are drawn, not taken from a font, so every face has them (§3.1
//! lets the UA synthesise them); a string's character is drawn as text.
//! Like ruby that fits its line's leading, the marks take no room.

use super::*;
use crate::css::text_css::{Emphasis, EmphasisMark, EmphasisShape};

/// Grows `line` by what its emphasis marks lack of its leading (§3.4:
/// marks affect a line's height as ruby does, CSS Ruby 1 §3.6): a mark,
/// half an em, fits when both half-leadings around its text, its own
/// line's and the next's, hold it; the line grows on the marks' side by
/// what they lack.
pub(crate) fn make_room(rc: &RichContent, fonts: &dyn Metrics, line: &mut RichLine) {
    let Some(e) = rc.emphasis else { return };
    let extra = line
        .pieces
        .iter()
        .filter(|p| !p.text.trim().is_empty())
        .filter_map(|p| {
            let s = &rc.spans[p.span];
            if s.atom.is_some() {
                return None;
            }
            let size = s.step.size;
            let m = fonts.metrics_styled(s.font, s.query(), &p.text);
            let (a, d) = super::ruby::content_edges(m, &p.text);
            let lh = if p.text.chars().any(crate::text::is_cjk) {
                s.step.cjk_line_height
            } else {
                s.step.line_height
            };
            let lead = (lh - (a + d) * size) / 2.0;
            Some((size / 2.0 - 2.0 * lead).max(0.0))
        })
        .fold(0.0f32, f32::max);
    line.height += extra;
    if !e.under {
        line.baseline += extra;
    }
}

/// The middle of each cluster of a piece that takes a mark, px from the
/// piece's start along its line: from its drawn glyphs (`index` the
/// character each starts at, `x0`..`x1` its stretch), clusters of several
/// glyphs merged.
pub(crate) fn spots(text: &str, glyphs: impl IntoIterator<Item = (usize, f32, f32)>) -> Vec<f32> {
    let chars: Vec<char> = text.chars().collect();
    let clusters = og_i18n::graphemes(text);
    let mut out: Vec<(usize, f32, f32)> = Vec::new();
    for (index, x0, x1) in glyphs {
        let Some(&(a, _)) = clusters.iter().find(|(a, b)| (*a..*b).contains(&index)) else {
            continue;
        };
        match out.iter_mut().find(|(k, ..)| *k == a) {
            Some(m) => (m.1, m.2) = (m.1.min(x0), m.2.max(x1)),
            None => out.push((a, x0.min(x1), x0.max(x1))),
        }
    }
    out.into_iter()
        .filter(|(a, ..)| {
            let b = clusters.iter().find(|c| c.0 == *a).map_or(*a + 1, |c| c.1);
            let cluster: String = chars[*a..b].iter().collect();
            og_i18n::takes_emphasis(&cluster)
        })
        .map(|(_, x0, x1)| (x0 + x1) / 2.0)
        .collect()
}

/// Draws mark `e` centred on each of `centres` for text `size` px tall in
/// `font`, its ink `ink` (linear RGBA); `vertical`: down a line (an
/// unnamed shape is a sesame there, a circle across).
#[allow(clippy::too_many_arguments)]
pub(crate) fn marks(
    e: &Emphasis,
    vertical: bool,
    centres: &[(f32, f32)],
    size: f32,
    font: &crate::UiFont,
    ink: [f32; 4],
    clip: Rect,
    cmds: &mut Vec<crate::draw::DrawCmd>,
) {
    use og_draw::{UiAlign, UiGeometry, UiMode, UiShape, UiStroke};
    // The mark's em box: half the text's.
    let b = size * 0.5;
    if let EmphasisMark::Char(c) = e.mark {
        let Some(handle) = font.handle else { return };
        let text = c.to_string();
        let m = &font.metrics;
        let w = m.measure_tracked(&text, b, 0.0);
        let (a, d) = super::ruby::content_edges(m, &text);
        for &(x, y) in centres {
            cmds.push(crate::draw::DrawCmd::Text {
                x: x - w / 2.0,
                baseline: y + (a - d) / 2.0 * b,
                size: b,
                text: text.clone(),
                color: ink,
                font: handle,
                bounds: Rect::new(x - b / 2.0, y - b / 2.0, b, b),
                clip,
                tracking: 0.0,
                weight: 0.0,
                glow: None,
                placements: Vec::new(),
            });
        }
        return;
    }
    let EmphasisMark::Shape(shape, filled) = e.mark else {
        return;
    };
    let shape = shape.unwrap_or(if vertical {
        EmphasisShape::Sesame
    } else {
        EmphasisShape::Circle
    });
    let line = (b * 0.1).max(0.75);
    let mode = |filled: bool| {
        if filled {
            UiMode::Fill
        } else {
            UiMode::Stroke(UiStroke {
                width: line,
                align: UiAlign::Inside,
                ..Default::default()
            })
        }
    };
    let triangle = UiGeometry::Polygon {
        sides: 3,
        inner: 1.0,
        rounding: 0.0,
        rotation: 0.0,
    };
    // Each part: its geometry, its half extent, whether it fills, its turn
    // (radians clockwise).
    let parts: Vec<(UiGeometry, [f32; 2], bool, f32)> = match shape {
        EmphasisShape::Dot => vec![(UiGeometry::Ellipse, [0.15 * b; 2], filled, 0.0)],
        EmphasisShape::Circle => vec![(UiGeometry::Ellipse, [0.35 * b; 2], filled, 0.0)],
        EmphasisShape::DoubleCircle => vec![
            (UiGeometry::Ellipse, [0.35 * b; 2], false, 0.0),
            (UiGeometry::Ellipse, [0.18 * b; 2], filled, 0.0),
        ],
        EmphasisShape::Triangle => vec![(triangle, [0.4 * b; 2], filled, 0.0)],
        EmphasisShape::Sesame => vec![(
            UiGeometry::Ellipse,
            [0.12 * b, 0.32 * b],
            filled,
            std::f32::consts::FRAC_PI_6,
        )],
    };
    for &(x, y) in centres {
        for (geometry, half, fill, turn) in &parts {
            let (sin, cos) = turn.sin_cos();
            let mut s = UiShape::rect(
                [x - half[0], y - half[1], 2.0 * half[0], 2.0 * half[1]],
                0.0,
                ink,
            );
            s.geometry = geometry.clone();
            s.mode = mode(*fill);
            s.transform = [cos, sin, -sin, cos];
            cmds.push(crate::draw::DrawCmd::Shape {
                shape: Box::new(s),
                clip,
            });
        }
    }
}
