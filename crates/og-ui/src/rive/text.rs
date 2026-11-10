//! A Rive text laid out on og-ui's text stack: its runs measured, shaped
//! and drawn with the fonts the cook made (`crate::text`), never by a
//! second shaper. The layout is rive-runtime's (`Text::update`,
//! `Text::buildRenderStyles`, `GlyphLine::ComputeLineSpacing`,
//! `OrderedLine::buildEllipsisRuns`, `Text::fitFontScale`; MIT,
//! `third_party/rive/`): paragraphs at line feeds, lines at U+2028 and
//! where words wrap (UAX #14 opportunities), each line's ascent and
//! descent from its runs' faces or their line heights, the first line of
//! the first paragraph at its face's ascent, then alignment, vertical
//! alignment, the origin and the overflow modes.

use super::*;
use crate::element::Affine;
use crate::shape::Features;
use crate::text::{FontMetrics, drawn_line, tokens};
use og_i18n::Direction;

/// One styled run as laid out: a run with a font, a size and words.
pub(super) struct Run<'a> {
    /// Its style's node.
    pub style: u32,
    pub metrics: &'a FontMetrics,
    pub handle: Option<og_draw::FontHandle>,
    pub ascent: f32,
    pub descent: f32,
    pub size: f32,
    pub line_height: f32,
    /// Em after each glyph (Rive's letter spacing over the size).
    pub tracking: f32,
    /// Its bytes in [`Laid::text`].
    pub range: std::ops::Range<usize>,
}

/// A run's part of a line, its pen `x` from the line's start.
#[derive(Clone, Debug)]
pub(super) struct Piece {
    pub run: usize,
    pub text: String,
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Debug, Default)]
struct Line {
    pieces: Vec<Piece>,
    width: f32,
    top: f32,
    baseline: f32,
    bottom: f32,
    start_x: f32,
}

/// A text laid out in its own space: its lines' pieces at their pens, its
/// box, and the map from its space to its node's (fit and origin).
pub(super) struct Laid<'a> {
    pub runs: Vec<Run<'a>>,
    /// The pieces drawn, each with its pen (x, baseline).
    pub drawn: Vec<(Piece, Vec2)>,
    /// The box, `[min x, min y, max x, max y]`, in the text's space.
    pub bounds: [f32; 4],
    /// The cut of an [`Overflow::Clipped`] text, in the text's space.
    pub clip: Option<[f32; 4]>,
    pub transform: Affine,
    /// The words drawn, in reading order.
    pub words: String,
}

/// The runs of text node `t` that draw: each with a bound font, a size
/// and words (rive-runtime's `makeStyled` skips the others), at `scale`.
fn runs<'a>(a: &'a Artboard, pose: &[Values], runs: &[u32], scale: f32) -> (String, Vec<Run<'a>>) {
    let mut text = String::new();
    let mut out = Vec::new();
    for &r in runs {
        let Kind::TextRun { style } = a.nodes[r as usize].kind else {
            continue;
        };
        let words = &a.strings[pose[r as usize].text as usize];
        let s = &pose[style as usize];
        let Some(font) = a.fonts.get(s.font as usize) else {
            continue;
        };
        let Some(ui) = &font.font else { continue };
        if words.is_empty() || s.font_size.is_nan() || s.font_size <= 0.0 {
            continue;
        }
        let size = s.font_size * scale;
        let start = text.len();
        text.push_str(words);
        out.push(Run {
            style,
            metrics: &ui.metrics,
            handle: ui.handle,
            ascent: font.ascent,
            descent: font.descent,
            size,
            line_height: if s.line_height >= 0.0 {
                s.line_height * scale
            } else {
                s.line_height
            },
            tracking: s.letter_spacing * scale / size,
            range: start..text.len(),
        });
    }
    (text, out)
}

fn paragraph_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{85}' | '\u{2029}')
}

/// The pieces of `text[a..b]`, one per run it crosses, measured.
fn pieces(text: &str, runs: &[Run], a: usize, b: usize) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut x = 0.0;
    for (k, r) in runs.iter().enumerate() {
        let (s, e) = (a.max(r.range.start), b.min(r.range.end));
        if s >= e {
            continue;
        }
        let t = &text[s..e];
        let width = r
            .metrics
            .measure_with(t, r.size, r.tracking, Features::NONE);
        out.push(Piece {
            run: k,
            text: t.to_string(),
            x,
            width,
        });
        x += width;
    }
    out
}

fn width_of(text: &str, runs: &[Run], a: usize, b: usize) -> f32 {
    pieces(text, runs, a, b).iter().map(|p| p.width).sum()
}

/// The run at byte `at` (the one before it at the end).
fn run_at(runs: &[Run], at: usize) -> usize {
    runs.iter()
        .position(|r| r.range.contains(&at))
        .unwrap_or_else(|| runs.iter().rposition(|r| r.range.start <= at).unwrap_or(0))
}

/// One hard line (`text[start..end]`, no line feed or U+2028) broken at
/// `limit` (`None`: never): each line's bytes, its trailing spaces left out.
fn wrap(
    text: &str,
    runs: &[Run],
    start: usize,
    end: usize,
    limit: Option<f32>,
    break_words: bool,
) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut line = start;
    let mut content = start;
    let mut has_word = false;
    let hard = &text[start..end];
    for (unit, space) in tokens(hard) {
        let off = start + (unit.as_ptr() as usize - hard.as_ptr() as usize);
        let e = off + unit.len();
        if space {
            continue;
        }
        let over = |a: usize, b: usize| limit.is_some_and(|l| width_of(text, runs, a, b) > l);
        if has_word && over(line, e) {
            lines.push((line, content));
            // The spaces at a break go with it.
            line = off;
        }
        if break_words && line == off && over(line, e) {
            // A word alone on its line and wider than it: cut where it
            // overflows, at least one character a line.
            let l = limit.unwrap_or(f32::MAX);
            loop {
                let mut cut = line;
                for (i, c) in text[line..e].char_indices() {
                    let next = line + i + c.len_utf8();
                    if cut > line && width_of(text, runs, line, next) > l {
                        break;
                    }
                    cut = next;
                }
                if cut >= e {
                    break;
                }
                lines.push((line, cut));
                line = cut;
            }
        }
        content = e;
        has_word = true;
    }
    lines.push((line, content.max(line)));
    lines
}

/// The lines of `text` broken at `limit`, by paragraph, spaced as Rive
/// spaces them (each paragraph's lines from its own top) and aligned in
/// `width` (`None`: the widest line's).
fn lines(
    text: &str,
    runs: &[Run],
    limit: Option<f32>,
    break_words: bool,
    align: HAlign,
    width: Option<f32>,
) -> Vec<Vec<Line>> {
    let mut paragraphs: Vec<Vec<Line>> = Vec::new();
    let mut para_start = 0;
    let mut bounds: Vec<(usize, usize)> = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if paragraph_break(c) {
            let mut next = i + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|(_, n)| *n == '\n') {
                chars.next();
                next += 1;
            }
            bounds.push((para_start, i));
            para_start = next;
        }
    }
    if para_start < text.len() || bounds.is_empty() {
        bounds.push((para_start, text.len()));
    }
    for (ps, pe) in bounds {
        let mut out = Vec::new();
        let mut hs = ps;
        let para = &text[ps..pe];
        let mut hards: Vec<(usize, usize)> = para
            .match_indices('\u{2028}')
            .map(|(i, _)| {
                let h = (hs, ps + i);
                hs = ps + i + '\u{2028}'.len_utf8();
                h
            })
            .collect();
        hards.push((hs, pe));
        for (a, b) in hards {
            for (s, e) in wrap(text, runs, a, b, limit, break_words) {
                let pieces = pieces(text, runs, s, e);
                let width = pieces.iter().map(|p| p.width).sum();
                let mut line = Line {
                    pieces,
                    width,
                    ..Line::default()
                };
                // An empty line takes the metrics of the run it is in.
                if line.pieces.is_empty() {
                    line.pieces.push(Piece {
                        run: run_at(runs, s),
                        text: String::new(),
                        x: 0.0,
                        width: 0.0,
                    });
                }
                out.push(line);
            }
        }
        paragraphs.push(out);
    }
    let width = width.unwrap_or_else(|| {
        paragraphs
            .iter()
            .flatten()
            .map(|l| l.width)
            .fold(0.0, f32::max)
    });
    for (p, para) in paragraphs.iter_mut().enumerate() {
        let mut first = p == 0;
        let mut y = 0.0;
        for line in para.iter_mut() {
            let (mut asc, mut real, mut des) = (0.0f32, 0.0f32, 0.0f32);
            for piece in &line.pieces {
                let r = &runs[piece.run];
                let (a, d) = if r.line_height < 0.0 {
                    (-r.ascent * r.size, r.descent * r.size)
                } else {
                    let factor = r.ascent / (r.ascent + r.descent).max(1e-6);
                    let a = -factor * r.line_height;
                    (a, r.line_height + a)
                };
                real = real.min(-r.ascent * r.size);
                asc = asc.min(a);
                des = des.max(d);
            }
            line.top = y;
            if first {
                y = -real;
                first = false;
            } else {
                y -= asc;
            }
            line.baseline = y;
            y += des;
            line.bottom = y;
            line.start_x = match align {
                HAlign::Left => 0.0,
                HAlign::Right => width - line.width,
                HAlign::Center => width / 2.0 - line.width / 2.0,
            };
        }
    }
    paragraphs
}

/// Rive's `fitFontScale`: the largest whole top size that fits `w` by `h`,
/// over the largest size the runs have.
fn fit_scale(a: &Artboard, pose: &[Values], node: u32, w: f32, h: f32) -> f32 {
    let Kind::Text { runs: ids, layout } = &a.nodes[node as usize].kind else {
        return 1.0;
    };
    let (_, base) = runs(a, pose, ids, 1.0);
    let max = base.iter().map(|r| r.size).fold(0.0, f32::max);
    if max <= 1.0 || !max.is_finite() {
        return 1.0;
    }
    let spacing = pose[node as usize].spacing;
    let fits = |top: i32| {
        let s = top as f32 / max;
        let (text, rs) = runs(a, pose, ids, s);
        if rs.is_empty() {
            return true;
        }
        let limit = layout.wrap.then_some(w);
        let ls = lines(&text, &rs, limit, layout.break_words, layout.align, Some(w));
        let widest = ls.iter().flatten().map(|l| l.width).fold(0.0, f32::max);
        let y: f32 = ls
            .iter()
            .map(|p| p.last().map_or(0.0, |l| l.bottom) + spacing * s)
            .sum();
        widest <= w && y <= h
    };
    let mut hi = max.min((1 << 24) as f32) as i32;
    if fits(hi) {
        return hi as f32 / max;
    }
    hi -= 1;
    let (mut lo, mut best) = (1, 1);
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if fits(mid) {
            best = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    best as f32 / max
}

/// Text node `node` of `a` laid out at `pose` (`None`: it draws no run).
pub(super) fn layout<'a>(a: &'a Artboard, pose: &[Values], node: u32) -> Option<Laid<'a>> {
    let Kind::Text {
        runs: ids,
        layout: l,
    } = &a.nodes[node as usize].kind
    else {
        return None;
    };
    let v = &pose[node as usize];
    let fixed = l.sizing == Sizing::Fixed;
    let (w, h) = (v.width, v.height);
    let scale = if l.overflow == Overflow::FitFontSize && (l.sizing != Sizing::AutoWidth || fixed) {
        fit_scale(a, pose, node, w, if fixed { h } else { f32::MAX })
    } else {
        1.0
    };
    let spacing = if l.overflow == Overflow::FitFontSize {
        v.spacing * scale
    } else {
        v.spacing
    };
    let (text, runs) = runs(a, pose, ids, scale);
    if runs.is_empty() {
        return None;
    }
    let auto = l.sizing == Sizing::AutoWidth;
    let limit = (!auto && l.wrap).then_some(w);
    let paras = lines(
        &text,
        &runs,
        limit,
        l.break_words,
        l.align,
        (!auto).then_some(w),
    );
    // computeBoundsInfo.
    let mut y = 0.0f32;
    let mut min_y = 0.0f32;
    if l.baseline
        && let Some(first) = paras.first().and_then(|p| p.first())
    {
        y -= first.baseline;
        min_y = y;
    }
    let want_ellipsis = l.overflow == Overflow::Ellipsis && fixed;
    let mut ellipsis_line: i32 = -1;
    let mut ellipsed = 0.0;
    let mut max_width = 0.0f32;
    let mut last = -1;
    for p in &paras {
        for line in p {
            max_width = max_width.max(line.width);
            last += 1;
            if want_ellipsis && y + line.bottom <= h {
                ellipsed = y + line.bottom;
                ellipsis_line += 1;
            }
        }
        y += p.last().map_or(0.0, |l| l.bottom);
        y += spacing;
    }
    if want_ellipsis && ellipsis_line == -1 {
        ellipsis_line = 0;
    }
    let total = if ellipsis_line > 0 { ellipsed } else { y };
    let ellipsis_last = last == ellipsis_line;
    let bounds = match l.sizing {
        Sizing::AutoWidth => [0.0, min_y, max_width, min_y.max(total - spacing)],
        Sizing::AutoHeight => [0.0, min_y, w, min_y.max(total - spacing)],
        Sizing::Fixed => [0.0, min_y, w, min_y + h],
    };
    let bh = bounds[3] - bounds[1];
    let bw = bounds[2] - bounds[0];
    let valign = match l.valign {
        VAlign::Middle => (total - bh) / 2.0,
        VAlign::Bottom => total - bh,
        VAlign::Top => 0.0,
    };
    let clip = (l.overflow == Overflow::Clipped).then(|| {
        [
            bounds[0],
            bounds[1] + valign,
            bounds[0] + bw,
            bounds[1] + valign + bh,
        ]
    });
    // The lines drawn (shouldDrawLine), the ellipsis on its line.
    let mut drawn = Vec::new();
    let mut words = String::new();
    let mut cur_y = min_y;
    let mut index = 0;
    let mut min_x = f32::MAX;
    'paras: for p in &paras {
        for line in p {
            // (skip this line, stop drawing) as Rive's shouldDrawLine.
            let (top, bottom) = (cur_y + line.top, cur_y + line.bottom);
            let (lo, hi) = (total / 2.0 - h / 2.0, total / 2.0 + h / 2.0);
            let (skip, stop) = match (l.overflow, fixed, l.valign) {
                (Overflow::Hidden, true, VAlign::Top) => (false, bottom > h),
                (Overflow::Hidden, true, VAlign::Middle) => (top < lo, top >= lo && bottom > hi),
                (Overflow::Hidden, true, VAlign::Bottom) => (top < total - h, false),
                (Overflow::Clipped, true, VAlign::Top) => (false, top > h),
                (Overflow::Clipped, true, VAlign::Middle) => {
                    (bottom < lo, bottom >= lo && top > hi)
                }
                (Overflow::Clipped, true, VAlign::Bottom) => (bottom < total - h, false),
                _ => (false, false),
            };
            if stop {
                break 'paras;
            }
            if skip {
                index += 1;
                continue;
            }
            let baseline = cur_y + line.baseline;
            let mut x = line.start_x;
            let mut pieces = line.pieces.clone();
            if index == ellipsis_line {
                let full = line.width;
                if let Some((cut, width)) = ellipsize(&runs, &pieces, w, ellipsis_last) {
                    pieces = cut;
                    x += match l.align {
                        HAlign::Right => full - width,
                        HAlign::Center => (full - width) / 2.0,
                        HAlign::Left => 0.0,
                    };
                }
            }
            min_x = min_x.min(x);
            for piece in pieces {
                if !words.is_empty() && piece.x == 0.0 {
                    words.push('\n');
                }
                words.push_str(&piece.text);
                let pen = Vec2::new(x + piece.x, baseline);
                if !piece.text.is_empty() {
                    drawn.push((piece, pen));
                }
            }
            if index == ellipsis_line {
                break 'paras;
            }
            index += 1;
        }
        cur_y += p.last().map_or(0.0, |l| l.bottom);
        cur_y += spacing;
    }
    // Fit and the origin.
    let mut s = 1.0f32;
    let mut xo = -bw * v.origin.x;
    let mut yo = -bh * v.origin.y;
    if l.overflow == Overflow::Fit {
        let xs = if (!auto || fixed) && max_width > bw {
            bw / max_width
        } else {
            1.0
        };
        let base = if l.fit_from_baseline {
            paras
                .first()
                .and_then(|p| p.first())
                .map_or(0.0, |l| l.baseline)
        } else {
            0.0
        };
        let ys = if fixed && total > bh {
            (bh - base) / (total - base)
        } else {
            1.0
        };
        if xs != 1.0 || ys != 1.0 {
            s = xs.min(ys).max(0.0);
            yo += base * (1.0 - s);
            let min_x = if min_x == f32::MAX { 0.0 } else { min_x };
            match l.align {
                HAlign::Center => xo += (bw - max_width * s) / 2.0 - min_x * s,
                HAlign::Right => xo += bw - max_width * s - min_x * s,
                HAlign::Left => {}
            }
        }
    }
    if l.valign != VAlign::Top && fixed {
        yo = -bh * v.origin.y;
        match l.valign {
            VAlign::Middle => yo += (bh - total * s) / 2.0,
            VAlign::Bottom => yo += bh - total * s,
            VAlign::Top => {}
        }
    }
    // A text drawn with no line (all hidden) still has its box.
    Some(Laid {
        runs,
        drawn,
        bounds,
        clip,
        transform: Affine {
            a: s,
            d: s,
            e: xo,
            f: yo,
            ..Affine::IDENTITY
        },
        words,
    })
}

/// A line's pieces cut to end in "..." within `width` (rive-runtime's
/// `buildEllipsisRuns`): `None` when it is the last line and fits whole.
/// The ellipsis takes the font of the last run reached whose dots still
/// fit. Answers the pieces and their width.
fn ellipsize(runs: &[Run], pieces: &[Piece], width: f32, last: bool) -> Option<(Vec<Piece>, f32)> {
    if last && pieces.iter().map(|p| p.width).sum::<f32>() <= width {
        return None;
    }
    let mut out: Vec<Piece> = Vec::new();
    let mut x = 0.0f32;
    let mut dots: Option<(usize, f32)> = None;
    let mut checked: Option<(*const FontMetrics, f32)> = None;
    'pieces: for p in pieces {
        let r = &runs[p.run];
        let key = (r.metrics as *const FontMetrics, r.size);
        if checked != Some(key) {
            checked = Some(key);
            let w = r
                .metrics
                .measure_with("...", r.size, r.tracking, Features::NONE);
            if dots.is_none() || x + w <= width {
                dots = Some((p.run, w));
            }
        }
        let dw = dots.map_or(0.0, |d| d.1);
        let mut kept = String::new();
        let mut kw = 0.0;
        for (i, c) in p.text.char_indices() {
            let next = &p.text[..i + c.len_utf8()];
            let nw = r
                .metrics
                .measure_with(next, r.size, r.tracking, Features::NONE);
            if x + nw + dw > width {
                if !kept.is_empty() {
                    out.push(Piece {
                        run: p.run,
                        text: kept,
                        x,
                        width: kw,
                    });
                }
                x += kw;
                break 'pieces;
            }
            kept = next.to_string();
            kw = nw;
        }
        out.push(Piece {
            run: p.run,
            text: kept,
            x,
            width: kw,
        });
        x += kw;
    }
    let (run, dw) = dots?;
    out.push(Piece {
        run,
        text: "...".into(),
        x,
        width: dw,
    });
    Some((out, x + dw))
}

/// A piece as a text command draws it: what it draws and the placements.
pub(super) fn shaped(r: &Run, text: &str) -> (String, Vec<og_draw::GlyphPlacement>) {
    drawn_line(r.metrics, text, Direction::Ltr, r.tracking, Features::NONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::UiFont;

    /// One text of `words` in a style of `size` px over a face whose every
    /// character is half an em wide, its ascent 0.8 em and descent 0.2.
    fn board(words: &str, size: f32, l: TextLayout, w: f32, h: f32) -> Artboard {
        let node = |kind, parent| Node {
            name: "t".into(),
            parent,
            kind,
            values: Values::default(),
        };
        let mut text = node(
            Kind::Text {
                runs: vec![3],
                layout: l,
            },
            0,
        );
        text.values.width = w;
        text.values.height = h;
        text.values.origin = Vec2::ZERO;
        let mut style = node(Kind::TextStyle { paints: vec![] }, 1);
        style.values.font_size = size;
        style.values.font = 0;
        let run = node(Kind::TextRun { style: 2 }, 1);
        Artboard {
            name: "a".into(),
            nodes: vec![node(Kind::Artboard { paints: vec![] }, 0), text, style, run],
            fonts: vec![Font {
                id: "f".into(),
                ascent: 0.8,
                descent: 0.2,
                font: Some(UiFont::unrendered(0.5)),
            }],
            strings: vec![words.into()],
            ..Artboard::default()
        }
    }

    fn laid(a: &Artboard) -> Laid<'_> {
        let pose: Vec<Values> = a.nodes.iter().map(|n| n.values).collect();
        layout(a, &pose, 1).expect("draws")
    }

    fn lines(l: &Laid) -> Vec<(String, f32, f32)> {
        l.drawn
            .iter()
            .map(|(p, pen)| (p.text.clone(), pen.x, pen.y))
            .collect()
    }

    /// Wrapped at a fixed width, centred: each line's baseline at the
    /// ascent below the one before's bottom (the first at the face's
    /// ascent), each line centred in the box; the box as tall as the lines.
    #[test]
    fn lines_wrap_space_and_align_as_rive_lays_them() {
        let l = TextLayout {
            sizing: Sizing::AutoHeight,
            align: HAlign::Center,
            wrap: true,
            ..TextLayout::default()
        };
        // 10 px a character: "ab cd" in 30 px breaks after "ab".
        let a = board("ab cd e", 20.0, l, 30.0, 0.0);
        let t = laid(&a);
        assert_eq!(
            lines(&t),
            [
                ("ab".into(), 5.0, 16.0),
                ("cd".into(), 5.0, 36.0),
                ("e".into(), 10.0, 56.0)
            ]
        );
        assert_eq!(t.bounds, [0.0, 0.0, 30.0, 60.0]);
        // Auto width: one line, as wide as its words, no wrap.
        let a = board("ab cd", 20.0, TextLayout::default(), 30.0, 0.0);
        let t = laid(&a);
        assert_eq!(lines(&t), [("ab cd".into(), 0.0, 16.0)]);
        assert_eq!(t.bounds, [0.0, 0.0, 50.0, 20.0]);
        // A line feed is a paragraph, U+2028 a line of the same one.
        let a = board("ab\u{2028}c\nd", 20.0, TextLayout::default(), 0.0, 0.0);
        let ys: Vec<f32> = laid(&a).drawn.iter().map(|d| d.1.y).collect();
        assert_eq!(ys, [16.0, 36.0, 56.0]);
    }

    /// A fixed box: the line that fits last ends in "...", cut so the dots
    /// fit; hidden drops what does not fit whole; middle centres the lines
    /// down the box; fit scales them into it; fit font size sets the
    /// largest whole size that fits.
    #[test]
    fn overflow_modes_keep_the_text_in_its_box() {
        let fixed = |overflow, valign| TextLayout {
            sizing: Sizing::Fixed,
            overflow,
            valign,
            wrap: true,
            ..TextLayout::default()
        };
        // 50 px wide, 30 tall: "abcd efgh" is two lines of 20 px each.
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::Ellipsis, VAlign::Top),
            50.0,
            30.0,
        );
        let t = laid(&a);
        // "abcd" + "..." is 70 px: "ab" and the dots fill 50.
        assert_eq!(t.words, "ab...");
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::Hidden, VAlign::Top),
            50.0,
            30.0,
        );
        assert_eq!(laid(&a).words, "abcd");
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::Visible, VAlign::Top),
            50.0,
            30.0,
        );
        assert_eq!(laid(&a).words, "abcd\nefgh");
        // Middle: 40 px of lines in 30, raised by 5.
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::Visible, VAlign::Middle),
            50.0,
            30.0,
        );
        assert_eq!(laid(&a).transform.f, -5.0);
        // Fit: (30 − 0) / 40 of its size.
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::Fit, VAlign::Top),
            50.0,
            30.0,
        );
        assert_eq!(laid(&a).transform.a, 0.75);
        // Fit font size: at 15 px both lines of 30 px fit in 50 wide and
        // 30 tall; at 16, two lines of 16 px are 32 tall.
        let a = board(
            "abcd efgh",
            20.0,
            fixed(Overflow::FitFontSize, VAlign::Top),
            50.0,
            30.0,
        );
        let t = laid(&a);
        assert_eq!(t.runs[0].size, 15.0);
        assert_eq!(t.words, "abcd\nefgh");
    }
}
