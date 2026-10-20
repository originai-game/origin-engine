//! Environment maps through the cook and back (phase 4): a `type
//! environment` .ogt and its equirectangular HDR cook into og-render's
//! prefiltered map, byte for byte what og-render's `bake` makes of the same
//! image and settings, which the runtime loads; the image is part of the
//! environment (not listed as ignored, its edit recooks it); a bad file is
//! refused by name.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-environment")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

const STUDIO: &str = "type environment\nversion 1\n---\nsource \"studio.hdr\"\nsize 32\nintensity 1.5\nrotation 30\nsamples 64\n";

fn cook(root: &Path) -> og_cook::Report {
    let report = cook_project(root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    report
}

#[test]
fn an_environment_cooks_into_the_baked_map_and_loads_back() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let root = project(
        "studio",
        &[
            ("env/studio.ogt", STUDIO.as_bytes()),
            ("env/studio.hdr", &hdr),
        ],
    );
    let report = cook(&root);
    assert!(
        !report.ignored.iter().any(|f| f.ends_with(".hdr")),
        "the image is the environment's: {:?}",
        report.ignored
    );
    let outputs: Vec<String> = report
        .cooked_outputs()
        .iter()
        .map(|o| o.id.clone())
        .collect();
    assert_eq!(outputs, vec!["env/studio.ogt".to_string()]);
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes.iter().any(|n| n.contains("32² cube of 5 levels")),
        "{notes:?}"
    );

    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let map = bundle.load_environment("env/studio.ogt").unwrap();
    let img = image::load_from_memory(&hdr).unwrap().to_rgb32f();
    let baked = og_render::bake::bake(
        &og_render::bake::Equirect {
            width: img.width(),
            height: img.height(),
            rgb: img.as_raw(),
        },
        &og_render::bake::BakeDesc {
            size: 32,
            intensity: 1.5,
            rotation_degrees: 30.0,
            samples: 64,
        },
    )
    .unwrap();
    assert_eq!(
        map, baked,
        "the cooked map is og-render's bake of the image"
    );
    assert_eq!(
        map.texels.len(),
        og_render::EnvironmentMapData::texel_count(32)
    );

    // Nothing changed: nothing cooked; the image edited: the map again.
    let again = cook(&root);
    assert!(!again.cooked_outputs().iter().any(|o| o.cooked));
    let mut edited = hdr.clone();
    let last = edited.len() - 2;
    edited[last] = edited[last].wrapping_add(7);
    std::fs::write(root.join("assets/env/studio.hdr"), &edited).unwrap();
    let third = cook(&root);
    let cooked: Vec<&str> = third
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(cooked, vec!["env/studio.ogt"]);
}

#[test]
fn refusals_name_the_fix() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let cook_err = |name: &str, text: &str| -> String {
        let root = project(
            name,
            &[("e/x.ogt", text.as_bytes()), ("e/studio.hdr", &hdr)],
        );
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let head = "type environment\nversion 1\n---\n";
    let e = cook_err("none", head);
    assert!(
        e.contains("source \\\"<file>.hdr\\\"") || e.contains("source \"<file>.hdr\""),
        "{e}"
    );
    let e = cook_err("png", &format!("{head}source \"studio.png\"\n"));
    assert!(e.contains(".hdr or OpenEXR .exr"), "{e}");
    let e = cook_err("size", &format!("{head}source \"studio.hdr\"\nsize 100\n"));
    assert!(e.contains("a power of two from 16 to 2048"), "{e}");
    let e = cook_err("key", &format!("{head}source \"studio.hdr\"\nblur 2\n"));
    assert!(e.contains("'blur' is not an environment entry"), "{e}");
    let e = cook_err("missing", &format!("{head}source \"gone.hdr\"\n"));
    assert!(e.contains("gone.hdr"), "{e}");
}
