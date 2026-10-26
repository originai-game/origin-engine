//! `og run --device chrome|android[:serial]|iphone[:udid] [project] [--seconds N]
//! [--port P] [--out DIR] [--size WxH] [--headed] [--webgl] [--chrome <path>]
//! [--cert <pem> --key <pem>] [--host <ip>] [--json]`
//!
//! The game on a real browser: cooks, exports the publish build into
//! `<project>/.og/device` (as `og build --publish`), then
//! `packages/engine/src/node/device.ts` serves it with COOP/COEP on
//! 127.0.0.1 (for the iPhone on 0.0.0.0, at the LAN address it prints or
//! `--host`), opens it in the target (local Chrome over the
//! DevTools protocol, Chrome on Android over adb, Safari on an iPhone over
//! safaridriver), passes the play gate with a real tap and times the first
//! frame and --seconds of played frames; the target's tools (Chrome, adb,
//! safaridriver) are checked before the build. The report, og polish's shape, is
//! `report.md` and `device.json` in `--out` (default
//! `<project>/og-out/device`). A missing prerequisite exits 1 with its fix.
//! SIGINT, SIGTERM and SIGHUP to og are passed on to the bun process, which
//! undoes its adb forwards, Chrome profile, WebDriver session and server and
//! exits 130, 143 or 129; og waits for it and exits with its code.

use crate::project::{self, Project};
use crate::relay;
use std::path::Path;
use std::process::{Command, ExitCode, ExitStatus};

/// What `og run --device` was asked: the target, and the options handed to device.ts.
pub struct DeviceArgs {
    pub target: String,
    pub forward: Vec<String>,
}

pub fn run(project: &str, out: Option<&str>, port: Option<&str>, a: &DeviceArgs) -> ExitCode {
    let p = match Project::open(Path::new(project)) {
        Ok(p) if p.manifest => p,
        Ok(p) => {
            eprintln!(
                "og run --device: {} is not a project (no project.ogt); the device plays the exported page of a project",
                p.root.display()
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("og run --device: {e}");
            return ExitCode::FAILURE;
        }
    };
    let settings = match crate::publish::settings(&p.root.join("project.ogt")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og run --device: {e}");
            return ExitCode::FAILURE;
        }
    };
    // The device plays the publish build.
    if let Err(e) = crate::publish::refuse_3d_world(&p.root, &settings) {
        eprintln!("og run --device: {e}");
        return ExitCode::FAILURE;
    }
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og run --device: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    // The target's tools and the options first: a missing Chrome, adb or
    // safaridriver is told before a build is spent on it.
    match status(
        project::bun(&package, "src/node/device.ts")
            .arg("--preflight")
            .arg("--device")
            .arg(&a.target)
            .args(&a.forward),
    ) {
        Ok(s) if s.success() => {}
        Ok(s) => return project::exit_of(s),
        Err(e) => {
            eprintln!("og run --device: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    }
    if p.has_assets()
        && let Err(e) = crate::cook::cook(&p.root, 0, true)
    {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    let dist = p.work_dir().join("device");
    let built = status(
        crate::bundle::web_build(&p, &package, &settings, &dist, true)
            .arg("--json")
            .stdout(std::process::Stdio::null()),
    );
    match built {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("og run --device: the publish build failed (the error is above)");
            return project::exit_of(s);
        }
        Err(e) => {
            eprintln!("og run --device: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    }
    let mut cmd = project::bun(&package, "src/node/device.ts");
    cmd.arg("--dist")
        .arg(&dist)
        .arg("--project")
        .arg(&p.root)
        .arg("--device")
        .arg(&a.target)
        .arg("--out")
        .arg(out.map_or_else(|| p.root.join("og-out").join("device"), Into::into))
        .args(&a.forward);
    if let Some(port) = port {
        cmd.arg("--port").arg(port);
    }
    match status(&mut cmd) {
        Ok(s) => project::exit_of(s),
        Err(e) => {
            eprintln!("og run --device: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}

/// Runs a bun child to its end with SIGINT, SIGTERM and SIGHUP passed on to
/// it (crate::relay), so og neither dies before the child has cleaned up nor
/// leaves it running.
fn status(cmd: &mut Command) -> std::io::Result<ExitStatus> {
    let mut child = cmd.spawn()?;
    relay::start(relay::Target::Process(child.id()));
    let s = child.wait();
    relay::stop();
    s
}
