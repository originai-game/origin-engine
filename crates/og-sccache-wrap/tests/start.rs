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

/// A fake server: a process listening on a port of its own, running a
/// compiler (`busy`, a `sleep` child) or nothing. Returns it and its port.
#[cfg(unix)]
fn fake_server(busy: bool) -> (std::process::Child, u16) {
    use std::io::BufRead;
    use std::os::unix::process::CommandExt;
    let child_line = if busy {
        "import subprocess; c = subprocess.Popen(['sleep', '600'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL);"
    } else {
        ""
    };
    let mut server = Command::new("python3")
        .args([
            "-c",
            &format!(
                "import socket, time; {child_line} s = socket.socket(); \
                 s.bind(('127.0.0.1', 0)); s.listen(); \
                 print(s.getsockname()[1], flush=True); time.sleep(600)"
            ),
        ])
        .stdout(std::process::Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("python3");
    let mut line = String::new();
    std::io::BufReader::new(server.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    (server, line.trim().parse().unwrap())
}

/// Runs the wrapper against `server` with a 1 s stall window and a fake
/// sccache client that waits on the server until it stops listening (as a client
/// waits on a server whose jobserver ran dry) or `patience` seconds pass,
/// then compiles "locally" and exits 7. Returns the exit status, the
/// client's log, whether the server is still running and the watchdog log.
#[cfg(unix)]
fn run_stalled(name: &str, busy: bool, patience: u32) -> (i32, String, bool, String) {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch(name);
    let (mut server, port) = fake_server(busy);
    let log = dir.join("log");
    let fake = dir.join("sccache");
    std::fs::write(
        &fake,
        format!(
            // One process, as a real client waiting on its server is: a
            // child would read as a compile of its own.
            "#!/usr/bin/env python3\nimport os, socket, sys, time\n\
             for _ in range({ticks}):\n\
             \x20   try: socket.create_connection(('127.0.0.1', {port})).close()\n\
             \x20   except OSError: break\n\
             \x20   time.sleep(0.1)\n\
             open('{log}', 'a').write('run ' + ' '.join(sys.argv[1:]) + ' IGNORE=' + os.environ.get('SCCACHE_IGNORE_SERVER_IO_ERROR', '') + '\\n')\n\
             sys.exit(7)\n",
            ticks = patience * 10,
            log = log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    let status = Command::new(WRAP)
        .args(["rustc", "--crate-name", "x"])
        .env("OG_SCCACHE", &fake)
        .env("OG_SCCACHE_STATE", dir.join("state"))
        .env("OG_SCCACHE_STALL", "1")
        .env("SCCACHE_SERVER_PORT", port.to_string())
        .status()
        .unwrap();
    let alive = server.try_wait().unwrap().is_none();
    // The server and its `sleep`: its whole process group. Not `kill -KILL
    // -<pgid>`: procps-ng 4.0.4's kill (Ubuntu 24.04) answers 0 to it and
    // signals nothing, and the server left behind held the test's output
    // open until nextest's timeout.
    let _ = Command::new("pkill")
        .args(["-KILL", "-g", &server.id().to_string()])
        .stderr(std::process::Stdio::null())
        .status();
    let _ = server.wait();
    (
        status.code().unwrap_or(-1),
        std::fs::read_to_string(&log).unwrap_or_default(),
        alive,
        std::fs::read_to_string(dir.join("state/watchdog.log")).unwrap_or_default(),
    )
}

/// The hang of 2026-11-09 (second): a server whose jobserver ran dry runs
/// no compiler while clients wait. The wrapper stops it, and its compile
/// finishes with its own status (locally, told to ignore the lost server).
/// Ways it could fail: the watch never fires, fires without stopping the
/// server, or loses the compile's status.
#[cfg(unix)]
#[test]
fn a_server_running_no_compiler_while_a_compile_waits_is_stopped() {
    let (code, log, alive, watchdog) = run_stalled("stalled", false, 60);
    assert!(!alive, "the stalled server must be stopped");
    assert!(watchdog.contains("stopped sccache server"), "{watchdog}");
    assert_eq!(log.trim_end(), "run rustc --crate-name x IGNORE=1");
    assert_eq!(code, 7, "the compile's exit status");
}

/// A server running a compiler is busy, not stalled, however long a
/// compile waits on it. Way it could fail: the watch stops a healthy
/// server under every lane's builds.
#[cfg(unix)]
#[test]
fn a_server_running_a_compiler_is_left_alone() {
    let (code, log, alive, watchdog) = run_stalled("busy", true, 4);
    assert!(alive, "a busy server must keep running: {watchdog}");
    assert!(watchdog.is_empty(), "{watchdog}");
    assert_eq!(log.trim_end(), "run rustc --crate-name x IGNORE=1");
    assert_eq!(code, 7);
}
