//! Text editing, shared by the one-line field and the multi-line area: the
//! caret and the selection (Shift with the arrows, Home and End; a press
//! places the caret, a drag selects; Ctrl or Cmd with A, C and X), typed
//! text (og-input's `og.TextInput` stream, or the keys before one exists)
//! replacing the selection, an input method's composition shown underlined
//! at the caret until it commits, and the caret's blink by the tick.
//!
//! Positions come from the same visual layout the text is drawn with
//! (`FontMetrics::visual`: kerning, bidi runs, shaped clusters), so a press
//! lands on the character drawn under it and a selection covers exactly
//! the glyphs it shows, in either direction.

use crate::input::UiKey;
use crate::layout::{FontRole, Rect};
use crate::text::FontMetrics;
use crate::ui::Ui;
use og_i18n::Direction;

/// A glyph's box on its line, CSS px from the line's start.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphBox {
    /// The logical index of the first character it draws.
    pub index: usize,
    pub x0: f32,
    pub x1: f32,
    pub rtl: bool,
}

/// The boxes of a line's glyphs, left to right.
pub(crate) fn boxes(
    m: &FontMetrics,
    line: &str,
    dir: Direction,
    size: f32,
    tracking: f32,
) -> Vec<GlyphBox> {
    let mut x = 0.0f32;
    m.visual(line, dir, tracking)
        .into_iter()
        .map(|g| {
            let w = g.advance * size;
            let b = GlyphBox {
                index: g.index,
                x0: x,
                x1: x + w,
                rtl: g.rtl,
            };
            x += w;
            b
        })
        .collect()
}

/// Where the caret before logical character `k` sits on a line (CSS px
/// from its start): at the leading edge of the glyph drawing `k` (its left
/// in a left-to-right run, its right in a right-to-left one), or after the
/// glyph before it when `k` ends the line or is inside a shaped cluster.
pub(crate) fn caret_x(boxes: &[GlyphBox], k: usize) -> f32 {
    if let Some(b) = boxes.iter().find(|b| b.index == k) {
        return if b.rtl { b.x1 } else { b.x0 };
    }
    match boxes.iter().filter(|b| b.index < k).max_by_key(|b| b.index) {
        Some(b) if b.rtl => b.x0,
        Some(b) => b.x1,
        None => 0.0,
    }
}

/// The character boundary (0 to `n`) whose caret is nearest `x`.
pub(crate) fn nearest(boxes: &[GlyphBox], n: usize, x: f32) -> usize {
    let mut best = (0usize, f32::MAX);
    for k in 0..=n {
        let d = (caret_x(boxes, k) - x).abs();
        if d < best.1 {
            best = (k, d);
        }
    }
    best.0
}

/// A multi-line text's lines at `width`: character ranges `[start, end)`
/// (a `\n` ends a line and belongs to none), broken greedily after the
/// spaces between words, a word wider than the line between characters.
pub(crate) fn wrap(
    chars: &[char],
    m: &FontMetrics,
    size: f32,
    tracking: f32,
    width: f32,
) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let w = |a: usize, b: usize| {
        let s: String = chars[a..b].iter().collect();
        m.measure_tracked(s.trim_end_matches(' '), size, tracking)
    };
    let mut p = 0;
    loop {
        let end = chars[p..]
            .iter()
            .position(|&c| c == '\n')
            .map_or(chars.len(), |i| p + i);
        // Words with their trailing spaces.
        let mut words = Vec::new();
        let mut i = p;
        while i < end {
            let mut j = i;
            while j < end && chars[j] != ' ' {
                j += 1;
            }
            while j < end && chars[j] == ' ' {
                j += 1;
            }
            words.push((i, j));
            i = j;
        }
        let mut start = p;
        for (a, b) in words {
            if a > start && w(start, b) > width {
                lines.push((start, a));
                start = a;
            }
            // A word wider than the line: between characters.
            while w(start, b) > width && b - start > 1 {
                let mut k = start + 1;
                while k < b && w(start, k + 1) <= width {
                    k += 1;
                }
                lines.push((start, k));
                start = k;
            }
        }
        lines.push((start, end));
        if end >= chars.len() {
            return lines;
        }
        p = end + 1;
    }
}

/// How an editable text is laid out, for placing the caret and selecting.
pub(crate) struct EditDesc {
    pub multiline: bool,
    pub max_chars: u32,
    pub role: FontRole,
    pub size: f32,
    pub tracking: f32,
    pub line_height: f32,
    /// The text's box last frame (the control's rectangle less its padding).
    pub inner: Option<Rect>,
}

/// What an edit did this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Edited {
    pub changed: bool,
    pub submitted: bool,
    /// Enter (one line) or Escape: focus leaves.
    pub left: bool,
}

/// The ticks the caret stays shown, then hidden, while blinking: 530 ms,
/// the platforms' usual half period.
pub(crate) fn blink_half(ticks_per_second: u32) -> u64 {
    ((ticks_per_second.max(1) as f32 * 0.53).round() as u64).max(1)
}

impl Ui {
    /// Edits `value` as the focused text control `key`: pointer, typed and
    /// composed text, editing keys. The caret, selection anchor and
    /// composition live in the UI state (and its saved bytes).
    pub(crate) fn edit(&mut self, key: u64, value: &mut String, d: &EditDesc) -> Edited {
        let mut out = Edited::default();
        let dir = self.direction;
        let tick = self.state.tick;
        let mut chars: Vec<char> = value.chars().collect();
        let before = chars.clone();
        // Focus arrived: the caret at the end (a press moves it below).
        if self.state.edit_key != key {
            self.state.edit_key = key;
            self.state.caret = chars.len();
            self.state.anchor = chars.len();
            self.state.composition.clear();
            self.state.edit_scroll = 0.0;
            self.state.edit_tick = tick;
        }
        let mut caret = self.state.caret.min(chars.len());
        let mut anchor = self.state.anchor.min(chars.len());
        let mut composition = std::mem::take(&mut self.state.composition);
        let mut clipboard = None;
        {
            let m = &self.fonts.for_text(d.role, value).metrics;
            let scroll = self.state.edit_scroll;
            let (size, tracking) = (d.size, d.tracking);
            // The text's lines now: (start, end, boxes, y from the text's top).
            let layout = |chars: &[char]| -> Vec<(usize, usize, Vec<GlyphBox>)> {
                let ranges = if d.multiline {
                    let width = d.inner.map_or(f32::MAX, |r| r.width.max(1.0));
                    wrap(chars, m, size, tracking, width)
                } else {
                    vec![(0, chars.len())]
                };
                ranges
                    .into_iter()
                    .map(|(a, b)| {
                        let s: String = chars[a..b].iter().collect();
                        (a, b, boxes(m, &s, dir, size, tracking))
                    })
                    .collect()
            };
            // The line holding the caret, and the caret's x on it.
            let locate = |lines: &[(usize, usize, Vec<GlyphBox>)], k: usize| -> (usize, f32) {
                let li = lines
                    .iter()
                    .rposition(|l| l.0 <= k && k <= l.1)
                    .unwrap_or(0);
                let (a, _, bx) = &lines[li];
                (li, caret_x(bx, k - a))
            };
            let line_x0 = |l: &(usize, usize, Vec<GlyphBox>), inner: &Rect| {
                let w = l.2.last().map_or(0.0, |b| b.x1);
                if dir == Direction::Rtl && d.multiline {
                    inner.right() - w
                } else {
                    inner.x
                }
            };
            // A press places the caret, a drag (the press held) selects.
            let pressed_here = self.input.pressed && self.state.active == key;
            let dragging = !self.input.pressed && self.input.down && self.state.active == key;
            if (pressed_here || dragging)
                && let Some(inner) = d.inner
            {
                let p = self.input.pointer;
                let lines = layout(&chars);
                let (y0, x0) = if d.multiline {
                    (inner.y - scroll, 0.0)
                } else {
                    (inner.y, -scroll)
                };
                let li = (((p.y - y0) / d.line_height).floor().max(0.0) as usize)
                    .min(lines.len().saturating_sub(1));
                let l = &lines[li];
                let x = p.x - line_x0(l, &inner) - x0;
                let k = l.0 + nearest(&l.2, l.1 - l.0, x);
                caret = k;
                if pressed_here && !self.input.shift {
                    anchor = k;
                }
            }
            // An input method's composition replaces the selection as it starts.
            if let Some(c) = &self.input.composition {
                if !c.is_empty() && composition.is_empty() && caret != anchor {
                    let (a, b) = (caret.min(anchor), caret.max(anchor));
                    chars.drain(a..b);
                    caret = a;
                    anchor = a;
                }
                composition = c.clone();
            }
            // Typed text replaces the selection.
            let typed: Vec<char> = self
                .input
                .text
                .chars()
                .filter(|&c| d.multiline || c != '\n')
                .collect();
            if !typed.is_empty() {
                let (a, b) = (caret.min(anchor), caret.max(anchor));
                chars.drain(a..b);
                caret = a;
                for c in typed {
                    if d.max_chars == 0 || chars.len() < d.max_chars as usize {
                        chars.insert(caret, c);
                        caret += 1;
                    }
                }
                anchor = caret;
            }
            let shift = self.input.shift;
            let ctrl = self.input.ctrl;
            for k in self.input.keys.iter().copied() {
                let (a, b) = (caret.min(anchor), caret.max(anchor));
                let selected = a != b;
                let mut moved = None;
                match k {
                    UiKey::Backspace | UiKey::Delete if selected => {
                        chars.drain(a..b);
                        caret = a;
                        anchor = a;
                    }
                    UiKey::Backspace if caret > 0 => {
                        chars.remove(caret - 1);
                        caret -= 1;
                        anchor = caret;
                    }
                    UiKey::Delete if caret < chars.len() => {
                        chars.remove(caret);
                        anchor = caret;
                    }
                    UiKey::Left if selected && !shift => moved = Some(a),
                    UiKey::Right if selected && !shift => moved = Some(b),
                    UiKey::Left => moved = Some(caret.saturating_sub(1)),
                    UiKey::Right => moved = Some((caret + 1).min(chars.len())),
                    UiKey::Home | UiKey::End => {
                        let lines = layout(&chars);
                        let (li, _) = locate(&lines, caret);
                        moved = Some(if k == UiKey::Home {
                            lines[li].0
                        } else {
                            lines[li].1
                        });
                    }
                    UiKey::Up | UiKey::Down if d.multiline => {
                        let lines = layout(&chars);
                        let (li, x) = locate(&lines, caret);
                        let to = if k == UiKey::Up {
                            li.checked_sub(1)
                        } else {
                            (li + 1 < lines.len()).then_some(li + 1)
                        };
                        moved = Some(match to {
                            Some(t) => {
                                let l = &lines[t];
                                l.0 + nearest(&l.2, l.1 - l.0, x)
                            }
                            None if k == UiKey::Up => 0,
                            None => chars.len(),
                        });
                    }
                    UiKey::KeyA if ctrl => {
                        anchor = 0;
                        caret = chars.len();
                    }
                    UiKey::KeyC | UiKey::KeyX if ctrl && selected => {
                        clipboard = Some(chars[a..b].iter().collect::<String>());
                        if k == UiKey::KeyX {
                            chars.drain(a..b);
                            caret = a;
                            anchor = a;
                        }
                    }
                    UiKey::Enter | UiKey::NumpadEnter if d.multiline => {
                        chars.drain(a..b);
                        caret = a;
                        if d.max_chars == 0 || chars.len() < d.max_chars as usize {
                            chars.insert(caret, '\n');
                            caret += 1;
                        }
                        anchor = caret;
                    }
                    UiKey::Enter | UiKey::NumpadEnter => {
                        out.submitted = true;
                        out.left = true;
                    }
                    UiKey::Escape => out.left = true,
                    _ => {}
                }
                if let Some(to) = moved {
                    caret = to;
                    if !shift {
                        anchor = to;
                    }
                }
            }
            // Keep the caret in view: horizontally in a field, by lines in an area.
            if let Some(inner) = d.inner {
                let mut shown = chars.clone();
                let comp: Vec<char> = composition.chars().collect();
                let at = caret.min(shown.len());
                shown.splice(at..at, comp.iter().copied());
                let lines = layout(&shown);
                let (li, x) = locate(&lines, at + comp.len());
                let mut s = self.state.edit_scroll;
                if d.multiline {
                    let top = li as f32 * d.line_height;
                    let bottom = top + d.line_height;
                    if top < s {
                        s = top;
                    }
                    if bottom > s + inner.height {
                        s = bottom - inner.height;
                    }
                    let content = lines.len() as f32 * d.line_height;
                    s = s.min((content - inner.height).max(0.0)).max(0.0);
                } else {
                    let width = lines[0].2.last().map_or(0.0, |b| b.x1);
                    let room = (inner.width - 1.0).max(1.0);
                    if x - s > room {
                        s = x - room;
                    }
                    if x < s {
                        s = x;
                    }
                    s = s.min((width - room).max(0.0)).max(0.0);
                }
                self.state.edit_scroll = s;
            }
        }
        if chars != before {
            *value = chars.iter().collect();
            out.changed = true;
        }
        if out.changed
            || caret != self.state.caret
            || anchor != self.state.anchor
            || composition != self.state.composition
        {
            self.state.edit_tick = tick;
        }
        self.state.caret = caret;
        self.state.anchor = anchor;
        self.state.composition = composition;
        if clipboard.is_some() {
            self.clipboard = clipboard;
        }
        if out.left {
            self.state.focus = 0;
            self.state.composition.clear();
        }
        out
    }

    /// Whether the caret shows this tick: always under reduced motion, else
    /// for [`blink_half`] ticks after the last edit or move, then hidden as
    /// long, by the tick.
    pub(crate) fn caret_shown(&self) -> bool {
        if self.theme.reduced_motion {
            return true;
        }
        let half = blink_half(self.ticks_per_second);
        (self.state.tick.saturating_sub(self.state.edit_tick) / half).is_multiple_of(2)
    }
}
