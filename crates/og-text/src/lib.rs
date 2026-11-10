//! Origin Engine text: measurement, line breaking, bidirectional order and
//! shaping over a cooked MSDF font's glyph table and its font file
//! ([`text`], [`shape`]), and glyphs rendered at run time ([`glyphs`]).
//! og-ui lays out and draws its text with it.

pub mod glyphs;
pub mod shape;
pub mod text;
