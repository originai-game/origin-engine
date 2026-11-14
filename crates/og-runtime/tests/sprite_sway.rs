//! og.Sprite.sway: a sprite whose texture was cooked with wind parts
//! (`texture { sway clumps }`) is drawn as those parts, moved by og.Wind.
//!
//! A project with one pixel-art tree (a trunk under five masses of leaves,
//! 48 x 54 texels) is cooked once with `sway clumps`; a renderer that
//! draws nothing records each frame's world sprites.
//!
//! Ways this could fail: the cook writes no `<id>#sway` or the runtime
//! never loads it (the sprite stays whole in the wind); the parts draw in
//! still air or at sway 0 (the tree no longer the texture it was given);
//! the wood moves; the leaves move off the texel grid or upwind; the
//! parts stand still while the tick goes on; a sprite of a texture without
//! parts disappears instead of drawing whole.

use glam::Vec2;
use og_render::{RenderScene, Sprite};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_load_texture, og_world_insert,
    og_world_set, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The tree: a brown trunk under five round masses of leaves, each
/// brightest at its middle, with a dark brown rim.
fn tree() -> og_render::RgbaImage {
    let (w, h) = (48u32, 54u32);
    let blobs = [
        (10.0f32, 16.0f32),
        (38.0, 16.0),
        (24.0, 5.0),
        (24.0, 27.0),
        (24.0, 16.0),
    ];
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let o = ((y * w + x) * 4) as usize;
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let near = blobs
                .iter()
                .map(|(bx, by)| Vec2::new(fx, fy).distance(Vec2::new(*bx, *by)))
                .fold(f32::MAX, f32::min);
            if near < 8.5 {
                let c = if near > 7.5 {
                    [50, 30, 12, 255]
                } else {
                    [30, (220.0 - 16.0 * near) as u8, 40, 255]
                };
                pixels[o..o + 4].copy_from_slice(&c);
            } else if (21..27).contains(&x) && (30..54).contains(&y) {
                pixels[o..o + 4].copy_from_slice(&[110, 70, 30, 255]);
            }
        }
    }
    og_render::RgbaImage {
        width: w,
        height: h,
        pixels,
    }
}

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-sprite-sway-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let png = og_runtime::png(&tree());
        write("trees/oak.png", &png);
        write(
            "trees/oak.png.ogt",
            b"type import\nversion 1\n---\ntexture {\n  kind pixel\n  sway clumps\n}\n",
        );
        // The same picture without parts.
        write("trees/plain.png", &png);
        write(
            "trees/plain.png.ogt",
            b"type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n",
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

struct Recorder(Rc<RefCell<Vec<Sprite>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.sprite.sprites.clone();
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

/// A sprite's bottom-left corner in the world.
fn corner(s: &Sprite) -> Vec2 {
    s.position.truncate() - s.anchor * s.size
}

#[test]
fn a_tree_cooked_with_wind_parts_sways_in_the_worlds_wind() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let frame = || {
        value(og_engine_frame(engine, 1000.0 / 60.0));
        seen.borrow().clone()
    };
    let load = |path: &str| {
        value(og_runtime::host_impl::answer(og_load_texture(
            engine,
            path.into(),
        )))
    };
    // 3 by 3.375 world units: 16 texels a unit, standing on its foot at x 2.
    let sprite = |texture: &Value, sway: f32| {
        json!({ "texture": texture, "width": 3, "height": 3.375,
                "anchor": { "x": 0.5, "y": 0 }, "sway": sway })
    };
    let oak = load("trees/oak.png");
    let e = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(json!({ "position": { "x": 2, "y": 0, "z": 0 } }).to_string()),
    )))
    .as_f64()
    .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        e,
        "og.Sprite".into(),
        Some(sprite(&oak, 1.0).to_string()),
    )));

    // Still air: the sprite as it was given, whole.
    let still = frame();
    assert_eq!(still.len(), 1, "{still:?}");
    let whole = still[0];
    assert_eq!(whole.size, Vec2::new(3.0, 3.375));
    assert_eq!(corner(&whole), Vec2::new(0.5, 0.0));

    // A fresh breeze along +x: the wood, the crown's dark copy, then five
    // clumps.
    value(og_engine_write_resource(
        engine,
        "og.Wind".into(),
        json!({ "direction": { "x": 1, "y": 0, "z": 0 }, "speed": 8 }).to_string(),
    ));
    let texel = 1.0 / 16.0;
    let mut poses = std::collections::BTreeSet::new();
    let mut wood = None;
    for _ in 0..120 {
        let parts = frame();
        assert_eq!(parts.len(), 7, "{parts:?}");
        assert!(parts.iter().all(|p| p.texture != whole.texture));
        assert!(parts.iter().all(|p| p.texture == parts[0].texture));
        // The wood stands on the tree's foot, up to the crown (19 rows of
        // trunk show under it, and the row of the rim on the trunk is its
        // colour and stays on it), and never moves.
        let w = (corner(&parts[0]), parts[0].size);
        assert_eq!((w.0.y, w.1.y), (0.0, 20.0 * texel), "{w:?}");
        assert_eq!(*wood.get_or_insert(w), w);
        let mut pose = Vec::new();
        for p in &parts[1..] {
            // Within the tree's rectangle moved downwind, on whole texels.
            let at = (corner(p) - corner(&whole)) / texel;
            assert!((at - at.round()).length() < 1e-2, "{at}");
            assert!(at.x >= -0.01 && at.y >= -0.01, "{at}");
            pose.push((at.x.round() as i32, at.y.round() as i32));
        }
        poses.insert(pose);
    }
    // Two seconds of gusts at ten poses a second: the clumps keep moving,
    // each pose held for more than a frame.
    assert!((4..=21).contains(&poses.len()), "{}", poses.len());
    // The leftmost clump rests 2 texels in from the tree's left edge.
    let lean = poses
        .iter()
        .map(|p| p.iter().map(|c| c.0).min().unwrap() - 2)
        .max()
        .unwrap();
    // 8 m/s leans a crown 0.05 a unit of height, with the gusts up to
    // twice that (less what its limit of 0.07 takes) and the loop 0.6
    // more: of 54 texels, 2.3 to 6 at the most.
    assert!((2..=6).contains(&lean), "{lean}");

    // No sway: whole again, in the same wind. Parts or none, the sprite draws.
    value(og_runtime::host_impl::answer(og_world_set(
        engine,
        e,
        "og.Sprite".into(),
        sprite(&oak, 0.0).to_string(),
    )));
    assert_eq!(frame(), vec![whole]);
    let plain = load("trees/plain.png");
    value(og_runtime::host_impl::answer(og_world_set(
        engine,
        e,
        "og.Sprite".into(),
        sprite(&plain, 1.0).to_string(),
    )));
    let drawn = frame();
    assert_eq!(drawn.len(), 1, "{drawn:?}");
    assert!(drawn[0].texture.is_some() && drawn[0].texture != whole.texture);
    assert_eq!(corner(&drawn[0]), Vec2::new(0.5, 0.0));
}
