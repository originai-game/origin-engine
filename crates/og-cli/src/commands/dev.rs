//! `og dev [project] [--port P] [--exit-after N] [--json] [--engine]`: the edit loop.
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
//!
//! `--engine` (an origin-engine checkout, for engine developers; a game
//! author never needs it) also watches the engine's crates: each crate's
//! `src/`, `shaders/`, `data/`, `Cargo.toml` and `build.rs`, and the
//! workspace's `Cargo.toml` and `Cargo.lock`. A change rebuilds the dev addon
//! (`cargo build -p og-host-node -p og-cli`) while the game keeps running; a failed
//! build leaves it running, and a change that did not reach the addon
//! restarts nothing. Otherwise the game hands its world over (SIGUSR2 to
//! run.ts's `--handoff`: the world's save written between two frames) and a
//! new process on the new addon restores it, migrating by name what the
//! schema changed, and goes on from that tick: a hot restart, not a patch of
//! the running code. Each prints its latency (build, handoff, start); with
//! `--exit-after` restarts count with reloads. og itself is not restarted:
//! an og-cli change takes the next `og dev`.
//!
//! `--web [--host H]` serves the development web build instead and pushes
//! each edit to the open page ([`web_dev`], src/node/dev-web.ts,
//! docs/web-export.md §og dev --web).

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
const USAGE: &str = "usage: og dev [project] [--port P] [--exit-after N] [--json] [--engine]\n       og dev [project] --web [--port P] [--host H]";
/// `--web`'s port when none is given.
const WEB_PORT: &str = "47810";
/// How often `--engine` looks at the engine's sources.
const ENGINE_POLL: Duration = Duration::from_millis(250);

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
    fn start(&mut self, n: usize, project: &Project) {
        self.latest.store(n, Ordering::SeqCst);
        if let Some(pid) = self
            .running
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
        {
            kill(pid);
        }
        let (checker, project, checkers) = (self.checker.clone(), project.clone(), self.checkers);
        let (latest, running, tx) = (self.latest.clone(), self.running.clone(), self.tx.clone());
        std::thread::spawn(move || {
            let result = (|| {
                let targets = crate::check::Target::all(&project)?;
                Report::combine(targets.iter().map(|target| {
                    if latest.load(Ordering::SeqCst) != n {
                        return Err("superseded check".into());
                    }
                    let t0 = Instant::now();
                    let child = checker
                        .command_for(target, checkers, None)
                        .spawn()
                        .map_err(|e| checker.spawn_error(e))?;
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
                    let out = out.map_err(|e| checker.spawn_error(e))?;
                    checker.report(target, out, t0.elapsed().as_secs_f64() * 1000.0)
                }))
            })();
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

/// The game's stdout, line by line, into `tx` from a thread of its own.
fn pipe(child: &mut std::process::Child, tx: mpsc::Sender<String>) {
    let stdout = child.stdout.take().expect("piped");
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
}

fn ms(since: Instant) -> f64 {
    (since.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// `--engine`'s state: the checkout it builds, the sources it watches, the
/// copy of the addon the game runs (a new name per build, so the running
/// process's file is never overwritten) and the restart in flight.
struct EngineLoop {
    repo: PathBuf,
    dir: PathBuf,
    save: PathBuf,
    addon: PathBuf,
    builds: usize,
    seen: BTreeMap<PathBuf, SystemTime>,
    polled: Instant,
    pending: Option<Restart>,
}

impl EngineLoop {
    /// The checkout above the project or the engine package, its addon built once.
    fn open(project: &Project, package: &Path) -> Result<Self, String> {
        if !cfg!(unix) {
            return Err(
                "the hot restart needs a Unix signal (SIGUSR2); not on this platform".into(),
            );
        }
        let repo = crate::find_root(&project.root)
            .or_else(|| crate::find_root(package))
            .ok_or_else(|| {
                format!(
                    "rebuilds the engine of an origin-engine checkout, and there is none above {} or {}",
                    project.root.display(),
                    package.display()
                )
            })?;
        let dir = project.work_dir().join("dev-engine");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        eprintln!(
            "og dev: building the engine in {} (cargo build -p og-host-node -p og-cli)",
            repo.display()
        );
        let (built, _) = build_addon(&repo)?;
        let mut e = EngineLoop {
            save: dir.join("world.save"),
            addon: PathBuf::new(),
            builds: 0,
            seen: BTreeMap::new(),
            polled: Instant::now(),
            pending: None,
            repo,
            dir,
        };
        e.seen = e.take();
        e.addon = e.install(&built)?;
        Ok(e)
    }

    /// Every watched file's modification time: what reaches the addon.
    fn take(&self) -> BTreeMap<PathBuf, SystemTime> {
        let mut m = BTreeMap::new();
        let mut file = |f: PathBuf| {
            if let Ok(t) = std::fs::metadata(&f).and_then(|m| m.modified()) {
                m.insert(f, t);
            }
        };
        file(self.repo.join("Cargo.toml"));
        file(self.repo.join("Cargo.lock"));
        let mut crates: Vec<PathBuf> = std::fs::read_dir(self.repo.join("crates"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        crates.sort();
        for c in &crates {
            file(c.join("Cargo.toml"));
            file(c.join("build.rs"));
        }
        for c in &crates {
            for d in ["src", "shaders", "data"] {
                snapshot(&c.join(d), &mut m);
            }
        }
        m
    }

    /// A copy of the built addon under a new name (bun loads a `.node`).
    fn install(&mut self, built: &Path) -> Result<PathBuf, String> {
        self.builds += 1;
        let to = self.dir.join(format!("addon-{}.node", self.builds));
        std::fs::copy(built, &to).map_err(|e| format!("cannot copy {} ({e})", built.display()))?;
        Ok(to)
    }
}

/// `cargo build -p og-host-node -p og-cli` in the checkout, its diagnostics on
/// stderr: the addon's path and whether cargo built it anew.
fn build_addon(repo: &Path) -> Result<(PathBuf, bool), String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = std::process::Command::new(cargo)
        .current_dir(repo)
        // og with it: the pair packages/engine/scripts/build.sh builds, which
        // resolves the features the test build does (docs/ci.md §Test
        // binaries); og-host-node alone is a third graph, every crate twice.
        .args([
            "build",
            "-p",
            "og-host-node",
            "-p",
            "og-cli",
            "--message-format=json-render-diagnostics",
        ])
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cannot run cargo ({e})"))?;
    if !out.status.success() {
        return Err("the engine build failed (above)".into());
    }
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v["reason"] == "compiler-artifact"
            && v["target"]["name"] == "og_host_node"
            && let Some(f) = v["filenames"][0].as_str()
        {
            return Ok((PathBuf::from(f), !v["fresh"].as_bool().unwrap_or(false)));
        }
    }
    Err("cargo named no og-host-node artifact".into())
}

/// A hot restart in flight: the save seen, the build's and the handoff's milliseconds.
struct Restart {
    saved: Instant,
    build_ms: f64,
    handoff_ms: f64,
}

impl Restart {
    /// One line per restart; `restored` is run.ts's restored event.
    fn report(&self, n: usize, restored: &serde_json::Value, json: bool) {
        let total = ms(self.saved);
        let start = ((total - self.build_ms - self.handoff_ms) * 10.0).round() / 10.0;
        let ok = restored["ok"].as_bool().unwrap_or(true);
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "event": "restart", "n": n, "ok": ok, "tick": restored["tick"],
                    "reason": restored["reason"], "buildMs": self.build_ms,
                    "handoffMs": self.handoff_ms, "startMs": start, "totalMs": total,
                })
            );
        } else if ok {
            println!(
                "engine restart {n}: the world kept at tick {} in {total:.0} ms (build {:.0} ms + handoff {:.0} ms + start {start:.0} ms)",
                restored["tick"], self.build_ms, self.handoff_ms
            );
        } else {
            println!(
                "engine restart {n}: the new engine started fresh ({}) in {total:.0} ms",
                restored["reason"]
                    .as_str()
                    .unwrap_or("the world was refused")
            );
        }
    }
}

/// Asks the game to hand its world over (run.ts `--handoff`).
fn ask_handoff(pid: u32) {
    #[cfg(unix)]
    // SAFETY: kill has no memory effects; the pid is the game og started.
    unsafe {
        libc::kill(pid as i32, libc::SIGUSR2);
    }
    #[cfg(not(unix))]
    let _ = pid;
}

/// The child's exit within `limit`, else it is killed and None.
fn wait_exit(child: &mut std::process::Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let t0 = Instant::now();
    loop {
        if let Ok(Some(s)) = child.try_wait() {
            return Some(s);
        }
        if t0.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
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

/// `og dev --web`: the project's development web build served on `host:port`
/// with every edit pushed to the open page (src/node/dev-web.ts; guide
/// testing §og dev --web). It cooks and builds once, then keeps one
/// `og cook --watch` and bundles the game's code on each src/ change; a cook,
/// bundle or type error is reported on the page and in `_dev/status`, never
/// an exit. Signals reach the server's whole process group, as the native
/// loop's do.
fn web_dev(p: &Project, package: &Path, port: &str, host: &str) -> ExitCode {
    let og = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("og"));
    let mut cmd = project::bun(package, "src/node/dev-web.ts");
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    cmd.arg("--project")
        .arg(&p.root)
        .arg("--entry")
        .arg(p.root.join(&p.entry))
        .arg("--port")
        .arg(port)
        .arg("--host")
        .arg(host)
        .arg("--og")
        .arg(&og)
        .env("OG_BIN", &og)
        .env("OG_ENGINE_PACKAGE", package);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("og dev --web: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    };
    let group = Target::Group(child.id());
    relay::start(group);
    loop {
        if let Some(sig) = relay::received() {
            relay::finish(group, sig, Duration::from_secs(3), || {
                matches!(child.try_wait(), Ok(Some(_)))
            });
        }
        if let Ok(Some(status)) = child.try_wait() {
            relay::stop();
            stop_group(group);
            return project::exit_of(status);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut project = ".".to_string();
    let mut port = crate::probe::DEFAULT_PORT.to_string();
    let mut exit_after: Option<usize> = None;
    let mut json = false;
    let mut engine_mode = false;
    let mut web = false;
    let mut port_given = false;
    let mut host = "127.0.0.1".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--web" => web = true,
            "--host" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og dev: --host needs a value. {USAGE}");
                    return ExitCode::from(2);
                };
                host = v.to_string();
                i += 1;
            }
            "--port" | "--exit-after" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og dev: {} needs a value. {USAGE}", args[i]);
                    return ExitCode::from(2);
                };
                if args[i] == "--port" {
                    port = v.to_string();
                    port_given = true;
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
            "--engine" => engine_mode = true,
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
                "og dev: {} has no og-project.json; og dev watches a project",
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
    if web {
        if engine_mode || exit_after.is_some() {
            eprintln!("og dev --web: --engine and --exit-after are the native loop's. {USAGE}");
            return ExitCode::from(2);
        }
        return web_dev(
            &p,
            &package,
            if port_given { &port } else { WEB_PORT },
            &host,
        );
    }
    let bundle = match crate::run::prepare(&p, &package, false, json) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og dev: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut checks = match Checker::find(Some(&package), None) {
        Ok((checker, note)) => {
            if let Some(n) = note {
                eprintln!("og check: {n}");
            }
            let (tx, rx) = mpsc::channel();
            Some(Checks {
                checker,
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
        c.start(source, &p);
    }
    let mut engine = if engine_mode {
        match EngineLoop::open(&p, &package) {
            Ok(e) => Some(e),
            Err(e) => {
                eprintln!("og dev --engine: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };
    let mut p = p;
    let refresh_file = p.work_dir().join("refresh");
    let _ = std::fs::write(&refresh_file, b"0\n");
    let handoff = engine.as_ref().map(|e| e.save.clone());
    let spawn = |root: &Path,
                 bundle: &Path,
                 refresh: &Path,
                 addon: Option<&Path>,
                 restore: Option<&Path>| {
        let mut cmd = project::bun(&package, "src/node/run.ts");
        // Its own process group: a signal to og reaches everything the game started.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
        cmd.arg("--entry")
            .arg(bundle)
            .arg("--root")
            .arg(root)
            .arg("--watch")
            .arg(bundle)
            .arg("--watch-refresh")
            .arg(refresh)
            .arg("--port")
            .arg(&port)
            .arg("--realtime")
            .arg("--out")
            .arg(root.join("og-out"))
            .stdout(Stdio::piped());
        if let Some(a) = addon {
            cmd.env("OG_NATIVE_ADDON", a);
        }
        if let Some(h) = &handoff {
            cmd.arg("--handoff").arg(h);
        }
        if let Some(r) = restore {
            cmd.arg("--restore").arg(r);
        }
        cmd.spawn()
    };
    let mut child = match spawn(
        &p.root,
        &bundle,
        &refresh_file,
        engine.as_ref().map(|e| e.addon.as_path()),
        None,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("og dev: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    };
    let mut group = Target::Group(child.id());
    relay::start(group);
    let (tx, rx) = mpsc::channel::<String>();
    pipe(&mut child, tx.clone());
    let project_file = p.root.join("project.ts");
    let watch_root = p.root.clone();
    let take = || {
        let mut m = BTreeMap::new();
        for file in og_cook::cook::source_project::files(&watch_root).unwrap_or_default() {
            if let Ok(time) = std::fs::metadata(&file).and_then(|m| m.modified()) {
                m.insert(file, time);
            }
        }
        m
    };
    let mut seen = take();
    let mut reloads = 0usize;
    let mut restarts = 0usize;
    let mut refresh_seq = 0u64;
    // The rebuild time (save seen to bundle written) of each reload in
    // flight, and the source change its bundle holds.
    let mut pending: Vec<(Instant, f64, usize)> = Vec::new();
    // Cook time (save seen to refresh sentinel touched) of each in-place
    // asset-only refresh in flight.
    let mut pending_refresh: Vec<(Instant, f64)> = Vec::new();
    // --exit-after was reached: stop once the last check has landed.
    let mut stopping: Option<Instant> = None;
    if !json {
        println!(
            "og dev: watching project metadata, configuration, content, modules and sources of {}; reloads print below (budget {BUDGET_MS} ms)",
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
            return ExitCode::SUCCESS;
        }
        while let Ok(line) = rx.try_recv() {
            let parsed: Option<serde_json::Value> = serde_json::from_str(&line).ok();
            // --engine's handoff and restore lines: the restart is reported once restored.
            if let Some(v) = parsed
                .as_ref()
                .filter(|v| v["event"] == "handoff" || v["event"] == "restored")
            {
                if v["event"] == "restored"
                    && let Some(r) = engine.as_mut().and_then(|e| e.pending.take())
                {
                    restarts += 1;
                    r.report(restarts, v, json);
                    if exit_after.is_some_and(|n| reloads + restarts >= n) {
                        stopping = Some(Instant::now());
                        break;
                    }
                }
                continue;
            }
            if let Some(v) = parsed.as_ref().filter(|v| v["event"] == "refresh") {
                reloads += 1;
                let swap = v["latencyMs"].as_f64().unwrap_or(f64::NAN);
                let cook_ms = if pending_refresh.is_empty() {
                    0.0
                } else {
                    pending_refresh.remove(0).1
                };
                let total = cook_ms + swap;
                let ok = v["ok"].as_bool().unwrap_or(false);
                let within = total <= BUDGET_MS;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "event": "refresh", "n": reloads, "ok": ok,
                            "cookMs": (cook_ms * 10.0).round() / 10.0,
                            "swapMs": swap, "totalMs": (total * 10.0).round() / 10.0,
                            "budgetMs": BUDGET_MS, "withinBudget": within,
                            "report": v["report"], "tick": v["tick"],
                            "rendered": v["rendered"], "timings": v["timings"],
                        })
                    );
                } else {
                    println!(
                        "refresh {reloads}: {} in {total:.0} ms (cook {cook_ms:.0} ms + apply {swap:.0} ms; budget {BUDGET_MS} ms{})",
                        if ok { "ok" } else { "failed" },
                        if within { "" } else { ", OVER" }
                    );
                }
                if exit_after.is_some_and(|n| reloads + restarts >= n) {
                    stopping = Some(Instant::now());
                    break;
                }
                continue;
            }
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
                                "rendered": v["rendered"], "timings": v["timings"],
                            })
                        );
                    } else {
                        let checked = match (check["ok"].as_bool(), check["errors"].as_u64()) {
                            _ if checks.is_none() => String::new(),
                            (Some(true), _) => format!("; check ok {} ms", check["ms"]),
                            (Some(false), Some(n)) => format!("; check: {n} error(s)"),
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
                    if exit_after.is_some_and(|n| reloads + restarts >= n) {
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
        if let Some(e) = &mut engine
            && e.polled.elapsed() >= ENGINE_POLL
        {
            e.polled = Instant::now();
            let now = e.take();
            if !changed(&e.seen, &now).is_empty() {
                e.seen = now;
                let saved = Instant::now();
                eprintln!(
                    "og dev: the engine changed; building the addon (the game keeps running)"
                );
                match build_addon(&e.repo) {
                    Err(err) => eprintln!("og dev: {err}; the game keeps running the last engine"),
                    Ok((_, false)) => {
                        eprintln!("og dev: the change does not reach the addon; nothing to restart")
                    }
                    Ok((built, true)) => {
                        let build_ms = ms(saved);
                        let addon = match e.install(&built) {
                            Ok(a) => a,
                            Err(err) => {
                                eprintln!("og dev: {err}; the game keeps running the last engine");
                                continue;
                            }
                        };
                        let handed = Instant::now();
                        let _ = std::fs::remove_file(&e.save);
                        ask_handoff(child.id());
                        let status = wait_exit(&mut child, Duration::from_secs(10));
                        stop_group(group);
                        let handoff_ms = ms(handed);
                        let kept = status.is_some_and(|s| s.success()) && e.save.is_file();
                        if !kept {
                            eprintln!(
                                "og dev: the game handed no world over ({}); the new engine starts fresh",
                                status.map_or("it did not stop in 10 s".into(), |s| s.to_string())
                            );
                        }
                        let restore = kept.then_some(e.save.as_path());
                        match spawn(&p.root, &bundle, &refresh_file, Some(&addon), restore) {
                            Ok(mut c) => {
                                pipe(&mut c, tx.clone());
                                group = Target::Group(c.id());
                                relay::retarget(group);
                                child = c;
                                let _ = std::fs::remove_file(&e.addon);
                                e.addon = addon;
                                let r = Restart {
                                    saved,
                                    build_ms,
                                    handoff_ms,
                                };
                                if kept {
                                    e.pending = Some(r);
                                } else {
                                    restarts += 1;
                                    r.report(restarts, &serde_json::json!({ "ok": false, "reason": "no world handed over" }), json);
                                }
                            }
                            Err(err) => {
                                relay::stop();
                                eprintln!("og dev: {} ({err})", project::NO_BUN);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                }
            }
        }
        let now = take();
        let changes = changed(&seen, &now);
        if changes.is_empty() {
            continue;
        }
        seen = now;
        let saved = Instant::now();
        // Non-game TS/data inputs may affect bootstrap, module membership or
        // author evaluation. Restart through the checked project, never reuse
        // a stale bootstrap or merely hot-reload an author helper as game code.
        let project_changed = changes.iter().any(|c| {
            c == &project_file
                || (!c.starts_with(p.root.join("src")) && !c.starts_with(p.root.join("assets")))
                || c.starts_with(p.root.join("src/schema"))
        });
        let assets = changes.iter().any(|c| c.starts_with(p.root.join("assets")));
        let src_changed = changes.iter().any(|c| c.starts_with(p.root.join("src")));
        if project_changed {
            eprintln!("og dev: project author inputs changed; re-cooking and restarting");
            match Project::open(&p.root) {
                Ok(reopened) => p = reopened,
                Err(e) => {
                    eprintln!("og dev: {e}; the game keeps running");
                    continue;
                }
            }
            if let Err(e) = crate::cook::cook(&p.root, 0, false) {
                eprintln!("{e}");
                continue;
            }
            if let Some(c) = &mut checks {
                source += 1;
                c.start(source, &p);
            }
            if let Err(e) = project::bundle_native_to(&p, &package, &bundle) {
                eprintln!("og dev: {e}; the game keeps running");
                continue;
            }
            let _ = child.kill();
            let _ = child.wait();
            stop_group(group);
            match spawn(
                &p.root,
                &bundle,
                &refresh_file,
                engine.as_ref().map(|e| e.addon.as_path()),
                None,
            ) {
                Ok(mut c) => {
                    pipe(&mut c, tx.clone());
                    group = Target::Group(c.id());
                    relay::retarget(group);
                    child = c;
                    restarts += 1;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "event": "restart", "n": restarts, "reason": "author-inputs",
                                "ms": (saved.elapsed().as_secs_f64() * 10000.0).round() / 10.0,
                            })
                        );
                    } else {
                        println!(
                            "restart {restarts}: project author inputs reloaded in {:.0} ms",
                            saved.elapsed().as_secs_f64() * 1000.0
                        );
                    }
                    if exit_after.is_some_and(|n| reloads + restarts >= n) {
                        stopping = Some(Instant::now());
                    }
                }
                Err(err) => {
                    relay::stop();
                    eprintln!("og dev: {} ({err})", project::NO_BUN);
                    return ExitCode::FAILURE;
                }
            }
            continue;
        }
        if assets {
            match crate::cook::cook(&p.root, 0, false) {
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
            if !src_changed {
                refresh_seq += 1;
                if let Err(e) = std::fs::write(&refresh_file, format!("{refresh_seq}\n")) {
                    eprintln!(
                        "og dev: cannot signal asset refresh ({e}); falling back to bundle reload"
                    );
                } else {
                    pending_refresh.push((saved, saved.elapsed().as_secs_f64() * 1000.0));
                    continue;
                }
            }
        }
        // A source change is checked beside the rebuild; other assets are not.
        if src_changed && let Some(c) = &mut checks {
            source += 1;
            c.start(source, &p);
        }
        match project::bundle_native_to(&p, &package, &bundle) {
            Ok(_) => pending.push((saved, saved.elapsed().as_secs_f64() * 1000.0, source)),
            Err(e) => eprintln!("og dev: {e}; the game keeps running the last bundle"),
        }
    }
}
