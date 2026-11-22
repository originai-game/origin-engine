#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../device_formats.rs"]
mod device_formats;
#[path = "../gfx.rs"]
mod gfx;
#[path = "../render.rs"]
mod render;
#[path = "../terrain.rs"]
mod terrain;
#[path = "../ui.rs"]
mod ui;
