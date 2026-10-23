//! Origin Engine tests: the gfx reference harness (`crates/og-render/README.md`
//! §Testing). Scenes are rendered headless at a fixed size and compared with
//! committed frames under `references/<backend-tier>/<scene>.png`. The same
//! scenes compile for wasm32, where `crates/og-tests-web` renders them in a
//! browser on WebGPU and WebGL2.

pub mod atmo_scenes;
pub mod blocks;
pub mod compare;
pub mod culling_scenes;
pub mod fx_scenes;
// Headless native runs (files, a blocking renderer); the browser's is og-tests-web.
pub mod geometry_scenes;
#[cfg(not(target_arch = "wasm32"))]
pub mod gfx;
pub mod ibl_scenes;
pub mod p2_scenes;
pub mod physical_scenes;
pub mod scenes;
pub mod shadow_scenes;
pub mod stylise_scenes;
pub mod terrain_scenes;
// Cooked through og-cook (native only).
#[cfg(not(target_arch = "wasm32"))]
pub mod vox_scenes;
// Posed by og-anim (native only).
#[cfg(not(target_arch = "wasm32"))]
pub mod skeleton_scenes;
// Levels of detail cooked by og-cook, reflection probes and SSR (native only).
#[cfg(not(target_arch = "wasm32"))]
pub mod depth_render_scenes;
