//! Project-locked persistence for already patched author sources.
//!
//! The validator checks the entire candidate overlay without writing files. This
//! module knows no author semantics: callers obtain text from `Source::patch` or
//! their provenance-aware equivalent, and supply evaluation/type/schema checks.
//!
//! Each rename replaces an existing file on Unix AND Windows (`std::fs::rename`
//! uses replacement semantics on both); there is never an unlink-before-rename
//! window. Multiple paths cannot be exchanged atomically by a normal filesystem.
//! Readers/writers needing a consistent batch must share the project lock. An
//! uncooperative editor can still race the final digest check and rename.
//!
//! A synced intent and original-byte backups allow the next lock holder to roll
//! back an interrupted save. Recovery only replaces files still containing this
//! transaction's bytes, never a third party's new bytes. Failed recovery leaves
//! the journal/backups and blocks further saves. Successful recovery also rejects
//! the incoming save: re-read/re-evaluate before retrying. This is recoverability,
//! not an absolute guarantee against hardware failure. Unix directory entries are
//! synced as well; Windows has no portable directory fsync in std, so power-loss
//! durability of renames there depends on the filesystem.

use super::generator::source::{Diagnostic, Location};
use super::source_path::{checked, relative};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A project-root-relative, canonical slash path, its original byte identity,
/// and the complete candidate source text. Only existing regular files change.
#[derive(Clone, Debug)]
pub struct FileEdit {
    pub file: String,
    pub digest: String,
    pub text: String,
}

/// Changed files only, sorted by project-relative path. Digests use exactly the
/// BLAKE3 byte identity returned by `generator::source::Source::digest`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedFile {
    pub file: String,
    pub digest: String,
}

const JOURNAL: &str = "source-transaction.json";
const PENDING: &str = "source-transaction.pending";
const STAGING: &str = ".og-source-edit-";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    files: Vec<Replacement>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    file: String,
    old: String,
    new: String,
    staging: String,
}

fn diagnostic(code: &'static str, file: &str, message: impl ToString) -> Diagnostic {
    Diagnostic {
        code,
        file: file.into(),
        path: String::new(),
        location: Location {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        },
        message: message.to_string(),
    }
}

fn io_error(file: &str, error: impl ToString) -> Vec<Diagnostic> {
    vec![diagnostic("source-io", file, error)]
}

fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn read_source(root: &Path, file: &str) -> io::Result<Vec<u8>> {
    let path = checked(root, file)?;
    if !fs::metadata(&path)?.is_file() {
        return Err(io::Error::other("source is not a regular file"));
    }
    fs::read(path)
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8], permissions: Option<fs::Permissions>) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()
}

fn lock(root: &Path) -> io::Result<File> {
    let dir = root.join(".og");
    match fs::create_dir(&dir) {
        Ok(()) => sync_directory(root)?,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    if !fs::symlink_metadata(&dir)?.is_dir() {
        return Err(io::Error::other(".og must be a directory, not a symlink"));
    }
    let path = dir.join("source-edit.lock");
    if fs::symlink_metadata(&path).is_ok_and(|m| !m.is_file()) {
        return Err(io::Error::other("source-edit.lock must be a regular file"));
    }
    // Persistent inode: deleting the lock file would let another process lock a
    // different inode. Separate opens serialize threads as well as processes.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.lock()?;
    Ok(file)
}

fn journal(root: &Path, intent: &Intent) -> io::Result<()> {
    let dir = root.join(".og");
    let pending = dir.join(PENDING);
    match fs::remove_file(&pending) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    write_new(&pending, &serde_json::to_vec(intent)?, None)?;
    fs::rename(&pending, dir.join(JOURNAL))?;
    sync_directory(&dir)
}

fn staging(root: &Path, replacement: &Replacement) -> io::Result<PathBuf> {
    let target = relative(&replacement.file)?;
    let stage = relative(&replacement.staging)?;
    if stage.parent() != target.parent()
        || !stage
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(STAGING))
    {
        return Err(io::Error::other("invalid transaction staging path"));
    }
    let path = checked(root, &replacement.staging)?;
    if !fs::symlink_metadata(&path)?.is_dir() {
        return Err(io::Error::other("staging path is not a directory"));
    }
    Ok(path)
}

fn cleanup(root: &Path, intent: &Intent) {
    for replacement in &intent.files {
        // Only our known files, never recursive deletion through a journal.
        if let Ok(path) = staging(root, replacement) {
            let _ = fs::remove_file(path.join("old"));
            let _ = fs::remove_file(path.join("new"));
            let _ = fs::remove_file(path.join("restore"));
            let _ = fs::remove_dir(path);
        }
    }
}

fn clear_journal(root: &Path) -> io::Result<()> {
    fs::remove_file(root.join(".og").join(JOURNAL))?;
    sync_directory(&root.join(".og"))
}

fn rollback(root: &Path, intent: &Intent) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    for replacement in intent.files.iter().rev() {
        let restore = || -> io::Result<()> {
            let bytes = read_source(root, &replacement.file)?;
            let current = digest(&bytes);
            if current == replacement.old {
                return sync_directory(
                    root.join(&replacement.file)
                        .parent()
                        .expect("source has parent"),
                );
            }
            if current != replacement.new {
                return Err(io::Error::other(
                    "source changed outside the transaction; preserved external edit",
                ));
            }
            let stage = staging(root, replacement)?;
            let backup = stage.join("old");
            let original = fs::read(&backup)?;
            if fs::symlink_metadata(&backup)?.file_type().is_symlink()
                || digest(&original) != replacement.old
            {
                return Err(io::Error::other("original-byte backup is invalid"));
            }
            // Never consume the backup: a crash before the rollback rename's
            // directory sync must still leave the original bytes recoverable.
            let restore = stage.join("restore");
            match fs::remove_file(&restore) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            write_new(
                &restore,
                &original,
                Some(fs::metadata(&backup)?.permissions()),
            )?;
            let target = checked(root, &replacement.file)?;
            if digest(&read_source(root, &replacement.file)?) != replacement.new {
                return Err(io::Error::other(
                    "source changed during rollback; preserved external edit",
                ));
            }
            fs::rename(restore, &target)?;
            sync_directory(target.parent().expect("source has parent"))?;
            #[cfg(test)]
            checkpoint("restored", &target)?;
            Ok(())
        };
        if let Err(error) = restore() {
            errors.push(diagnostic(
                "source-recovery-failed",
                &replacement.file,
                error,
            ));
        }
    }
    if errors.is_empty() {
        if let Err(error) = clear_journal(root) {
            // If unlink succeeded but the directory sync failed, re-establish
            // the intent before reporting failure. Source bytes are all old.
            let _ = journal(root, intent);
            errors.push(diagnostic("source-recovery-failed", "", error));
        } else {
            cleanup(root, intent);
        }
    }
    errors
}

fn recover(root: &Path) -> Result<(), Vec<Diagnostic>> {
    let path = root.join(".og").join(JOURNAL);
    let bytes = match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Ok(m) if m.is_file() => fs::read(&path).map_err(|e| io_error("", e))?,
        Ok(_) => return Err(io_error("", "transaction journal is not a regular file")),
        Err(e) => return Err(io_error("", e)),
    };
    let intent: Intent = serde_json::from_slice(&bytes).map_err(|e| io_error("", e))?;
    let errors = rollback(root, &intent);
    if errors.is_empty() {
        Err(vec![diagnostic(
            "recovered-transaction",
            "",
            "interrupted save rolled back; re-read and validate sources before retrying",
        )])
    } else {
        Err(errors)
    }
}

/// Validate and save a batch under one cross-process project lock. All inputs
/// and validator candidates are path-sorted; unchanged files are not rewritten.
/// Rejections carry no saved files. IO failures attempt conditional rollback;
/// `source-recovery-failed` means retained journal/backups need recovery on the
/// next call (and possibly manual resolution of external edits or IO failures).
///
/// `validate` must not write files or recursively call `commit` for this project.
/// It sees all proposed files, including no-ops, and must check them as an overlay
/// over the remaining project. All source digests are checked again afterwards,
/// after staging and immediately before the first source replacement.
pub fn commit(
    root: &Path,
    edits: &[FileEdit],
    validate: impl FnOnce(&[FileEdit]) -> Result<(), Vec<Diagnostic>>,
) -> Result<Vec<SavedFile>, Vec<Diagnostic>> {
    let root = root.canonicalize().map_err(|e| io_error("", e))?;
    let _lock = lock(&root).map_err(|e| io_error("", e))?;
    recover(&root)?;
    let mut edits = edits.to_vec();
    edits.sort_by(|a, b| a.file.cmp(&b.file));
    let mut identities = BTreeSet::new();
    let mut originals = Vec::with_capacity(edits.len());
    for edit in &edits {
        let path = checked(&root, &edit.file).map_err(|e| io_error(&edit.file, e))?;
        let identity = path.canonicalize().map_err(|e| io_error(&edit.file, e))?;
        #[cfg(windows)]
        let identity = PathBuf::from(identity.as_os_str().to_string_lossy().to_lowercase());
        if !identities.insert(identity) {
            return Err(vec![diagnostic(
                "duplicate-source",
                &edit.file,
                "a transaction may edit a source only once",
            )]);
        }
        let bytes = read_source(&root, &edit.file).map_err(|e| io_error(&edit.file, e))?;
        if digest(&bytes) != edit.digest {
            return Err(vec![diagnostic(
                "source-conflict",
                &edit.file,
                "source bytes changed; re-read and evaluate before saving",
            )]);
        }
        originals.push(bytes);
    }
    if let Err(errors) = validate(&edits) {
        return Err(if errors.is_empty() {
            vec![diagnostic(
                "source-validation",
                "",
                "candidate source validation rejected the save",
            )]
        } else {
            errors
        });
    }
    let mut intent = Intent { files: Vec::new() };
    let mut prepare = || -> Result<(), Vec<Diagnostic>> {
        for (edit, original) in edits.iter().zip(&originals) {
            if edit.text.as_bytes() == original {
                continue;
            }
            let target = checked(&root, &edit.file).map_err(|e| io_error(&edit.file, e))?;
            let permissions = fs::metadata(&target)
                .map_err(|e| io_error(&edit.file, e))?
                .permissions();
            if permissions.readonly() {
                return Err(io_error(&edit.file, "source is read-only"));
            }
            let parent = target.parent().expect("source has parent");
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let stage = loop {
                let name = format!(
                    "{STAGING}{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                );
                let stage = parent.join(name);
                match fs::create_dir(&stage) {
                    Ok(()) => break stage,
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(io_error(&edit.file, e)),
                }
            };
            intent.files.push(Replacement {
                file: edit.file.clone(),
                old: edit.digest.clone(),
                new: digest(edit.text.as_bytes()),
                staging: stage
                    .strip_prefix(&root)
                    .expect("stage in project")
                    .to_string_lossy()
                    .replace('\\', "/"),
            });
            write_new(&stage.join("old"), original, Some(permissions.clone()))
                .and_then(|()| {
                    write_new(&stage.join("new"), edit.text.as_bytes(), Some(permissions))
                })
                .and_then(|()| sync_directory(&stage))
                .and_then(|()| sync_directory(parent))
                .map_err(|e| io_error(&edit.file, e))?;
        }
        for edit in &edits {
            let current = read_source(&root, &edit.file).map_err(|e| io_error(&edit.file, e))?;
            if digest(&current) != edit.digest {
                return Err(vec![diagnostic(
                    "source-conflict",
                    &edit.file,
                    "source changed during validation/staging; save rejected",
                )]);
            }
        }
        Ok(())
    };
    if let Err(errors) = prepare() {
        cleanup(&root, &intent);
        return Err(errors);
    }
    if intent.files.is_empty() {
        return Ok(Vec::new());
    }
    // No source mutation until BOTH durable backups and the intent exist.
    if let Err(error) = journal(&root, &intent) {
        let mut errors = io_error("", error);
        if root.join(".og").join(JOURNAL).exists() {
            errors.extend(rollback(&root, &intent));
        } else {
            cleanup(&root, &intent);
        }
        return Err(errors);
    }
    let apply = || -> Result<(), Vec<Diagnostic>> {
        #[cfg(test)]
        checkpoint("journal", &root).map_err(|e| io_error("", e))?;
        // Journal IO can take time too. Check the WHOLE overlay, including
        // no-ops, once more before making the first source replacement.
        for edit in &edits {
            let current = read_source(&root, &edit.file).map_err(|e| io_error(&edit.file, e))?;
            if digest(&current) != edit.digest {
                return Err(vec![diagnostic(
                    "source-conflict",
                    &edit.file,
                    "source changed before commit",
                )]);
            }
        }
        for replacement in &intent.files {
            #[cfg(test)]
            checkpoint("replace", &root.join(&replacement.file))
                .map_err(|e| io_error(&replacement.file, e))?;
            // Catch later uncooperative changes too; never knowingly overwrite.
            let current = read_source(&root, &replacement.file)
                .map_err(|e| io_error(&replacement.file, e))?;
            if digest(&current) != replacement.old {
                return Err(vec![diagnostic(
                    "source-conflict",
                    &replacement.file,
                    "source changed during commit",
                )]);
            }
            let target =
                checked(&root, &replacement.file).map_err(|e| io_error(&replacement.file, e))?;
            let stage = staging(&root, replacement).map_err(|e| io_error(&replacement.file, e))?;
            fs::rename(stage.join("new"), &target)
                .and_then(|()| sync_directory(target.parent().expect("source has parent")))
                .map_err(|e| io_error(&replacement.file, e))?;
            #[cfg(test)]
            checkpoint("replaced", &target).map_err(|e| io_error(&replacement.file, e))?;
        }
        clear_journal(&root).map_err(|e| io_error("", e))
    };
    if let Err(mut errors) = apply() {
        // Re-establish a log if its unlink succeeded but directory sync failed.
        if !root.join(".og").join(JOURNAL).exists()
            && let Err(error) = journal(&root, &intent)
        {
            errors.push(diagnostic("source-recovery-failed", "", error));
        }
        errors.extend(rollback(&root, &intent));
        return Err(errors);
    }
    let saved = intent
        .files
        .iter()
        .map(|r| SavedFile {
            file: r.file.clone(),
            digest: r.new.clone(),
        })
        .collect();
    cleanup(&root, &intent);
    Ok(saved)
}

#[cfg(test)]
type Checkpoint = Box<dyn FnMut(&str, &Path) -> io::Result<()>>;
#[cfg(test)]
thread_local! {
    static CHECKPOINT: std::cell::RefCell<Option<Checkpoint>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
fn checkpoint(phase: &str, path: &Path) -> io::Result<()> {
    CHECKPOINT.with_borrow_mut(|hook| match hook {
        Some(hook) => hook(phase, path),
        None => Ok(()),
    })
}

#[cfg(test)]
#[path = "source_transaction_tests.rs"]
mod tests;
