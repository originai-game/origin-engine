//! `og model` on the model guide's programs: each builds and measures
//! the size its own numbers give.

use std::process::Command;

fn shape(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_og"))
        .arg("model")
        .args(args)
        .output()
        .expect("the binary runs")
}

/// Runs a command that must succeed and parses its JSON report.
fn report(args: &[&str]) -> serde_json::Value {
    let out = shape(args);
    assert!(
        out.status.success(),
        "{args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("a JSON report")
}

/// Every ```json program block of the model guide builds, and measures the
/// size its own numbers give (the recipe is checked, not only shown).
#[test]
fn the_model_guides_programs_build() {
    let guide = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../og-api/guides/model.md"
    ))
    .unwrap();
    let mut blocks = Vec::new();
    let mut lines = guide.lines();
    while let Some(l) = lines.next() {
        if l.trim() == "```json program" {
            blocks.push(
                lines
                    .by_ref()
                    .take_while(|l| l.trim() != "```")
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
    }
    assert!(!blocks.is_empty());
    let dir = std::env::temp_dir().join(format!("shape-guide-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (k, b) in blocks.iter().enumerate() {
        let path = dir.join(format!("g{k}.shape.json"));
        std::fs::write(&path, b).unwrap();
        let m = report(&["measure", path.to_str().unwrap(), "--json"]);
        let size: Vec<f64> = m["size"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        // The side table: 600 wide, 720 tall (690 legs and a 30 top), 450 deep.
        if k == 0 {
            for (a, b) in size.iter().zip([0.6, 0.72, 0.45]) {
                assert!((a - b).abs() < 1e-6, "{size:?}");
            }
            assert_eq!(m["partCount"], 5);
        }
    }
    // The checked program and variant cross the actual CLI -> bake -> cook
    // boundary. Parameters and unpainted material controls must not disappear.
    for folder in ["content/props", "sources", "src"] {
        std::fs::create_dir_all(dir.join(folder)).unwrap();
    }
    std::fs::write(
        dir.join("og-project.json"),
        r#"{"version":1,"name":"Model","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(dir.join("project.ts"), "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'model',entry:'src/main.ts'});").unwrap();
    std::fs::write(dir.join("src/main.ts"), "").unwrap();
    std::fs::write(dir.join("content/program.ts"), format!("import {{file}} from '@origingame/engine/author';export default file['model-program']({});", blocks[0])).unwrap();
    let declaration = "import {file,ref} from '@origingame/engine/author';export default file.model({kind:'program',desc:{program:ref('base/program'),params:{width:830},materials:[{name:'oak',roughness:0.23}],conform:'off'}});";
    let variant = dir.join("content/props/table.ts");
    std::fs::write(&variant, declaration).unwrap();
    let built = report(&["build", variant.to_str().unwrap(), "--json"]);
    assert_eq!(built["parts"].as_array().unwrap().len(), 5);
    let uv_mesh = dir.join("sources/uvs.glb");
    report(&[
        "export",
        variant.to_str().unwrap(),
        "--uvs",
        "--out",
        uv_mesh.to_str().unwrap(),
        "--json",
    ]);
    let (mesh, buffers, _) = gltf::import(&uv_mesh).unwrap();
    for primitive in mesh.meshes().flat_map(|m| m.primitives()) {
        assert!(
            primitive
                .reader(|b| Some(&buffers[b.index()]))
                .read_tex_coords(0)
                .is_some()
        );
    }
    let image = dir.join("sources/paint.png");
    let mut paint = image::RgbaImage::new(32, 32);
    for y in 4..28 {
        for x in 4..28 {
            paint.put_pixel(x, y, image::Rgba([211, 83, 127, 255]));
        }
    }
    paint.save(&image).unwrap();
    let painted = report(&[
        "bake",
        variant.to_str().unwrap(),
        "--project",
        &format!("front={}", image.display()),
        "--size",
        "32",
        "--view-size",
        "64",
        "--json",
    ]);
    assert!(
        painted["output"]["model"]["path"]
            .as_str()
            .unwrap()
            .ends_with("table.baked.ts")
    );
    assert_eq!(std::fs::read_to_string(&variant).unwrap(), declaration);
    let cooked = super::og()
        .args(["cook", dir.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(cooked.status.success(), "{}", super::text(&cooked));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let model = bundle.load_model("base/props/table.baked").unwrap();
    assert!((model.max.x - model.min.x - 0.83).abs() < 0.001);
    assert!(
        model
            .materials
            .iter()
            .any(|m| (m.roughness - 0.23).abs() < 0.001 && m.base_color_texture.is_some())
    );
    let _ = std::fs::remove_dir_all(&dir);
}
