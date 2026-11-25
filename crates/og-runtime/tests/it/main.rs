#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../flow_acceptance.rs"]
mod flow_acceptance;
#[path = "../mid_tick_save.rs"]
mod mid_tick_save;
#[path = "../no_renderer.rs"]
mod no_renderer;
#[path = "../render_compile.rs"]
mod render_compile;
#[path = "../vfx.rs"]
mod vfx;
#[path = "../wire.rs"]
mod wire;
