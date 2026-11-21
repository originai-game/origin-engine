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
//! On Windows a command line holds at most 32,767 characters. Cargo passes
//! a longer one as `rustc @argfile`, but sccache (0.19.0) reads the
//! argfile and spawns rustc with every argument on the command line, which
//! fails with os error 206 (studio-win, 2026-11-19: og-cook's release
//! build, whose `-L dependency=` list alone passes the limit). So on
//! Windows, when the expanded command line would come near the limit, the
//! wrapper runs rustc itself with cargo's arguments unchanged, the argfile
//! included, and that one compile is not cached; a command line cargo
//! passed whole that is too long once the remaps below are added goes to
//! rustc in a response file of the wrapper's. sccache's main branch
//! spawns such a command through a response file (mozilla/sccache 91d34f1,
//! 2026-11-14); once `SCCACHE_VERSION` has it, this bypass can go.
//!
//! Without sccache (`OG_SCCACHE` or `sccache` on the PATH not found) the
//! wrapper runs rustc itself, uncached, so a machine that has the wrapper
//! but not the cache still builds.
//!
//! A real executable rather than a script: cargo on Windows runs only an
//! `.exe` as the wrapper (there `scripts/rustc-wrapper.exe`, this copied by
//! `scripts/install-tools.sh`), and studio-win's gate uses this too.
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
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    if let Some(remaps) = std::env::var_os("OG_WASM_REPRODUCIBLE") {
        if args.len() == 2 && args[1] == "-vV" {
            // Cargo fingerprints every unit with this answer (with
            // CARGO_CACHE_RUSTC_INFO=0, never a cached one), so units built
            // without the rewrite below are rebuilt; release-wasm.sh reads the
            // line to know this wrapper does the rewrite.
            let out = Command::new(&args[0]).arg("-vV").output();
            let Ok(out) = out else { std::process::exit(1) };
            use std::io::Write;
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(&out.stdout);
            let _ = writeln!(stdout, "og-wasm-reproducible: 1");
            std::process::exit(out.status.code().unwrap_or(1));
        }
        reproducible(&mut args, &remaps);
    }
    let sccache = std::env::var_os("OG_SCCACHE").unwrap_or_else(|| "sccache".into());
    if !installed(&sccache) {
        rustc_alone(&args);
    }
    #[cfg(windows)]
    if expanded_command_line_len(&args) > TOO_LONG_FOR_SCCACHE {
        rustc_alone(&args);
    }
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

/// Runs the compile without sccache: cargo's rustc with its arguments
/// unchanged (an `@argfile` included), exiting with its status.
///
/// On Windows, arguments that do not fit one command line go to rustc in
/// a response file of their own (`rustc @file`, one argument a line).
/// Cargo passes a command line just under the limit as it is, and the
/// remaps `reproducible` adds take it past (studio-win, 2026-11-21: the
/// 3D lanes og-host-web of ui-rewrite's release wasm, os error 206).
fn rustc_alone(args: &[OsString]) -> ! {
    let Some((rustc, rest)) = args.split_first() else {
        eprintln!("og-sccache-wrap: usage: og-sccache-wrap <rustc> <args...>");
        std::process::exit(2);
    };
    #[cfg(windows)]
    let response = response_file(args);
    #[cfg(windows)]
    let status = match &response {
        Some(file) => {
            let mut at = OsString::from("@");
            at.push(file);
            Command::new(rustc).arg(at).status()
        }
        None => Command::new(rustc).args(rest).status(),
    };
    #[cfg(not(windows))]
    let status = Command::new(rustc).args(rest).status();
    #[cfg(windows)]
    if let Some(file) = response {
        let _ = std::fs::remove_file(file);
    }
    match status {
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!(
                "og-sccache-wrap: cannot run {}: {e}",
                rustc.to_string_lossy()
            );
            std::process::exit(127);
        }
    }
}

/// Whether `program` names a file: as a path when it has a separator, else
/// on the PATH (with `.exe` on Windows).
fn installed(program: &std::ffi::OsStr) -> bool {
    let path = std::path::Path::new(program);
    if path.components().count() > 1 {
        return path.is_file();
    }
    let exe = if cfg!(windows) && path.extension().is_none() {
        path.with_extension("exe")
    } else {
        path.to_path_buf()
    };
    std::env::var_os("PATH")
        .is_some_and(|dirs| std::env::split_paths(&dirs).any(|d| d.join(&exe).is_file()))
}

/// The expanded command line length past which a compile skips sccache:
/// Windows' 32,767 less room for what sccache itself adds to the command.
#[cfg(windows)]
const TOO_LONG_FOR_SCCACHE: usize = 30_000;

/// The length, in UTF-16 units, of the command line sccache would spawn for
/// `args` (rustc first): every `@file` replaced by its lines, as rustc and
/// sccache read it, with each argument quoted, its quotes escaped and a
/// space after it.
#[cfg(windows)]
fn expanded_command_line_len(args: &[OsString]) -> usize {
    let mut len = 0;
    for arg in args {
        let file = arg.to_str().and_then(|a| a.strip_prefix('@'));
        match file.and_then(|f| std::fs::read_to_string(f).ok()) {
            Some(contents) => {
                for line in contents.lines().map(str::trim).filter(|l| !l.is_empty()) {
                    len += quoted(line.as_ref());
                }
            }
            None => len += quoted(arg),
        }
    }
    len
}

/// The length an argument takes on a command line: quoted, its quotes
/// escaped, and a space after it.
#[cfg(windows)]
fn quoted(a: &std::ffi::OsStr) -> usize {
    use std::os::windows::ffi::OsStrExt;
    a.encode_wide()
        .map(|c| if c == u16::from(b'"') { 2 } else { 1 })
        .sum::<usize>()
        + 3
}

/// For `args` (rustc first) too long for one command line as they are, a
/// response file holding every argument but rustc, one a line; `None` when
/// they fit, or when one argument holds a line break, which a response
/// file cannot.
#[cfg(windows)]
fn response_file(args: &[OsString]) -> Option<PathBuf> {
    if args.iter().map(|a| quoted(a)).sum::<usize>() <= TOO_LONG_FOR_SCCACHE {
        return None;
    }
    let mut lines = String::new();
    for a in &args[1..] {
        let a = a.to_str()?;
        if a.contains(['\n', '\r']) {
            return None;
        }
        lines.push_str(a);
        lines.push('\n');
    }
    let file = std::env::temp_dir().join(format!("og-sccache-wrap-{}.args", std::process::id()));
    std::fs::write(&file, lines).ok()?;
    Some(file)
}

/// The release wasm (`scripts/release-wasm.sh` sets `OG_WASM_REPRODUCIBLE`)
/// is a function of the source alone, so its frame archives, pinned by
/// hash, rebuild to the same bytes on any host. For each wasm32 compile:
///
/// - cargo's `-C metadata` becomes a hash of the unit's own identity (its
///   package, version, crate name and types, `--cfg`s and target). Cargo's
///   hash takes in its dependencies' hashes, and on nightly a proc-macro's
///   or build script's takes in the host triple, so every crate that
///   depends on one (wasm-bindgen, serde) got a crate id, and so mangled
///   names and type ids, that differ between a Linux and a Mac build. A
///   package appears once per feature set in one build, and the variants
///   that differ in their dependencies' features are separate builds, so
///   the identity is unique wherever two crates meet.
/// - each line of `OG_WASM_REPRODUCIBLE` (`from=to`) becomes a
///   `--remap-path-prefix`, later lines winning, for the absolute paths
///   panics carry: the cargo and rustup homes, build-std's sources and the
///   checkout.
///
/// A host build (no `--target wasm32-unknown-unknown`) or a query is left
/// as it is.
fn reproducible(args: &mut Vec<OsString>, remaps: &std::ffi::OsStr) {
    // args[0] is rustc; a compile names its crate.
    let text: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    // Every value of `flag`, given as `flag value` or `flag=value`.
    let value = |flag: &str| -> Vec<String> {
        let joined = format!("{flag}=");
        (0..text.len())
            .filter_map(|i| match text[i].strip_prefix(&joined) {
                Some(v) => Some(v.to_owned()),
                None => (text[i] == flag)
                    .then(|| text.get(i + 1).cloned())
                    .flatten(),
            })
            .collect()
    };
    let crate_name = value("--crate-name");
    if value("--target").first().map(String::as_str) != Some("wasm32-unknown-unknown")
        || crate_name.is_empty()
    {
        return;
    }
    let mut cfgs = value("--cfg");
    cfgs.sort();
    let mut types = value("--crate-type");
    types.sort();
    let identity = [
        std::env::var("CARGO_PKG_NAME").unwrap_or_default(),
        std::env::var("CARGO_PKG_VERSION").unwrap_or_default(),
        crate_name[0].clone(),
        types.join(","),
        cfgs.join(","),
        "wasm32-unknown-unknown".to_owned(),
    ]
    .join("\n");
    // FNV-1a: stable across hosts and toolchains, unlike std's hashers.
    let hash = identity.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    let metadata = format!("metadata={hash:016x}");
    let mut i = 1;
    while i < args.len() {
        if text[i] == "-C" && text.get(i + 1).is_some_and(|v| v.starts_with("metadata=")) {
            args[i + 1] = metadata.clone().into();
            i += 2;
        } else if text[i].starts_with("-Cmetadata=") {
            args[i] = format!("-C{metadata}").into();
            i += 1;
        } else {
            i += 1;
        }
    }
    for remap in remaps.to_string_lossy().lines().filter(|l| !l.is_empty()) {
        args.push(format!("--remap-path-prefix={remap}").into());
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
