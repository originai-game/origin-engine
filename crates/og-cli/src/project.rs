//! A game project as `og` sees it: `project.ogt`, the engine package that
//! runs it, and the bun scripts of that package.
//!
//! `project.ogt` (type `project`) carries `name`, `id`, `engine` (the pinned
//! engine version), `entry`, `profile` (`2d`/`3d`), `tick`, `seed`, a
//! `scenes { name "path" }` block, `scene`, and `publish { probe player; out
//! "dist" }`; `og` reads `entry` and `publish.out` here, `publish` in
//! `publish.rs`, and hands the whole text to the engine (CreateOptions.project:
//! `packages/engine/src/node/run.ts`, `polish.ts`, the play page's config),
//! which takes `tick`, `seed`, `scenes` and `scene` for the GameDesc fields a
//! game leaves unset and refuses one set in both to different values
//! (og-runtime `engine::with_project`). A bare entry file (`game.ts`) is also
//! accepted where a project is: it runs without a cooked bundle.

use og_core::ogt::Entry;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What `og` runs.
#[derive(Clone, Debug)]
pub struct Project {
    /// The project directory (the entry's directory for a bare entry).
    pub root: PathBuf,
    /// Whether a `project.ogt` was found.
    pub manifest: bool,
    pub entry: PathBuf,
    pub out: String,
}

fn string(v: Option<&Entry>) -> Option<String> {
    v.and_then(|v| v.string(0).ok().map(str::to_string))
}

impl Project {
    /// `path` is a project directory, a `project.ogt`, or an entry file.
    pub fn open(path: &Path) -> Result<Project, String> {
        let (root, manifest) = if path.is_dir() {
            (path.to_path_buf(), path.join("project.ogt"))
        } else if path.file_name().is_some_and(|f| f == "project.ogt") {
            (
                path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                path.to_path_buf(),
            )
        } else if path.is_file() {
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let root = dir
                .ancestors()
                .find(|d| d.join("project.ogt").is_file())
                .map(Path::to_path_buf);
            return Ok(match root {
                Some(r) => Project {
                    entry: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
                    ..Project::open(&r)?
                },
                None => Project {
                    root: dir,
                    manifest: false,
                    entry: path.to_path_buf(),
                    out: "dist".into(),
                },
            });
        } else {
            return Err(format!(
                "{} is neither a game entry nor a project directory",
                path.display()
            ));
        };
        let root = root.canonicalize().unwrap_or(root);
        if !manifest.is_file() {
            return Err(format!(
                "{}: no project.ogt; scaffold a project with `npm create @origingame/game` or pass a game entry file",
                root.display()
            ));
        }
        let text = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("cannot read {}: {e}", manifest.display()))?;
        let doc = og_core::ogt::parse(&text)
            .map_err(|e| format!("{}: {}", manifest.display(), e.reason()))?;
        if doc.ty != "project" {
            return Err(format!(
                "{}: a `{}` document; project.ogt is `type project`",
                manifest.display(),
                doc.ty
            ));
        }
        let b = &doc.body;
        let entry = string(b.get("entry")).unwrap_or_else(|| "src/main.ts".into());
        let publish = b.get("publish").and_then(|p| p.block.as_ref());
        let out = publish
            .and_then(|p| string(p.get("out")))
            .unwrap_or_else(|| "dist".into());
        let entry = root.join(&entry);
        if !entry.is_file() {
            return Err(format!(
                "{}: the game entry {} does not exist",
                manifest.display(),
                entry.display()
            ));
        }
        Ok(Project {
            entry,
            root,
            manifest: true,
            out,
        })
    }

    /// Where `og` keeps its own build products (`.og/` in the project).
    pub fn work_dir(&self) -> PathBuf {
        self.root.join(".og")
    }

    /// Whether the project has an `assets/` directory to cook.
    pub fn has_assets(&self) -> bool {
        self.manifest && self.root.join("assets").is_dir()
    }
}

/// The engine package that runs games: `OG_ENGINE_PACKAGE`, the project's
/// `node_modules/@origingame/engine`, or `packages/engine` of the
/// origin-engine checkout `og` runs in or from.
pub fn engine_package(project: Option<&Path>) -> Option<PathBuf> {
    let ok = |p: &Path| p.join("src/node/run.ts").is_file();
    if let Ok(p) = std::env::var("OG_ENGINE_PACKAGE") {
        return Some(PathBuf::from(p));
    }
    if let Some(root) = project {
        for dir in root.ancestors() {
            let p = dir.join("node_modules/@origingame/engine");
            if ok(&p) {
                return Some(p);
            }
        }
        if let Some(repo) = crate::find_root(root) {
            let p = repo.join("packages/engine");
            if ok(&p) {
                return Some(p);
            }
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let repo = crate::find_root(&cwd).or_else(|| {
        let exe = std::env::current_exe().ok()?;
        crate::find_root(&exe)
    })?;
    let p = repo.join("packages/engine");
    ok(&p).then_some(p)
}

pub const NO_ENGINE: &str = "no engine package: run `bun install` in the project (it depends on @origingame/engine), set OG_ENGINE_PACKAGE to packages/engine of an origin-engine checkout, or run inside one";

/// `bun <script> <args>` with the script resolved in the engine package.
pub fn bun(package: &Path, script: &str) -> Command {
    let mut c = Command::new("bun");
    c.arg(package.join(script));
    c
}

pub const NO_BUN: &str = "cannot start bun; install it: curl -fsSL https://bun.sh/install | bash";

/// Bundles the game for the native host into `.og/game.native.js` and
/// returns its path.
pub fn bundle_native(project: &Project, package: &Path) -> Result<PathBuf, String> {
    let out = if project.manifest {
        project.work_dir().join("game.native.js")
    } else {
        std::env::temp_dir()
            .join(format!("og-run-{}", std::process::id()))
            .join("game.native.js")
    };
    let status = bun(package, "src/node/build.ts")
        .arg("--entry")
        .arg(&project.entry)
        .arg("--out")
        .arg(&out)
        .arg("--target")
        .arg("native")
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("{NO_BUN} ({e})"))?;
    if !status.success() {
        return Err(format!(
            "{} does not bundle (the error is above)",
            project.entry.display()
        ));
    }
    Ok(out)
}

/// The exit code of a child, as og's own.
pub fn exit_of(status: std::process::ExitStatus) -> std::process::ExitCode {
    if status.success() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(status.code().map_or(1, |c| c.clamp(1, 255) as u8))
    }
}
