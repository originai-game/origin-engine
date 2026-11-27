//! `og materials`: a material library's maps from images an image model
//! draws (og-cook's `cook::materials`).
//!
//! - `og materials brief <library.ts> [--json]` lists the images the
//!   library asks for: each material that states a `brief`, with the art
//!   brief kind and size to request it at and where the drawn image goes
//!   (`<map>.source.png`). It calls nothing: drawing is the platform's.
//! - `og materials make <library.ts> [--json]` makes each map (and normal
//!   map) from its drawn image, rewriting a file only when its bytes
//!   change, and reports each seam before and after against the step
//!   between neighbours inside. An image not drawn yet is listed; the
//!   command exits 1 while any is.
//!
//! Images are relative to the library's owning module's `assets/` directory.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use og_cook::cook::materials::{Request, cache_key, hash, make, requests};
use serde_json::json;

const USAGE: &str =
    "usage: og materials brief <library.ts> [--json] | og materials make <library.ts> [--json]";

/// Checked requests, module assets and project root share cook's discovery.
fn library(path: &str) -> Result<(Vec<Request>, PathBuf, PathBuf), String> {
    use og_cook::cook::{source_content, source_index, source_project, source_schemas};
    let abs = std::path::absolute(path).map_err(|e| format!("{path}: {e}"))?;
    let root = abs
        .ancestors()
        .find(|d| d.join("og-project.json").is_file())
        .ok_or_else(|| format!("{path} is not under a project"))?
        .to_path_buf();
    let file = abs
        .strip_prefix(&root)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    let project = source_project::Project::open(&root).map_err(|e| e.to_string())?;
    let index = source_index::Index::open(&root, &project).map_err(|e| e.to_string())?;
    let entry = index
        .files
        .get(&file)
        .and_then(|id| index.entries.get(id))
        .filter(|e| e.factory == "materials")
        .ok_or_else(|| format!("{path} is not a file.materials declaration"))?;
    let content = source_content::Evaluator::default()
        .evaluate(&root, &project, entry, &Default::default(), None)
        .map_err(|e| e.to_string())?;
    let file: source_schemas::File =
        serde_json::from_value(content.data().root().into_json()).map_err(|e| e.to_string())?;
    let input = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    let assets = root
        .join(&project.modules[&entry.module].root)
        .join("assets");
    Ok((requests(input)?, assets, root))
}

fn brief(path: &str, as_json: bool) -> Result<bool, String> {
    let (rs, _, root) = library(path)?;
    let package = crate::project::engine_package(Some(&root)).ok_or(crate::project::NO_ENGINE)?;
    let script = [
        package.join("knowledge/scripts/brief.ts"),
        package.join("../knowledge/scripts/brief.ts"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .ok_or("engine package has no knowledge/scripts/brief.ts")?;
    let mut list = Vec::new();
    for r in &rs {
        // Use the one public artBrief, including the project's look and
        // accepted style. Never keep a second prompt builder in Rust.
        let span = r.tile[0].max(r.tile[1]);
        let aspect = r
            .tile
            .map(|v| ((v / span * 1_000_000.0).round() as u64).max(1));
        let output = std::process::Command::new("bun")
            .arg(&script)
            .args([r.kind, &r.subject, "--project"])
            .arg(&root)
            .args([
                "--metres",
                &r.tile[0].to_string(),
                "--aspect",
                &format!("{}:{}", aspect[0], aspect[1]),
                "--json",
            ])
            .output()
            .map_err(|e| format!("{}: {e}", crate::project::NO_BUN))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }
        let art: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        if !as_json {
            println!(
                "{} -> assets/{}\n{}\n",
                r.material,
                r.source,
                art["prompt"].as_str().unwrap_or("")
            );
        }
        let request_hash = hash(&serde_json::to_vec(&art).map_err(|e| e.to_string())?);
        list.push(json!({ "material": r.material, "source": r.source, "tile": r.tile, "requestHash": request_hash, "brief": art }));
    }
    if as_json {
        println!("{}", json!({ "requests": list }));
    } else if rs.is_empty() {
        println!("og materials: no material states a brief");
    }
    Ok(true)
}

/// Walk only within assets/, never through a directory or file symlink.
fn asset_path(assets: &Path, id: &str) -> Result<PathBuf, String> {
    let mut path = assets.parent().expect("owning module assets").to_path_buf();
    for part in Path::new("assets").join(id).components() {
        path.push(part);
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(format!(
                "{} is a symlink; material tools never follow asset symlinks",
                path.display()
            ));
        }
    }
    Ok(path)
}

fn make_all(path: &str, as_json: bool) -> Result<bool, String> {
    let (rs, assets, root) = library(path)?;
    let cache = root.join(".og/materials");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let (mut done, mut missing) = (Vec::new(), Vec::new());
    for r in &rs {
        let src = asset_path(&assets, &r.source)?;
        let bytes = match std::fs::read(&src) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing.push(r.source.clone());
                continue;
            }
            Err(e) => return Err(format!("{}: {e}", src.display())),
        };
        let stamp_path = cache.join(format!("{}.json", cache_key(r, &bytes)));
        let stamp: Option<serde_json::Value> = std::fs::read(&stamp_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
        let intact = if let Some(s) = &stamp {
            let mut intact = s["report"].is_object();
            for id in r.destinations() {
                let path = asset_path(&assets, id)?;
                intact &= std::fs::read(&path).is_ok_and(|b| s["hashes"][id] == hash(&b));
            }
            intact
        } else {
            false
        };
        if intact {
            let mut report = stamp.expect("an intact stamp")["report"].clone();
            report["cached"] = true.into();
            report["written"] = json!([]);
            if !as_json {
                println!("{}: cached (outputs verified)", r.material);
            }
            done.push(report);
            continue;
        }
        let m = make(r, &bytes)?;
        let mut written = Vec::new();
        let mut hashes = serde_json::Map::new();
        for (id, data) in &m.files {
            let out = asset_path(&assets, id)?;
            hashes.insert(id.clone(), hash(data).into());
            if std::fs::read(&out).ok().as_deref() != Some(data.as_slice()) {
                if let Some(dir) = out.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
                }
                std::fs::write(&out, data).map_err(|e| format!("{}: {e}", out.display()))?;
                written.push(id.clone());
            }
        }
        if !as_json {
            println!(
                "{}: {} x {} ({:.0} x {:.0} texels per metre), seam {:.4} -> {:.4} (inside {:.4}), {}",
                r.material,
                m.width,
                m.height,
                m.density[0],
                m.density[1],
                m.seam_before,
                m.seam_after,
                m.interior,
                if written.is_empty() {
                    "unchanged".to_string()
                } else {
                    format!("wrote {}", written.join(", "))
                }
            );
        }
        let report = json!({
            "material": r.material, "width": m.width, "height": m.height,
            "density": m.density, "seamBefore": m.seam_before, "seamAfter": m.seam_after,
            "interior": m.interior,
            "files": m.files.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            "written": written,
            "cached": false,
        });
        std::fs::write(
            stamp_path,
            serde_json::to_vec(&json!({ "report": report, "hashes": hashes }))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        done.push(report);
    }
    if as_json {
        println!("{}", json!({ "made": done, "missing": missing }));
    } else {
        for s in &missing {
            println!("missing: assets/{s} (not drawn yet: `og materials brief` lists it)");
        }
    }
    Ok(missing.is_empty())
}

pub fn run(args: &[&str]) -> ExitCode {
    let as_json = args.contains(&"--json");
    let rest: Vec<&str> = args.iter().copied().filter(|a| *a != "--json").collect();
    let result = match rest.as_slice() {
        ["brief", lib] => brief(lib, as_json),
        ["make", lib] => make_all(lib, as_json),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("og materials: {e}");
            ExitCode::from(2)
        }
    }
}
