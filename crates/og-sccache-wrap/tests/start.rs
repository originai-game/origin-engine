//! The wrapper starts the sccache server without the build's jobserver and
//! runs the compile with it (docs/ci.md, The compiler cache), against a fake
//! sccache that records how it was run.
//!
//! Ways it could fail, each covered below:
//! - the server started with the build's `CARGO_MAKEFLAGS`, `MAKEFLAGS` or
//!   `MFLAGS` (the hang of 2026-11-09), or with the build's temporary
//!   directory, or without its error log; the compile run without the
//!   jobserver, without its arguments, or its exit status lost →
//!   `no_server_starts_one_without_the_jobserver_then_compiles`
//! - a server started (and a running one disturbed) when one answers →
//!   `an_answering_server_is_left_alone`

use std::path::{Path, PathBuf};
use std::process::Command;

const WRAP: &str = env!("CARGO_BIN_EXE_og-sccache-wrap");

/// A fresh directory for one test.
fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("og-sccache-wrap-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A fake sccache in `dir` that appends to `log`: for `--start-server`, a
/// `start` line with the jobserver variables, `SCCACHE_ERROR_LOG` and
/// `TMPDIR` it sees; else a `run` line with its arguments and
/// `CARGO_MAKEFLAGS`, exiting 7.
fn fake_sccache(dir: &Path, log: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("sccache");
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = --start-server ]; then\n  echo \"start CARGO_MAKEFLAGS=${{CARGO_MAKEFLAGS-unset}} MAKEFLAGS=${{MAKEFLAGS-unset}} MFLAGS=${{MFLAGS-unset}} SCCACHE_ERROR_LOG=$SCCACHE_ERROR_LOG TMPDIR=$TMPDIR\" >> '{log}'\n  exit 0\nfi\necho \"run $* CARGO_MAKEFLAGS=${{CARGO_MAKEFLAGS-unset}}\" >> '{log}'\nexit 7\n",
            log = log.display()
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
    #[cfg(windows)]
    {
        let path = dir.join("sccache.cmd");
        let script = format!(
            "@echo off\r\nif \"%1\"==\"--start-server\" (\r\n  if not defined CARGO_MAKEFLAGS set CARGO_MAKEFLAGS=unset\r\n  if not defined MAKEFLAGS set MAKEFLAGS=unset\r\n  if not defined MFLAGS set MFLAGS=unset\r\n  echo start CARGO_MAKEFLAGS=%CARGO_MAKEFLAGS% MAKEFLAGS=%MAKEFLAGS% MFLAGS=%MFLAGS% SCCACHE_ERROR_LOG=%SCCACHE_ERROR_LOG% TMPDIR=%TMPDIR%>> \"{log}\"\r\n  exit /b 0\r\n)\r\necho run %* CARGO_MAKEFLAGS=%CARGO_MAKEFLAGS%>> \"{log}\"\r\nexit /b 7\r\n",
            log = log.display()
        );
        std::fs::write(&path, script).unwrap();
        path
    }
}

/// A port nothing listens on, or (`listen`) one a listener holds.
fn port(listen: bool) -> (u16, Option<std::net::TcpListener>) {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let p = l.local_addr().unwrap().port();
    (p, listen.then_some(l))
}

fn run(dir: &Path, port: u16) -> (i32, String) {
    let log = dir.join("log");
    let fake = fake_sccache(dir, &log);
    let jobserver = "-j --jobserver-fds=3,4 --jobserver-auth=3,4";
    let status = Command::new(WRAP)
        .args(["rustc", "--crate-name", "x"])
        .env("OG_SCCACHE", &fake)
        .env("OG_SCCACHE_STATE", dir.join("state"))
        .env("SCCACHE_SERVER_PORT", port.to_string())
        .env("CARGO_MAKEFLAGS", jobserver)
        .env("MAKEFLAGS", jobserver)
        .env("MFLAGS", "-j")
        .env("TMPDIR", dir.join("build-tmp"))
        .status()
        .unwrap();
    (
        status.code().unwrap_or(-1),
        std::fs::read_to_string(&log).unwrap_or_default(),
    )
}

#[test]
fn no_server_starts_one_without_the_jobserver_then_compiles() {
    let dir = scratch("start");
    let (p, _) = port(false);
    let (code, log) = run(&dir, p);
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 2, "{log}");
    let state = dir.join("state");
    assert_eq!(
        lines[0].trim_end(),
        format!(
            "start CARGO_MAKEFLAGS=unset MAKEFLAGS=unset MFLAGS=unset SCCACHE_ERROR_LOG={} TMPDIR={}",
            state.join("server-error.log").display(),
            state.join("tmp").display()
        ),
        "the server must start with no jobserver, its own error log and temporary directory"
    );
    assert_eq!(
        lines[1].trim_end(),
        "run rustc --crate-name x CARGO_MAKEFLAGS=-j --jobserver-fds=3,4 --jobserver-auth=3,4",
        "the compile keeps its arguments and the build's jobserver"
    );
    assert_eq!(code, 7, "the compile's exit status");
}

#[test]
fn an_answering_server_is_left_alone() {
    let dir = scratch("answering");
    let (p, _listener) = port(true);
    let (code, log) = run(&dir, p);
    assert!(!log.contains("start"), "{log}");
    assert!(log.starts_with("run rustc"), "{log}");
    assert_eq!(code, 7);
}
