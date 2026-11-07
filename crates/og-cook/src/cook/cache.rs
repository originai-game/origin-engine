//! Disposable native cook results: BLAKE3 action keys and authenticated content.
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// How often a cook trims the store ([`Store::gc_if_due`]).
pub const GC_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3600);

pub fn default_path() -> Option<PathBuf> {
    if let Some(path) =
        std::env::var_os("OG_COOK_CACHE").or_else(|| std::env::var_os("OG_TEXTURE_CACHE"))
    {
        return (!path.is_empty() && path != "off").then(|| PathBuf::from(path));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".cache/origingame/cook-v1"))
        .or_else(|| {
            std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("origingame/cook-v1"))
        })
}

pub fn max_bytes() -> u64 {
    std::env::var("OG_COOK_CACHE_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BYTES)
}

pub(super) fn key(kind: &str, parts: &[&[u8]]) -> String {
    let mut hash = blake3::Hasher::new();
    for part in [super::COOKER_VERSION.as_bytes(), kind.as_bytes()]
        .into_iter()
        .chain(parts.iter().copied())
    {
        hash.update(&(part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    hash.finalize().to_hex().to_string()
}

fn digest_name(name: &str) -> bool {
    name.len() == 64
        && name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Action {
    result: String,
    notes: Vec<String>,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn content(&self, digest: &str) -> Option<Vec<u8>> {
        if !digest_name(digest) {
            return None;
        }
        let bytes = std::fs::read(self.root.join("cas").join(digest)).ok()?;
        (blake3::hash(&bytes).to_hex().as_str() == digest).then_some(bytes)
    }

    pub fn put_content(&self, bytes: &[u8]) -> std::io::Result<String> {
        let digest = blake3::hash(bytes).to_hex().to_string();
        let path = self.root.join("cas").join(&digest);
        if self.content(&digest).is_none() {
            atomic_write(&path, bytes)?;
        }
        Ok(digest)
    }

    pub fn action(&self, key: &str) -> Option<(Vec<u8>, Vec<String>)> {
        if !digest_name(key) {
            return None;
        }
        let path = self.root.join("ac").join(key);
        let entry: Action = serde_json::from_slice(&std::fs::read(&path).ok()?).ok()?;
        let bytes = self.content(&entry.result)?;
        // Recency is disposable metadata, not part of a deterministic result.
        if let Ok(file) = std::fs::File::options().write(true).open(path) {
            let _ = file.set_modified(std::time::SystemTime::now());
        }
        Some((bytes, entry.notes))
    }

    pub fn put_action(&self, key: &str, bytes: &[u8], notes: &[String]) -> std::io::Result<()> {
        if !digest_name(key) {
            return Err(std::io::Error::other("invalid cook action key"));
        }
        let entry = Action {
            result: self.put_content(bytes)?,
            notes: notes.to_vec(),
        };
        atomic_write(
            &self.root.join("ac").join(key),
            &serde_json::to_vec(&entry).map_err(std::io::Error::other)?,
        )
    }

    /// [`Store::gc`] after a cook, at most once per [`GC_INTERVAL`] per store.
    /// A pass reads every action, so its cost grows with the store (about
    /// 5 s at 116k actions on macrunner); running it after every cook made
    /// each cook pay that. Between passes the store may pass its cap by what
    /// the interval's cooks write. `gc.stamp`'s modification time records the
    /// last pass; processes racing past a stale stamp may each run one.
    pub fn gc_if_due(&self, cap: u64) -> std::io::Result<Option<GcReport>> {
        let stamp = self.root.join("gc.stamp");
        let due = std::fs::metadata(&stamp)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_none_or(|age| age >= GC_INTERVAL);
        if !due {
            return Ok(None);
        }
        std::fs::create_dir_all(&self.root)?;
        std::fs::write(&stamp, b"")?;
        self.gc(cap).map(Some)
    }

    /// Evict oldest actions, then unreferenced contents, until at or below cap.
    /// A racing writer or reader can only cause a later cache miss.
    pub fn gc(&self, cap: u64) -> std::io::Result<GcReport> {
        let files = |dir: &Path| -> std::io::Result<Vec<(PathBuf, std::fs::Metadata)>> {
            let entries = match std::fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
                Err(e) => return Err(e),
            };
            Ok(entries
                .filter_map(Result::ok)
                .filter_map(|e| {
                    e.metadata()
                        .ok()
                        .filter(|m| m.is_file())
                        .map(|m| (e.path(), m))
                })
                .collect())
        };
        let mut actions = files(&self.root.join("ac"))?;
        let contents = files(&self.root.join("cas"))?;
        let mut refs = std::collections::BTreeMap::<String, usize>::new();
        let mut entries = std::collections::BTreeMap::new();
        for (path, _) in &actions {
            if path
                .file_name()
                .is_some_and(|n| digest_name(&n.to_string_lossy()))
                && let Ok(bytes) = std::fs::read(path)
                && let Ok(action) = serde_json::from_slice::<Action>(&bytes)
                && digest_name(&action.result)
            {
                *refs.entry(action.result.clone()).or_default() += 1;
                entries.insert(path.clone(), action.result);
            }
        }
        let before = actions.iter().chain(&contents).map(|(_, m)| m.len()).sum();
        let mut report = GcReport {
            before,
            after: before,
            removed: 0,
        };
        let mut removed = std::collections::BTreeSet::new();
        let remove = |path: &Path, size: u64, report: &mut GcReport| {
            if std::fs::remove_file(path).is_ok() {
                report.after = report.after.saturating_sub(size);
                report.removed += 1;
            }
        };
        for (path, meta) in actions.iter().chain(&contents) {
            if path.extension().is_some_and(|e| e == "tmp")
                && meta
                    .modified()
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|age| age.as_secs() > 3600)
            {
                remove(path, meta.len(), &mut report);
            }
        }
        // Inputs and interrupted publications may have no action yet. They are
        // disposable, but leave in-progress temporary files alone.
        for (path, meta) in &contents {
            let name = path.file_name().unwrap().to_string_lossy();
            if report.after > cap && digest_name(&name) && !refs.contains_key(name.as_ref()) {
                remove(path, meta.len(), &mut report);
                removed.insert(path.clone());
            }
        }
        actions.sort_by_key(|(path, m)| (m.modified().ok(), path.clone()));
        for (path, meta) in actions {
            if report.after <= cap {
                break;
            }
            if !path
                .file_name()
                .is_some_and(|n| digest_name(&n.to_string_lossy()))
            {
                continue;
            }
            remove(&path, meta.len(), &mut report);
            if let Some(digest) = entries.get(&path)
                && let Some(count) = refs.get_mut(digest)
            {
                *count -= 1;
                if *count == 0 {
                    let content = self.root.join("cas").join(digest);
                    if let Some((_, meta)) = contents.iter().find(|(p, _)| p == &content)
                        && removed.insert(content.clone())
                    {
                        remove(&content, meta.len(), &mut report);
                    }
                }
            }
        }
        Ok(report)
    }
}

#[derive(Debug, Serialize)]
pub struct GcReport {
    pub before: u64,
    pub after: u64,
    pub removed: usize,
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::fs::create_dir_all(path.parent().expect("cache directory"))?;
    let temp = path.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(bytes)?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
