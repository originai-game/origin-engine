//! A text's measurements along its line: its widths, lines and fit.

use super::*;

/// Greedy `lines` at `width` chosen again by `style` (CSS Text 4 §6.2):
/// `balance` breaks at the narrowest width that keeps their count, so the
/// lines come out as even as it allows; `pretty` breaks at the widest width
/// that keeps the count and gives the last line two words or more, when the
/// greedy last line is one word. `at` breaks at a width.
pub(super) fn rewrap(
    lines: Vec<Line>,
    style: WrapStyle,
    width: f32,
    at: impl Fn(f32) -> Vec<Line>,
) -> Vec<Line> {
    let n = lines.len();
    let words = |l: &[Line]| l.last().map_or(0, |l| l.text.split_whitespace().count());
    if style == WrapStyle::Pretty && words(&lines) > 1 {
        return lines;
    }
    // The narrowest width keeping `n` lines, to half a CSS px.
    let (mut lo, mut hi) = (0.0f32, width);
    while hi - lo > 0.5 {
        let mid = (lo + hi) / 2.0;
        if at(mid).len() <= n {
            hi = mid
        } else {
            lo = mid
        }
    }
    if style == WrapStyle::Balance {
        return at(hi);
    }
    // Pretty: the widest width from there that still ends in two words.
    let ok = |w: f32| {
        let l = at(w);
        (l.len() == n && words(&l) > 1).then_some(l)
    };
    let Some(mut best) = ok(hi) else { return lines };
    let (mut lo, mut top) = (hi, width);
    while top - lo > 0.5 {
        let mid = (lo + top) / 2.0;
        match ok(mid) {
            Some(l) => {
                best = l;
                lo = mid;
            }
            None => top = mid,
        }
    }
    best
}

/// A text's extent along its line: across, or down a vertical one.
pub(super) fn text_measure(t: UsedText<'_>, m: &FontMetrics, text: &str) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::measure(m, text, size, tr, ft, v),
        None => m.measure_with(text, size, tr, ft),
    }
}

/// A text's widest line along its line.
pub(super) fn text_widest(t: UsedText<'_>, m: &FontMetrics) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::widest_line(&t.text, m, size, tr, ft, v),
        None => widest_line(&t.text, m, size, tr, ft),
    }
}

/// A text's longest unbreakable unit along its line.
pub(super) fn text_longest(t: UsedText<'_>, m: &FontMetrics) -> f32 {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::longest_word(&t.text, m, size, tr, ft, v),
        None => longest_word(&t.text, m, size, tr, ft),
    }
}

/// A text's lines at `width` along its line (`None`: only at `\n`).
pub(crate) fn text_lines(t: UsedText<'_>, m: &FontMetrics, width: Option<f32>) -> Vec<Line> {
    let (size, tr, ft) = (t.step.size, t.tracking(), t.features());
    match t.vertical {
        Some(v) => crate::text::vertical::break_lines(&t.text, m, size, tr, ft, v, width),
        None => break_lines(&t.text, m, size, tr, ft, width),
    }
}

/// A text's preferred width (its widest line) and the least it can take:
/// its ellipsis, its longest word when it wraps, else all of it.
pub(super) fn text_widths(t: UsedText<'_>, fonts: &dyn Metrics) -> (f32, f32) {
    let m = t.metrics(fonts);
    let pref = text_widest(t, m);
    let min = if t.ellipsis {
        // It can shrink to its ellipsis.
        text_measure(t, m, t.cut.as_deref().unwrap_or_else(|| ellipsis_of(m))).min(pref)
    } else if t.wrap {
        text_longest(t, m)
    } else {
        pref
    };
    (pref, min)
}

/// A text shows whole at `width` at its step: one line no wider, or (wrapping)
/// no word wider and no more lines than its limit.
pub(super) fn fits(t: UsedText<'_>, fonts: &dyn Metrics, width: f32) -> bool {
    let m = t.metrics(fonts);
    if !t.wrap {
        return text_widest(t, m) <= width;
    }
    text_longest(t, m) <= width
        && (t.max_lines == 0 || text_lines(t, m, Some(width)).len() <= t.max_lines as usize)
}

/// A text's preferred extent along its line: its widest line.
pub(crate) fn text_preferred(t: UsedText<'_>, fonts: &dyn Metrics) -> f32 {
    super::text::text_widest(t, t.metrics(fonts))
}
