//! Origin Engine tests: the gfx reference harness (`crates/og-render/README.md`
//! §Testing). Scenes are rendered headless at a fixed size and compared with
//! committed frames under `references/<backend-tier>/<scene>.png`. The same
//! scenes compile for wasm32, where `crates/og-tests-web` renders them in a
//! browser on WebGPU.

pub mod atmo_scenes;
pub mod blocks;
pub mod compare;
pub mod culling_scenes;
pub mod fork_scenes;
pub mod fx_scenes;
// Headless native runs (files, a blocking renderer); the browser's is og-tests-web.
pub mod geometry_scenes;
#[cfg(not(target_arch = "wasm32"))]
pub mod gfx;
pub mod grass_scenes;
pub mod ibl_scenes;
pub mod impostor_scenes;
pub mod light_scenes;
pub mod ocean_scenes;
pub mod p2_scenes;
pub mod physical_scenes;
pub mod scenes;
pub mod shadow_scenes;
pub mod sky_scenes;
pub mod stylise_scenes;
pub mod target_scenes;
pub mod terrain_scenes;
/// The frames a screen-space reflection scene is drawn for before it is
/// read: its rays read the previous frame's lit image, and the denoiser
/// accumulates them (`og_render`'s `ssr.rs`), so the first frame reflects
/// nothing and the reflection settles over the next few. Shared by the
/// depth-render and weather scenes on native and browser tiers.
pub const SSR_FRAMES: u32 = 16;

pub mod ui_scenes;
pub mod visual_scenes;
pub mod weather_scenes;
pub mod wind_scenes;
// Posed by og-anim (native only).
#[cfg(not(target_arch = "wasm32"))]
pub mod skeleton_scenes;
// Procedural reflection probes and SSR; cooked `lod-levels` is og-tests-cooked's.
pub mod depth_render_scenes;
#[cfg(not(target_arch = "wasm32"))]
pub mod probe_scenes;
