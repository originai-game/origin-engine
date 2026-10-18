//! Character classes for layout: right-to-left characters, the scripts
//! whose glyphs depend on their neighbours (shaped), and a paragraph's
//! direction by its first strong character.

use crate::locale::Direction;

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
