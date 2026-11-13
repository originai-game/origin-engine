//! Per-project stat-to-content index. A clean cook needs no source byte reads.
//!
//! A stat proves a file unchanged only when it is older than the clock it is
//! compared under. File times advance in ticks (4 ms on ext4, 2 s on FAT), so
//! a file rewritten twice in one tick, with the same length, keeps its stat.
//! As git does for a racily clean index entry, a stat taken in the tick of the
//! file's last change is racy: it is never trusted alone, and the file, a
//! source or a cooked output, is compared by content until a later cook sees it
//! settled.
use super::{BeforeCode, CookOptions, Report, cache};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Stat {
    size: u64,
    modified: SystemTime,
    /// The inode change time, or `modified` where the platform has none.
    changed: SystemTime,
    identity: String,
}

impl Stat {
    /// Whether a rewrite at or after `clock` could leave this stat as it is.
    fn racy(&self, clock: SystemTime) -> bool {
        self.modified.max(self.changed) >= clock
    }
}

fn stat(path: &Path) -> std::io::Result<Stat> {
    let m = std::fs::metadata(path)?;
    let modified = m.modified()?;
    #[cfg(unix)]
    let (changed, identity) = {
        use std::os::unix::fs::MetadataExt;
        let changed = SystemTime::UNIX_EPOCH
            + std::time::Duration::new(m.ctime().try_into().unwrap_or(0), m.ctime_nsec() as u32);
        (changed, format!("{}:{}", m.dev(), m.ino()))
    };
    #[cfg(windows)]
    let (changed, identity) = {
        use std::os::windows::fs::MetadataExt;
        // 100 ns ticks since 1601; 11_644_473_600 s separate it from 1970.
        let changed = m
            .change_time()
            .and_then(|ticks| ticks.checked_sub(116_444_736_000_000_000))
            .map_or(modified, |ticks| {
                SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(ticks.saturating_mul(100))
            });
        (
            changed,
            format!("{:?}:{:?}", m.volume_serial_number(), m.file_index()),
        )
    };
    #[cfg(not(any(unix, windows)))]
    let (changed, identity) = (modified, format!("{:?}", m.created()));
    Ok(Stat {
        size: m.len(),
        modified,
        changed,
        identity,
    })
}

/// The project filesystem's clock: the time it gives a file written now. The
/// system clock does not compare with file times, which lag it by up to a tick.
fn clock(root: &Path) -> Option<SystemTime> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let probe = root.join(format!(
        ".og/cook-clock.{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(probe.parent()?).ok()?;
    std::fs::write(&probe, b"").ok()?;
    let time = std::fs::metadata(&probe).and_then(|m| m.modified()).ok();
    let _ = std::fs::remove_file(&probe);
    time
}

fn hash(path: &Path) -> std::io::Result<String> {
    Ok(blake3::hash(&std::fs::read(path)?).to_hex().to_string())
}

/// The content hash of each file whose stat is racy under `clock`.
fn racy(
    files: &BTreeMap<PathBuf, Stat>,
    clock: SystemTime,
) -> std::io::Result<BTreeMap<PathBuf, String>> {
    files
        .iter()
        .filter(|(_, stat)| stat.racy(clock))
        .map(|(path, _)| Ok((path.clone(), hash(path)?)))
        .collect()
}

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    stat: Stat,
    hash: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Saved {
    options: String,
    headers: BTreeMap<String, Option<(u64, u32)>>,
    inputs: BTreeMap<PathBuf, Stat>,
    outputs: BTreeMap<PathBuf, Stat>,
    entries: BTreeMap<PathBuf, Entry>,
    /// The content hash of each input and output whose saved stat was racy.
    /// No serde default: an index from before this field trusted every stat,
    /// and is dropped.
    racy: BTreeMap<PathBuf, String>,
    report: Option<Report>,
    /// What the checkpointed cook held before it read the project's code.
    #[serde(default)]
    before_code: Option<BeforeCode>,
}

pub(crate) struct Index {
    path: PathBuf,
    root: PathBuf,
    store: Option<cache::Store>,
    saved: Mutex<Saved>,
    /// The filesystem's time before any input was read. Without it (`.og/`
    /// cannot be written) or the inputs, nothing is indexed.
    clock: Option<SystemTime>,
    inputs: Option<BTreeMap<PathBuf, Stat>>,
    /// The racy inputs' content hashes, taken before the cook reads them: a
    /// rewrite the stat cannot show then differs from the hash that is saved.
    /// A racy output's hash is taken as the index is saved.
    racy: BTreeMap<PathBuf, String>,
    options: String,
}

fn tree(dir: &Path, out: &mut BTreeMap<PathBuf, Stat>) -> std::io::Result<()> {
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        let path = e.path();
        if e.file_type()?.is_dir() {
            tree(&path, out)?;
        } else if path.is_file() {
            out.insert(path.clone(), stat(&path)?);
        } else {
            return Err(std::io::Error::other("untracked asset directory link"));
        }
    }
    Ok(())
}

fn inputs(root: &Path) -> std::io::Result<BTreeMap<PathBuf, Stat>> {
    let mut files = BTreeMap::new();
    tree(&root.join("assets"), &mut files)?;
    let project = root.join("project.ogt");
    if project.exists() {
        files.insert(project.clone(), stat(&project)?);
    }
    for path in super::assets::game_typescript(root).map_err(std::io::Error::other)? {
        files.insert(path.clone(), stat(&path)?);
    }
    Ok(files)
}

fn outputs(root: &Path) -> std::io::Result<BTreeMap<PathBuf, Stat>> {
    let mut files = BTreeMap::new();
    tree(&root.join("cooked"), &mut files)?;
    let plan = root.join(".og/publish-assets.json");
    files.insert(plan.clone(), stat(&plan)?);
    Ok(files)
}

fn headers(report: &Report) -> BTreeMap<String, Option<(u64, u32)>> {
    report
        .assets
        .iter()
        .flat_map(|a| &a.outputs)
        .map(|o| (o.ty.clone(), super::format::expected_header(&o.ty)))
        .collect()
}

impl Index {
    pub fn open(root: &Path, options: &CookOptions, store: Option<&Path>) -> Self {
        let path = root.join(".og/cook-index");
        let saved = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let mut options = options.clone();
        options.threads = 0;
        let options = cache::key(
            "index-v1",
            &[format!(
                "{options:?} {:?}",
                super::texture::encoder_name(&options.texture)
            )
            .as_bytes()],
        );
        // The stats, then the clock, then the racy inputs' content: every read
        // of the cook comes after the clock, so a change the clock cannot
        // order against a stat is one the hash or the cook has already seen.
        let opened = inputs(root).ok().and_then(|inputs| {
            let clock = clock(root)?;
            let racy = racy(&inputs, clock).ok()?;
            Some((clock, inputs, racy))
        });
        let (clock, inputs, racy) = match opened {
            Some((clock, inputs, racy)) => (Some(clock), Some(inputs), racy),
            None => (None, None, BTreeMap::new()),
        };
        Self {
            path,
            root: root.to_owned(),
            store: store.map(|p| cache::Store::new(p.to_owned())),
            saved: Mutex::new(saved),
            clock,
            inputs,
            racy,
            options,
        }
    }

    /// The checkpointed cook's report, when nothing it read or wrote has
    /// changed since.
    pub fn unchanged(&self) -> Option<Report> {
        self.checkpoint(|_| false)
    }

    /// The checkpointed cook's report and what it held before it read the
    /// project's code, when code is all that has changed since: the asset
    /// tree and `project.ogt` are as they were, and so is what was written.
    pub fn only_code_changed(&self) -> Option<(Report, BeforeCode)> {
        let report = self.checkpoint(|path| {
            !path.starts_with(self.root.join("assets")) && path != self.root.join("project.ogt")
        })?;
        let before = self.saved.lock().expect("index lock").before_code.clone()?;
        Some((report, before))
    }

    /// The checkpointed cook's report as a cook that did nothing answers
    /// it, when the options and what was written are as they were, and so
    /// is every input but those `code` names.
    fn checkpoint(&self, code: impl Fn(&Path) -> bool) -> Option<Report> {
        let mut saved = self.saved.lock().expect("index lock");
        let now = self.inputs.as_ref()?;
        if saved.options != self.options
            || !now
                .iter()
                .filter(|(path, _)| !code(path))
                .eq(saved.inputs.iter().filter(|(path, _)| !code(path)))
            || outputs(&self.root).ok()? != saved.outputs
        {
            return None;
        }
        let mut report = saved.report.clone()?;
        if headers(&report) != saved.headers {
            return None;
        }
        // An equal racy stat proves nothing; the content decides.
        for (path, was) in &saved.racy {
            // Code that may differ is read again, whatever its stat says.
            if saved.inputs.contains_key(path) && code(path) {
                continue;
            }
            let now = match self.racy.get(path) {
                Some(now) => now.clone(),
                None => hash(path).ok()?,
            };
            if &now != was {
                return None;
            }
        }
        // What has settled since is trusted on its stat from here on.
        let clock = self.clock?;
        let Saved {
            racy,
            inputs,
            outputs,
            ..
        } = &mut *saved;
        let before = racy.len();
        racy.retain(|path, _| {
            inputs
                .get(path)
                .or_else(|| outputs.get(path))
                .is_some_and(|stat| stat.racy(clock))
        });
        if racy.len() != before
            && let Ok(bytes) = serde_json::to_vec(&*saved)
        {
            let _ = cache::atomic_write(&self.path, &bytes);
        }
        report.removed.clear();
        for asset in &mut report.assets {
            asset.notes.retain(|n| {
                !n.contains("shared texture cache hit") && !n.contains("shared cook cache hit")
            });
            if asset.outputs.iter().any(|o| o.ty == "og.Fracture")
                && !asset.notes.iter().any(|n| n.contains("reused prefracture"))
            {
                asset
                    .notes
                    .push("fracture: inputs unchanged, reused prefracture".into());
            }
            for output in &mut asset.outputs {
                output.cooked = false;
                output.written = false;
                output.ms = 0.0;
                output.start_ms = 0.0;
                output.worker = 0;
            }
        }
        Some(report)
    }

    pub fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        let before = stat(path)?;
        // The lock is let go before the content is read: files are read
        // from many threads at once.
        let known = self
            .saved
            .lock()
            .expect("index lock")
            .entries
            .get(path)
            .filter(|entry| entry.stat == before)
            .map(|entry| entry.hash.clone());
        if let Some(store) = &self.store
            && let Some(hash) = known
            && let Some(bytes) = store.content(&hash)
        {
            return Ok(bytes);
        }
        let bytes = std::fs::read(path)?;
        // A racy stat is not remembered, so an entry's stat is always proof.
        if self.clock.is_some_and(|clock| !before.racy(clock)) && stat(path)? == before {
            let hash = blake3::hash(&bytes).to_hex().to_string();
            if let Some(store) = &self.store {
                let _ = store.put_content(&bytes);
            }
            self.saved
                .lock()
                .expect("index lock")
                .entries
                .insert(path.to_owned(), Entry { stat: before, hash });
        }
        Ok(bytes)
    }

    pub fn content_key(&self) -> Option<String> {
        let mut content = Vec::new();
        for (path, expected) in self.inputs.as_ref()? {
            let known = self
                .saved
                .lock()
                .expect("index lock")
                .entries
                .get(path)
                .filter(|entry| &entry.stat == expected)
                .map(|entry| entry.hash.clone());
            let hash = match known {
                Some(hash) => hash,
                None => blake3::hash(&self.read(path).ok()?).to_hex().to_string(),
            };
            content.push((
                path.strip_prefix(&self.root)
                    .ok()?
                    .to_string_lossy()
                    .into_owned(),
                hash,
            ));
        }
        Some(cache::key(
            "project-inputs",
            &[&serde_json::to_vec(&content).ok()?],
        ))
    }

    /// Checkpoints a cook that held `before_code` before it read the
    /// project's code. `None` for a cook whose sources have dependencies
    /// outside the watched asset tree: they still use content-addressed
    /// actions, but must be planned again.
    pub fn save(&self, report: &Report, before_code: Option<BeforeCode>) {
        let mut saved = self.saved.lock().expect("index lock");
        saved.report = None;
        saved.before_code = None;
        // The clock comes before the outputs' stats here: nothing reads an
        // output again, so a rewrite between the two must show in the stat.
        if report.ok()
            && let Some(before_code) = before_code
            && let Some(before) = &self.inputs
            && inputs(&self.root).ok().as_ref() == Some(before)
            && let Some(clock) = clock(&self.root)
            && let Ok(outputs) = outputs(&self.root)
            && let Ok(mut racy) = racy(&outputs, clock)
        {
            racy.extend(self.racy.clone());
            saved.options.clone_from(&self.options);
            saved.headers = headers(report);
            saved.inputs.clone_from(before);
            saved.racy = racy;
            saved.outputs = outputs;
            saved.report = Some(report.clone());
            saved.before_code = Some(before_code);
        }
        if let Ok(bytes) = serde_json::to_vec(&*saved) {
            let _ = cache::atomic_write(&self.path, &bytes);
        }
    }
}
