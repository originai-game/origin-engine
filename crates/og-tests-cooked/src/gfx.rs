//! og-tests' gfx harness ([`og_tests::gfx`]) over this crate's registry
//! ([`crate::scenes::build`]), so a scene that cooks is rendered and
//! checked by name as og-tests' own are.

use crate::scenes;
pub use og_tests::gfx::*;
use std::path::Path;

/// Renders the scene called `scene` headless (see [`render_built`]).
pub fn render(scene: &str) -> Result<Rendered, String> {
    render_built(&mut scenes::build(scene)?)
}

/// Renders `scene` and compares it with its reference (see [`check_built`]).
pub fn check(scene: &str, out_dir: &Path) -> Result<GfxReport, String> {
    check_built(scene, &mut scenes::build(scene)?, out_dir)
}
