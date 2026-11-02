//! og.CrowdAnimation through the runtime (R9 GPU crowds): models that play
//! a clip on the GPU are drawn as skinned instances posed by one baked clip
//! shared by all of them, each at its own frame, with the frame before's
//! for TAA, and no og-anim pose.
//!
//! Ways this could fail: a clip baked per entity (the scene's baked clips
//! grow with the crowd); every member at the same frame (`offset` unread);
//! the frame never advancing with the world's time; the previous frame
//! not the one drawn before; a crowd member posed by og-anim instead.

use og_render::RenderScene;
use og_runtime::{
    Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_animation, og_load_gltf, og_world_insert, og_world_instantiate,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-crowd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let walker = std::fs::read(repo.join("templates/game/assets/models/walker.glb")).unwrap();
        let p = root.join("assets/models/walker.glb");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, walker).unwrap();
        let options = og_cook::CookOptions {
            threads: 1,
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &og_runtime::Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// What each frame drew: the baked clips' (joints, frames) and every
/// skinned instance's baked pose (clip, frame, previous clip, frame), or
/// `None` for one posed by a palette.
#[derive(Default)]
struct Seen {
    clips: Vec<(u32, usize)>,
    poses: Vec<Option<(u32, f32, u32, f32)>>,
    palettes: usize,
}

struct Recorder(Rc<RefCell<Vec<Seen>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().push(Seen {
            clips: scene
                .baked_clips
                .iter()
                .map(|c| (c.joints, c.frame_count()))
                .collect(),
            poses: scene
                .skinned_instances
                .iter()
                .map(|i| {
                    i.baked
                        .map(|b| (b.clip, b.frame, b.previous_clip, b.previous_frame))
                })
                .collect(),
            palettes: scene.joint_palettes.len(),
        });
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

#[test]
fn a_crowd_shares_one_baked_clip_at_its_own_frames() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let anim = value(og_load_animation(engine, "models/walker.glb".into()));
    let clip = anim["clips"].as_array().unwrap()[0]["clip"].clone();
    let asset = value(og_load_gltf(engine, "models/walker.glb".into(), None));
    const MEMBERS: usize = 40;
    for k in 0..MEMBERS {
        let e = value(og_world_instantiate(engine, asset.as_f64().unwrap(), None))
            .as_f64()
            .unwrap();
        let crowd = json!({
            "skeleton": anim["skeleton"],
            "clip": clip,
            "offset": k as f64 * 0.137,
        });
        value(og_world_insert(
            engine,
            e,
            "og.CrowdAnimation".into(),
            Some(crowd.to_string()),
        ));
    }
    for _ in 0..4 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
    let seen = seen.borrow();
    let (before, last) = (&seen[seen.len() - 2], &seen[seen.len() - 1]);
    // One clip, shared, whatever the crowd's size.
    assert_eq!(last.clips.len(), 1, "{:?}", last.clips);
    let (joints, frames) = last.clips[0];
    println!(
        "walker's clip: {joints} joints, {frames} frames, {} bytes",
        joints as usize * frames * 64
    );
    assert!(joints > 0 && frames >= 2);
    // Every member drawn baked (no og-anim palette), at its own frame.
    assert_eq!(last.palettes, 0);
    let poses: Vec<_> = last.poses.iter().map(|p| p.expect("baked")).collect();
    assert!(poses.len() >= MEMBERS, "{} instances", poses.len());
    let mut distinct: Vec<u32> = poses.iter().map(|p| p.1.to_bits()).collect();
    distinct.sort();
    distinct.dedup();
    assert!(
        distinct.len() > MEMBERS / 2,
        "{} distinct frames",
        distinct.len()
    );
    // The clip plays: each frame's previous frame is the frame drawn
    // before, and they differ.
    let then: Vec<_> = before.poses.iter().map(|p| p.unwrap().1).collect();
    assert_eq!(then.len(), poses.len());
    let mut moved = 0;
    for (now, was) in poses.iter().zip(&then) {
        assert_eq!(now.3, *was);
        assert_eq!(now.2, now.0);
        moved += usize::from(now.1 != *was);
    }
    assert!(moved > MEMBERS / 2, "{moved} members played on");
}
