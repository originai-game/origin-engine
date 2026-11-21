//! og.render.compile (three.js compileAsync): behind a loading screen it
//! makes every pipeline the world draws with, so once og.loading.idle says
//! so a level never compiles in play; og.loading.status counts them.
//!
//! Ways this could fail, each caught below: compile makes only what the
//! camera saw (turning around compiles), or only the variants of the
//! features in use (a light switched on later compiles its full lit
//! pipelines in play); idle turns true while it still has pipelines to
//! make; or the counts go back or never meet (a loading bar that moves
//! backwards or never fills). Natively a pipeline is a render or compute
//! pipeline the device holds, so the compute a dispatch would make at its
//! use is covered too.

use og_render::{RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage, wgpu};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_loading_idle,
    og_loading_status, og_render_box_geometry, og_render_compile, og_render_standard_material,
    og_world_insert, og_world_spawn, render_desc, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// A renderer drawing offscreen, as the native hosts' backends do.
struct Offscreen(Renderer);

impl RenderBackend for Offscreen {
    fn tier(&self) -> String {
        self.0.info().backend_tier()
    }
    fn size(&self) -> (u32, u32) {
        self.0.size()
    }
    fn info(&self) -> Option<og_render::RenderInfo> {
        Some(self.0.info().clone())
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0
            .render(scene, &RenderTarget::Offscreen)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    fn warm(&mut self) -> bool {
        self.0.warm()
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn(engine: u32, at: Value, component: &str, v: Value) {
    let e = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(json!({ "position": at }).to_string()),
    )))
    .as_f64()
    .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        e,
        component.into(),
        Some(v.to_string()),
    )));
}

/// Looking down -Z from z = 5, or turned around (down +Z).
fn camera(engine: u32, turned: bool) {
    let q = if turned {
        json!({ "x": 0, "y": 1, "z": 0, "w": 0 })
    } else {
        json!({ "x": 0, "y": 0, "z": 0, "w": 1 })
    };
    let cam = json!({ "position": { "x": 0, "y": 1, "z": 5 }, "quaternion": q, "fov": 50.0 });
    value(og_engine_write_resource(
        engine,
        "og.Camera".into(),
        cam.to_string(),
    ));
}

/// The render and compute pipelines the device holds.
fn held(device: &wgpu::Device) -> (isize, isize) {
    let c = device.get_internal_counters().hal;
    (c.render_pipelines.read(), c.compute_pipelines.read())
}

#[test]
#[ignore = "requires GPU"]
fn compile_makes_every_pipeline_before_play_and_counts_them() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, r#"{}"#.into()))
        .as_u64()
        .unwrap() as u32;
    let desc = render_desc(
        engine,
        RenderDesc {
            width: 160,
            height: 90,
            ..RenderDesc::default()
        },
    );
    let renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let device = renderer.device().clone();
    set_render_backend(engine, Some(Box::new(Offscreen(renderer)))).unwrap();

    let geometry = value(og_runtime::host_impl::answer(og_render_box_geometry(
        engine,
        Some("{}".into()),
    )));
    let material = |m: Value| {
        value(og_runtime::host_impl::answer(og_render_standard_material(
            engine,
            Some(m.to_string()),
        )))
    };
    // In front of the camera, and behind it in a material of its own: a
    // double-sided, alpha-blended one no frame facing -Z draws.
    let front = material(json!({}));
    let back = material(json!({ "side": "double", "transparent": true, "opacity": 0.5 }));
    let mesh = |m: &Value| json!({ "geometry": geometry, "material": m, "castShadow": true });
    spawn(
        engine,
        json!({ "x": 0, "y": 0.5, "z": 0 }),
        "og.Mesh",
        mesh(&front),
    );
    spawn(
        engine,
        json!({ "x": 0, "y": 0.5, "z": 10 }),
        "og.Mesh",
        mesh(&back),
    );
    camera(engine, false);
    for _ in 0..3 {
        frame(engine);
    }

    // What the frames made: compile adds the rest (the passes this scene
    // has not drawn yet, every variant).
    og_render::settle_pipelines();
    let drawn = held(&device);
    og_render_compile(engine);
    assert!(
        !og_loading_idle(engine),
        "idle before compile made anything"
    );
    let mut last = (0u32, 0u32);
    for _ in 0..600 {
        frame(engine);
        let s = og_loading_status(engine);
        let now = (s.pipelines_requested, s.pipelines_ready);
        assert!(
            now.0 >= last.0 && now.1 >= last.1 && now.1 <= now.0,
            "the counts went back or past each other: {last:?} then {now:?}"
        );
        last = now;
        if og_loading_idle(engine) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(og_loading_idle(engine), "compile never finished: {last:?}");
    let s = og_loading_status(engine);
    assert_eq!(
        s.pipelines_requested, s.pipelines_ready,
        "idle with pipelines compiling"
    );
    assert!(s.pipelines_ready > 0, "nothing was compiled off the frame");

    og_render::settle_pipelines();
    let compiled = held(&device);
    assert!(
        compiled.0 > drawn.0,
        "compile made no render pipeline the frames had not: {drawn:?} then {compiled:?}"
    );

    // Play: the camera turns around and a lamp (a lit feature the frames
    // so far did not use) switches on. Nothing new compiles.
    let before = held(&device);
    camera(engine, true);
    spawn(
        engine,
        json!({ "x": 1, "y": 2, "z": 8 }),
        "og.PointLight",
        json!({ "intensity": 5, "distance": 6 }),
    );
    for _ in 0..6 {
        frame(engine);
    }
    og_render::settle_pipelines();
    assert_eq!(
        held(&device),
        before,
        "(render, compute) pipelines made in play after compile"
    );

    og_render::settle_pipelines();
    set_render_backend(engine, None).unwrap();
}
