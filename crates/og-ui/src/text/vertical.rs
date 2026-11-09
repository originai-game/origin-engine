//! Text in a vertical line (CSS Writing Modes 4 §5). A line runs down
//! (up in `sideways-lr`) in runs that stand upright or lie sideways:
//! `text-orientation: mixed` stands a character upright where Unicode's
//! vertical orientation (UAX #50) does (CJK, kana, Hangul, symbols), and
//! turns the others clockwise, as a horizontal line turned; `upright`
//! stands every character up, `sideways` and the `sideways-*` modes lay
//! every one down. An upright character advances by the font's vertical
//! metrics (`vmtx`), else one em; one lying down by its horizontal
//! advance, shaped as a horizontal run. `text-combine-upright` sets a
//! whole text side by side in one upright em.

use super::{FontMetrics, Line, greedy_lines, tokens};
use crate::layout::flow::{TextOrientation, VerticalText};
use crate::shape::Features;
use std::ops::Range;

/// UAX #50's `Vertical_Orientation` of `c` is U, Tu or Tr: it stands
/// upright in a mixed vertical line.
pub(crate) fn upright_char(c: char) -> bool {
    let u = c as u32;
    if matches!(u, 0xff61..=0xffdc | 0xffe8..=0xffee) {
        // Halfwidth forms lie down.
        return false;
    }
    matches!(u,
        0x00a7 | 0x00a9 | 0x00ae | 0x00b1 | 0x00bc..=0x00be | 0x00d7 | 0x00f7
        | 0x02ea..=0x02eb
        | 0x1100..=0x11ff | 0x1401..=0x167f | 0x18b0..=0x18ff
        | 0x2016 | 0x2020..=0x2021 | 0x2030..=0x2031 | 0x203b..=0x203c | 0x2042
        | 0x2047..=0x2049 | 0x2051 | 0x20dd..=0x20e0 | 0x20e2..=0x20e4
        | 0x2100..=0x2101 | 0x2103..=0x2109 | 0x210f | 0x2113..=0x2114 | 0x2116..=0x2117
        | 0x211e..=0x2123 | 0x2125 | 0x2127 | 0x2129 | 0x212e | 0x2135..=0x213f
        | 0x2145..=0x214a | 0x214c..=0x214d | 0x214f..=0x2189 | 0x218c..=0x218f
        | 0x221e | 0x2234..=0x2235 | 0x2300..=0x2307 | 0x230c..=0x231f | 0x2322..=0x232b
        | 0x237d..=0x239a | 0x23be..=0x23cd | 0x23cf | 0x23d1..=0x23db | 0x23e2..=0x2422
        | 0x2424..=0x24ff | 0x25a0..=0x2619 | 0x2620..=0x2767 | 0x2776..=0x2793
        | 0x2b12..=0x2b2f | 0x2b50..=0x2b59 | 0x2bb8..=0x2bff | 0x2e50..=0x2e51
        | 0x2e80..=0x2fff | 0x3000..=0x303f | 0x3040..=0x30ff | 0x3100..=0xa4cf
        | 0xa960..=0xa97f | 0xac00..=0xd7ff | 0xe000..=0xfaff | 0xfe10..=0xfe1f
        | 0xfe30..=0xfe6f | 0xff00..=0xffef | 0x1_f000..=0x1_faff | 0x2_0000..=0x3_fffd
    )
}

/// How an upright character takes the vertical line when its font has no
/// vertical form for it (UAX #50 Tr and Tu).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fallback {
    /// Drawn as it is.
    Plain,
    /// Brackets, dashes and the long vowel mark: turned clockwise.
    Turn,
    /// The ideographic comma and full stop: moved to the em's upper right.
    Corner,
}

pub(crate) fn fallback(c: char) -> Fallback {
    match c as u32 {
        0x3001 | 0x3002 | 0xff0c | 0xff0e => Fallback::Corner,
        0x2329..=0x232a
        | 0x3008..=0x3011
        | 0x3014..=0x301f
        | 0x3030
        | 0x30a0
        | 0x30fc
        | 0xfe59..=0xfe5e
        | 0xff08..=0xff09
        | 0xff0d
        | 0xff1a..=0xff1b
        | 0xff1c..=0xff1e
        | 0xff3b
        | 0xff3d
        | 0xff3f
        | 0xff5b..=0xff60
        | 0xffe3 => Fallback::Turn,
        _ => Fallback::Plain,
    }
}

/// A vertical line's runs in logical order: byte ranges and whether each
/// stands upright (one upright run for a combined text).
pub(crate) fn runs(line: &str, v: VerticalText) -> Vec<(Range<usize>, bool)> {
    if line.is_empty() {
        return Vec::new();
    }
    if v.combine {
        return vec![(0..line.len(), true)];
    }
    let all = |up| vec![(0..line.len(), up)];
    if v.sideways() {
        return all(false);
    }
    if v.orient == TextOrientation::Upright {
        return all(true);
    }
    let bytes: Vec<usize> = line
        .char_indices()
        .map(|(b, _)| b)
        .chain([line.len()])
        .collect();
    let mut out: Vec<(Range<usize>, bool)> = Vec::new();
    for (a, b) in og_i18n::graphemes(line) {
        let range = bytes[a]..bytes[b];
        let base = line[range.clone()].chars().next().unwrap_or(' ');
        // Spaces join the run before them.
        let up = match out.last() {
            Some(&(_, up)) if base.is_whitespace() => up,
            _ => upright_char(base),
        };
        match out.last_mut() {
            Some((r, u)) if *u == up => r.end = range.end,
            _ => out.push((range, up)),
        }
    }
    out
}

/// An upright cluster's advance down the line, em: its first character's
/// vertical advance in the font (`vmtx`), else one em.
pub(crate) fn upright_advance(m: &FontMetrics, cluster: &str) -> f32 {
    let c = cluster.chars().next().unwrap_or(' ');
    m.file
        .as_ref()
        .and_then(|f| f.vertical_advance(c))
        .unwrap_or(1.0)
}

/// The length of a line of `text` down a vertical line at `size`.
pub(crate) fn measure(
    m: &FontMetrics,
    text: &str,
    size: f32,
    tracking: f32,
    features: Features,
    v: VerticalText,
) -> f32 {
    if v.combine {
        return if text.is_empty() { 0.0 } else { size };
    }
    runs(text, v)
        .into_iter()
        .map(|(r, up)| {
            let run = &text[r];
            if up {
                upright_length(m, run, size, tracking)
            } else {
                m.measure_with(run, size, tracking, features)
            }
        })
        .sum()
}

/// An upright run's length: each cluster's advance and its tracking.
fn upright_length(m: &FontMetrics, run: &str, size: f32, tracking: f32) -> f32 {
    let chars: Vec<char> = run.chars().collect();
    og_i18n::graphemes(run)
        .into_iter()
        .map(|(a, b)| {
            let cluster: String = chars[a..b].iter().collect();
            if cluster.trim().is_empty() {
                // A space in an upright run keeps its own width.
                m.measure_with(&cluster, size, tracking, Features::NONE)
            } else {
                (upright_advance(m, &cluster) + tracking) * size
            }
        })
        .sum()
}

/// The longest `\n`-separated line, unwrapped.
pub(crate) fn widest_line(
    text: &str,
    m: &FontMetrics,
    size: f32,
    tracking: f32,
    features: Features,
    v: VerticalText,
) -> f32 {
    text.split('\n')
        .map(|l| measure(m, l, size, tracking, features, v))
        .fold(0.0, f32::max)
}

/// The longest unbreakable unit: a wrapping vertical text's least length.
pub(crate) fn longest_word(
    text: &str,
    m: &FontMetrics,
    size: f32,
    tracking: f32,
    features: Features,
    v: VerticalText,
) -> f32 {
    if v.combine {
        return measure(m, text, size, tracking, features, v);
    }
    text.split('\n')
        .flat_map(tokens)
        .filter(|(_, space)| !space)
        .map(|(w, _)| measure(m, w, size, tracking, features, v))
        .fold(0.0, f32::max)
}

/// A vertical text's lines at a length `max` (`None`: only at `\n`); a
/// combined text is one line.
pub(crate) fn break_lines(
    text: &str,
    m: &FontMetrics,
    size: f32,
    tracking: f32,
    features: Features,
    v: VerticalText,
    max: Option<f32>,
) -> Vec<Line> {
    if v.combine {
        let one = text.replace('\n', "");
        return vec![Line {
            width: measure(m, &one, size, tracking, features, v),
            text: one,
        }];
    }
    let mut lines = greedy_lines(
        text,
        max,
        &|t| measure(m, t, size, tracking, features, v),
        &|_, _| 0.0,
    );
    for l in &mut lines {
        l.width = measure(m, &l.text, size, tracking, features, v);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::flow::WritingMode;

    fn v(orient: TextOrientation) -> VerticalText {
        VerticalText {
            mode: WritingMode::VerticalRl,
            orient,
            combine: false,
        }
    }

    #[test]
    fn mixed_lines_stand_cjk_up_and_lay_latin_down() {
        let line = "縦書きでCSS 3を";
        let got: Vec<(&str, bool)> = runs(line, v(TextOrientation::Mixed))
            .into_iter()
            .map(|(r, up)| (&line[r], up))
            .collect();
        assert_eq!(got, [("縦書きで", true), ("CSS 3", false), ("を", true)]);
        assert_eq!(runs(line, v(TextOrientation::Upright)).len(), 1);
        let sideways = VerticalText {
            mode: WritingMode::SidewaysRl,
            ..v(TextOrientation::Mixed)
        };
        assert_eq!(runs(line, sideways), [(0..line.len(), false)]);
        assert_eq!(fallback('「'), Fallback::Turn);
        assert_eq!(fallback('。'), Fallback::Corner);
        assert!(!upright_char('ｱ'), "halfwidth katakana lie down");
    }

    #[test]
    fn upright_text_advances_an_em_per_character() {
        // Half-em advances: lying down, "ab" is 1 em; upright, 2.
        let m = FontMetrics::fixed(0.5);
        let up = measure(
            &m,
            "ab",
            10.0,
            0.0,
            Features::NONE,
            v(TextOrientation::Upright),
        );
        let down = measure(
            &m,
            "ab",
            10.0,
            0.0,
            Features::NONE,
            v(TextOrientation::Mixed),
        );
        assert_eq!((up, down), (20.0, 10.0));
        let combined = VerticalText {
            combine: true,
            ..v(TextOrientation::Mixed)
        };
        assert_eq!(measure(&m, "12", 10.0, 0.0, Features::NONE, combined), 10.0);
        let lines = break_lines(
            "あいうえお",
            &m,
            10.0,
            0.0,
            Features::NONE,
            v(TextOrientation::Mixed),
            Some(25.0),
        );
        let texts: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["あい", "うえ", "お"]);
    }
}
