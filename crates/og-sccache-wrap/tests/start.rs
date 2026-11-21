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
//! - on Windows, a compile whose argfile expands past the command line
//!   limit run through sccache (os error 206), or one that fits not →
//!   `an_argfile_too_long_for_one_command_line_skips_sccache`,
//!   `an_argfile_that_fits_goes_through_sccache`
//! - on Windows, a command line cargo passed whole that the release
//!   remaps take past the limit spawned as it is (os error 206) →
//!   `a_command_line_too_long_with_the_remaps_goes_in_a_response_file`

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

/// os error 206 (studio-win, 2026-11-19): cargo passes a long command line
/// as `rustc @argfile`, and sccache spawns it expanded, past Windows'
/// limit. Runs the wrapper with a fake rustc and an argfile of `lines`
/// arguments of 100 characters; returns the exit status and the log.
#[cfg(windows)]
fn run_argfile(name: &str, lines: usize) -> (i32, String) {
    let dir = scratch(name);
    let log = dir.join("log");
    let fake = fake_sccache(&dir, &log);
    let rustc = dir.join("rustc.cmd");
    std::fs::write(
        &rustc,
        format!(
            "@echo off\r\necho rustc %*>> \"{}\"\r\nexit /b 5\r\n",
            log.display()
        ),
    )
    .unwrap();
    let argfile = dir.join("cargo-argfile");
    let arg = format!("-Ldependency={}", "x".repeat(87));
    std::fs::write(&argfile, vec![arg; lines].join("\n")).unwrap();
    let (p, _listener) = port(true);
    let status = Command::new(WRAP)
        .arg(&rustc)
        .arg(format!("@{}", argfile.display()))
        .env("OG_SCCACHE", &fake)
        .env("OG_SCCACHE_STATE", dir.join("state"))
        .env("SCCACHE_SERVER_PORT", p.to_string())
        .status()
        .unwrap();
    (
        status.code().unwrap_or(-1),
        std::fs::read_to_string(&log).unwrap_or_default(),
    )
}

/// An argfile that expands past the limit is passed to rustc unexpanded,
/// not through sccache. Ways it could fail: sccache still runs it (206 in
/// a real build), the argfile is dropped or expanded, or rustc's exit
/// status is lost.
#[cfg(windows)]
#[test]
fn an_argfile_too_long_for_one_command_line_skips_sccache() {
    // 330 lines of 103 characters with quotes and a space: 33,990.
    let (code, log) = run_argfile("argfile-long", 330);
    assert!(!log.contains("run "), "sccache must not run it: {log}");
    let line = log.lines().next().unwrap_or_default();
    assert!(
        line.starts_with("rustc @") && line.trim_end().ends_with("cargo-argfile"),
        "rustc gets cargo's argfile as it is: {log}"
    );
    assert_eq!(code, 5, "rustc's exit status");
}

/// A command line cargo passed whole that the wrapper's remaps take past
/// the limit (os error 206, studio-win 2026-11-21: ui-rewrite's 3D lanes
/// og-host-web) reaches rustc in a response file holding every argument,
/// the remaps included. Ways it could fail: the spawn fails with 206,
/// sccache runs it, an argument is lost or the file left behind, or
/// rustc's exit status is lost.
#[cfg(windows)]
#[test]
fn a_command_line_too_long_with_the_remaps_goes_in_a_response_file() {
    let dir = scratch("response-file");
    let log = dir.join("log");
    let fake = fake_sccache(&dir, &log);
    let rustc = dir.join("rustc.cmd");
    // Logs its arguments, then the response file's lines.
    std::fs::write(
        &rustc,
        format!(
            "@echo off\r\necho rustc %*>> \"{log}\"\r\nset f=%1\r\ntype \"%f:~1%\">> \"{log}\"\r\nexit /b 5\r\n",
            log = log.display()
        ),
    )
    .unwrap();
    // 314 arguments of 100 characters, 32,342 with quotes: this spawn of
    // the wrapper fits in 32,767; the remaps below take rustc's past it.
    let deps: Vec<String> = (0..314)
        .map(|i| format!("-Ldependency={i:03}{}", "x".repeat(84)))
        .collect();
    let (p, _listener) = port(true);
    let status = Command::new(WRAP)
        .arg(&rustc)
        .args(["--crate-name", "x", "--target", "wasm32-unknown-unknown"])
        .args(&deps)
        .env("OG_SCCACHE", &fake)
        .env("OG_SCCACHE_STATE", dir.join("state"))
        .env("SCCACHE_SERVER_PORT", p.to_string())
        .env(
            "OG_WASM_REPRODUCIBLE",
            format!("/a/.cargo=/cargo\n/a/{}=/origin-engine", "r".repeat(500)),
        )
        .status()
        .unwrap();
    let out = std::fs::read_to_string(&log).unwrap_or_default();
    let mut lines = out.lines();
    let first = lines.next().unwrap_or_default();
    assert!(!out.contains("run "), "sccache must not run it: {first}");
    assert!(
        first.starts_with("rustc @") && first.trim_end().ends_with(".args"),
        "rustc gets one response file: {first}"
    );
    let file: Vec<&str> = lines.collect();
    assert_eq!(
        &file[..4],
        ["--crate-name", "x", "--target", "wasm32-unknown-unknown"]
    );
    assert_eq!(
        &file[4..318],
        deps.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert_eq!(file.len(), 320, "{} lines", file.len());
    assert_eq!(file[318], "--remap-path-prefix=/a/.cargo=/cargo");
    assert!(
        file[319].starts_with("--remap-path-prefix=/a/rrr"),
        "{}",
        file[319]
    );
    let left = first.trim_end().trim_start_matches("rustc @");
    assert!(!Path::new(left).exists(), "the response file is removed");
    assert_eq!(status.code(), Some(5), "rustc's exit status");
}

/// An argfile that fits still compiles through sccache. Way it could fail:
/// every compile with an argfile loses the cache.
#[cfg(windows)]
#[test]
fn an_argfile_that_fits_goes_through_sccache() {
    // 250 lines: 25,750.
    let (code, log) = run_argfile("argfile-short", 250);
    assert!(log.starts_with("run "), "{log}");
    assert!(log.contains("cargo-argfile"), "{log}");
    assert_eq!(code, 7, "sccache's exit status");
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

/// The release wasm's units (`OG_WASM_REPRODUCIBLE`, scripts/release-wasm.sh)
/// get a crate id of their own identity, whatever cargo's host-dependent
/// `-C metadata` was, and the remaps; a host unit keeps cargo's. Fails if
/// the rewrite stops, keeps cargo's value, or reaches a host build.
#[test]
fn a_release_wasm_unit_gets_a_host_free_crate_id_and_the_remaps() {
    let dir = scratch("reproducible");
    let (p, _listener) = port(true);
    let log = dir.join("log");
    let fake = fake_sccache(&dir, &log);
    let compile = |package: &str, metadata: &str, target: &[&str]| {
        let _ = std::fs::remove_file(&log);
        Command::new(WRAP)
            .args(["rustc", "--crate-name", "x", "--cfg", "feature=\"a\""])
            .args(target)
            .args(["-C", &format!("metadata={metadata}"), "-C", "opt-level=s"])
            .env("OG_SCCACHE", &fake)
            .env("OG_SCCACHE_STATE", dir.join("state"))
            .env("SCCACHE_SERVER_PORT", p.to_string())
            .env(
                "OG_WASM_REPRODUCIBLE",
                "/home/a/.cargo=/cargo\n/home/a/repo=/origin-engine",
            )
            .env("CARGO_PKG_NAME", package)
            .env("CARGO_PKG_VERSION", "1.0.0")
            .status()
            .unwrap();
        std::fs::read_to_string(&log).unwrap().trim_end().to_owned()
    };
    let wasm = ["--target", "wasm32-unknown-unknown"];
    let linux = compile("x", "1111", &wasm);
    let mac = compile("x", "2222", &wasm);
    assert_eq!(linux, mac, "cargo's metadata must not reach the crate id");
    assert!(!linux.contains("metadata=1111"), "{linux}");
    assert!(
        linux.contains("--remap-path-prefix=/home/a/.cargo=/cargo --remap-path-prefix=/home/a/repo=/origin-engine"),
        "{linux}"
    );
    assert_ne!(
        compile("y", "1111", &wasm),
        linux,
        "another package gets another crate id"
    );
    let host = compile("x", "1111", &[]);
    assert!(
        host.contains("-C metadata=1111") && !host.contains("remap"),
        "a host unit is left alone: {host}"
    );
}
