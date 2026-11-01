//! The `og` binary as a user runs it: exit codes and what it prints.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn og() -> Command {
    Command::new(env!("CARGO_BIN_EXE_og"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A fresh directory that holds a copy of the repository's IDL, as a root.
fn scratch_root() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "og-cli-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let idl = dir.join(og_api_path());
    std::fs::create_dir_all(idl.parent().unwrap()).unwrap();
    std::fs::copy(repo().join(og_api_path()), &idl).unwrap();
    dir
}

fn og_api_path() -> &'static str {
    "crates/og-api/api.ogt"
}

/// A stand-in tsrs in `dir`: `--version` names it `version`, a check
/// prints `out` and exits `status`; `lib` puts lib.es2022.d.ts beside it.
#[cfg(unix)]
fn fake_tsrs(dir: &Path, version: &str, out: &str, status: i32, lib: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir).unwrap();
    let bin = dir.join("tsrs");
    std::fs::write(
        &bin,
        format!(
            "#!/bin/sh\nPATH=/usr/bin:/bin\nif [ \"$1\" = --version ]; then echo 'Version {version} (tsrs 0.0.0-{version}, test)'; exit 0; fi\necho \"$@\" > \"$(dirname \"$0\")/args.txt\"\ncat \"$(dirname \"$0\")/out.txt\"\nexit {status}\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("out.txt"), out).unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    if lib {
        std::fs::write(dir.join("lib.es2022.d.ts"), "").unwrap();
    }
    bin
}

mod cli {
    use super::*;

    mod check;
    mod maintenance;
    mod project;
    mod style;
}
