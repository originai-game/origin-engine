//! Optional native preparation reuse. Unknown dependency boundaries stay cold.

use crate::project::Project;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAX_FILES: usize = 32768;
const MAX_OUTPUT: u64 = 64 * 1024 * 1024;
const OMIT: &[&str] = &[".git", ".og", "og-out", "dist"];

pub struct BundleCache {
    lock: File,
    directory: PathBuf,
    root: PathBuf,
    package: PathBuf,
    entry: PathBuf,
    output_name: std::ffi::OsString,
    key: u64,
}

impl BundleCache {
    pub fn open(project: &Project, package: &Path, out: &Path) -> Option<Self> {
        // Unix change times catch same-size edits with a restored modification
        // time. Other hosts keep the existing uncached preparation path.
        if !cfg!(unix) || std::env::var("OG_SESSION").as_deref() == Ok("0") {
            return None;
        }
        for name in ["BUN_OPTIONS", "NODE_OPTIONS"] {
            if std::env::var_os(name).is_some_and(|value| !value.is_empty()) {
                return None;
            }
        }
        let root = project.root.canonicalize().ok()?;
        let package = package.canonicalize().ok()?;
        let entry = project.entry.canonicalize().ok()?;
        let output_name = out.file_name()?.to_owned();
        // One generation per project bounds disk use even when entries change.
        let directory = root.join(".og/bundle");
        fs::create_dir_all(&directory).ok()?;
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("state.lock"))
            .ok()?;
        lock.lock().ok()?;
        let mut cache = Self {
            lock,
            directory,
            root,
            package,
            entry,
            output_name,
            key: 0,
        };
        let (key, files) = cache.inputs()?;
        cache.key = key;
        let plan = serde_json::json!({ "files": files });
        fs::write(
            cache.directory.join("plan.json"),
            serde_json::to_vec(&plan).ok()?,
        )
        .ok()?;
        Some(cache)
    }

    fn inputs(&self) -> Option<(u64, BTreeSet<PathBuf>)> {
        let mut hash = std::hash::DefaultHasher::new();
        let mut files = BTreeSet::new();
        let mut count = 0;
        fn stamp(path: &Path, hash: &mut impl Hasher) -> Option<fs::Metadata> {
            path.hash(hash);
            let metadata = fs::symlink_metadata(path).ok()?;
            if !metadata.is_dir() {
                metadata.len().hash(hash);
                metadata.modified().ok()?.hash(hash);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                (metadata.dev(), metadata.ino()).hash(hash);
                if !metadata.is_dir() {
                    (metadata.ctime(), metadata.ctime_nsec()).hash(hash);
                }
            }
            Some(metadata)
        }
        fn walk(
            path: &Path,
            package: &Path,
            hash: &mut impl Hasher,
            files: &mut BTreeSet<PathBuf>,
            count: &mut usize,
            depth: usize,
        ) -> Option<()> {
            *count += 1;
            if *count > MAX_FILES || depth > 64 {
                return None;
            }
            let metadata = stamp(path, hash)?;
            if metadata.is_symlink() {
                // The engine import is external to native bundles. Other
                // symlinks may escape this tree or loop: do not cache them.
                return (path.canonicalize().ok()? == package).then_some(());
            }
            if metadata.is_dir() {
                let mut children = fs::read_dir(path)
                    .ok()?
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                children.sort_by_key(|child| child.file_name());
                for child in children {
                    if OMIT.iter().any(|name| child.file_name() == *name) {
                        continue;
                    }
                    walk(&child.path(), package, hash, files, count, depth + 1)?;
                }
            } else if metadata.is_file() {
                files.insert(path.to_path_buf());
            } else {
                return None;
            }
            Some(())
        }
        self.entry.hash(&mut hash);
        self.output_name.hash(&mut hash);
        let cwd = std::env::current_dir().ok()?;
        cwd.hash(&mut hash);
        std::env::vars_os()
            .collect::<std::collections::BTreeMap<_, _>>()
            .hash(&mut hash);
        stamp(&std::env::current_exe().ok()?, &mut hash)?;
        let bun = std::env::split_paths(&std::env::var_os("PATH")?)
            .map(|dir| dir.join("bun"))
            .find(|path| path.is_file())?
            .canonicalize()
            .ok()?;
        stamp(&bun, &mut hash)?;
        walk(
            &self.root,
            &self.package,
            &mut hash,
            &mut files,
            &mut count,
            0,
        )?;
        // Engine exports and the fixed build plugin are compiler inputs too.
        walk(
            &self.package.join("src"),
            &self.package,
            &mut hash,
            &mut BTreeSet::new(),
            &mut count,
            0,
        )?;
        if stamp(&self.package.join("package.json"), &mut hash)?.is_symlink() {
            return None;
        }
        let mut ancestors = BTreeSet::new();
        ancestors.extend(self.root.ancestors().map(Path::to_path_buf));
        ancestors.extend(cwd.ancestors().map(Path::to_path_buf));
        if let Some(home) = std::env::var_os("HOME") {
            ancestors.insert(PathBuf::from(&home).join(".config"));
            ancestors.insert(PathBuf::from(home));
        }
        if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
            ancestors.insert(PathBuf::from(config));
        }
        for directory in ancestors {
            // Bun config can preload arbitrary code. Its inputs are not a
            // static bundle graph, so retain ordinary compilation instead.
            if directory.join("bunfig.toml").exists() || directory.join(".bunfig.toml").exists() {
                return None;
            }
            for name in ["package.json", "tsconfig.json", "jsconfig.json"] {
                let path = directory.join(name);
                path.hash(&mut hash);
                if path.exists() {
                    if stamp(&path, &mut hash)?.is_symlink() {
                        return None;
                    }
                    files.insert(path);
                }
            }
        }
        Some((hash.finish(), files))
    }

    pub fn restore(&self, out: &Path) -> bool {
        let Ok(record) = fs::read_to_string(self.directory.join("key")) else {
            return false;
        };
        let Some((code, map)) = self.outputs() else {
            return false;
        };
        if record != self.record(&code, &map) {
            return false;
        }
        fs::write(out, code).is_ok() && fs::write(out.with_extension("js.map"), map).is_ok()
    }

    pub fn configure(&self, command: &mut Command) -> std::io::Result<()> {
        // Eligibility belongs to this compilation, never a prior one.
        match fs::remove_file(self.directory.join("eligible")) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
        }
        let inherited = self.lock.try_clone()?;
        command
            .arg("--cache-plan")
            .arg(self.directory.join("plan.json"))
            .stdin(Stdio::from(inherited));
        Ok(())
    }

    pub fn save(&self, out: &Path) {
        if !self.directory.join("eligible").is_file()
            || self.inputs().map(|(key, _)| key) != Some(self.key)
        {
            return;
        }
        let read = |path: &Path| -> Option<Vec<u8>> {
            (fs::metadata(path).ok()?.len() <= MAX_OUTPUT)
                .then(|| fs::read(path).ok())
                .flatten()
        };
        let (Some(code), Some(map)) = (read(out), read(&out.with_extension("js.map"))) else {
            return;
        };
        let key = self.directory.join("key");
        // A crash between artifact writes leaves no valid generation marker.
        let _ = fs::remove_file(&key);
        if fs::write(self.directory.join("bundle.js"), &code).is_ok()
            && fs::write(self.directory.join("bundle.js.map"), &map).is_ok()
        {
            let _ = fs::write(key, self.record(&code, &map));
        }
    }

    fn outputs(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        let read = |name: &str| {
            let path = self.directory.join(name);
            (fs::metadata(&path).ok()?.len() <= MAX_OUTPUT)
                .then(|| fs::read(path).ok())
                .flatten()
        };
        Some((read("bundle.js")?, read("bundle.js.map")?))
    }

    fn record(&self, code: &[u8], map: &[u8]) -> String {
        let mut hash = std::hash::DefaultHasher::new();
        code.hash(&mut hash);
        map.hash(&mut hash);
        format!("{} {}", self.key, hash.finish())
    }
}
