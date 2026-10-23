//! `og bake probes [project] [--commands "t:action[:args];..."] [--at S]
//! [--size WxH]`: renders the project's baked og.ReflectionProbe captures
//! on the native host and writes them under its cook cache
//! (`cooked/probes/<key>.ogb`, og-cook's `cook::probes`), where the
//! runtime loads them instead of drawing them on first use.
//!
//! The project runs headless (`og run`) with the given commands (to reach
//! the scene or station whose probes are to be baked; none: the start
//! scene); at `--at` seconds (default 0.5) the `bake_probes` dump reads
//! back every probe the renderer captured that frame, and each becomes a
//! file. A probe already baked samples its file and is not captured
//! again: delete `cooked/probes/` to bake every probe anew. A bake is keyed
//! by the probe's position, box and face size, so a probe that moves or
//! changes size draws its own capture until it is baked again.

use crate::project::Project;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str =
    "usage: og bake probes [project] [--commands \"t:action[:args];...\"] [--at S] [--size WxH]";

pub fn run(args: &[&str]) -> ExitCode {
    match bake(args) {
        Ok(n) => {
            println!("og bake probes: {n} probe(s) written under cooked/probes/");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("og bake probes: {e}");
            ExitCode::FAILURE
        }
    }
}

fn bake(args: &[&str]) -> Result<usize, String> {
    let mut project = ".";
    let mut commands = String::new();
    let mut at = 0.5f64;
    let mut size = "640x360";
    let mut it = args.iter();
    while let Some(&a) = it.next() {
        let mut value = || {
            it.next()
                .copied()
                .ok_or(format!("{a} takes a value\n{USAGE}"))
        };
        match a {
            "probes" => {}
            "--commands" => commands = value()?.trim_end_matches(';').to_string(),
            "--at" => {
                at = value()?
                    .parse()
                    .map_err(|_| format!("--at takes seconds\n{USAGE}"))?
            }
            "--size" => size = value()?,
            s if s.starts_with('-') => return Err(format!("unknown option {s}\n{USAGE}")),
            s => project = s,
        }
    }
    let p = Project::open(Path::new(project))?;
    let out = std::env::temp_dir().join(format!("og-bake-probes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let script = format!(
        "{}{}{at}:bake_probes:probes.json;{}:quit",
        commands,
        if commands.is_empty() { "" } else { ";" },
        at + 0.05
    );
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let status = std::process::Command::new(exe)
        .arg("run")
        .arg(&p.root)
        .args(["--headless", "--size", size, "--out"])
        .arg(&out)
        .args(["--commands", &script])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("the run failed ({status})"));
    }
    let path = out.join("probes.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: {e} (did the run reach {at} s?)", path.display()))?;
    let _ = std::fs::remove_dir_all(&out);
    let dump: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let probes = dump["probes"].as_array().cloned().unwrap_or_default();
    for pr in &probes {
        let key = pr["key"].as_str().ok_or("a probe without a key")?;
        let face = pr["size"].as_u64().ok_or("a probe without a size")? as u32;
        let levels: Vec<Vec<f32>> = pr["levels"]
            .as_array()
            .ok_or("a probe without levels")?
            .iter()
            .map(|l| {
                l.as_array()
                    .map(|v| v.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect())
                    .unwrap_or_default()
            })
            .collect();
        let file = og_cook::cook::probes::write_probe(&p.root, key, face, &levels)
            .map_err(|e| e.to_string())?;
        println!("  {key} ({face}²) -> {}", file.display());
    }
    Ok(probes.len())
}
