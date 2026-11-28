//! Restricted project bootstrap. This runs before content discovery and type
//! generation; it never imports the game entry to discover its configuration.
use super::generator::{
    Program,
    source::{Diagnostic, Source},
    value::Data,
};
use super::source_metadata::{self, Metadata};
use super::source_path::{checked, relative};
use super::source_schemas::project::{DeliveryPack, ModuleInput, ProjectInput};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Source files watched by cook and dev, including module/helper additions.
/// Outputs and installs never become inputs; directory symlinks are not walked.
/// This remains usable while a project edit temporarily fails validation.
pub fn files(root: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for child in std::fs::read_dir(directory)? {
            let child = child?;
            let name = child.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.')
                || matches!(
                    name.as_ref(),
                    "node_modules" | "target" | "cooked" | "og-out" | "dist"
                )
            {
                continue;
            }
            let ty = child.file_type()?;
            if ty.is_dir() {
                directories.push(child.path());
            } else if ty.is_file() && child.path() != root.join("project.ogt") {
                files.push(child.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Checked bootstrap data. Source namespaces and output delivery units have
/// independent dependency orders. Neither is a scene streaming partition.
pub struct Project {
    /// A partial cook's declaration closure, selected before domain import.
    pub(crate) selection: Option<BTreeSet<String>>,
    pub metadata: Metadata,
    pub input: ProjectInput,
    pub modules: BTreeMap<String, ModuleInput>,
    pub packs: BTreeMap<String, DeliveryPack>,
    pub module_order: Vec<String>,
    pub pack_order: Vec<String>,
    pub source: Source,
    pub data: Data,
    /// Evaluation identity only; content/schema/cook dependencies are separate.
    pub evaluation_key: String,
    pub dependencies: Vec<String>,
    pub fingerprints: BTreeMap<String, String>,
    roots: BTreeMap<String, String>,
}

impl Project {
    pub fn open(root: &Path) -> Result<Self, Diagnostic> {
        Self::open_with_overlay(root, &BTreeMap::new())
    }

    /// Candidate project/helper text is read by the same closed reader as the
    /// eventual committed source; this method does not write project files.
    pub fn open_with_overlay(
        root: &Path,
        overlay: &BTreeMap<String, String>,
    ) -> Result<Self, Diagnostic> {
        let metadata_bytes = std::fs::read(root.join(source_metadata::FILE))
            .map_err(|e| metadata_error("project_metadata_invalid", e.to_string()))?;
        let metadata =
            Metadata::parse(&metadata_bytes).map_err(|e| metadata_error(e.code, e.message))?;
        let program = Program::prepare_value_with_overlay(root, "project.ts", overlay)
            .map_err(|e| diagnostic(&Source::new("project.ts", ""), "", e))?;
        let source = &program.source;
        // Check the closed graph BEFORE running any source. An invalid content
        // import must not execute even if it would throw or loop forever.
        for file in &program.dependencies {
            relative(file).map_err(|e| diagnostic(source, "", e.to_string()))?;
            if !bootstrap_source(file) {
                return Err(diagnostic(
                    source,
                    "",
                    format!(
                        "bootstrap cannot read {file}; import only project.ts, module.ts, lib/**/*.ts or src/schema/**/*.ts, never content, raw assets or game code"
                    ),
                ));
            }
        }
        let data = program.run_ir().map_err(|e| diagnostic(source, "", e))?;
        let input = ProjectInput::schema()
            .decode(data.root().into_json())
            .map_err(|e| diagnostic(source, &e.path, e.message))?;
        source_file(root, source, "/entry", &input.entry)?;
        if input.entry.split('/').any(|s| {
            matches!(
                s.to_ascii_lowercase().as_str(),
                "content" | "assets" | "sources"
            )
        }) || bootstrap_source(&input.entry)
        {
            return Err(diagnostic(
                source,
                "/entry",
                "entry must be game code, not bootstrap, content or a raw asset",
            ));
        }
        for (i, pattern) in input.inputs.iter().enumerate() {
            pattern_path(source, &format!("/inputs/{i}"), pattern)?;
        }
        let mut modules = BTreeMap::from([(
            "base".into(),
            ModuleInput {
                id: "base".into(),
                root: String::new(),
                depends_on: input.depends_on.clone(),
                inputs: input.inputs.clone(),
            },
        )]);
        let mut folded_roots = BTreeSet::new();
        let mut roots = BTreeMap::new();
        let mut module_edges =
            BTreeMap::from([("base", (input.depends_on.as_slice(), "/dependsOn".into()))]);
        for (i, module) in input.modules.iter().enumerate() {
            let path = format!("/modules/{i}");
            if modules.contains_key(&module.id) {
                return Err(diagnostic(
                    source,
                    &format!("{path}/id"),
                    format!("duplicate or reserved module {}", module.id),
                ));
            }
            let at = checked(root, &module.root)
                .map_err(|e| diagnostic(source, &format!("{path}/root"), e.to_string()))?;
            if !at.is_dir() || module.root.split('/').any(reserved) {
                return Err(diagnostic(
                    source,
                    &format!("{path}/root"),
                    "module root must be a source directory outside reserved source/output/install directories",
                ));
            }
            if !folded_roots.insert(module.root.to_lowercase()) {
                return Err(diagnostic(
                    source,
                    &format!("{path}/root"),
                    "module roots collide after case folding",
                ));
            }
            for (j, pattern) in module.inputs.iter().enumerate() {
                pattern_path(source, &format!("{path}/inputs/{j}"), pattern)?;
            }
            module_edges.insert(
                module.id.as_str(),
                (&module.depends_on, format!("{path}/dependsOn")),
            );
            roots.insert(module.root.clone(), module.id.clone());
            modules.insert(module.id.clone(), module.clone());
        }
        let module_order = order(source, &module_edges)?;
        let mut packs = BTreeMap::from([(
            "base".into(),
            DeliveryPack {
                id: "base".into(),
                include: Vec::new(),
                roots: Vec::new(),
                depends_on: Vec::new(),
                optional: false,
            },
        )]);
        let mut seen = BTreeSet::new();
        let mut pack_edges: BTreeMap<&str, (&[String], String)> =
            BTreeMap::from([("base", (&[][..], String::new()))]);
        for (i, pack) in input.packs.iter().enumerate() {
            let path = format!("/packs/{i}");
            if !seen.insert(&pack.id) {
                return Err(diagnostic(
                    source,
                    &format!("{path}/id"),
                    format!("duplicate pack {}", pack.id),
                ));
            }
            if pack.id == "base" && pack.optional {
                return Err(diagnostic(
                    source,
                    &format!("{path}/optional"),
                    "the base boot pack cannot be optional",
                ));
            }
            for (j, pattern) in pack.include.iter().enumerate() {
                pattern_path(source, &format!("{path}/include/{j}"), pattern)?;
            }
            pack_edges.insert(&pack.id, (&pack.depends_on, format!("{path}/dependsOn")));
            packs.insert(pack.id.clone(), pack.clone());
        }
        let pack_order = order(source, &pack_edges)?;
        for (id, (dependencies, path)) in &pack_edges {
            for (i, dependency) in dependencies.iter().enumerate() {
                if !packs[*id].optional && packs[dependency].optional {
                    return Err(diagnostic(
                        source,
                        &format!("{path}/{i}"),
                        "a required pack cannot depend on an optional pack",
                    ));
                }
            }
        }
        if let Some(scene) = &input.scene
            && !input.scenes.contains_key(scene)
        {
            return Err(diagnostic(
                source,
                "/scene",
                "initial scene is absent from scenes",
            ));
        }
        let mut identity = blake3::Hasher::new();
        identity.update(program.key.as_bytes());
        identity.update(&metadata_bytes);
        let mut dependencies = program.dependencies.clone();
        dependencies.push(source_metadata::FILE.into());
        dependencies.sort();
        let mut fingerprints = program.fingerprints;
        fingerprints.insert(
            source_metadata::FILE.into(),
            super::integrity(&metadata_bytes),
        );
        let project = Self {
            selection: None,
            metadata,
            input,
            modules,
            packs,
            module_order,
            pack_order,
            data,
            source: program.source,
            evaluation_key: identity.finalize().to_hex().to_string(),
            dependencies,
            fingerprints,
            roots,
        };
        for (i, file) in project.input.schemas.iter().enumerate() {
            let path = format!("/schemas/{i}");
            source_file(root, &project.source, &path, file)?;
            let module = project.module_for(file);
            let local = if module.root.is_empty() {
                file.as_str()
            } else {
                &file[module.root.len() + 1..]
            };
            if !local.starts_with("src/schema/") {
                return Err(diagnostic(
                    &project.source,
                    &path,
                    "record declarations belong in a module's src/schema/",
                ));
            }
        }
        // A path may look like metadata but still belong to an undeclared
        // module. Ownership is decided by the longest explicit module root.
        for file in &program.dependencies {
            let module = project.module_for(file);
            let local = if module.root.is_empty() {
                file.as_str()
            } else {
                &file[module.root.len() + 1..]
            };
            if !(module.id == "base" && local == "project.ts"
                || local == "module.ts"
                || local.starts_with("lib/")
                || local.starts_with("src/schema/"))
            {
                return Err(diagnostic(
                    &project.source,
                    "",
                    format!("{file}: metadata is outside a declared module's bootstrap roots"),
                ));
            }
        }
        Ok(project)
    }

    /// Paths come from canonical discovery. Nested modules own their subtree;
    /// the implicit base source module owns everything not explicitly claimed.
    pub fn module_for(&self, file: &str) -> &ModuleInput {
        let mut parent = file;
        while let Some((directory, _)) = parent.rsplit_once('/') {
            if let Some(id) = self.roots.get(directory) {
                return &self.modules[id];
            }
            parent = directory;
        }
        &self.modules["base"]
    }
}

fn reserved(part: &str) -> bool {
    matches!(
        part.to_ascii_lowercase().as_str(),
        "content"
            | "assets"
            | "sources"
            | "src"
            | "lib"
            | ".og"
            | "cooked"
            | "og-out"
            | "dist"
            | "node_modules"
            | ".git"
            | ".amp"
    )
}

fn bootstrap_source(file: &str) -> bool {
    if !file.ends_with(".ts") {
        return false;
    }
    let parts: Vec<_> = file.split('/').collect();
    if parts
        .iter()
        .any(|s| reserved(s) && !matches!(*s, "src" | "lib"))
    {
        return false;
    }
    // src is a bootstrap root only for pure schema declarations.
    if parts
        .windows(2)
        .any(|s| s[0].eq_ignore_ascii_case("src") && s[1] != "schema")
    {
        return false;
    }
    file == "project.ts"
        || parts.last() == Some(&"module.ts")
        || parts.contains(&"lib")
        || parts.windows(2).any(|s| s == ["src", "schema"])
}

fn source_file(root: &Path, source: &Source, path: &str, file: &str) -> Result<(), Diagnostic> {
    let at = checked(root, file).map_err(|e| diagnostic(source, path, e.to_string()))?;
    if !(file.ends_with(".ts") || path == "/entry" && file.ends_with(".tsx"))
        || !at.is_file()
        || file.split('/').any(|p| {
            matches!(
                p.to_ascii_lowercase().as_str(),
                ".og" | "cooked" | "og-out" | "dist" | "node_modules" | ".git" | ".amp"
            )
        })
    {
        return Err(diagnostic(
            source,
            path,
            "expected a project-relative TypeScript source file outside output/install directories",
        ));
    }
    Ok(())
}

fn pattern_path(source: &Source, path: &str, pattern: &str) -> Result<(), Diagnostic> {
    relative(pattern.strip_suffix('/').unwrap_or(pattern))
        .map(|_| ())
        .map_err(|e| diagnostic(source, path, e.to_string()))
}

/// Kahn order with a name-ordered ready queue, not a quadratic repeated scan.
fn order(
    source: &Source,
    nodes: &BTreeMap<&str, (&[String], String)>,
) -> Result<Vec<String>, Diagnostic> {
    let mut remaining = BTreeMap::new();
    let mut consumers: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (id, (dependencies, path)) in nodes {
        let mut seen = BTreeSet::new();
        for (i, dependency) in dependencies.iter().enumerate() {
            let path = format!("{path}/{i}");
            if !nodes.contains_key(dependency.as_str()) {
                return Err(diagnostic(
                    source,
                    &path,
                    format!("unknown dependency {dependency}"),
                ));
            }
            if !seen.insert(dependency) {
                return Err(diagnostic(
                    source,
                    &path,
                    format!("duplicate dependency {dependency}"),
                ));
            }
            consumers.entry(dependency).or_default().push(id);
        }
        remaining.insert(*id, dependencies.len());
    }
    let mut ready: BTreeSet<_> = remaining
        .iter()
        .filter_map(|(id, &n)| (n == 0).then_some(*id))
        .collect();
    let mut result = Vec::with_capacity(nodes.len());
    while let Some(id) = ready.pop_first() {
        result.push(id.to_string());
        for consumer in consumers.get(id).into_iter().flatten() {
            let n = remaining.get_mut(consumer).expect("known consumer");
            *n -= 1;
            if *n == 0 {
                ready.insert(*consumer);
            }
        }
    }
    if result.len() != nodes.len() {
        let (id, _) = remaining.iter().find(|(_, n)| **n > 0).expect("cycle");
        return Err(diagnostic(
            source,
            &nodes[id].1,
            format!("dependency cycle involving {id}"),
        ));
    }
    Ok(result)
}

fn metadata_error(code: &'static str, message: String) -> Diagnostic {
    Diagnostic {
        code,
        file: source_metadata::FILE.into(),
        path: String::new(),
        location: super::generator::source::Location {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        },
        message,
    }
}

fn diagnostic(source: &Source, path: &str, message: impl Into<String>) -> Diagnostic {
    let location = source
        .field(path)
        .map_or_else(|e| e.location, |field| field.location);
    Diagnostic {
        code: "invalid-project",
        file: "project.ts".into(),
        path: path.into(),
        location,
        message: message.into(),
    }
}
