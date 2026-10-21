//! Text editing, shared by the one-line field and the multi-line area: the
//! caret and the selection (Shift with the arrows, Home and End; by words
//! with the arrows, Backspace and Delete; Ctrl with Home and End to the
//! text's ends; a press places the caret, a drag selects and scrolls the
//! text past its edge, a double click selects a word and a triple click the
//! line; the shortcut key with A, C and X), typed text (og-input's
//! `og.TextInput` stream, or the keys before one exists) replacing the
//! selection, an input method's composition shown underlined at the caret
//! until it commits, undo and redo (the shortcut key with Z, Shift+Z and Y;
//! a bounded history per control, typing grouped by words), and the
//! caret's blink by the tick.
//!
//! The keys follow the platform ([`crate::Viewport::mac_keys`]: the web
//! shell's browser on an Apple platform, a native host on macOS): on a Mac
//! Command is the shortcut key and goes to the line's ends with ←/→ (and
//! deletes to them with Backspace and Delete, ↑/↓ the text's ends in an
//! area), Option moves and deletes by words, and Control takes the emacs
//! keys (A and E the line's ends, F and B a character, N and P a line, D
//! and H delete a character, K to the paragraph's end, U to its start, W
//! the word before); elsewhere Control is the shortcut key and moves by
//! words (Alt too), and Command only takes the shortcuts.
//!
//! The caret stands between grapheme clusters ([`clusters`]): an arrow,
//! Backspace and Delete step over a base and its marks, an emoji with its
//! modifiers and joiners, at once, and a press lands beside one. ← and →
//! move as they point: on in the text in a right-to-left run, by clusters
//! and by words alike (word moves stay UAX #29's words).
//!
//! Positions come from the same visual layout the text is drawn with
//! (`FontMetrics::visual`: kerning, bidi runs, shaped clusters), so a press
//! lands on the character drawn under it and a selection covers exactly
//! the glyphs it shows, in either direction.

use crate::input::UiKey;
use crate::layout::{FontRole, Rect};
use crate::text::FontMetrics;
use crate::ui::Ui;
use og_core::glam::Vec2;
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

/// The grapheme cluster boundary of a line (0 to its length) whose caret
/// is nearest `x`: a press never lands between a base and its marks.
pub(crate) fn nearest(boxes: &[GlyphBox], line: &[char], x: f32) -> usize {
    let mut best = (0usize, f32::MAX);
    for k in clusters(line) {
        let d = (caret_x(boxes, k) - x).abs();
        if d < best.1 {
            best = (k, d);
        }
    }
    best.0
}

/// A text's grapheme cluster boundaries (UAX #29, extended: og-i18n's
/// [`og_i18n::graphemes`]), 0 and each cluster's end: where the caret may
/// stand. A base and its combining marks, an emoji with its modifiers and
/// joiners, a Hangul syllable of jamo, an Indic consonant with its vowel
/// signs are each one step of the caret and one Backspace.
pub(crate) fn clusters(chars: &[char]) -> Vec<usize> {
    let s: String = chars.iter().collect();
    let mut out = vec![0];
    out.extend(og_i18n::graphemes(&s).into_iter().map(|(_, b)| b));
    out
}

/// The cluster boundary after `k` (the text's end at most).
pub(crate) fn next_cluster(chars: &[char], k: usize) -> usize {
    clusters(chars)
        .into_iter()
        .find(|&b| b > k)
        .unwrap_or(chars.len())
        .min(chars.len())
}

/// The cluster boundary before `k` (0 at least).
pub(crate) fn prev_cluster(chars: &[char], k: usize) -> usize {
    clusters(chars)
        .into_iter()
        .rev()
        .find(|&b| b < k)
        .unwrap_or(0)
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
    // Where a word wider than the line may break: between clusters.
    let stops = clusters(chars);
    let after = |k: usize| {
        stops
            .iter()
            .copied()
            .find(|&b| b > k)
            .unwrap_or(chars.len())
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
            // A word wider than the line: between grapheme clusters.
            while w(start, b) > width && after(start) < b {
                let mut k = after(start);
                while k < b && w(start, after(k).min(b)) <= width {
                    k = after(k).min(b);
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

/// The segment holding character `i` (UAX #29 words: og-i18n's
/// [`og_i18n::word_segments`]).
fn segment_at(segs: &[og_i18n::WordSegment], i: usize) -> Option<usize> {
    segs.iter().position(|s| s.start <= i && i < s.end)
}

/// Spaces and line breaks: what a word move passes over.
fn blank(s: &og_i18n::WordSegment) -> bool {
    matches!(s.kind, og_i18n::WordKind::Space | og_i18n::WordKind::Break)
}

/// Where a word move left puts the caret from `k`: back over spaces, then
/// to the start of the word (or the run of punctuation) before it. Words
/// are UAX #29's ([`og_i18n::word_segments`]): a Han character or a Thai
/// cluster is one.
pub(crate) fn word_left(chars: &[char], k: usize) -> usize {
    let k = k.min(chars.len());
    if k == 0 {
        return 0;
    }
    let segs = og_i18n::word_segments(chars);
    let Some(mut i) = segment_at(&segs, k - 1) else {
        return 0;
    };
    while blank(&segs[i]) {
        if i == 0 {
            return 0;
        }
        i -= 1;
    }
    segs[i].start
}

/// Where a word move right puts the caret from `k`: on over spaces, then to
/// the end of the word (or the run of punctuation) after it.
pub(crate) fn word_right(chars: &[char], k: usize) -> usize {
    let n = chars.len();
    let segs = og_i18n::word_segments(chars);
    let Some(mut i) = segment_at(&segs, k.min(n)) else {
        return n;
    };
    while blank(&segs[i]) {
        i += 1;
        if i == segs.len() {
            return n;
        }
    }
    segs[i].end
}

/// The word a double click at boundary `k` selects: the segment (a word,
/// spaces or punctuation) holding the character after `k`, or the one
/// before it at a word's end or the text's.
pub(crate) fn word_at(chars: &[char], k: usize) -> (usize, usize) {
    let n = chars.len();
    if n == 0 {
        return (0, 0);
    }
    let k = k.min(n);
    let space = |c: char| c.is_whitespace();
    let i = if k < n && (!space(chars[k]) || k == 0 || space(chars[k - 1])) {
        k
    } else {
        k - 1
    };
    if chars[i] == '\n' {
        return (k, k);
    }
    let segs = og_i18n::word_segments(chars);
    match segment_at(&segs, i) {
        Some(j) if segs[j].kind != og_i18n::WordKind::Break => (segs[j].start, segs[j].end),
        _ => (k, k),
    }
}

/// Whether typing `inserted` at `at` of `after` (the text with it) starts
/// a new word for the undo history: after spaces, or next to a word of one
/// character (Han and Hiragana, where UAX #29 puts a boundary between every
/// two; an input method commits them a phrase at a time). Thai and other
/// scripts without spaces group to the next space, as Latin does.
fn starts_word(last: &str, after: &[char], at: usize) -> bool {
    let first = after.get(at).copied();
    if last.ends_with(char::is_whitespace) {
        return first.is_some_and(|c| !c.is_whitespace());
    }
    let (Some(prev), Some(next)) = (at.checked_sub(1).and_then(|i| after.get(i)), first) else {
        return false;
    };
    (og_i18n::word_per_character(*prev) || og_i18n::word_per_character(next))
        && prev.is_alphanumeric()
        && next.is_alphanumeric()
}

/// The line a triple click at `k` selects: from the line break before it
/// to the one after it (a field's whole text).
pub(crate) fn line_at(chars: &[char], k: usize) -> (usize, usize) {
    let k = k.min(chars.len());
    let a = chars[..k]
        .iter()
        .rposition(|&c| c == '\n')
        .map_or(0, |i| i + 1);
    let b = chars[k..]
        .iter()
        .position(|&c| c == '\n')
        .map_or(chars.len(), |i| k + i);
    (a, b)
}

/// The presses a text control counts into double and triple clicks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Click {
    /// The tick of the last press, and where it landed.
    pub tick: u64,
    pub at: Vec2,
    /// Presses in a row (1 a click, 2 a double click selecting words, 3 a
    /// triple click selecting lines; 0 none).
    pub count: u8,
    /// The press moved past the slop: the next press starts again.
    pub moved: bool,
    /// The word or line the press selected (a drag extends by them).
    pub origin: (usize, usize),
}

/// How far a press may land from the last one, CSS px, and still count
/// as the next click of a double or triple click (and how far it may move
/// before it is a drag).
const CLICK_SLOP: f32 = 4.0;

/// The ticks within which a press is the next click of a double or triple
/// click: 500 ms, the platforms' usual default.
pub(crate) fn click_ticks(ticks_per_second: u32) -> u64 {
    ((ticks_per_second.max(1) as f32 * 0.5).round() as u64).max(1)
}

/// The kind of an edit, for grouping the undo history: typing on groups
/// with the typing before it (a word and its spaces at a time), Backspace
/// with the Backspaces before it, Delete with the Deletes; anything else
/// (a cut, a paste, a line break, an undo) stands alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EditKind {
    #[default]
    Other = 0,
    Typing = 1,
    Back = 2,
    Forward = 3,
}

impl EditKind {
    pub fn from_u8(v: u8) -> EditKind {
        match v {
            1 => EditKind::Typing,
            2 => EditKind::Back,
            3 => EditKind::Forward,
            _ => EditKind::Other,
        }
    }
}

/// One step of a text control's undo history: at character `at`, the text
/// `removed` became `inserted`, the caret and anchor moving from `before`
/// to `after`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Change {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
    pub kind: EditKind,
    pub before: (usize, usize),
    pub after: (usize, usize),
}

/// A text control's undo and redo stacks.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct History {
    pub undo: Vec<Change>,
    pub redo: Vec<Change>,
    /// The hash of the text after the last change it knows: a text that
    /// hashes otherwise was set by the game, and the history starts again.
    pub seen: u64,
    /// The last change may still grow (typing or deleting on).
    pub open: bool,
    /// The tick it was last edited (the oldest goes first past
    /// [`HISTORY_FIELDS`]).
    pub used: u64,
}

/// Steps each text control keeps.
pub(crate) const HISTORY_DEPTH: usize = 100;
/// Text controls whose history is kept (the least recently edited goes).
pub(crate) const HISTORY_FIELDS: usize = 8;

/// FNV-1a over a text's characters.
pub(crate) fn text_hash(chars: &[char]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &c in chars {
        for b in (c as u32).to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

impl History {
    /// Records the edit from `before` to `after` (the characters before
    /// and after it, the caret and anchor before and after it), grouping it
    /// with the last change when both are typing on, or deleting on, at the
    /// same place.
    pub fn record(
        &mut self,
        before: &[char],
        after: &[char],
        kind: EditKind,
        sel_before: (usize, usize),
        sel_after: (usize, usize),
    ) {
        let p = before.iter().zip(after).take_while(|(a, b)| a == b).count();
        let room = before.len().min(after.len()) - p;
        let s = before
            .iter()
            .rev()
            .zip(after.iter().rev())
            .take(room)
            .take_while(|(a, b)| a == b)
            .count();
        let removed: String = before[p..before.len() - s].iter().collect();
        let inserted: String = after[p..after.len() - s].iter().collect();
        self.redo.clear();
        if self.open
            && let Some(last) = self.undo.last_mut()
            && last.kind == kind
        {
            let grows = match kind {
                EditKind::Typing => {
                    removed.is_empty()
                        && p == last.at + last.inserted.chars().count()
                        && !starts_word(&last.inserted, after, p)
                }
                EditKind::Back => inserted.is_empty() && p + removed.chars().count() == last.at,
                EditKind::Forward => inserted.is_empty() && p == last.at,
                EditKind::Other => false,
            };
            if grows {
                match kind {
                    EditKind::Typing => last.inserted.push_str(&inserted),
                    EditKind::Back => {
                        last.removed.insert_str(0, &removed);
                        last.at = p;
                    }
                    _ => last.removed.push_str(&removed),
                }
                last.after = sel_after;
                return;
            }
        }
        self.undo.push(Change {
            at: p,
            removed,
            inserted,
            kind,
            before: sel_before,
            after: sel_after,
        });
        if self.undo.len() > HISTORY_DEPTH {
            self.undo.remove(0);
        }
        self.open = kind != EditKind::Other;
    }

    /// Undoes the last change in `chars`: the caret and anchor it leaves.
    pub fn undo(&mut self, chars: &mut Vec<char>) -> Option<(usize, usize)> {
        let c = self.undo.pop()?;
        let n = c.inserted.chars().count();
        if c.at + n > chars.len() {
            *self = History::default();
            return None;
        }
        chars.splice(c.at..c.at + n, c.removed.chars());
        self.open = false;
        let to = c.before;
        self.redo.push(c);
        Some(to)
    }

    /// Redoes the last change undone: the caret and anchor it leaves.
    pub fn redo(&mut self, chars: &mut Vec<char>) -> Option<(usize, usize)> {
        let c = self.redo.pop()?;
        let n = c.removed.chars().count();
        if c.at + n > chars.len() {
            *self = History::default();
            return None;
        }
        chars.splice(c.at..c.at + n, c.inserted.chars());
        self.open = false;
        let to = c.after;
        self.undo.push(c);
        Some(to)
    }
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
    /// composed text, editing keys, undo and redo. The caret, selection
    /// anchor, composition, click count and history live in the UI state
    /// (and its saved bytes).
    pub(crate) fn edit(&mut self, key: u64, value: &mut String, d: &EditDesc) -> Edited {
        let mut out = Edited::default();
        let dir = self.direction;
        let tick = self.state.tick;
        let mut chars: Vec<char> = value.chars().collect();
        let mut before = chars.clone();
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
        let mut sel_before = (caret, anchor);
        let mut composition = std::mem::take(&mut self.state.composition);
        let mut clipboard = None;
        // This control's history; a text the game set since starts it again.
        let mut history = self.state.histories.remove(&key).unwrap_or_default();
        if history.seen != text_hash(&chars) {
            history = History::default();
        }
        let mut kind: Option<EditKind> = None;
        let note = |kind: &mut Option<EditKind>, k: EditKind| {
            *kind = Some(match *kind {
                Some(was) if was != k => EditKind::Other,
                _ => k,
            });
        };
        let mut click = self.state.click;
        // A drag past the edge scrolled the text this tick: it owns the scroll.
        let mut autoscrolled = false;
        {
            let m = &self.fonts.for_text(d.role, value).metrics;
            let mut scroll = self.state.edit_scroll;
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
            // A press places the caret, a drag (the press held) selects; a
            // second press in a row selects a word, a third the line, and a
            // drag from them extends by words or lines.
            let pressed_here = self.input.pressed && self.state.active == key;
            let dragging = !self.input.pressed && self.input.down && self.state.active == key;
            if (pressed_here || dragging)
                && let Some(inner) = d.inner
            {
                let mut p = self.input.pointer;
                let lines = layout(&chars);
                // A drag past the text's edge scrolls it, faster the further
                // out, and selects up to the edge.
                if dragging {
                    let lh = d.line_height.max(1.0);
                    let speed =
                        |over: f32| over.signum() * (over.abs() * 0.25).clamp(lh * 0.25, lh * 2.0);
                    if d.multiline {
                        let over = if p.y < inner.y {
                            p.y - inner.y
                        } else if p.y > inner.bottom() {
                            p.y - inner.bottom()
                        } else {
                            0.0
                        };
                        if over != 0.0 {
                            let content = lines.len() as f32 * d.line_height;
                            let most = (content - inner.height).max(0.0);
                            scroll = (scroll + speed(over)).clamp(0.0, most);
                            p.y = p.y.clamp(inner.y, (inner.bottom() - 0.01).max(inner.y));
                            autoscrolled = true;
                        }
                    } else {
                        let over = if p.x < inner.x {
                            p.x - inner.x
                        } else if p.x > inner.right() {
                            p.x - inner.right()
                        } else {
                            0.0
                        };
                        if over != 0.0 {
                            let width = lines[0].2.last().map_or(0.0, |b| b.x1);
                            let room = (inner.width - 1.0).max(1.0);
                            let most = (width - room).max(0.0);
                            scroll = (scroll + speed(over)).clamp(0.0, most);
                            p.x = p.x.clamp(inner.x, inner.right());
                            autoscrolled = true;
                        }
                    }
                    self.state.edit_scroll = scroll;
                }
                let (y0, x0) = if d.multiline {
                    (inner.y - scroll, 0.0)
                } else {
                    (inner.y, -scroll)
                };
                let li = (((p.y - y0) / d.line_height).floor().max(0.0) as usize)
                    .min(lines.len().saturating_sub(1));
                let l = &lines[li];
                let x = p.x - line_x0(l, &inner) - x0;
                let k = l.0 + nearest(&l.2, &chars[l.0..l.1], x);
                let unit = |count: u8, k: usize| match count {
                    2 => word_at(&chars, k),
                    3 if d.multiline => line_at(&chars, k),
                    3 => (0, chars.len()),
                    _ => (k, k),
                };
                if pressed_here {
                    let near = (p - click.at).length() <= CLICK_SLOP;
                    let soon =
                        tick.saturating_sub(click.tick) <= click_ticks(self.ticks_per_second);
                    click.count = if click.count > 0 && !click.moved && near && soon {
                        (click.count + 1).min(3)
                    } else {
                        1
                    };
                    click.tick = tick;
                    click.at = p;
                    click.moved = false;
                    if self.input.shift {
                        click.count = 1;
                        caret = k;
                    } else {
                        let (a, b) = unit(click.count, k);
                        click.origin = (a, b);
                        anchor = a;
                        caret = b;
                    }
                } else {
                    if (self.input.pointer - click.at).length() > CLICK_SLOP {
                        click.moved = true;
                    }
                    if click.count >= 2 {
                        let (o0, o1) = click.origin;
                        let (a, b) = unit(click.count, k);
                        if a < o0 {
                            anchor = o1;
                            caret = a;
                        } else {
                            anchor = o0;
                            caret = b.max(o1);
                        }
                    } else {
                        caret = k;
                    }
                }
            }
            // An input method's composition replaces the selection as it starts.
            if let Some(c) = &self.input.composition {
                if !c.is_empty() && composition.is_empty() && caret != anchor {
                    let (a, b) = (caret.min(anchor), caret.max(anchor));
                    chars.drain(a..b);
                    caret = a;
                    anchor = a;
                    note(&mut kind, EditKind::Typing);
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
                note(&mut kind, EditKind::Typing);
            }
            let shift = self.input.shift;
            let (ctrl, meta, alt) = (self.input.ctrl, self.input.meta, self.input.alt);
            let mac = self.viewport.mac_keys;
            // The shortcut key (A, C, X, Z, Y): Command under mac keys;
            // elsewhere Control, and Command too (the OS's own there).
            let shortcut = if mac { meta } else { ctrl || meta };
            // By words: Option under mac keys; elsewhere Control (or Alt).
            let word = if mac { alt && !meta } else { ctrl || alt };
            // To the line's ends: Command with the arrows, Backspace and
            // Delete under mac keys.
            let line = mac && meta;
            // Control under mac keys: the emacs keys a Mac text field
            // honours (A and E the line's ends, F and B a character on and
            // back, N and P a line down and up, D and H delete a character
            // on and back, K to the line's end, U to its start, W the word
            // before the caret).
            let emacs = mac && ctrl && !meta;
            // The line holding the caret: its start and end.
            let line_of = |chars: &[char], caret: usize| {
                let lines = layout(chars);
                let (li, _) = locate(&lines, caret);
                (lines[li].0, lines[li].1)
            };
            // Whether the caret at `k` stands in a right-to-left run (the
            // glyph after it, or before it at a line's end; the paragraph's
            // direction in an empty text): there ← moves on in the text and
            // → back, by clusters and by words alike, as the arrow points.
            let rtl_at = |chars: &[char], k: usize| {
                let lines = layout(chars);
                let (li, _) = locate(&lines, k);
                let (a, b, bx) = &lines[li];
                let at = |i: usize| {
                    bx.iter()
                        .filter(|g| g.index <= i)
                        .max_by_key(|g| g.index)
                        .map(|g| g.rtl)
                };
                let line: String = chars[*a..*b].iter().collect();
                let run = if k < *b {
                    at(k - a)
                } else if k > *a {
                    at(k - a - 1)
                } else {
                    None
                };
                run.unwrap_or(og_i18n::paragraph_direction(&line, dir) == Direction::Rtl)
            };
            for k in self.input.keys.iter().copied() {
                let (a, b) = (caret.min(anchor), caret.max(anchor));
                let selected = a != b;
                let mut moved = None;
                // The emacs keys that are another key's edit.
                let k = match k {
                    UiKey::KeyD if emacs => UiKey::Delete,
                    UiKey::KeyH if emacs => UiKey::Backspace,
                    UiKey::KeyN if emacs && d.multiline => UiKey::Down,
                    UiKey::KeyP if emacs && d.multiline => UiKey::Up,
                    k => k,
                };
                match k {
                    UiKey::Backspace | UiKey::Delete if selected => {
                        chars.drain(a..b);
                        caret = a;
                        anchor = a;
                        note(&mut kind, EditKind::Other);
                    }
                    UiKey::Backspace if caret > 0 => {
                        let from = if line {
                            let start = line_of(&chars, caret).0;
                            if start < caret {
                                start
                            } else {
                                prev_cluster(&chars, caret)
                            }
                        } else if word {
                            word_left(&chars, caret)
                        } else {
                            prev_cluster(&chars, caret)
                        };
                        chars.drain(from..caret);
                        caret = from;
                        anchor = caret;
                        note(&mut kind, EditKind::Back);
                    }
                    UiKey::Delete if caret < chars.len() => {
                        let to = if line {
                            let end = line_of(&chars, caret).1;
                            if end > caret {
                                end
                            } else {
                                next_cluster(&chars, caret)
                            }
                        } else if word {
                            word_right(&chars, caret)
                        } else {
                            next_cluster(&chars, caret)
                        };
                        chars.drain(caret..to);
                        anchor = caret;
                        note(&mut kind, EditKind::Forward);
                    }
                    // ← and → as they point: on in the text in a
                    // right-to-left run. Command under mac keys: the line's
                    // ends; the word key: by words; a selection collapses
                    // to its side; else a grapheme cluster.
                    UiKey::Left | UiKey::Right => {
                        let on = (k == UiKey::Right) != rtl_at(&chars, caret);
                        moved = Some(if line {
                            let (start, end) = line_of(&chars, caret);
                            if on { end } else { start }
                        } else if word {
                            if on {
                                word_right(&chars, caret)
                            } else {
                                word_left(&chars, caret)
                            }
                        } else if selected && !shift {
                            if on { b } else { a }
                        } else if on {
                            next_cluster(&chars, caret)
                        } else {
                            prev_cluster(&chars, caret)
                        });
                    }
                    // Control+F and B under mac keys: a cluster on and back
                    // in the text (whichever way it runs).
                    UiKey::KeyF if emacs => {
                        moved = Some(if selected && !shift {
                            b
                        } else {
                            next_cluster(&chars, caret)
                        })
                    }
                    UiKey::KeyB if emacs => {
                        moved = Some(if selected && !shift {
                            a
                        } else {
                            prev_cluster(&chars, caret)
                        })
                    }
                    // Control+N and P in a field: the text's ends.
                    UiKey::KeyN if emacs => moved = Some(chars.len()),
                    UiKey::KeyP if emacs => moved = Some(0),
                    // Control+K, U and W under mac keys: delete to the
                    // paragraph's end (its line break when the caret is at
                    // it), to its start, the word before the caret; a
                    // selection goes first.
                    UiKey::KeyK | UiKey::KeyU | UiKey::KeyW if emacs => {
                        let (from, to) = if selected {
                            (a, b)
                        } else {
                            let (start, end) = line_at(&chars, caret);
                            match k {
                                UiKey::KeyK if end > caret => (caret, end),
                                UiKey::KeyK => (caret, (caret + 1).min(chars.len())),
                                UiKey::KeyU => (start, caret),
                                _ => (word_left(&chars, caret), caret),
                            }
                        };
                        if from < to {
                            chars.drain(from..to);
                            caret = from;
                            anchor = from;
                            note(
                                &mut kind,
                                if selected || k == UiKey::KeyK {
                                    EditKind::Other
                                } else {
                                    EditKind::Back
                                },
                            );
                        }
                    }
                    // Ctrl (or Command) with Home and End: the text's ends.
                    UiKey::Home if ctrl || meta => moved = Some(0),
                    UiKey::End if ctrl || meta => moved = Some(chars.len()),
                    UiKey::Home => moved = Some(line_of(&chars, caret).0),
                    UiKey::End => moved = Some(line_of(&chars, caret).1),
                    // Command+↑/↓ in an area under mac keys: the text's ends.
                    UiKey::Up if line && d.multiline => moved = Some(0),
                    UiKey::Down if line && d.multiline => moved = Some(chars.len()),
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
                                l.0 + nearest(&l.2, &chars[l.0..l.1], x)
                            }
                            None if k == UiKey::Up => 0,
                            None => chars.len(),
                        });
                    }
                    UiKey::KeyA if shortcut => {
                        anchor = 0;
                        caret = chars.len();
                    }
                    // Control+A and E under mac keys: the line's ends (emacs).
                    UiKey::KeyA if emacs => moved = Some(line_of(&chars, caret).0),
                    UiKey::KeyE if emacs => moved = Some(line_of(&chars, caret).1),
                    UiKey::KeyC | UiKey::KeyX if shortcut && selected => {
                        clipboard = Some(chars[a..b].iter().collect::<String>());
                        if k == UiKey::KeyX {
                            chars.drain(a..b);
                            caret = a;
                            anchor = a;
                            note(&mut kind, EditKind::Other);
                        }
                    }
                    // Undo and redo: what this tick did so far is a step first.
                    UiKey::KeyZ | UiKey::KeyY if shortcut && composition.is_empty() => {
                        if chars != before {
                            history.record(
                                &before,
                                &chars,
                                kind.unwrap_or_default(),
                                sel_before,
                                (caret, anchor),
                            );
                        }
                        let to = if k == UiKey::KeyZ && !shift {
                            history.undo(&mut chars)
                        } else {
                            history.redo(&mut chars)
                        };
                        if let Some((c, a)) = to {
                            caret = c.min(chars.len());
                            anchor = a.min(chars.len());
                        }
                        before = chars.clone();
                        sel_before = (caret, anchor);
                        kind = None;
                    }
                    UiKey::Enter | UiKey::NumpadEnter if d.multiline => {
                        chars.drain(a..b);
                        caret = a;
                        if d.max_chars == 0 || chars.len() < d.max_chars as usize {
                            chars.insert(caret, '\n');
                            caret += 1;
                        }
                        anchor = caret;
                        note(&mut kind, EditKind::Other);
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
            // Keep the caret in view: horizontally in a field, by lines in an
            // area (not while a drag past the edge scrolls it).
            if let Some(inner) = d.inner
                && !(autoscrolled && self.input.keys.is_empty() && self.input.text.is_empty())
            {
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
        // The tick's edit is a step of the history; a move ends the step
        // typing or deleting was growing.
        if chars != before {
            history.record(
                &before,
                &chars,
                kind.unwrap_or_default(),
                sel_before,
                (caret, anchor),
            );
        } else if (caret, anchor) != sel_before {
            history.open = false;
        }
        history.seen = text_hash(&chars);
        history.used = tick;
        self.state.histories.insert(key, history);
        if self.state.histories.len() > HISTORY_FIELDS
            && let Some(oldest) = self
                .state
                .histories
                .iter()
                .filter(|(k, _)| **k != key)
                .min_by_key(|(_, h)| h.used)
                .map(|(k, _)| *k)
        {
            self.state.histories.remove(&oldest);
        }
        self.state.click = click;
        let text: String = chars.iter().collect();
        if text != *value {
            *value = text;
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
