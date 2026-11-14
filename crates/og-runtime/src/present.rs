//! What a frame shows: the world extracted into the renderer's scene
//! ([`extract`], [`interpolate`], [`presentation`]), the looks, effects
//! and geometry a game declares as data, and game feel (time scale,
//! camera feel, tweens, tilemaps).

pub(crate) mod art;
pub(crate) mod atmosphere;
pub(crate) mod baked;
pub(crate) mod display;
pub(crate) mod effect;
pub(crate) mod environment;
pub(crate) mod extract;
pub(crate) mod feel;
pub(crate) mod fx;
pub(crate) mod geometry;
pub(crate) mod glyphs;
pub(crate) mod interpolate;
pub(crate) mod light2d_at;
pub(crate) mod look;
pub(crate) mod presentation;
pub(crate) mod probes;
pub(crate) mod ui_draw;
pub(crate) mod vfx;
pub(crate) mod views;
