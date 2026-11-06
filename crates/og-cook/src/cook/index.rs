//! Per-project stat-to-content index. A clean cook needs no source byte reads.
use super::{CookOptions, Report, cache};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Stat {
    size: u64,
    modified: std::time::SystemTime,
    identity: String,
}

fn stat(path: &Path) -> std::io::Result<Stat> {
    let m = std::fs::metadata(path)?;
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        format!("{}:{}:{}:{}", m.dev(), m.ino(), m.ctime(), m.ctime_nsec())
    };
    #[cfg(windows)]
    let identity = {
        use std::os::windows::fs::MetadataExt;
        format!(
            "{:?}:{:?}:{:?}",
            m.volume_serial_number(),
            m.file_index(),
            m.change_time()
        )
    };
    #[cfg(not(any(unix, windows)))]
    let identity = format!("{:?}", m.created());
    Ok(Stat {
        size: m.len(),
        modified: m.modified()?,
        identity,
    })
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
    report: Option<Report>,
}

pub(super) struct Index {
    path: PathBuf,
    root: PathBuf,
    store: Option<cache::Store>,
    saved: Mutex<Saved>,
    inputs: Option<BTreeMap<PathBuf, Stat>>,
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
        Self {
            path,
            root: root.to_owned(),
            store: store.map(|p| cache::Store::new(p.to_owned())),
            saved: Mutex::new(saved),
            inputs: inputs(root).ok(),
            options,
        }
    }

    pub fn unchanged(&self) -> Option<Report> {
        let saved = self.saved.lock().expect("index lock");
        if saved.options != self.options
            || self.inputs.as_ref()? != &saved.inputs
            || outputs(&self.root).ok()? != saved.outputs
        {
            return None;
        }
        let mut report = saved.report.clone()?;
        if headers(&report) != saved.headers {
            return None;
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
        if let Some(store) = &self.store
            && let Some(entry) = self.saved.lock().expect("index lock").entries.get(path)
            && entry.stat == before
            && let Some(bytes) = store.content(&entry.hash)
        {
            return Ok(bytes);
        }
        let bytes = std::fs::read(path)?;
        if stat(path)? == before {
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

    /// Do not checkpoint sources with dependencies outside the watched asset
    /// tree. They still use content-addressed actions, but must be planned again.
    pub fn save(&self, report: &Report, bounded: bool) {
        let mut saved = self.saved.lock().expect("index lock");
        saved.report = None;
        if report.ok()
            && bounded
            && let Some(before) = &self.inputs
            && inputs(&self.root).ok().as_ref() == Some(before)
            && let Ok(outputs) = outputs(&self.root)
        {
            saved.options.clone_from(&self.options);
            saved.headers = headers(report);
            saved.inputs.clone_from(before);
            saved.outputs = outputs;
            saved.report = Some(report.clone());
        }
        if let Ok(bytes) = serde_json::to_vec(&*saved) {
            let _ = cache::atomic_write(&self.path, &bytes);
        }
    }
}
