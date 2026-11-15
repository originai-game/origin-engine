//! Text editing, shared by the one-line field and the multi-line area.
//!
//! Parley owns the text's geometry and the caret's moves: the edited text
//! is laid out by Parley ([`Editable`], og-text's `paragraph::editable`, in
//! the fonts it is drawn with), and its `Cursor` and `Selection` place the
//! caret from a press, step it by grapheme cluster on screen (across a bidi
//! run's edge, where one place in the text has two on screen: the cursor's
//! affinity, `State::caret_upstream`), by word (UAX #29 with ICU's
//! dictionaries, so Thai and Chinese step by their words) and by line,
//! find a line's ends, give Backspace and Delete what they remove, and
//! measure the selection and the caret for drawing.
//!
//! og-ui keeps what is the control's own: the pointer's gestures (a press
//! places the caret, a drag selects and scrolls the text past its edge, a
//! double click selects a word and a triple click the line), the scroll,
//! typed text (og-input's `og.TextInput` stream, or the keys before one
//! exists) replacing the selection, an input method's composition shown
//! underlined at the caret until it commits, the shortcut keys, undo and
//! redo (the shortcut key with Z, Shift+Z and Y; a bounded history per
//! control, typing grouped by words), what an input type accepts, and the
//! caret's blink by the tick.
//!
//! The keys follow the platform ([`crate::Viewport::mac_keys`]: the web
//! shell's browser on an Apple platform, a native host on macOS): on a Mac
//! Command is the shortcut key and goes to the line's ends with ←/→ (and
//! deletes to them with Backspace and Delete, ↑/↓ the text's ends in an
//! area), Option moves and deletes by words, and Control takes the emacs
//! keys (A and E the line's ends, F and B a character, N and P a line, D
//! and H delete a character, K kills to the paragraph's end, U to its
//! start, W the word before, each into a kill buffer Y yanks, kills in a
//! row adding up; T transposes the characters at the caret); elsewhere
//! Control is the shortcut key and moves by words (Alt too), and Command
//! only takes the shortcuts.
//!
//! The glyph boxes below ([`boxes`], [`nearest`], [`word_at`]) place read-
//! only text's selection (`user-select`, vertical lines too) and its marks.

use crate::input::UiKey;
use crate::layout::{FontRole, Rect};
use crate::text::FontMetrics;
use og_core::glam::Vec2;
use og_i18n::Direction;
use og_text::text::paragraph::Brush;
use parley::{Affinity, Cursor, Layout, Selection};

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

/// An edited text laid out by Parley for its cursor and selection
/// (og-text's `paragraph::editable`): positions in characters of `text`,
/// as og-ui's state keeps them, a caret's place at a bidi edge or a soft
/// line break as `upstream` (Parley's [`Affinity::Upstream`]).
pub(crate) struct Editable {
    /// Each character's byte offset in the text, and the text's length.
    bytes: Vec<usize>,
    pub layout: Layout<Brush>,
    /// The paragraph's direction (its bidi base).
    pub base: Direction,
}

impl Editable {
    /// `text` in `m` at `size` CSS px (`tracking` em), its bidi base its
    /// first strong character's (else `ui`'s), broken at `width` (none: one
    /// line per paragraph) into lines `line_height` tall, aligned to `ui`'s
    /// start.
    pub fn new(
        m: &FontMetrics,
        text: &str,
        ui: Direction,
        size: f32,
        tracking: f32,
        width: Option<f32>,
        line_height: f32,
    ) -> Editable {
        let base = crate::text::line_direction(text, ui);
        let layout = og_text::text::paragraph::editable(
            m,
            text,
            size,
            tracking,
            base,
            ui,
            width,
            line_height,
        );
        let mut bytes: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();
        bytes.push(text.len());
        Editable {
            bytes,
            layout,
            base,
        }
    }

    /// Its length in characters.
    pub fn len(&self) -> usize {
        self.bytes.len() - 1
    }

    fn byte(&self, k: usize) -> usize {
        self.bytes[k.min(self.len())]
    }

    fn char_of(&self, byte: usize) -> usize {
        self.bytes.partition_point(|&b| b < byte)
    }

    /// The cursor before character `k`.
    pub fn cursor(&self, k: usize, upstream: bool) -> Cursor {
        let affinity = if upstream {
            Affinity::Upstream
        } else {
            Affinity::Downstream
        };
        Cursor::from_byte_index(&self.layout, self.byte(k), affinity)
    }

    /// A cursor's character and place.
    pub fn place(&self, c: Cursor) -> (usize, bool) {
        (self.char_of(c.index()), c.affinity() == Affinity::Upstream)
    }

    /// The selection from `anchor` to the caret.
    pub fn selection(&self, anchor: usize, caret: usize, upstream: bool) -> Selection {
        Selection::new(self.cursor(anchor, false), self.cursor(caret, upstream))
    }

    /// Where a press at `(x, y)` (from the layout's top left) puts the caret.
    pub fn hit(&self, x: f32, y: f32) -> (usize, bool) {
        self.place(Cursor::from_point(&self.layout, x, y))
    }

    /// A selection's ends in characters, first first.
    pub fn range(&self, s: &Selection) -> (usize, usize) {
        let r = s.text_range();
        (self.char_of(r.start), self.char_of(r.end))
    }

    /// The word at `(x, y)` (a double click's): Parley's word, its spaces
    /// or its punctuation.
    pub fn word_at_point(&self, x: f32, y: f32) -> (usize, usize) {
        self.range(&Selection::word_from_point(&self.layout, x, y))
    }

    /// The paragraph at `(x, y)` (a triple click's), without its line break.
    pub fn paragraph_at_point(&self, x: f32, y: f32) -> (usize, usize) {
        self.range(&Selection::hard_line_from_point(&self.layout, x, y))
    }

    /// The start and end of the line the caret is on.
    pub fn line_ends(&self, caret: usize, upstream: bool) -> ((usize, bool), (usize, bool)) {
        let s: Selection = self.cursor(caret, upstream).into();
        (
            self.place(s.line_start(&self.layout, false).focus()),
            self.place(s.line_end(&self.layout, false).focus()),
        )
    }

    /// The grapheme cluster after character `k` (its end; `k` at the text's end).
    pub fn next_cluster(&self, k: usize) -> usize {
        match self.cursor(k, false).logical_clusters(&self.layout)[1] {
            Some(c) => self.char_of(c.text_range().end),
            None => k,
        }
    }

    /// The grapheme cluster before character `k` (its start; 0 at the start).
    pub fn prev_cluster(&self, k: usize) -> usize {
        match self.cursor(k, false).logical_clusters(&self.layout)[0] {
            Some(c) => self.char_of(c.text_range().start),
            None => 0,
        }
    }

    /// Where Backspace at `k` deletes from: the character before it, or
    /// the whole cluster before it when that is a line break or an emoji
    /// (Parley's `backdelete`), so a letter's accent or an Indic vowel sign
    /// goes alone.
    pub fn backspace_from(&self, k: usize) -> usize {
        match self.cursor(k, false).logical_clusters(&self.layout)[0] {
            Some(c) if c.is_hard_line_break() || c.is_emoji() => self.char_of(c.text_range().start),
            Some(_) => k - 1,
            None => k,
        }
    }

    /// The start of the word before `k` (over the spaces before it).
    pub fn word_before(&self, k: usize) -> usize {
        self.place(self.cursor(k, false).previous_logical_word(&self.layout))
            .0
    }

    /// The start of the word after the one `k` is in (over its spaces).
    pub fn word_after(&self, k: usize) -> usize {
        self.place(self.cursor(k, false).next_logical_word(&self.layout))
            .0
    }

    /// The caret before `k`: its x and the index of its line.
    pub fn caret(&self, k: usize, upstream: bool) -> (f32, usize) {
        let g = self.cursor(k, upstream).geometry(&self.layout, 0.0);
        (g.x0 as f32, self.line_at_y(((g.y0 + g.y1) / 2.0) as f32))
    }

    /// The index of the line at `y`.
    fn line_at_y(&self, y: f32) -> usize {
        self.layout
            .lines()
            .position(|l| y < l.metrics().block_max_coord)
            .unwrap_or(self.layout.len().saturating_sub(1))
    }

    /// The stretches characters `[a, b)` cover on screen: x from, x to and
    /// the line's index.
    pub fn spans(&self, a: usize, b: usize) -> Vec<(f32, f32, usize)> {
        if a >= b {
            return Vec::new();
        }
        Selection::new(self.cursor(a, false), self.cursor(b, false))
            .geometry(&self.layout)
            .into_iter()
            .map(|(r, line)| (r.x0 as f32, r.x1 as f32, line))
            .collect()
    }

    /// Its lines: their characters and where they start across.
    pub fn lines(&self) -> Vec<((usize, usize), f32)> {
        self.layout
            .lines()
            .map(|l| {
                let r = l.text_range();
                (
                    (self.char_of(r.start), self.char_of(r.end)),
                    l.metrics().offset,
                )
            })
            .collect()
    }

    /// Its widest line, with its spaces.
    pub fn width(&self) -> f32 {
        self.layout.full_width()
    }

    /// Its lines' height.
    pub fn height(&self) -> f32 {
        self.layout.height()
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
    /// What it takes ([`crate::InputType`]): a password is laid out masked,
    /// moves by words to its ends and never fills the clipboard or the kill
    /// buffer; a number filters what is typed.
    pub input_type: crate::InputType,
    /// Laid out masked (a password not revealed).
    pub masked: bool,
    /// Read-only: its text is selected and copied, never changed.
    pub readonly: bool,
}

/// The character a secret text is drawn with in `m`: the bullet, else `*`.
pub(crate) fn mask_char(m: &FontMetrics) -> char {
    if m.has(crate::input_type::MASK) {
        crate::input_type::MASK
    } else {
        '*'
    }
}

/// `chars` as shown: each character masked in a secret text.
pub(crate) fn shown(chars: &[char], mask: Option<char>) -> String {
    match mask {
        Some(m) => chars.iter().map(|_| m).collect(),
        None => chars.iter().collect(),
    }
}

/// The segment holding character `i` (UAX #29 words: og-i18n's
/// [`og_i18n::word_segments`]).
fn segment_at(segs: &[og_i18n::WordSegment], i: usize) -> Option<usize> {
    segs.iter().position(|s| s.start <= i && i < s.end)
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
/// a new word for the undo history: after spaces, or next to Han or
/// Hiragana (an input method commits them a phrase at a time, and each
/// commit is a step). Thai and other scripts without spaces group to the
/// next space, as Latin does.
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
pub struct Click {
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
pub enum EditKind {
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
pub struct Change {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
    pub kind: EditKind,
    pub before: (usize, usize),
    pub after: (usize, usize),
}

/// A text control's undo and redo stacks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct History {
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

impl crate::ui::Interaction {
    /// Edits `value` as the focused text control `key`: pointer, typed and
    /// composed text, editing keys, undo and redo. The caret, selection
    /// anchor, composition, click count and history live in the UI state
    /// (and its saved bytes); Parley's cursor and selection
    /// ([`Editable`]) move the caret over the text as it is laid out.
    pub(crate) fn edit(
        &mut self,
        key: u64,
        value: &mut String,
        d: &EditDesc,
        env: &crate::ui::Env,
        ui_out: &mut crate::ui::Out,
    ) -> Edited {
        let mut out = Edited::default();
        let dir = env.direction;
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
            self.state.caret_upstream = false;
            self.state.killing = false;
        }
        let mut caret = self.state.caret.min(chars.len());
        let mut anchor = self.state.anchor.min(chars.len());
        let mut sel_before = (caret, anchor);
        let mut upstream = self.state.caret_upstream;
        let mut killing = self.state.killing;
        let mut kill = std::mem::take(&mut self.state.kill);
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
        let secret = d.input_type == crate::InputType::Password;
        let accepts = |c: char| (d.multiline || c != '\n') && d.input_type.accepts(c);
        let room = |chars: &[char]| d.max_chars == 0 || chars.len() < d.max_chars as usize;
        {
            let m = &env
                .fonts
                .for_text(d.role, if secret { "\u{2022}" } else { value.as_str() })
                .metrics;
            let mask = d.masked.then(|| mask_char(m));
            let width = d.inner.filter(|_| d.multiline).map(|r| r.width.max(1.0));
            // The text as it is drawn, laid out by Parley.
            let lay = |chars: &[char]| {
                Editable::new(
                    m,
                    &shown(chars, mask),
                    dir,
                    d.size,
                    d.tracking,
                    width,
                    d.line_height,
                )
            };
            let mut ed = lay(&chars);
            let mut laid = chars.clone();
            let mut scroll = self.state.edit_scroll;
            // A press places the caret, a drag (the press held) selects; a
            // second press in a row selects a word, a third the line, and a
            // drag from them extends by words or lines.
            let pressed_here = self.input.pressed && self.state.active == key;
            let dragging = !self.input.pressed && self.input.down && self.state.active == key;
            if (pressed_here || dragging)
                && let Some(inner) = d.inner
            {
                let mut p = self.input.pointer;
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
                            let most = (ed.height() - inner.height).max(0.0);
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
                            let room = (inner.width - 1.0).max(1.0);
                            let most = (ed.width() - room).max(0.0);
                            scroll = (scroll + speed(over)).clamp(0.0, most);
                            p.x = p.x.clamp(inner.x, inner.right());
                            autoscrolled = true;
                        }
                    }
                    self.state.edit_scroll = scroll;
                }
                // The point in the layout: an area's lines scroll down, a
                // field's line across (and is pressed at its middle).
                let (x, y) = if d.multiline {
                    (p.x - inner.x, p.y - inner.y + scroll)
                } else {
                    (p.x - inner.x + scroll, d.line_height / 2.0)
                };
                let n = chars.len();
                let (k, k_up) = ed.hit(x, y);
                let unit = |count: u8| match count {
                    2 if secret => (0, n),
                    2 => ed.word_at_point(x, y),
                    3 if d.multiline => ed.paragraph_at_point(x, y),
                    3 => (0, n),
                    _ => (k, k),
                };
                upstream = k_up;
                killing = false;
                if pressed_here {
                    let near = (p - click.at).length() <= CLICK_SLOP;
                    let soon = tick.saturating_sub(click.tick) <= click_ticks(env.ticks_per_second);
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
                        let (a, b) = unit(click.count);
                        click.origin = (a, b);
                        anchor = a;
                        caret = b;
                        upstream &= a == b;
                    }
                } else {
                    if (self.input.pointer - click.at).length() > CLICK_SLOP {
                        click.moved = true;
                    }
                    if click.count >= 2 {
                        let (o0, o1) = click.origin;
                        let (a, b) = unit(click.count);
                        if a < o0 {
                            anchor = o1;
                            caret = a;
                        } else {
                            anchor = o0;
                            caret = b.max(o1);
                        }
                        upstream = false;
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
                if !c.is_empty() {
                    upstream = false;
                    killing = false;
                }
                composition = c.clone();
            }
            // Typed text replaces the selection.
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(accesskit::ActionData::Value(value)) = self.native_input.values.remove(&key)
            {
                chars = value
                    .chars()
                    .filter(|&c| accepts(c))
                    .take(if d.max_chars == 0 {
                        usize::MAX
                    } else {
                        d.max_chars as usize
                    })
                    .collect();
                caret = chars.len();
                anchor = caret;
                composition.clear();
                note(&mut kind, EditKind::Other);
            }
            let typed: Vec<char> = self.input.text.chars().filter(|&c| accepts(c)).collect();
            if !typed.is_empty() {
                let (a, b) = (caret.min(anchor), caret.max(anchor));
                chars.drain(a..b);
                caret = a;
                for &c in &typed {
                    if room(&chars) {
                        chars.insert(caret, c);
                        caret += 1;
                    }
                }
                anchor = caret;
                // The caret keeps to what was typed (Parley's insertion),
                // after a line break to the line it starts.
                upstream = typed.last() != Some(&'\n');
                killing = false;
                note(&mut kind, EditKind::Typing);
            }
            let shift = self.input.shift;
            let (ctrl, meta, alt) = (self.input.ctrl, self.input.meta, self.input.alt);
            let mac = env.viewport.mac_keys;
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
            // before the caret, each into the kill buffer Y yanks, T
            // transposes the characters at the caret).
            let emacs = mac && ctrl && !meta;
            for k in self.input.keys.iter().copied() {
                if chars != laid {
                    ed = lay(&chars);
                    laid.clone_from(&chars);
                }
                let layout = &ed.layout;
                let (a, b) = (caret.min(anchor), caret.max(anchor));
                let selected = a != b;
                let sel = ed.selection(anchor, caret, upstream);
                // Where a move puts the caret, and its place on screen.
                let mut moved: Option<(usize, bool)> = None;
                let at_key = (caret, chars.clone());
                // A kill right after a kill adds to the buffer.
                let chained = killing;
                killing = false;
                // The emacs keys that are another key's edit.
                let k = match k {
                    UiKey::KeyD if emacs => UiKey::Delete,
                    UiKey::KeyH if emacs => UiKey::Backspace,
                    UiKey::KeyN if emacs && d.multiline => UiKey::Down,
                    UiKey::KeyP if emacs && d.multiline => UiKey::Up,
                    k => k,
                };
                let (start, end) = ed.line_ends(caret, upstream);
                match k {
                    UiKey::Backspace | UiKey::Delete if selected => {
                        chars.drain(a..b);
                        caret = a;
                        anchor = a;
                        note(&mut kind, EditKind::Other);
                    }
                    UiKey::Backspace if caret > 0 => {
                        let from = if line && start.0 < caret {
                            start.0
                        } else if word && secret {
                            0
                        } else if word {
                            ed.word_before(caret)
                        } else {
                            ed.backspace_from(caret)
                        };
                        chars.drain(from..caret);
                        caret = from;
                        anchor = caret;
                        note(&mut kind, EditKind::Back);
                    }
                    UiKey::Delete if caret < chars.len() => {
                        let to = if line && end.0 > caret {
                            end.0
                        } else if word && secret {
                            chars.len()
                        } else if word {
                            ed.word_after(caret)
                        } else {
                            ed.next_cluster(caret)
                        };
                        chars.drain(caret..to);
                        anchor = caret;
                        note(&mut kind, EditKind::Forward);
                    }
                    // ← and → on screen. Command under mac keys: the line's
                    // ends as the paragraph runs; the word key: by words (a
                    // secret text's ends); else a cluster, a selection
                    // collapsing to its side.
                    UiKey::Left | UiKey::Right => {
                        let right = k == UiKey::Right;
                        let on = right != (ed.base == Direction::Rtl);
                        moved = Some(if line {
                            if on { end } else { start }
                        } else if word && secret {
                            (if on { chars.len() } else { 0 }, false)
                        } else {
                            let s = match (right, word) {
                                (true, true) => sel.next_visual_word(layout, shift),
                                (false, true) => sel.previous_visual_word(layout, shift),
                                (true, false) => sel.next_visual(layout, shift),
                                (false, false) => sel.previous_visual(layout, shift),
                            };
                            ed.place(s.focus())
                        });
                    }
                    // Control+F and B under mac keys: a cluster on and back
                    // in the text (whichever way it runs).
                    UiKey::KeyF if emacs => {
                        moved = Some(if selected && !shift {
                            (b, false)
                        } else {
                            (ed.next_cluster(caret), false)
                        })
                    }
                    UiKey::KeyB if emacs => {
                        moved = Some(if selected && !shift {
                            (a, false)
                        } else {
                            (ed.prev_cluster(caret), false)
                        })
                    }
                    // Control+N and P in a field: the text's ends.
                    UiKey::KeyN if emacs => moved = Some((chars.len(), false)),
                    UiKey::KeyP if emacs => moved = Some((0, false)),
                    // Control+K, U and W under mac keys: kill to the
                    // paragraph's end (its line break when the caret is at
                    // it), to its start, the word before the caret; a
                    // selection goes first. What they take goes into the
                    // kill buffer, after it (K) or before it (U, W) when
                    // the last edit was a kill too.
                    UiKey::KeyK | UiKey::KeyU | UiKey::KeyW if emacs => {
                        let (from, to) = if selected {
                            (a, b)
                        } else {
                            let (start, end) = line_at(&chars, caret);
                            match k {
                                UiKey::KeyK if end > caret => (caret, end),
                                UiKey::KeyK => (caret, (caret + 1).min(chars.len())),
                                UiKey::KeyU => (start, caret),
                                _ if secret => (0, caret),
                                _ => (ed.word_before(caret), caret),
                            }
                        };
                        killing = true;
                        if from < to {
                            let taken: String = chars.drain(from..to).collect();
                            // A secret text deletes without filling the kill buffer.
                            kill = match (chained, k) {
                                _ if secret => kill,
                                (false, _) => taken,
                                (true, UiKey::KeyK) => kill + &taken,
                                (true, _) => taken + &kill,
                            };
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
                    // Control+Y under mac keys: yank the kill buffer over the
                    // selection (a field takes it without line breaks).
                    UiKey::KeyY if emacs && !kill.is_empty() => {
                        chars.drain(a..b);
                        caret = a;
                        for c in kill.chars().filter(|&c| accepts(c)) {
                            if room(&chars) {
                                chars.insert(caret, c);
                                caret += 1;
                            }
                        }
                        anchor = caret;
                        note(&mut kind, EditKind::Other);
                    }
                    // Control+T under mac keys: transpose the clusters on
                    // either side of the caret (the two before it at the
                    // paragraph's end), the caret after them.
                    UiKey::KeyT if emacs && !selected => {
                        let (start, end) = line_at(&chars, caret);
                        let mid = if caret == end && caret > start {
                            ed.prev_cluster(caret)
                        } else {
                            caret
                        };
                        let (l, r) = (ed.prev_cluster(mid), ed.next_cluster(mid));
                        if start <= l && l < mid && mid < r && r <= end {
                            let left: Vec<char> = chars[l..mid].to_vec();
                            let right: Vec<char> = chars[mid..r].to_vec();
                            chars.splice(l..r, right.into_iter().chain(left));
                            caret = r;
                            anchor = r;
                            note(&mut kind, EditKind::Other);
                        }
                    }
                    // Ctrl (or Command) with Home and End: the text's ends.
                    UiKey::Home if ctrl || meta => moved = Some((0, false)),
                    UiKey::End if ctrl || meta => moved = Some((chars.len(), true)),
                    UiKey::Home => moved = Some(start),
                    UiKey::End => moved = Some(end),
                    // Command+↑/↓ in an area under mac keys: the text's ends.
                    UiKey::Up if line && d.multiline => moved = Some((0, false)),
                    UiKey::Down if line && d.multiline => moved = Some((chars.len(), true)),
                    UiKey::Up | UiKey::Down if d.multiline => {
                        let s = if k == UiKey::Up {
                            sel.previous_line(layout, shift)
                        } else {
                            sel.next_line(layout, shift)
                        };
                        moved = Some(ed.place(s.focus()));
                    }
                    UiKey::KeyA if shortcut => {
                        anchor = 0;
                        caret = chars.len();
                    }
                    // Control+A and E under mac keys: the line's ends (emacs).
                    UiKey::KeyA if emacs => moved = Some(start),
                    UiKey::KeyE if emacs => moved = Some(end),
                    // A secret text is never copied or cut.
                    UiKey::KeyC | UiKey::KeyX if shortcut && selected && !secret => {
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
                        if room(&chars) {
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
                if let Some((to, up)) = moved {
                    caret = to;
                    upstream = up;
                    if !shift {
                        anchor = to;
                    }
                } else if (caret, &chars) != (at_key.0, &at_key.1) {
                    upstream = false;
                }
            }
            // Keep the caret in view: horizontally in a field, by lines in an
            // area (not while a drag past the edge scrolls it).
            if let Some(inner) = d.inner
                && !(autoscrolled && self.input.keys.is_empty() && self.input.text.is_empty())
            {
                let comp: Vec<char> = composition.chars().collect();
                let at = caret.min(chars.len());
                let mut with = chars.clone();
                with.splice(at..at, comp.iter().copied());
                if with != laid {
                    ed = lay(&with);
                }
                let (x, li) = ed.caret(at + comp.len(), upstream || !comp.is_empty());
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
                    s = s.min((ed.height() - inner.height).max(0.0)).max(0.0);
                } else {
                    let room = (inner.width - 1.0).max(1.0);
                    if x - s > room {
                        s = x - room;
                    }
                    if x < s {
                        s = x;
                    }
                    s = s.min((ed.width() - room).max(0.0)).max(0.0);
                }
                self.state.edit_scroll = s;
            }
        }
        // A read-only text keeps its text: what this tick would have
        // changed (typing, deleting, a cut, a paste, an undo) is dropped with
        // the caret it moved; selecting, moving and copying stay.
        if d.readonly {
            composition.clear();
            if chars != before {
                chars.clone_from(&before);
                (caret, anchor) = sel_before;
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
        self.state.caret_upstream = upstream;
        self.state.killing = killing;
        self.state.kill = kill;
        self.state.composition = composition;
        if clipboard.is_some() {
            ui_out.clipboard = clipboard;
        }
        if out.left {
            self.state.focus = 0;
            self.state.composition.clear();
        }
        out
    }
    /// Whether the caret shows this tick: always under reduced motion, else
    /// the 530 ms half period, shown after an edit or
    /// move and then hidden as long, by the tick.
    pub(crate) fn caret_shown(&self, env: &crate::ui::Env) -> bool {
        if env.theme.reduced_motion {
            return true;
        }
        let ms = 530.0;
        let half = ((env.ticks_per_second.max(1) as f32 * (ms / 1000.0)).round() as u64).max(1);
        (self.state.tick.saturating_sub(self.state.edit_tick) / half).is_multiple_of(2)
    }
}
