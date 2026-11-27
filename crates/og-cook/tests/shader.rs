//! TS shader-family cook/load contract: native backend validation, parameter
//! order, captured WGSL inputs, module identities and no author text fallback.
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_tests::atmo_scenes::{DISSOLVE_WGSL, dissolve_shader};

const DISSOLVE: &str = r#"import {file,input} from '@origingame/engine/author';
import {scale} from '../lib/scale.ts';
export default file.shader({
  code:input.text('sources/dissolve.wgsl'),
  params:[{name:'amount',kind:'float'}, {name:'scale',kind:'float',default:scale},
          {name:'edgeWidth',kind:'float',default:0.08},
          {name:'edgeColor',kind:'color',default:[4,1.6,0.4]}]
});"#;

fn write(root: &std::path::Path, file: &str, text: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn a_shader_cooks_for_every_back_end_and_loads() {
    use og_render_data::{ParamKind, ShaderMode};
    let root = crate::temp::TempDir::new("typed-shaders");
    crate::init_project(&root);
    write(
        &root,
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'shaders',entry:'src/main.ts',modules:[{id:'north',root:'modules/north'}]});",
    );
    write(&root, "modules/north/content/dissolve.ts", DISSOLVE);
    write(&root, "modules/north/content/copy.ts", DISSOLVE);
    write(&root, "modules/north/lib/scale.ts", "export const scale=3;");
    write(
        &root,
        "content/post.ts",
        "import {file} from '@origingame/engine/author';export default file.post({code:'fn post(f: PostInput, p: Params) -> vec4<f32> {return previous(f.uv + p.offset)*p.tint;}',params:[{kind:'vec2',name:'offset',default:[0.125,-0.25]},{kind:'vec4',name:'tint',default:[0.5,0.75,1.25,0.875]}]});",
    );
    write(
        &root,
        "content/ui.ts",
        "import {file} from '@origingame/engine/author';export default file['ui-material']({code:'fn material(m: MaterialInput, p: Params) -> vec4<f32> {return layer(m.uv)*vec4<f32>(p.tint,1.0);}',params:[{kind:'vec3',name:'tint',default:[0.25,0.5,0.75]}]});",
    );
    write(
        &root,
        "content/sprite.ts",
        "import {file} from '@origingame/engine/author';export default file.shader({mode:{kind:'sprite'},code:'fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {return sprite_texel(h.uv)*c;}'});",
    );
    let full = og_render::example_full_shader();
    for lit in [false, true] {
        write(&root, &format!("content/full-{lit}.ts"), &format!("import {{file}} from '@origingame/engine/author';export default file.shader({{mode:{{kind:'full',lit:{lit}}},code:{},params:{}}});",
            serde_json::to_string(&full.code).unwrap(),
            serde_json::to_string(&full.params.iter().map(|p| serde_json::json!({"name":p.name,"kind":match p.kind {ParamKind::Float=>"float",ParamKind::Vec2=>"vec2",ParamKind::Vec3=>"vec3",ParamKind::Vec4=>"vec4"},"default":if p.kind==ParamKind::Float {serde_json::json!(p.default[0])} else {serde_json::json!(&p.default[..p.kind.count()])}})).collect::<Vec<_>>()).unwrap()));
    }
    let id = "north/dissolve";
    let options = CookOptions {
        threads: 3,
        ..Default::default()
    };
    let mut previous = None;
    for revision in [1, 2] {
        let code = format!("{DISSOLVE_WGSL}\n// captured revision {revision}\n");
        write(&root, "modules/north/sources/dissolve.wgsl", &code);
        for warm in [false, true] {
            let report = cook_project(&root, &options).unwrap();
            assert!(report.ok(), "{:?}", report.errors());
            let bundle = Bundle::open(&root.join("cooked")).unwrap();
            let bytes = bundle.read(id).unwrap().to_vec();
            let shader = bundle.shader(id, &bytes).unwrap();
            assert_eq!(shader.code, code, "external input was stale or absent");
            assert_eq!(shader.params, dissolve_shader().params);
            assert_eq!(shader.mode, ShaderMode::Hooks);
            assert_eq!(shader.name, id);
            let copy = bundle.read("north/copy").unwrap();
            assert_eq!(
                bytes.as_slice(),
                &*copy,
                "identical declarations must share payloads"
            );
            assert_eq!(
                bundle.shader("north/copy", &copy).unwrap().name,
                "north/copy"
            );
            if warm {
                assert_eq!(previous.as_ref().unwrap(), &bytes);
            } else if let Some(previous) = &previous {
                assert_ne!(previous, &bytes, "stale WGSL cache");
            }
            previous = Some(bytes);
            let post = bundle
                .post("base/post", &bundle.read("base/post").unwrap())
                .unwrap();
            assert_eq!(post.defaults()[0], [0.125, -0.25, 0.0, 0.0]);
            assert_eq!(post.defaults()[1], [0.5, 0.75, 1.25, 0.875]);
            let ui = bundle
                .ui_material("base/ui", &bundle.read("base/ui").unwrap())
                .unwrap();
            assert_eq!(ui.defaults()[0], [0.25, 0.5, 0.75, 0.0]);
            assert_eq!(
                bundle
                    .shader("base/sprite", &bundle.read("base/sprite").unwrap())
                    .unwrap()
                    .mode,
                ShaderMode::Sprite
            );
            for lit in [false, true] {
                let id = format!("base/full-{lit}");
                let shader = bundle.shader(&id, &bundle.read(&id).unwrap()).unwrap();
                assert_eq!(shader.mode, ShaderMode::Full { lit });
                assert_eq!(shader.code, full.code);
                assert_eq!(shader.params, full.params);
            }
        }
    }
    write(
        &root,
        "modules/north/lib/scale.ts",
        "export const scale=2.5;",
    );
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle
            .shader(id, &bundle.read(id).unwrap())
            .unwrap()
            .defaults()[1][0],
        2.5
    );
    for kind in ["shader", "post", "ui-material"] {
        write(
            &root,
            "assets/old.ogt",
            &format!("type {kind}\nversion 1\n---\n"),
        );
        let report = cook_project(&root, &options).unwrap();
        assert!(!report.ok(), "legacy {kind} author entry was accepted");
    }
}

#[test]
fn shaders_that_do_not_compile_are_refused_with_the_reason() {
    let root = crate::temp::TempDir::new("typed-shader-errors");
    crate::init_project(&root);
    for (body, expected) in [
        (
            "code:'fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c * ; }'",
            "hooks:1",
        ),
        (
            "code:'fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c.rgb; }'",
            "surface_color",
        ),
        (
            "code:'fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> { return e * p.glow; }'",
            "glow",
        ),
        (
            "code:'fn helper() -> f32 { return 1.0; }'",
            "defines none of the hooks",
        ),
        (
            "code:input.text('sources/nowhere.wgsl')",
            "sources/nowhere.wgsl",
        ),
        ("code:'',params:[{kind:'matrix',name:'scale'}]", "params"),
    ] {
        write(
            &root,
            "content/shader.ts",
            &format!(
                "import {{file,input}} from '@origingame/engine/author';\nexport default file.shader({{{body}}});"
            ),
        );
        let error = match cook_project(&root, &CookOptions::default()) {
            Ok(report) => {
                assert!(!report.ok());
                format!("{:?}", report.errors())
            }
            Err(e) => e.to_string(),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("shader"), "{error}");
    }
}
