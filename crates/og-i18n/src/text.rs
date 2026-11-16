//! Character classes for layout: right-to-left characters, the scripts
//! whose glyphs depend on their neighbours (shaped), a paragraph's
//! direction by its first strong character, and segmentation by ICU4X's
//! `icu_segmenter` (UAX #29 with the installed dictionaries): words ([`word_segments`])
//! and grapheme clusters ([`graphemes`]).

use crate::locale::Direction;
use icu_segmenter::GraphemeClusterSegmenter;

/// Full Unicode uppercase for display text in a BCP-47 locale. Contextual
/// mappings (including Greek accents and Lithuanian dots) come from ICU;
/// an absent or invalid locale uses the Unicode default mapping.
pub fn uppercase(text: &str, locale: &str) -> String {
    let locale = locale
        .parse::<icu_locale_core::Locale>()
        .unwrap_or(icu_locale_core::Locale::UNKNOWN);
    icu_casemap::CaseMapper::new()
        .uppercase_to_string(text, &locale.id)
        .into_owned()
}

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

/// Whether `c` is one of the bidi formatting characters a text may carry
/// (marks, embeddings, overrides and isolates, U+200E–F, U+202A–E,
/// U+2066–9): they steer the bidi algorithm and are never drawn.
pub fn is_bidi_control(c: char) -> bool {
    matches!(c as u32, 0x200e..=0x200f | 0x202a..=0x202e | 0x2066..=0x2069)
}

/// The direction of a paragraph: its first strong character's, or
/// `default`, skipping what an isolate (U+2066–8 to U+2069) holds (UAX #9
/// P2), so game text isolated inside a string does not set its direction.
pub fn paragraph_direction(text: &str, default: Direction) -> Direction {
    let mut depth = 0usize;
    for c in text.chars() {
        match c as u32 {
            0x2066..=0x2068 => depth += 1,
            0x2069 => depth = depth.saturating_sub(1),
            0x200e if depth == 0 => return Direction::Ltr,
            0x200f if depth == 0 => return Direction::Rtl,
            _ => {}
        }
        if depth > 0 {
            continue;
        }
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

/// A text cut at its word boundaries (UAX #29 with the installed ICU4X
/// dictionaries, [`crate::segmenter`], as Parley breaks the text it lays
/// out), for double-click
/// selection of static text and undo grouping. Latin, Greek, Cyrillic,
/// Hebrew, Arabic and Devanagari words are their letters, digits and marks
/// (an apostrophe or a point between letters or digits stays inside); a
/// Katakana run is one word (WB13), Hangul words run to the spaces; Chinese,
/// Japanese, Thai, Lao, Khmer and Myanmar, written without spaces, are cut
/// into the words of ICU's dictionaries when the game ships them (else by
/// the rules alone). Spaces run together, and so does
/// a run of punctuation (one segment where UAX #29 has one per character).
pub fn word_segments(chars: &[char]) -> Vec<WordSegment> {
    let text: String = chars.iter().collect();
    let breaks: Vec<usize> = crate::segmenter::word().segment_str(&text).collect();
    let mut out: Vec<WordSegment> = Vec::new();
    let mut at = 0;
    for w in breaks.windows(2) {
        let piece = &text[w[0]..w[1]];
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

/// Whether `c` is Han or Hiragana, which an input method commits a phrase
/// at a time: typing on next to one starts a new step of an undo history.
pub fn word_per_character(c: char) -> bool {
    matches!(c as u32,
        0x3005..=0x3007 | 0x3021..=0x3029 | 0x3038..=0x303b | 0x3040..=0x309f
        | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff | 0x20000..=0x3ffff)
}

/// Whether CSS Text Decoration 4 §3.1 draws an emphasis mark on a
/// typographic character unit (`cluster`, its characters): not on a word
/// separator or other Z* character (but on a space a combining mark
/// joins), not on punctuation (P*) unless it decomposes (NFKD) to one of
/// the symbols §3.1 lists, not on a control, format or unassigned
/// character (Cc, Cf, Cn).
pub fn takes_emphasis(cluster: &str) -> bool {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{GeneralCategory as G, GeneralCategoryGroup as Group};
    let Some(c) = cluster.chars().next() else {
        return false;
    };
    let gc = CodePointMapData::<G>::new().get(c);
    if Group::Separator.contains(gc) {
        return cluster.chars().count() > 1;
    }
    if Group::Punctuation.contains(gc) {
        // §3.1's symbols and their compatibility forms (full-width, small).
        return matches!(
            c as u32,
            0x23 | 0x25
                | 0x26
                | 0x40
                | 0xa7
                | 0xb6
                | 0x609
                | 0x60a
                | 0x66a
                | 0x2030
                | 0x2031
                | 0x204a
                | 0x204b
                | 0x2053
                | 0x303d
                | 0xfe5f
                | 0xfe60
                | 0xfe6a
                | 0xfe6b
                | 0xff03
                | 0xff05
                | 0xff06
                | 0xff20
        );
    }
    !matches!(gc, G::Control | G::Format | G::Unassigned)
}

/// Whether `c` is CSS Text Decoration 4 §2.10.4's CJK text, which
/// `text-decoration-skip-ink: auto` does not skip: `Some(true)` for the
/// Han, Hiragana, Katakana, Bopomofo and Hangul scripts and a Common or
/// Inherited character whose Script_Extensions name one of them,
/// `Some(false)` for any other script, `None` for a Common or Inherited
/// character that takes its script run's (UAX #24 §5.1).
pub fn cjk_script(c: char) -> Option<bool> {
    use icu_properties::props::Script as S;
    use icu_properties::script::ScriptWithExtensions;
    const CJK: [S; 5] = [S::Han, S::Hiragana, S::Katakana, S::Bopomofo, S::Hangul];
    let scripts = ScriptWithExtensions::new();
    match scripts.get_script_val(c) {
        S::Common | S::Inherited => CJK
            .iter()
            .any(|&s| scripts.has_script(c, s))
            .then_some(true),
        s => Some(CJK.contains(&s)),
    }
}

/// CSS Text Decoration 4 §2.10.3's spacer: a character of General
/// Category Zs other than U+202F NARROW NO-BREAK SPACE.
pub fn spacer(c: char) -> bool {
    use icu_properties::CodePointMapData;
    use icu_properties::props::GeneralCategory;
    c != '\u{202f}'
        && CodePointMapData::<GeneralCategory>::new().get(c) == GeneralCategory::SpaceSeparator
}

/// CSS Text 3 §4.1.1's word separators.
pub fn word_separator(c: char) -> bool {
    matches!(
        c,
        ' ' | '\u{a0}' | '\u{1361}' | '\u{10100}' | '\u{10101}' | '\u{1039f}' | '\u{1091f}'
    )
}

/// A combining mark (General Category M): part of the word it follows.
pub(crate) fn is_mark(c: char) -> bool {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{GeneralCategory, GeneralCategoryGroup as G};
    G::Mark.contains(CodePointMapData::<GeneralCategory>::new().get(c))
}

/// CSS Pseudo-Elements 4 §2.2's `::first-letter` of a text that starts a
/// line: its character range `[start, end)`. It is the first typographic
/// letter unit, a grapheme cluster that is a letter, number or symbol,
/// with the punctuation (Ps, Pe, Pi, Pf, Po) before and after it and the
/// spaces (Zs) between them; white space before it is not part of it.
/// `None` when the text starts with anything else, such as a dash.
pub fn first_letter(text: &str) -> Option<(usize, usize)> {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{GeneralCategory, GeneralCategoryGroup as G};
    let categories = CodePointMapData::<GeneralCategory>::new();
    let chars: Vec<char> = text.chars().collect();
    let clusters = graphemes(text);
    let first = |k: usize| categories.get(chars[clusters[k].0]);
    let punctuation = |k: usize| {
        [
            G::OpenPunctuation,
            G::ClosePunctuation,
            G::InitialPunctuation,
            G::FinalPunctuation,
            G::OtherPunctuation,
        ]
        .iter()
        .any(|g| g.contains(first(k)))
    };
    let space = |k: usize| G::SpaceSeparator.contains(first(k));
    let letter = |k: usize| {
        [G::Letter, G::Number, G::Symbol]
            .iter()
            .any(|g| g.contains(first(k)))
    };
    let mut k = 0;
    while k < clusters.len() && chars[clusters[k].0].is_whitespace() {
        k += 1;
    }
    let start = clusters.get(k)?.0;
    let mut j = k;
    while j < clusters.len() && (punctuation(j) || (j > k && space(j))) {
        j += 1;
    }
    if j == clusters.len() || !letter(j) {
        return None;
    }
    let mut end = clusters[j].1;
    // Punctuation after it, with the spaces between.
    let mut m = j + 1;
    loop {
        while m < clusters.len() && space(m) {
            m += 1;
        }
        if m < clusters.len() && punctuation(m) {
            end = clusters[m].1;
            m += 1;
        } else {
            return Some((start, end));
        }
    }
}

/// A text's grapheme clusters (UAX #29, extended, `icu_segmenter`):
/// character ranges `[start, end)`, a base with its combining marks, an
/// emoji with its modifiers and joiners, a Hangul syllable of jamo, an
/// Indic consonant with its vowel signs.
pub fn graphemes(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let (mut at, mut from) = (0, 0);
    for b in GraphemeClusterSegmenter::new().segment_str(text).skip(1) {
        let n = text[from..b].chars().count();
        out.push((at, at + n));
        (at, from) = (at + n, b);
    }
    out
}
