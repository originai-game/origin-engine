//! `og-sccache-wrap <rustc> <args…>`: the rustc wrapper `.cargo/config.toml`
//! names (docs/ci.md, The compiler cache).
//!
//! sccache's client starts the server when none answers, and the server
//! keeps the environment of the build that started it: the jobserver of
//! that one cargo (`CARGO_MAKEFLAGS`) and its temporary directory. The
//! server then waits on that cargo's tokens for every lane's compiles; once
//! that cargo's jobs are all sccache clients waiting on the server, or once
//! it exits, no token comes and every lane's build stalls (macrunner,
//! 2026-11-09). So the wrapper starts the server itself when none answers,
//! under a file lock so two lanes do not race, without any jobserver
//! (`CARGO_MAKEFLAGS`, `MAKEFLAGS`, `MFLAGS`), with a temporary directory
//! and an error log of its own, and then runs sccache with its arguments.
//! The compile itself keeps the build's jobserver.
//!
//! A real executable rather than a script: cargo on Windows runs only an
//! `.exe` as the wrapper, and studio-win's gate uses this too.
//! `RUSTC_WRAPPER=` (empty) still bypasses it, and sccache with it.
//!
//! Environment: `OG_SCCACHE` the sccache to run (default `sccache` on the
//! PATH); `OG_SCCACHE_STATE` the directory of the lock, the server's
//! temporary directory and its error log (default `~/.cache/og-sccache`);
//! `SCCACHE_SERVER_PORT` as sccache reads it (default 4226).

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// What a stale environment must not hand the server: every form of a
/// jobserver the `jobserver` crate (sccache's) reads.
const JOBSERVER_VARS: [&str; 3] = ["CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS"];

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let sccache = std::env::var_os("OG_SCCACHE").unwrap_or_else(|| "sccache".into());
    if !server_answers() {
        // A server that does not start is sccache's to report: the run
        // below starts one as it always did.
        if let Err(e) = start_server(&sccache) {
            eprintln!("og-sccache-wrap: could not start the sccache server cleanly: {e}");
        }
    }
    let mut cmd = Command::new(&sccache);
    cmd.args(&args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let e = cmd.exec();
        eprintln!(
            "og-sccache-wrap: cannot run {}: {e}",
            sccache.to_string_lossy()
        );
        std::process::exit(127);
    }
    #[cfg(not(unix))]
    match cmd.status() {
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!(
                "og-sccache-wrap: cannot run {}: {e}",
                sccache.to_string_lossy()
            );
            std::process::exit(127);
        }
    }
}

/// The port sccache's server listens on.
fn port() -> u16 {
    std::env::var("SCCACHE_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4226)
}

/// Whether a server takes connections (one that answers, even a slow one,
/// is left alone).
fn server_answers() -> bool {
    let at = std::net::SocketAddr::from(([127, 0, 0, 1], port()));
    std::net::TcpStream::connect_timeout(&at, Duration::from_millis(500)).is_ok()
}

/// Where the lock, the server's temporary directory and its error log live:
/// one place per user for every lane (not `TMPDIR`, which a gate run sets
/// per run and deletes after it).
fn state_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("OG_SCCACHE_STATE") {
        return d.into();
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join(".cache").join("og-sccache")
}

/// Starts the server under the lock, unless another lane started one
/// while this one waited for it.
fn start_server(sccache: &OsString) -> std::io::Result<()> {
    let state = state_dir();
    let tmp = state.join("tmp");
    std::fs::create_dir_all(&tmp)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(state.join("server.lock"))?;
    lock.lock()?;
    if server_answers() {
        return Ok(());
    }
    let mut cmd = Command::new(sccache);
    cmd.arg("--start-server")
        .env("SCCACHE_ERROR_LOG", state.join("server-error.log"))
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        // The server outlives this compile: it must not hold cargo's pipes.
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for v in JOBSERVER_VARS {
        cmd.env_remove(v);
    }
    if std::env::var_os("SCCACHE_LOG").is_none() {
        cmd.env("SCCACHE_LOG", "warn");
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "sccache --start-server exited with {status}"
        )));
    }
    Ok(())
}
