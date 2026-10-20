//! Character classes for layout: right-to-left characters, the scripts
//! whose glyphs depend on their neighbours (shaped), a paragraph's
//! direction by its first strong character, and segmentation by UAX #29:
//! words ([`word_segments`]) and grapheme clusters ([`graphemes`]).

use crate::locale::Direction;
use unicode_segmentation::UnicodeSegmentation;

/// Hebrew, Arabic, Syriac, Thaana, NKo, Samaritan, Mandaic, Arabic
/// supplements and presentation forms: strong right-to-left.
pub fn is_rtl_char(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x08ff | 0xfb1d..=0xfdff | 0xfe70..=0xfeff | 0x10800..=0x10fff | 0x1e800..=0x1efff)
}

/// Scripts whose letters change shape with their neighbours or combine:
/// Arabic and Syriac (and their supplements), Devanagari to Malayalam, Thai,
/// Myanmar, Khmer, Hangul Jamo (composed by the font).
pub fn needs_shaping(c: char) -> bool {
    matches!(c as u32,
        0x0600..=0x08ff | 0x0900..=0x0dff | 0x0e00..=0x0e7f | 0x1000..=0x109f
        | 0x1780..=0x17ff | 0x1100..=0x11ff | 0xfb50..=0xfdff | 0xfe70..=0xfeff)
}

/// Strong left-to-right: letters of the scripts that write that way.
fn is_ltr_strong(c: char) -> bool {
    c.is_alphabetic() && !is_rtl_char(c)
}

/// The direction of a paragraph: its first strong character's, or `default`.
pub fn paragraph_direction(text: &str, default: Direction) -> Direction {
    for c in text.chars() {
        if is_rtl_char(c) && c.is_alphabetic() {
            return Direction::Rtl;
        }
        if is_ltr_strong(c) {
            return Direction::Ltr;
        }
    }
    default
}

/// What a word segment holds ([`word_segments`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordKind {
    /// A word: letters, digits, ideographs, with their marks and the
    /// apostrophes, points and connectors inside them (`can't`, `3.14`,
    /// `stop_now`).
    Word,
    /// Spaces and tabs.
    Space,
    /// Punctuation and symbols: a run of them is one segment (`...`, `?!`).
    Punct,
    /// A line break.
    Break,
}

/// One segment of a text, in characters `[start, end)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordSegment {
    pub start: usize,
    pub end: usize,
    pub kind: WordKind,
}

/// A text cut at its word boundaries (UAX #29, `unicode-segmentation`),
/// for word moves, double-click selection and undo grouping. Latin, Greek,
/// Cyrillic, Hebrew, Arabic and Devanagari words are their letters, digits
/// and marks (an apostrophe or a point between letters or digits stays
/// inside); Han and Hiragana are a word per character (no dictionary), a
/// Katakana run one word (WB13), Hangul words run to the spaces; Thai, Lao,
/// Khmer and Myanmar, which need a dictionary UAX #29 leaves to the
/// implementation, are a word per grapheme cluster. Spaces run together,
/// and so does a run of punctuation (one segment where UAX #29 has one per
/// character).
pub fn word_segments(chars: &[char]) -> Vec<WordSegment> {
    let text: String = chars.iter().collect();
    let mut out: Vec<WordSegment> = Vec::new();
    let mut at = 0;
    for piece in text.split_word_bounds() {
        let n = piece.chars().count();
        let kind = if piece.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
            WordKind::Break
        } else if piece.chars().all(char::is_whitespace) {
            WordKind::Space
        } else if piece.chars().any(|c| c.is_alphanumeric() || c == '_') {
            WordKind::Word
        } else {
            WordKind::Punct
        };
        match out.last_mut() {
            Some(last) if kind == WordKind::Punct && last.kind == WordKind::Punct => {
                last.end = at + n;
            }
            _ => out.push(WordSegment {
                start: at,
                end: at + n,
                kind,
            }),
        }
        at += n;
    }
    out
}

/// Whether a word of `c`'s script is one character (Han and Hiragana, as
/// [`word_segments`] cuts them): typing on after one starts a new word.
pub fn word_per_character(c: char) -> bool {
    matches!(c as u32,
        0x3005..=0x3007 | 0x3021..=0x3029 | 0x3038..=0x303b | 0x3040..=0x309f
        | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff | 0x20000..=0x3ffff)
}

/// A text's grapheme clusters (UAX #29, extended): character ranges
/// `[start, end)`, a base with its combining marks, an emoji with its
/// modifiers and joiners, a Hangul syllable of jamo, an Indic consonant
/// with its vowel signs.
pub fn graphemes(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut at = 0;
    for g in text.graphemes(true) {
        let n = g.chars().count();
        out.push((at, at + n));
        at += n;
    }
    out
}
