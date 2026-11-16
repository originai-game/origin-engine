//! Origin Engine text: measurement, line breaking, bidirectional order and
//! shaping over a cooked MSDF font's glyph table and its font file
//! ([`text`], [`shape`]), glyphs rendered at run time ([`glyphs`]) and
//! where their ink crosses a decoration line ([`ink`]).
//! og-ui lays out and draws its text with it.

pub mod glyphs;
pub mod ink;
pub mod shape;
pub mod text;
