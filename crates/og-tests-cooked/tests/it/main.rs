#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../device_formats.rs"]
mod device_formats;
#[path = "../gfx.rs"]
mod gfx;
#[path = "../materials_gfx.rs"]
mod materials_gfx;
#[path = "../render.rs"]
mod render;
#[path = "../support/mod.rs"]
mod support;
#[path = "../terrain.rs"]
mod terrain;
#[path = "../text_gfx.rs"]
mod text_gfx;
#[path = "../text_outline.rs"]
mod text_outline;
#[path = "../ui.rs"]
mod ui;
