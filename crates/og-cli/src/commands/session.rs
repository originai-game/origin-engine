//! An optional worktree-local host. Refusal falls back before execution;
//! loss of an accepted request never retries a command with side effects.

use std::path::Path;
use std::process::{Command, ExitCode};

#[cfg(unix)]
mod local {
    use super::*;
    use crate::project::{self, Project};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use std::fs::{self, File, OpenOptions};
    use std::hash::{Hash, Hasher};
    use std::io::{BufRead, BufReader, Read, Seek, Write};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::process::Stdio;

    /// One host per worktree and environment. A host serves only commands
    /// with its own environment (session.ts), so commands that differ in it
    /// (a test's OG_RENDERER, a terminal's variables) each start their own
    /// host instead of finding another's and running alone. The variables
    /// the shell changes per command are left out, as session.ts does.
    pub(crate) fn directory(project: &Path) -> Result<PathBuf, String> {
        let worktree = project
            .ancestors()
            .find(|path| path.join(".git").exists())
            .unwrap_or(project);
        let mut hash = std::hash::DefaultHasher::new();
        worktree.hash(&mut hash);
        std::env::vars_os()
            .filter(|(key, _)| !["_", "SHLVL", "PWD", "OLDPWD"].iter().any(|k| key == k))
            .collect::<BTreeMap<_, _>>()
            .hash(&mut hash);
        let key = format!("{:016x}", hash.finish());
        // Both calls only read plain OS identity/address data. sockaddr_un
        // has integer fields and an integer array, so all-zero is valid.
        let uid = unsafe { libc::geteuid() };
        let limit = unsafe { std::mem::zeroed::<libc::sockaddr_un>() }
            .sun_path
            .len();
        let candidates = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .into_iter()
            .chain([std::env::temp_dir(), PathBuf::from("/tmp")]);
        let mut reason = "no absolute runtime directory".to_string();
        for base in candidates.filter(|base| base.is_absolute()) {
            let root = base.join(format!("og-{uid}"));
            let dir = root.join(&key);
            // sun_path includes the terminating NUL. Count encoded bytes,
            // not characters; Darwin's limit is smaller than Linux's.
            if dir.join("host.sock").as_os_str().as_bytes().len() >= limit {
                reason = format!("{} exceeds sun_path's {limit} bytes", dir.display());
                continue;
            }
            let private = || -> std::io::Result<()> {
                match fs::DirBuilder::new().mode(0o700).create(&root) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e),
                }
                let metadata = fs::symlink_metadata(&root)?;
                if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
                    return Err(std::io::Error::other(
                        "session runtime directory is not private to this user",
                    ));
                }
                Ok(())
            };
            match private() {
                Ok(()) => return Ok(dir),
                Err(e) => reason = format!("{}: {e}", root.display()),
            }
        }
        Err(format!(
            "cannot place session socket within sun_path's {limit}-byte limit: {reason}"
        ))
    }

    // A local build identity, not a cook-cache key. Replaced binaries and
    // edited host sources invalidate the host, even at the same version.
    pub(crate) fn build(package: &Path) -> Result<String, String> {
        fn stamp(path: &Path, hash: &mut impl Hasher) -> std::io::Result<()> {
            path.hash(hash);
            let m = fs::metadata(path)
                .map_err(|e| std::io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
            (
                m.dev(),
                m.ino(),
                m.len(),
                m.mtime(),
                m.mtime_nsec(),
                m.ctime(),
                m.ctime_nsec(),
            )
                .hash(hash);
            if m.is_dir() {
                let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
                entries.sort_by_key(|entry| entry.file_name());
                for entry in entries {
                    stamp(&entry.path(), hash)?;
                }
            }
            Ok(())
        }
        let mut hash = std::hash::DefaultHasher::new();
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        stamp(&exe, &mut hash).map_err(|e| e.to_string())?;
        stamp(&package.join("src"), &mut hash).map_err(|e| e.to_string())?;
        let os = if cfg!(target_os = "macos") {
            "darwin"
        } else {
            std::env::consts::OS
        };
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x64"
        };
        let addon = std::env::var_os("OG_NATIVE_ADDON")
            .map(PathBuf::from)
            .unwrap_or_else(|| package.join(format!("native/origin-engine.{os}-{arch}.node")));
        stamp(&addon, &mut hash).map_err(|e| e.to_string())?;
        Ok(format!("{:016x}", hash.finish()))
    }

    #[derive(Deserialize)]
    #[serde(tag = "event", rename_all = "lowercase")]
    enum Event {
        Accepted,
        Busy,
        Stdout { text: String },
        Stderr { text: String },
        Exit { code: u8 },
        Rejected { reason: String },
    }

    pub(crate) fn connect(project: &Path) -> Result<UnixStream, String> {
        use std::os::unix::process::CommandExt;
        use std::time::{Duration, Instant};
        let dir = directory(project)?;
        let socket = dir.join("host.sock");
        match UnixStream::connect(&socket) {
            Ok(stream) => return Ok(stream),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) => {}
            Err(e) => return Err(e.to_string()),
        }
        // Start races converge on the supervisor's inherited file lock.
        // Detached descriptors keep a caller's output pipe from staying open.
        // An unlinked file captures startup errors without a pipe held open
        // for the lifetime of the resident host, or competing starters sharing
        // and truncating each other's diagnostics.
        use std::sync::atomic::{AtomicU64, Ordering};
        static START: AtomicU64 = AtomicU64::new(0);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let log = dir.join(format!(
            "startup-{}-{}.log",
            std::process::id(),
            START.fetch_add(1, Ordering::Relaxed)
        ));
        let mut errors = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(&log)
            .map_err(|e| e.to_string())?;
        fs::remove_file(&log).map_err(|e| e.to_string())?;
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg("session")
            .arg(project)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errors.try_clone().map_err(|e| e.to_string())?))
            .process_group(0)
            .spawn()
            .map_err(|e| format!("cannot start session: {e}"))?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Ok(stream) = UnixStream::connect(&socket) {
                // Reap this supervisor if the caller is long lived; it is
                // otherwise adopted by the OS when the CLI exits.
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                return Ok(stream);
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                // A losing starter exits while the winner owns this lock.
                // Keep waiting for that host, not the exited starter.
                let owner = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(dir.join("host.lock"));
                if !owner.is_ok_and(|file| file.try_lock().is_err()) {
                    errors.rewind().map_err(|e| e.to_string())?;
                    let mut reason = String::new();
                    errors
                        .take(4096)
                        .read_to_string(&mut reason)
                        .map_err(|e| e.to_string())?;
                    return Err(format!(
                        "session exited before becoming ready ({status}); no command was sent: {}",
                        reason.trim()
                    ));
                }
            }
            if Instant::now() >= deadline {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                return Err(
                    "session did not become ready within 10 seconds; no command was sent".into(),
                );
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn try_run(
        project: &Path,
        package: &Path,
        command: &Command,
    ) -> Result<Option<ExitCode>, String> {
        if std::env::var("OG_SESSION").as_deref() == Ok("0")
            || !package.join("src/node/session.ts").is_file()
        {
            return Ok(None);
        }
        let argv = command
            .get_args()
            .skip(1)
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        if argv.iter().any(|arg| {
            ["--port", "--watch", "--room", "--handoff", "--realtime"].contains(&arg.as_str())
        }) {
            return Ok(None);
        }
        let kind = if command.get_args().next().is_some_and(|script| {
            Path::new(script).file_name() == Some(std::ffi::OsStr::new("ui-run.ts"))
        }) {
            "ui"
        } else {
            "run"
        };
        let mut socket = connect(project)?;
        let call = serde_json::json!({
            "kind": kind,
            "build": build(package)?,
            "cwd": std::env::current_dir().map_err(|e| e.to_string())?,
            "env": std::env::vars().collect::<BTreeMap<_, _>>(),
            "argv": argv,
        });
        writeln!(socket, "{call}").map_err(|e| {
            format!("session request interrupted; outcome unknown, not retried: {e}")
        })?;
        let mut accepted = false;
        for line in BufReader::new(socket).take(20 * 1024 * 1024).lines() {
            let line = line
                .map_err(|e| format!("session disconnected; outcome unknown, not retried: {e}"))?;
            let event: Event = serde_json::from_str(&line)
                .map_err(|e| format!("invalid session reply; not retried: {e}"))?;
            match event {
                Event::Accepted if !accepted => accepted = true,
                Event::Accepted => return Err("session accepted twice; not retried".into()),
                Event::Busy if !accepted => {}
                Event::Busy => return Err("session busy after accepting; not retried".into()),
                Event::Stdout { text } => std::io::stdout()
                    .write_all(text.as_bytes())
                    .map_err(|e| e.to_string())?,
                Event::Stderr { text } => std::io::stderr()
                    .write_all(text.as_bytes())
                    .map_err(|e| e.to_string())?,
                Event::Exit { code } => return Ok(Some(ExitCode::from(code))),
                Event::Rejected { .. } if accepted => {
                    return Err(
                        "session rejected after accepting; outcome unknown, not retried".into(),
                    );
                }
                Event::Rejected { reason } => {
                    eprintln!("og session: {reason}; running alone");
                    return Ok(None);
                }
            }
        }
        Err("session disconnected before its result; outcome unknown, not retried".into())
    }

    pub fn start(args: &[&str]) -> Result<ExitCode, String> {
        let path = match args {
            [] => ".",
            [path] if !path.starts_with('-') => path,
            _ => return Err("usage: og session [project]".into()),
        };
        let root = if Path::new(path).is_dir() {
            Path::new(path).canonicalize().map_err(|e| e.to_string())?
        } else {
            Project::open(Path::new(path))?.root
        };
        let package = project::engine_package(Some(&root)).ok_or(project::NO_ENGINE)?;
        let dir = directory(&root)?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
        let lock: File = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(dir.join("host.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock()
            .map_err(|e| format!("another session owns this worktree: {e}"))?;
        let socket = dir.join("host.sock");
        if UnixStream::connect(&socket).is_ok() {
            return Err("a session still owns the socket".into());
        }
        // The child owns the locked open file as stdin. A killed supervisor
        // cannot release the lock while its host is still alive.
        loop {
            if socket.exists() {
                fs::remove_file(&socket).map_err(|e| e.to_string())?;
            }
            let inputs = dir.join("inputs");
            if inputs.exists() {
                fs::remove_dir_all(&inputs).map_err(|e| e.to_string())?;
            }
            let status = project::bun(&package, "src/node/session.ts")
                .arg(&socket)
                .arg(build(&package)?)
                .stdin(Stdio::from(lock.try_clone().map_err(|e| e.to_string())?))
                .status()
                .map_err(|e| format!("{}: {e}", project::NO_BUN))?;
            // Retire Bun's module cache after a bounded number of runs. A
            // crash is reported, never automatically replayed or restarted.
            if status.code() != Some(75) {
                let _ = fs::remove_file(&socket);
                let _ = fs::remove_dir_all(&inputs);
                return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
            }
        }
    }
}

pub fn try_run(
    project: &Path,
    package: &Path,
    command: &Command,
) -> Result<Option<ExitCode>, String> {
    #[cfg(unix)]
    return local::try_run(project, package, command);
    #[cfg(not(unix))]
    {
        let _ = (project, package, command);
        Ok(None)
    }
}

/// Give each playtest harness the same host without changing its environment.
pub fn playtest_args(project: &Path, package: &Path) -> Result<Vec<std::ffi::OsString>, String> {
    #[cfg(unix)]
    {
        if std::env::var("OG_SESSION").as_deref() != Ok("0")
            && package.join("src/node/session.ts").is_file()
        {
            let socket = local::directory(project)?.join("host.sock");
            drop(local::connect(project)?);
            return Ok(vec![
                "--session-socket".into(),
                socket.into_os_string(),
                "--session-build".into(),
                local::build(package)?.into(),
            ]);
        }
    }
    #[cfg(not(unix))]
    let _ = (project, package);
    Ok(Vec::new())
}

pub fn run(args: &[&str]) -> ExitCode {
    #[cfg(unix)]
    let result = local::start(args);
    #[cfg(not(unix))]
    let result: Result<ExitCode, String> = {
        let _ = args;
        Err("resident sessions are not available on this host; commands run alone".into())
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("og session: {error}");
            ExitCode::FAILURE
        }
    }
}
