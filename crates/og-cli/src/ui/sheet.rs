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

fn sheet(args: &[&str]) -> Result<(), String> {
    let Some((&name, rest)) = args.split_first().filter(|(n, _)| !n.starts_with('-')) else {
        return Err("usage: og ui sheet <pack|kit> [project] [--out DIR] [--json]".into());
    };
    let mut root = Path::new(".");
    let mut out = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--out" if i + 1 < rest.len() => {
                i += 1;
                out = Some(PathBuf::from(rest[i]));
            }
            "--json" => json = true,
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            a => root = Path::new(a),
        }
        i += 1;
    }
    let package = project::engine_package(Some(root)).ok_or(project::NO_ENGINE)?;
    let matches: Vec<_> = extensions::discover(root)?
        .into_iter()
        .filter(|e| e.manifest.name == name)
        .collect();
    if matches.len() != 1 {
        return Err(format!("`{name}` must identify one extension (og ui list)"));
    }
    let extension = &matches[0];
    let pack = extension.manifest.kind == "style";
    let out = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(out.unwrap_or_else(|| root.join("og-out/ui-sheet").join(name)));
    let scratch = std::env::temp_dir().join(format!("og-ui-sheet-{}", std::process::id()));
    std::fs::create_dir(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let result = (|| {
        let demo = if pack {
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
            copy_tree(&extension.directory, &scratch)?;
            let demo = scratch.join("sheet");
            if !demo.join("project.ogt").is_file() || declared_screens(&demo)?.is_empty() {
                return Err(format!(
                    "kit `{name}` needs sheet/project.ogt with fixed ui screen entries (docs/ui.md §6)"
                ));
            }
            // Installed as a game installs it, so its pages load the kit's documents from
            // `ui-kits/<name>/` exactly as a game does.
            extensions::install(extension, &demo)?;
            demo
        };
        let p = Project::open(&demo)?;
        crate::cook::cook(&p.root, 0, false)?;
        let bundle = project::bundle_native(&p, &package)?;
        let mut forward = vec![
            "--breakpoints",
            "desktop,phonePortrait",
            "--scales",
            "100,150",
            "--locales",
            "game",
            "--settle",
            "2",
        ];
        if pack {
            forward.extend(["--baseline", "--styles", name]);
        }
        if json {
            forward.push("--json");
        }
        let status = spawn("sheet", &p, &package, &bundle, &out, &forward)?;
        if status != ExitCode::SUCCESS {
            return Err(format!("renderer exited {status:?}"));
        }
        Ok(())
    })();
    let cleanup = std::fs::remove_dir_all(&scratch).map_err(|e| e.to_string());
    result.and(cleanup)
}

pub(super) fn run(args: &[&str]) -> ExitCode {
    match sheet(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og ui sheet: {e}");
            ExitCode::FAILURE
        }
    }
}
