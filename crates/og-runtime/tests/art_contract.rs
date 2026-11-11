//! The art contract (the IDL's `art` resources, docs/art.md §2.1) through
//! a look: project.ogt's `look` is the look GameDesc.look applies when it
//! names none, and a look applied inside a system writes the contract only
//! where the system declared it, so a system that declares the frame's
//! resources (every game before the contract) applies its looks as before.
//!
//! Ways this could fail: GameDesc.look left out applies no look although
//! project.ogt names one; the two naming different looks are both taken
//! silently; applying a look whose blocks include ArtPalette inside a
//! system that declares only og.Stylise is refused (kestrel-stair, lantern
//! vale and hollowbell apply looks that way, through presets that now carry
//! a contract); or the contract is skipped even where the system declares
//! it.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_load_look, og_render_apply_look, og_render_look, og_system_define_system, og_world_resource,
    set_hooks,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const FRAME_ONLY: u32 = 1;
const WITH_CONTRACT: u32 = 2;

const PROJECT: &str =
    "type project\nversion 1\n---\nname \"Art\"\nentry \"src/main.ts\"\nlook \"looks/a.ogt\"\n";

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-art-contract-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, text: &str| {
            let p = root.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        write("project.ogt", PROJECT);
        write(
            "assets/looks/a.ogt",
            "type look\nversion 1\n---\nname \"A\"\nStylise {\n  posterize 5\n}\nArtPalette {\n  maxColors 12\n  accents 1 3\n}\nArtLine {\n  width 2\n}\n",
        );
        write(
            "assets/looks/b.ogt",
            "type look\nversion 1\n---\nname \"B\"\nStylise {\n  posterize 3\n}\nArtPalette {\n  maxColors 7\n}\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

fn answer(text: String) -> Result<Value, String> {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    if v["ok"] == json!(true) {
        Ok(v["value"].clone())
    } else {
        Err(v["reason"].as_str().unwrap_or(&text).to_string())
    }
}

fn value(text: String) -> Value {
    answer(text).unwrap_or_else(|e| panic!("{e}"))
}

/// Reads the cooked files from disk; the system whose id is `acting`
/// applies the look `look` once and records the answer.
#[derive(Default)]
struct Apply {
    acting: Cell<u32>,
    look: Cell<f64>,
    answers: RefCell<Vec<Result<Value, String>>>,
}

struct Host(Rc<Apply>);

impl Hooks for Host {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == self.0.acting.get() {
            self.0.acting.set(0);
            let a = answer(og_runtime::host_impl::answer(og_render_apply_look(
                call.engine,
                self.0.look.get(),
            )));
            self.0.answers.borrow_mut().push(a);
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// og.render.look's name (an Option, not a Result, on the wire).
fn look_name(e: u32) -> Value {
    serde_json::from_str::<Value>(&og_render_look(e)).unwrap()["name"].clone()
}

fn field(e: u32, resource: &str, name: &str) -> Value {
    value(og_runtime::host_impl::answer(og_world_resource(
        e,
        resource.into(),
    )))[name]
        .clone()
}

/// An engine over the project, with GameDesc `desc` and the two systems.
fn engine(desc: Value) -> (Result<u32, String>, Rc<Apply>) {
    let root = project();
    let apply = Rc::new(Apply::default());
    set_hooks(Rc::new(Host(apply.clone())));
    og_host_begin_bundle();
    let stylise = json!({ "kind": "resource", "name": "og.Stylise" });
    let palette = json!({ "kind": "resource", "name": "og.ArtPalette" });
    let frame_only = og_system_define_system(
        "FrameOnly".into(),
        json!({ "reads": [], "writes": [stylise] }).to_string(),
        FRAME_ONLY,
    );
    let with_contract = og_system_define_system(
        "WithContract".into(),
        json!({ "reads": [], "writes": [stylise, palette] }).to_string(),
        WITH_CONTRACT,
    );
    let mut desc = desc;
    desc["seed"] = json!(1);
    desc["systems"] = json!([frame_only, with_contract]);
    let game = og_game_game(Some(desc.to_string()));
    let options = json!({ "root": root.to_str().unwrap(), "project": PROJECT }).to_string();
    let e = answer(og_engine_create(game, options)).map(|v| v.as_u64().unwrap() as u32);
    (e, apply)
}

#[test]
fn the_project_look_applies_and_a_system_writes_only_the_contract_it_declares() {
    let (e, apply) = engine(json!({}));
    let e = e.unwrap();
    value(og_engine_frame(e, 1000.0 / 60.0));
    // GameDesc.look left out: project.ogt's look, contract and all.
    assert_eq!(look_name(e), json!("A"));
    assert_eq!(field(e, "og.Stylise", "posterize"), json!(5));
    assert_eq!(field(e, "og.ArtPalette", "maxColors"), json!(12));
    assert_eq!(field(e, "og.ArtPalette", "accents"), json!([1, 3]));
    assert_eq!(field(e, "og.ArtLine", "width"), json!(2.0));

    let b = value(og_runtime::host_impl::answer(og_load_look(
        e,
        "looks/b.ogt".into(),
    )))
    .as_f64()
    .unwrap();
    apply.look.set(b);
    // A system that declares the frame's resources only: the look applies,
    // the contract stays the project's.
    apply.acting.set(FRAME_ONLY);
    value(og_engine_frame(e, 1000.0 / 60.0));
    let answers = apply.answers.borrow_mut().drain(..).collect::<Vec<_>>();
    assert_eq!(answers.len(), 1);
    assert!(answers[0].is_ok(), "{:?}", answers[0]);
    assert_eq!(field(e, "og.Stylise", "posterize"), json!(3));
    assert_eq!(field(e, "og.ArtPalette", "maxColors"), json!(12));
    assert_eq!(field(e, "og.ArtLine", "width"), json!(2.0));

    // One that declares ArtPalette too: the look replaces it whole.
    apply.acting.set(WITH_CONTRACT);
    value(og_engine_frame(e, 1000.0 / 60.0));
    let answers = apply.answers.borrow_mut().drain(..).collect::<Vec<_>>();
    assert!(answers[0].is_ok(), "{:?}", answers[0]);
    assert_eq!(field(e, "og.ArtPalette", "maxColors"), json!(7));
    assert_eq!(field(e, "og.ArtPalette", "accents"), json!([]));
    // ArtLine is undeclared there: left as it is.
    assert_eq!(field(e, "og.ArtLine", "width"), json!(2.0));
}

/// Records the frame's quantise palette (its colours, sRGB).
struct PaletteRecorder(Rc<RefCell<Option<Vec<[u8; 3]>>>>);

impl og_runtime::RenderBackend for PaletteRecorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .look
            .stylise
            .quantize
            .and_then(|q| q.palette)
            .and_then(|h| scene.palette(h))
            .map(|p| p.colors.clone());
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

/// A look that quantises (posterize and a dither) without naming a frame
/// palette takes the contract's ArtPalette.colors, so the frame keeps the
/// colours the cook put the art on. Old code quantised to the RGB cube
/// alone, with no palette.
#[test]
fn a_quantising_frame_takes_the_contracts_palette() {
    let project =
        "type project\nversion 1\n---\nname \"Art\"\nentry \"src/main.ts\"\nlook \"looks/c.ogt\"\n";
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-art-palette-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |path: &str, text: &str| {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    write("project.ogt", project);
    write(
        "assets/looks/c.ogt",
        "type look\nversion 1\n---\nname \"C\"\nStylise {\n  posterize 4\n  dither bayer2\n}\nArtPalette {\n  colors \"@palettes/dusk16\"\n  strict true\n}\n",
    );
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    // The palette as its file lists it, read here, not through the engine.
    let dusk: Vec<[u8; 3]> = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/looks/assets/palettes/dusk16.hex"),
    )
    .unwrap()
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty() && !l.starts_with(';'))
    .map(|l| [0, 2, 4].map(|i| u8::from_str_radix(&l[i..i + 2], 16).unwrap()))
    .collect();
    assert_eq!(dusk.len(), 16);
    set_hooks(Rc::new(Host(Rc::new(Apply::default()))));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options = json!({ "root": root.to_str().unwrap(), "project": project }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(None));
    og_runtime::set_render_backend(e, Some(Box::new(PaletteRecorder(seen.clone())))).unwrap();
    for _ in 0..10 {
        value(og_engine_frame(e, 1000.0 / 60.0));
        if seen.borrow().is_some() {
            break;
        }
    }
    assert_eq!(
        seen.borrow().as_deref(),
        Some(&dusk[..]),
        "the contract's palette"
    );
}
