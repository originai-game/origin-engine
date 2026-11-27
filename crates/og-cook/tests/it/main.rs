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

fn declare_import(root: &std::path::Path, id: &str, kind: &str, source: &str) {
    let path = root.join(format!("content/{id}.ts"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        format!(
            "import {{file}} from '@origingame/engine/author';export default file.import({});",
            serde_json::json!({"kind":kind,"source":source})
        ),
    )
    .unwrap();
}

#[path = "../fx_source.rs"]
mod fx_source;
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
