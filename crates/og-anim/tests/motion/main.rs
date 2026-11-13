#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

mod gpu;
mod intent;
mod reference;
mod shaders;
