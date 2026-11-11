//! og-ui's frame-test support (crates/og-tests-ui/tests/support: the
//! sample screens and their render scenes, the MSDF test font, the
//! reference comparison), shared by path, and the cooked-font text scenes
//! beside it.

#[path = "../../../og-tests-ui/tests/support/mod.rs"]
mod og_ui;
pub use og_ui::*;
// text_scenes reaches `super::scenes`, as it does in og-ui's support.
use og_ui as scenes;

pub mod text_scenes;
