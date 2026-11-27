//! The CLI's checked TS project, its pinned engine package and host scripts.
//! Only og-project.json marks a project. Bootstrap configuration is evaluated
//! by og-cook's restricted reader, never by importing the game's entry.

use og_cook::cook::{source_metadata, source_project};
use std::path::{Path, PathBuf};
use std::process::Command;

/// What `og` runs.
#[derive(Clone, Debug)]
pub struct Project {
    /// The project directory (the entry's directory for a bare entry).
    pub root: PathBuf,
    /// Whether the entry belongs to an og-project.json project.
    pub manifest: bool,
    pub entry: PathBuf,
    pub out: String,
    /// The ordinary JSON project's exact engine pin; absent for a bare entry.
    pub engine: Option<String>,
    /// project.ts's named median-time limits in milliseconds.
    pub budgets: Vec<(String, f64)>,
}

// Bun imports and TypeScript do not understand canonicalize's Win32 verbatim
// paths. Nor may a verbatim root be joined with '/' by the native host.
pub(crate) fn typescript_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if cfg!(windows) {
        let s = if let Some(unc) = s.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{unc}")
        } else {
            s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
        };
        s.replace('\\', "/")
    } else {
        s.into_owned()
    }
}

impl Project {
    /// A project directory/config, a nested directory, or a bare game entry.
    pub fn open(path: &Path) -> Result<Project, String> {
        let absolute = path
            .canonicalize()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let directory = if absolute.is_dir() {
            absolute.as_path()
        } else {
            absolute.parent().unwrap_or(Path::new("."))
        };
        let config = absolute
            .file_name()
            .is_some_and(|f| f == source_metadata::FILE || f == "project.ts");
        let entry = absolute.is_file()
            && !config
            && absolute
                .extension()
                .is_some_and(|e| matches!(e.to_str(), Some("ts" | "tsx" | "js" | "jsx")));
        if !absolute.is_dir() && !config && !entry {
            return Err(format!(
                "{} is neither a game entry nor a project directory",
                path.display()
            ));
        }
        let root = directory
            .ancestors()
            .find(|root| root.join(source_metadata::FILE).exists());
        let Some(root) = root else {
            if entry {
                return Ok(Project {
                    root: PathBuf::from(typescript_path(directory)),
                    manifest: false,
                    entry: PathBuf::from(typescript_path(&absolute)),
                    out: "dist".into(),
                    engine: None,
                    budgets: Vec::new(),
                });
            }
            return Err(format!(
                "{}: no og-project.json; scaffold a project with `og new <dir>` or pass a game entry file",
                directory.display()
            ));
        };
        let root = PathBuf::from(typescript_path(root));
        let checked = source_project::Project::open(&root).map_err(|e| e.to_string())?;
        let out = checked
            .input
            .publish
            .as_ref()
            .and_then(|p| p.out.clone())
            .unwrap_or_else(|| "dist".into());
        Ok(Project {
            entry: if entry {
                PathBuf::from(typescript_path(&absolute))
            } else {
                root.join(&checked.input.entry)
            },
            root,
            manifest: true,
            out,
            engine: Some(checked.metadata.engine),
            budgets: checked.input.perf.into_iter().collect(),
        })
    }

    /// Where `og` keeps its own build products (`.og/` in the project).
    pub fn work_dir(&self) -> PathBuf {
        self.root.join(".og")
    }

    /// A TS project cooks its declared content even without binary assets.
    pub fn has_assets(&self) -> bool {
        self.manifest
    }
}

/// Checked bootstrap data for host tools. No game or content entry executes,
/// and nothing is written back to the ordinary JSON project file.
pub fn snapshot(root: &Path) -> Result<serde_json::Value, String> {
    use og_api::wire::{ProjectStartup, ToJson};
    use og_cook::cook::source_schemas::procedural::LookBase;
    use serde_json::{Value, json};

    let project = source_project::Project::open(root).map_err(|e| e.to_string())?;
    let assets = og_cook::cook::project_assets(root, &project).map_err(|e| e.to_string())?;
    let asset = |id: &str, kind: &str| -> Result<String, String> {
        let output = assets
            .get(id)
            .ok_or_else(|| format!("project ref {id}: no scanned {kind} output"))?;
        if output.kind != kind {
            return Err(format!(
                "project ref {id}: expected {kind}, got {}",
                output.kind
            ));
        }
        Ok(output.asset.clone())
    };
    let input = project.data.root().into_json();
    let mut startup = json!({"profile": if input.pointer("/create/form").and_then(Value::as_str) == Some("2d") {"2d"} else {"3d"}});
    for field in ["tick", "seed", "scene", "saveVersion"] {
        if let Some(value) = input.get(field).filter(|value| !value.is_null()) {
            startup[field] = value.clone();
        }
    }
    if !project.input.scenes.is_empty() {
        startup["scenes"] = json!(
            project
                .input
                .scenes
                .iter()
                .map(|(name, reference)| Ok((name, asset(&reference.id, "scene")?)))
                .collect::<Result<std::collections::BTreeMap<_, _>, String>>()?
        );
    }
    if let Some(look) = &project.metadata.look {
        startup["look"] = json!(match look {
            LookBase::Asset(reference) => asset(&reference.id, "look")?,
            LookBase::Preset(name) => format!("@looks/{name}"),
        });
    }
    if let Some(value) = input
        .pointer("/publish/loading")
        .filter(|value| !value.is_null())
    {
        startup["loading"] = value.clone();
    }
    if let Some(flow) = &project.input.flow {
        let mut value = json!({"dialogue": asset(&flow.source.id, "dialogue")?});
        if let Some(screen) = input
            .pointer("/flow/screen")
            .filter(|value| !value.is_null())
        {
            value["screen"] = screen.clone();
        }
        startup["flow"] = value;
    }
    // The generated host type is the startup contract, not a second handwritten
    // wire struct. Its writer also omits unset fields on both hosts.
    let startup: ProjectStartup = og_api::wire::value(&startup, "project startup")?;
    Ok(json!({
        "name": project.metadata.name,
        "engine": project.metadata.engine,
        "id": project.input.id,
        "input": input,
        "startup": startup.to_json(),
        "publish": crate::publish::normalize(&input["publish"])?,
        "dependencies": project.dependencies,
    }))
}

pub fn run(args: &[&str]) -> std::process::ExitCode {
    let path = match args {
        ["--json"] => Path::new("."),
        [path, "--json"] | ["--json", path] => Path::new(path),
        _ => {
            eprintln!("usage: og project [project] --json");
            return std::process::ExitCode::from(2);
        }
    };
    let result = crate::author_types::root(path)
        .ok_or_else(|| format!("{}: no og-project.json project", path.display()))
        .and_then(|root| snapshot(&root));
    match result {
        Ok(value) => {
            println!("{value}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("og project: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// The engine package that runs games: `OG_ENGINE_PACKAGE` (what the `og`
/// launcher and the platform's runner set: a dist's `engine/`), the
/// project's link to it ([`LINK`]) or one in a `node_modules` above it
/// (an engine checkout's own), or `packages/engine` of the origin-engine
/// checkout `og` runs in or from. A game found with a package that no
/// `node_modules` above it resolves gets its link pointed at it
/// ([`link_engine`]).
pub fn engine_package(project: Option<&Path>) -> Option<PathBuf> {
    let ok = |p: &Path| p.join("src/node/run.ts").is_file();
    let found = |p: PathBuf| {
        if let Some(root) = project {
            link_engine(root, &p);
        }
        Some(p)
    };
    if let Ok(p) = std::env::var("OG_ENGINE_PACKAGE") {
        return found(PathBuf::from(p));
    }
    if let Some(root) = project {
        // Through the link to where it points: tsrs is looked for beside the package.
        if let Ok(p) = root.join(LINK).canonicalize()
            && ok(&p)
        {
            return Some(p);
        }
        // A `node_modules` above the project already resolves it (an
        // engine checkout's own link): no link of the game's.
        for dir in root.ancestors() {
            if let Ok(p) = dir.join(LINK).canonicalize()
                && ok(&p)
            {
                return Some(p);
            }
        }
        if let Some(repo) = crate::find_root(root) {
            let p = repo.join("packages/engine");
            if ok(&p) {
                return found(p);
            }
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let repo = crate::find_root(&cwd).or_else(|| {
        let exe = std::env::current_exe().ok()?;
        crate::find_root(&exe)
    })?;
    // The checkout og runs in or from (a game outside it, run by a checkout's
    // og): linked like any other package, or the game's type check cannot
    // resolve `@origingame/engine`.
    let p = repo.join("packages/engine");
    if ok(&p) { found(p) } else { None }
}

/// The link beside a game to the engine package og runs it with, where a
/// package manager would install it: TypeScript (`moduleResolution
/// "bundler"`), the bundler and an editor resolve `@origingame/engine` and
/// its subpaths through it, by the package's `exports`, so package.json is
/// the one map from specifier to file (docs/release.md §1). Ignored by Git
/// as `node_modules/`.
pub const LINK: &str = "node_modules/@origingame/engine";

/// Points `<root>/node_modules/@origingame/engine` at `package` when `root`
/// is a game (`og-project.json`), no `node_modules` above it resolves that
/// package, and the link is missing or points elsewhere: a symlink,
/// a junction on Windows. A directory there (not a link) is left alone.
/// Best effort: a game without the link still runs (the bundler resolves
/// the engine from inside the package); its type check needs it.
pub fn link_engine(root: &Path, package: &Path) {
    if !root.join(source_metadata::FILE).is_file() {
        return;
    }
    let link = root.join(LINK);
    let Ok(target) = package.canonicalize() else {
        return;
    };
    // A `node_modules` above the game that resolves this package already
    // serves it (an engine checkout's own link).
    if root
        .ancestors()
        .skip(1)
        .any(|d| d.join(LINK).canonicalize().ok().as_deref() == Some(target.as_path()))
    {
        return;
    }
    match std::fs::symlink_metadata(&link) {
        Ok(m) if m.file_type().is_symlink() || is_junction(&link) => {
            if link.canonicalize().ok().as_deref() == Some(target.as_path()) {
                return;
            }
            if std::fs::remove_file(&link)
                .or_else(|_| std::fs::remove_dir(&link))
                .is_err()
            {
                return;
            }
        }
        Ok(_) => return,
        Err(_) => {}
    }
    if let Some(dir) = link.parent()
        && std::fs::create_dir_all(dir).is_err()
    {
        return;
    }
    #[cfg(unix)]
    let _ = std::os::unix::fs::symlink(&target, &link);
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_dir(&target, &link).or_else(|_| {
        Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .output()
            .map(|_| ())
    });
}

/// Whether `p` is a Windows junction (a mount-point reparse point, which
/// `symlink_metadata` does not call a symlink).
fn is_junction(p: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const REPARSE_POINT: u32 = 0x400;
        std::fs::symlink_metadata(p).is_ok_and(|m| m.file_attributes() & REPARSE_POINT != 0)
    }
    #[cfg(not(windows))]
    {
        let _ = p;
        false
    }
}

/// The installed engine package's version: `version` of its package.json.
pub fn package_version(package: &Path) -> Option<String> {
    let text = std::fs::read_to_string(package.join("package.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v["version"].as_str().map(str::to_string)
}

/// A version's numbers (major, minor, patch); a pre-release suffix is
/// left out. `None` for anything else, a dev build `dev.<date>.<n>` too.
pub fn semver(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim().trim_start_matches('v').split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u64>().ok());
    let r = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(r)
}

/// A dev build `dev.<YYYYMMDD>.<n>` (docs/release.md §1a): its date and n.
pub fn dated_dev(v: &str) -> Option<(u64, u64)> {
    let mut it = v.strip_prefix("dev.")?.split('.');
    let (date, n) = (it.next()?, it.next()?);
    (it.next().is_none()
        && date.len() == 8
        && date.bytes().all(|c| c.is_ascii_digit())
        && !n.is_empty()
        && n.bytes().all(|c| c.is_ascii_digit()))
    .then(|| Some((date.parse().ok()?, n.parse().ok()?)))?
}

/// Whether `v` is a dev build: `dev.<date>.<n>`, or `x.y.z-dev.n` of the
/// scheme before 0.9.0.
pub fn is_dev(v: &str) -> bool {
    v.starts_with("dev.") || v.contains("-dev.")
}

/// Where an engine version stands among the others. A dev build
/// `dev.<date>.<n>` stands after the release it is based on (`release`)
/// and before the next, by date then n: its own numbers never compare
/// with a release's. `x.y.z-dev.n` stands before the release x.y.z.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order {
    /// The release's numbers, a dated dev build's base release's.
    pub release: (u64, u64, u64),
    rank: u8,
    date: u64,
    n: u64,
}

/// `v`'s order, `base` the release a dated dev build is based on (see
/// [`base_of`]); `None` when `v` is no version or a dated dev build's base
/// is unknown.
pub fn order(v: &str, base: Option<&str>) -> Option<Order> {
    if let Some((date, n)) = dated_dev(v) {
        let b = base?;
        let release = semver(b).filter(|_| !b.contains(['-', '+']))?;
        return Some(Order {
            release,
            rank: 2,
            date,
            n,
        });
    }
    let release = semver(v)?;
    let (rank, n) = match v.trim().split_once('-') {
        None => (1, 0),
        Some((_, pre)) => (
            0,
            pre.strip_prefix("dev.")
                .and_then(|n| n.parse().ok())
                .unwrap_or(0),
        ),
    };
    Some(Order {
        release,
        rank,
        date: 0,
        n,
    })
}

/// The release a dated dev build `v` is based on: the newest version in
/// the changelog.json of the engine package at `package` when that is `v`,
/// else the `build.base` of `v`'s installed dist in the engine cache
/// (`$ORIGIN_ENGINE_CACHE`, default `~/.cache/origin-engine`, where the og
/// launcher installs every version). `None` for any other version.
pub fn base_of(v: &str, package: &Path) -> Option<String> {
    dated_dev(v)?;
    let json = |p: PathBuf| -> Option<serde_json::Value> {
        serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
    };
    if package_version(package).as_deref() == Some(v) {
        return json(package.join("changelog.json"))?["versions"][0]["version"]
            .as_str()
            .map(str::to_string);
    }
    let cache = std::env::var_os("ORIGIN_ENGINE_CACHE")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|h| PathBuf::from(h).join(".cache/origin-engine"))
        })?;
    std::fs::read_dir(cache.join("engines").join(v))
        .ok()?
        .flatten()
        .find_map(|t| {
            json(t.path().join("origin-engine.json"))?["build"]["base"]
                .as_str()
                .map(str::to_string)
        })
}

/// Whether a game pinned at `pinned` runs as written on `installed`, the
/// version of the engine package at `package` (docs/release.md §3): the same
/// minor (major too), not older, and no version after the pin up to the
/// installed one carries a breaking entry in the package's changelog.json
/// (a patch may carry breaks). A dev build `dev.<date>.<n>` compares as
/// the release it is based on, its own changes after it; a dev build
/// counts its `unreleased` entry as its own. `Err` says why not, naming
/// the breaking versions crossed.
pub fn compatible(pinned: &str, installed: &str, package: &Path) -> Result<(), String> {
    if pinned == installed {
        return Ok(());
    }
    let p = order(pinned, base_of(pinned, package).as_deref()).ok_or_else(|| {
        if dated_dev(pinned).is_some() {
            format!("the release {pinned} is based on is unknown: it is not installed")
        } else {
            "not a version".into()
        }
    })?;
    let i = order(installed, base_of(installed, package).as_deref()).ok_or("not a version")?;
    if (p.release.0, p.release.1) != (i.release.0, i.release.1) {
        return Err("another minor".into());
    }
    if i < p {
        return Err("older".into());
    }
    let log: serde_json::Value = std::fs::read_to_string(package.join("changelog.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or("the engine package has no readable changelog.json")?;
    let breaks = |e: &serde_json::Value| e["breaking"].as_array().is_some_and(|b| !b.is_empty());
    let mut crossed: Vec<&str> = log["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|e| {
            breaks(e)
                && e["version"]
                    .as_str()
                    .and_then(|v| order(v, None))
                    .is_some_and(|v| v > p && v <= i)
        })
        .filter_map(|e| e["version"].as_str())
        .collect();
    crossed.reverse();
    if is_dev(installed) && breaks(&log["unreleased"]) {
        crossed.push(installed);
    }
    match crossed.as_slice() {
        [] => Ok(()),
        [v] => Err(format!("{v} carries breaking changes")),
        vs => Err(format!("{} carry breaking changes", vs.join(", "))),
    }
}

/// This og's engine version: a dev build's crates carry it as build
/// metadata of the release it is based on (`0.9.0+dev.20261119.1`,
/// scripts/release-bump.ts), a release's is the crate version.
pub fn own_version() -> &'static str {
    let v = env!("CARGO_PKG_VERSION");
    match v.split_once('+') {
        Some((_, dev)) if dev.starts_with("dev.") => dev,
        _ => v,
    }
}

/// The release this og is or is based on.
pub fn own_release() -> &'static str {
    env!("CARGO_PKG_VERSION")
        .split('+')
        .next()
        .unwrap_or_default()
}

/// What `og run`, `og test`, `og build` and `og dev` print to
/// stderr when the project pins another engine version than the installed
/// package (docs/release.md §3): the installed package runs; the pin is the
/// author's record of what the game was written against, and moving it is
/// an explicit edit (`og upgrade`). `None` when they agree, when the
/// installed one is compatible, or either is unknown.
pub fn pin_notice(project: &Project, package: &Path) -> Option<String> {
    let pinned = project.engine.as_deref()?;
    let installed = package_version(package)?;
    if pinned == installed {
        return None;
    }
    let why = compatible(pinned, &installed, package).err()?;
    Some(format!(
        "og-project.json pins engine \"{pinned}\" and og runs engine {installed}, not compatible ({why}): {installed} runs; `og upgrade --to {installed}` moves the game to it, applies the changelog's codemods and lists what to edit, or run the pinned version through the og launcher (docs/release.md §1)"
    ))
}

pub const NO_ENGINE: &str = "no engine package: run og through its launcher (curl -fsSL https://origingame.ai/downloads/engine/install.sh | sh), which runs the version og-project.json pins, set OG_ENGINE_PACKAGE to an engine (a dist's engine/, or packages/engine of an origin-engine checkout), or run inside a checkout";

/// `bun <script> <args>` with the script resolved in the engine package.
pub fn bun(package: &Path, script: &str) -> Command {
    let mut c = Command::new("bun");
    c.env(COOK_FORMATS, cook_formats());
    if let Ok(binary) = std::env::current_exe() {
        c.env("OG_BIN", binary);
    }
    c.arg(package.join(script));
    c
}

/// The variable carrying this og's cooked-format stamp to the engine's
/// scripts ([`bun`] sets it): the native host loader (src/node/index.ts)
/// compares it with the host library's own and names a host built from
/// other sources, which would refuse every blob this og cooks.
pub const COOK_FORMATS: &str = "OG_COOK_FORMATS";

/// This og's cooked-format stamp, as the host library reports its own
/// (`og_node_formats`).
pub fn cook_formats() -> String {
    format!("{:016x}", og_cook::format::format_stamp())
}

pub const NO_BUN: &str = "cannot start bun; install it: curl -fsSL https://bun.sh/install | bash";

/// A command owns its bundle until its last run completes. Another command
/// cannot replace it while it waits for the resident host.
pub struct NativeBundle(PathBuf);

impl std::ops::Deref for NativeBundle {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for NativeBundle {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for NativeBundle {
    fn drop(&mut self) {
        if let Some(dir) = self.0.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

pub fn bundle_native(project: &Project, package: &Path) -> Result<NativeBundle, String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[cfg_attr(not(unix), expect(unused_mut))]
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    let dir = loop {
        let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("og-bundle-{}-{serial}", std::process::id()));
        match builder.create(&dir) {
            Ok(()) => break dir,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{}: {error}", dir.display())),
        }
    };
    let bundle = NativeBundle(dir.join("game.native.js"));
    bundle_native_to(project, package, &bundle)?;
    Ok(bundle)
}

/// The import graph the bundler wrote beside the bundle at `out`
/// (`<out>.graph.json`, og-cook's `CodeGraph`).
pub fn graph_of(out: &Path) -> PathBuf {
    let mut name = out.as_os_str().to_owned();
    name.push(".graph.json");
    PathBuf::from(name)
}

/// Only a live watch session replaces its own bundle.
pub fn bundle_native_to(project: &Project, package: &Path, out: &Path) -> Result<(), String> {
    let cache = crate::bundle_cache::BundleCache::open(project, package, out);
    if cache.as_ref().is_some_and(|cache| cache.restore(out)) {
        return Ok(());
    }
    let mut command = bun(package, "src/node/build.ts");
    command
        .arg("--entry")
        .arg(&project.entry)
        .arg("--out")
        .arg(out)
        .arg("--target")
        .arg("native")
        .stdout(std::process::Stdio::null());
    let cache = cache.filter(|cache| cache.configure(&mut command).is_ok());
    let status = command.status().map_err(|e| format!("{NO_BUN} ({e})"))?;
    if !status.success() {
        return Err(format!(
            "{} does not bundle (the error is above)",
            project.entry.display()
        ));
    }
    if let Some(cache) = &cache {
        cache.save(out);
    }
    Ok(())
}

/// The exit code of a child, as og's own.
pub fn exit_of(status: std::process::ExitStatus) -> std::process::ExitCode {
    if status.success() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(status.code().map_or(1, |c| c.clamp(1, 255) as u8))
    }
}
