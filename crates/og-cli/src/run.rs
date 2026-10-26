//! `og run [project|entry] [--headless] [--commands "t:action[:args];..."] [--seed S]
//! [--port P] [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]...
//! [--audio null|device|<out.wav>] [--platform-store <file.json>] [--clock <UTC ms>] [--locale <tag>] [--trace <out.json>] [--json] [--no-check]`
//!
//! Cooks the project, type-checks it (crate::check: a type error stops the
//! run before anything is bundled, its diagnostics printed, or with
//! `--json` as the `check` event on the run's stream; `--no-check` skips
//! it), bundles its entry for the native host and runs it with
//! `packages/engine/src/node/run.ts` under bun on the napi addon: the probe is
//! mounted, `--commands` are scheduled through og-probe, outputs
//! (`screenshot`, `dump_*`) are written under `--out` (default
//! `<project>/og-out`) at the probe's `Output::file_name()`, and `--port`
//! serves the probe at `ws://127.0.0.1:<port>/probe` (`og probe`). `--audio
//! out.wav` renders the run's audio with a capture mixer into a 16-bit WAV
//! (relative to the current directory; its SHA-256 is in `--json`'s summary,
//! the same for two runs of the same script). og.platform's calls are
//! answered from a local store, in memory unless `--platform-store
//! file.json` keeps it in a file; `og.platform.now` reads the system clock,
//! or with `--clock <UTC ms>` a clock that starts there and moves with the
//! ticks (reproducible, as playtests run). `--trace out.json` writes the
//! run in the Chrome trace format (`trace.rs`): the cook, each tick with its
//! systems, each frame with its render passes, each load. `--room
//! ws://127.0.0.1:P/<room>` joins that room through `og relay` (og.platform's
//! rooms, og.net's lockstep; a run in a room keeps going until `quit` or
//! --ticks, waiting for the other players). `--locale ja` is the locale og.i18n is
//! asked for (default: none, the base table). Native runs
//! are always headless: `--headless` is accepted, and without it the run is
//! paced by the wall clock instead of as fast as it can go.
//!
//! `og run --device chrome|android[:serial]|iphone[:udid] [project]
//! [--seconds N] [--port P] [--out DIR] [--size WxH] [--headed] [--webgl]
//! [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>] [--json]`
//! plays the publish build in a real browser instead and reports its first
//! frame and frame times (`device.rs`).

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og run [project|entry] [--headless] [--commands \"t:action[:args];...\"] [--seed S] [--port P] [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--audio null|device|<out.wav>] [--platform-store <file.json>] [--clock <UTC ms>] [--locale <tag>] [--trace <out.json>] [--room ws://127.0.0.1:P/<room>] [--json] [--no-check]\n       og run --device chrome|android[:serial]|iphone[:udid] [project] [--seconds N] [--port P] [--out DIR] [--size WxH] [--headed] [--webgl] [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>] [--json]";

/// What `og run` was asked.
pub struct RunArgs {
    pub project: String,
    pub headless: bool,
    pub forward: Vec<String>,
    pub out: Option<String>,
    pub port: Option<String>,
    /// Type-check before bundling (not `--no-check`).
    pub check: bool,
    pub json: bool,
    /// `--device`: the browser target and device.ts's own options.
    pub device: Option<crate::device::DeviceArgs>,
    /// `--trace`: the Chrome trace's path (absolute).
    pub trace: Option<std::path::PathBuf>,
    /// Native-only options given (refused with `--device`).
    native: Vec<String>,
}

pub fn parse(args: &[&str]) -> Result<RunArgs, String> {
    let mut r = RunArgs {
        project: ".".into(),
        headless: false,
        forward: Vec::new(),
        out: None,
        port: None,
        check: true,
        json: false,
        device: None,
        trace: None,
        native: Vec::new(),
    };
    let mut device_forward = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--headless" => {
                r.headless = true;
                r.native.push(args[i].into());
            }
            "--json" => {
                r.json = true;
                r.forward.push(args[i].into());
                device_forward.push(args[i].to_string());
            }
            "--no-check" => {
                r.check = false;
                r.native.push(args[i].into());
            }
            "--no-render" => {
                r.forward.push(args[i].into());
                r.native.push(args[i].into());
            }
            f @ ("--headed" | "--webgl") => device_forward.push(f.to_string()),
            flag @ ("--device" | "--seconds" | "--host" | "--chrome" | "--cert" | "--key") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--device" => {
                        r.device = Some(crate::device::DeviceArgs {
                            target: v.to_string(),
                            forward: Vec::new(),
                        })
                    }
                    "--seconds" | "--host" => {
                        device_forward.extend([flag.to_string(), v.to_string()])
                    }
                    _ => device_forward.extend([
                        flag.to_string(),
                        std::path::absolute(v)
                            .map_err(|e| format!("{flag} {v}: {e}"))?
                            .display()
                            .to_string(),
                    ]),
                }
                i += 1;
            }
            flag @ ("--ticks" | "--seed" | "--setup" | "--commands" | "--size" | "--out"
            | "--port" | "--audio" | "--platform-store" | "--clock" | "--locale"
            | "--trace" | "--room") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--out" => r.out = Some(v.to_string()),
                    "--port" => r.port = Some(v.to_string()),
                    "--trace" => {
                        r.trace =
                            Some(std::path::absolute(v).map_err(|e| format!("--trace {v}: {e}"))?);
                        r.native.push(flag.into());
                    }
                    "--size" => {
                        r.forward.extend([flag.into(), v.to_string()]);
                        device_forward.extend([flag.to_string(), v.to_string()]);
                    }
                    "--audio" | "--platform-store" => {
                        // A path is the caller's, not bun's working directory's.
                        let v = match *v {
                            "null" | "device" if flag == "--audio" => v.to_string(),
                            path => std::path::absolute(path)
                                .map_err(|e| format!("--audio {path}: {e}"))?
                                .display()
                                .to_string(),
                        };
                        r.forward.push(flag.into());
                        r.forward.push(v);
                        r.native.push(flag.into());
                    }
                    _ => {
                        r.forward.push(flag.into());
                        r.forward.push(v.to_string());
                        r.native.push(flag.into());
                    }
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`. {USAGE}"));
            }
            other => r.project = other.to_string(),
        }
        i += 1;
    }
    match &mut r.device {
        Some(d) => {
            if let Some(f) = r.native.first() {
                return Err(format!(
                    "{f} is a native run's option; og run --device takes --seconds, --port, --out, --size, --headed, --webgl, --chrome, --cert, --key, --host and --json"
                ));
            }
            d.forward = device_forward;
        }
        None => {
            if let Some(f) = device_forward
                .iter()
                .find(|f| f.starts_with("--") && !matches!(f.as_str(), "--json" | "--size"))
            {
                return Err(format!("{f} goes with --device. {USAGE}"));
            }
        }
    }
    Ok(r)
}

/// Cook (when the project has assets), type-check when `check` (the
/// diagnostics printed, as the `check` event with `json`) and bundle; the
/// bundle's path.
pub fn prepare(
    p: &Project,
    package: &Path,
    check: bool,
    json: bool,
) -> Result<std::path::PathBuf, String> {
    prepare_traced(p, package, check, json, None)
}

/// [`prepare`], and with `trace` the trace's first part written there: its
/// origin (now) and the cook's spans (`trace.rs`).
pub fn prepare_traced(
    p: &Project,
    package: &Path,
    check: bool,
    json: bool,
    trace: Option<&Path>,
) -> Result<std::path::PathBuf, String> {
    let origin = crate::trace::epoch_ms();
    let mut cook_spans = Vec::new();
    if p.has_assets() {
        let report = crate::cook::cook(&p.root, 0, false)?;
        cook_spans = crate::trace::cook_events(&report, origin, origin);
        let cooked = report
            .assets
            .iter()
            .flat_map(|a| a.outputs.iter())
            .filter(|o| o.cooked)
            .count();
        if cooked > 0 {
            eprintln!("{}", crate::cook::summary(&report));
        }
    }
    if let Some(path) = trace {
        crate::trace::write_prelude(path, cook_spans, origin)?;
    }
    if check {
        crate::check::require(p, package, json)?;
    }
    project::bundle_native(p, package)
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::from(2);
        }
    };
    if let Some(d) = &a.device {
        return crate::device::run(&a.project, a.out.as_deref(), a.port.as_deref(), d);
    }
    let p = match Project::open(Path::new(&a.project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og run: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og run: {n}");
    }
    let bundle = match prepare_traced(&p, &package, a.check, a.json, a.trace.as_deref()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og run: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut cmd = project::bun(&package, "src/node/run.ts");
    cmd.arg("--entry").arg(&bundle).args(&a.forward);
    if p.manifest {
        cmd.arg("--root").arg(&p.root);
    }
    cmd.arg("--out").arg(
        a.out
            .clone()
            .unwrap_or_else(|| p.root.join("og-out").display().to_string()),
    );
    if let Some(port) = &a.port {
        cmd.arg("--port").arg(port);
    }
    if !a.headless {
        cmd.arg("--realtime");
    }
    if let Some(trace) = &a.trace {
        cmd.arg("--trace").arg(trace).arg("--trace-append");
    }
    supervise(cmd, a.port.is_some())
}

/// Lines of the game process's stderr kept for the report when it dies.
const TAIL_LINES: usize = 40;

/// Runs the game process, passing its output through, and when it dies with
/// a failure reports at once: its exit status (or signal), whether its probe
/// had started listening, and its last stderr lines. The exit code is the
/// process's (128 + the signal when it was killed by one).
fn supervise(mut cmd: std::process::Command, probing: bool) -> ExitCode {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::{Arc, Mutex};
    cmd.stderr(std::process::Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("og run: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    };
    let tail = Arc::new(Mutex::new(std::collections::VecDeque::<String>::new()));
    let reader = child.stderr.take().map(|stderr| {
        let tail = tail.clone();
        std::thread::spawn(move || {
            let mut err = std::io::stderr();
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = writeln!(err, "{line}");
                let mut t = tail.lock().unwrap_or_else(|p| p.into_inner());
                if t.len() == TAIL_LINES {
                    t.pop_front();
                }
                t.push_back(line);
            }
        })
    });
    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og run: lost the game process ({e})");
            return ExitCode::FAILURE;
        }
    };
    if let Some(r) = reader {
        let _ = r.join();
    }
    if status.success() {
        return ExitCode::SUCCESS;
    }
    let tail = tail.lock().unwrap_or_else(|p| p.into_inner());
    let (how, code) = describe(&status);
    eprintln!(
        "og run: the game process {how}{}; its last {} stderr line(s):",
        if probing {
            " (its probe stops answering)"
        } else {
            ""
        },
        tail.len()
    );
    for line in tail
        .iter()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        eprintln!("  | {line}");
    }
    ExitCode::from(code)
}

/// How a process ended and the exit code `og run` reports for it.
fn describe(status: &std::process::ExitStatus) -> (String, u8) {
    if let Some(c) = status.code() {
        return (format!("exited with status {c}"), c.clamp(1, 255) as u8);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return (
                format!("was killed by signal {sig}"),
                (128 + sig).clamp(1, 255) as u8,
            );
        }
    }
    ("ended without a status".into(), 1)
}
