//! og.load.data reads a content catalog's files from the content base
//! (guide loading §14): a version's manifest, then each file's bytes under
//! its digest, checked against the manifest's integrity.
//!
//! Ways this could fail: a file over a store value's 64 KiB is refused or
//! cut; a missing path, a tampered blob, an oversized file, a malformed one
//! or a host without a content base hangs instead of failing with its
//! reason; a read the host holds back (the web) lands at a tick the replay
//! does not reproduce; a replay reading other bytes passes; a released load
//! is still put in place or journaled; a manifest the catalog's head did not
//! name is read as if it were; a pack (og.load.pack) opens from bytes its
//! version does not list, or a replay opens one of another integrity.

use og_runtime::host_impl::answer;
use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_destroy, og_engine_frame,
    og_engine_set_content, og_game_game, og_load_bundle_info, og_load_completions, og_load_data,
    og_load_data_bytes, og_load_data_json, og_load_error, og_load_pack, og_load_release,
    og_load_replay_completions, og_load_status, og_platform_answer, og_platform_catalog_head,
    og_platform_notify, og_platform_take, set_hooks,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

fn sri(bytes: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA384, bytes);
    format!("sha384-{}", og_core::base64::encode(digest.as_ref()))
}

fn hex(integrity: &str) -> String {
    let digest = og_core::base64::decode(integrity.strip_prefix("sha384-").unwrap()).unwrap();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// A content base on disk; a read whose path contains one of `held` is
/// pending (a web fetch on its way) until the test clears it.
struct Content {
    held: RefCell<Vec<String>>,
    clock: Cell<f64>,
    asked: RefCell<Vec<(String, String)>>,
}
impl Hooks for Content {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn now_ms(&self) -> f64 {
        self.clock.get()
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
    fn read_content(&self, path: &str, integrity: &str) -> Fetch<Vec<u8>> {
        self.asked
            .borrow_mut()
            .push((path.to_string(), integrity.to_string()));
        if self.held.borrow().iter().any(|h| path.contains(h.as_str())) {
            return Fetch::Pending;
        }
        self.read_asset(path)
    }
}

/// `world/v1`: `index/big.json` (200 KiB of JSON), `raw.bin`, `bad.json`
/// (not JSON) and `tampered.json` (its blob is not the bytes the manifest
/// lists). `world/v2` lists `index/big.json` with other bytes.
fn catalog(base: &Path) -> (Value, Vec<u8>, String) {
    let rows: Vec<Value> = (0..4000)
        .map(|i| json!({ "cell": i, "name": format!("parcel-{i:05}"), "y": i % 7 }))
        .collect();
    let big = json!({ "cells": rows });
    let big_bytes = serde_json::to_vec(&big).unwrap();
    assert!(big_bytes.len() > 128 * 1024, "{}", big_bytes.len());
    let raw: Vec<u8> = (0..=255u8).cycle().take(70_000).collect();
    let bad = b"{ \"cells\": [1, 2".to_vec();
    let tampered = b"{\"ok\":true}".to_vec();
    let mut big2 = big.clone();
    big2["cells"][0]["y"] = json!(99);
    let big2_bytes = serde_json::to_vec(&big2).unwrap();

    let blobs = base.join("world/blobs");
    std::fs::create_dir_all(&blobs).unwrap();
    let put = |bytes: &[u8], stored: &[u8]| {
        let i = sri(bytes);
        std::fs::write(blobs.join(hex(&i)), stored).unwrap();
        json!({ "size": bytes.len(), "integrity": i })
    };
    let files = json!({
        "index/big.json": put(&big_bytes, &big_bytes),
        "raw.bin": put(&raw, &raw),
        "bad.json": put(&bad, &bad),
        "tampered.json": put(&tampered, b"{\"ok\":fals}"),
    });
    let v2 = json!({ "index/big.json": put(&big2_bytes, &big2_bytes) });
    for (version, files) in [("v1", files), ("v2", v2)] {
        let dir = base.join("world").join(version);
        std::fs::create_dir_all(&dir).unwrap();
        let m = json!({ "format": 1, "catalog": "world", "version": version, "files": files });
        std::fs::write(dir.join("manifest.json"), m.to_string()).unwrap();
    }
    (big, raw, sri(&big_bytes))
}

fn engine(hooks: &Rc<Content>, content: Option<&Path>) -> u32 {
    set_hooks(hooks.clone());
    og_runtime::og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = match content {
        Some(c) => json!({ "content": c }),
        None => json!({}),
    };
    value(og_engine_create(game, options.to_string()))
        .as_u64()
        .unwrap() as u32
}

fn data(e: u32, version: &str, path: &str, format: &str, bytes: Option<f64>) -> f64 {
    let mut desc = json!({ "entry": { "catalog": "world", "version": version, "path": path }, "format": format });
    if let Some(b) = bytes {
        desc["budget"] = json!({ "bytes": b });
    }
    value(answer(og_load_data(e, Some(desc.to_string()))))
        .as_f64()
        .unwrap()
}

fn frame(e: u32) {
    value(og_engine_frame(e, 1000.0 / 60.0));
}

fn failed(e: u32, h: f64) -> String {
    assert_eq!(og_load_status(e, h), "failed");
    og_load_error(e, h).unwrap()
}

fn base(name: &str) -> PathBuf {
    let b = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-load-data-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&b);
    b
}

fn hooks() -> Rc<Content> {
    Rc::new(Content {
        held: RefCell::default(),
        clock: Cell::new(1.0),
        asked: RefCell::default(),
    })
}

#[test]
fn catalog_files_load_by_entry_and_fail_with_their_reason() {
    let base = base("reasons");
    let (big, raw, big_sri) = catalog(&base);
    let h = hooks();
    let e = engine(&h, Some(&base));

    let json_h = data(e, "v1", "index/big.json", "json", None);
    let bytes_h = data(e, "v1", "raw.bin", "bytes", None);
    let missing = data(e, "v1", "index/none.json", "json", None);
    let no_version = data(e, "v9", "index/big.json", "json", None);
    let tampered = data(e, "v1", "tampered.json", "json", None);
    let small = data(e, "v1", "index/big.json", "json", Some(1000.0));
    let bad = data(e, "v1", "bad.json", "json", None);
    // Pending until the tick boundary, even natively.
    assert_eq!(og_load_status(e, json_h), "pending");
    frame(e);

    assert_eq!(og_load_status(e, json_h), "ready");
    assert_eq!(og_load_data_json(e, json_h).unwrap(), big);
    let b64 = og_load_data_bytes(e, bytes_h).unwrap();
    assert_eq!(og_core::base64::decode(b64.as_str().unwrap()).unwrap(), raw);
    // A file read as one format is not handed out as the other.
    assert!(og_load_data_bytes(e, json_h).is_err());

    for (handle, code) in [
        (missing, "not_found:"),
        (no_version, "not_found:"),
        (tampered, "integrity:"),
        (small, "too_large:"),
        (bad, "invalid:"),
    ] {
        let why = failed(e, handle);
        assert!(why.starts_with(code), "{code} expected: {why}");
    }
    // The blob is read under its digest and the integrity goes with it.
    let asked = h.asked.borrow();
    let blob = format!("world/blobs/{}", hex(&big_sri));
    assert!(
        asked
            .iter()
            .any(|(p, i)| p.ends_with(&blob) && *i == big_sri),
        "{asked:?}"
    );
    // The manifest is read once for every load of its version.
    let manifests = asked
        .iter()
        .filter(|(p, _)| p.ends_with("world/v1/manifest.json"))
        .count();
    assert_eq!(manifests, 1, "{asked:?}");
    drop(asked);

    // The journal: every finished load with its tick, the ready ones with
    // the integrity they read.
    let journal = og_load_completions(e);
    let list = journal.as_array().unwrap();
    assert_eq!(list.len(), 7, "{journal}");
    assert!(list.iter().all(|c| c["kind"] == "data"), "{journal}");
    let first = list.iter().find(|c| c["request"] == 0).unwrap();
    assert_eq!(first["ok"], true);
    assert_eq!(first["integrity"], json!(big_sri));
    let missing_c = list.iter().find(|c| c["request"] == 2).unwrap();
    assert_eq!(missing_c["ok"], false);
    assert!(missing_c.get("integrity").is_none(), "{journal}");

    // Released: its bytes are gone and the handle says why.
    og_load_release(e, json_h).unwrap();
    assert!(failed(e, json_h).contains("released"));

    // A path outside the version is refused at the call.
    let refused = og_load_data(
        e,
        Some(
            json!({ "entry": { "catalog": "world", "version": "v1", "path": "../v2/x" } })
                .to_string(),
        ),
    );
    assert!(refused.is_err());
    assert!(og_engine_destroy(e));

    // No content base: a failed load, not a hang.
    let e = engine(&h, None);
    let none = data(e, "v1", "index/big.json", "json", None);
    frame(e);
    assert!(failed(e, none).starts_with("unavailable:"));
    assert!(og_engine_destroy(e));
    std::fs::remove_dir_all(base).unwrap();
}

/// The frame after which `index/big.json` was ready, and the journal; the
/// blob is held for `held` frames. With `replay`, that journal goes first.
fn late(
    base: &Path,
    version: &str,
    held: u32,
    replay: Option<&Value>,
) -> (Option<u32>, Value, f64, u32) {
    let h = hooks();
    let e = engine(&h, Some(base));
    if let Some(j) = replay {
        og_load_replay_completions(e, j.to_string()).unwrap();
    }
    if held > 0 {
        h.held.borrow_mut().push("world/blobs/".into());
    }
    frame(e);
    let handle = data(e, version, "index/big.json", "json", None);
    let mut ready = None;
    for f in 0..10 {
        if f == held {
            h.held.borrow_mut().clear();
        }
        frame(e);
        if ready.is_none() && og_load_status(e, handle) != "pending" {
            ready = Some(f);
        }
    }
    (ready, og_load_completions(e), handle, e)
}

#[test]
fn a_late_file_replays_at_its_journaled_tick_with_its_bytes() {
    let base = base("replay");
    catalog(&base);
    // On the web the blob arrives a few frames late.
    let (web, journal, h, e) = late(&base, "v1", 4, None);
    assert_eq!(og_load_status(e, h), "ready");
    assert!(og_engine_destroy(e));
    let (native, _, _, e) = late(&base, "v1", 0, None);
    assert!(og_engine_destroy(e));
    assert_ne!(
        web, native,
        "the held read must land later for this test to see anything"
    );

    // A replay reads at once, yet puts it in place at the recorded tick.
    let (replayed, again, h, e) = late(&base, "v1", 0, Some(&journal));
    assert_eq!(replayed, web);
    assert_eq!(again, journal);
    assert_eq!(og_load_status(e, h), "ready");
    assert!(og_engine_destroy(e));

    // Other bytes than the recorded run read fail instead of drifting.
    let (_, _, h, e) = late(&base, "v2", 0, Some(&journal));
    let why = failed(e, h);
    assert!(why.starts_with("integrity:"), "{why}");
    assert!(og_engine_destroy(e));

    // Released while on its way: never put in place, never journaled.
    let hk = hooks();
    let e = engine(&hk, Some(&base));
    hk.held.borrow_mut().push("world/blobs/".into());
    let pending = data(e, "v1", "index/big.json", "json", None);
    frame(e);
    og_load_release(e, pending).unwrap();
    hk.held.borrow_mut().clear();
    frame(e);
    frame(e);
    assert!(failed(e, pending).contains("released"));
    assert_eq!(og_load_completions(e), json!([]));

    // Nothing arrives: it times out after 30 seconds.
    hk.held.borrow_mut().push("world/blobs/".into());
    let stuck = data(e, "v1", "raw.bin", "bytes", None);
    frame(e);
    assert_eq!(og_load_status(e, stuck), "pending");
    hk.clock.set(hk.clock.get() + 31_000.0);
    frame(e);
    assert!(failed(e, stuck).starts_with("timeout:"));
    assert!(og_engine_destroy(e));
    std::fs::remove_dir_all(base).unwrap();
}

/// The head (its answer, or a `catalog` notice) names the version
/// manifest's integrity: a manifest read after it with other bytes fails
/// `integrity`, the one it names loads, and a head for another version
/// pins nothing here.
#[test]
fn the_catalog_head_pins_the_version_manifest() {
    let base = base("pin");
    catalog(&base);
    let manifest =
        |v: &str| std::fs::read(base.join("world").join(v).join("manifest.json")).unwrap();
    let (v1, v2) = (sri(&manifest("v1")), sri(&manifest("v2")));
    let h = hooks();

    // Answered: v1's head names v2's manifest (a server that swapped them).
    let e = engine(&h, Some(&base));
    let request = og_platform_catalog_head(e, "world".into()).unwrap();
    assert!(og_platform_catalog_head(e, "../x".into()).is_err());
    frame(e);
    let calls: Value = serde_json::from_str(&og_platform_take(e)).unwrap();
    let call = calls
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["request"] == request)
        .unwrap();
    assert_eq!(call["method"], "catalog.head", "{calls}");
    assert_eq!(call["args"], json!({ "catalog": "world" }));
    let head = json!({ "ok": true, "data": { "catalog": "world", "version": "v1", "manifest": v2, "published": 1 } });
    assert!(og_platform_answer(e, request, head.to_string()));
    frame(e);
    let swapped = data(e, "v1", "index/big.json", "json", None);
    frame(e);
    let why = failed(e, swapped);
    assert!(
        why.starts_with("integrity:") && why.contains("manifest.json"),
        "{why}"
    );
    assert!(og_engine_destroy(e));

    // Noticed: the manifest the head names loads.
    let e = engine(&h, Some(&base));
    let notice = json!({ "kind": "catalog", "data": { "catalog": "world", "version": "v1", "manifest": v1 } });
    assert!(og_platform_notify(e, notice.to_string()));
    frame(e);
    let good = data(e, "v1", "index/big.json", "json", None);
    frame(e);
    assert_eq!(
        og_load_status(e, good),
        "ready",
        "{:?}",
        og_load_error(e, good)
    );
    // A head naming only v1 leaves v2 unpinned: read as it is.
    let other = data(e, "v2", "index/big.json", "json", None);
    frame(e);
    assert_eq!(
        og_load_status(e, other),
        "ready",
        "{:?}",
        og_load_error(e, other)
    );
    assert!(og_engine_destroy(e));
    std::fs::remove_dir_all(base).unwrap();
}

/// A page that learns its content base from its Player's hello creates the
/// engine with `contentLater`: a data load waits (it neither fails
/// `unavailable` nor reads anything) until the host names the base, then
/// loads; named empty, it fails `unavailable`.
#[test]
fn a_content_base_named_later_holds_data_loads_until_then() {
    let base = base("later");
    catalog(&base);
    let h = hooks();
    let later = || {
        set_hooks(h.clone());
        og_runtime::og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        value(og_engine_create(
            game,
            json!({ "contentLater": true }).to_string(),
        ))
        .as_u64()
        .unwrap() as u32
    };
    let e = later();
    let waiting = data(e, "v1", "raw.bin", "bytes", None);
    for _ in 0..5 {
        frame(e);
    }
    assert_eq!(og_load_status(e, waiting), "pending");
    assert!(h.asked.borrow().is_empty(), "{:?}", h.asked.borrow());
    og_engine_set_content(e, format!("{}/", base.display()));
    frame(e);
    assert_eq!(
        og_load_status(e, waiting),
        "ready",
        "{:?}",
        og_load_error(e, waiting)
    );
    assert!(og_engine_destroy(e));

    let e = later();
    let none = data(e, "v1", "raw.bin", "bytes", None);
    frame(e);
    og_engine_set_content(e, String::new());
    frame(e);
    assert!(failed(e, none).starts_with("unavailable:"));
    assert!(og_engine_destroy(e));
    std::fs::remove_dir_all(base).unwrap();
}

/// Cooks a one-model bundle at `<base>/world/packs/<name>/cooked` and
/// answers its manifest's integrity.
fn cooked_pack(base: &Path, name: &str) -> String {
    let root = base.join("world/packs").join(name);
    let model = root.join("assets/rock.ogt");
    std::fs::create_dir_all(model.parent().unwrap()).unwrap();
    std::fs::write(
        &model,
        "type model\nversion 1\n---\npart \"rock\" {\n  sphereGeometry {\n    radius 1\n  }\n}\n",
    )
    .unwrap();
    let report = og_cook::cook_project(
        &root,
        &og_cook::CookOptions {
            threads: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    sri(&std::fs::read(root.join("cooked/.cook.ogt")).unwrap())
}

fn pack(e: u32, version: &str, name: &str) -> f64 {
    let entry = json!({ "catalog": "world", "version": version, "path": name });
    value(answer(og_load_pack(e, Some(entry.to_string()))))
        .as_f64()
        .unwrap()
}

#[test]
fn a_pack_opens_as_a_bundle_against_its_versions_integrity() {
    let base = base("packs");
    let region = cooked_pack(&base, "region");
    // `tampered` is a real bundle the version lists under other bytes.
    let other = cooked_pack(&base, "tampered");
    assert_eq!(other, region, "the two packs cook alike");
    let dir = base.join("world/p1");
    std::fs::create_dir_all(&dir).unwrap();
    let m = json!({
        "format": 1, "catalog": "world", "version": "p1", "files": {},
        "packs": {
            "region": { "integrity": region },
            "tampered": { "integrity": sri(b"not its manifest") },
        },
    });
    std::fs::write(dir.join("manifest.json"), m.to_string()).unwrap();

    let opens = |h: &Rc<Content>, replay: Option<&Value>| {
        let e = engine(h, Some(&base));
        if let Some(journal) = replay {
            value(answer(og_load_replay_completions(e, journal.to_string())));
        }
        let handles = [
            pack(e, "p1", "region"),
            pack(e, "p1", "tampered"),
            pack(e, "p1", "none"),
        ];
        assert_eq!(og_load_status(e, handles[0]), "pending");
        frame(e);
        (e, handles)
    };
    let h = hooks();
    let (e, [ready, tampered, none]) = opens(&h, None);
    assert_eq!(
        og_load_status(e, ready),
        "ready",
        "{:?}",
        og_load_error(e, ready)
    );
    let info = og_load_bundle_info(e, ready).unwrap();
    assert!(
        info.path.ends_with("world/packs/region/cooked"),
        "{}",
        info.path
    );
    assert_eq!(info.assets, 1);
    // Its manifest was read against the integrity the version lists.
    assert!(
        h.asked
            .borrow()
            .iter()
            .any(|(p, i)| p.ends_with("region/cooked/.cook.ogt") && *i == region),
        "{:?}",
        h.asked.borrow()
    );
    let why = failed(e, tampered);
    assert!(why.starts_with("integrity:"), "{why}");
    let why = failed(e, none);
    assert!(why.starts_with("not_found:"), "{why}");
    // Asking again answers the same handle; a malformed name is refused.
    assert_eq!(pack(e, "p1", "region"), ready);
    let refused = og_load_pack(
        e,
        Some(json!({ "catalog": "world", "version": "p1", "path": "a/b" }).to_string()),
    );
    assert!(refused.is_err());

    // Journaled as bundle opens, the ready one with its manifest integrity.
    let journal = og_load_completions(e);
    let list = journal.as_array().unwrap();
    assert_eq!(list.len(), 3, "{journal}");
    assert!(list.iter().all(|c| c["kind"] == "bundle"), "{journal}");
    let first = list.iter().find(|c| c["request"] == 0).unwrap();
    assert_eq!(
        (first["ok"].clone(), first["integrity"].clone()),
        (json!(true), json!(region))
    );

    // Released: closed like any runtime bundle.
    og_load_release(e, ready).unwrap();
    assert!(failed(e, ready).contains("released"));
    assert!(og_engine_destroy(e));

    // A replay opens it again at its tick; one recorded under another
    // integrity is refused.
    let (e, [again, ..]) = opens(&hooks(), Some(&journal));
    assert_eq!(og_load_status(e, again), "ready");
    assert!(og_engine_destroy(e));
    let mut forged = journal.clone();
    for c in forged.as_array_mut().unwrap() {
        if c["request"] == 0 {
            c["integrity"] = json!(sri(b"another pack"));
        }
    }
    let (e, [again, ..]) = opens(&hooks(), Some(&forged));
    let why = failed(e, again);
    assert!(why.starts_with("integrity:"), "{why}");
    assert!(og_engine_destroy(e));
    std::fs::remove_dir_all(base).unwrap();
}
