//! `og dev [project] [--port P] [--exit-after N] [--json]`: the edit loop.
//!
//! Cooks and bundles, then runs the game paced by the wall clock with the
//! probe on `--port` (default 47800) and the bundle watched. A change under
//! `src/` rebuilds the bundle; a change under `assets/` re-cooks (only the
//! changed sources cook) and rebuilds it. The running game sees the new
//! bundle, re-imports it and hot-reloads through the probe's `reload`
//! (snapshot, swap, re-register by name, restore), so world state is kept.
//! Each reload prints its latency from the save to the first frame of the new
//! bundle against the 300 ms budget (design §7), split into rebuild (save to
//! bundle written) and swap (bundle written to first new frame).
//! `--exit-after N` stops after N reloads (for scripted measurements).

use crate::project::{self, Project};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};

pub const BUDGET_MS: f64 = 300.0;
const USAGE: &str = "usage: og dev [project] [--port P] [--exit-after N] [--json]";

fn snapshot(dir: &Path, out: &mut BTreeMap<PathBuf, SystemTime>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for e in read.flatten() {
        let p = e.path();
        let name = e.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if p.is_dir() {
            snapshot(&p, out);
        } else if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
            out.insert(p, m);
        }
    }
}

fn changed(
    before: &BTreeMap<PathBuf, SystemTime>,
    now: &BTreeMap<PathBuf, SystemTime>,
) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = now
        .iter()
        .filter(|(p, m)| before.get(*p) != Some(m))
        .map(|(p, _)| p.clone())
        .collect();
    out.extend(before.keys().filter(|p| !now.contains_key(*p)).cloned());
    out
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut port = crate::probe::DEFAULT_PORT.to_string();
    let mut exit_after: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--port" | "--exit-after" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og dev: {} needs a value. {USAGE}", args[i]);
                    return ExitCode::from(2);
                };
                if args[i] == "--port" {
                    port = v.to_string();
                } else {
                    exit_after = v.parse().ok();
                    if exit_after.is_none() {
                        eprintln!("og dev: --exit-after takes a number. {USAGE}");
                        return ExitCode::from(2);
                    }
                }
                i += 1;
            }
            "--json" => json = true,
            a if a.starts_with('-') => {
                eprintln!("og dev: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a.to_string(),
        }
        i += 1;
    }
    let p = match Project::open(Path::new(&project)) {
        Ok(p) if p.manifest => p,
        Ok(p) => {
            eprintln!(
                "og dev: {} has no project.ogt; og dev watches a project",
                p.root.display()
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("og dev: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og dev: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let bundle = match crate::run::prepare(&p, &package) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og dev: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut child = match project::bun(&package, "src/node/run.ts")
        .arg("--entry")
        .arg(&bundle)
        .arg("--root")
        .arg(&p.root)
        .arg("--watch")
        .arg(&bundle)
        .arg("--port")
        .arg(&port)
        .arg("--realtime")
        .arg("--out")
        .arg(p.root.join("og-out"))
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("og dev: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    };
    let (tx, rx) = mpsc::channel::<String>();
    let stdout = child.stdout.take().expect("piped");
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let watched = [p.root.join("src"), p.root.join("assets")];
    let take = || {
        let mut m = BTreeMap::new();
        for d in &watched {
            snapshot(d, &mut m);
        }
        m
    };
    let mut seen = take();
    let mut reloads = 0usize;
    let mut worst = 0f64;
    // The rebuild time (save seen to bundle written) of each reload in flight.
    let mut pending: Vec<(Instant, f64)> = Vec::new();
    if !json {
        println!(
            "og dev: watching src/ and assets/ of {}; reloads print below (budget {BUDGET_MS} ms)",
            p.root.display()
        );
    }
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            eprintln!("og dev: the game stopped ({status})");
            return ExitCode::FAILURE;
        }
        while let Ok(line) = rx.try_recv() {
            let parsed: Option<serde_json::Value> = serde_json::from_str(&line).ok();
            match parsed.filter(|v| v["event"] == "reload") {
                Some(v) => {
                    reloads += 1;
                    let swap = v["latencyMs"].as_f64().unwrap_or(f64::NAN);
                    let rebuild = if pending.is_empty() {
                        0.0
                    } else {
                        pending.remove(0).1
                    };
                    let total = rebuild + swap;
                    worst = worst.max(total);
                    let ok = v["ok"].as_bool().unwrap_or(false);
                    let within = total <= BUDGET_MS;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "event": "reload", "n": reloads, "ok": ok,
                                "rebuildMs": (rebuild * 10.0).round() / 10.0,
                                "swapMs": swap, "totalMs": (total * 10.0).round() / 10.0,
                                "budgetMs": BUDGET_MS, "withinBudget": within,
                                "report": v["report"], "tick": v["tick"],
                            })
                        );
                    } else {
                        println!(
                            "reload {reloads}: {} in {total:.0} ms (rebuild {rebuild:.0} ms + swap {swap:.0} ms; budget {BUDGET_MS} ms{})",
                            if ok { "ok" } else { "refused, the old bundle keeps running" },
                            if within { "" } else { ", OVER" }
                        );
                    }
                    if exit_after.is_some_and(|n| reloads >= n) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return if worst <= BUDGET_MS {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::FAILURE
                        };
                    }
                }
                None => println!("{line}"),
            }
        }
        std::thread::sleep(Duration::from_millis(20));
        let now = take();
        let changes = changed(&seen, &now);
        if changes.is_empty() {
            continue;
        }
        seen = now;
        let saved = Instant::now();
        let assets = changes.iter().any(|c| c.starts_with(p.root.join("assets")));
        if assets {
            match crate::cook::cook(&p.root, 0) {
                Ok(r) => eprintln!("{}", crate::cook::summary(&r)),
                Err(e) => {
                    eprintln!("{e}");
                    continue;
                }
            }
        }
        match project::bundle_native(&p, &package) {
            Ok(_) => pending.push((saved, saved.elapsed().as_secs_f64() * 1000.0)),
            Err(e) => eprintln!("og dev: {e}; the game keeps running the last bundle"),
        }
    }
}
