//! The 2D profile's runtime (`--features profile2d`: the 2D web modules)
//! leaves the terrain out with the rest of the 3D world: every og.terrain
//! call, on a terrain the full build loads, answers the refusal naming
//! RenderCapabilities.world3d, and so does a heightfield collider.
//!
//! Ways this could fail: a call still loads, samples or edits (the module
//! is still built); a call answers some other error (a missing file, a
//! dead handle) that does not say why; the collider is refused as a
//! missing registration instead of the profile, or not at all.
#![cfg(feature = "profile2d")]

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_terrain_bounds, og_terrain_edit, og_terrain_layer, og_terrain_load,
    og_terrain_sample, og_terrain_set_layer, og_world_insert, og_world_restore, og_world_snapshot,
    og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

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

/// A cooked 9 × 9 terrain with a collider, as a 3D game ships it.
fn project() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-profile2d-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |path: &str, bytes: &[u8]| {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    };
    let png = og_runtime::png(&og_render::RgbaImage {
        width: 9,
        height: 9,
        pixels: (0..81)
            .flat_map(|i| [i as u8, i as u8, i as u8, 255])
            .collect(),
    });
    write("terrain/hill.png", &png);
    write(
        "terrain/hill.ogt",
        b"type terrain\nversion 1\n---\nheightmap \"hill.png\"\nheight 0 4\nsize 16 16\n",
    );
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

fn wire(text: &str) -> Value {
    serde_json::from_str(text).expect("wire JSON")
}

/// The answer is the refusal that names the profile's missing 3D world.
fn refused(what: &str, text: String) {
    let v = wire(&text);
    assert_eq!(v["ok"], json!(false), "{what}: {text}");
    let reason = v["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("terrain is not built into this module")
            && reason.contains("RenderCapabilities.world3d is false"),
        "{what}: {text}"
    );
}

#[test]
fn the_terrain_and_its_height_fields_answer_the_profile() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = wire(&og_engine_create(game, options))["value"]
        .as_u64()
        .unwrap() as u32;
    wire(&og_engine_frame(engine, 1000.0 / 60.0));

    refused(
        "load",
        og_runtime::host_impl::answer(og_terrain_load(engine, "terrain/hill.ogt".into())),
    );
    // The calls on a terrain's data: refused for the profile, not as a
    // handle that is not alive.
    let data = 1.0;
    let at = json!({ "data": data, "x": 1, "z": 2 }).to_string();
    refused(
        "sample",
        og_runtime::host_impl::answer(og_terrain_sample(engine, Some(at))),
    );
    refused(
        "bounds",
        og_runtime::host_impl::answer(og_terrain_bounds(engine, data, None, None, None)),
    );
    let brush = json!({ "x": 0, "z": 0, "radius": 2, "strength": 1 }).to_string();
    refused(
        "edit",
        og_runtime::host_impl::answer(og_terrain_edit(engine, data, Some(brush))),
    );
    refused(
        "layer",
        og_runtime::host_impl::answer(og_terrain_layer(engine, data, 0)),
    );
    refused(
        "setLayer",
        og_runtime::host_impl::answer(og_terrain_set_layer(engine, data, 0, Some("{}".into()))),
    );

    // A heightfield collider naming a terrain's geometry.
    let e = wire(&og_runtime::host_impl::answer(og_world_spawn(engine, None)))["value"]
        .as_f64()
        .unwrap();
    let collider = json!({ "shape": "heightfield", "mesh": (0x7eu64 << 32) as f64 });
    let inserted = og_runtime::host_impl::answer(og_world_insert(
        engine,
        e,
        "og.Collider".into(),
        Some(collider.to_string()),
    ));
    assert_eq!(wire(&inserted)["ok"], json!(true), "{inserted}");
    let frame = og_engine_frame(engine, 1000.0 / 60.0);
    let log = og_engine_take_log(engine);
    let said = format!("{frame}\n{log}");
    assert!(
        said.contains("og.Collider.shape heightfield (a terrain's height field)")
            && said.contains("RenderCapabilities.world3d is false"),
        "{said}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The simulation bytes and tick, excluding the save's UI checkpoint.
fn snapshot_of(save: &str) -> (Vec<u8>, u64) {
    let deflated =
        og_core::base64::decode(save.strip_prefix("og-save:3:").expect("a save")).expect("base64");
    let payload = miniz_oxide::inflate::decompress_to_vec(&deflated).expect("a deflated save");
    assert_eq!(&payload[..4], b"OGSV");
    let tick = u64::from_le_bytes(payload[8..16].try_into().unwrap());
    (
        og_runtime::save_snapshot_bytes(save).expect("a world snapshot"),
        tick,
    )
}

/// A current save envelope with no presentation checkpoint.
fn save_of(snapshot: &[u8], tick: u64) -> String {
    let mut body = (snapshot.len() as u32).to_le_bytes().to_vec();
    body.extend_from_slice(snapshot);
    body.extend_from_slice(b"null");
    let mut payload = b"OGSV".to_vec();
    payload.extend_from_slice(&3u32.to_le_bytes());
    payload.extend_from_slice(&tick.to_le_bytes());
    payload.extend_from_slice(&og_core::hash::hash_bytes(&body).to_le_bytes());
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&body);
    format!(
        "og-save:3:{}",
        og_core::base64::encode(&miniz_oxide::deflate::compress_to_vec(&payload, 6))
    )
}

fn str_bytes(s: &str) -> Vec<u8> {
    let mut out = (s.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(s.as_bytes());
    out
}

/// A 2D world's save is the bytes the full build writes: it carries the
/// terrain edit log's state block (og.terrain, no edits), so a save moves
/// between this module and the native host unchanged. One that carries an
/// edit is a 3D game's and is refused with the profile's reason.
#[test]
fn a_save_carries_the_terrain_block_and_one_with_edits_is_refused() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = wire(&og_engine_create(game, options))["value"]
        .as_u64()
        .unwrap() as u32;
    wire(&og_engine_frame(engine, 1000.0 / 60.0));

    let saved = wire(&og_runtime::host_impl::answer(og_world_snapshot(
        engine, None,
    )));
    assert_eq!(saved["ok"], json!(true), "{saved}");
    let (snapshot, tick) = snapshot_of(saved["value"].as_str().unwrap());
    // The block as the full build's terrain.rs writes it with no edits:
    // str "og.terrain", then str of a u32 count 0.
    let mut empty = str_bytes("og.terrain");
    empty.extend_from_slice(&str_bytes("\0\0\0\0"));
    let at = snapshot
        .windows(empty.len())
        .position(|w| w == empty)
        .expect("the save carries the og.terrain state block with no edits");
    let restored = og_runtime::host_impl::answer(og_world_restore(
        engine,
        saved["value"].as_str().unwrap().into(),
        None,
    ));
    assert_eq!(wire(&restored)["ok"], json!(true), "{restored}");

    // The same save with one edit in the block.
    let mut edits = 1u32.to_le_bytes().to_vec();
    edits.extend(str_bytes("terrain/hill.ogt"));
    edits.extend(str_bytes(r#"{"x":0,"z":0,"radius":2,"strength":1}"#));
    let mut block = str_bytes("og.terrain");
    block.extend_from_slice(&(edits.len() as u32).to_le_bytes());
    block.extend(edits);
    let mut edited = snapshot[..at].to_vec();
    edited.extend(block);
    edited.extend_from_slice(&snapshot[at + empty.len()..]);
    let refused =
        og_runtime::host_impl::answer(og_world_restore(engine, save_of(&edited, tick), None));
    let v = wire(&refused);
    assert_eq!(v["ok"], json!(false), "{refused}");
    let reason = v["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("state block 'og.terrain'")
            && reason.contains("a save with 1 terrain edit(s)")
            && reason.contains("terrain is not built into this module"),
        "{refused}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
