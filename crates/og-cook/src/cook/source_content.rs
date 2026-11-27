//! Evaluated content and its identity/reference graph, before domain cooking.
//! Entries run independently; a ref never executes its target. The builder keeps
//! compact facts, not every entry's source text, numeric payload or JS realm.

use super::generator::{
    Program,
    source::{Diagnostic, Location, Source},
    value::Data,
};
use super::source_index::{Entry, Index};
use super::source_project::Project;
use super::source_schemas::{File, project::ProjectInput, references::*};
use og_core::reflect::source::AuthorSchema;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One checked entry. Feed its data to its domain cooker, add its facts to the
/// graph, then drop it; a large build need not retain all source/IR at once.
pub struct Content {
    entry: Entry,
    source: Source,
    data: Data,
    project_key: String,
    evaluation_key: String,
    schema_key: String,
    imports: BTreeMap<String, Vec<String>>,
    input_dependencies: Vec<String>,
    references: Vec<Reference>,
}

impl Content {
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Canonical envelope, retaining owned typed buffers. Schema validation
    /// currently projects one entry to JSON for the existing native decoders;
    /// that temporary projection is not kept in this stage or the graph.
    pub fn data(&self) -> &Data {
        &self.data
    }
}

struct Bound<'a> {
    schema: &'a dyn AuthorSchema,
    key: String,
    references: ReferenceSchema<'a>,
}

/// Reuse compiled reference plans per bound schema, never JS module instances.
/// Contextual decoders are borrowed from the project, not globally registered.
#[derive(Default)]
pub struct Evaluator<'a> {
    schemas: Vec<Bound<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn evaluate(
        &mut self,
        root: &Path,
        project: &Project,
        entry: &Entry,
        overlay: &BTreeMap<String, String>,
        contextual: Option<&'a dyn AuthorSchema>,
    ) -> Result<Content, Diagnostic> {
        let program = Program::prepare_content(root, project, entry, overlay)
            .map_err(|e| diagnostic(&Source::new(&entry.file, ""), &entry.file, "", e))?;
        let source = &program.source;
        let at = |path: &str, message: String| diagnostic(source, &entry.file, path, message);
        let data = program.run_ir().map_err(|e| at("", e))?;
        let file: File = serde_json::from_value(data.root().into_json())
            .map_err(|e| at("", format!("expected one content declaration: {e}")))?;
        // JS descriptors are data, not an authority to change indexed identity.
        if file.kind != entry.factory
            || (file.kind == "import"
                && file.value.get("kind").and_then(serde_json::Value::as_str)
                    != Some(entry.kind.as_str()))
        {
            return Err(at(
                "",
                "evaluated kind differs from indexed factory/primary kind".into(),
            ));
        }
        let schema = file
            .schema(contextual)
            .map_err(|e| at(&e.path, e.message))?;
        let i = match self
            .schemas
            .iter()
            .position(|s| std::ptr::eq(s.schema, schema))
        {
            Some(i) => i,
            None => {
                let references = ReferenceSchema::new(schema).map_err(|e| at("", e))?;
                let key =
                    blake3::hash(&serde_json::to_vec(schema.document()).expect("schema is JSON"))
                        .to_hex()
                        .to_string();
                self.schemas.push(Bound {
                    schema,
                    key,
                    references,
                });
                self.schemas.len() - 1
            }
        };
        let bound = &self.schemas[i];
        let references = bound
            .references
            .collect(&file.value)
            .map_err(|e| at(&e.path, e.message))?;
        Ok(Content {
            entry: entry.clone(),
            source: program.source,
            data,
            project_key: project.evaluation_key.clone(),
            evaluation_key: program.key,
            schema_key: bound.key.clone(),
            imports: program.imports,
            input_dependencies: program.input_dependencies,
            references,
        })
    }
}

/// A primary or actual domain-reported subresource; never a suffix heuristic.
#[derive(Clone, Debug)]
pub struct Output {
    pub entry: String,
    pub kind: String,
    /// The domain's one cooked output ID. Unmigrated domain bindings are
    /// removed with that domain's parser, not added to runtime lookup.
    pub asset: String,
}

/// One resolved schema-position edge, with its original entry expression span.
#[derive(Debug)]
pub struct Edge {
    pub path: String,
    pub target: String,
    pub entry: String,
    pub kind: String,
    pub location: Location,
}

#[derive(Debug)]
pub struct Node {
    pub entry: Entry,
    /// Local evaluation identity, NOT a domain cook or transitive cache key.
    pub evaluation_key: String,
    /// Structural document identity; domain action keys also need decoder and
    /// algorithm implementation identities, not just these schema bytes.
    pub schema_key: String,
    pub imports: BTreeMap<String, Vec<String>>,
    pub input_dependencies: Vec<String>,
    pub references: Vec<Edge>,
}

/// Complete identity/reference closure for one project/index snapshot. Domain
/// build ordering, pack assignment, record discovery and action/CAS caches are
/// separate stages; legal data-reference cycles are deliberately retained.
#[derive(Debug)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub outputs: BTreeMap<String, Output>,
    /// Project/scenes/look/pack-root refs, not source module imports. A project
    /// assembles all its modules, so these do not grant base-module privileges.
    pub roots: Vec<Edge>,
    pub dependents: BTreeMap<String, BTreeSet<String>>,
}

struct PendingReference {
    reference: Reference,
    location: Location,
}

impl PendingReference {
    fn error(&self, file: &str, message: String) -> Diagnostic {
        Diagnostic {
            code: "invalid-content-reference",
            file: file.into(),
            path: self.reference.path.clone(),
            location: self.location.clone(),
            message,
        }
    }
}

/// Add entries as bounded evaluation/domain jobs finish; forward refs resolve
/// only after the complete catalogue of actual outputs is available.
pub struct GraphBuilder<'a> {
    project: &'a Project,
    index: &'a Index,
    existing: BTreeMap<String, Entry>,
    provided: BTreeSet<String>,
    nodes: BTreeMap<String, (Node, Vec<PendingReference>)>,
    outputs: BTreeMap<String, Output>,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(project: &'a Project, index: &'a Index) -> Self {
        Self {
            project,
            index,
            existing: BTreeMap::new(),
            provided: BTreeSet::new(),
            nodes: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }

    /// cook.uses grants access to actual cooked material outputs, not to the
    /// provider's source namespace or author-only capabilities such as fields.
    pub(crate) fn insert_provided(&mut self, provided: &super::provided::Provided) {
        for (id, kind) in provided.material_outputs() {
            if self.index.entries.contains_key(id) || self.outputs.contains_key(id) {
                continue;
            }
            self.provided.insert(id.into());
            self.outputs.insert(
                id.into(),
                Output {
                    entry: id.into(),
                    kind: kind.into(),
                    asset: id.into(),
                },
            );
        }
    }

    /// Actual scanner results until their P1/P2/P3 kind cutovers. These are
    /// not TS declarations and cannot pretend to supply generated subresources.
    pub(crate) fn insert_scanned(&mut self, entry: Entry, asset: String) -> Result<Output, String> {
        if !matches!(
            entry.factory.as_str(),
            "scene" | "prefab" | "look" | "dialogue" | "model" | "texture"
        ) {
            return Err(format!(
                "{}: kind is outside the temporary scaffold catalogue",
                entry.file
            ));
        }
        if self
            .index
            .entries
            .keys()
            .chain(self.existing.keys())
            .any(|id| id.eq_ignore_ascii_case(&entry.id))
        {
            return Err(format!(
                "{}: duplicate content identity {}",
                entry.file, entry.id
            ));
        }
        let output = Output {
            entry: entry.id.clone(),
            kind: entry.kind.clone(),
            asset,
        };
        self.outputs.insert(entry.id.clone(), output.clone());
        self.existing.insert(entry.id.clone(), entry);
        Ok(output)
    }

    /// `derived` maps suffix (without the first '#') to actual output kind.
    /// Only the domain producer supplies these facts, after discovering its
    /// outputs; TS cannot assert a catalogue or invent subresource kinds.
    /// An error adds nothing, and this retains neither source nor data.
    pub fn insert(
        &mut self,
        content: &Content,
        derived: &BTreeMap<String, String>,
    ) -> Result<(), Diagnostic> {
        let entry = &content.entry;
        let at = |message| diagnostic(&content.source, &entry.file, "", message);
        if self.index.entries.get(&entry.id) != Some(entry)
            || self.project.evaluation_key != content.project_key
        {
            return Err(at("content belongs to a different project/index snapshot"));
        }
        if self.nodes.contains_key(&entry.id) {
            return Err(at("content entry was already added"));
        }
        let mut outputs = BTreeMap::from([(
            entry.id.clone(),
            Output {
                entry: entry.id.clone(),
                kind: entry.kind.clone(),
                asset: entry.id.clone(),
            },
        )]);
        let mut folded = BTreeSet::new();
        for (suffix, kind) in derived {
            if suffix.split('#').any(str::is_empty) || kind.is_empty() {
                return Err(at(
                    "derived outputs need nonempty suffix segments and a kind",
                ));
            }
            if !folded.insert(suffix.to_lowercase()) {
                return Err(at("derived output identities collide after case folding"));
            }
            outputs.insert(
                format!("{}#{suffix}", entry.id),
                Output {
                    entry: entry.id.clone(),
                    kind: kind.clone(),
                    asset: format!("{}#{suffix}", entry.id),
                },
            );
        }
        let references = located(&content.source, &content.references)?;
        self.outputs.extend(outputs);
        self.nodes.insert(
            entry.id.clone(),
            (
                Node {
                    entry: entry.clone(),
                    evaluation_key: content.evaluation_key.clone(),
                    schema_key: content.schema_key.clone(),
                    imports: content.imports.clone(),
                    input_dependencies: content.input_dependencies.clone(),
                    references: Vec::new(),
                },
                references,
            ),
        );
        Ok(())
    }

    pub fn finish(mut self) -> Result<Graph, Diagnostic> {
        for entry in self.index.entries.values() {
            if !self.nodes.contains_key(&entry.id) {
                return Err(diagnostic(
                    &Source::new(&entry.file, ""),
                    &entry.file,
                    "",
                    format!(
                        "incomplete content graph: {} has not been evaluated",
                        entry.id
                    ),
                ));
            }
        }
        let root_refs = ReferenceSchema::new(ProjectInput::schema())
            .map_err(|e| diagnostic(&self.project.source, "project.ts", "", e))?
            .collect(&self.project.data.root().into_json())
            .map_err(|e| diagnostic(&self.project.source, "project.ts", &e.path, e.message))?;
        let resolve = |file, module, r| {
            resolve(
                self.project,
                self.index,
                &self.existing,
                &self.outputs,
                file,
                module,
                r,
            )
        };
        let mut roots: Vec<_> = located(&self.project.source, &root_refs)?
            .into_iter()
            .map(|r| resolve("project.ts", None, r))
            .collect::<Result<_, _>>()?;
        if let Some(super::source_schemas::procedural::LookBase::Asset(look)) =
            &self.project.metadata.look
        {
            roots.push(resolve(
                super::source_metadata::FILE,
                None,
                PendingReference {
                    reference: Reference {
                        path: "/look".into(),
                        id: look.id.clone(),
                        kinds: Some(BTreeSet::from(["look".into()])),
                        factories: None,
                    },
                    location: Location {
                        start: 0,
                        end: 0,
                        line: 1,
                        column: 1,
                    },
                },
            )?);
        }
        let mut dependents: BTreeMap<_, BTreeSet<_>> = self
            .index
            .entries
            .keys()
            .chain(self.existing.keys())
            .chain(self.provided.iter())
            .map(|id| (id.clone(), BTreeSet::new()))
            .collect();
        for (id, (node, references)) in &mut self.nodes {
            let edges: Vec<_> = std::mem::take(references)
                .into_iter()
                .map(|r| resolve(&node.entry.file, Some(node.entry.module.as_str()), r))
                .collect::<Result<_, _>>()?;
            for edge in &edges {
                dependents
                    .get_mut(&edge.entry)
                    .expect("indexed output owner")
                    .insert(id.clone());
            }
            node.references = edges;
        }
        Ok(Graph {
            nodes: self
                .nodes
                .into_iter()
                .map(|(id, (node, _))| (id, node))
                .collect(),
            outputs: self.outputs,
            roots,
            dependents,
        })
    }
}

fn resolve(
    project: &Project,
    index: &Index,
    existing: &BTreeMap<String, Entry>,
    outputs: &BTreeMap<String, Output>,
    file: &str,
    module: Option<&str>,
    pending: PendingReference,
) -> Result<Edge, Diagnostic> {
    let reference = &pending.reference;
    let target = outputs.get(&reference.id).ok_or_else(|| {
        pending.error(
            file,
            format!(
                "unknown content output {}; subresources must be reported by their domain producer",
                reference.id
            ),
        )
    })?;
    let owner = index
        .entries
        .get(&target.entry)
        .or_else(|| existing.get(&target.entry));
    // Only actual provided bundle outputs have no local source owner.
    if let Some(module) = module
        && let Some(owner) = owner
        && module != owner.module
        && !project.modules[module].depends_on.contains(&owner.module)
    {
        return Err(pending.error(
            file,
            format!(
                "reference {} requires module {module} to declare a direct dependency on {}",
                reference.id, owner.module,
            ),
        ));
    }
    if let Some(kinds) = &reference.kinds
        && !kinds.contains(&target.kind)
    {
        return Err(pending.error(
            file,
            format!(
                "reference {} has kind {}, allowed kinds: {:?}",
                reference.id, target.kind, kinds,
            ),
        ));
    }
    if let Some(factories) = &reference.factories
        && owner.is_none_or(|owner| reference.id != owner.id || !factories.contains(&owner.factory))
    {
        return Err(pending.error(
            file,
            format!(
                "reference {} needs a primary source declaration from factory {:?}, not {} from {}",
                reference.id,
                factories,
                target.kind,
                owner.map_or("cooked bundle", |o| o.factory.as_str()),
            ),
        ));
    }
    Ok(Edge {
        path: pending.reference.path,
        target: pending.reference.id,
        entry: target.entry.clone(),
        kind: target.kind.clone(),
        location: pending.location,
    })
}

fn located(source: &Source, references: &[Reference]) -> Result<Vec<PendingReference>, Diagnostic> {
    let paths: Vec<_> = references.iter().map(|r| r.path.as_str()).collect();
    let locations = source.locations(&paths)?;
    Ok(references
        .iter()
        .zip(locations)
        .map(|(r, location)| PendingReference {
            reference: Reference {
                path: r.path.clone(),
                id: r.id.clone(),
                kinds: r.kinds.clone(),
                factories: r.factories.clone(),
            },
            location,
        })
        .collect())
}

pub(crate) fn diagnostic(
    source: &Source,
    file: &str,
    path: &str,
    message: impl Into<String>,
) -> Diagnostic {
    let location = source
        .locations(&[path])
        .map_or_else(|e| e.location, |mut locations| locations.remove(0));
    Diagnostic {
        code: "invalid-content",
        file: file.into(),
        path: path.into(),
        location,
        message: message.into(),
    }
}
