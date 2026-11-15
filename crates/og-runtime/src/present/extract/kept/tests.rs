//! The kept list against a fresh one: two engines run the same world in
//! step, one forgetting its kept models before every frame, and every
//! frame each draws the same instances, levels, impostors, keys, levels
//! of detail and HLOD replacements.
//!
//! Ways this could fail: a row copied after its model moved, was tinted
//! or hidden, or after the camera moved its level; a proxy copied after
//! a house it stands for changed; a copied row put at another place in
//! the list or with another key; the cluster path's levels shifted or
//! padded differently; the hysteresis of a copied row lost; and nothing
//! copied, so the test checks nothing (as when the wind's writes to its
//! copies' parameters, every tick, counted as changed materials).

use crate::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_load_gltf, og_load_scene,
    og_world_despawn, og_world_find, og_world_insert, og_world_instantiate, og_world_set,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const HOUSE: &str = r#"type model
version 1
---
material "wall" {
  color 0.6 0.08 0.02
  roughness 0.9
}
material "roof" {
  color 0.02 0.1 0.5
  roughness 0.4
}
part "walls" {
  material "wall"
  position 0 2 0
  boxGeometry {
    width 6
    height 4
    depth 6
  }
}
part "roof" {
  material "roof"
  position 0 5.5 0
  coneGeometry {
    radius 4.2
    height 3
    radialSegments 4
  }
}
"#;

/// 6 × 6 houses 12 m apart under an HLOD tree of 32 m leaves.
fn town() -> String {
    let mut text = String::from(
        "type scene\nversion 1\n---\nname \"Town\"\nhlod {\n  size 32\n  error 0.25\n  texels 256\n}\n",
    );
    for j in 0..6 {
        for i in 0..6 {
            text.push_str(&format!(
                "entity \"house-{i}-{j}\" {{\n  model \"models/house.ogt\"\n  static true\n  Transform {{\n    position {} 0 {}\n  }}\n}}\n",
                3 + 12 * i,
                3 + 12 * j
            ));
        }
    }
    text
}

/// og.Model's fields the town sets, as `og cook` packs them.
fn model_schema() -> og_core::Schema {
    use og_core::reflect::{FieldSchema, FieldType};
    let field = |name: &str, on: f64| {
        let mut f = FieldSchema::new(name.to_string(), FieldType::Bool);
        f.default = Some(vec![on]);
        f
    };
    og_core::Schema {
        name: "Model".into(),
        version: 1,
        doc: String::new(),
        fields: vec![field("static", 0.0), field("visible", 1.0)],
    }
}

/// The town and the strider (a glTF with cooked levels), cooked once.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-runtime-kept-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (id, text) in [
            ("models/house.ogt", HOUSE.to_string()),
            ("scenes/town.ogt", town()),
        ] {
            let p = root.join("assets").join(id);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::fs::copy(
            repo.join("tests/fixtures/motion/assets/models/strider.glb"),
            root.join("assets/models/strider.glb"),
        )
        .unwrap();
        let options = og_cook::CookOptions {
            schemas: vec![model_schema()],
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

/// What a frame handed the renderer of the models, every bit of it.
struct Recorder(Rc<RefCell<String>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (1920, 1080)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        let o = &scene.objects;
        *self.0.borrow_mut() = format!(
            "{:?}\n{:?}\n{:?}",
            o.instances, o.instance_levels, o.impostors
        );
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
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn answer<T: crate::host::wire::ToJson>(r: Result<T, String>) -> Value {
    value(crate::host_impl::answer(r))
}

/// One of the two engines in step.
struct Run {
    e: u32,
    drawn: Rc<RefCell<String>>,
    /// Forgets its kept models before every frame.
    fresh: bool,
}

impl Run {
    fn new(fresh: bool) -> Run {
        let root = project();
        set_hooks(Rc::new(Disk));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let options = json!({ "root": root.to_str().unwrap() }).to_string();
        let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
        let drawn = Rc::new(RefCell::new(String::new()));
        set_render_backend(e, Some(Box::new(Recorder(drawn.clone())))).unwrap();
        // The cluster path on, so the rows' levels for it are kept too.
        crate::engine::engine(e)
            .unwrap()
            .render
            .borrow_mut()
            .scene
            .objects
            .meshlets = true;
        // A wind: the swaying striders' wind copies are written every tick.
        let wind = json!({ "direction": { "x": 1, "y": 0, "z": 0 }, "speed": 5 });
        value(og_engine_write_resource(
            e,
            "og.Wind".into(),
            wind.to_string(),
        ));
        let town = answer(og_load_scene(e, "scenes/town.ogt".into()));
        answer(og_world_instantiate(e, town.as_f64().unwrap(), None));
        let strider = answer(og_load_gltf(e, "models/strider.glb".into(), None));
        // Striders from a few metres to a kilometre off, some tinted,
        // flashing or swaying: every level of theirs is drawn.
        for i in 0..24 {
            let z = -(2.0 + 1.8 * (i * i) as f64);
            let desc = json!({
                "name": format!("strider-{i}"),
                "position": { "x": 30 + i % 5, "y": 0, "z": z },
            });
            let id = answer(og_world_spawn(e, Some(desc.to_string())));
            let model = json!({
                "asset": strider,
                "tint": { "r": 1.0, "g": if i % 3 == 0 { 0.7 } else { 1.0 }, "b": 1.0 },
                "flash": if i % 7 == 0 { 0.3 } else { 0.0 },
                "sway": if i % 4 == 0 { 0.5 } else { 0.0 },
            });
            answer(og_world_insert(
                e,
                id.as_f64().unwrap(),
                "og.Model".into(),
                Some(model.to_string()),
            ));
        }
        Run { e, drawn, fresh }
    }

    fn frame(&self) -> String {
        if self.fresh {
            crate::engine::engine(self.e)
                .unwrap()
                .render
                .borrow_mut()
                .kept_models
                .forget();
        }
        value(og_engine_frame(self.e, 1000.0 / 60.0));
        let eng = crate::engine::engine(self.e).unwrap();
        let rs = eng.render.borrow();
        let mut levels: Vec<_> = rs.lod_drawn.iter().map(|(&e, &l)| (e, l)).collect();
        levels.sort_unstable();
        let mut replaced: Vec<_> = rs.hlod.replaced.keys().copied().collect();
        replaced.sort_unstable();
        format!(
            "{}\n{:?}\n{levels:?}\n{replaced:?}",
            self.drawn.borrow(),
            rs.ids.instances
        )
    }

    fn copied(&self) -> (usize, bool) {
        let eng = crate::engine::engine(self.e).unwrap();
        let k = &eng.render.borrow().kept_models;
        (k.copied, k.copied_hlod)
    }

    fn find(&self, name: &str) -> f64 {
        let found = crate::host_impl::json(og_world_find(self.e, name.into()));
        serde_json::from_str::<Value>(&found)
            .unwrap()
            .as_f64()
            .unwrap_or_else(|| panic!("no entity {name}"))
    }

    fn set(&self, name: &str, comp: &str, v: Value) {
        answer(og_world_set(
            self.e,
            self.find(name),
            comp.into(),
            v.to_string(),
        ));
    }

    /// Looks at the town's middle from `back` metres off, 20 m up.
    fn look(&self, back: f32) {
        let camera = json!({
            "position": { "x": 36, "y": 20, "z": 36.0 + back },
            "quaternion": { "x": 0, "y": 0, "z": 0, "w": 1 },
            "fov": 50,
        });
        value(og_engine_write_resource(
            self.e,
            "og.Camera".into(),
            camera.to_string(),
        ));
    }
}

/// A change both engines make, named.
type Step = (&'static str, Box<dyn Fn(&Run)>);

#[test]
fn a_kept_frame_draws_what_a_fresh_one_does() {
    let (kept, fresh) = (Run::new(false), Run::new(true));
    let both = |f: &dyn Fn(&Run)| {
        f(&kept);
        f(&fresh);
    };
    // Per step: what changes, then frames compared one by one.
    let steps: Vec<Step> = vec![
        ("near", Box::new(|r| r.look(30.0))),
        (
            "a strider moved",
            Box::new(|r| {
                r.set(
                    "strider-5",
                    "og.Transform",
                    json!({ "position": { "x": 40, "y": 0, "z": -40 } }),
                )
            }),
        ),
        (
            "a strider tinted",
            Box::new(|r| {
                r.set(
                    "strider-9",
                    "og.Model",
                    json!({ "tint": { "r": 0.5, "g": 1, "b": 1 } }),
                )
            }),
        ),
        ("further", Box::new(|r| r.look(400.0))),
        ("far: the town's proxies", Box::new(|r| r.look(3000.0))),
        (
            "a far house hidden",
            Box::new(|r| r.set("house-2-2", "og.Model", json!({ "visible": false }))),
        ),
        (
            "a strider despawned",
            Box::new(|r| {
                answer(og_world_despawn(r.e, r.find("strider-3")));
            }),
        ),
        ("near again", Box::new(|r| r.look(30.0))),
    ];
    let mut copied_rows = 0;
    let mut copied_hlod = false;
    let mut leveled = false;
    for (label, change) in &steps {
        both(&|r| change(r));
        for f in 0..5 {
            let (a, b) = (kept.frame(), fresh.frame());
            assert!(
                a == b,
                "{label}, frame {f}: the kept frame drew\n{a}\n\nthe fresh one\n{b}"
            );
            let (rows, hlod) = kept.copied();
            copied_rows += rows;
            copied_hlod |= hlod;
            leveled |= {
                let eng = crate::engine::engine(kept.e).unwrap();
                let rs = eng.render.borrow();
                rs.scene
                    .objects
                    .instance_levels
                    .iter()
                    .any(|&l| l != u8::MAX)
            };
            assert_eq!(
                fresh.copied(),
                (0, false),
                "{label}: the fresh engine copied"
            );
        }
    }
    assert!(copied_rows > 100, "only {copied_rows} rows were copied");
    assert!(copied_hlod, "the proxies were never copied");
    assert!(leveled, "no instance had a level for the cluster path");
}
