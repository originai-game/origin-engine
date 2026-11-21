//! Origin Engine text: measurement, line breaking, bidirectional order and
//! shaping over a cooked MSDF font's glyph table and its font file
//! ([`text`], [`shape`]), glyphs rendered at run time ([`glyphs`]) and
//! where their ink crosses a decoration line ([`ink`]), glyph coverage at
//! its drawn size ([`raster`]), the closed collection of faces texts are
//! laid out from ([`fonts`]), and the variant
//! forms a face has (`FontFile::has_variant`).
//! og-ui lays out and draws its text with it.

pub mod baselines;
pub mod fonts;
pub mod glyphs;
pub mod ink;
mod ot_languages;
pub mod raster;
pub mod shape;
pub mod text;
mod variants;
