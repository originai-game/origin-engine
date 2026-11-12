//! Origin Engine tests that cook a project: og-tests' gfx scenes whose
//! content og-cook makes (`vox-keyframes`, `lod-levels`, the `meshlet-*` scenes,
//! `terrain-generated`, `plant-grown`, `plant-scatter`, `ui-vector`, `ui-rive`, the glTF extension scenes), and og-ui's frames drawn with cooked
//! fonts and icons (`tests/text_gfx.rs`, `materials_gfx.rs`,
//! `pack_contrast.rs`). A package of its own so that og-tests and og-ui
//! do not build on og-cook: a change to og-cook or og-api reruns these GPU
//! reference tests and leaves og-tests' and og-ui's skipped
//! (scripts/check.sh `rule_gpu`, docs/ci.md §Lanes). The frames stay in
//! og-tests' and og-ui's `references/`.
//!
//! [`scenes`] and [`gfx`] are og-tests' registry and harness with this
//! crate's scenes added; `og-gfx` (src/bin) renders every scene of both.

pub mod depth_render_scenes;
pub mod gfx;
pub mod gltf_scenes;
pub mod impostor_scenes;
pub mod meshlet_scenes;
pub mod plant_scenes;
pub mod scenes;
pub mod terrain_scenes;
pub mod ui_scenes;
pub mod vox_scenes;
