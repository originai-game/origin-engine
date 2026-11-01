//! og-tests-cooked's integration tests, one binary (docs/ci.md §Test
//! binaries): og-tests' tests of the scenes that cook (`gfx`, `render`,
//! `terrain`, `ui`, named as they were there) and og-ui's tests that draw
//! with cooked fonts and icons (`materials_gfx`, `pack_contrast`,
//! `text_gfx`) over og-ui's own test support. A new test file is a `mod`
//! line here.

mod device_formats;
mod gfx;
mod materials_gfx;
mod pack_contrast;
mod render;
mod support;
mod terrain;
mod text_gfx;
mod ui;
