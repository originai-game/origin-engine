//! Canonical source identities shared by author discovery and persistence.
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

// Reject aliases rather than normalizing them: caller identities and results
// must name one stable project path, independently of the host OS.
pub(super) fn relative(file: &str) -> io::Result<&Path> {
    let path = Path::new(file);
    if file.is_empty()
        || file.contains(['\\', ':', '\0'])
        || file
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == ".." || s.ends_with(['.', ' ']))
        || !path.components().all(|c| matches!(c, Component::Normal(_)))
        || file
            .split('/')
            .next()
            .is_some_and(|s| s.eq_ignore_ascii_case(".og"))
    {
        return Err(io::Error::other(
            "expected a canonical project-relative source path outside .og",
        ));
    }
    Ok(path)
}

pub(super) fn checked(root: &Path, file: &str) -> io::Result<PathBuf> {
    let mut path = root.to_path_buf();
    for component in relative(file)?.components() {
        path.push(component);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(io::Error::other(
                "symlink sources and parent directories are not writable",
            ));
        }
    }
    Ok(path)
}
