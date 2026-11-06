//! Source moves with parsed static-reference edits, prepared before any write.
use super::{assets, ts_refs};
use og_core::ogt::Document;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct MoveReport {
    pub from: String,
    pub to: String,
    pub rewritten: Vec<String>,
    pub warnings: Vec<String>,
}

struct Change {
    from: PathBuf,
    to: PathBuf,
    before: Vec<u8>,
    after: Vec<u8>,
}

fn id(value: &str) -> Result<&str, String> {
    let value = crate::runtime::asset_id(value);
    if value.is_empty()
        || value.starts_with('@')
        || value.contains(['\\', '#', ':'])
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return Err(format!("{value:?} must name a source file under assets/"));
    }
    Ok(value)
}

fn renamed(value: &str, from: &str, to: &str) -> Option<String> {
    let (prefix, value) = value
        .strip_prefix("assets/")
        .map_or(("", value), |v| ("assets/", v));
    let suffix = value.strip_prefix(from)?;
    (suffix.is_empty() || suffix.starts_with('#')).then(|| format!("{prefix}{to}{suffix}"))
}

/// The document parser validates escapes and grammar; this scan keeps the
/// original formatting and comments by locating only quoted value tokens.
fn ogt_literals(text: &str) -> Result<Vec<ts_refs::Literal>, String> {
    Document::parse(text).map_err(|e| e.to_string())?;
    let bytes = text.as_bytes();
    let (mut i, mut literals) = (0, Vec::new());
    while i < bytes.len() {
        match bytes[i] {
            b'#' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
                let doc = Document::parse(&format!(
                    "type literal\nversion 1\n---\nvalue {}\n",
                    &text[start..i]
                ))
                .map_err(|e| e.to_string())?;
                literals.push(ts_refs::Literal {
                    value: doc
                        .body
                        .get("value")
                        .expect("literal entry")
                        .string(0)
                        .map_err(|e| e.to_string())?
                        .to_owned(),
                    range: start..i,
                });
            }
            _ => i += 1,
        }
    }
    Ok(literals)
}

fn rewrite(text: &str, literals: Vec<ts_refs::Literal>, from: &str, to: &str) -> String {
    let mut edits: Vec<(Range<usize>, String)> = literals
        .into_iter()
        .filter_map(|literal| {
            renamed(&literal.value, from, to).map(|value| {
                (
                    literal.range,
                    serde_json::to_string(&value).expect("asset path"),
                )
            })
        })
        .collect();
    edits.sort_by_key(|(range, _)| range.start);
    edits.dedup_by(|a, b| a.0 == b.0);
    let mut text = text.to_owned();
    for (range, value) in edits.into_iter().rev() {
        text.replace_range(range, &value);
    }
    text
}

fn documents(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            documents(&entry.path(), paths)?;
        } else if entry.path().extension().is_some_and(|e| e == "ogt") {
            if ty.is_symlink() {
                return Err(format!(
                    "{}: refusing to rewrite a symlink",
                    entry.path().display()
                ));
            }
            paths.push(entry.path());
        }
    }
    Ok(())
}

/// Move one source, its import sidecar, and matching static OGT/TS literals.
/// Destinations must be absent. Parsing or preflight failure writes nothing.
pub fn move_asset(root: &Path, from: &str, to: &str) -> Result<MoveReport, String> {
    let (from, to) = (id(from)?, id(to)?);
    if from == to {
        return Err("source and destination are the same".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let assets = root.join("assets");
    let source = assets.join(from);
    let destination = assets.join(to);
    if std::fs::symlink_metadata(assets.join(format!("{to}.ogt"))).is_ok() {
        return Err(format!(
            "{to}.ogt already exists; refusing to replace or adopt import settings"
        ));
    }
    let mut moves = BTreeMap::from([(source.clone(), destination)]);
    let settings = assets.join(format!("{from}.ogt"));
    if settings.is_file() {
        let text = std::fs::read_to_string(&settings).map_err(|e| e.to_string())?;
        if Document::parse(&text).map_err(|e| e.to_string())?.ty == "import" {
            moves.insert(settings, assets.join(format!("{to}.ogt")));
        }
    }
    for (old, new) in &moves {
        let meta = std::fs::symlink_metadata(old).map_err(|e| format!("{}: {e}", old.display()))?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(format!("{} is not a regular source file", old.display()));
        }
        if std::fs::symlink_metadata(new).is_ok() {
            return Err(format!("{} already exists", new.display()));
        }
        for path in [old.as_path(), new.as_path()] {
            for ancestor in path.ancestors().take_while(|p| p.starts_with(&assets)) {
                if std::fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
                    return Err(format!("{}: refusing a symlink path", ancestor.display()));
                }
            }
            let existing = path
                .ancestors()
                .find(|p| p.exists())
                .ok_or("missing project directory")?;
            if !existing
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(&assets)
            {
                return Err(format!(
                    "{} escapes assets/ through a symlink",
                    path.display()
                ));
            }
        }
    }
    let mut paths = Vec::new();
    documents(&assets, &mut paths)?;
    if root.join("project.ogt").is_file() {
        paths.push(root.join("project.ogt"));
    }
    paths.extend(assets::game_typescript(&root)?);
    paths.extend(moves.keys().cloned());
    paths.sort();
    paths.dedup();
    let mut report = MoveReport {
        from: from.into(),
        to: to.into(),
        rewritten: Vec::new(),
        warnings: Vec::new(),
    };
    if Path::new(from).parent() != Path::new(to).parent()
        && matches!(
            super::ext(from).as_str(),
            "gltf" | "glb" | "obj" | "fbx" | "ogt" | "tmj" | "ldtk"
        )
    {
        report.warnings.push(format!("{from}: directory changed; review source-relative dependencies inside this asset (only OGT/TS asset-id literals are rewritten)"));
    }
    let mut changes = Vec::new();
    for path in paths {
        if std::fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err(format!("{}: refusing to rewrite a symlink", path.display()));
        }
        let before = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let after = if matches!(
            extension,
            "ogt" | "ts" | "tsx" | "js" | "jsx" | "mts" | "mjs"
        ) {
            let text =
                std::str::from_utf8(&before).map_err(|e| format!("{}: {e}", path.display()))?;
            let literals = if extension == "ogt" {
                ogt_literals(text).map_err(|e| format!("{}: {e}", path.display()))?
            } else {
                let refs = ts_refs::parse(&path, text)?;
                for computed in refs.computed {
                    let prefix = crate::runtime::asset_id(&computed.prefix);
                    if from.starts_with(prefix) {
                        report.warnings.push(format!(
                            "{}:{}: computed load with prefix {:?} may need a manual edit",
                            path.strip_prefix(&root).unwrap().display(),
                            computed.line,
                            computed.prefix
                        ));
                    }
                }
                refs.literals
            };
            rewrite(text, literals, from, to).into_bytes()
        } else {
            before.clone()
        };
        if before != after {
            report.rewritten.push(
                path.strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        if before != after || moves.contains_key(&path) {
            changes.push(Change {
                to: moves.get(&path).unwrap_or(&path).clone(),
                from: path,
                before,
                after,
            });
        }
    }
    apply(&root, &changes)?;
    Ok(report)
}

fn apply(root: &Path, changes: &[Change]) -> Result<(), String> {
    let journal = root.join(".og/asset-move");
    std::fs::create_dir_all(journal.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::create_dir(&journal).map_err(|e| {
        format!(
            "{}: {e}; another move is active or needs recovery",
            journal.display()
        )
    })?;
    let prepare = (|| -> std::io::Result<()> {
        let paths: Vec<_> = changes.iter().map(|c| (&c.from, &c.to)).collect();
        std::fs::write(
            journal.join("paths.json"),
            serde_json::to_vec_pretty(&paths)?,
        )?;
        for (i, change) in changes.iter().enumerate() {
            if std::fs::read(&change.from)? != change.before {
                return Err(std::io::Error::other(format!(
                    "{} changed during preflight",
                    change.from.display()
                )));
            }
            std::fs::create_dir_all(change.to.parent().unwrap())?;
            let new = journal.join(format!("new-{i}"));
            std::fs::copy(&change.from, &new)?;
            if change.before != change.after {
                std::fs::write(new, &change.after)?;
            }
        }
        Ok(())
    })();
    if let Err(error) = prepare {
        let _ = std::fs::remove_dir_all(&journal);
        return Err(error.to_string());
    }
    let (mut backed_up, mut installed) = (0, 0);
    let commit = (|| -> std::io::Result<()> {
        for (i, change) in changes.iter().enumerate() {
            if std::fs::read(&change.from)? != change.before {
                return Err(std::io::Error::other(format!(
                    "{} changed during preparation",
                    change.from.display()
                )));
            }
            std::fs::rename(&change.from, journal.join(format!("old-{i}")))?;
            backed_up += 1;
        }
        for (i, change) in changes.iter().enumerate() {
            // Hard-link installation is atomic and refuses an intervening file.
            std::fs::hard_link(journal.join(format!("new-{i}")), &change.to)?;
            installed += 1;
        }
        Ok(())
    })();
    if let Err(error) = commit {
        let mut errors = Vec::new();
        for change in &changes[..installed] {
            if !std::fs::read(&change.to).is_ok_and(|bytes| bytes == change.after) {
                errors.push(format!(
                    "{} changed after installation; preserved",
                    change.to.display()
                ));
                continue;
            }
            if let Err(e) = std::fs::remove_file(&change.to) {
                errors.push(e.to_string());
            }
        }
        for (i, change) in changes[..backed_up].iter().enumerate() {
            if let Err(e) = std::fs::hard_link(journal.join(format!("old-{i}")), &change.from) {
                errors.push(e.to_string());
            }
        }
        if errors.is_empty() {
            let _ = std::fs::remove_dir_all(&journal);
            return Err(format!("{error}; move rolled back"));
        }
        return Err(format!(
            "{error}; recovery files retained at {}: {}",
            journal.display(),
            errors.join("; ")
        ));
    }
    std::fs::remove_dir_all(&journal).map_err(|e| {
        format!(
            "move completed, but could not remove {}: {e}",
            journal.display()
        )
    })
}
