//! World systems: each describes itself and calls the mechanisms (`gpu`, `shader`, `frame`, `visibility`, `mesh`, `shading`, `media`, `post`, `particles`, `debug`), which never name a system (tests/structure.rs).

pub(crate) mod crowd;
pub(crate) mod destruction;
pub(crate) mod environment;
pub(crate) mod fluids;
pub(crate) mod softbody;
pub(crate) mod sprites;
pub(crate) mod stylise;
pub(crate) mod terrain;
pub mod ui;
pub(crate) mod vegetation;
pub(crate) mod water;
