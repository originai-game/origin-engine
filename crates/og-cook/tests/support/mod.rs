//! A cook made once and read by every test process that needs it.
//!
//! nextest runs each test in its own process, so a `OnceLock` alone cooks
//! the same sources once per test. [`shared`] cooks into
//! `<target tmp>/og-cook-shared/<name>-<key>/` under a file lock and every
//! later process reads that directory. The key covers the test executable
//! (relinked whenever the cook's code or a dependency changes) and the
//! sources (path, length, modification time), so a stale cook is never
//! reused. The tests that read it only read: a test that edits a project
//! copies it first.

#![allow(dead_code)] // each test binary uses part of it

use og_cook::cook::{CookOptions, cook_project};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// `templates/game/assets`: what the template ships.
pub fn template_assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/game/assets")
}

/// A model of the template fixture by file name. `trickster.glb` is the
/// engine's test copy (`crates/og-tests/assets/`): it left the template on
/// 2026-10-22 for want of a licence (docs/release.md R6; the template's
/// `trickster` entity is the CC0 walker since), while the tests below and
/// their frames were recorded with it, and the engine's copy is in no
/// package. Every other model is the template's.
pub fn template_model(name: &str) -> PathBuf {
    if name == "trickster.glb" {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-tests/assets/trickster.glb")
    } else {
        template_assets().join("models").join(name)
    }
}

/// The sources of the template fixture, for [`shared`]'s key.
pub fn template_sources() -> Vec<PathBuf> {
    vec![template_assets(), template_model("trickster.glb")]
}

/// Stages the template fixture at `assets`: the template's assets with
/// `trickster.glb` beside them, as the template shipped until 2026-10-22.
pub fn stage_template(assets: &Path) {
    copy_dir(&template_assets(), assets);
    std::fs::copy(
        template_model("trickster.glb"),
        assets.join("models/trickster.glb"),
    )
    .unwrap();
}

pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).unwrap();
        }
    }
}

/// What the cook that made a shared directory reported.
pub struct SharedCook {
    /// The project directory: `assets/` as staged, `cooked/` as cooked.
    pub root: PathBuf,
    pub report_markdown: String,
    pub report_json: String,
    pub source_bytes: u64,
    pub cooked_bytes: u64,
    pub elapsed_ms: f64,
}

fn key_of(sources: &[PathBuf]) -> u64 {
    let mut h = DefaultHasher::new();
    let exe = std::env::current_exe().expect("the test executable");
    for path in std::iter::once(&exe).chain(sources) {
        let mut files = Vec::new();
        walk(path, &mut files);
        for f in files {
            let m = std::fs::metadata(&f).unwrap();
            f.hash(&mut h);
            m.len().hash(&mut h);
            m.modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .hash(&mut h);
        }
    }
    h.finish()
}

fn walk(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for e in entries {
            walk(&e, out);
        }
    } else {
        out.push(path.to_path_buf());
    }
}

/// The project `stage` writes (into the empty directory it is given), cooked
/// with the default options, made by the first test process that asks and
/// reused by the rest. `sources` are the files or directories `stage` reads.
pub fn shared(name: &str, sources: &[PathBuf], stage: impl FnOnce(&Path)) -> SharedCook {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-shared");
    std::fs::create_dir_all(&base).unwrap();
    let dir = base.join(format!("{name}-{:016x}", key_of(sources)));
    let lock = std::fs::File::create(base.join(format!("{name}.lock"))).unwrap();
    lock.lock().unwrap();
    if !dir.join(".shared/ok").exists() {
        // Earlier keys of this name are stale (exactly `<name>-<16 hex>`: a
        // longer name sharing the prefix belongs to another test binary).
        for e in std::fs::read_dir(&base).unwrap() {
            let p = e.unwrap().path();
            let stale = p
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix(&format!("{name}-")))
                .is_some_and(|k| k.len() == 16 && k.bytes().all(|b| b.is_ascii_hexdigit()));
            if stale && p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            }
        }
        let work = base.join(format!("{name}.work"));
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).unwrap();
        stage(&work);
        let report = cook_project(&work, &CookOptions::default()).expect("the cook runs");
        assert!(report.ok(), "cook errors: {:?}", report.errors());
        let meta = work.join(".shared");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::write(meta.join("report.md"), report.to_markdown()).unwrap();
        std::fs::write(meta.join("report.json"), report.to_json()).unwrap();
        std::fs::write(
            meta.join("numbers"),
            format!(
                "{} {} {}",
                report.source_bytes(),
                report.cooked_bytes(),
                report.elapsed_ms
            ),
        )
        .unwrap();
        std::fs::write(meta.join("ok"), "").unwrap();
        std::fs::rename(&work, &dir).unwrap();
    }
    drop(lock);
    let meta = dir.join(".shared");
    let numbers = std::fs::read_to_string(meta.join("numbers")).unwrap();
    let n: Vec<&str> = numbers.split(' ').collect();
    SharedCook {
        report_markdown: std::fs::read_to_string(meta.join("report.md")).unwrap(),
        report_json: std::fs::read_to_string(meta.join("report.json")).unwrap(),
        source_bytes: n[0].parse().unwrap(),
        cooked_bytes: n[1].parse().unwrap(),
        elapsed_ms: n[2].parse().unwrap(),
        root: dir,
    }
}
