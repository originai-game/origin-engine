#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../effects.rs"]
mod effects;
#[path = "../exposure.rs"]
mod exposure;
#[path = "../perf.rs"]
mod perf;
#[path = "../presentation.rs"]
mod presentation;
#[path = "../render.rs"]
mod render;
