//! Directory extensions; discovery never evaluates an extension's code.
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub description: String,
    /// Retired: the engine ships no skills (`og api show kits` documents the
    /// bundled kits). Still read, and checked when present, so a game's older
    /// pack keeps loading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub entry: Option<String>,
    pub types: Option<String>,
    /// A vfx pack's catalog fields (the asset library reads them; the engine
    /// checks their shape): the lowest engine release it needs (x.y.z), its
    /// tags and its lowercase search words.
    #[serde(rename = "minEngine", default, skip_serializing_if = "Option::is_none")]
    pub min_engine: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<VfxTags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    /// A font pack's ISO 15924 scripts (`Arab`, `Hans`), the language tags
    /// it is the face of, and its faces' files, first first (docs/ui.md
    /// §3.4). Only a font pack has them, and it has all three; only its
    /// languages may be empty, for a pack of no language (`emoji`: `Zsye`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faces: Option<Vec<String>>,
}

/// A vfx pack's tags: its style and dimension, and the union of its
/// effects' categories and elements.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct VfxTags {
    pub style: String,
    pub dim: String,
    #[serde(default)]
    pub category: Vec<String>,
    #[serde(default)]
    pub element: Vec<String>,
}

/// A pack's previews (sheets, clips, cost files): stored with the pack,
/// never installed into a game.
const PREVIEW_DIR: &str = "preview";

fn semver(v: &str) -> bool {
    v.split('.').count() == 3
        && v.split('.')
            .all(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()))
}

/// Whether x.y.z version `a` is newer than `b`.
fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    parts(a) > parts(b)
}

fn lower_word(w: &str) -> bool {
    !w.trim().is_empty() && !w.chars().any(|c| c.is_uppercase())
}

#[derive(Debug, serde::Serialize)]
pub struct Extension {
    pub manifest: Manifest,
    pub directory: PathBuf,
    pub installed: bool,
    pub documents: Vec<String>,
}

fn relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

pub(super) fn files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let ty = entry.file_type().map_err(|e| e.to_string())?;
            if ty.is_symlink() {
                return Err(format!(
                    "{}: extension symlinks are not supported",
                    path.display()
                ));
            }
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if ty.is_dir() {
                walk(root, &path, out)?;
            } else if ty.is_file() {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
            } else {
                return Err(format!("{}: expected an ordinary file", path.display()));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

pub fn read(directory: &Path, installed: bool) -> Result<Extension, String> {
    if std::fs::symlink_metadata(directory)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err(format!(
            "{}: extension symlinks are not supported",
            directory.display()
        ));
    }
    let path = directory.join("extension.json");
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))?;
    let m = &manifest;
    if m.format != 1
        || !matches!(m.kind.as_str(), "kit" | "style" | "vfx" | "font")
        || !m.name.starts_with(|c: char| c.is_ascii_lowercase())
        || !m
            .name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        || !semver(&m.version)
        || m.description.trim().is_empty()
        || m.entry.is_some() != m.types.is_some()
        || (m.kind != "kit" && m.entry.is_some())
        || (m.kind != "vfx" && (m.min_engine.is_some() || m.tags.is_some() || m.keywords.is_some()))
        || m.min_engine.as_deref().is_some_and(|v| !semver(v))
        || m.tags.as_ref().is_some_and(|t| {
            !matches!(t.style.as_str(), "stylised" | "realistic" | "pixel")
                || !matches!(t.dim.as_str(), "2d" | "3d")
                || !t.category.iter().chain(&t.element).all(|w| lower_word(w))
        })
        || m.keywords
            .as_ref()
            .is_some_and(|k| !k.iter().all(|w| lower_word(w)))
        || (m.kind == "font") != (m.scripts.is_some() || m.languages.is_some() || m.faces.is_some())
        || (m.kind == "font"
            && (m.languages.is_none()
                || [&m.scripts, &m.faces]
                    .iter()
                    .any(|v| v.as_ref().is_none_or(|v| v.is_empty()))))
        || m.scripts.as_ref().is_some_and(|s| {
            !s.iter().all(|s| {
                s.len() == 4
                    && s.starts_with(|c: char| c.is_ascii_uppercase())
                    && s[1..].bytes().all(|c| c.is_ascii_lowercase())
            })
        })
    {
        return Err(format!(
            "{}: invalid extension manifest (docs/ui.md §5)",
            path.display()
        ));
    }
    let all = files(directory)?;
    for file in m
        .skill
        .as_deref()
        .into_iter()
        .chain(m.entry.as_deref())
        .chain(m.types.as_deref())
        .chain((m.kind == "style").then_some("style.ogt"))
        .chain(m.faces.iter().flatten().map(String::as_str))
    {
        if !relative(file) || !all.iter().any(|p| p == Path::new(file)) {
            return Err(format!(
                "{}: missing or unsafe extension file `{file}`",
                path.display()
            ));
        }
    }
    let mut documents = Vec::new();
    for file in all
        .iter()
        .filter(|f| f.extension().is_some_and(|e| e == "svelte"))
    {
        let text = std::fs::read_to_string(directory.join(file)).map_err(|e| e.to_string())?;
        let id = file.to_string_lossy().replace('\\', "/");
        og_ui::UiDocument::parse(&id, &text).map_err(|e| e.join("\n"))?;
        documents.push(id);
    }
    Ok(Extension {
        manifest,
        directory: directory.into(),
        installed,
        documents,
    })
}

pub fn discover(project: &Path) -> Result<Vec<Extension>, String> {
    let mut roots = Vec::new();
    if let Some(package) = crate::project::engine_package(Some(project)) {
        roots.push((package.join("extensions"), false));
    }
    if let Some(styles) = og_cook::cook::look::styles_dir() {
        roots.push((styles.to_path_buf(), false));
    }
    if let Some(fonts) = og_cook::cook::look::fonts_dir() {
        roots.push((fonts.to_path_buf(), false));
    }
    roots.push((project.join("assets/ui-kits"), true));
    roots.push((project.join("assets/styles"), true));
    roots.push((project.join("assets/vfx"), true));
    roots.push((project.join("assets/fonts"), true));
    let mut extensions = BTreeMap::new();
    for (root, installed) in roots {
        if !root.exists() {
            continue;
        }
        let mut dirs = std::fs::read_dir(&root)
            .map_err(|e| format!("{}: {e}", root.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        dirs.sort();
        for dir in dirs {
            if !dir.join("extension.json").is_file() {
                continue;
            }
            let extension = read(&dir, installed)?;
            if dir.file_name().and_then(|n| n.to_str()) != Some(&extension.manifest.name) {
                return Err(format!(
                    "{}: directory must match manifest name",
                    dir.display()
                ));
            }
            extensions.insert(
                (
                    extension.manifest.kind.clone(),
                    extension.manifest.name.clone(),
                ),
                extension,
            );
        }
    }
    Ok(extensions.into_values().collect())
}

pub(super) fn install(extension: &Extension, project: &Path) -> Result<PathBuf, String> {
    if !project.join("project.ogt").is_file() {
        return Err(format!("{}: no project.ogt", project.display()));
    }
    let family = match extension.manifest.kind.as_str() {
        "kit" => "ui-kits",
        "style" => "styles",
        "font" => "fonts",
        _ => "vfx",
    };
    let target = project
        .join("assets")
        .join(family)
        .join(&extension.manifest.name);
    if target.exists() {
        return Err(format!(
            "{} already exists; nothing written",
            target.display()
        ));
    }
    let all: Vec<PathBuf> = files(&extension.directory)?
        .into_iter()
        .filter(|f| extension.manifest.kind != "vfx" || !f.starts_with(PREVIEW_DIR))
        .collect();
    std::fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
    // Stage next to the destination: a failed copy cannot leave an installed half-kit.
    let staging = target.with_file_name(format!(
        ".{}-{}",
        extension.manifest.name,
        std::process::id()
    ));
    std::fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result = (|| {
        for file in all {
            let dest = staging.join(&file);
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::copy(extension.directory.join(file), dest).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&staging, &target).map_err(|e| e.to_string())?;
        Ok(target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(staging);
    }
    result
}

pub fn add(args: &[&str], kind: &str) -> ExitCode {
    let result = (|| {
        // `--any-engine` installs a pack whose minEngine is newer than this
        // engine: validating a pack on an unreleased build.
        let any_engine = args.contains(&"--any-engine");
        let args: Vec<&str> = args
            .iter()
            .copied()
            .filter(|a| *a != "--any-engine")
            .collect();
        let (name, project) = match args[..] {
            [name] => (name, Path::new(".")),
            [name, project] => (name, Path::new(project)),
            _ => {
                return Err(
                    "usage: og ui|style|vfx add <name|directory> [project] [--any-engine]".into(),
                );
            }
        };
        let found = if Path::new(name).join("extension.json").is_file() {
            Some(read(Path::new(name), false)?)
        } else {
            discover(project)?
                .into_iter()
                .find(|e| e.manifest.name == name && e.manifest.kind == kind)
        };
        // Neither a directory nor a bundled or installed pack: a vfx pack is
        // downloaded from this engine's release, then installed as a local
        // directory is; the download is removed whatever happens.
        if found.is_none() && kind == "vfx" && !Path::new(name).exists() {
            if !project.join("project.ogt").is_file() {
                return Err(format!("{}: no project.ogt", project.display()));
            }
            let download = fetch_vfx(name, project)?;
            let result = read(&download.join(name), false)
                .and_then(|extension| add_extension(&extension, name, kind, project, any_engine));
            let _ = std::fs::remove_dir_all(&download);
            return result;
        }
        let extension = found
            .ok_or_else(|| format!("unknown {kind} extension `{name}` (og ui list names them)"))?;
        add_extension(&extension, name, kind, project, any_engine)
    })();
    outcome(result)
}

/// Downloads vfx pack `name` of this engine's release into a fresh temporary
/// directory, verified and unpacked (`src/node/vfx-pack.ts` in the engine
/// package: the signed SHA256SUMS, the archive's sha256, safe paths only);
/// returns that directory, which holds `<name>/`. Leaves nothing on failure.
fn fetch_vfx(name: &str, project: &Path) -> Result<PathBuf, String> {
    let package = crate::project::engine_package(Some(project)).ok_or(crate::project::NO_ENGINE)?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let download =
        std::env::temp_dir().join(format!("og-vfx-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir(&download).map_err(|e| format!("{}: {e}", download.display()))?;
    let out = crate::project::bun(&package, "src/node/vfx-pack.ts")
        .args(["fetch", name, env!("CARGO_PKG_VERSION")])
        .arg(&download)
        .stdin(std::process::Stdio::null())
        .output();
    let failed = match out {
        Ok(out) if out.status.success() => {
            eprint!("{}", String::from_utf8_lossy(&out.stderr));
            return Ok(download);
        }
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(_) => crate::project::NO_BUN.to_string(),
    };
    let _ = std::fs::remove_dir_all(&download);
    Err(format!(
        "`{name}` is no bundled or installed vfx pack, and downloading it from engine {}'s release failed: {failed}; nothing written",
        env!("CARGO_PKG_VERSION")
    ))
}

fn add_extension(
    extension: &Extension,
    name: &str,
    kind: &str,
    project: &Path,
    any_engine: bool,
) -> Result<(), String> {
    if extension.manifest.kind != kind {
        return Err(format!("`{name}` is not a {kind}"));
    }
    if let Some(min) = extension.manifest.min_engine.as_deref()
        && !any_engine
        && newer(min, env!("CARGO_PKG_VERSION"))
    {
        return Err(format!(
            "`{}` needs engine {min} or newer; this is {} (--any-engine installs it anyway)",
            extension.manifest.name,
            env!("CARGO_PKG_VERSION")
        ));
    }
    let target = install(extension, project)?;
    println!(
        "installed {} {} into {}",
        extension.manifest.name,
        extension.manifest.version,
        target.display()
    );
    Ok(())
}

pub fn list(args: &[&str]) -> ExitCode {
    let result = (|| {
        let (project, json) = match args {
            [] => (".", false),
            ["--json"] => (".", true),
            [p] => (*p, false),
            [p, "--json"] => (*p, true),
            _ => return Err("usage: og ui list [project] [--json]".into()),
        };
        let extensions = discover(Path::new(project))?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&extensions).map_err(|e| e.to_string())?
            );
        } else {
            for e in extensions {
                let m = &e.manifest;
                println!(
                    "{} {} {} ({}) — {}",
                    m.kind,
                    m.name,
                    m.version,
                    if e.installed { "installed" } else { "bundled" },
                    m.description
                );
                if let Some(skill) = &m.skill {
                    println!("  skill: {}", e.directory.join(skill).display());
                }
                for p in [m.entry.as_ref(), m.types.as_ref()].into_iter().flatten() {
                    println!("  {}", e.directory.join(p).display());
                }
                for doc in e.documents {
                    println!("  {}/{}", m.name, doc.trim_end_matches(".svelte"));
                }
            }
        }
        Ok(())
    })();
    outcome(result)
}

/// `og font list`: each bundled and installed font pack, its languages and
/// whether the game has it.
pub fn list_fonts(args: &[&str]) -> ExitCode {
    let result = (|| {
        let project = match args {
            [] => ".",
            [p] => p,
            _ => return Err("usage: og font list [project]".into()),
        };
        for e in discover(Path::new(project))?
            .into_iter()
            .filter(|e| e.manifest.kind == "font")
        {
            let m = &e.manifest;
            println!(
                "{:<20} {} ({}) — {}; {}",
                m.name,
                m.version,
                if e.installed { "installed" } else { "bundled" },
                match m.languages.as_deref().unwrap_or_default() {
                    [] => m.scripts.as_deref().unwrap_or_default().join(" "),
                    l => l.join(" "),
                },
                m.description
            );
        }
        Ok(())
    })();
    outcome(result)
}

pub fn eject(name: &str, project: &Path, force: bool) -> Result<PathBuf, String> {
    if !project.join("project.ogt").is_file() {
        return Err("no project.ogt".into());
    }
    let (kit, doc) = name.split_once('/').ok_or("expected <kit>/<document>")?;
    let extension = discover(project)?
        .into_iter()
        .find(|e| e.manifest.kind == "kit" && e.manifest.name == kit)
        .ok_or_else(|| format!("unknown kit `{kit}`"))?;
    let file = format!("{}.svelte", doc.trim_end_matches(".svelte"));
    if !extension.documents.contains(&file) {
        return Err(format!("unknown document `{name}` (og ui list names them)"));
    }
    let destination = project.join("assets/ui").join(kit);
    let target = destination.join(&file);
    if target.exists() && !force {
        return Err(format!("{} exists (--force overwrites)", target.display()));
    }
    // Keep sibling documents/assets at the same relative paths. Existing game
    // edits of companions stay; --force replaces only the requested document.
    // A kit's code, manifest, skill and review sheet are not a document's
    // companions: `og ui add` installs those.
    let skill = extension.manifest.skill.as_deref().map(Path::new);
    let skill_dir = skill.and_then(Path::parent);
    for companion in files(&extension.directory)? {
        let kit_only = companion == Path::new("extension.json")
            || companion.starts_with("sheet")
            || companion.extension().is_some_and(|e| e == "ts")
            || skill_dir.is_some_and(|d| !d.as_os_str().is_empty() && companion.starts_with(d))
            || skill == Some(companion.as_path());
        if kit_only {
            continue;
        }
        let dest = destination.join(&companion);
        if dest.exists() && dest != target {
            continue;
        }
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(extension.directory.join(companion), dest).map_err(|e| e.to_string())?;
    }
    Ok(target)
}

fn outcome(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og ui extension: {e}");
            ExitCode::FAILURE
        }
    }
}
