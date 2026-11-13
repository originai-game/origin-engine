//! Prefabs (`docs/design.md` §2.5, G2-1): `type prefab` texts placed in
//! scenes and in other prefabs, flattened by the cook into the entities
//! the scene blob already holds.
//!
//! ```text
//! type prefab
//! version 1
//! ---
//! entity "crate" {
//!   Transform { scale 1 1 1 }
//!   entity "lid" {
//!     Health { hp 10 }
//!   }
//! }
//! ```
//!
//! An entity with a `prefab "<id>"` line is an instance: its tree is the
//! prefab's, flattened, and its block holds only the deltas, applied in
//! text order: its own component blocks (and `model`, `static`) over the
//! root's, field by field; `override "<node path>" { ... }` the same over
//! a descendant's; `remove "<node path>" [Component]` drops a node's
//! subtree or one component; `entity` blocks are added under the root. A
//! prefab whose body starts with `base "<id>"` is a variant: its one
//! entity is an instance of the base. Flattening is a pure function of
//! the texts, depth-first in text order, so both hosts spawn the same
//! entities in the same order.
//!
//! A delta whose target is gone (the prefab renamed or dropped the node,
//! the component or the field) is an [`Orphan`]: it stays in the text, a
//! dev cook notes it and a publish cook refuses it.

use super::CookError;
use og_core::Schema;
use og_core::ogt::{Block, Document, Entry, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The document type of a prefab.
pub const PREFAB_TYPE: &str = "prefab";

/// Where a flattened entity came from: the prefab placed (the outermost
/// one, for a nested instance), the node's index in that prefab's
/// flattened table, and the instance it belongs to (an index the
/// flattener hands out; its root is the first entity with it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub prefab: String,
    pub node: u32,
    pub instance: u32,
}

/// One entity of a flattened tree.
#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    /// Its component blocks, `model` and `static` lines, in text order.
    pub entries: Vec<Entry>,
    pub children: Vec<Node>,
    pub link: Option<Link>,
    pub line: usize,
    /// Per `(component, field)`, the text that gave its value (a prefab
    /// id, or `<id> override` for an instance's delta): `og prefab show`.
    pub origins: BTreeMap<(String, String), String>,
}

impl Node {
    /// The node at `path` (`""` itself, `lid/hinge` a grandchild).
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

    /// Every node depth-first, parents first, with its path.
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

    /// The `entity` entry the scene cook reads.
    pub fn to_entry(&self) -> Entry {
        let mut block = Block::new();
        block.entries.extend(self.entries.iter().cloned());
        for c in &self.children {
            block.push(c.to_entry());
        }
        Entry {
            key: "entity".into(),
            values: vec![Value::String(self.name.clone())],
            block: Some(block),
            line: self.line,
        }
    }

    /// Its tree's links, depth-first, parents first (the scene cook's
    /// entity order).
    pub fn links(&self, out: &mut Vec<Option<Link>>) {
        out.push(self.link.clone());
        for c in &self.children {
            c.links(out);
        }
    }
}

/// An override or `remove` whose target the prefab no longer has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orphan {
    /// The text holding it (a scene's or prefab's asset id).
    pub source: String,
    pub line: usize,
    /// The instance entity's name (a variant's root, for `base`).
    pub instance: String,
    /// The node path it targets (`""`: the instance's root).
    pub node: String,
    /// `Component` or `Component.field`, when it targets one.
    pub field: Option<String>,
}

impl std::fmt::Display for Orphan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = if self.node.is_empty() {
            "its root".to_string()
        } else {
            format!("node '{}'", self.node)
        };
        let target = match &self.field {
            Some(field) => format!("{field} of {node}"),
            None => node,
        };
        write!(
            f,
            "{} line {}: instance '{}': {target} is not in the prefab (orphaned override, kept in the text)",
            self.source, self.line, self.instance
        )
    }
}

/// Flattens scenes and prefabs. `texts` gives a prefab's text by asset id
/// (`None`: not a prefab of the project).
pub struct Flattener<'a> {
    texts: &'a dyn Fn(&str) -> Option<String>,
    schemas: &'a BTreeMap<String, Schema>,
    memo: BTreeMap<String, Node>,
    stack: Vec<String>,
    next_instance: u32,
    /// Every orphaned delta met, in the order met (a prefab's once).
    pub orphans: Vec<Orphan>,
    /// Every prefab reached.
    pub reached: BTreeSet<String>,
}

fn err(source: &str, line: usize, what: impl std::fmt::Display) -> CookError {
    CookError(format!("{source} line {line}: {what}"))
}

fn quoted(e: &Entry, source: &str, what: &str) -> Result<String, CookError> {
    e.string(0)
        .map(str::to_string)
        .map_err(|_| err(source, e.line, format!("{what}: a quoted string")))
}

/// Merges `delta`'s component blocks and lines into `entries`, field by
/// field. Returns the fields the schema says the component has not
/// (orphans).
fn merge(
    node: &mut Node,
    delta: &Entry,
    origin: &str,
    schemas: &BTreeMap<String, Schema>,
) -> Vec<String> {
    let mut gone = Vec::new();
    match &delta.block {
        // `model`, `static`: replaced.
        None => {
            match node.entries.iter_mut().find(|e| e.key == delta.key) {
                Some(e) => *e = delta.clone(),
                None => node.entries.push(delta.clone()),
            }
            node.origins
                .insert((delta.key.clone(), String::new()), origin.to_string());
        }
        Some(fields) => {
            let schema = schemas
                .get(&delta.key)
                .or_else(|| schemas.get(&format!("og.{}", delta.key)));
            let mut kept = Block::new();
            for f in &fields.entries {
                if let Some(s) = schema
                    && s.field_index(&f.key).is_none()
                {
                    gone.push(f.key.clone());
                    continue;
                }
                kept.push(f.clone());
                node.origins
                    .insert((delta.key.clone(), f.key.clone()), origin.to_string());
            }
            match node
                .entries
                .iter_mut()
                .find(|e| e.key == delta.key && e.block.is_some())
            {
                Some(e) => {
                    let block = e.block.as_mut().expect("a block");
                    for f in kept.entries {
                        match block.entries.iter_mut().find(|x| x.key == f.key) {
                            Some(x) => *x = f,
                            None => block.push(f),
                        }
                    }
                }
                None => node.entries.push(Entry {
                    key: delta.key.clone(),
                    values: delta.values.clone(),
                    block: Some(kept),
                    line: delta.line,
                }),
            }
        }
    }
    gone
}

impl<'a> Flattener<'a> {
    pub fn new(
        texts: &'a dyn Fn(&str) -> Option<String>,
        schemas: &'a BTreeMap<String, Schema>,
    ) -> Flattener<'a> {
        Flattener {
            texts,
            schemas,
            memo: BTreeMap::new(),
            stack: Vec::new(),
            next_instance: 1,
            orphans: Vec::new(),
            reached: BTreeSet::new(),
        }
    }

    /// A scene's top-level entities, flattened; `source` is its asset id.
    /// Descendants of an instance are named `<root>/<node path>`; a
    /// generated name that is another entity's too is refused, naming
    /// both.
    pub fn scene(&mut self, source: &str, body: &Block) -> Result<Vec<Node>, CookError> {
        let mut out = Vec::new();
        for e in body.get_all("entity") {
            out.push(self.entity(source, e, true)?);
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
        // By name: who has it, and whether the cook made the name.
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
                    _ => (format!("the hand-written entity '{}'", node.name), false),
                };
                names.entry(node.name.as_str()).or_default().push(who);
            }
        }
        for whos in names.values() {
            if whos.len() > 1 && whos.iter().any(|w| w.1) {
                let made = whos.iter().position(|w| w.1).expect("one made");
                let other = if made == 0 { 1 } else { 0 };
                return Err(CookError(format!(
                    "{source}: {} collides with {}; rename one of them",
                    whos[made].0, whos[other].0
                )));
            }
        }
        Ok(out)
    }

    /// The prefab `id`, flattened: every node linked to it, with its index
    /// in the prefab's own table (depth-first, parents first).
    pub fn prefab(&mut self, id: &str) -> Result<Node, CookError> {
        if let Some(n) = self.memo.get(id) {
            return Ok(n.clone());
        }
        if let Some(at) = self.stack.iter().position(|s| s == id) {
            let mut chain: Vec<&str> = self.stack[at..].iter().map(String::as_str).collect();
            chain.push(id);
            return Err(CookError(format!(
                "prefab cycle: {} (a prefab cannot place itself, through its instances or its base)",
                chain.join(" → ")
            )));
        }
        let text = (self.texts)(id)
            .ok_or_else(|| CookError(format!("'{id}' is not a prefab of this project")))?;
        let doc = Document::parse(&text).map_err(|e| CookError(format!("{id}: {e}")))?;
        if doc.ty != PREFAB_TYPE {
            return Err(CookError(format!(
                "'{id}' is a '{}' document, not a prefab",
                doc.ty
            )));
        }
        if doc.version != 1 {
            return Err(CookError(format!(
                "{id}: prefab version {} is not supported (1)",
                doc.version
            )));
        }
        let mut base = None;
        let mut root = None;
        for e in &doc.body.entries {
            match e.key.as_str() {
                "base" if base.is_none() && root.is_none() => {
                    base = Some(crate::runtime::asset_id(&quoted(e, id, "base")?).to_string())
                }
                "base" => {
                    return Err(err(
                        id,
                        e.line,
                        "base comes once, before the prefab's entity",
                    ));
                }
                "entity" if root.is_none() => root = Some(e),
                "entity" => {
                    return Err(err(
                        id,
                        e.line,
                        "a prefab has exactly one top-level entity (its root)",
                    ));
                }
                k => {
                    return Err(err(
                        id,
                        e.line,
                        format!("unknown prefab entry '{k}' (expected base or entity)"),
                    ));
                }
            }
        }
        let root = root.ok_or_else(|| CookError(format!("{id}: a prefab has one entity")))?;
        self.stack.push(id.to_string());
        let built = match &base {
            Some(b) => {
                if root
                    .block
                    .as_ref()
                    .is_some_and(|bl| bl.get("prefab").is_some())
                {
                    return Err(err(
                        id,
                        root.line,
                        "a variant's entity is already an instance of its base: no prefab line",
                    ));
                }
                let name = quoted(root, id, "entity")?;
                self.instance(id, b, &name, root, false)
            }
            None => self.entity(id, root, false),
        };
        self.stack.pop();
        let mut node = built?;
        // Siblings are addressed by name: unique.
        for (path, n) in node.walk() {
            let mut seen = BTreeSet::new();
            for c in &n.children {
                if !c.name.is_empty() && !seen.insert(c.name.as_str()) {
                    return Err(err(
                        id,
                        c.line,
                        format!(
                            "two children of {} are named '{}'; a prefab's nodes are addressed by name",
                            if path.is_empty() {
                                "the root".to_string()
                            } else {
                                format!("'{path}'")
                            },
                            c.name
                        ),
                    ));
                }
            }
        }
        let mut index = 0;
        node.walk_mut(&mut |n| {
            n.link = Some(Link {
                prefab: id.to_string(),
                node: index,
                instance: 0,
            });
            index += 1;
        });
        self.reached.insert(id.to_string());
        self.memo.insert(id.to_string(), node.clone());
        Ok(node)
    }

    /// One `entity` entry: authored, or an instance when it has a
    /// `prefab` line.
    fn entity(&mut self, source: &str, e: &Entry, scene: bool) -> Result<Node, CookError> {
        let name = e
            .string(0)
            .map_err(|x| err(source, e.line, format!("entity: {x}")))?
            .to_string();
        let empty = Block::new();
        let block = e.block.as_ref().unwrap_or(&empty);
        let prefabs: Vec<&Entry> = block.get_all("prefab").collect();
        match prefabs.as_slice() {
            [] => {}
            [p] => {
                let id = crate::runtime::asset_id(&quoted(p, source, "prefab")?).to_string();
                return self.instance(source, &id, &name, e, scene);
            }
            [_, p, ..] => {
                return Err(err(
                    source,
                    p.line,
                    "an entity is an instance of one prefab",
                ));
            }
        }
        let mut node = Node {
            name,
            entries: Vec::new(),
            children: Vec::new(),
            link: None,
            line: e.line,
            origins: BTreeMap::new(),
        };
        for c in &block.entries {
            match c.key.as_str() {
                "entity" => node.children.push(self.entity(source, c, scene)?),
                "override" | "remove" => {
                    return Err(err(
                        source,
                        c.line,
                        format!(
                            "{} belongs in an instance (an entity with a prefab line)",
                            c.key
                        ),
                    ));
                }
                _ => {
                    if let Some(b) = &c.block {
                        for f in &b.entries {
                            node.origins
                                .insert((c.key.clone(), f.key.clone()), source.to_string());
                        }
                    }
                    node.entries.push(c.clone());
                }
            }
        }
        Ok(node)
    }

    /// An instance of prefab `id` named `name`, with the deltas of entry
    /// `e`'s block.
    fn instance(
        &mut self,
        source: &str,
        id: &str,
        name: &str,
        e: &Entry,
        scene: bool,
    ) -> Result<Node, CookError> {
        let mut root = self
            .prefab(id)
            .map_err(|x| err(source, e.line, format!("prefab '{id}': {}", x.0)))?;
        let instance = self.next_instance;
        self.next_instance += 1;
        root.walk_mut(&mut |n| {
            if let Some(l) = &mut n.link {
                l.instance = instance;
            }
        });
        let origin = format!("{source} override");
        let orphan = |line: usize, node: &str, field: Option<String>| Orphan {
            source: source.to_string(),
            line,
            instance: name.to_string(),
            node: node.to_string(),
            field,
        };
        let empty = Block::new();
        let block = e.block.as_ref().unwrap_or(&empty);
        let mut added = Vec::new();
        for c in &block.entries {
            match c.key.as_str() {
                "prefab" => {}
                "entity" => added.push(self.entity(source, c, scene)?),
                "override" => {
                    let path = quoted(c, source, "override")?;
                    let deltas = c.block.as_ref().ok_or_else(|| {
                        err(
                            source,
                            c.line,
                            "override \"<node>\" { Component { field value } }",
                        )
                    })?;
                    let Some(node) = root.find_mut(&path) else {
                        self.orphans.push(orphan(c.line, &path, None));
                        continue;
                    };
                    for d in &deltas.entries {
                        if d.key == "entity" || d.key == "prefab" {
                            return Err(err(
                                source,
                                d.line,
                                "an override holds component blocks; add an entity under the instance's root",
                            ));
                        }
                        for f in merge(node, d, &origin, self.schemas) {
                            self.orphans.push(orphan(
                                d.line,
                                &path,
                                Some(format!("{}.{f}", d.key)),
                            ));
                        }
                    }
                }
                "remove" => {
                    let path = quoted(c, source, "remove")?;
                    let comp = match c.values.get(1) {
                        None => None,
                        Some(v) => {
                            Some(v.as_word().or_else(|| v.as_str()).ok_or_else(|| {
                                err(source, c.line, "remove \"<node>\" [Component]")
                            })?)
                        }
                    };
                    match comp {
                        None if path.is_empty() => {
                            return Err(err(
                                source,
                                c.line,
                                "an instance cannot remove its own root",
                            ));
                        }
                        None => {
                            let (parent, leaf) = path.rsplit_once('/').unwrap_or(("", &path));
                            let at = root.find_mut(parent).and_then(|p| {
                                let i = p.children.iter().position(|x| x.name == leaf)?;
                                Some((p, i))
                            });
                            match at {
                                Some((p, i)) => {
                                    p.children.remove(i);
                                }
                                None => self.orphans.push(orphan(c.line, &path, None)),
                            }
                        }
                        Some(comp) => {
                            let removed = root.find_mut(&path).is_some_and(|n| {
                                let before = n.entries.len();
                                n.entries.retain(|x| !(x.key == comp && x.block.is_some()));
                                n.entries.len() != before
                            });
                            if !removed {
                                self.orphans
                                    .push(orphan(c.line, &path, Some(comp.to_string())));
                            }
                        }
                    }
                }
                _ => {
                    for f in merge(&mut root, c, &origin, self.schemas) {
                        self.orphans
                            .push(orphan(c.line, "", Some(format!("{}.{f}", c.key))));
                    }
                }
            }
        }
        root.children.extend(added);
        root.name = name.to_string();
        root.line = e.line;
        if scene {
            // The tree's own descendants are `<root>/<path>`; entities the
            // instance added keep the names written for them.
            let paths: Vec<String> = root.walk().into_iter().map(|(p, _)| p).collect();
            let mut k = 0;
            root.walk_mut(&mut |n| {
                let path = &paths[k];
                k += 1;
                if !path.is_empty() && n.link.as_ref().is_some_and(|l| l.instance == instance) {
                    n.name = if name.is_empty() {
                        String::new()
                    } else {
                        format!("{name}/{path}")
                    };
                }
            });
        }
        Ok(root)
    }
}

/// The prefabs a scene's or prefab's text places, directly (`prefab` and
/// `base` lines, at any depth).
pub fn placed(text: &str) -> Vec<String> {
    fn walk(b: &Block, out: &mut BTreeSet<String>) {
        for e in &b.entries {
            if (e.key == "prefab" || e.key == "base")
                && let Ok(s) = e.string(0)
            {
                out.insert(crate::runtime::asset_id(s).to_string());
            }
            if let Some(b) = &e.block {
                walk(b, out);
            }
        }
    }
    let mut out = BTreeSet::new();
    if let Ok(doc) = Document::parse(text) {
        walk(&doc.body, &mut out);
    }
    out.into_iter().collect()
}

/// Every prefab `text` reaches, transitively, sorted (for the stamp).
pub fn reached(text: &str, texts: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut todo = placed(text);
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(t) = texts(&id) {
            todo.extend(placed(&t));
        }
    }
    seen.into_iter().collect()
}
