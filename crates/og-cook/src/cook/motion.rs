//! og.motion's model as an optional pack: `cooked/motion/model.ogb`, the
//! engine's motion weights (`motion/model.safetensors` inside a stream
//! pack), written only for a project whose code uses og.motion. It is not
//! in the manifest, so the boot bundle never fetches it: the runtime reads
//! it the first time an agent is drawn (og-runtime's `motion::present`).
//!
//! The weights ship inside `@origingame/engine` (`motion/model.safetensors`,
//! put there by `scripts/pack.sh`; docs/design.md §2.7). They are looked
//! for at `OG_MOTION_MODEL` (a file: named, it is the one place looked at),
//! then in the engine package (`OG_ENGINE_PACKAGE`, the package beside this
//! `og`, or a checkout's `packages/engine`). A project that uses og.motion
//! without them cooks with a warning, and its agents draw og.anim's pose.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::CookError;
use og_core::hash::hash_bytes;

/// The pack, under `cooked/`.
pub(super) const FILE: &str = "motion/model.ogb";
/// The weights' path inside the pack.
const ENTRY: &str = "motion/model.safetensors";
/// The weights in an engine package.
const IN_PACKAGE: &str = "motion/model.safetensors";

/// Whether the project's code under `src/` names og.motion.
fn uses_motion(root: &Path) -> bool {
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
fn model_path() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("OG_MOTION_MODEL") {
        let p = PathBuf::from(p);
        return if p.is_file() {
            Ok(p)
        } else {
            Err(format!(
                "OG_MOTION_MODEL names {}, which is not a file",
                p.display()
            ))
        };
    }
    let mut tried = Vec::new();
    if let Some(pkg) = std::env::var_os("OG_ENGINE_PACKAGE") {
        tried.push(PathBuf::from(pkg).join(IN_PACKAGE));
    }
    // The og of an npm install sits in @origingame/engine-<platform>/, a
    // sibling of @origingame/engine; a checkout's og under target/ or
    // packages/engine/native/.
    if let Ok(exe) = std::env::current_exe() {
        for a in exe.ancestors().skip(1) {
            tried.push(a.join("engine").join(IN_PACKAGE));
            tried.push(a.join("packages/engine").join(IN_PACKAGE));
        }
    }
    tried.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine")
            .join(IN_PACKAGE),
    );
    tried.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        "the engine's motion model is not installed (@origingame/engine's motion/model.safetensors; OG_MOTION_MODEL names another file)".into()
    })
}

/// Writes (or removes) the pack: kept in `keep` when written; a warning
/// when the project uses og.motion and no weights are found.
pub(super) fn cook(
    root: &Path,
    cooked_dir: &Path,
    keep: &mut BTreeSet<String>,
    warnings: &mut Vec<String>,
) -> Result<(), CookError> {
    let path = cooked_dir.join(FILE);
    if !uses_motion(root) {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| CookError(format!("cooked/{FILE}: {e}")))?;
        }
        return Ok(());
    }
    let model = match model_path() {
        Ok(p) => p,
        Err(why) => {
            warnings.push(format!(
                "src/ uses og.motion, but {why}: its agents draw og.anim's pose (og.motion.capabilities answers none)"
            ));
            return Ok(());
        }
    };
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
    Ok(())
}
