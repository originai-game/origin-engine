//! Two web findings (examples/WEB-FINDINGS.md, wave 13) at the runtime:
//!
//! - #4: `og.Mesh.castShadow` reaches the renderer: a mesh casts shadows
//!   only when its flag says so (off by default, as three.js), so The
//!   Kestrel Stair's crypt lamp, a glass box around its own point light,
//!   no longer shadows the crypt from that light. Broken once by casting
//!   every mesh again: the default mesh's instance casts.
//! - #3: GameDesc.look's antialias is the renderer's even when the look's
//!   assets cannot load before the renderer is attached (the web host
//!   decodes textures only then), so Lantern Vale's TAA is TAA on WebGL2
//!   too, not the tier's 4x MSAA. Broken once by loading the whole look at
//!   boot again: no antialias reaches the render options.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_box_geometry, og_render_standard_material, og_world_insert,
    og_world_spawn, render_options, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project whose look asks for TAA and names a palette, cooked once per
/// test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-web-findings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        write(
            "palettes/dusk.hex",
            &std::fs::read(repo().join("packages/engine/tests/fixtures/look/assets/palettes/dusk.hex"))
                .unwrap(),
        );
        write(
            "looks/dusk.ogt",
            b"type look\nversion 1\n---\nname \"Dusk\"\nantialias taa\nStylise {\n  posterize 6\n  palette \"palettes/dusk.hex\"\n}\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// Reads the cooked files from disk, except the ones whose path holds
/// `missing` (an asset the host cannot decode yet).
struct Disk {
    missing: &'static str,
}

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if !self.missing.is_empty() && path.contains(self.missing) {
            return Fetch::Failed(format!("{path}: not decoded yet"));
        }
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// Every instance's caster flag in the last frame.
struct Recorder(Rc<RefCell<Vec<bool>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.instances.iter().map(|i| i.cast_shadows).collect();
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

fn engine(game: &str, missing: &'static str) -> u32 {
    set_hooks(Rc::new(Disk { missing }));
    og_host_begin_bundle();
    let game = og_game_game(Some(game.into()));
    let options = json!({ "root": project().to_str().unwrap(), "deferStart": true }).to_string();
    value(og_engine_create(game, options)).as_u64().unwrap() as u32
}

#[test]
fn a_mesh_casts_shadows_only_when_its_flag_says_so() {
    let engine = engine(r#"{"seed":1}"#, "");
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    value(og_engine_frame(engine, 0.0));
    let geometry = value(og_render_box_geometry(engine, Some("{}".into())));
    let material = value(og_render_standard_material(engine, Some("{}".into())));
    for cast in [None, Some(true), Some(false)] {
        let e = value(og_world_spawn(engine, Some("{}".into())))
            .as_f64()
            .unwrap();
        let mut mesh = json!({ "geometry": geometry, "material": material });
        if let Some(c) = cast {
            mesh["castShadow"] = json!(c);
        }
        for (name, v) in [("og.Transform", json!({})), ("og.Mesh", mesh)] {
            value(og_world_insert(engine, e, name.into(), Some(v.to_string())));
        }
    }
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let mut casts = seen.borrow().clone();
    casts.sort();
    assert_eq!(casts, vec![false, false, true], "default off, then as set");
}

#[test]
fn the_looks_antialias_reaches_the_renderer_before_its_assets_can_load() {
    let game = r#"{"seed":1,"look":"looks/dusk.ogt"}"#;
    // Every asset readable: the look loads, its antialias is the renderer's.
    let ready = engine(game, "");
    assert_eq!(render_options(ready)["antialias"], json!("taa"));
    // Its palette not decodable yet (the web before its renderer is
    // attached): the look's own blob still says TAA.
    let early = engine(game, "dusk.hex");
    assert_eq!(render_options(early)["antialias"], json!("taa"));
}
