//! A model's clips seen from game code: `GltfInfo.animations` lists the
//! clip names of the model's `#anim` blob (FINDINGS #3 of
//! `examples/lantern-vale`: it was always empty), and an animated `.vox`
//! (MagicaVoxel frames cooked into mesh-swap clips) plays on an
//! og.AnimationPlayer: one frame's primitive is drawn at a time, as a plain
//! instance (a frame that never moves has no joint: no palette matrix, no
//! skinning), the frames folded away hidden by the clip's visibility
//! channels and not drawn at all (before the visibility track every frame
//! was drawn, the folded ones through a joint scaled to 0: three draws a
//! tick, now one; before frames without motion lost their joints, the one
//! was a skinned draw through a palette of three matrices).
//!
//! A project with a three-frame `.vox` (`voxFps 4`, two `voxClip`s), a
//! static `.vox` and the template walker is cooked once; a renderer that
//! draws nothing records each frame's palettes and skinned instances.
//!
//! Ways this could fail: gltfInfo answers no names (or the static model's
//! load is refused for its missing `#anim`); a hidden frame still drawn
//! (two frames or more a tick), or the wrong one shown; a frame drawn at
//! its rest place but scaled away; a `.vox` drawn without a skeleton drawing
//! the frames after 0 (they are hidden at rest: the model blob's node rows
//! carry `KHR_node_visibility`, which the static draw honours).

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_animation, og_load_gltf, og_load_gltf_info, og_world_insert,
    og_world_instantiate, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(body);
}

/// A `.vox` of solid models of these sizes, no scene graph (several:
/// MagicaVoxel 0.98's animation, model `i` frame `i`).
fn vox(sizes: &[[u32; 3]]) -> Vec<u8> {
    let mut children = Vec::new();
    for (i, size) in sizes.iter().enumerate() {
        let body: Vec<u8> = size.iter().flat_map(|s| s.to_le_bytes()).collect();
        chunk(&mut children, b"SIZE", &body);
        let mut voxels = Vec::new();
        for z in 0..size[2] as u8 {
            for y in 0..size[1] as u8 {
                for x in 0..size[0] as u8 {
                    voxels.extend_from_slice(&[x, y, z, 1 + i as u8]);
                }
            }
        }
        let mut xyzi = ((voxels.len() / 4) as u32).to_le_bytes().to_vec();
        xyzi.extend_from_slice(&voxels);
        chunk(&mut children, b"XYZI", &xyzi);
    }
    let mut out = b"VOX ".to_vec();
    out.extend_from_slice(&150u32.to_le_bytes());
    out.extend_from_slice(b"MAIN");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(children.len() as u32).to_le_bytes());
    out.extend_from_slice(&children);
    out
}

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-model-clips-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        write("models/blob.vox", &vox(&[[2, 2, 2], [2, 2, 3], [2, 2, 4]]));
        write(
            "models/blob.vox.ogt",
            b"type import\nversion 1\n---\nmodel {\n  voxFps 4\n  voxClip idle 0 0\n  voxClip grow 0 2\n}\n",
        );
        write("models/crate.vox", &vox(&[[3, 3, 3]]));
        let walker = std::fs::read(repo().join("templates/game/assets/models/walker.glb")).unwrap();
        write("models/walker.glb", &walker);
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

/// What the last frame's scene drew: per palette, each joint's x scale;
/// and the height of each skinned and each plain instance's mesh.
#[derive(Default)]
struct Seen {
    palettes: Vec<Vec<f32>>,
    skinned: Vec<f32>,
    instances: Vec<f32>,
}

/// The height of `mesh`'s vertices.
fn height(scene: &RenderScene, mesh: og_render::MeshHandle) -> f32 {
    let ys = scene
        .mesh(mesh)
        .unwrap()
        .vertices
        .iter()
        .map(|v| v.position[1]);
    let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
    hi - lo
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
        s.palettes = scene
            .joint_palettes
            .iter()
            .map(|p| p.matrices.iter().map(|m| m.x_axis.length()).collect())
            .collect();
        s.skinned = scene
            .skinned_instances
            .iter()
            .map(|i| height(scene, i.mesh))
            .collect();
        s.instances = scene
            .instances
            .iter()
            .map(|i| height(scene, i.mesh))
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
    value(og_engine_frame(engine, 1000.0 / 60.0));
    (engine, seen)
}

fn names(engine: u32, path: &str) -> Vec<String> {
    let asset = value(og_load_gltf(engine, path.into(), None));
    let info = value(og_load_gltf_info(engine, asset.as_f64().unwrap()));
    serde_json::from_value(info["animations"].clone()).unwrap()
}

#[test]
fn gltf_info_lists_the_clips_of_the_models_anim_blob() {
    let (engine, _) = engine();
    let walker = names(engine, "models/walker.glb");
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let clips: Vec<String> = anim["clips"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    assert!(!walker.is_empty(), "the walker has clips");
    assert_eq!(
        walker, clips,
        "the names og.load.animation answers, in order"
    );
    assert_eq!(names(engine, "models/blob.vox"), vec!["idle", "grow"]);
    assert!(
        names(engine, "models/crate.vox").is_empty(),
        "a static model: no #anim, no clips"
    );
}

#[test]
fn an_animated_vox_plays_one_frame_at_a_time() {
    let (engine, seen) = engine();
    let anim = value(og_load_animation(engine, "models/blob.vox".into()));
    let grow = anim["clips"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "grow")
        .unwrap();
    assert_eq!(grow["duration"], json!(0.75));
    let asset = value(og_load_gltf(engine, "models/blob.vox".into(), None));
    let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.Skeleton".into(),
        Some(json!({ "asset": anim["skeleton"] }).to_string()),
    ));
    value(og_world_insert(
        engine,
        e,
        "og.AnimationPlayer".into(),
        Some(json!({ "clip": grow["clip"], "loop": true }).to_string()),
    ));
    // 0.25 s a frame at 60 ticks a second: each frame's middle.
    let mut shown = Vec::new();
    let mut tick = 0;
    for until in [7, 22, 37, 52] {
        while tick < until {
            value(og_engine_frame(engine, 1000.0 / 60.0));
            tick += 1;
        }
        let s = seen.borrow();
        assert!(
            s.palettes.iter().all(|p| p.is_empty()),
            "no joint: {:?}",
            s.palettes
        );
        assert!(s.skinned.is_empty(), "nothing skinned");
        shown.push(s.instances.clone());
    }
    // Frame k is 2 + k voxels (metres) tall.
    assert_eq!(
        shown,
        vec![vec![2.0], vec![3.0], vec![4.0], vec![2.0]],
        "one plain draw a tick: frame 0, 1, 2, then frame 0 again (looped)"
    );
}

#[test]
fn a_vox_without_a_skeleton_draws_frame_0_only() {
    let (engine, seen) = engine();
    for path in ["models/blob.vox", "models/crate.vox"] {
        let asset = value(og_load_gltf(engine, path.into(), None));
        value(og_world_instantiate(engine, asset.as_f64().unwrap(), None));
    }
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let mut drawn = seen.borrow().instances.clone();
    drawn.sort_by(f32::total_cmp);
    // Frame 0 of the animated one (2 tall) and the static crate (3): the
    // frames after 0 (3 and 4 tall), hidden at rest, are not drawn.
    assert_eq!(drawn, vec![2.0, 3.0], "the plain draws");
    assert!(seen.borrow().skinned.is_empty());
}

#[test]
fn a_mirrored_clip_is_registered_once_and_plays() {
    let (engine, _) = engine();
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let walk = anim["clips"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "walk")
        .expect("the walker walks")
        .clone();
    let desc = json!({ "clip": walk["clip"], "skeleton": anim["skeleton"] }).to_string();
    let m = value(og_runtime::og_anim_mirror(engine, Some(desc.clone())));
    assert_ne!(m["clip"], walk["clip"]);
    assert_eq!(
        (&m["name"], &m["duration"], &m["markers"]),
        (&walk["name"], &walk["duration"], &walk["markers"])
    );
    assert_eq!(value(og_runtime::og_anim_mirror(engine, Some(desc))), m);
    let asset = value(og_load_gltf(engine, "models/walker.glb".into(), None));
    let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.Skeleton".into(),
        Some(json!({ "asset": anim["skeleton"] }).to_string()),
    ));
    value(og_world_insert(
        engine,
        e,
        "og.AnimationPlayer".into(),
        Some(json!({ "clip": m["clip"] }).to_string()),
    ));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    // An unregistered clip is refused by name.
    let refused: Value = serde_json::from_str(&og_runtime::og_anim_mirror(
        engine,
        Some(json!({ "clip": 12345, "skeleton": anim["skeleton"] }).to_string()),
    ))
    .unwrap();
    assert_eq!(refused["ok"], json!(false));
    assert!(refused.to_string().contains("12345"), "{refused}");
}
