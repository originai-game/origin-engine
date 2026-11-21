//! What an agent asks of a frame it cannot see (docs/roadmap.md §Direction,
//! Agent observability): why an entity is or is not drawn (the `visible`
//! member and `dump_entity`'s `visible`), the record of what the main
//! camera's cull kept (`dump_drawn`), and one tick's frames from several
//! viewpoints in one call (`capture_views`).
//!
//! Ways this could fail: an entity behind the camera, a hidden one, one off
//! the camera's layers and one behind a wall all answer the same (or
//! "drawn", from the extraction alone, or a CPU frustum test that knows no
//! occlusion); the verdict is another frame's, or another entity's (the
//! GPU's words read in the wrong order); a backend without a renderer
//! claims a verdict no cull made; the dump leaves out an entity the cull
//! dropped; a viewpoint's frame is drawn from og.Camera, or og.Camera is
//! written and the next frame comes from the last viewpoint.

use og_render::{Camera, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_capture, og_engine_probe_capture_views, og_engine_probe_commands,
    og_engine_probe_dump, og_engine_probe_visible, og_game_game, og_host_begin_bundle,
    og_render_box_geometry, og_render_standard_material, og_world_insert, og_world_spawn,
    render_desc, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
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

/// A renderer that keeps the camera of every capture.
struct Recorder(Rc<RefCell<Vec<Camera>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (160, 90)
    }
    fn render(&mut self, _scene: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, scene: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        self.0.borrow_mut().push(scene.frame.camera);
        Some(RgbaImage {
            width: 2,
            height: 2,
            pixels: vec![0; 16],
        })
    }
}

/// A renderer drawing offscreen, as the native hosts' backends do.
struct Offscreen(Renderer, og_render::FrameStats);

impl RenderBackend for Offscreen {
    fn tier(&self) -> String {
        self.0.info().backend_tier()
    }
    fn size(&self) -> (u32, u32) {
        self.0.size()
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.1 = self
            .0
            .render(scene, &RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        self.1
    }
    fn cull_record(&mut self) -> Option<og_render::CullRecord> {
        self.0.cull_record().cloned()
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

fn raw(text: String) -> Value {
    serde_json::from_str(&text).expect("JSON")
}

#[test]
fn without_a_renderer_no_cull_is_claimed_and_captures_take_several_viewpoints() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(w, Some(Box::new(Recorder(seen.clone())))).unwrap();

    let geometry = value(og_runtime::host_impl::answer(og_render_box_geometry(
        w,
        Some(json!({ "width": 1, "height": 1, "depth": 1 }).to_string()),
    )));
    let material = value(og_runtime::host_impl::answer(og_render_standard_material(
        w,
        Some("{}".into()),
    )));
    let spawn = |name: &str, z: f32, mesh: Option<Value>| {
        let at = json!({ "name": name, "position": { "x": 0, "y": 0, "z": z } });
        let e = value(og_runtime::host_impl::answer(og_world_spawn(
            w,
            Some(at.to_string()),
        )))
        .as_f64()
        .unwrap();
        if let Some(mut m) = mesh {
            m["geometry"] = geometry.clone();
            m["material"] = material.clone();
            value(og_runtime::host_impl::answer(og_world_insert(
                w,
                e,
                "og.Mesh".into(),
                Some(m.to_string()),
            )));
        }
    };
    // The camera at the origin looks down -Z.
    spawn("ahead", -5.0, Some(json!({})));
    spawn("behind", 5.0, Some(json!({})));
    spawn("hidden", -5.0, Some(json!({ "visible": false })));
    spawn("masked", -5.0, Some(json!({ "layers": 2 })));
    spawn("empty", -5.0, None);
    raw(og_engine_probe_commands(
        w,
        "0:set:og.Camera.position=0,0,0".into(),
    ));
    value(og_engine_frame(w, 1000.0 / 60.0));

    // `visible` without a renderer: no cull ran, so none is claimed. What
    // the components say still holds.
    let reason = |id: &str| raw(og_engine_probe_visible(w, id.into()));
    let ahead = reason("ahead");
    assert_eq!(ahead["id"], json!("ahead"));
    assert_eq!(ahead["reason"], json!("untracked"), "{ahead}");
    assert_eq!(ahead["visible"], Value::Null, "{ahead}");
    assert_eq!(reason("behind")["reason"], json!("untracked"));
    let hidden = reason("hidden");
    assert_eq!(hidden["reason"], json!("hidden"), "{hidden}");
    assert_eq!(hidden["detail"], json!("og.Mesh.visible"), "{hidden}");
    assert_eq!(reason("empty")["reason"], json!("nothing_to_draw"));
    assert_eq!(reason("nobody"), Value::Null);

    // capture_views: one call, one tick, each frame from its own camera.
    seen.borrow_mut().clear();
    let views = json!([
        { "position": [0, 10, 0.01], "target": [0, 0, 0], "fov": 30 },
        { "position": { "x": 8, "y": 1, "z": -5 }, "target": [0, 0, -5], "height": 4 },
    ]);
    let frames = raw(og_engine_probe_capture_views(
        w,
        views.to_string(),
        None,
        None,
        None,
    ));
    let list = frames["frames"].as_array().expect("frames");
    assert_eq!(list.len(), 2, "{frames}");
    assert_eq!(list[1]["view"], json!(1));
    assert!(list[0]["png"].as_str().is_some_and(|p| !p.is_empty()));
    let cams = seen.borrow().clone();
    assert_eq!(cams.len(), 2, "one draw per viewpoint");
    assert_eq!(cams[0].position.to_array(), [0.0, 10.0, 0.01]);
    assert!(
        matches!(cams[0].projection, og_render::Projection::Perspective { fov_y_degrees } if fov_y_degrees == 30.0)
    );
    assert_eq!(cams[1].position.to_array(), [8.0, 1.0, -5.0]);
    assert_eq!(
        cams[1].projection,
        og_render::Projection::Orthographic { height: 4.0 }
    );
    // og.Camera was never written: the next capture is its own.
    seen.borrow_mut().clear();
    raw(og_engine_probe_capture(w, None, None, None, None));
    assert_eq!(seen.borrow()[0].position.to_array(), [0.0, 0.0, 0.0]);
}

fn spawn_mesh(w: u32, name: &str, at: [f32; 3], mesh: Value) {
    let at = json!({ "name": name, "position": { "x": at[0], "y": at[1], "z": at[2] } });
    let e = value(og_runtime::host_impl::answer(og_world_spawn(
        w,
        Some(at.to_string()),
    )))
    .as_f64()
    .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        w,
        e,
        "og.Mesh".into(),
        Some(mesh.to_string()),
    )));
}

/// Each entity's reason in `dump_drawn`.
fn drawn_reasons(w: u32) -> (Value, Vec<(String, String)>) {
    let drawn = raw(og_engine_probe_dump(
        w,
        Some(json!({ "what": "drawn" }).to_string()),
    ));
    let reasons = drawn["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| {
            Some((
                e["id"].as_str()?.to_owned(),
                e["reason"].as_str()?.to_owned(),
            ))
        })
        .collect();
    (drawn, reasons)
}

/// The verdicts are the frame's own culls': the GPU cull's camera pass with
/// its occlusion test, read back in instance order. The scene has enough
/// instances for occlusion (og-render's `OCCLUSION_MIN_INSTANCES`, the
/// filler behind the camera), and a wall the occluders draw.
#[test]
#[ignore = "requires GPU"]
fn the_probe_says_why_each_entity_is_drawn_as_the_frames_culls_decided() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let desc = render_desc(
        w,
        RenderDesc {
            width: 320,
            height: 180,
            ..RenderDesc::default()
        },
    );
    let renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    set_render_backend(w, Some(Box::new(Offscreen(renderer, Default::default())))).unwrap();

    let geometry = value(og_runtime::host_impl::answer(og_render_box_geometry(
        w,
        Some(json!({ "width": 1, "height": 1, "depth": 1 }).to_string()),
    )));
    let wall = value(og_runtime::host_impl::answer(og_render_box_geometry(
        w,
        Some(json!({ "width": 40, "height": 40, "depth": 1 }).to_string()),
    )));
    let material = value(og_runtime::host_impl::answer(og_render_standard_material(
        w,
        Some("{}".into()),
    )));
    let mesh = |g: &Value, extra: Value| {
        let mut m = json!({ "geometry": g, "material": material });
        for (k, v) in extra.as_object().unwrap() {
            m[k] = v.clone();
        }
        m
    };
    // The camera at the origin looks down -Z: a cube to the right in view,
    // one behind the camera, one off its layers, one behind the wall.
    spawn_mesh(w, "ahead", [3.0, 0.0, -5.0], mesh(&geometry, json!({})));
    spawn_mesh(w, "behind", [0.0, 0.0, 5.0], mesh(&geometry, json!({})));
    spawn_mesh(
        w,
        "masked",
        [3.0, 0.0, -5.0],
        mesh(&geometry, json!({ "layers": 2 })),
    );
    spawn_mesh(
        w,
        "hidden",
        [3.0, 0.0, -5.0],
        mesh(&geometry, json!({ "visible": false })),
    );
    spawn_mesh(w, "wall", [-10.0, 0.0, -8.0], mesh(&wall, json!({})));
    spawn_mesh(w, "walled", [-4.0, 0.0, -20.0], mesh(&geometry, json!({})));
    for i in 0..16_384u32 {
        let (x, y) = ((i % 128) as f32, (i / 128) as f32);
        spawn_mesh(
            w,
            &format!("filler{i}"),
            [x, y, 50.0],
            mesh(&geometry, json!({})),
        );
    }
    raw(og_engine_probe_commands(
        w,
        "0:set:og.Camera.position=0,0,0".into(),
    ));

    // The GPU cull's pipelines are made in the background, and occlusion
    // needs a frame's camera visibility first: draw until the wall hides.
    let reason = |id: &str| raw(og_engine_probe_visible(w, id.into()));
    let mut walled = Value::Null;
    for _ in 0..600 {
        value(og_engine_frame(w, 1000.0 / 60.0));
        walled = reason("walled");
        if walled["reason"] == json!("occluded") {
            break;
        }
    }
    assert_eq!(walled["reason"], json!("occluded"), "{walled}");
    assert_eq!(walled["visible"], json!(false), "{walled}");
    assert_eq!(walled["culled"], json!(1), "{walled}");
    let ahead = reason("ahead");
    assert_eq!(ahead["reason"], json!("drawn"), "{ahead}");
    assert_eq!(ahead["visible"], json!(true), "{ahead}");
    assert_eq!(ahead["drawn"], json!(1), "{ahead}");
    assert_eq!(reason("wall")["reason"], json!("drawn"));
    let behind = reason("behind");
    assert_eq!(behind["reason"], json!("outside_view"), "{behind}");
    assert_eq!(behind["culled"], json!(1), "{behind}");
    assert_eq!(reason("masked")["reason"], json!("layer"));
    let hidden = reason("hidden");
    assert_eq!(hidden["reason"], json!("hidden"), "{hidden}");
    assert_eq!(reason("filler9")["reason"], json!("outside_view"));

    // dump_entity carries the same verdict.
    let dump = raw(og_engine_probe_dump(
        w,
        Some(json!({ "what": "entity", "id": "walled" }).to_string()),
    ));
    assert_eq!(dump["visible"]["reason"], json!("occluded"), "{dump}");

    // dump_drawn: every entity with something to draw, culled ones included.
    let (drawn, reasons) = drawn_reasons(w);
    let of = |id: &str| reasons.iter().find(|r| r.0 == id).map(|r| r.1.as_str());
    assert_eq!(of("ahead"), Some("drawn"), "{drawn}");
    assert_eq!(of("walled"), Some("occluded"), "{drawn}");
    assert_eq!(of("behind"), Some("outside_view"), "{drawn}");
    assert_eq!(of("hidden"), Some("hidden"), "{drawn}");
    assert_eq!(of("masked"), Some("layer"), "{drawn}");
    assert_eq!(drawn["drawn"], json!(2), "ahead and the wall: {drawn}");

    // Turned around: the next frame's own cull, not the last record.
    raw(og_engine_probe_commands(
        w,
        "0:set:og.Camera.quaternion=0,1,0,0".into(),
    ));
    value(og_engine_frame(w, 1000.0 / 60.0));
    assert_eq!(reason("behind")["reason"], json!("drawn"));
    assert_eq!(reason("ahead")["reason"], json!("outside_view"));
}
