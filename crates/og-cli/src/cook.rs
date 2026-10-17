//! `og cook [project] [--threads N] [--report out.md] [--json]`: assets/ into
//! cooked/ through og-cook's `cook_project`, incrementally by stamp. `og run`,
//! `og build`, `og dev` and `og test` cook first, so a run never reads stale
//! blobs.

use crate::project::Project;
use og_cook::{cook_project, CookOptions, Report};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og cook [project] [--threads N] [--report out.md] [--json]";

/// The engine's own components as the cook packs scene tables with them
/// (`Transform`, `Model`, the lights), by the names scenes use.
pub fn engine_schemas() -> Vec<og_core::Schema> {
    use og_core::reflect::{FieldSchema, FieldType};
    let ty = |column: &str| -> Option<FieldType> {
        Some(match column {
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "u32" => FieldType::U32,
            "i32" => FieldType::I32,
            "Vector2" => FieldType::Vec2,
            "Vector3" | "Color" => FieldType::Vec3,
            "Quaternion" => FieldType::Quat,
            "Entity" | "Geometry" | "Material" | "Asset" => FieldType::U64,
            other => {
                return serde_json::from_value(serde_json::Value::String(
                    match other {
                        "bool" => "bool",
                        "Matrix4" => "mat4",
                        o => o,
                    }
                    .into(),
                ))
                .ok()
            }
        })
    };
    og_api::host::RECORDS
        .iter()
        .filter(|r| r.kind == "component")
        .filter_map(|r| {
            let fields = r
                .fields
                .iter()
                .map(|f| ty(f.column).map(|t| FieldSchema::new(f.name.to_string(), t)))
                .collect::<Option<Vec<_>>>()?;
            Some(og_core::Schema {
                name: r.name.to_string(),
                version: 1,
                doc: String::new(),
                fields,
            })
        })
        .collect()
}

/// Cooks a project; `Err` names the failing assets.
pub fn cook(root: &Path, threads: usize) -> Result<Report, String> {
    let options = CookOptions {
        threads,
        schemas: engine_schemas(),
        ..CookOptions::default()
    };
    let report = cook_project(root, &options).map_err(|e| format!("og cook: {e}"))?;
    if report.ok() {
        Ok(report)
    } else {
        Err(format!(
            "og cook: {} asset(s) failed:\n  {}",
            report.errors().len(),
            report.errors().join("\n  ")
        ))
    }
}

/// One line: what was cooked this time.
pub fn summary(r: &Report) -> String {
    let outputs: Vec<_> = r.assets.iter().flat_map(|a| a.outputs.iter()).collect();
    let cooked = outputs.iter().filter(|o| o.cooked).count();
    let bytes: u64 = outputs.iter().map(|o| o.bytes).sum();
    format!(
        "og cook: {} assets, {} outputs ({cooked} cooked, {} up to date), {bytes} bytes in cooked/, {:.0} ms",
        r.assets.len(),
        outputs.len(),
        outputs.len() - cooked,
        r.elapsed_ms
    )
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut threads = 0;
    let mut report = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--threads" => {
                let Some(n) = args.get(i + 1).and_then(|v| v.parse().ok()) else {
                    eprintln!("og cook: --threads takes a number. {USAGE}");
                    return ExitCode::from(2);
                };
                threads = n;
                i += 1;
            }
            "--report" => {
                let Some(p) = args.get(i + 1) else {
                    eprintln!("og cook: --report takes a path. {USAGE}");
                    return ExitCode::from(2);
                };
                report = Some(p.to_string());
                i += 1;
            }
            "--json" => json = true,
            a if a.starts_with('-') => {
                eprintln!("og cook: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a.to_string(),
        }
        i += 1;
    }
    let p = match Project::open(Path::new(&project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og cook: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !p.has_assets() {
        eprintln!("og cook: {} has no assets/ to cook", p.root.display());
        return ExitCode::FAILURE;
    }
    match cook(&p.root, threads) {
        Ok(r) => {
            if let Some(path) = report {
                if let Err(e) = std::fs::write(&path, r.to_markdown()) {
                    eprintln!("og cook: cannot write {path}: {e}");
                    return ExitCode::FAILURE;
                }
            }
            if json {
                println!("{}", r.to_json());
            } else {
                println!("{}", summary(&r));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
