//! Entry-scoped filesystem authority for content preparation. The realm still
//! receives only captured modules/inputs, never a loader or filesystem binding.
use super::super::{source_index::Entry, source_path::relative, source_project::Project};

pub(super) struct Scope<'a> {
    pub project: &'a Project,
    pub root: Root<'a>,
}

pub(super) enum Root<'a> {
    Content(&'a Entry),
    Records(&'a str),
}

impl Scope<'_> {
    pub fn stale(&self, bytes: &[u8]) -> bool {
        matches!(&self.root, Root::Content(entry) if blake3::hash(bytes).to_hex().as_str() != entry.digest)
    }

    /// Check every edge before memoization: a previously captured helper does
    /// not grant a later importer permission to read another module's code.
    pub fn source(&self, from: Option<&str>, file: &str) -> Result<(), String> {
        relative(file).map_err(|e| format!("{file}: {e}"))?;
        let module = self.project.module_for(file);
        let local = file
            .strip_prefix(&module.root)
            .unwrap()
            .trim_start_matches('/');
        let source = file.ends_with(".ts")
            && !file.ends_with(".d.ts")
            && !local.split('/').any(|part| {
                part.starts_with('.')
                    || matches!(
                        part.to_ascii_lowercase().as_str(),
                        "cooked" | "og-out" | "dist" | "node_modules"
                    )
            });
        let Some(from) = from else {
            let valid = match &self.root {
                Root::Content(entry) => {
                    let stem = local
                        .strip_prefix("content/")
                        .and_then(|s| s.strip_suffix(".ts"));
                    file == entry.file
                        && module.id == entry.module
                        && stem.is_some_and(|stem| entry.id == format!("{}/{stem}", module.id))
                }
                Root::Records(entry) => file == *entry && local.starts_with("src/schema/"),
            };
            if !source || !valid {
                return Err(format!(
                    "{file}: source entry identity does not match its declared module and role"
                ));
            }
            return Ok(());
        };
        if !source || !(local.starts_with("lib/") || local.starts_with("src/schema/")) {
            return Err(format!(
                "{from}: cannot import {file}; content imports may read only lib/**/*.ts and src/schema/**/*.ts in declared modules; use og.ref for content, never import game code, manifests or raw inputs"
            ));
        }
        let importer = self.project.module_for(from);
        if importer.id != module.id && !importer.depends_on.contains(&module.id) {
            return Err(format!(
                "{from}: importing {file} requires module {} to declare a direct dependency on {}",
                importer.id, module.id
            ));
        }
        Ok(())
    }

    /// Literal inputs belong to the content entry's module, not an imported
    /// helper's directory or module. Helpers are reusable code, not raw-asset
    /// exports. A different module's assets must be exposed as content refs.
    pub fn input(&self, path: &str) -> Result<String, String> {
        let Root::Content(entry) = &self.root else {
            return Err("record declarations cannot read raw inputs".into());
        };
        relative(path).map_err(|e| format!("{}: input {path}: {e}", entry.file))?;
        if !(path.starts_with("assets/") || path.starts_with("sources/")) {
            return Err(format!(
                "{}: input {path} must be inside this content module's assets/ or sources/; use og.ref for another module's content",
                entry.file
            ));
        }
        let module = &self.project.modules[&entry.module];
        Ok(if module.root.is_empty() {
            path.into()
        } else {
            format!("{}/{path}", module.root)
        })
    }
}
