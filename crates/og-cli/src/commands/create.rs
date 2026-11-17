//! Scaffold with the creator shipped beside the engine in a distribution
//! (or packages/create beside packages/engine in a source checkout).

use crate::project;
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    let Some(engine) = project::engine_package(None) else {
        eprintln!("og create: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let script = engine.join("../create/bin/create.ts");
    if !script.is_file() {
        eprintln!(
            "og create: no creator at {}; reinstall the engine distribution",
            script.display()
        );
        return ExitCode::FAILURE;
    }
    match project::bun(&engine, "../create/bin/create.ts")
        .args(args)
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!("og create: {} ({error})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
}
