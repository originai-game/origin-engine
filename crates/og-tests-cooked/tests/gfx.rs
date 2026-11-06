//! gfx reference frames of the scenes that cook (og-tests' tests/gfx.rs
//! has the others): each matches its committed reference in og-tests'
//! references/, plus behaviour read from the frames.

use og_tests_cooked::gfx;
use std::path::PathBuf;

#[path = "gfx/gltf_extensions.rs"]
mod gltf_extensions;
#[path = "gfx/impostors.rs"]
mod impostors;
#[path = "gfx/meshlets.rs"]
mod meshlets;
#[path = "gfx/overlay_effects.rs"]
mod overlay_effects;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn check(scene: &str) -> gfx::GfxReport {
    match gfx::check(scene, &out_dir()) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}
