//! HLOD at run time (docs/design.md §10.6): a scene's distant static
//! models draw as its cooked proxies, and as themselves again once near
//! or once one of them is no longer what the cook read.
//!
//! The town is 36 houses 12 m apart with 32 m leaves: four leaves of nine
//! under one root. The view is 1080 px tall at fov 50°, so the root's
//! half-metre budget is under a pixel past about 600 m.
//!
//! Ways this could fail, and the test that catches each:
//! - the proxies never draw, draw with their models (both at once), or
//!   draw somewhere else than the houses (the node's centre applied twice,
//!   the scene's root left out), or a near town draws proxies →
//!   `a_far_town_draws_its_proxies_where_its_houses_stand`;
//! - a house destroyed, moved or hidden still shows in its proxy, or one
//!   changed house returns the whole town to its models rather than its
//!   node → `a_changed_house_returns_its_node_to_its_models`;
//! - a cell not loaded draws nothing, draws at the world's zero, or keeps
//!   its proxies over its models once loaded →
//!   `a_cell_not_loaded_draws_its_proxies_at_the_cell`.

use og_core::glam::Vec3;
use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_take_outputs, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, og_load_scene, og_world_despawn, og_world_find, og_world_get,
    og_world_instantiate, og_world_set, set_hooks, set_render_backend,
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

/// The parts of a house.
const PARTS: usize = 2;

/// 6 × 6 houses over 72 m from the scene's zero.
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

/// A world whose one cell, 1000 m along x, loads the town within 50 m of
/// the player.
const WORLD: &str = "type scene
version 1
---
name \"World\"
entity \"player\" {
  Transform {
    position 0 0 0
  }
  StreamAnchor {
    reach 0
  }
}
entity \"district\" {
  scene \"scenes/town.ogt\"
  Transform {
    position 1000 0 0
  }
  Volume {
    size 80 20 80
  }
  StreamCell {
    load 50
  }
}
";

/// Two houses under one leaf whose pages, at 5 cm a texel, pass the mip
/// tail's 256 texels.
const PAIR: &str = "type scene
version 1
---
name \"Pair\"
hlod {
  size 32
  error 0.05
  texels 1024
}
entity \"a\" {
  model \"models/house.ogt\"
  static true
  Transform {
    position 3 0 3
  }
}
entity \"b\" {
  model \"models/house.ogt\"
  static true
  Transform {
    position 15 0 3
  }
}
";

/// og.Model's fields these scenes set, as `og cook` packs them.
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

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-hlod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (id, text) in [
            ("models/house.ogt", HOUSE.to_string()),
            ("scenes/town.ogt", town()),
            ("scenes/world.ogt", WORLD.to_string()),
            ("scenes/pair.ogt", PAIR.to_string()),
        ] {
            let p = root.join("assets").join(id);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
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

/// What the last frame drew: how many instances, and the box around them.
#[derive(Clone, Debug, Default)]
struct Frame {
    instances: usize,
    min: Vec3,
    max: Vec3,
    /// Each instance drawn in part: how much, and whether fading out.
    fades: Vec<(f32, bool)>,
    /// Of the textures the instances draw, the widest one's width as held
    /// and as its whole chain (a streamed texture holds its tail).
    widest: (u32, u32),
}

struct Recorder(Rc<RefCell<Frame>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (1920, 1080)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let mut f = Frame {
            instances: scene.objects.instances.len(),
            min: Vec3::MAX,
            max: Vec3::MIN,
            fades: Vec::new(),
            widest: (0, 0),
        };
        for t in scene
            .objects
            .instances
            .iter()
            .flat_map(|i| scene.material_textures(i.material))
        {
            let size = scene.texture_size(t).map_or(0, |s| s.0);
            if size > f.widest.1 {
                f.widest = (scene.texture(t).map_or(0, |d| d.width), size);
            }
        }
        for i in &scene.objects.instances {
            let Some((a, b)) = scene.mesh(i.mesh).and_then(|m| m.bounds()) else {
                continue;
            };
            for c in 0..8 {
                let p = Vec3::new(
                    if c & 1 == 0 { a.x } else { b.x },
                    if c & 2 == 0 { a.y } else { b.y },
                    if c & 4 == 0 { a.z } else { b.z },
                );
                let p = i.transform.transform_point3(p);
                f.min = f.min.min(p);
                f.max = f.max.max(p);
            }
        }
        f.fades = scene
            .objects
            .instances
            .iter()
            .filter(|i| i.fx.reveal < 1.0)
            .map(|i| (i.fx.reveal, i.fx.fade_out))
            .collect();
        *self.0.borrow_mut() = f;
        Ok(())
    }
    fn texture_uploaded(&self, _: &RenderScene, _: og_render::TextureHandle) -> bool {
        true
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

/// An engine with `scene` instantiated, and what its frames draw.
fn engine(scene: &str) -> (u32, Rc<RefCell<Frame>>) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let frame = Rc::new(RefCell::new(Frame::default()));
    set_render_backend(e, Some(Box::new(Recorder(frame.clone())))).unwrap();
    let scene = value(og_runtime::host_impl::answer(og_load_scene(
        e,
        scene.into(),
    )));
    value(og_runtime::host_impl::answer(og_world_instantiate(
        e,
        scene.as_f64().unwrap(),
        None,
    )));
    (e, frame)
}

/// Looks at `target` from `back` metres along +z of it, 20 m up.
fn look(e: u32, target: Vec3, back: f32) {
    let camera = json!({
        "position": { "x": target.x, "y": target.y + 20.0, "z": target.z + back },
        "quaternion": { "x": 0, "y": 0, "z": 0, "w": 1 },
        "fov": 50,
    });
    value(og_engine_write_resource(
        e,
        "og.Camera".into(),
        camera.to_string(),
    ));
}

fn step(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

/// Frames past a cross-fade's 0.4 s, then the frame drawn and
/// dump_render's `hlod` of it: scenes, proxies, unloaded, replaced.
fn drawn(e: u32, frame: &Rc<RefCell<Frame>>) -> (Frame, [u64; 4]) {
    settle(e, frame, 30)
}

/// `n` frames, then the frame drawn and dump_render's `hlod` of it.
fn settle(e: u32, frame: &Rc<RefCell<Frame>>, n: u32) -> (Frame, [u64; 4]) {
    step(e, n);
    let _ = og_engine_take_outputs(e);
    // The command runs as the next tick starts, on the frame before it;
    // nothing changes between the two.
    let r: Value =
        serde_json::from_str(&og_engine_probe_commands(e, "0:dump_render".into())).unwrap();
    assert_eq!(r["scheduled"], json!(1), "{r}");
    step(e, 1);
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(e)).unwrap();
    let dump = outputs
        .as_array()
        .and_then(|o| o.iter().rev().find(|o| o["kind"] == "json"))
        .unwrap_or_else(|| panic!("no dump: {outputs}"));
    let dump: Value = serde_json::from_str(dump["data"].as_str().unwrap()).unwrap();
    let h = &dump["hlod"];
    let n = |k: &str| {
        h[k].as_u64()
            .unwrap_or_else(|| panic!("no hlod.{k}: {dump}"))
    };
    (
        frame.borrow().clone(),
        [n("scenes"), n("proxies"), n("unloaded"), n("replaced")],
    )
}

fn find(e: u32, name: &str) -> f64 {
    serde_json::from_str::<Value>(&og_runtime::host_impl::json(og_world_find(e, name.into())))
        .unwrap()
        .as_f64()
        .unwrap_or_else(|| panic!("no entity {name}"))
}

fn set(e: u32, name: &str, comp: &str, v: Value) {
    value(og_runtime::host_impl::answer(og_world_set(
        e,
        find(e, name),
        comp.into(),
        v.to_string(),
    )));
}

/// The middle of the town, in its scene.
const MIDDLE: Vec3 = Vec3::new(36.0, 0.0, 36.0);

/// Whether two boxes are the same to within `by` metres.
fn same_box(a: &Frame, b: &Frame, by: f32) -> bool {
    a.min.abs_diff_eq(b.min, by) && a.max.abs_diff_eq(b.max, by)
}

#[test]
fn a_far_town_draws_its_proxies_where_its_houses_stand() {
    let (e, frame) = engine("scenes/town.ogt");
    // Among the houses: every one draws itself.
    look(e, MIDDLE, 30.0);
    let (near, h) = drawn(e, &frame);
    assert_eq!(h, [1, 0, 0, 0], "near: {h:?}");
    assert_eq!(near.instances, 36 * PARTS, "near: {near:?}");
    // Far: the root's proxy alone, no house, where the houses stand to
    // within its error.
    look(e, MIDDLE, 3000.0);
    let (far, h) = drawn(e, &frame);
    assert_eq!(h, [1, 1, 0, 36], "far: {h:?}");
    assert!(far.instances <= 2, "far: {far:?}");
    assert!(same_box(&far, &near, 0.5), "{far:?} against {near:?}");
    // And back.
    look(e, MIDDLE, 30.0);
    let (again, h) = drawn(e, &frame);
    assert_eq!((h, again.instances), ([1, 0, 0, 0], 36 * PARTS));
}

#[test]
fn a_turning_node_cross_fades_its_proxy_with_its_houses() {
    let (e, frame) = engine("scenes/town.ogt");
    look(e, MIDDLE, 30.0);
    assert_eq!(drawn(e, &frame).1, [1, 0, 0, 0]);
    // Turned to the proxy: a few frames in, both draw, the proxy fading in
    // and every house part fading out by as much, so each pixel is the
    // one or the other; no house counts as replaced yet.
    look(e, MIDDLE, 3000.0);
    let (mid, h) = settle(e, &frame, 6);
    assert_eq!(h, [1, 1, 0, 0], "mid-fade: {h:?}");
    assert_eq!(mid.instances, 36 * PARTS + 1, "{mid:?}");
    let ins: Vec<f32> = mid.fades.iter().filter(|f| !f.1).map(|f| f.0).collect();
    let outs: Vec<f32> = mid.fades.iter().filter(|f| f.1).map(|f| f.0).collect();
    assert_eq!((ins.len(), outs.len()), (1, 36 * PARTS), "{:?}", mid.fades);
    assert!(ins[0] > 0.05 && ins[0] < 0.95, "{ins:?}");
    assert!(
        outs.iter().all(|o| (o + ins[0] - 1.0).abs() < 1e-5),
        "{ins:?} {outs:?}"
    );
    // Turned back mid-fade: reversed from where it was, never a pop.
    look(e, MIDDLE, 30.0);
    let (back, h) = settle(e, &frame, 1);
    assert_eq!(h, [1, 1, 0, 0], "reversed: {h:?}");
    let proxy = back
        .fades
        .iter()
        .find(|f| f.1)
        .expect("the proxy fades out");
    assert!((proxy.0 - ins[0]).abs() < 0.2, "{proxy:?} from {ins:?}");
    // Settled: the houses alone again, whole.
    let (done, h) = drawn(e, &frame);
    assert_eq!(
        (h, done.instances, done.fades.len()),
        ([1, 0, 0, 0], 36 * PARTS, 0)
    );
    // And to the proxy, to the end: the proxy alone, whole.
    look(e, MIDDLE, 3000.0);
    let (far, h) = drawn(e, &frame);
    assert_eq!((h, far.fades.len()), ([1, 1, 0, 36], 0));
}

#[test]
fn a_changed_house_returns_its_node_to_its_models() {
    let (e, frame) = engine("scenes/town.ogt");
    look(e, MIDDLE, 3000.0);
    assert_eq!(drawn(e, &frame).1, [1, 1, 0, 36]);
    // Hidden: its leaf's eight others draw themselves, the three other
    // leaves their proxies.
    set(e, "house-0-0", "og.Model", json!({ "visible": false }));
    let (hidden, h) = drawn(e, &frame);
    assert_eq!(h, [1, 3, 0, 27], "hidden: {h:?}");
    assert!(
        hidden.instances >= 8 * PARTS + 3 && hidden.instances <= 8 * PARTS + 6,
        "hidden: {hidden:?}"
    );
    set(e, "house-0-0", "og.Model", json!({ "visible": true }));
    assert_eq!(drawn(e, &frame).1, [1, 1, 0, 36], "shown again");
    // Moved 5 m up: it is drawn where it is.
    let at = value(og_runtime::host_impl::answer(og_world_get(
        e,
        find(e, "house-5-5"),
        "og.Transform".into(),
    )));
    set(
        e,
        "house-5-5",
        "og.Transform",
        json!({ "position": { "x": at["position"]["x"], "y": 5, "z": at["position"]["z"] } }),
    );
    let (moved, h) = drawn(e, &frame);
    assert_eq!(h, [1, 3, 0, 27], "moved: {h:?}");
    assert!(moved.max.y > 11.5, "the moved house's roof: {moved:?}");
    // Destroyed: gone from the frame, for good.
    value(og_runtime::host_impl::answer(og_world_despawn(
        e,
        find(e, "house-5-5"),
    )));
    let (gone, h) = drawn(e, &frame);
    assert_eq!(h, [1, 3, 0, 27], "destroyed: {h:?}");
    assert!(gone.max.y < 7.5, "{gone:?}");
    assert!(
        gone.instances >= 8 * PARTS + 3 && gone.instances <= 8 * PARTS + 6,
        "destroyed: {gone:?}"
    );
}

#[test]
fn a_cell_not_loaded_draws_its_proxies_at_the_cell() {
    let (e, frame) = engine("scenes/world.ogt");
    let cell = Vec3::new(1000.0, 0.0, 0.0);
    let loaded = |e: u32| {
        value(og_runtime::host_impl::answer(og_world_get(
            e,
            find(e, "district"),
            "og.StreamCell".into(),
        )))["loaded"]
            == json!(true)
    };
    // The player is a kilometre off: the town is not in the world, and
    // its root's proxy draws at the cell.
    look(e, cell + MIDDLE, 3000.0);
    let (far, h) = drawn(e, &frame);
    assert!(!loaded(e));
    assert_eq!(h, [1, 1, 1, 0], "not loaded, far: {h:?}");
    // The houses' walls span 66 m from the cell, their eaves 1.2 m more.
    assert!(
        far.min.abs_diff_eq(cell, 1.5)
            && far.max.abs_diff_eq(cell + Vec3::new(66.0, 7.0, 66.0), 1.5),
        "{far:?}"
    );
    // Nearer than the leaves' pixel and still not loaded: the leaves are
    // the finest there is.
    look(e, cell + MIDDLE, 60.0);
    let (near, h) = drawn(e, &frame);
    assert_eq!(h, [1, 4, 4, 0], "not loaded, near: {h:?}");
    assert!(same_box(&near, &far, 0.5), "{near:?} against {far:?}");
    // The player walks in: the houses are in the world and draw themselves.
    set(
        e,
        "player",
        "og.Transform",
        json!({ "position": { "x": 1036, "y": 0, "z": 36 } }),
    );
    let (inside, h) = drawn(e, &frame);
    assert!(loaded(e));
    assert_eq!(h, [1, 0, 0, 0], "loaded, near: {h:?}");
    assert_eq!(inside.instances, 36 * PARTS);
    assert!(same_box(&inside, &far, 0.5), "{inside:?} against {far:?}");
    // Loaded and seen from afar: the proxy stands in for the houses.
    look(e, cell + MIDDLE, 3000.0);
    assert_eq!(drawn(e, &frame).1, [1, 1, 0, 36], "loaded, far");
    // The player leaves: the proxy alone again.
    set(
        e,
        "player",
        "og.Transform",
        json!({ "position": { "x": 0, "y": 0, "z": 0 } }),
    );
    let (left, h) = drawn(e, &frame);
    assert!(!loaded(e));
    assert_eq!(h, [1, 1, 1, 0], "left: {h:?}");
    assert!(same_box(&left, &far, 0.01), "{left:?} against {far:?}");
}

#[test]
fn a_proxy_loads_its_pages_tails_and_streams_their_chains() {
    let (e, frame) = engine("scenes/pair.ogt");
    // Far, small on screen: the proxy draws its pages' tails, each sized
    // as its whole chain.
    look(e, Vec3::new(9.0, 0.0, 3.0), 3000.0);
    let (far, h) = drawn(e, &frame);
    assert_eq!(h, [1, 1, 0, 2], "{h:?}");
    let (held, whole) = far.widest;
    assert!(whole > 256 && held <= 256, "{:?}", far.widest);
    // Every streamed texture whole (the probe's resident mode): the
    // proxy's pages are among them.
    let r: Value =
        serde_json::from_str(&og_engine_probe_commands(e, "0:mip_stream:resident".into())).unwrap();
    assert_eq!(r["scheduled"], json!(1), "{r}");
    let (resident, _) = drawn(e, &frame);
    assert_eq!(resident.widest, (whole, whole), "{resident:?}");
}
