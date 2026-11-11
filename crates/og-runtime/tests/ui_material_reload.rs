use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_destroy, og_engine_frame,
    og_engine_probe_reload, og_game_game, og_host_begin_bundle, og_load_ui_material, set_hooks,
    set_render_backend,
};
use serde_json::{Value, json};
use std::{cell::RefCell, path::Path, rc::Rc};

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

struct Recorder(
    Rc<RefCell<Vec<og_render::UiMaterialShader>>>,
    Rc<RefCell<Vec<og_render::UiLayer>>>,
);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.ui.ui_materials.clone();
        *self.1.borrow_mut() = scene
            .ui
            .overlay
            .as_ref()
            .map(|o| o.ui.layers.clone())
            .unwrap_or_default();
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

fn write(root: &Path, channel: &str, amount: f32) {
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("assets/swatch.svelte"),
        r#"<meta name="screen" content="Swatch">
<div id="swatch"></div>
<style>
#swatch { width: 64px; height: 64px; background: #234567; -og-material: "effect.ogt"; }
</style>"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/effect.ogt"),
        format!("type ui-material\nversion 1\n---\nsource \"effect.wgsl\"\nparam amount float {amount}\nparam offset vec2 0.2 0.7\n"),
    ).unwrap();
    std::fs::write(
        root.join("assets/effect.wgsl"),
        format!("fn material(m: MaterialInput, p: Params) -> vec4<f32> {{ return vec4<f32>(layer(m.uv).{channel} * p.amount); }}"),
    ).unwrap();
    let report = og_cook::cook_project(root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

#[test]
fn reload_replaces_ui_material_code_and_defaults_without_changing_its_handle() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("ui-material-reload-{}", std::process::id()));
    write(&root, "r", 0.25);
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 7 }).to_string()));
    let engine = value(og_engine_create(game, json!({ "root": root }).to_string()))
        .as_u64()
        .unwrap() as u32;
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        engine,
        "swatch.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        engine,
        "Swatch".into(),
        None,
    )));
    let material = value(og_runtime::host_impl::answer(og_load_ui_material(
        engine,
        "effect.ogt".into(),
    )));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let layers = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(
        engine,
        Some(Box::new(Recorder(seen.clone(), layers.clone()))),
    )
    .unwrap();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert!(seen.borrow()[0].code.contains(".r *"));
    assert_eq!(seen.borrow()[0].defaults()[0][0], 0.25);
    assert_eq!(
        layers.borrow().len(),
        1,
        "a custom material referenced only by CSS is bound"
    );
    assert_eq!(layers.borrow()[0].params[0][0], 0.25);

    let dump = || -> Value {
        serde_json::from_str(&og_runtime::og_engine_probe_dump(
            engine,
            Some(json!({ "what": "ui" }).to_string()),
        ))
        .unwrap()
    };
    let parameter = &dump()["materials"][0]["parameters"][0];
    assert_eq!(parameter["name"], "amount");
    assert_eq!(parameter["type"], "f32");
    assert_eq!(parameter["value"], json!([0.25]));
    value(og_runtime::host_impl::answer(
        og_runtime::og_commands_schedule(engine, "0:ui:material:effect.ogt.offset=0.4 0.9".into()),
    ));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(layers.borrow()[0].params[1], [0.4, 0.9, 0.0, 0.0]);
    for (input, expected) in [("0.6", 0.6_f32), ("NaN", 0.6), ("1 2", 0.6), ("", 0.25)] {
        value(og_runtime::host_impl::answer(
            og_runtime::og_commands_schedule(
                engine,
                format!("0:ui:material:effect.ogt.amount={input}"),
            ),
        ));
        value(og_engine_frame(engine, 1000.0 / 60.0));
        let current = dump()["materials"][0]["parameters"][0]["value"][0]
            .as_f64()
            .unwrap();
        assert!(
            (current - f64::from(expected)).abs() < 1e-6,
            "{input}: {current}"
        );
        assert_eq!(layers.borrow()[0].params[0][0], expected);
        assert_eq!(layers.borrow()[0].params[1], [0.4, 0.9, 0.0, 0.0]);
        assert_eq!(
            seen.borrow()[0].defaults()[0][0],
            0.25,
            "tuning uniforms does not change the shader"
        );
    }

    // Only the external WGSL changes; the descriptor's bytes stay the same.
    og_runtime::shared_ui(engine)
        .unwrap()
        .lock()
        .set_part_css("button::part(base) { opacity: 0.43; }")
        .unwrap();
    write(&root, "g", 0.25);
    let reloaded: Value = serde_json::from_str(&og_engine_probe_reload(engine)).unwrap();
    assert!(reloaded.is_object(), "{reloaded}");
    let button = og_ui::dress::PartRef::new("Button", "base", og_ui::tokens::ElementClass::Button);
    assert_eq!(
        og_runtime::shared_ui(engine)
            .unwrap()
            .lock()
            .resolve_part(&button, [80.0, 32.0])
            .unwrap()
            .opacity,
        0.43,
        "hot reload keeps installed part CSS"
    );
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(
        value(og_runtime::host_impl::answer(og_load_ui_material(
            engine,
            "effect.ogt".into()
        ))),
        material
    );
    assert_eq!(seen.borrow().len(), 1);
    assert!(
        seen.borrow()[0].code.contains(".g *"),
        "renderer received stale WGSL"
    );
    assert_eq!(seen.borrow()[0].defaults()[0][0], 0.25);
    write(&root, "g", 0.75);
    og_engine_probe_reload(engine);
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(seen.borrow().len(), 1);
    assert_eq!(seen.borrow()[0].defaults()[0][0], 0.75);
    assert_eq!(
        layers.borrow()[0].params[0][0],
        0.75,
        "CSS rebinds refreshed defaults"
    );
    og_engine_destroy(engine);
    std::fs::remove_dir_all(root).unwrap();
}
