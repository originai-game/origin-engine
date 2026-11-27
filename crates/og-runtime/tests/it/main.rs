#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

fn init_project(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    for (file, text) in [
        (
            "og-project.json",
            include_str!("../../../../tests/fixtures/cook-assets/og-project.json"),
        ),
        (
            "project.ts",
            include_str!("../../../../tests/fixtures/cook-assets/project.ts"),
        ),
        (
            "src/main.ts",
            include_str!("../../../../tests/fixtures/cook-assets/src/main.ts"),
        ),
    ] {
        std::fs::write(root.join(file), text).unwrap();
    }
}

#[path = "../flow_acceptance.rs"]
mod flow_acceptance;
#[path = "../mid_tick_save.rs"]
mod mid_tick_save;
#[path = "../model_animation.rs"]
mod model_animation;
#[path = "../no_renderer.rs"]
mod no_renderer;
#[path = "../render_compile.rs"]
mod render_compile;
#[path = "../vfx.rs"]
mod vfx;
#[path = "../wire.rs"]
mod wire;
