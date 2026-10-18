//! Material shaders through the cook and back (the E5 round): a shader
//! `.ogt` naming its WGSL file cooks to a blob the runtime loads into the
//! same og-render MaterialShader the reference scenes draw with; shaders
//! that do not compile for every back end are refused with naga's message.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_tests::atmo_scenes::{DISSOLVE_WGSL, dissolve_shader};
use std::path::PathBuf;

const DISSOLVE: &str = r#"type shader
version 1
---
source "shaders/dissolve.wgsl"
param amount float 0
param scale float 3
param edgeWidth float 0.08
param edgeColor color 4 1.6 0.4
"#;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-shader")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

#[test]
fn a_shader_cooks_for_every_back_end_and_loads() {
    let root = project(
        "ok",
        &[
            ("shaders/dissolve.ogt", DISSOLVE),
            ("shaders/dissolve.wgsl", DISSOLVE_WGSL),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "shaders/dissolve.ogt";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let shader = bundle.shader(id, &bytes).unwrap();
    let expected = dissolve_shader();
    assert_eq!(
        shader.code, expected.code,
        "the WGSL file is carried in the blob"
    );
    assert_eq!(shader.params, expected.params);
}

#[test]
fn shaders_that_do_not_compile_are_refused_with_the_reason() {
    let with = |code: &str| DISSOLVE.replace("source \"shaders/dissolve.wgsl\"", code);
    let syntax = with(
        r#"code "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c * ; }""#,
    );
    let types = with(
        r#"code "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c.rgb; }""#,
    );
    let unknown = with(
        r#"code "fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> { return e * p.glow; }""#,
    );
    let none = with(r#"code "fn helper() -> f32 { return 1.0; }""#);
    let missing = with(r#"source "shaders/nowhere.wgsl""#);
    let kind = with(
        r#"code "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }""#,
    )
    .replace("param scale float 3", "param scale matrix 3");
    let root = project(
        "bad",
        &[
            ("s/syntax.ogt", syntax.as_str()),
            ("s/types.ogt", types.as_str()),
            ("s/unknown.ogt", unknown.as_str()),
            ("s/none.ogt", none.as_str()),
            ("s/missing.ogt", missing.as_str()),
            ("s/kind.ogt", kind.as_str()),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    let of = |file: &str| {
        errors
            .iter()
            .find(|e| e.contains(file))
            .unwrap_or_else(|| panic!("no error for {file}: {errors:?}"))
            .clone()
    };
    for file in ["s/syntax.ogt", "s/types.ogt", "s/unknown.ogt"] {
        let e = of(file);
        assert!(e.contains("the shader does not compile"), "{e}");
        assert!(
            e.contains("hooks:1"),
            "the location is a line of the hooks: {e}"
        );
    }
    assert!(
        of("s/types.ogt").contains("surface_color"),
        "naga names the function"
    );
    assert!(
        of("s/unknown.ogt").contains("glow"),
        "naga names the missing member"
    );
    assert!(of("s/none.ogt").contains("defines none of the hooks"));
    assert!(of("s/missing.ogt").contains("shaders/nowhere.wgsl"));
    assert!(of("s/kind.ogt").contains("parameter type `matrix`"));
}
