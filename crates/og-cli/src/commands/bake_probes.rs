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
//! by the probe's position, box and face size and its room's content (the
//! meshes, materials, textures, terrain and lights in its box), so a probe
//! that moves, changes size or whose room changes draws its own capture
//! until it is baked again. Each bake's record (`cooked/probes/<key>.ogt`)
//! names the model assets in its room and, when the game's code built
//! part of it, the source's hash: `og doctor --project` reports the bakes
//! whose assets or code changed since.

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
    let code = code_hash(&p.root);
    let bundle =
        og_cook::runtime::Bundle::open(&p.root.join("cooked")).map_err(|e| e.to_string())?;
    for pr in &probes {
        let key = pr["key"].as_str().ok_or("a probe without a key")?;
        let three = |k: &str| -> [f32; 3] {
            std::array::from_fn(|i| pr[k][i].as_f64().unwrap_or(0.0) as f32)
        };
        let record = og_cook::cook::probes::ProbeRecord {
            position: three("position"),
            size: three("box"),
            room: pr["room"].as_str().unwrap_or("").to_string(),
            commands: commands.clone(),
            assets: pr["assets"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|a| a.as_str())
                .map(|a| (a.to_string(), bundle.output(a).map_or(0, |o| o.hash)))
                .collect(),
            code: pr["built"].as_bool().unwrap_or(false).then_some(code),
        };
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
        let file = og_cook::cook::probes::write_probe(&p.root, key, face, &levels, &record)
            .map_err(|e| e.to_string())?;
        println!("  {key} ({face}²) -> {}", file.display());
    }
    Ok(probes.len())
}

/// The hash of the game's source (every file under `src/`, by path): a
/// bake whose room the code built part of is stale once it changes.
pub fn code_hash(root: &Path) -> u64 {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    let src = root.join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    files.sort();
    let mut h = og_core::hash::StableHasher::new();
    for f in files {
        h.write_str(&f.strip_prefix(&src).unwrap_or(&f).to_string_lossy());
        h.write(&std::fs::read(&f).unwrap_or_default());
    }
    h.finish()
}
