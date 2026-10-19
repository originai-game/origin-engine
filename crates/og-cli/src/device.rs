//! `og run --device chrome|android[:serial]|iphone[:udid] [project] [--seconds N]
//! [--port P] [--out DIR] [--size WxH] [--headed] [--webgl] [--chrome <path>]
//! [--cert <pem> --key <pem>] [--json]`
//!
//! The game on a real browser: cooks, exports the publish build into
//! `<project>/.og/device` (as `og build --publish`), then
//! `packages/engine/src/node/device.ts` serves it on 0.0.0.0 with COOP/COEP,
//! prints the LAN URL, opens it in the target (local Chrome over the
//! DevTools protocol, Chrome on Android over adb, Safari on an iPhone over
//! safaridriver), passes the play gate with a real tap and times the first
//! frame and --seconds of played frames; the target's tools (Chrome, adb,
//! safaridriver) are checked before the build. The report, og polish's shape, is
//! `report.md` and `device.json` in `--out` (default
//! `<project>/og-out/device`). A missing prerequisite exits 1 with its fix.

use crate::project::{self, Project};
use std::path::Path;
use std::process::ExitCode;

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
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og run --device: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    // The target's tools and the options first: a missing Chrome, adb or
    // safaridriver is told before a build is spent on it.
    match project::bun(&package, "src/node/device.ts")
        .arg("--preflight")
        .arg("--device")
        .arg(&a.target)
        .args(&a.forward)
        .status()
    {
        Ok(s) if s.success() => {}
        Ok(s) => return project::exit_of(s),
        Err(e) => {
            eprintln!("og run --device: {} ({e})", project::NO_BUN);
            return ExitCode::FAILURE;
        }
    }
    if p.has_assets()
        && let Err(e) = crate::cook::cook(&p.root, 0)
    {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    let dist = p.work_dir().join("device");
    let built = crate::bundle::web_build(&p, &package, &settings, &dist, true)
        .arg("--json")
        .stdout(std::process::Stdio::null())
        .status();
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
    match cmd.status() {
        Ok(s) => project::exit_of(s),
        Err(e) => {
            eprintln!("og run --device: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
