//! og.motion's model as an optional pack: `cooked/motion/model.ogb`, the
//! engine's motion weights (`motion/model.safetensors` inside a stream
//! pack), written only for a project whose code uses og.motion. The
//! manifest names it in its `motion` entry, outside the outputs, so the
//! boot bundle never fetches it: the runtime reads it in the background
//! once the bundle is open (og-runtime's `motion::present`).
//!
//! `motion.json` in the engine pins the weights' URL, length and SHA256.
//! Only a cook that uses motion installs them, in the shared engine cache.
//! A verified cache works offline; a missing model offline is an error,
//! never a successful cook whose agents silently lose motion.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use super::CookError;
use og_bundle::runtime::ManifestFile;
use og_core::hash::hash_bytes;

/// The pack, under `cooked/`.
pub(crate) const FILE: &str = "motion/model.ogb";
/// The weights' path inside the pack.
const ENTRY: &str = "motion/model.safetensors";
/// The weights in an engine package.
const IN_PACKAGE: &str = "motion/model.safetensors";

/// Whether the project's code under `src/` names og.motion.
pub(crate) fn uses_motion(root: &Path) -> bool {
    fn walk(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.flatten().any(|e| {
            let p = e.path();
            if p.is_dir() {
                return !e.file_name().to_string_lossy().starts_with('.') && walk(&p);
            }
            matches!(
                p.extension().and_then(|x| x.to_str()),
                Some("ts" | "tsx" | "js")
            ) && std::fs::read_to_string(&p)
                .is_ok_and(|t| t.contains("og.motion.") || t.contains("og.MotionAgent"))
        })
    }
    walk(&root.join("src"))
}

/// Where the weights are, or why none were found.
pub(crate) fn model_path() -> Result<PathBuf, String> {
    let mut tried = Vec::new();
    if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE") {
        tried.push(PathBuf::from(pkg));
    }
    // The og of an npm install sits in @origingame/engine-<platform>/, a
    // sibling of @origingame/engine; a checkout's og under target/ or
    // packages/engine/native/.
    if let Ok(exe) = std::env::current_exe() {
        for a in exe.ancestors().skip(1) {
            tried.push(a.join("engine"));
            tried.push(a.join("packages/engine"));
        }
    }
    tried.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine"));
    let package = tried
        .into_iter()
        .find(|p| p.join("motion.json").is_file())
        .ok_or("the engine has no motion.json pin; reinstall the engine")?;
    let text = std::fs::read(package.join("motion.json")).map_err(|e| e.to_string())?;
    #[derive(serde::Deserialize)]
    struct Pin {
        bytes: u64,
        sha256: String,
        url: String,
    }
    let pin: Pin = serde_json::from_slice(&text).map_err(|e| format!("motion.json: {e}"))?;
    if pin.sha256.len() != 64
        || !pin.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || pin.bytes == 0
    {
        return Err("motion.json: invalid SHA256 or byte length".into());
    }
    let verify = |path: &Path| -> Result<(), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
        let sha: String = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect();
        if bytes.len() as u64 != pin.bytes || sha != pin.sha256 {
            return Err(format!(
                "{} does not match motion.json (expected {} bytes, SHA256 {})",
                path.display(),
                pin.bytes,
                pin.sha256
            ));
        }
        Ok(())
    };
    if let Some(path) = std::env::var_os("OG_MOTION_MODEL").filter(|p| !p.is_empty()) {
        let path = PathBuf::from(path);
        verify(&path)?;
        return Ok(path);
    }
    // Development installs may already carry the pin; never mutate the install.
    let bundled = package.join(IN_PACKAGE);
    if bundled.is_file() {
        verify(&bundled)?;
        return Ok(bundled);
    }
    let cache = std::env::var_os("ORIGIN_ENGINE_CACHE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache/origin-engine")))
        .or_else(|| {
            std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("origin-engine"))
        })
        .ok_or("set ORIGIN_ENGINE_CACHE to install the motion model")?
        .join("models/motion");
    std::fs::create_dir_all(&cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let path = cache.join(format!("{}.safetensors", pin.sha256));
    if verify(&path).is_ok() {
        return Ok(path);
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary = cache.join(format!(
        ".{}-{}-{}.part",
        pin.sha256,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let install = || -> Result<(), String> {
        let output = Command::new("curl")
            .args([
                "--fail",
                "--location",
                "--silent",
                "--show-error",
                "--connect-timeout",
                "10",
                "--max-time",
                "120",
                "--max-filesize",
                &pin.bytes.to_string(),
                "--output",
            ])
            .arg(&temporary)
            .arg(&pin.url)
            .output()
            .map_err(|e| format!("cannot start curl: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "download {} failed: {}",
                pin.url,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        verify(&temporary)?;
        // Concurrent cooks use separate temporary files and converge on the pin.
        if verify(&path).is_ok() {
            return Ok(());
        }
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        if let Err(e) = std::fs::rename(&temporary, &path) {
            verify(&path).map_err(|_| e.to_string())?;
        }
        Ok(())
    };
    let result = install();
    let _ = std::fs::remove_file(&temporary);
    result.map_err(|e| format!("motion model is not available in the verified cache (offline or download refused): {e}; connect and cook again, or set OG_MOTION_MODEL to the pinned file (SHA256 {})", pin.sha256))?;
    Ok(path)
}

/// Writes (or removes) the pack, for a project whose code uses og.motion
/// ([`uses_motion`]) or does not: kept in `keep` and answered as the
/// manifest's `motion` entry when written; an error if required weights
/// cannot be installed and verified.
pub(crate) fn cook(
    uses_motion: bool,
    cooked_dir: &Path,
    keep: &mut BTreeSet<String>,
    _warnings: &mut Vec<String>,
) -> Result<Option<ManifestFile>, CookError> {
    let path = cooked_dir.join(FILE);
    let remove = || -> Result<Option<ManifestFile>, CookError> {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| CookError(format!("cooked/{FILE}: {e}")))?;
        }
        Ok(None)
    };
    if !uses_motion {
        return remove();
    }
    let model = model_path().map_err(CookError)?;
    let bytes =
        std::fs::read(&model).map_err(|e| CookError(format!("{}: {e}", model.display())))?;
    let pack = super::stream::pack(&[(ENTRY.to_string(), bytes.clone())], hash_bytes(&bytes));
    if std::fs::read(&path).ok().as_deref() != Some(&pack[..]) {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
        }
        std::fs::write(&path, &pack).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    }
    keep.insert(FILE.to_string());
    Ok(Some(ManifestFile {
        file: FILE.to_string(),
        bytes: pack.len() as u64,
        hash: hash_bytes(&pack),
        integrity: crate::cook::integrity(&pack),
    }))
}
