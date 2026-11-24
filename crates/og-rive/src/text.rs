//! A Rive text laid out on og-text's Parley layout (`og_text::layout`):
//! its runs shaped by Parley once each, its glyphs drawn by face id and
//! glyph id as Parley positioned them, never re-placed per character or
//! shaped by a second shaper. The lines are Rive's (rive-runtime's
//! `Text::update`, `GlyphLine::ComputeLineSpacing`,
//! `OrderedLine::buildEllipsisRuns`, `Text::fitFontScale`; MIT,
//! `third_party/rive/`): paragraphs at line feeds, hard lines at U+2028,
//! a greedy wrap at UAX #14 soft-wrap opportunities (Chromium's policy,
//! as Parley breaks lines) measured over Parley's clusters, each line's
//! box the union of its runs' Parley line boxes (CSS half-leading, not
//! Rive's ascent split), then alignment, vertical alignment, the origin
//! and the overflow modes. A word is cut where it overflows at cluster
//! boundaries (Rive's `breakWord`; Parley never splits a glyph cluster).

use super::*;
use og_text::fonts::{Collection, FaceId};
use og_text::layout::Text as Shaped;
use og_text::layout::{Alignment, LineHeight, Style, WhiteSpaceCollapse};
use og_ui_model::Affine;
use parley::layout::PositionedLayoutItem;
use parley_engine::break_overrides::CHROMIUM_LINE_BREAK_OVERRIDE;
use parley_engine::{Analysis, AnalysisOptions, Analyzer};
use std::ops::Range;

/// One styled run as laid out: a run with a bound font, a size and words.
pub(super) struct Run {
    /// Its style's node.
    pub style: u32,
    /// The family its face answers to in the collection.
    pub family: String,
    /// CSS px per em.
    pub size: f32,
    /// CSS px, or negative for the face's own.
    pub line_height: f32,
    /// CSS px after each glyph (Rive's letter spacing over the size).
    pub letter_spacing: f32,
    /// Its bytes in the runs' text.
    pub range: Range<usize>,
    /// Its text shaped in its style, left to right and left-aligned: the
    /// pieces set their own pens.
    pub shaped: Shaped,
    /// Its clusters in text order: their bytes and advance, CSS px.
    pub clusters: Vec<(Range<usize>, f32)>,
    /// Its Parley lines: their bytes, and each line's baseline below the
    /// box's top and its ascent and descent about the baseline, CSS px.
    pub lines: Vec<(Range<usize>, f32, f32, f32)>,
}

impl Run {
    /// The advance of `range` (its own text's bytes): its clusters inside
    /// it. A cut whose edge splits a cluster drops the cluster, so a
    /// measured width never takes part of a glyph.
    fn width(&self, range: Range<usize>) -> f32 {
        self.clusters
            .iter()
            .filter(|(c, _)| c.start >= range.start && c.end <= range.end)
            .map(|(_, w)| *w)
            .sum()
    }

    /// The Parley line of its own text that holds byte `at` (the last
    /// one past the end): its baseline, ascent and descent about the
    /// baseline, CSS px.
    fn line_at(&self, at: usize) -> (f32, f32, f32) {
        let at = at - self.range.start;
        for (r, b, a, d) in &self.lines {
            if r.start <= at && (at < r.end || (r.start == r.end && r.start == at)) {
                return (*b, *a, *d);
            }
        }
        self.lines
            .last()
            .map_or((0.0, 0.0, 0.0), |(_, b, a, d)| (*b, *a, *d))
    }

    /// Its style as a Parley style, in the collection.
    fn style(&self) -> Style<'_> {
        Style {
            family: &self.family,
            size: self.size,
            letter_spacing: self.letter_spacing,
            line_height: if self.line_height < 0.0 {
                LineHeight::NORMAL
            } else {
                LineHeight::Absolute(self.line_height)
            },
            white_space: WhiteSpaceCollapse::Preserve,
            wrap: false,
            align: Alignment::Start,
            ..Style::default()
        }
    }
}

/// A run's part of a line: its bytes in the runs' text, its pen `x` from
/// the line's start, its width, its line box, and the glyphs drawn.
#[derive(Clone, Debug)]
pub(super) struct Piece {
    pub run: usize,
    pub style: u32,
    pub range: Range<usize>,
    pub x: f32,
    pub width: f32,
    /// Its line box: ascent above and descent below its baseline, CSS px.
    pub ascent: f32,
    pub descent: f32,
    /// Its glyphs by face, filled for the pieces drawn.
    pub faces: Vec<FaceGlyphs>,
}

/// A piece's glyphs of one face (a Parley glyph run of it).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FaceGlyphs {
    pub face: FaceId,
    /// The face's index in its file (a collection file).
    pub index: u32,
    /// CSS px per em.
    pub size: f32,
    /// The face's variation location, 2.14 fixed point.
    pub coords: Vec<i16>,
    /// Bold the face lacks, drawn by emboldening (fontique's synthesis).
    pub embolden: bool,
    /// Italic the face lacks: a skew, degrees.
    pub skew: Option<f32>,
    /// Glyph ids and pens about the piece's pen, CSS px, y down.
    pub glyphs: Vec<TextGlyph>,
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
pub(super) struct Laid {
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
/// and words (rive-runtime's `makeStyled` skips the others), at `scale`,
/// shaped into the collection its face was registered in.
fn runs(
    a: &Artboard,
    pose: &[Values],
    ids: &[u32],
    scale: f32,
    fonts: &mut Collection,
) -> (String, Vec<Run>) {
    let mut text = String::new();
    let mut out = Vec::new();
    for &r in ids {
        let Kind::TextRun { style: node_style } = a.nodes[r as usize].kind else {
            continue;
        };
        let words = &a.strings[pose[r as usize].text as usize];
        let s = &pose[node_style as usize];
        let Some(font) = a.fonts.get(s.font as usize) else {
            continue;
        };
        let Some(face) = font.font else { continue };
        if words.is_empty() || s.font_size.is_nan() || s.font_size <= 0.0 {
            continue;
        }
        let size = s.font_size * scale;
        let line_height = if s.line_height >= 0.0 {
            s.line_height * scale
        } else {
            s.line_height
        };
        let letter_spacing = s.letter_spacing * scale;
        let start = text.len();
        text.push_str(words);
        let family = fonts.family_name(face).unwrap_or_default().to_string();
        let style = Style {
            family: &family,
            size,
            letter_spacing,
            line_height: if line_height < 0.0 {
                LineHeight::NORMAL
            } else {
                LineHeight::Absolute(line_height)
            },
            white_space: WhiteSpaceCollapse::Preserve,
            wrap: false,
            align: Alignment::Start,
            ..Style::default()
        };
        let run_text = text[start..].to_string();
        let mut shaped = Shaped::new();
        shaped.shape(fonts, &run_text, &style);
        let mut clusters = Vec::new();
        let mut lines = Vec::new();
        read(&shaped, &mut clusters, &mut lines);
        out.push(Run {
            style: node_style,
            family,
            size,
            line_height,
            letter_spacing,
            range: start..text.len(),
            shaped,
            clusters,
            lines,
        });
    }
    (text, out)
}

/// Reads a shaped text: its clusters in text order, and its lines with
/// their bytes and their boxes (each line's baseline, ascent and descent,
/// CSS px).
fn read(
    shaped: &Shaped,
    clusters: &mut Vec<(Range<usize>, f32)>,
    lines: &mut Vec<(Range<usize>, f32, f32, f32)>,
) {
    let mut top = 0.0;
    for line in shaped.layout().lines() {
        let m = line.metrics();
        let ascent = m.baseline - top;
        lines.push((
            line.text_range(),
            m.baseline,
            ascent,
            m.line_height - ascent,
        ));
        top += m.line_height;
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(gr) = item else {
                continue;
            };
            clusters.extend(gr.run().clusters().map(|c| (c.text_range(), c.advance())));
        }
    }
}

/// The tokens of a hard line: each word (a maximal run between UAX #14
/// soft-wrap opportunities, its trailing spaces trimmed off) and the
/// spaces after it, as byte offsets and lengths in the line. Chromium's
/// line-break policy, as Parley breaks lines with.
fn tokens(line: &str) -> Vec<(usize, usize, bool)> {
    let mut analysis = Analysis::new();
    Analyzer::new().analyze(
        line,
        &AnalysisOptions {
            line_break_override: Some(CHROMIUM_LINE_BREAK_OVERRIDE),
            ..AnalysisOptions::default()
        },
        &mut analysis,
    );
    let breaks = line
        .char_indices()
        .zip(analysis.char_info().iter())
        .filter_map(|((byte, _), info)| {
            (byte > 0 && info.is_soft_wrap_opportunity()).then_some(byte)
        })
        .chain([line.len()]);
    let mut start = 0;
    let mut out = Vec::new();
    for end in breaks {
        let unit = &line[start..end];
        let word = unit.trim_end_matches([' ', '\t', '\u{200b}']);
        if !word.is_empty() {
            out.push((start, word.len(), false));
        }
        if word.len() < unit.len() {
            out.push((start + word.len(), unit.len() - word.len(), true));
        }
        start = end;
    }
    out
}

fn paragraph_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{85}' | '\u{2029}')
}

/// The pieces of `text[a..b]`, one per run it crosses, measured.
fn pieces(runs: &[Run], a: usize, b: usize) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut x = 0.0;
    for (k, r) in runs.iter().enumerate() {
        let (s, e) = (a.max(r.range.start), b.min(r.range.end));
        if s >= e {
            continue;
        }
        let width = r.width(s..e);
        out.push(Piece {
            run: k,
            style: r.style,
            range: s..e,
            x,
            width,
            ascent: 0.0,
            descent: 0.0,
            faces: Vec::new(),
        });
        x += width;
    }
    out
}

fn width_of(runs: &[Run], a: usize, b: usize) -> f32 {
    runs.iter()
        .map(|r| r.width(a.max(r.range.start)..b.min(r.range.end)))
        .sum()
}

/// The run at byte `at` (the one before it at the end).
fn run_at(runs: &[Run], at: usize) -> usize {
    runs.iter()
        .position(|r| r.range.contains(&at))
        .unwrap_or_else(|| runs.iter().rposition(|r| r.range.start <= at).unwrap_or(0))
}

/// The byte ends of the clusters in `text[a..b]`, in order.
fn cluster_ends(runs: &[Run], a: usize, b: usize) -> Vec<usize> {
    let mut out = Vec::new();
    for r in runs {
        let (s, e) = (a.max(r.range.start), b.min(r.range.end));
        if s >= e {
            continue;
        }
        for (c, _) in &r.clusters {
            if c.start >= s - r.range.start && c.end <= e - r.range.start {
                out.push(r.range.start + c.end);
            }
        }
    }
    out
}

/// One hard line (`text[start..end]`, no line feed or U+2028) broken at
/// `limit` (`None`: never): each line's bytes, its trailing spaces left
/// out.
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
    for (off, len, space) in tokens(&text[start..end]) {
        let (off, e) = (start + off, start + off + len);
        if space {
            continue;
        }
        let over = |a: usize, b: usize| limit.is_some_and(|l| width_of(runs, a, b) > l);
        if has_word && over(line, e) {
            lines.push((line, content));
            // The spaces at a break go with it.
            line = off;
        }
        if break_words && line == off && over(line, e) {
            // A word alone on its line and wider than it: cut where it
            // overflows, at least one cluster a line.
            let l = limit.unwrap_or(f32::MAX);
            let ends = cluster_ends(runs, start, e);
            loop {
                let mut cut = line;
                for &ce in &ends {
                    if ce <= line {
                        continue;
                    }
                    if cut > line && width_of(runs, line, ce) > l {
                        break;
                    }
                    cut = ce;
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
/// `width` (`None`: the widest line's). Each line's box is the union of
/// its pieces' Parley line boxes, CSS half-leading down each one.
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
                let pieces = pieces(runs, s, e);
                let width = pieces.iter().map(|p| p.width).sum();
                let mut line = Line {
                    pieces,
                    width,
                    ..Line::default()
                };
                // An empty line takes the metrics of the run it is in.
                if line.pieces.is_empty() {
                    let run = run_at(runs, s);
                    line.pieces.push(Piece {
                        run,
                        style: runs[run].style,
                        range: s..s,
                        x: 0.0,
                        width: 0.0,
                        ascent: 0.0,
                        descent: 0.0,
                        faces: Vec::new(),
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
    for para in paragraphs.iter_mut() {
        let mut y = 0.0;
        for line in para.iter_mut() {
            let (mut asc, mut des) = (0.0f32, 0.0f32);
            for piece in &mut line.pieces {
                let (_, a, d) = runs[piece.run].line_at(piece.range.start);
                piece.ascent = a;
                piece.descent = d;
                asc = asc.max(a);
                des = des.max(d);
            }
            line.top = y;
            line.baseline = y + asc;
            y = line.baseline + des;
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
fn fit_scale(
    a: &Artboard,
    pose: &[Values],
    node: u32,
    fonts: &mut Collection,
    w: f32,
    h: f32,
) -> f32 {
    let Kind::Text { runs: ids, layout } = &a.nodes[node as usize].kind else {
        return 1.0;
    };
    let (_, base) = runs(a, pose, ids, 1.0, fonts);
    let max = base.iter().map(|r| r.size).fold(0.0, f32::max);
    if max <= 1.0 || !max.is_finite() {
        return 1.0;
    }
    let spacing = pose[node as usize].spacing;
    let fits = |top: i32, fonts: &mut Collection| {
        let s = top as f32 / max;
        let (text, rs) = runs(a, pose, ids, s, fonts);
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
    if fits(hi, fonts) {
        return hi as f32 / max;
    }
    hi -= 1;
    let (mut lo, mut best) = (1, 1);
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if fits(mid, fonts) {
            best = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    best as f32 / max
}

/// Text node `node` of `a` laid out at `pose` into the collection its
/// fonts were registered in (`None`: it draws no run).
pub(super) fn layout(
    a: &Artboard,
    pose: &[Values],
    node: u32,
    fonts: &mut Collection,
) -> Option<Laid> {
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
        fit_scale(a, pose, node, fonts, w, if fixed { h } else { f32::MAX })
    } else {
        1.0
    };
    let spacing = if l.overflow == Overflow::FitFontSize {
        v.spacing * scale
    } else {
        v.spacing
    };
    let (text, runs) = runs(a, pose, ids, scale, fonts);
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
            let mut dots = false;
            // A line wider than the box is cut to "..." in it, and so is
            // the last line drawn when the ones after it do not fit.
            if want_ellipsis && (line.width > w || (index == ellipsis_line && !ellipsis_last)) {
                let full = line.width;
                if let Some((cut, width)) = ellipsize(&runs, &pieces, w, fonts) {
                    pieces = cut;
                    dots = true;
                    x += match l.align {
                        HAlign::Right => full - width,
                        HAlign::Center => (full - width) / 2.0,
                        HAlign::Left => 0.0,
                    };
                }
            }
            min_x = min_x.min(x);
            if !words.is_empty() {
                words.push('\n');
            }
            for piece in &pieces {
                words.push_str(text.get(piece.range.clone()).unwrap_or(""));
                let mut piece = piece.clone();
                if piece.faces.is_empty() && !piece.range.is_empty() {
                    piece.faces = glyphs_of(&runs[piece.run].shaped, fonts, piece.range.clone());
                }
                if !piece.range.is_empty() || !piece.faces.is_empty() {
                    let pen = Vec2::new(x + piece.x, baseline);
                    drawn.push((piece, pen));
                }
            }
            if dots {
                words.push_str("...");
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
/// `buildEllipsisRuns`). The ellipsis takes the style of the last run
/// reached whose dots still fit, its glyphs shaped in the collection.
/// Answers the pieces and their width.
fn ellipsize(
    runs: &[Run],
    pieces: &[Piece],
    width: f32,
    fonts: &mut Collection,
) -> Option<(Vec<Piece>, f32)> {
    // "..." per run's style, as it is reached.
    let mut cache: Vec<(usize, Dots)> = Vec::new();
    let dots_for = |cache: &mut Vec<(usize, Dots)>, fonts: &mut Collection, run: usize| -> Dots {
        if let Some((_, d)) = cache.iter().find(|(k, _)| *k == run) {
            return d.clone();
        }
        let d = dots_of(fonts, &runs[run]);
        cache.push((run, d.clone()));
        d
    };
    let mut out: Vec<Piece> = Vec::new();
    let mut x = 0.0f32;
    let mut dots: Option<(usize, f32)> = None;
    'pieces: for p in pieces {
        if p.range.is_empty() {
            continue;
        }
        if dots.is_none_or(|d| d.0 != p.run) {
            let dw = dots_for(&mut cache, fonts, p.run).0;
            if dots.is_none() || x + dw <= width {
                dots = Some((p.run, dw));
            }
        }
        let dw = dots.map_or(0.0, |d| d.1);
        let mut kept = Piece {
            run: p.run,
            style: p.style,
            range: p.range.start..p.range.start,
            x: p.x,
            width: 0.0,
            ascent: p.ascent,
            descent: p.descent,
            faces: Vec::new(),
        };
        for (c, w) in &runs[p.run].clusters {
            if c.start < p.range.start - runs[p.run].range.start
                || c.end > p.range.end - runs[p.run].range.start
            {
                continue;
            }
            if x + kept.width + w + dw > width {
                break 'pieces;
            }
            kept.range.end = runs[p.run].range.start + c.end;
            kept.width += w;
        }
        if kept.range.is_empty() {
            break 'pieces;
        }
        x += kept.width;
        out.push(kept);
    }
    let (run, dw) = dots?;
    let (_, mut faces, ascent, descent) = dots_for(&mut cache, fonts, run);
    let fx = faces
        .first()
        .and_then(|f| f.glyphs.first())
        .map_or(0.0, |g| g.x);
    for f in &mut faces {
        for g in &mut f.glyphs {
            g.x -= fx;
        }
    }
    let end = out.last().map_or(0, |p| p.range.end);
    out.push(Piece {
        run,
        style: runs[run].style,
        range: end..end,
        x,
        width: dw,
        ascent,
        descent,
        faces,
    });
    Some((out, x + dw))
}

/// The width of "..." in one run's style: its width, its glyphs about
/// their pen, and its line box (ascent, descent), CSS px.
type Dots = (f32, Vec<FaceGlyphs>, f32, f32);

/// The width of "..." in a run's style, and its glyphs about their pen
/// and its line box (ascent, descent), CSS px.
fn dots_of(fonts: &mut Collection, r: &Run) -> Dots {
    let style = r.style();
    let mut t = Shaped::new();
    t.shape(fonts, "...", &style);
    let mut clusters = Vec::new();
    let mut lines = Vec::new();
    read(&t, &mut clusters, &mut lines);
    let (ascent, descent) = lines.first().map_or((0.0, 0.0), |(_, _, a, d)| (*a, *d));
    let faces = glyphs_of(&t, fonts, 0.."...".len());
    (t.width(), faces, ascent, descent)
}

/// The glyph runs of a shaped text's line that holds byte `at`, cut to
/// `range` and re-zeroed about its first drawn glyph: each one's face
/// data and its glyphs' pens, CSS px about the piece's pen (y down).
fn glyphs_of(shaped: &Shaped, fonts: &Collection, range: Range<usize>) -> Vec<FaceGlyphs> {
    let at = range.start;
    let mut out: Vec<FaceGlyphs> = Vec::new();
    let mut first: Option<f32> = None;
    for line in shaped.layout().lines() {
        let lr = line.text_range();
        let held = lr.start <= at && (at < lr.end || (lr.start == lr.end && lr.start == at));
        if !held {
            continue;
        }
        let m = line.metrics();
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(gr) = item else {
                continue;
            };
            let r = gr.run();
            let Some(face) = fonts.face_of(r.font()) else {
                continue;
            };
            let mut fg = FaceGlyphs {
                face,
                index: r.font().index,
                size: r.font_size(),
                coords: r.normalized_coords().iter().map(|c| c.to_bits()).collect(),
                embolden: r.synthesis().embolden(),
                skew: r.synthesis().skew(),
                glyphs: Vec::new(),
            };
            let mut gs = gr.positioned_glyphs();
            for c in r.visual_clusters() {
                let n = c.glyphs().count();
                if c.text_range().start < range.start || c.text_range().end > range.end {
                    for _ in 0..n {
                        gs.next();
                    }
                    continue;
                }
                for _ in 0..n {
                    let Some(g) = gs.next() else { break };
                    // A glyph no face draws (0) draws nothing.
                    if g.id == 0 {
                        continue;
                    }
                    first = Some(first.unwrap_or(g.x));
                    fg.glyphs.push(TextGlyph {
                        id: g.id,
                        x: g.x,
                        y: g.y - m.baseline,
                    });
                }
            }
            if !fg.glyphs.is_empty() {
                out.push(fg);
            }
        }
        break;
    }
    let fx = first.unwrap_or(0.0);
    for f in &mut out {
        for g in &mut f.glyphs {
            g.x -= fx;
        }
    }
    out
}
