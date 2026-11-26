//! The `og` binary as a user runs it: exit codes and what it prints.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
#[path = "../../../tests/temp.rs"]
mod temp;
use temp::TempDir;

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
fn scratch_root() -> TempDir {
    let dir = TempDir::new("og-cli-test");
    let idl = dir.join(og_api_path());
    std::fs::create_dir_all(&idl).unwrap();
    for entry in std::fs::read_dir(repo().join(og_api_path())).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), idl.join(entry.file_name())).unwrap();
    }
    dir
}

fn og_api_path() -> &'static str {
    "crates/og-api/api"
}

mod cli {
    use super::*;

    mod create;
    mod generating;
    mod maintenance;
    mod model;
    mod probe;
    mod ui;
}
