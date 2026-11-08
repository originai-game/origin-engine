#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../meshlet.rs"]
mod meshlet;
#[path = "../params_motion.rs"]
mod params_motion;
#[path = "../png.rs"]
mod png;
#[path = "../profile2d.rs"]
mod profile2d;
#[path = "../shaders.rs"]
mod shaders;
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
#[path = "../shared_frames.rs"]
mod shared_frames;
#[path = "../sprite_hooks.rs"]
mod sprite_hooks;
#[path = "../surface.rs"]
mod surface;
#[path = "../tier.rs"]
mod tier;
#[path = "../wgsl_rules.rs"]
mod wgsl_rules;
