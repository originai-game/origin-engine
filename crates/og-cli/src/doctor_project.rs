//! `og doctor --project <dir> [--json]`: a project's diagnostics as data.
//! Reads the cook's report (`og-out/cook-report.json`, written by `og cook
//! --report`) and checks it against the project's budgets (project.ogt's
//! `budgets { }`, read now, so a budget moved since the cook applies), then
//! prints one finding per problem — each with its severity, a code, the
//! asset, the budget, the limit and the value where they apply, and the fix
//! — as lines, or with `--json` as one JSON document. The exit status is 1
//! when any finding is an error (an asset over a budget, an asset that
//! failed to cook, no report), 0 otherwise.

use og_cook::cook::stats::{self, Budgets};
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og doctor --project <dir> [--json]";

fn finding(severity: &str, code: &str, message: String, fix: &str) -> Value {
    json!({ "severity": severity, "code": code, "message": message, "fix": fix })
}

/// The findings for the project at `root`.
pub fn findings(root: &Path) -> Vec<Value> {
    let mut out = Vec::new();
    let budgets = match crate::cook::budgets(root) {
        Ok(b) => b,
        Err(e) => {
            out.push(finding(
                "error",
                "budgets",
                e,
                "fix the budgets block in project.ogt",
            ));
            Budgets::default()
        }
    };
    let path = root.join(crate::cook::COOK_REPORT);
    let report: Value = match std::fs::read_to_string(&path) {
        Ok(t) => match serde_json::from_str(&t) {
            Ok(v) => v,
            Err(e) => {
                out.push(finding(
                    "error",
                    "report-unreadable",
                    format!("{}: {e}", path.display()),
                    "run `og cook --report` in the project",
                ));
                return out;
            }
        },
        Err(_) => {
            out.push(finding(
                "error",
                "no-report",
                format!(
                    "{} has no cook report ({})",
                    root.display(),
                    crate::cook::COOK_REPORT
                ),
                "run `og cook --report` in the project",
            ));
            return out;
        }
    };
    // A cook after the report: its numbers may be old.
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    if let (Some(r), Some(c)) = (
        modified(&path),
        modified(&root.join("cooked").join(og_cook::runtime::MANIFEST)),
    ) && c > r
    {
        out.push(finding(
            "warning",
            "stale-report",
            "cooked/ changed after the cook report was written".into(),
            "run `og cook --report` again",
        ));
    }
    for a in report["assets"].as_array().into_iter().flatten() {
        let source = a["source"].as_str().unwrap_or("");
        if let Some(e) = a["error"].as_str() {
            let mut f = finding(
                "error",
                "cook-failed",
                format!("{source}: {e}"),
                "fix the source asset",
            );
            f["asset"] = json!(source);
            out.push(f);
        }
        for u in a["unread"].as_array().into_iter().flatten() {
            let mut f = finding(
                "warning",
                "unreadable-output",
                u.as_str().unwrap_or("").to_string(),
                "cook again; if it persists the cooked blob is damaged",
            );
            f["asset"] = json!(source);
            out.push(f);
        }
        for fl in stats::asset_flags(a, &budgets) {
            let mut f = finding(
                "error",
                "over-budget",
                fl["message"].as_str().unwrap_or("").to_string(),
                fix_for(fl["budget"].as_str().unwrap_or("")),
            );
            f["asset"] = json!(source);
            for k in ["budget", "limit", "value"] {
                f[k] = fl[k].clone();
            }
            out.push(f);
        }
    }
    // Scenes that draw unbaked (static models or light volumes without a
    // matching bake).
    for (scene, reason) in crate::bake_lightmaps::status(root) {
        let mut f = finding(
            "warning",
            "unbaked",
            format!("{reason}: it renders unbaked"),
            "run `og bake lightmaps` in the project",
        );
        f["asset"] = json!(scene);
        out.push(f);
    }
    // Baked reflection probes whose room changed since (`og bake probes`).
    let code = crate::bake_probes::code_hash(root);
    for (probe, reason) in og_cook::cook::probes::status(root, code) {
        let mut f = finding(
            "warning",
            "stale-probe",
            format!("{reason}: it draws its own capture on first use"),
            "run `og bake probes` in the project again, with the commands that reach the probe",
        );
        f["asset"] = json!(probe);
        out.push(f);
    }
    for fl in stats::total_flags(&report, &budgets) {
        let mut f = finding(
            "error",
            "over-budget",
            fl["message"].as_str().unwrap_or("").to_string(),
            fix_for("totalBytes"),
        );
        for k in ["budget", "limit", "value"] {
            f[k] = fl[k].clone();
        }
        out.push(f);
    }
    out
}

fn fix_for(budget: &str) -> &'static str {
    match budget {
        "triangles" | "vertices" => {
            "decimate the model in its source, or raise budgets.triangles/vertices in project.ogt"
        }
        "textureSize" => "resize the image in its source, or raise budgets.textureSize",
        "bones" => "reduce the skin's joints in its source, or raise budgets.bones",
        "clips" => "split the clips across models, or raise budgets.clips",
        "assetBytes" => "reduce the asset's content, or raise budgets.assetBytes",
        "soundSeconds" => "shorten the sound in its source, or raise budgets.soundSeconds",
        "videoSeconds" => "shorten the video in its source, or raise budgets.videoSeconds",
        "fontGlyphs" => {
            "narrow the font's charset in its .ogt (only the scripts the game shows), or raise budgets.fontGlyphs"
        }
        "navPolygons" => {
            "raise the navmesh's cellSize or maxError, narrow its bounds, or raise budgets.navPolygons"
        }
        "sceneEntities" => "split the scene, or raise budgets.sceneEntities",
        "emitterParticles" => {
            "lower the emitter's max_particles (or its rate), or raise budgets.emitterParticles"
        }
        _ => "reduce the project's assets, or raise budgets.totalBytes",
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut dir = None;
    let mut as_json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--project" => {
                let Some(d) = args.get(i + 1) else {
                    eprintln!("og doctor: --project needs a directory. {USAGE}");
                    return ExitCode::from(2);
                };
                dir = Some(d.to_string());
                i += 1;
            }
            "--json" => as_json = true,
            a => {
                eprintln!("og doctor: unexpected argument `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(dir) = dir else {
        eprintln!("og doctor: {USAGE}");
        return ExitCode::from(2);
    };
    let root = Path::new(&dir);
    let found = findings(root);
    let errors = found.iter().filter(|f| f["severity"] == "error").count();
    if as_json {
        let doc = json!({
            "project": root.display().to_string(),
            "report": crate::cook::COOK_REPORT,
            "errors": errors,
            "warnings": found.len() - errors,
            "findings": found,
        });
        println!("{doc}");
    } else {
        println!("og doctor --project {}", root.display());
        if found.is_empty() {
            println!("ok          every asset within the project's budgets");
        }
        for f in &found {
            println!(
                "{:<11} {}\n            fix: {}",
                f["severity"].as_str().unwrap_or(""),
                f["message"].as_str().unwrap_or(""),
                f["fix"].as_str().unwrap_or("")
            );
        }
    }
    if errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
