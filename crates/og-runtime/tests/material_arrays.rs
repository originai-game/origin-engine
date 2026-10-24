//! Texture arrays through the runtime's materials: a StandardMaterialDesc
//! naming `mapArray` (with `mapLayer`) and `normalMapArray` reaches
//! og-render's StandardMaterial as the arrays' slots, and an og.Mesh's
//! `materialLayer` becomes its instance's layer (-1, the default, keeps the
//! material's), so meshes of one material on different layers stay one
//! material. A terrain whose layers name texture arrays (`layers`,
//! `layerNormals`) draws with both arrays and each layer's layers.
//!
//! Ways this could fail: the desc's array fields are dropped (no slot), a
//! mesh's layer is lost on its way to the instance, or -1 reaches the
//! renderer as a layer. Broken once by pushing every mesh with
//! `add_instance` (no layer): the layer assertion fails.

use og_render::{RenderScene, StandardMaterial};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_texture_array, og_render_box_geometry,
    og_render_standard_material, og_render_update_material, og_world_insert, og_world_spawn,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project of two texture arrays (colour and normal), cooked once.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-material-arrays-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let png =
            std::fs::read(repo().join("templates/fx-showcase/assets/textures/toon.png")).unwrap();
        write("tex/a.png", &png);
        write("tex/b.png", &png);
        write(
            "tex/tiles.ogt",
            b"type texture-array\nversion 1\n---\nlayer \"a.png\"\nlayer \"b.png\"\n",
        );
        let hill = std::fs::read(
            repo().join("packages/engine/tests/fixtures/terrain/assets/terrain/hill.png"),
        )
        .unwrap();
        write("terrain/hill.png", &hill);
        write(
            "terrain/hill.ogt",
            b"type terrain\nversion 1\n---\nheightmap \"hill.png\"\nheight 0 10\nsize 64 64\ntile 64\nlayers \"tex/tiles.ogt\"\nlayerNormals \"tex/tiles-n.ogt\"\nlayer {\n  tile 4\n}\nlayer {\n  albedoLayer 0\n  normalLayer 1\n  slope 30 90\n}\n",
        );
        write(
            "tex/tiles-n.ogt",
            b"type texture-array\nversion 1\n---\nkind normal\nlayer \"a.png\"\nlayer \"b.png\"\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// What the last frame's scene named: per instance its layer and its
/// material's array slots and layers.
#[derive(Default)]
struct Seen {
    instances: Vec<(Option<u32>, StandardMaterial)>,
    terrains: Vec<og_render::Terrain>,
}

struct Recorder(Rc<RefCell<Seen>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().instances = scene
            .instances
            .iter()
            .map(|i| {
                (
                    i.material_layer,
                    *scene.standard_material(i.material).unwrap(),
                )
            })
            .collect();
        self.0.borrow_mut().terrains = scene.terrains.clone();
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

fn engine() -> (u32, Rc<RefCell<Seen>>) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    (engine, seen)
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn(engine: u32, components: &[(&str, Value)]) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    for (name, v) in components {
        value(og_world_insert(
            engine,
            e,
            name.to_string(),
            Some(v.to_string()),
        ));
    }
    e
}

#[test]
fn a_material_samples_arrays_and_each_mesh_picks_its_layer() {
    let (engine, seen) = engine();
    let tiles = value(og_load_texture_array(engine, "tex/tiles.ogt".into()))["array"]
        .as_f64()
        .unwrap();
    let normals = value(og_load_texture_array(engine, "tex/tiles-n.ogt".into()))["array"]
        .as_f64()
        .unwrap();
    let desc = json!({ "mapArray": tiles, "mapLayer": 1, "normalMapArray": normals });
    let material = value(og_render_standard_material(engine, Some(desc.to_string())));
    let geometry = value(og_render_box_geometry(engine, Some("{}".into())));
    for layer in [0, -1, 1] {
        let mut mesh = json!({ "geometry": geometry, "material": material });
        if layer >= 0 {
            mesh["materialLayer"] = json!(layer);
        }
        spawn(engine, &[("og.Transform", json!({})), ("og.Mesh", mesh)]);
    }
    frame(engine);
    {
        let s = seen.borrow();
        assert_eq!(s.instances.len(), 3);
        let layers: Vec<Option<u32>> = s.instances.iter().map(|i| i.0).collect();
        assert_eq!(
            layers,
            vec![Some(0), None, Some(1)],
            "-1 keeps the material's"
        );
        let m = &s.instances[0].1;
        assert!(m.mapArray != 0 && m.normalMapArray != 0 && m.mapArray != m.normalMapArray);
        assert_eq!((m.mapLayer, m.normalMapLayer), (1, 0));
        assert!(m.draws_array_maps());
        assert!(s.instances.iter().all(|i| i.1 == *m), "one material");
    }
    // null clears an array; an unknown handle is refused by name.
    let cleared = json!({ "normalMapArray": null, "normalMapLayer": 1 });
    value(og_render_update_material(
        engine,
        material.as_f64().unwrap(),
        Some(cleared.to_string()),
    ));
    frame(engine);
    let m = seen.borrow().instances[0].1;
    assert_eq!((m.normalMapArray, m.normalMapLayer), (0, 1));
    let bad = json!({ "mapArray": 77.0 });
    let refused: Value =
        serde_json::from_str(&og_render_standard_material(engine, Some(bad.to_string()))).unwrap();
    assert_eq!(refused["ok"], json!(false));
    assert!(
        refused
            .to_string()
            .contains("mapArray: texture array 77 is not loaded"),
        "{refused}"
    );
}

#[test]
#[cfg(not(feature = "profile2d"))] // the 2D profile has no terrain (tests/profile2d.rs)
fn a_terrain_samples_its_layer_arrays() {
    let (engine, seen) = engine();
    let info = value(og_runtime::og_terrain_load(
        engine,
        "terrain/hill.ogt".into(),
    ));
    spawn(
        engine,
        &[
            ("og.Transform", json!({})),
            ("og.Terrain", json!({ "data": info["data"] })),
        ],
    );
    frame(engine);
    let s = seen.borrow();
    assert_eq!(s.terrains.len(), 1);
    let t = &s.terrains[0];
    let (albedo, normal) = (t.layer_arrays.albedo, t.layer_arrays.normal);
    assert!(
        albedo.is_some() && normal.is_some() && albedo != normal,
        "{:?}",
        t.layer_arrays
    );
    let picked: Vec<_> = t
        .layers
        .iter()
        .map(|l| (l.albedo_layer, l.normal_layer))
        .collect();
    assert_eq!(picked, vec![(Some(0), Some(0)), (Some(0), Some(1))]);
    assert!(
        t.layers
            .iter()
            .all(|l| l.albedo.is_none() && l.normal.is_none())
    );
}
