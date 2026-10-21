//! Shadow flags from game code to the renderer (owed row 87), and
//! og.Model's `visible` (meridian-loop FINDINGS #14): og.Mesh's
//! `receiveShadow`, og.Model's `castShadow` and `receiveShadow`, and
//! GltfDesc's, through reflection, world.insert/get, world.instantiate and
//! a cooked scene's text, each reaching the frame's instances
//! (`Instance::cast_shadows`/`receive_shadows`, and a posed model's
//! `SkinnedInstance` ones).
//!
//! The defaults are the contract: a model loaded without a desc casts and
//! receives (every game loads its models so, and a model that silently
//! stopped casting would be a regression), a mesh receives and does not
//! cast (three.js's castShadow; every mesh received before the flag was
//! applied).
//!
//! Ways this could fail: the flags stay declared but unread (a model with
//! castShadow false still casts: the old runtime drew every model casting
//! and receiving, every mesh receiving); a default flips (a model loaded
//! without a desc casts nothing, a mesh without the flag is lit unshadowed);
//! the scene text's Model table is dropped beside its `model` line, or a
//! second og.Model is spawned for it; a posed model's skinned instances
//! keep casting.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_animation, og_load_gltf, og_load_scene, og_render_box_geometry,
    og_render_standard_material, og_world_find, og_world_get, og_world_insert,
    og_world_instantiate, og_world_set, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// og.Model's and og.Mesh's schemas as `og cook` packs scene tables with
/// them (og-cli's `engine_schemas`: the IDL's fields and defaults).
fn engine_schema(name: &str) -> og_core::Schema {
    use og_core::reflect::{FieldSchema, FieldType};
    let r = og_api::host::RECORDS
        .iter()
        .find(|r| r.kind == "component" && r.name == name)
        .unwrap_or_else(|| panic!("no component {name} in the IDL"));
    let fields = r
        .fields
        .iter()
        .map(|f| {
            let (ty, lanes) = match f.column {
                "f32" => (FieldType::F32, 1),
                "i32" => (FieldType::I32, 1),
                "bool" => (FieldType::Bool, 1),
                "Color" => (FieldType::Vec3, 3),
                "Asset" | "Geometry" | "Material" => (FieldType::U64, 1),
                other => panic!("{name}.{}: column {other}", f.name),
            };
            let mut fs = FieldSchema::new(f.name.to_string(), ty);
            let v: Value = serde_json::from_str(f.default).unwrap();
            let lane = |v: &Value| match v {
                Value::Bool(b) => f64::from(u8::from(*b)),
                v => v.as_f64().unwrap(),
            };
            fs.default = Some(match &v {
                Value::Object(m) => ["r", "g", "b"].iter().map(|k| lane(&m[*k])).collect(),
                v => vec![lane(v)],
            });
            assert_eq!(fs.default.as_ref().unwrap().len(), lanes);
            fs
        })
        .collect();
    og_core::Schema {
        name: name.into(),
        version: 1,
        doc: String::new(),
        fields,
    }
}

const SCENE: &str = r#"type scene
version 1
---
name "Flags"
entity "plain" {
  model "models/crate.vox"
}
entity "dark" {
  model "models/crate.vox"
  Model {
    castShadow false
    receiveShadow false
  }
}
entity "sign" {
  Mesh {
    castShadow true
    receiveShadow false
    visible false
  }
}
"#;

/// The project (a static `.vox`, the template walker and the scene),
/// cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-shadow-flags-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let crate_vox = std::fs::read(repo().join("examples/lantern-vale/assets/models/rock.vox"))
            .expect("a static .vox");
        write("models/crate.vox", &crate_vox);
        let walker = std::fs::read(repo().join("templates/game/assets/models/walker.glb")).unwrap();
        write("models/walker.glb", &walker);
        write("scenes/flags.ogt", SCENE.as_bytes());
        let options = og_cook::CookOptions {
            threads: 1,
            schemas: vec![engine_schema("Model"), engine_schema("Mesh")],
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
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

/// The last frame's (cast, receive) flags: of its instances by mesh
/// index, and of its skinned instances.
#[derive(Default)]
struct Seen {
    instances: Vec<(usize, bool, bool)>,
    skinned: Vec<(bool, bool)>,
}

impl Seen {
    /// The flags of the instances of meshes `meshes`, each pair once.
    fn of(&self, meshes: &[usize]) -> Vec<(bool, bool)> {
        let mut out: Vec<(bool, bool)> = self
            .instances
            .iter()
            .filter(|i| meshes.contains(&i.0))
            .map(|i| (i.1, i.2))
            .collect();
        out.sort();
        out.dedup();
        out
    }
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
        let mut s = self.0.borrow_mut();
        s.instances = scene
            .instances
            .iter()
            .map(|i| (i.mesh.index(), i.cast_shadows, i.receive_shadows))
            .collect();
        s.skinned = scene
            .skinned_instances
            .iter()
            .map(|i| (i.cast_shadows, i.receive_shadows))
            .collect();
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

fn refused(text: String) -> String {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(false), "{text}");
    v["reason"].as_str().unwrap().to_string()
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
    frame(engine);
    (engine, seen)
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

/// (castShadow, receiveShadow) of an entity's `component`.
fn flags(engine: u32, e: f64, component: &str) -> (bool, bool) {
    let v = value(og_world_get(engine, e, component.into()));
    (
        v["castShadow"].as_bool().unwrap(),
        v["receiveShadow"].as_bool().unwrap(),
    )
}

/// The scene meshes a frame draws for model entity `e` alone: the meshes
/// of the instances that appear when it is the only new model.
fn meshes_of(seen: &Rc<RefCell<Seen>>, before: &[(usize, bool, bool)]) -> Vec<usize> {
    let mut m: Vec<usize> = seen
        .borrow()
        .instances
        .iter()
        .map(|i| i.0)
        .filter(|m| !before.iter().any(|b| b.0 == *m))
        .collect();
    m.sort();
    m.dedup();
    m
}

/// The schema every reader shares (the inspector, Tune, the probe, scene
/// text, the wiki): the IDL's defaults are the contract's.
#[test]
fn reflection_declares_the_defaults() {
    let default = |record: &str, field: &str| {
        og_api::host::RECORDS
            .iter()
            .find(|r| r.kind == "component" && r.name == record)
            .and_then(|r| r.fields.iter().find(|f| f.name == field))
            .unwrap_or_else(|| panic!("{record}.{field}"))
            .default
    };
    assert_eq!(default("Mesh", "castShadow"), "false");
    assert_eq!(default("Mesh", "receiveShadow"), "true");
    assert_eq!(default("Model", "castShadow"), "true");
    assert_eq!(default("Model", "receiveShadow"), "true");
}

#[test]
fn a_model_loaded_without_flags_casts_and_receives_and_one_turned_off_does_neither() {
    let (engine, seen) = engine();
    let asset = value(og_load_gltf(engine, "models/crate.vox".into(), None));
    let plain = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    assert_eq!(flags(engine, plain, "og.Model"), (true, true));
    frame(engine);
    let crate_meshes = meshes_of(&seen, &[]);
    assert!(!crate_meshes.is_empty());
    assert_eq!(seen.borrow().of(&crate_meshes), vec![(true, true)]);

    // The same path loaded again with a desc: the same handle, its flags
    // for the models instantiated after it.
    let off = json!({ "castShadow": false, "receiveShadow": false }).to_string();
    let again = value(og_load_gltf(engine, "models/crate.vox".into(), Some(off)));
    assert_eq!(again, asset);
    let dark = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    assert_eq!(flags(engine, dark, "og.Model"), (false, false));
    assert_eq!(flags(engine, plain, "og.Model"), (true, true), "kept");
    frame(engine);
    assert_eq!(
        seen.borrow().of(&crate_meshes),
        vec![(false, false), (true, true)]
    );

    // Per instance: og.Model's own fields, set after the fact.
    let only_cast = json!({ "castShadow": false }).to_string();
    value(og_world_set(engine, plain, "og.Model".into(), only_cast));
    frame(engine);
    assert_eq!(
        seen.borrow().of(&crate_meshes),
        vec![(false, false), (false, true)]
    );

    // A desc's omitted field is on; a flag that is not a boolean is refused.
    let half = json!({ "receiveShadow": false }).to_string();
    value(og_load_gltf(engine, "models/crate.vox".into(), Some(half)));
    let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    assert_eq!(flags(engine, e, "og.Model"), (true, false));
    let bad = json!({ "castShadow": "no" }).to_string();
    let reason = refused(og_load_gltf(engine, "models/crate.vox".into(), Some(bad)));
    assert!(reason.contains("GltfDesc.castShadow"), "{reason}");
}

/// A posed model's skinned instances carry its flags.
#[test]
fn a_posed_models_skinned_instances_carry_its_flags() {
    let (engine, seen) = engine();
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let clip = anim["clips"].as_array().unwrap()[0]["clip"].clone();
    let mut walkers = Vec::new();
    for desc in [
        None,
        Some(json!({ "castShadow": false, "receiveShadow": false })),
    ] {
        let asset = value(og_load_gltf(
            engine,
            "models/walker.glb".into(),
            desc.map(|d| d.to_string()),
        ));
        let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
            .as_f64()
            .unwrap();
        for (name, v) in [
            ("og.Skeleton", json!({ "asset": anim["skeleton"] })),
            ("og.AnimationPlayer", json!({ "clip": clip, "loop": true })),
        ] {
            value(og_world_insert(engine, e, name.into(), Some(v.to_string())));
        }
        walkers.push(e);
    }
    for _ in 0..3 {
        frame(engine);
    }
    let mut skinned = seen.borrow().skinned.clone();
    assert!(!skinned.is_empty(), "the walkers are posed");
    skinned.sort();
    skinned.dedup();
    assert_eq!(skinned, vec![(false, false), (true, true)]);
    value(og_world_set(
        engine,
        walkers[1],
        "og.Model".into(),
        json!({ "receiveShadow": true }).to_string(),
    ));
    frame(engine);
    let mut skinned = seen.borrow().skinned.clone();
    skinned.sort();
    skinned.dedup();
    assert_eq!(skinned, vec![(false, true), (true, true)]);
}

/// og.Model.visible (meridian-loop FINDINGS #14: a ghost car could only
/// be hidden by moving it under the world): a hidden model, static or
/// posed, draws no instance; shown again, it draws as before.
#[test]
fn a_hidden_model_draws_nothing_posed_or_not() {
    let (engine, seen) = engine();
    let crate_asset = value(og_load_gltf(engine, "models/crate.vox".into(), None));
    let still = value(og_world_instantiate(
        engine,
        crate_asset.as_f64().unwrap(),
        None,
    ))
    .as_f64()
    .unwrap();
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let clip = anim["clips"].as_array().unwrap()[0]["clip"].clone();
    let walker_asset = value(og_load_gltf(engine, "models/walker.glb".into(), None));
    let walker = value(og_world_instantiate(
        engine,
        walker_asset.as_f64().unwrap(),
        None,
    ))
    .as_f64()
    .unwrap();
    for (name, v) in [
        ("og.Skeleton", json!({ "asset": anim["skeleton"] })),
        ("og.AnimationPlayer", json!({ "clip": clip, "loop": true })),
    ] {
        value(og_world_insert(
            engine,
            walker,
            name.into(),
            Some(v.to_string()),
        ));
    }
    for _ in 0..3 {
        frame(engine);
    }
    let shown = (seen.borrow().instances.len(), seen.borrow().skinned.len());
    assert!(shown.0 > 0 && shown.1 > 0, "{shown:?}");
    assert_eq!(
        value(og_world_get(engine, still, "og.Model".into()))["visible"],
        json!(true)
    );
    for e in [still, walker] {
        let hide = json!({ "visible": false }).to_string();
        value(og_world_set(engine, e, "og.Model".into(), hide));
    }
    frame(engine);
    assert_eq!(seen.borrow().instances.len(), 0);
    assert_eq!(seen.borrow().skinned.len(), 0);
    for e in [still, walker] {
        let show = json!({ "visible": true }).to_string();
        value(og_world_set(engine, e, "og.Model".into(), show));
    }
    frame(engine);
    let again = (seen.borrow().instances.len(), seen.borrow().skinned.len());
    assert_eq!(again, shown);
}

#[test]
fn a_mesh_receives_unless_told_not_to() {
    let (engine, seen) = engine();
    let material = value(og_render_standard_material(engine, Some("{}".into())));
    let geometry = value(og_render_box_geometry(engine, Some("{}".into())));
    let spawn = |mesh: Value| {
        let e = value(og_world_spawn(engine, Some("{}".into())))
            .as_f64()
            .unwrap();
        value(og_world_insert(
            engine,
            e,
            "og.Mesh".into(),
            Some(mesh.to_string()),
        ));
        e
    };
    let plain = spawn(json!({ "geometry": geometry, "material": material }));
    let sign = spawn(json!({
        "geometry": geometry, "material": material, "castShadow": true, "receiveShadow": false
    }));
    assert_eq!(flags(engine, plain, "og.Mesh"), (false, true));
    assert_eq!(flags(engine, sign, "og.Mesh"), (true, false));
    frame(engine);
    let mut s: Vec<(bool, bool)> = seen.borrow().instances.iter().map(|i| (i.1, i.2)).collect();
    s.sort();
    assert_eq!(s, vec![(false, true), (true, false)]);
}

/// A cooked scene's text carries the flags: a `Model` table beside a
/// `model` line sets the model's (one og.Model, its asset the line's), a
/// `model` line alone takes the load's GltfDesc flags, and a Mesh table
/// its own (hidden: it has no geometry; `a_mesh_receives_unless_told_not_to`
/// draws a mesh's).
#[test]
fn the_flags_round_trip_through_the_scene_text() {
    let (engine, seen) = engine();
    let scene = value(og_load_scene(engine, "scenes/flags.ogt".into()));
    value(og_world_instantiate(engine, scene.as_f64().unwrap(), None));
    let find = |name: &str| {
        serde_json::from_str::<Value>(&og_world_find(engine, name.into()))
            .unwrap()
            .as_f64()
            .unwrap_or_else(|| panic!("no entity '{name}'"))
    };
    let (plain, dark, sign) = (find("plain"), find("dark"), find("sign"));
    assert_eq!(flags(engine, plain, "og.Model"), (true, true));
    assert_eq!(flags(engine, dark, "og.Model"), (false, false));
    assert_eq!(flags(engine, sign, "og.Mesh"), (true, false));
    let asset = |e: f64| value(og_world_get(engine, e, "og.Model".into()))["asset"].clone();
    assert_eq!(
        asset(plain),
        asset(dark),
        "the line's asset in the table's Model"
    );
    assert!(asset(dark).as_f64().unwrap() > 0.0);
    frame(engine);
    let crate_meshes = meshes_of(&seen, &[]);
    assert_eq!(
        seen.borrow().of(&crate_meshes),
        vec![(false, false), (true, true)]
    );
}

/// A `model` line without a table takes the flags of the glTF's load; a
/// `Model` table beside one keeps its own.
#[test]
fn a_scene_model_line_takes_the_loads_flags() {
    let (engine, _) = engine();
    let off = json!({ "castShadow": false }).to_string();
    value(og_load_gltf(engine, "models/crate.vox".into(), Some(off)));
    let scene = value(og_load_scene(engine, "scenes/flags.ogt".into()));
    value(og_world_instantiate(engine, scene.as_f64().unwrap(), None));
    let find = |name: &str| {
        serde_json::from_str::<Value>(&og_world_find(engine, name.into()))
            .unwrap()
            .as_f64()
            .unwrap_or_else(|| panic!("no entity '{name}'"))
    };
    assert_eq!(flags(engine, find("plain"), "og.Model"), (false, true));
    assert_eq!(flags(engine, find("dark"), "og.Model"), (false, false));
}
