#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
// The tests drive wgpu themselves: only the renderer counts its traffic
// (og-render clippy.toml, gpu/traffic.rs).
#![allow(clippy::disallowed_methods)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../material_alias.rs"]
mod material_alias;
#[path = "../meshlet.rs"]
mod meshlet;
#[path = "../params_motion.rs"]
mod params_motion;
#[path = "../png.rs"]
mod png;
#[path = "../shader_capabilities.rs"]
mod shader_capabilities;
#[path = "../shader_loops.rs"]
mod shader_loops;
#[path = "../shaders.rs"]
mod shaders;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
#[path = "../shared_frames.rs"]
mod shared_frames;
#[path = "../sprite_hooks.rs"]
mod sprite_hooks;
#[path = "../structure.rs"]
mod structure;
#[path = "../surface.rs"]
mod surface;
#[path = "../tier.rs"]
mod tier;
#[path = "../ui_image.rs"]
mod ui_image;
#[path = "../ui_paint.rs"]
mod ui_paint;
#[path = "../ui_shape.rs"]
mod ui_shape;
#[path = "../ui_text.rs"]
mod ui_text;
#[path = "../view_lights.rs"]
mod view_lights;
#[path = "../volumetric.rs"]
mod volumetric;
#[path = "../wgsl_rules.rs"]
mod wgsl_rules;
