//! Origin Engine tests: the gfx reference harness (`crates/og-render/README.md`
//! §Testing). Scenes are rendered headless at a fixed size and compared with
//! committed frames under `references/<backend-tier>/<scene>.png`.

pub mod compare;
pub mod gfx;
pub mod scenes;
