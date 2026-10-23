//! Baked lighting of scenes loaded together (debt render-light #1): every
//! instantiated scene's bake is handed to the renderer, each static model
//! drawn with its lightmapped parts and the pages of its own scene, and a
//! bake whose static models are all despawned stops drawing.
//!
//! The project is cooked here twice: once to learn the scenes' and the
//! model's output hashes, then with a lightmap beside each scene (the
//! fx-showcase Cornell box's bake, its hashes set to this project's), as
//! `og bake lightmaps` would have written them.
//!
//! Ways this could fail: the second scene's bake replaces the first's (the
//! old runtime drew "one scene's lightmap at a time": the first box's
//! instances then drew its unbaked parts, with no lightmap); both scenes'
//! meshes are handed over with one set of pages (the second box would
//! sample the first's); a despawned scene keeps its pages drawn; a render
//! snapshot's restore (`og render --jobs`) brings back the world with the
//! static models but not their bake, or keeps a bake drawn whose models the
//! restored world does not have.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_scene, og_render_baked_lighting, og_world_despawn, og_world_find,
    og_world_instantiate, render_restore, render_snapshot, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// og.Model's schema as `og cook` packs scene tables with it (og-cli's
/// `engine_schemas`: the IDL's fields and defaults).
fn model_schema() -> og_core::Schema {
    use og_core::reflect::{FieldSchema, FieldType};
    let r = og_api::host::RECORDS
        .iter()
        .find(|r| r.kind == "component" && r.name == "Model")
        .expect("og.Model in the IDL");
    let fields = r
        .fields
        .iter()
        .map(|f| {
            let ty = match f.column {
                "f32" => FieldType::F32,
                "i32" => FieldType::I32,
                "u32" => FieldType::U32,
                "bool" => FieldType::Bool,
                "Color" => FieldType::Vec3,
                "Asset" => FieldType::U64,
                other => panic!("Model.{}: column {other}", f.name),
            };
            let mut fs = FieldSchema::new(f.name.to_string(), ty);
            let lane = |v: &Value| match v {
                Value::Bool(b) => f64::from(u8::from(*b)),
                v => v.as_f64().unwrap(),
            };
            let v: Value = serde_json::from_str(f.default).unwrap();
            fs.default = Some(match &v {
                Value::Object(m) => ["r", "g", "b"].iter().map(|k| lane(&m[*k])).collect(),
                v => vec![lane(v)],
            });
            fs
        })
        .collect();
    og_core::Schema {
        name: "Model".into(),
        version: 1,
        doc: String::new(),
        fields,
    }
}

/// A scene of one static Cornell box named `entity`.
fn scene_text(name: &str, entity: &str) -> String {
    format!(
        "type scene\nversion 1\n---\nname \"{name}\"\nentity \"{entity}\" {{\n  model \"models/cornell.glb\"\n  static true\n}}\n"
    )
}

/// The project with two scenes, `scenes/hall.ogt` and `scenes/annex.ogt`,
/// each baked, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-baked-scenes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let assets = root.join("assets");
        let write = |path: &str, bytes: &[u8]| {
            let p = assets.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let showcase = repo().join("templates/fx-showcase/assets");
        write(
            "models/cornell.glb",
            &std::fs::read(showcase.join("models/cornell.glb")).unwrap(),
        );
        let scenes = [("hall", "hall-box"), ("annex", "annex-box")];
        for (name, entity) in scenes {
            write(
                &format!("scenes/{name}.ogt"),
                scene_text(name, entity).as_bytes(),
            );
        }
        let options = og_cook::CookOptions {
            threads: 1,
            schemas: vec![model_schema()],
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let hash = |id: &str| bundle.output(id).unwrap().hash;
        let text = std::fs::read_to_string(showcase.join("scenes/cornell.lightmap.ogt")).unwrap();
        let pages = std::fs::read(showcase.join("scenes/cornell.lightmap.ktx2")).unwrap();
        for (name, entity) in scenes {
            let id = format!("scenes/{name}.ogt");
            let mut file = og_cook::lightmap::LightmapFile::parse(&text).unwrap();
            file.scene = id.clone();
            file.scene_hash = hash(&id);
            file.pages = og_cook::lightmap::pages_name(&id);
            assert_eq!(file.models.len(), 1, "the box is one static model");
            file.models[0].entity = entity.into();
            file.models[0].hash = hash("models/cornell.glb");
            write(
                &og_cook::lightmap::lightmap_id(&id),
                file.to_text().as_bytes(),
            );
            write(&format!("scenes/{}", file.pages), &pages);
        }
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

/// What the last frame handed the renderer: its instances' meshes, and of
/// its lightmapped meshes the pages each samples (by the Arc's address).
#[derive(Default)]
struct Seen {
    instances: Vec<usize>,
    lightmapped: BTreeMap<usize, usize>,
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
        s.instances = scene.instances.iter().map(|i| i.mesh.index()).collect();
        s.lightmapped = scene
            .baked
            .meshes
            .iter()
            .map(|(&m, b)| (m, std::sync::Arc::as_ptr(&b.pages) as usize))
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

fn instantiate(engine: u32, scene: &str) {
    let scene = value(og_load_scene(engine, scene.into()));
    value(og_world_instantiate(engine, scene.as_f64().unwrap(), None));
}

/// `og.render.bakedLighting`: per scene, whether it is baked and why not.
fn reported(engine: u32) -> Vec<(String, bool, String)> {
    let list: Value = serde_json::from_str(&og_render_baked_lighting(engine)).unwrap();
    list.as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["scene"].as_str().unwrap().to_string(),
                s["baked"].as_bool().unwrap(),
                s["reason"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// Two baked scenes instantiated one after the other both draw their bake:
/// every instance of either box is a lightmapped mesh, and the two boxes'
/// meshes sample two sets of pages, each box one.
#[test]
fn two_baked_scenes_both_draw_their_bakes() {
    let (engine, seen) = engine();
    instantiate(engine, "scenes/hall.ogt");
    frame(engine);
    let hall: BTreeSet<usize> = seen.borrow().instances.iter().copied().collect();
    assert!(!hall.is_empty(), "the hall's box is drawn");
    instantiate(engine, "scenes/annex.ogt");
    frame(engine);
    assert_eq!(
        reported(engine),
        vec![
            ("scenes/hall.ogt".into(), true, String::new()),
            ("scenes/annex.ogt".into(), true, String::new()),
        ]
    );
    let s = seen.borrow();
    let annex: BTreeSet<usize> = s
        .instances
        .iter()
        .copied()
        .filter(|m| !hall.contains(m))
        .collect();
    assert_eq!(annex.len(), hall.len(), "the annex's box is drawn");
    for m in &s.instances {
        assert!(
            s.lightmapped.contains_key(m),
            "mesh {m} draws without its lightmap ({:?} lightmapped)",
            s.lightmapped.keys()
        );
    }
    let pages_of = |meshes: &BTreeSet<usize>| -> BTreeSet<usize> {
        meshes.iter().map(|m| s.lightmapped[m]).collect()
    };
    let (a, b) = (pages_of(&hall), pages_of(&annex));
    assert_eq!(a.len(), 1, "the hall's parts share its pages");
    assert_eq!(b.len(), 1, "the annex's parts share its pages");
    assert_ne!(a, b, "each box samples its own scene's pages");
    drop(s);

    // The hall's box despawned: its bake stops drawing, the annex's stays.
    let found: Value = serde_json::from_str(&og_world_find(engine, "hall-box".into())).unwrap();
    value(og_world_despawn(engine, found.as_f64().unwrap()));
    frame(engine);
    let s = seen.borrow();
    assert!(s.lightmapped.keys().all(|m| annex.contains(m)));
    assert_eq!(s.lightmapped.len(), annex.len());
    let r = reported(engine);
    assert_eq!(
        (r[0].1, r[0].2.as_str()),
        (false, "its static models were despawned")
    );
    assert!(r[1].1, "{r:?}");
}

/// A render snapshot keeps the baked lighting with the world it restores
/// (debt render-light #7): restored after the box was despawned, its bake
/// draws again with the same lightmapped meshes and pages; restored to a
/// snapshot taken before any scene was instantiated, nothing is drawn baked.
#[test]
fn a_render_snapshot_restores_the_bake_with_its_world() {
    let (engine, seen) = engine();
    let empty = render_snapshot(engine).expect("snapshot");
    instantiate(engine, "scenes/hall.ogt");
    frame(engine);
    let lit = |s: &Seen| (s.instances.clone(), s.lightmapped.clone());
    let before = lit(&seen.borrow());
    assert!(!before.1.is_empty(), "the hall draws its bake");
    let snapshot = render_snapshot(engine).expect("snapshot");

    let found: Value = serde_json::from_str(&og_world_find(engine, "hall-box".into())).unwrap();
    value(og_world_despawn(engine, found.as_f64().unwrap()));
    frame(engine);
    assert!(
        seen.borrow().lightmapped.is_empty(),
        "despawned, no bake drawn"
    );

    render_restore(engine, &snapshot).expect("restore");
    frame(engine);
    assert_eq!(
        lit(&seen.borrow()),
        before,
        "the restored box draws its bake as before"
    );
    assert_eq!(reported(engine)[0].1, true);

    render_restore(engine, &empty).expect("restore");
    frame(engine);
    let s = seen.borrow();
    assert!(
        s.instances.is_empty() && s.lightmapped.is_empty(),
        "nothing baked drawn"
    );
}
