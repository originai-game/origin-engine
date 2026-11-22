//! Origin Engine text: the closed collection of faces texts are laid out
//! from ([`fonts`]), one Parley layout per text ([`layout`]) and a face's
//! baselines ([`baselines`]); glyph coverage at its drawn size
//! ([`raster`]) or as a distance field ([`sdf`], its contours resolved by
//! [`overlap`]), colour glyphs painted ([`colr`]), and a run's glyphs put
//! into the glyph atlas ([`strike`]); where ink crosses a decoration line
//! ([`ink`]), and a pixel font's cells ([`glyphs`]).

pub mod baselines;
pub mod colr;
pub mod fonts;
pub mod glyphs;
pub mod ink;
pub mod layout;
pub mod overlap;
pub mod raster;
pub mod sdf;
pub mod strike;
pub mod text;
