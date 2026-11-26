//! Entry-scoped filesystem authority for content preparation. The realm still
//! receives only captured modules/inputs, never a loader or filesystem binding.
use super::super::{source_index::Entry, source_path::relative, source_project::Project};

pub(super) struct Scope<'a> {
    pub project: &'a Project,
    pub entry: &'a Entry,
}

impl Scope<'_> {
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
            let stem = local
                .strip_prefix("content/")
                .and_then(|s| s.strip_suffix(".ts"));
            if !source
                || file != self.entry.file
                || module.id != self.entry.module
                || stem.is_none_or(|stem| self.entry.id != format!("{}/{stem}", module.id))
            {
                return Err(format!(
                    "{file}: content entry identity does not match its declared module"
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
        relative(path).map_err(|e| format!("{}: input {path}: {e}", self.entry.file))?;
        if !(path.starts_with("assets/") || path.starts_with("sources/")) {
            return Err(format!(
                "{}: input {path} must be inside this content module's assets/ or sources/; use og.ref for another module's content",
                self.entry.file
            ));
        }
        let module = &self.project.modules[&self.entry.module];
        Ok(if module.root.is_empty() {
            path.into()
        } else {
            format!("{}/{path}", module.root)
        })
    }
}
