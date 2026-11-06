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
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::process::Stdio;

    pub(super) fn directory(project: &Path) -> PathBuf {
        let worktree = project
            .ancestors()
            .find(|path| path.join(".git").exists())
            .unwrap_or(project);
        let mut hash = std::hash::DefaultHasher::new();
        worktree.hash(&mut hash);
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| project.to_path_buf())
            .join(".cache/og/sessions")
            .join(format!("{:016x}", hash.finish()))
    }

    // A local build identity, not a cook-cache key. Replaced binaries and
    // edited host sources invalidate the host, even at the same version.
    pub(super) fn build(package: &Path) -> Result<String, String> {
        fn stamp(path: &Path, hash: &mut impl Hasher) -> std::io::Result<()> {
            path.hash(hash);
            let m = fs::metadata(path)?;
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

    pub fn try_run(
        project: &Path,
        package: &Path,
        command: &Command,
    ) -> Result<Option<ExitCode>, String> {
        if std::env::var("OG_SESSION").as_deref() == Ok("0") {
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
        let mut socket = match UnixStream::connect(directory(project).join("host.sock")) {
            Ok(socket) => socket,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error.to_string()),
        };
        let call = serde_json::json!({
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
        let project = Project::open(Path::new(path))?;
        let package = project::engine_package(Some(&project.root)).ok_or(project::NO_ENGINE)?;
        let dir = directory(&project.root);
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
        let socket = local::directory(project).join("host.sock");
        if std::env::var("OG_SESSION").as_deref() != Ok("0") && socket.exists() {
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
