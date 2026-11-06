#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../gpu.rs"]
mod gpu;
#[path = "../intent.rs"]
mod intent;
#[path = "../reference.rs"]
mod reference;
#[path = "../shaders.rs"]
mod shaders;
