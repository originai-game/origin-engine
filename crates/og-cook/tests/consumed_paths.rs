//! s59: a file a glTF names through `..` (`cells/../textures/a.png`) is
//! the asset `textures/a.png`, part of the model and not cooked as a
//! texture of its own (it was: the unnormalised path matched no asset, so
//! the image was cooked twice, once in the model and once standalone); a
//! referenced file with import settings of its own, or one a terrain names
//! as a layer, stays an asset of its own as well; a file no model names
//! is untouched.
use og_cook::cook::{CookOptions, cook_project};
use std::path::PathBuf;

const GLTF: &str = r#"{"asset":{"version":"2.0"},
"images":[{"uri":"../textures/a.png"},{"uri":"../textures/b.png"},{"uri":"./../textures/c.png"}],
"textures":[{"source":0},{"source":1},{"source":2}],
"materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}},
{"pbrMetallicRoughness":{"baseColorTexture":{"index":1}}},
{"pbrMetallicRoughness":{"baseColorTexture":{"index":2}}}]}"#;

const TERRAIN: &str = r#"type terrain
version 1
---
height -20 60
size 64 64
generate {
  samples 33 33
  seed 5
}
layer {
  albedo "garden/textures/c.png"
}
"#;

#[test]
fn a_dotted_gltf_path_is_the_asset_it_names() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("consumed-paths-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let a = root.join("assets");
    std::fs::create_dir_all(a.join("garden/cells")).unwrap();
    std::fs::create_dir_all(a.join("garden/textures")).unwrap();
    std::fs::create_dir_all(a.join("terrain")).unwrap();
    for (i, n) in ["a", "b", "c", "d"].iter().enumerate() {
        image::RgbaImage::from_pixel(8, 8, image::Rgba([40 * i as u8, 90, 140, 255]))
            .save(a.join(format!("garden/textures/{n}.png")))
            .unwrap();
    }
    std::fs::write(a.join("garden/cells/c.gltf"), GLTF).unwrap();
    // b: settings of its own (what the game loads by its id too)
    std::fs::write(
        a.join("garden/textures/b.png.ogt"),
        "type import\nversion 1\n---\ntexture {\n  kind color\n}\n",
    )
    .unwrap();
    std::fs::write(a.join("terrain/ground.ogt"), TERRAIN).unwrap();
    let r = cook_project(
        &root,
        &CookOptions {
            threads: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r.ok(), "{:?}", r.errors());
    let assets: Vec<&str> = r.assets.iter().map(|x| x.source.as_str()).collect();
    assert!(
        !assets.contains(&"garden/textures/a.png"),
        "a.png is the model's, not an asset of its own: {assets:?}"
    );
    for kept in [
        "garden/textures/b.png",
        "garden/textures/c.png",
        "garden/textures/d.png",
    ] {
        assert!(assets.contains(&kept), "{kept} stays an asset: {assets:?}");
    }
    // The model still carries all three of its images.
    let model = r
        .assets
        .iter()
        .find(|x| x.source == "garden/cells/c.gltf")
        .expect("the model cooked");
    let textures = model
        .outputs
        .iter()
        .filter(|o| o.ty == "og.Texture")
        .count();
    assert_eq!(textures, 3, "{:?}", model.outputs);
    let _ = std::fs::remove_dir_all(&root);
}
