//! Project-local author types and isolated per-module checker configurations.
//! This stage evaluates metadata/schema declarations, never content or game code.
use crate::commands::check::Target;
use og_cook::cook::{
    source_index::Index, source_project::Project, source_records::Records, source_types,
};
use serde_json::json;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Find the explicit TS project of a directory, config or entry. This does not
/// open the legacy CLI project representation or evaluate the game entry.
pub fn root(path: &Path) -> Option<PathBuf> {
    let path = path.canonicalize().ok()?;
    let directory = if path.is_dir() {
        path.as_path()
    } else {
        path.parent()?
    };
    directory
        .ancestors()
        .find(|root| root.join("project.ts").is_file())
        .map(Path::to_path_buf)
}

pub fn ensure(root: &Path) -> Result<Vec<Target>, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let work = directory(&root, ".og/types/author").map_err(|e| e.to_string())?;
    let lock_path = work.join("types.lock");
    regular(&lock_path).map_err(|e| e.to_string())?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|e| e.to_string())?;
    lock.lock().map_err(|e| e.to_string())?;
    let project = Project::open(&root).map_err(|e| e.to_string())?;
    let index = Index::open(&root, &project).map_err(|e| e.to_string())?;
    let records = Records::open(&root, &project)?;
    // Build every document before touching any generated index. A bad schema
    // must not leave a mixture of old/new roles for the checker to consume.
    let documents = source_types::declarations(&project, &index, &records)?;
    let mut targets = Vec::new();
    for (module, text) in &documents {
        let definition = &project.modules[module];
        let dir = directory(&work, module).map_err(|e| e.to_string())?;
        replace(&dir.join("index.d.ts"), text).map_err(|e| e.to_string())?;
        let source = if definition.root.is_empty() {
            root.clone()
        } else {
            root.join(&definition.root)
        };
        let path = |path: &Path| crate::project::typescript_path(path);
        let mut exclude = [
            "assets",
            "sources",
            ".og",
            "node_modules",
            "cooked",
            "dist",
            "og-out",
        ]
        .map(|name| path(&source.join(name)))
        .to_vec();
        for other in project.modules.values().filter(|other| other.id != *module) {
            let other = root.join(&other.root);
            if other.starts_with(&source) {
                exclude.push(path(&other));
            }
        }
        let mut config = json!({
            "files": ["index.d.ts"],
            "include": [format!("{}/**/*.ts",path(&source)),format!("{}/**/*.tsx",path(&source))],
            "exclude": exclude,
        });
        if root.join("tsconfig.json").is_file() {
            config["extends"] = json!(path(&root.join("tsconfig.json")));
        } else {
            let template: serde_json::Value = serde_json::from_str(include_str!(
                "../../../packages/create/template/tsconfig.json"
            ))
            .expect("template config");
            config["compilerOptions"] = template["compilerOptions"].clone();
        }
        let config_path = dir.join("tsconfig.json");
        replace(
            &config_path,
            &format!("{}\n", serde_json::to_string_pretty(&config).unwrap()),
        )
        .map_err(|e| e.to_string())?;
        targets.push(Target {
            root: root.clone(),
            config: config_path,
        });
    }
    // Removed modules cannot leave stale ambient augmentations in generated
    // output. Delete only the two files this stage owns, never arbitrary files.
    for entry in fs::read_dir(&work).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir()
            && !documents.contains_key(&entry.file_name().to_string_lossy().into_owned())
        {
            for file in ["index.d.ts", "tsconfig.json"] {
                let file = entry.path().join(file);
                regular(&file).map_err(|e| e.to_string())?;
                match fs::remove_file(file) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.to_string()),
                }
            }
            let _ = fs::remove_dir(entry.path());
        }
    }
    Ok(targets)
}

fn directory(root: &Path, name: &str) -> io::Result<PathBuf> {
    let mut directory = root.to_path_buf();
    for part in Path::new(name) {
        directory.push(part);
        match fs::create_dir(&directory) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        if !fs::symlink_metadata(&directory)?.is_dir() {
            return Err(io::Error::other(format!(
                "{} must be a regular directory",
                directory.display()
            )));
        }
    }
    Ok(directory)
}

fn regular(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(io::Error::other(format!(
            "{} must be a regular file",
            path.display()
        ))),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn replace(path: &Path, text: &str) -> io::Result<()> {
    regular(path)?;
    if fs::read_to_string(path).ok().as_deref() == Some(text) {
        return Ok(());
    }
    let temporary = path.with_extension("tmp");
    regular(&temporary)?;
    fs::write(&temporary, text)?;
    fs::rename(&temporary, path)
}
