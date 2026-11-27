#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;
#[path = "../../../../tests/temp.rs"]
mod temp;

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

#[path = "../scene_string.rs"]
mod scene_string;
#[path = "../shader.rs"]
mod shader;
#[path = "../source_content.rs"]
mod source_content;
#[path = "../source_schemas.rs"]
mod source_schemas;
#[path = "../template.rs"]
mod template;
#[path = "../world_source_schemas.rs"]
mod world_source_schemas;
