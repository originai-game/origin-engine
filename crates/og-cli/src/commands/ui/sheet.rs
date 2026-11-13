//! On-demand extension review, never part of the gate.
use super::{Project, declared_screens, extensions, project, spawn};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    for file in extensions::files(from)? {
        let dest = to.join(&file);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(from.join(file), dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The review sheet project of a component group: `name` itself when it
/// is a project directory, else the engine package's `sheets/<name>/`
/// (a checkout's; `sheets/style` is the style packs' sheet, not a group's).
pub(super) fn group_sheet(package: &Path, name: &str) -> Option<PathBuf> {
    if Path::new(name).join("project.ogt").is_file() {
        return Some(PathBuf::from(name));
    }
    let dir = package.join("sheets").join(name);
    (name != "style" && !name.contains(['/', '\\']) && dir.join("project.ogt").is_file())
        .then_some(dir)
}

pub(super) fn sheet(args: &[&str], review: bool) -> Result<(), String> {
    let Some((&name, rest)) = args.split_first().filter(|(n, _)| !n.starts_with('-')) else {
        return Err(
            "usage: og ui sheet <pack|group|sheet-directory> [project] [--out DIR] [--json]".into(),
        );
    };
    let mut root = Path::new(".");
    let mut out = None;
    let mut json = false;
    let mut locales = if review { "game,pseudo,zh,ar" } else { "game" };
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--out" if i + 1 < rest.len() => {
                i += 1;
                out = Some(PathBuf::from(rest[i]));
            }
            "--json" => json = true,
            "--locales" if review && i + 1 < rest.len() => {
                i += 1;
                locales = rest[i];
            }
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            a => root = Path::new(a),
        }
        i += 1;
    }
    let package = project::engine_package(Some(root)).ok_or(project::NO_ENGINE)?;
    let group = group_sheet(&package, name);
    let matches: Vec<_> = if group.is_some() {
        Vec::new()
    } else if Path::new(name).is_dir() {
        vec![extensions::read(Path::new(name), false)?]
    } else {
        extensions::discover(root)?
            .into_iter()
            .filter(|e| e.manifest.name == name)
            .collect()
    };
    if group.is_none() && matches.len() != 1 {
        return Err(format!(
            "`{name}` must identify one pack (og ui list), component group (a sheets/<group> of the engine package) or sheet directory"
        ));
    }
    let extension = matches.first();
    let name = match (&group, extension) {
        (Some(dir), _) => dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sheet")
            .to_owned(),
        (None, Some(e)) => e.manifest.name.clone(),
        (None, None) => unreachable!("checked above"),
    };
    let name = name.as_str();
    let pack = extension.is_some_and(|e| e.manifest.kind == "style");
    let mode = if review { "review" } else { "sheet" };
    let out = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(out.unwrap_or_else(|| root.join(format!("og-out/ui-{mode}")).join(name)));
    let scratch = std::env::temp_dir().join(format!("og-ui-sheet-{}", std::process::id()));
    std::fs::create_dir(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let result = (|| {
        let demo = if let Some(dir) = &group {
            // The group's components are built in: its pages use them by
            // name, as a game's do.
            copy_tree(dir, &scratch)?;
            if declared_screens(&scratch)?.is_empty() {
                return Err(format!(
                    "`{name}` needs a project.ogt with fixed ui screen entries (docs/ui.md §6)"
                ));
            }
            scratch.clone()
        } else if let Some(extension) = extension.filter(|_| pack) {
            let shipped = package.join("ui-sheet");
            let source = if shipped.join("project.ogt").is_file() {
                shipped
            } else {
                package.join("sheets/style")
            };
            for dir in ["assets", "src"] {
                copy_tree(&source.join(dir), &scratch.join(dir))?;
            }
            std::fs::copy(source.join("project.ogt"), scratch.join("project.ogt"))
                .map_err(|e| e.to_string())?;
            extensions::install(extension, &scratch)?;
            scratch.clone()
        } else {
            let extension = extension.expect("checked above");
            copy_tree(&extension.directory, &scratch)?;
            let demo = scratch.join("sheet");
            if !demo.join("project.ogt").is_file() || declared_screens(&demo)?.is_empty() {
                return Err(format!(
                    "`{name}` needs sheet/project.ogt with fixed ui screen entries (docs/ui.md §6)"
                ));
            }
            // Installed as a game installs it, so its pages name the pack's
            // files exactly as a game does.
            extensions::install(extension, &demo)?;
            demo
        };
        if review {
            super::review::install_font_packs(root, &demo, &["--locales".into(), locales.into()])?;
        }
        let p = Project::open(&demo)?;
        crate::cook::cook(&p.root, 0, false)?;
        let bundle = project::bundle_native(&p, &package)?;
        let mut forward = vec![
            "--breakpoints",
            "desktopHd,phonePortrait",
            "--scales",
            "100,150",
            "--locales",
            locales,
            "--settle",
            "2",
        ];
        if pack {
            forward.extend(["--baseline", "--styles", name]);
        }
        if json {
            forward.push("--json");
        }
        let status = spawn(mode, &p, &package, &bundle, &out, &forward)?;
        if status != ExitCode::SUCCESS {
            return Err(format!("renderer exited {status:?}"));
        }
        Ok(())
    })();
    let cleanup = std::fs::remove_dir_all(&scratch).map_err(|e| e.to_string());
    result.and(cleanup)
}

pub(super) fn run(args: &[&str]) -> ExitCode {
    match sheet(args, false) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og ui sheet: {e}");
            ExitCode::FAILURE
        }
    }
}
