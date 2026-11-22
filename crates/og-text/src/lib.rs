//! Origin Engine text: measurement, line breaking, bidirectional order and
//! shaping over a cooked MSDF font's glyph table and its font file
//! ([`text`], [`shape`]), glyphs rendered at run time ([`glyphs`]) and
//! where their ink crosses a decoration line ([`ink`]), glyph coverage at
//! its drawn size ([`raster`]) or as a distance field ([`sdf`]), colour
//! glyphs painted ([`colr`]), a run's glyphs put into the glyph atlas
//! ([`strike`]), the
//! closed collection of faces texts are laid out from ([`fonts`]), one
//! Parley layout per text ([`layout`]), and the variant forms a face has
//! (`FontFile::has_variant`).
//! og-ui lays out and draws its text with it.

pub mod baselines;
pub mod colr;
pub mod fonts;
pub mod glyphs;
pub mod ink;
pub mod layout;
mod ot_languages;
pub mod overlap;
pub mod raster;
pub mod sdf;
pub mod shape;
pub mod strike;
pub mod text;
mod variants;
