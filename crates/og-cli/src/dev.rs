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
//! `--exit-after N` stops after N reloads (for scripted measurements),
//! once the last type check has landed.
//!
//! Each `src/` change is type-checked (crate::check) beside the rebuild, in
//! a process of its own: the bundle is written and reloaded as soon as it
//! is built, whatever the check finds, so a type error never refuses a
//! reload. The check prints `check: ok (38 ms)` or its diagnostics when it
//! lands (`--json`: a `check` event, the diagnostics in it); the `reload`
//! event carries `check: {ok, ms, errors}` when the check of its source had
//! landed by then, else `null` and the `check` event follows. One check is
//! in flight: a newer change cancels the older one's process. `assets/`
//! changes are not checked; the budget's measure stays rebuild + swap.
//!
//! The game runs in a process group of its own. SIGINT, SIGTERM and SIGHUP
//! to og (Ctrl-C, a supervisor's stop, a closed terminal) are passed on to
//! that whole group (crate::relay), so the bun process and anything it
//! started stop with og; whatever is left after 3 s is killed, and og then
//! ends by the signal it got. A game that stops on its own ends og with
//! exit 1, and `--exit-after` stops the group the same way.

use crate::check::{Checker, Report};
use crate::project::{self, Project};
use crate::relay::{self, Target};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant, SystemTime};

pub const BUDGET_MS: f64 = 300.0;
const USAGE: &str = "usage: og dev [project] [--port P] [--exit-after N] [--json]";

/// Every file's modification time under `dir` (dot files skipped).
pub(crate) fn snapshot(dir: &Path, out: &mut BTreeMap<PathBuf, SystemTime>) {
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

/// The files added, modified or removed between two snapshots.
pub(crate) fn changed(
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

/// The type checks of `og dev`: one in flight, numbered by the source
/// change it checks; a newer one kills the older one's process.
struct Checks {
    checker: Checker,
    target: crate::check::Target,
    checkers: usize,
    /// The number of the newest check started.
    latest: Arc<AtomicUsize>,
    /// The process of the check in flight.
    running: Arc<Mutex<Option<u32>>>,
    tx: mpsc::Sender<(usize, Result<Report, String>)>,
    rx: mpsc::Receiver<(usize, Result<Report, String>)>,
    /// The short form of each check landed, by number.
    landed: BTreeMap<usize, serde_json::Value>,
}

impl Checks {
    fn start(&mut self, n: usize) {
        self.latest.store(n, Ordering::SeqCst);
        if let Some(pid) = self
            .running
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
        {
            kill(pid);
        }
        let (checker, target, checkers) =
            (self.checker.clone(), self.target.clone(), self.checkers);
        let (latest, running, tx) = (self.latest.clone(), self.running.clone(), self.tx.clone());
        std::thread::spawn(move || {
            let t0 = Instant::now();
            let child = match checker.command_for(&target, checkers).spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send((n, Err(checker.spawn_error(e))));
                    return;
                }
            };
            let pid = child.id();
            *running.lock().unwrap_or_else(|p| p.into_inner()) = Some(pid);
            // A newer change came while this one spawned.
            if latest.load(Ordering::SeqCst) != n {
                kill(pid);
            }
            let out = child.wait_with_output();
            {
                let mut r = running.lock().unwrap_or_else(|p| p.into_inner());
                if *r == Some(pid) {
                    *r = None;
                }
            }
            let result = out
                .map_err(|e| checker.spawn_error(e))
                .and_then(|o| checker.report(&target, o, t0.elapsed().as_secs_f64() * 1000.0));
            let _ = tx.send((n, result));
        });
    }

    /// Prints the checks that landed (a superseded one is dropped).
    fn drain(&mut self, json: bool) {
        while let Ok((n, result)) = self.rx.try_recv() {
            if n != self.latest.load(Ordering::SeqCst) {
                continue;
            }
            match result {
                Ok(r) => {
                    self.landed.insert(n, r.brief());
                    if json {
                        println!("{}", r.json());
                    } else {
                        print!("{}", r.raw);
                        println!("check: {}", r.summary().trim_start_matches("og check: "));
                    }
                }
                Err(e) => {
                    self.landed
                        .insert(n, serde_json::json!({ "ok": false, "error": e }));
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({ "event": "check", "ok": false, "error": e })
                        );
                    } else {
                        println!("check: {e}");
                    }
                }
            }
        }
    }

    fn settled(&self) -> bool {
        self.landed
            .contains_key(&self.latest.load(Ordering::SeqCst))
    }
}

fn kill(pid: u32) {
    #[cfg(unix)]
    // SAFETY: kill has no memory effects; the pid is a check og started.
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
    #[cfg(not(unix))]
    let _ = pid;
}

/// What is left of the game's process group once its leader ended.
fn stop_group(group: Target) {
    #[cfg(unix)]
    if let Target::Group(pid) = group {
        // SAFETY: kill has no memory effects; the group is the game's own.
        unsafe { libc::kill(-(pid as i32), libc::SIGKILL) };
    }
    #[cfg(not(unix))]
    let _ = group;
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
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og dev: {n}");
    }
    let bundle = match crate::run::prepare(&p, &package, false, json) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og dev: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut checks = match Checker::find(Some(&package), None)
        .and_then(|(c, note)| Ok((c, note, crate::check::Target::of(&p, Some(&package))?)))
    {
        Ok((checker, note, target)) => {
            if let Some(n) = note {
                eprintln!("og check: {n}");
            }
            let (tx, rx) = mpsc::channel();
            Some(Checks {
                checker,
                target,
                checkers: crate::check::default_checkers(),
                latest: Arc::new(AtomicUsize::new(0)),
                running: Arc::new(Mutex::new(None)),
                tx,
                rx,
                landed: BTreeMap::new(),
            })
        }
        Err(e) => {
            eprintln!("og dev: no type check ({e}); reloads go on unchecked");
            None
        }
    };
    // The first bundle's source, checked while the game starts.
    let mut source = 0usize;
    if let Some(c) = &mut checks {
        c.start(source);
    }
    let mut cmd = project::bun(&package, "src/node/run.ts");
    // Its own process group: a signal to og reaches everything the game started.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let mut child = match cmd
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
    let group = Target::Group(child.id());
    relay::start(group);
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
    // The rebuild time (save seen to bundle written) of each reload in
    // flight, and the source change its bundle holds.
    let mut pending: Vec<(Instant, f64, usize)> = Vec::new();
    // --exit-after was reached: stop once the last check has landed.
    let mut stopping: Option<Instant> = None;
    if !json {
        println!(
            "og dev: watching src/ and assets/ of {}; reloads print below (budget {BUDGET_MS} ms)",
            p.root.display()
        );
    }
    loop {
        if let Some(sig) = relay::received() {
            eprintln!("og dev: signal {sig}, stopping the game");
            relay::finish(group, sig, Duration::from_secs(3), || {
                matches!(child.try_wait(), Ok(Some(_)))
            });
        }
        if let Ok(Some(status)) = child.try_wait() {
            relay::stop();
            stop_group(group);
            eprintln!("og dev: the game stopped ({status})");
            return ExitCode::FAILURE;
        }
        if let Some(c) = &mut checks {
            c.drain(json);
        }
        if let Some(t) = stopping
            && (checks.as_ref().is_none_or(Checks::settled)
                || t.elapsed() > Duration::from_secs(60))
        {
            relay::stop();
            let _ = child.kill();
            let _ = child.wait();
            stop_group(group);
            return if worst <= BUDGET_MS {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        while let Ok(line) = rx.try_recv() {
            let parsed: Option<serde_json::Value> = serde_json::from_str(&line).ok();
            match parsed.filter(|v| v["event"] == "reload") {
                Some(v) => {
                    reloads += 1;
                    let swap = v["latencyMs"].as_f64().unwrap_or(f64::NAN);
                    let (rebuild, of) = if pending.is_empty() {
                        (0.0, source)
                    } else {
                        let (_, r, n) = pending.remove(0);
                        (r, n)
                    };
                    // The check of this bundle's source, when it has landed.
                    let check = checks
                        .as_ref()
                        .and_then(|c| c.landed.get(&of).cloned())
                        .unwrap_or(serde_json::Value::Null);
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
                                "report": v["report"], "tick": v["tick"], "check": check,
                            })
                        );
                    } else {
                        let checked = match (check["ok"].as_bool(), check["errors"].as_u64()) {
                            _ if checks.is_none() => String::new(),
                            (Some(true), _) => format!("; check ok {} ms", check["ms"]),
                            (Some(false), Some(n)) => format!("; check: {n} type error(s)"),
                            (Some(false), None) => "; check failed".into(),
                            _ => "; check pending".into(),
                        };
                        println!(
                            "reload {reloads}: {} in {total:.0} ms (rebuild {rebuild:.0} ms + swap {swap:.0} ms; budget {BUDGET_MS} ms{}){checked}",
                            if ok {
                                "ok"
                            } else {
                                "refused, the old bundle keeps running"
                            },
                            if within { "" } else { ", OVER" }
                        );
                    }
                    if exit_after.is_some_and(|n| reloads >= n) {
                        stopping = Some(Instant::now());
                        break;
                    }
                }
                None => println!("{line}"),
            }
        }
        std::thread::sleep(Duration::from_millis(20));
        if stopping.is_some() {
            continue;
        }
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
                Ok(r) => {
                    eprintln!("{}", crate::cook::summary(&r));
                    for line in crate::cook::cooked_lines(&r) {
                        eprintln!("{line}");
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                    continue;
                }
            }
        }
        // A source change is checked beside the rebuild; assets are not.
        if changes
            .iter()
            .any(|c| !c.starts_with(p.root.join("assets")))
            && let Some(c) = &mut checks
        {
            source += 1;
            c.start(source);
        }
        match project::bundle_native(&p, &package) {
            Ok(_) => pending.push((saved, saved.elapsed().as_secs_f64() * 1000.0, source)),
            Err(e) => eprintln!("og dev: {e}; the game keeps running the last bundle"),
        }
    }
}
