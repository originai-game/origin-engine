//! Checked prefab trees and ordered instance deltas. Flattening is a pure
//! function of native declarations, independent of author text syntax.
use super::CookError;
pub use super::scene::Document;
use crate::cook::source_schemas::world::scene::{EntityInput, PrefabEdit};
use og_core::Schema;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The same checked author snapshots used by cooking, without cooking assets
/// or running the game's runtime entry. Inspection keeps canonical references.
pub fn documents(
    root: &std::path::Path,
    overlay: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Arc<Document>>, String> {
    use crate::cook::{
        source_content, source_index, source_project, source_records, source_schemas,
    };
    let project =
        source_project::Project::open_with_overlay(root, overlay).map_err(|e| e.to_string())?;
    let index = source_index::Index::open_with_overlay(root, &project, overlay)
        .map_err(|e| e.to_string())?;
    let records = source_records::Records::open(root, &project)?;
    let packed = Arc::new(records.packed()?);
    let mut evaluator = source_content::Evaluator::default();
    let mut out = BTreeMap::new();
    for entry in index
        .entries
        .values()
        .filter(|e| matches!(e.factory.as_str(), "scene" | "prefab"))
    {
        let content = evaluator
            .evaluate(
                root,
                &project,
                entry,
                overlay,
                records.contexts[&entry.module].schema(&entry.factory),
            )
            .map_err(|e| e.to_string())?;
        let file: source_schemas::File =
            serde_json::from_value(content.data().root().into_json()).map_err(|e| e.to_string())?;
        let mut doc = if entry.factory == "scene" {
            Document::scene(serde_json::from_value(file.value).map_err(|e| e.to_string())?)
        } else {
            Document::prefab(serde_json::from_value(file.value).map_err(|e| e.to_string())?)
        };
        doc.records = packed.clone();
        doc.locate(&content);
        out.insert(entry.id.clone(), Arc::new(doc));
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub prefab: String,
    pub node: u32,
    pub instance: u32,
}

/// One flattened entity. Model/subscene bindings and record patches retain
/// their own types; there is no conversion to an OGT entry tree.
#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub model: Option<String>,
    pub subscene: Option<String>,
    pub static_model: Option<bool>,
    pub components: BTreeMap<String, Map<String, Value>>,
    pub children: Vec<Node>,
    pub link: Option<Link>,
    pub line: usize,
    pub file: String,
    pub origins: BTreeMap<(String, String), String>,
    pub sites: BTreeMap<String, super::scene::SourceSite>,
}

impl Node {
    pub fn find_mut(&mut self, path: &str) -> Option<&mut Node> {
        if path.is_empty() {
            return Some(self);
        }
        let (head, rest) = path.split_once('/').unwrap_or((path, ""));
        self.children
            .iter_mut()
            .find(|c| c.name == head)?
            .find_mut(rest)
    }

    /// Depth first, parents before children, with relative node paths.
    pub fn walk(&self) -> Vec<(String, &Node)> {
        fn go<'a>(n: &'a Node, path: String, out: &mut Vec<(String, &'a Node)>) {
            out.push((path.clone(), n));
            for c in &n.children {
                let p = if path.is_empty() {
                    c.name.clone()
                } else {
                    format!("{path}/{}", c.name)
                };
                go(c, p, out);
            }
        }
        let mut out = Vec::new();
        go(self, String::new(), &mut out);
        out
    }

    fn walk_mut(&mut self, f: &mut dyn FnMut(&mut Node)) {
        f(self);
        for c in &mut self.children {
            c.walk_mut(f);
        }
    }

    pub fn links(&self, out: &mut Vec<Option<Link>>) {
        out.push(self.link.clone());
        for c in &self.children {
            c.links(out);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orphan {
    pub source: String,
    pub line: usize,
    pub instance: String,
    pub node: String,
    pub field: Option<String>,
}

impl std::fmt::Display for Orphan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = if self.node.is_empty() {
            "its root".to_string()
        } else {
            format!("node '{}'", self.node)
        };
        let target = self
            .field
            .as_ref()
            .map_or(node.clone(), |field| format!("{field} of {node}"));
        write!(
            f,
            "{} line {}: instance '{}': {target} is not in the prefab (orphaned override, kept in the source)",
            self.source, self.line, self.instance
        )
    }
}

pub struct Flattener<'a> {
    documents: &'a BTreeMap<String, Arc<Document>>,
    schemas: &'a BTreeMap<String, Schema>,
    memo: BTreeMap<String, Node>,
    stack: Vec<String>,
    next_instance: u32,
    pub orphans: Vec<Orphan>,
    pub reached: BTreeSet<String>,
}

fn err(source: &str, line: usize, what: impl std::fmt::Display) -> CookError {
    CookError(format!("{source}:{line}: {what}"))
}

impl<'a> Flattener<'a> {
    pub fn new(
        documents: &'a BTreeMap<String, Arc<Document>>,
        schemas: &'a BTreeMap<String, Schema>,
    ) -> Self {
        Self {
            documents,
            schemas,
            memo: BTreeMap::new(),
            stack: Vec::new(),
            next_instance: 1,
            orphans: Vec::new(),
            reached: BTreeSet::new(),
        }
    }

    pub fn scene(&mut self, source: &str, doc: &Document) -> Result<Vec<Node>, CookError> {
        let mut out = Vec::new();
        for (i, e) in doc.scene.entities.iter().enumerate() {
            out.push(self.entity(source, doc, e, &format!("/entities/{i}"), true)?);
        }
        let mut roots: BTreeMap<u32, String> = BTreeMap::new();
        for n in &out {
            for (_, node) in n.walk() {
                if let Some(l) = &node.link
                    && l.node == 0
                {
                    roots.insert(l.instance, node.name.clone());
                }
            }
        }
        let mut names: BTreeMap<&str, Vec<(String, bool)>> = BTreeMap::new();
        for n in &out {
            for (_, node) in n.walk() {
                if node.name.is_empty() {
                    continue;
                }
                let who = match &node.link {
                    Some(l) if l.node != 0 => (
                        format!(
                            "'{}' (from instance '{}' of {})",
                            node.name,
                            roots.get(&l.instance).map_or("", String::as_str),
                            l.prefab
                        ),
                        true,
                    ),
                    _ => (format!("the authored entity '{}'", node.name), false),
                };
                names.entry(&node.name).or_default().push(who);
            }
        }
        for whos in names.values() {
            if whos.len() > 1 && whos.iter().any(|w| w.1) {
                return Err(CookError(format!(
                    "{source}: {} collides with {}; rename one of them",
                    whos[0].0, whos[1].0
                )));
            }
        }
        Ok(out)
    }

    pub fn prefab(&mut self, id: &str) -> Result<Node, CookError> {
        if let Some(n) = self.memo.get(id) {
            return Ok(n.clone());
        }
        if let Some(at) = self.stack.iter().position(|s| s == id) {
            let mut chain = self.stack[at..].to_vec();
            chain.push(id.into());
            return Err(CookError(format!(
                "prefab cycle: {} (a prefab cannot place itself, through its instances or its base)",
                chain.join(" → ")
            )));
        }
        let doc =
            self.documents.get(id).filter(|d| d.prefab).ok_or_else(|| {
                CookError(format!("'{id}' is not a checked prefab of this project"))
            })?;
        let root = &doc.scene.entities[0];
        if doc.base.is_some() && root.prefab.is_some() {
            return Err(err(
                &doc.file,
                doc.line("/entity"),
                "a variant's entity is already an instance of its base: no prefab property",
            ));
        }
        self.stack.push(id.into());
        let built = match &doc.base {
            Some(base) => self.instance(id, doc, base, root, "/entity", false),
            None => self.entity(id, doc, root, "/entity", false),
        };
        self.stack.pop();
        let mut node = built?;
        for (path, n) in node.walk() {
            let mut seen = BTreeSet::new();
            for c in &n.children {
                if !c.name.is_empty() && !seen.insert(&c.name) {
                    return Err(err(
                        id,
                        c.line,
                        format!(
                            "two children of '{path}' are named '{}'; a prefab's nodes are addressed by name",
                            c.name
                        ),
                    ));
                }
            }
        }
        let mut index = 0;
        node.walk_mut(&mut |n| {
            n.link = Some(Link {
                prefab: id.into(),
                node: index,
                instance: 0,
            });
            index += 1;
        });
        self.reached.insert(id.into());
        self.memo.insert(id.into(), node.clone());
        Ok(node)
    }

    fn entity(
        &mut self,
        source: &str,
        doc: &Document,
        e: &EntityInput,
        path: &str,
        scene: bool,
    ) -> Result<Node, CookError> {
        if let Some(prefab) = &e.prefab {
            return self.instance(source, doc, &prefab.id, e, path, scene);
        }
        if !e.edits.is_empty() {
            return Err(err(
                &doc.file,
                doc.line(path),
                "edits belong to an instance (an entity with prefab or a variant's base)",
            ));
        }
        let mut node = Node {
            name: e.name.clone(),
            model: e.model.as_ref().map(|r| r.id.clone()),
            subscene: e.scene.as_ref().map(|r| r.id.clone()),
            static_model: e.static_model,
            components: e.components.clone(),
            children: Vec::new(),
            link: None,
            line: doc.line(path),
            file: doc.file.clone(),
            origins: BTreeMap::new(),
            sites: BTreeMap::new(),
        };
        if let Some(site) = doc.site(path) {
            node.sites.insert(String::new(), site);
        }
        for (c, fields) in &e.components {
            for f in fields.keys() {
                node.origins.insert((c.clone(), f.clone()), source.into());
                if let Some(site) = doc.field_site(path, c, f) {
                    node.sites.insert(format!("{c}.{f}"), site);
                }
            }
        }
        for (i, c) in e.children.iter().enumerate() {
            node.children.push(self.entity(
                source,
                doc,
                c,
                &format!("{path}/children/{i}"),
                scene,
            )?);
        }
        Ok(node)
    }

    fn instance(
        &mut self,
        source: &str,
        doc: &Document,
        id: &str,
        e: &EntityInput,
        path: &str,
        scene: bool,
    ) -> Result<Node, CookError> {
        let line = doc.line(path);
        let mut root = self
            .prefab(id)
            .map_err(|x| err(&doc.file, line, format!("prefab '{id}': {x}")))?;
        let instance = self.next_instance;
        self.next_instance += 1;
        root.walk_mut(&mut |n| {
            if let Some(l) = &mut n.link {
                l.instance = instance;
            }
        });
        let origin = format!("{source} override");
        let orphan = |node: &str, field: Option<String>| Orphan {
            source: source.into(),
            line,
            instance: e.name.clone(),
            node: node.into(),
            field,
        };
        let merge = |node: &mut Node,
                     model: Option<&str>,
                     static_model,
                     components: &BTreeMap<String, Map<String, Value>>,
                     at: &str| {
            if let Some(m) = model {
                node.model = Some(m.into());
                node.origins
                    .insert(("model".into(), String::new()), origin.clone());
            }
            if let Some(s) = static_model {
                node.static_model = Some(s);
                node.origins
                    .insert(("static".into(), String::new()), origin.clone());
            }
            let mut gone = Vec::new();
            for (name, fields) in components {
                for (field, value) in fields {
                    if self
                        .schemas
                        .get(name)
                        .is_some_and(|s| s.field_index(field).is_none())
                    {
                        gone.push(format!("{name}.{field}"));
                        continue;
                    }
                    node.components
                        .entry(name.clone())
                        .or_default()
                        .insert(field.clone(), value.clone());
                    node.origins
                        .insert((name.clone(), field.clone()), origin.clone());
                    if let Some(site) = doc.field_site(at, name, field) {
                        node.sites.insert(format!("{name}.{field}"), site);
                    }
                }
                node.components.entry(name.clone()).or_default();
            }
            gone
        };
        for f in merge(
            &mut root,
            e.model.as_ref().map(|r| r.id.as_str()),
            e.static_model,
            &e.components,
            path,
        ) {
            self.orphans.push(orphan("", Some(f)));
        }
        if let Some(subscene) = &e.scene {
            root.subscene = Some(subscene.id.clone());
        }
        if let Some(site) = doc.site(path) {
            root.sites.insert(String::new(), site);
        }
        for (edit_index, edit) in e.edits.iter().enumerate() {
            match edit {
                PrefabEdit::Override {
                    path: target,
                    model,
                    static_model,
                    components,
                } => {
                    let Some(node) = root.find_mut(target) else {
                        self.orphans.push(orphan(target, None));
                        continue;
                    };
                    for f in merge(
                        node,
                        model.as_ref().map(|r| r.id.as_str()),
                        *static_model,
                        components,
                        &format!("{path}/edits/{edit_index}"),
                    ) {
                        self.orphans.push(orphan(target, Some(f)));
                    }
                }
                PrefabEdit::Remove { path, component } => match component {
                    None if path.is_empty() => {
                        return Err(err(
                            &doc.file,
                            line,
                            "an instance cannot remove its own root",
                        ));
                    }
                    None => {
                        let (parent, leaf) = path.rsplit_once('/').unwrap_or(("", path));
                        let at = root.find_mut(parent).and_then(|p| {
                            p.children
                                .iter()
                                .position(|c| c.name == leaf)
                                .map(|i| (p, i))
                        });
                        if let Some((p, i)) = at {
                            p.children.remove(i);
                        } else {
                            self.orphans.push(orphan(path, None));
                        }
                    }
                    Some(comp) => {
                        let removed = root
                            .find_mut(path)
                            .is_some_and(|n| n.components.remove(comp).is_some());
                        if !removed {
                            self.orphans.push(orphan(path, Some(comp.clone())));
                        }
                    }
                },
            }
        }
        for (i, child) in e.children.iter().enumerate() {
            root.children.push(self.entity(
                source,
                doc,
                child,
                &format!("{path}/children/{i}"),
                scene,
            )?);
        }
        root.name = e.name.clone();
        root.line = line;
        root.file = doc.file.clone();
        if scene {
            let paths: Vec<String> = root.walk().into_iter().map(|(p, _)| p).collect();
            let mut k = 0;
            root.walk_mut(&mut |n| {
                let path = &paths[k];
                k += 1;
                if !path.is_empty() && n.link.as_ref().is_some_and(|l| l.instance == instance) {
                    n.name = if e.name.is_empty() {
                        String::new()
                    } else {
                        format!("{}/{path}", e.name)
                    };
                }
            });
        }
        Ok(root)
    }
}

pub fn reached(doc: &Document, documents: &BTreeMap<String, Arc<Document>>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut todo = doc.placed();
    while let Some(id) = todo.pop() {
        if seen.insert(id.clone())
            && let Some(doc) = documents.get(&id)
        {
            todo.extend(doc.placed());
        }
    }
    seen.into_iter().collect()
}
