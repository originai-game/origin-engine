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
//! The server's own jobserver can still run dry: sccache gives rustc the
//! server's pool of one token per CPU, and a rustc killed while it holds
//! tokens for its threads never returns them (mozilla/sccache#2874). After
//! enough cancelled builds no compile starts again (macrunner, 2026-11-09:
//! 13,299 requests in 4 h 48 min, then clients waiting minutes with no
//! rustc running). So on Unix the wrapper watches its compile: when it has
//! waited `OG_SCCACHE_STALL` seconds (default 60; 0 turns the watch off)
//! while the server runs no rustc and the client compiles nothing itself,
//! it stops that server under the lock. Its waiting clients then compile
//! locally (`SCCACHE_IGNORE_SERVER_IO_ERROR=1`), and the next compile
//! starts a fresh server with a full pool. Each stop is logged in
//! `watchdog.log` beside the lock.
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
    cmd.args(&args)
        // A server stopped under a waiting client means compile here.
        .env("SCCACHE_IGNORE_SERVER_IO_ERROR", "1");
    #[cfg(unix)]
    {
        let stall = std::env::var("OG_SCCACHE_STALL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60u64);
        if stall == 0 {
            use std::os::unix::process::CommandExt;
            let e = cmd.exec();
            eprintln!(
                "og-sccache-wrap: cannot run {}: {e}",
                sccache.to_string_lossy()
            );
            std::process::exit(127);
        }
        let child = cmd.spawn().unwrap_or_else(|e| {
            eprintln!(
                "og-sccache-wrap: cannot run {}: {e}",
                sccache.to_string_lossy()
            );
            std::process::exit(127);
        });
        watch::run(child, Duration::from_secs(stall));
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

/// The stall watch (Unix): see the module documentation.
#[cfg(unix)]
mod watch {
    use std::os::unix::process::ExitStatusExt;
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    /// Waits for the client `child` and exits with its status, stopping the
    /// server when it stalls for `stall`.
    pub fn run(mut child: Child, stall: Duration) -> ! {
        let client = child.id();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || drop(tx.send(child.wait())));
        let every = stall.min(Duration::from_secs(5));
        let mut idle_since: Option<Instant> = None;
        loop {
            match rx.recv_timeout(every) {
                Ok(Ok(status)) => std::process::exit(
                    status
                        .code()
                        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
                ),
                Ok(Err(e)) => {
                    eprintln!("og-sccache-wrap: waiting for sccache: {e}");
                    std::process::exit(1);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => std::process::exit(1),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            // Busy: the server runs a compiler, or this client compiles
            // locally (and waits on no server).
            let server = server_pid();
            let busy = server.is_none_or(has_children) || has_children(client);
            if busy {
                idle_since = None;
                continue;
            }
            let since = *idle_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= stall {
                stop_stalled(server.unwrap_or_default());
                idle_since = None;
            }
        }
    }

    /// The pid listening on the server's port.
    fn server_pid() -> Option<u32> {
        let out = Command::new("lsof")
            .args([
                "-nP",
                &format!("-iTCP:{}", super::port()),
                "-sTCP:LISTEN",
                "-t",
            ])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()?
            .trim()
            .parse()
            .ok()
    }

    /// Whether `pid` has a child process.
    fn has_children(pid: u32) -> bool {
        Command::new("pgrep")
            .args(["-P", &pid.to_string()])
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    fn kill(signal: &str, pid: u32) {
        let _ = Command::new("kill")
            .args([signal, &pid.to_string()])
            .stderr(Stdio::null())
            .status();
    }

    /// Stops the stalled server `pid` under the start lock, unless another
    /// lane already replaced it or it started a compiler meanwhile.
    fn stop_stalled(pid: u32) {
        let state = super::state_dir();
        if std::fs::create_dir_all(&state).is_err() {
            return;
        }
        let Ok(lock) = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(state.join("server.lock"))
        else {
            return;
        };
        if lock.lock().is_err() || server_pid() != Some(pid) || has_children(pid) {
            return;
        }
        kill("-TERM", pid);
        let deadline = Instant::now() + Duration::from_secs(10);
        while server_pid() == Some(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(200));
        }
        if server_pid() == Some(pid) {
            kill("-KILL", pid);
        }
        let when = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let line = format!(
            "{when} og-sccache-wrap: stopped sccache server {pid}: no compiler ran for the stall window while a compile waited (jobserver tokens lost, mozilla/sccache#2874)\n"
        );
        eprint!("{line}");
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(state.join("watchdog.log"))
        {
            let _ = f.write_all(line.as_bytes());
        }
    }
}
