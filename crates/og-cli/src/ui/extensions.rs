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
        || !matches!(m.kind.as_str(), "kit" | "style")
        || !m.name.starts_with(|c: char| c.is_ascii_lowercase())
        || !m
            .name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        || m.version.split('.').count() != 3
        || !m
            .version
            .split('.')
            .all(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()))
        || m.description.trim().is_empty()
        || m.entry.is_some() != m.types.is_some()
        || (m.kind == "style" && m.entry.is_some())
    {
        return Err(format!(
            "{}: invalid extension manifest (docs/ui.md §17.1)",
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
        .filter(|f| f.extension().is_some_and(|e| e == "ogt"))
    {
        let text = std::fs::read_to_string(directory.join(file)).map_err(|e| e.to_string())?;
        if og_core::ogt::Document::parse_header(&text)
            .is_ok_and(|d| matches!(d.ty.as_str(), "screen" | "component"))
        {
            documents.push(file.to_string_lossy().replace('\\', "/"));
        }
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
    roots.push((project.join("assets/ui-kits"), true));
    roots.push((project.join("assets/styles"), true));
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
    let family = if extension.manifest.kind == "kit" {
        "ui-kits"
    } else {
        "styles"
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
    let all = files(&extension.directory)?;
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
        let (name, project) = match args {
            [name] => (*name, Path::new(".")),
            [name, project] => (*name, Path::new(project)),
            _ => return Err("usage: og ui|style add <name|directory> [project]".into()),
        };
        let extension = if Path::new(name).join("extension.json").is_file() {
            read(Path::new(name), false)?
        } else {
            discover(project)?
                .into_iter()
                .find(|e| e.manifest.name == name && e.manifest.kind == kind)
                .ok_or_else(|| {
                    format!("unknown {kind} extension `{name}` (og ui list names them)")
                })?
        };
        if extension.manifest.kind != kind {
            return Err(format!("`{name}` is not a {kind}"));
        }
        let target = install(&extension, project)?;
        println!(
            "installed {} {} into {}",
            extension.manifest.name,
            extension.manifest.version,
            target.display()
        );
        Ok(())
    })();
    outcome(result)
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
                    println!("  {}/{}", m.name, doc.trim_end_matches(".ogt"));
                }
            }
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
    let file = format!("{}.ogt", doc.trim_end_matches(".ogt"));
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
