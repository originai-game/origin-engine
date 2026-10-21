//! og.Mesh's hit flash rides in its instance's record (row 85), as
//! og.Model's does: three meshes of one geometry and material, one flashing,
//! reach the renderer as three instances of that one material, the flashing
//! one carrying its amount (continuous, not one of 16 levels) and colour.
//! `og.tween.flash` on an og.Mesh entity still lights it fully on its first
//! tick and fades it out.
//!
//! Ways this could fail: the flash still draws through a material copy (a
//! second material handle), the amount is quantised, or the flash is lost on
//! its way to the instance. Broken once by restoring the 16-level copy: the
//! one-material assertion fails.

use og_render::{InstanceFx, MaterialHandle, RenderScene};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_box_geometry, og_render_standard_material, og_tween_flash,
    og_world_insert, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

/// The last frame's instances: material and look.
type Seen = Rc<RefCell<Vec<(MaterialHandle, InstanceFx)>>>;

struct Recorder(Seen);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.instances.iter().map(|i| (i.material, i.fx)).collect();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn engine() -> (u32, Seen) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-mesh-flash-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Seen::default();
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    frame(engine);
    (engine, seen)
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn_mesh(engine: u32, mesh: Value) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    for (name, v) in [("og.Transform", json!({})), ("og.Mesh", mesh)] {
        value(og_world_insert(engine, e, name.into(), Some(v.to_string())));
    }
    e
}

#[test]
fn a_flashing_mesh_is_one_material_with_its_twins() {
    let (engine, seen) = engine();
    let material = value(og_render_standard_material(engine, Some("{}".into())));
    let geometry = value(og_render_box_geometry(engine, Some("{}".into())));
    let red = json!({ "r": 1, "g": 0.1, "b": 0.05 });
    spawn_mesh(
        engine,
        json!({ "geometry": geometry, "material": material }),
    );
    spawn_mesh(
        engine,
        json!({ "geometry": geometry, "material": material, "flash": 0.37, "flashColor": red }),
    );
    let plain = spawn_mesh(
        engine,
        json!({ "geometry": geometry, "material": material }),
    );
    frame(engine);
    {
        let s = seen.borrow();
        assert_eq!(s.len(), 3);
        assert!(s.iter().all(|i| i.0 == s[0].0), "one material: {s:?}");
        let flashes: Vec<f32> = s.iter().map(|i| i.1.flash).collect();
        assert_eq!(flashes, vec![0.0, 0.37, 0.0], "continuous, not 6/16");
        assert_eq!(s[1].1.flash_color, og_core::glam::Vec3::new(1.0, 0.1, 0.05));
        assert_eq!(s[1].1.tint, og_core::glam::Vec3::ONE);
    }
    // og.tween.flash takes an og.Mesh entity as before: full on the tick it
    // starts, then fading, the material never copied.
    let desc = json!({ "entity": plain, "duration": 0.2 }).to_string();
    value(og_tween_flash(engine, Some(desc)));
    frame(engine);
    let first = seen.borrow()[2].1.flash;
    assert_eq!(first, 1.0);
    for _ in 0..4 {
        frame(engine);
    }
    let s = seen.borrow();
    assert!(s[2].1.flash > 0.0 && s[2].1.flash < first, "{:?}", s[2]);
    assert!(s.iter().all(|i| i.0 == s[0].0), "one material: {s:?}");
}
