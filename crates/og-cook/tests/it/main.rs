#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../scene_string.rs"]
mod scene_string;
#[path = "../shader.rs"]
mod shader;
#[path = "../template.rs"]
mod template;
