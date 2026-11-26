//! Pre-evaluation content identities. Never run a content entry, resolve its
//! imports or infer a factory from a directory name to generate project types.
//! Derived outputs and reference/dependency edges belong to the evaluated graph.

use super::generator::{
    SOURCE_LIMIT, author,
    source::{Diagnostic, Location, Source},
};
use super::source_path::{checked, relative};
use super::{source_project::Project, source_schemas};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;

/// One entry, not a cooked blob. `kind` is the primary content kind; for import
/// factories it is the schema's literal discriminant, not the word "import".
/// Keep only byte identity, not every entry's source text, in a large index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub module: String,
    pub file: String,
    pub factory: String,
    pub kind: String,
    pub digest: String,
}

/// Sorted facts about this snapshot, independent of cook/evaluation caches.
/// Consumers must compare entry digests when they later read/evaluate source.
pub struct Index {
    pub entries: BTreeMap<String, Entry>,
    pub files: BTreeMap<String, String>,
    modules: BTreeSet<String>,
}

impl Index {
    pub fn open(root: &Path, project: &Project) -> Result<Self, Diagnostic> {
        Self::open_with_overlay(root, project, &BTreeMap::new())
    }

    /// Existing-file candidates use the same identities as source transactions.
    /// No files, generated types or cook caches are written by discovery.
    pub fn open_with_overlay(
        root: &Path,
        project: &Project,
        overlay: &BTreeMap<String, String>,
    ) -> Result<Self, Diagnostic> {
        for (file, text) in overlay {
            let at = checked(root, file).map_err(|e| diagnostic(file, e))?;
            if !file.ends_with(".ts") || !at.is_file() || text.len() > SOURCE_LIMIT {
                return Err(diagnostic(
                    file,
                    "candidate must be an existing regular .ts file of at most 1 MiB",
                ));
            }
        }
        let mut index = Self {
            entries: BTreeMap::new(),
            files: BTreeMap::new(),
            modules: project.modules.keys().cloned().collect(),
        };
        let kinds: BTreeSet<_> = author::kinds().into_iter().collect();
        let mut folded = BTreeMap::new();
        for module in project.modules.values() {
            if !module.root.is_empty() {
                let at = checked(root, &module.root).map_err(|e| diagnostic(&module.root, e))?;
                if !at.is_dir() {
                    return Err(diagnostic(&module.root, "module root is not a directory"));
                }
            }
            let content = if module.root.is_empty() {
                "content".to_string()
            } else {
                format!("{}/content", module.root)
            };
            // A tiny game need not create content/ at all. Check all existing
            // components before deciding this, so symlinks never hide entries.
            let at = match checked(root, &content) {
                Ok(at) => at,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(diagnostic(&content, e)),
            };
            if !at.is_dir() {
                return Err(diagnostic(&content, "content root is not a directory"));
            }
            let mut directories = vec![content.clone()];
            while let Some(directory) = directories.pop() {
                let mut children = fs::read_dir(root.join(&directory))
                    .and_then(|dir| dir.collect::<Result<Vec<_>, _>>())
                    .map_err(|e| diagnostic(&directory, e))?;
                children.sort_by_key(|child| child.file_name());
                for child in children {
                    let name = child
                        .file_name()
                        .into_string()
                        .map_err(|_| diagnostic(&directory, "content paths must be valid UTF-8"))?;
                    if name.starts_with('.')
                        || matches!(
                            name.to_ascii_lowercase().as_str(),
                            "cooked" | "og-out" | "dist" | "node_modules"
                        )
                    {
                        continue;
                    }
                    let file = format!("{directory}/{name}");
                    relative(&file).map_err(|e| diagnostic(&file, e))?;
                    let ty = child.file_type().map_err(|e| diagnostic(&file, e))?;
                    if ty.is_symlink() {
                        return Err(diagnostic(
                            &file,
                            "content discovery cannot follow symlinks",
                        ));
                    }
                    if ty.is_dir() {
                        directories.push(file);
                        continue;
                    }
                    let local = file
                        .strip_prefix(&content)
                        .unwrap()
                        .strip_prefix('/')
                        .unwrap();
                    let stem = local
                        .strip_suffix(".ts")
                        .filter(|_| !local.ends_with(".d.ts"));
                    let Some(stem) = stem.filter(|s| !s.contains(['#', '*'])) else {
                        return Err(diagnostic(
                            &file,
                            "content entries must be .ts sources without # or *; put helpers in lib/, declarations in src/schema/, binary inputs in assets/ and external text in sources/",
                        ));
                    };
                    let id = format!("{}/{stem}", module.id);
                    if let Some(previous) = folded.insert(id.to_lowercase(), file.clone()) {
                        return Err(diagnostic(
                            &file,
                            format!(
                                "content identity {id} collides with {previous} after case folding"
                            ),
                        ));
                    }
                    let at = checked(root, &file).map_err(|e| diagnostic(&file, e))?;
                    let meta = fs::metadata(&at).map_err(|e| diagnostic(&file, e))?;
                    if !meta.is_file() {
                        return Err(diagnostic(&file, "content entry is not a regular file"));
                    }
                    let bytes = if let Some(text) = overlay.get(&file) {
                        text.as_bytes().to_vec()
                    } else {
                        // Bound the read itself, including files that grow
                        // after metadata. There is no project-wide source cap:
                        // entries are independently budgeted and not retained.
                        let mut bytes = Vec::new();
                        fs::File::open(at)
                            .and_then(|f| f.take(SOURCE_LIMIT as u64 + 1).read_to_end(&mut bytes))
                            .map_err(|e| diagnostic(&file, e))?;
                        bytes
                    };
                    if bytes.len() > SOURCE_LIMIT {
                        return Err(diagnostic(
                            &file,
                            "content source budget is 1 MiB per module",
                        ));
                    }
                    let text = std::str::from_utf8(&bytes).map_err(|e| diagnostic(&file, e))?;
                    let source = Source::new(&file, text);
                    let factory = source.content_kind()?;
                    if !kinds.contains(factory.as_str()) {
                        return Err(diagnostic(
                            &file,
                            format!(
                                "unknown author file kind {factory}; use og api list --kind file"
                            ),
                        ));
                    }
                    let kind = if factory == "import" {
                        let field = source.field("/kind").map_err(|mut e| {
                            e.message = format!("import kind must be a direct string literal for pre-cook types; {}", e.message);
                            e
                        })?;
                        let value = field.value.as_ref().and_then(|v| v.as_str());
                        // Read the tagged union from its real domain schema,
                        // not a second handwritten list of importer variants.
                        let schema = source_schemas::schema("import").unwrap();
                        let known = schema["oneOf"].as_array().unwrap().iter().any(|branch| {
                            value.is_some_and(|v| {
                                branch
                                    .pointer("/properties/kind/const")
                                    .and_then(|v| v.as_str())
                                    == Some(v)
                            })
                        });
                        if !known {
                            return Err(Diagnostic {
                                code: "content-index",
                                file: file.clone(),
                                path: "/kind".into(),
                                location: field.location,
                                message: "import kind must be a direct string literal from the import schema; use og api show file:import".into(),
                            });
                        }
                        value.unwrap().to_string()
                    } else {
                        factory.clone()
                    };
                    index.files.insert(file.clone(), id.clone());
                    index.entries.insert(
                        id.clone(),
                        Entry {
                            id,
                            module: module.id.clone(),
                            file,
                            factory,
                            kind,
                            digest: source.digest(),
                        },
                    );
                }
            }
        }
        Ok(index)
    }

    /// Module ID → complete declaration file. Callers ensure these under
    /// .og/types/, replacing only changed bytes and removing obsolete files.
    /// This is an entry index, not a promise that ref edges/outputs are checked.
    pub fn declarations(&self) -> BTreeMap<String, String> {
        let mut out: BTreeMap<_, _> = self.modules.iter().map(|id| {
            (id.clone(), format!(
                "// Generated content entry identities. Do not edit.\nimport {module};\ndeclare module {module} {{\ninterface ContentIndex {{\n",
                module = serde_json::to_string(author::MODULE).unwrap(),
            ))
        }).collect();
        for entry in self.entries.values() {
            let quote = |s: &str| serde_json::to_string(s).unwrap();
            out.get_mut(&entry.module).unwrap().push_str(&format!(
                "readonly {}: {{ readonly file: {}; readonly factory: {}; readonly kind: {} }};\n",
                quote(&entry.id),
                quote(&entry.file),
                quote(&entry.factory),
                quote(&entry.kind),
            ));
        }
        for text in out.values_mut() {
            text.push_str("}\n}\n");
        }
        out
    }
}

fn diagnostic(file: &str, message: impl ToString) -> Diagnostic {
    Diagnostic {
        code: "content-index",
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
