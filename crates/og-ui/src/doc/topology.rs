//! Logical element order is independent of render materialization. Structural
//! blocks contribute their instantiated elements, not extra selector siblings.

use super::UiDocument;
use crate::bind::InteractionState;
use crate::reactive::graph::MountedNode;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Materialization {
    #[default]
    Deferred,
    Overscan,
    Visible,
}

#[derive(Clone, Debug)]
pub struct Instance {
    pub identity: MountedNode,
    pub document: Arc<UiDocument>,
    pub document_node: usize,
    pub attributes: BTreeMap<String, Value>,
    pub direct_text: String,
    pub states: BTreeSet<InteractionState>,
    pub materialization: Materialization,
}

pub struct SelectorNode<'a> {
    pub document: &'a UiDocument,
    pub document_node: usize,
    pub attributes: &'a BTreeMap<String, Value>,
    pub direct_text: &'a str,
    pub states: &'a BTreeSet<InteractionState>,
    pub materialization: Materialization,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FactChange {
    Attribute {
        node: MountedNode,
        name: String,
    },
    Text(MountedNode),
    State {
        node: MountedNode,
        state: InteractionState,
    },
}

#[derive(Clone, Debug)]
pub struct Removed {
    pub instance: Instance,
    pub parent: Option<MountedNode>,
    pub position: usize,
}

#[derive(Clone, Debug)]
pub struct SiblingChange {
    pub parent: Option<MountedNode>,
    pub old_range: Range<usize>,
    pub new_range: Range<usize>,
    pub old_count: usize,
    pub new_count: usize,
    pub old_neighbors: [Option<MountedNode>; 2],
    pub new_neighbors: [Option<MountedNode>; 2],
    pub added: Vec<MountedNode>,
    pub moved: Option<MountedNode>,
    /// Owns the old facts and relations until the consumer drops the change.
    pub removed: Vec<Removed>,
}

type Link = Option<usize>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopologyWork {
    pub order_nodes: usize,
    pub style_intervals: usize,
}

#[derive(Clone, Debug)]
struct Entry {
    instance: Instance,
    parent: Link,
    children: Link,
    left: Link,
    right: Link,
    order_parent: Link,
    priority: u64,
    size: usize,
    style_epoch: u64,
    subtree_epoch: u64,
    resolved_epoch: u64,
}

/// An order-statistic treap per parent. Insertions and moves change tree links,
/// not every following element's stored index. Priorities depend only on the
/// allocation serial, never a clock or the simulation's random stream.
#[derive(Clone, Debug, Default)]
pub struct Topology {
    entries: Vec<Option<Entry>>,
    free: Vec<usize>,
    identities: BTreeMap<MountedNode, usize>,
    roots: Link,
    serial: u64,
    style_epoch: u64,
    changes: Vec<SiblingChange>,
    facts: Vec<FactChange>,
    work: TopologyWork,
}

impl Topology {
    fn entry(&self, i: usize) -> &Entry {
        self.entries[i].as_ref().expect("live topology entry")
    }

    fn entry_mut(&mut self, i: usize) -> &mut Entry {
        self.entries[i].as_mut().expect("live topology entry")
    }

    fn size(&self, root: Link) -> usize {
        root.map_or(0, |i| self.entry(i).size)
    }

    fn root(&self, parent: Link) -> Link {
        parent.map_or(self.roots, |i| self.entry(i).children)
    }

    fn set_root(&mut self, parent: Link, root: Link) {
        if let Some(i) = parent {
            self.entry_mut(i).children = root;
        } else {
            self.roots = root;
        }
        if let Some(i) = root {
            self.entry_mut(i).order_parent = None;
        }
    }

    fn update(&mut self, i: usize, left: Link, right: Link) {
        let size = 1 + self.size(left) + self.size(right);
        let e = self.entry_mut(i);
        e.left = left;
        e.right = right;
        e.size = size;
        e.order_parent = None;
        for child in [left, right].into_iter().flatten() {
            self.entry_mut(child).order_parent = Some(i);
        }
    }

    fn push_style(&mut self, i: usize) {
        let e = self.entry_mut(i);
        let epoch = e.subtree_epoch;
        e.subtree_epoch = 0;
        e.style_epoch = e.style_epoch.max(epoch);
        let children = [e.left, e.right];
        for child in children.into_iter().flatten() {
            let e = self.entry_mut(child);
            e.subtree_epoch = e.subtree_epoch.max(epoch);
        }
    }

    fn split(&mut self, root: Link, at: usize) -> (Link, Link) {
        let Some(i) = root else { return (None, None) };
        self.work.order_nodes += 1;
        self.push_style(i);
        let (left, right) = (self.entry(i).left, self.entry(i).right);
        let count = self.size(left);
        if at <= count {
            let (a, b) = self.split(left, at);
            self.update(i, b, right);
            if let Some(a) = a {
                self.entry_mut(a).order_parent = None;
            }
            (a, Some(i))
        } else {
            let (a, b) = self.split(right, at - count - 1);
            self.update(i, left, a);
            if let Some(b) = b {
                self.entry_mut(b).order_parent = None;
            }
            (Some(i), b)
        }
    }

    fn merge(&mut self, left: Link, right: Link) -> Link {
        let (Some(a), Some(b)) = (left, right) else {
            let root = left.or(right);
            if let Some(i) = root {
                self.entry_mut(i).order_parent = None;
            }
            return root;
        };
        self.work.order_nodes += 1;
        if self.entry(a).priority < self.entry(b).priority {
            self.push_style(a);
            let l = self.entry(a).left;
            let r = self.merge(self.entry(a).right, right);
            self.update(a, l, r);
            Some(a)
        } else {
            self.push_style(b);
            let r = self.entry(b).right;
            let l = self.merge(left, self.entry(b).left);
            self.update(b, l, r);
            Some(b)
        }
    }

    fn at(&self, mut root: Link, mut index: usize) -> Link {
        while let Some(i) = root {
            let e = self.entry(i);
            let count = self.size(e.left);
            match index.cmp(&count) {
                std::cmp::Ordering::Less => root = e.left,
                std::cmp::Ordering::Equal => return Some(i),
                std::cmp::Ordering::Greater => {
                    root = e.right;
                    index -= count + 1;
                }
            }
        }
        None
    }

    fn rank(&self, mut i: usize) -> usize {
        let mut rank = self.size(self.entry(i).left);
        while let Some(up) = self.entry(i).order_parent {
            if self.entry(up).right == Some(i) {
                rank += self.size(self.entry(up).left) + 1;
            }
            i = up;
        }
        rank
    }

    fn parent_index(&self, parent: Option<&MountedNode>) -> Result<Link, String> {
        parent
            .map(|p| {
                self.identities
                    .get(p)
                    .copied()
                    .ok_or_else(|| "missing instantiated parent".into())
            })
            .transpose()
    }

    fn identity(&self, i: usize) -> &MountedNode {
        &self.entry(i).instance.identity
    }

    fn neighbors(&self, parent: Link, range: &Range<usize>) -> [Option<MountedNode>; 2] {
        let root = self.root(parent);
        [
            range.start.checked_sub(1).and_then(|n| self.at(root, n)),
            self.at(root, range.end),
        ]
        .map(|i| i.map(|i| self.identity(i).clone()))
    }

    pub fn node(&self, id: &MountedNode) -> Option<SelectorNode<'_>> {
        let instance = &self.entry(*self.identities.get(id)?).instance;
        Some(SelectorNode {
            document: &instance.document,
            document_node: instance.document_node,
            attributes: &instance.attributes,
            direct_text: &instance.direct_text,
            states: &instance.states,
            materialization: instance.materialization,
        })
    }

    pub fn set_attribute(
        &mut self,
        id: &MountedNode,
        name: &str,
        value: Option<Value>,
    ) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        let attributes = &mut self.entry_mut(i).instance.attributes;
        if attributes.get(name) == value.as_ref() {
            return Ok(());
        }
        match value {
            Some(value) => {
                attributes.insert(name.into(), value);
            }
            None => {
                attributes.remove(name);
            }
        }
        self.facts.push(FactChange::Attribute {
            node: id.clone(),
            name: name.into(),
        });
        Ok(())
    }

    pub fn set_text(&mut self, id: &MountedNode, text: String) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        let old = &mut self.entry_mut(i).instance.direct_text;
        if *old != text {
            *old = text;
            self.facts.push(FactChange::Text(id.clone()));
        }
        Ok(())
    }

    pub fn set_state(
        &mut self,
        id: &MountedNode,
        state: InteractionState,
        enabled: bool,
    ) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        let states = &mut self.entry_mut(i).instance.states;
        let changed = if enabled {
            states.insert(state)
        } else {
            states.remove(&state)
        };
        if changed {
            self.facts.push(FactChange::State {
                node: id.clone(),
                state,
            });
        }
        Ok(())
    }

    pub fn set_materialization(
        &mut self,
        id: &MountedNode,
        state: Materialization,
    ) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        self.entry_mut(i).instance.materialization = state;
        Ok(())
    }

    /// Defers styling an affected sibling interval without visiting its rows.
    /// Structural edits preserve the pending invalidation on those identities.
    pub fn invalidate_styles(
        &mut self,
        parent: Option<&MountedNode>,
        range: Range<usize>,
    ) -> Result<(), String> {
        let p = self.parent_index(parent)?;
        let count = self.size(self.root(p));
        let end = range.end.min(count);
        if range.start >= end {
            return Ok(());
        }
        self.work.style_intervals += 1;
        self.style_epoch += 1;
        let (left, rest) = self.split(self.root(p), range.start);
        let (middle, right) = self.split(rest, end - range.start);
        if let Some(i) = middle {
            self.entry_mut(i).subtree_epoch = self.style_epoch;
        }
        let left = self.merge(left, middle);
        let root = self.merge(left, right);
        self.set_root(p, root);
        Ok(())
    }

    fn pending_style_epoch(&self, mut i: usize) -> u64 {
        let mut epoch = self.entry(i).style_epoch;
        loop {
            let e = self.entry(i);
            epoch = epoch.max(e.subtree_epoch);
            let Some(parent) = e.order_parent else {
                return epoch;
            };
            i = parent;
        }
    }

    pub fn style_is_stale(&self, id: &MountedNode) -> Option<bool> {
        let i = *self.identities.get(id)?;
        Some(self.pending_style_epoch(i) > self.entry(i).resolved_epoch)
    }

    /// Called after successfully computing this instance's current style, not
    /// just after making its render subtree visible.
    pub fn style_resolved(&mut self, id: &MountedNode) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        self.entry_mut(i).resolved_epoch = self.pending_style_epoch(i);
        Ok(())
    }

    pub fn parent(&self, id: &MountedNode) -> Option<&MountedNode> {
        self.entry(*self.identities.get(id)?)
            .parent
            .map(|i| self.identity(i))
    }

    pub fn position(&self, id: &MountedNode) -> Option<usize> {
        Some(self.rank(*self.identities.get(id)?))
    }

    pub fn child_count(&self, parent: Option<&MountedNode>) -> usize {
        self.parent_index(parent)
            .map_or(0, |p| self.size(self.root(p)))
    }

    pub fn child_at(&self, parent: Option<&MountedNode>, index: usize) -> Option<&MountedNode> {
        let p = self.parent_index(parent).ok()?;
        self.at(self.root(p), index).map(|i| self.identity(i))
    }

    pub fn children(&self, parent: Option<&MountedNode>, range: Range<usize>) -> Children<'_> {
        let first = self
            .parent_index(parent)
            .ok()
            .and_then(|p| self.at(self.root(p), range.start));
        Children {
            topology: self,
            next: first,
            remaining: range.end.saturating_sub(range.start),
        }
    }

    pub fn insert(
        &mut self,
        parent: Option<&MountedNode>,
        at: usize,
        instance: Instance,
    ) -> Result<(), String> {
        let p = self.parent_index(parent)?;
        let count = self.size(self.root(p));
        if at > count || self.identities.contains_key(&instance.identity) {
            return Err("invalid sibling position or duplicate mounted identity".into());
        }
        if !instance
            .document
            .nodes
            .get(instance.document_node)
            .is_some_and(|n| matches!(n.kind, super::NodeKind::Element(_)))
        {
            return Err("selector topology accepts instantiated elements only".into());
        }
        let old_neighbors = self.neighbors(p, &(at..at));
        self.serial = self.serial.wrapping_add(1);
        let mut priority = self.serial.wrapping_add(0x9e3779b97f4a7c15);
        priority = (priority ^ (priority >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        priority = (priority ^ (priority >> 27)).wrapping_mul(0x94d049bb133111eb);
        priority ^= priority >> 31;
        let id = instance.identity.clone();
        self.style_epoch += 1;
        let entry = Entry {
            instance,
            parent: p,
            children: None,
            left: None,
            right: None,
            order_parent: None,
            priority,
            size: 1,
            style_epoch: self.style_epoch,
            subtree_epoch: 0,
            resolved_epoch: 0,
        };
        let i = if let Some(i) = self.free.pop() {
            self.entries[i] = Some(entry);
            i
        } else {
            self.entries.push(Some(entry));
            self.entries.len() - 1
        };
        self.identities.insert(id.clone(), i);
        let (left, right) = self.split(self.root(p), at);
        let left = self.merge(left, Some(i));
        let root = self.merge(left, right);
        self.set_root(p, root);
        self.changes.push(SiblingChange {
            parent: parent.cloned(),
            old_range: at..at,
            new_range: at..at + 1,
            old_count: count,
            new_count: count + 1,
            old_neighbors,
            new_neighbors: self.neighbors(p, &(at..at + 1)),
            added: vec![id],
            moved: None,
            removed: Vec::new(),
        });
        Ok(())
    }

    /// `to` is the final index, after removing the node from its old position.
    pub fn move_to(&mut self, id: &MountedNode, to: usize) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        let p = self.entry(i).parent;
        let count = self.size(self.root(p));
        if to >= count {
            return Err("sibling move is out of bounds".into());
        }
        let from = self.rank(i);
        if from == to {
            return Ok(());
        }
        let old_neighbors = self.neighbors(p, &(from..from + 1));
        let (left, rest) = self.split(self.root(p), from);
        let (node, right) = self.split(rest, 1);
        let root = self.merge(left, right);
        let (left, right) = self.split(root, to);
        let left = self.merge(left, node);
        let root = self.merge(left, right);
        self.set_root(p, root);
        self.changes.push(SiblingChange {
            parent: p.map(|i| self.identity(i).clone()),
            old_range: from..from + 1,
            new_range: to..to + 1,
            old_count: count,
            new_count: count,
            old_neighbors,
            new_neighbors: self.neighbors(p, &(to..to + 1)),
            added: Vec::new(),
            moved: Some(id.clone()),
            removed: Vec::new(),
        });
        Ok(())
    }

    pub fn remove(&mut self, id: &MountedNode) -> Result<(), String> {
        let i = *self.identities.get(id).ok_or("missing mounted identity")?;
        let p = self.entry(i).parent;
        let at = self.rank(i);
        let count = self.size(self.root(p));
        let old_neighbors = self.neighbors(p, &(at..at + 1));
        let parent = p.map(|i| self.identity(i).clone());
        let mut pending = vec![i];
        let mut removed = Vec::new();
        let mut slots = Vec::new();
        while let Some(i) = pending.pop() {
            let e = self.entry(i);
            let children: Vec<_> = self
                .children(Some(&e.instance.identity), 0..self.size(e.children))
                .map(|id| self.identities[id])
                .collect();
            pending.extend(children);
            removed.push(Removed {
                instance: e.instance.clone(),
                parent: e.parent.map(|p| self.identity(p).clone()),
                position: self.rank(i),
            });
            slots.push(i);
        }
        let (left, rest) = self.split(self.root(p), at);
        let (_, right) = self.split(rest, 1);
        let root = self.merge(left, right);
        self.set_root(p, root);
        for i in slots {
            let e = self.entries[i].take().expect("removed topology entry");
            self.identities.remove(&e.instance.identity);
            self.free.push(i);
        }
        self.changes.push(SiblingChange {
            parent,
            old_range: at..at + 1,
            new_range: at..at,
            old_count: count,
            new_count: count - 1,
            old_neighbors,
            new_neighbors: self.neighbors(p, &(at..at)),
            added: Vec::new(),
            moved: None,
            removed,
        });
        Ok(())
    }

    pub fn take_changes(&mut self) -> Vec<SiblingChange> {
        std::mem::take(&mut self.changes)
    }

    pub fn take_fact_changes(&mut self) -> Vec<FactChange> {
        std::mem::take(&mut self.facts)
    }

    pub fn take_work(&mut self) -> TopologyWork {
        std::mem::take(&mut self.work)
    }
}

pub struct Children<'a> {
    topology: &'a Topology,
    next: Link,
    remaining: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a MountedNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let i = self.next?;
        self.remaining -= 1;
        let t = self.topology;
        self.next = if let Some(mut r) = t.entry(i).right {
            while let Some(l) = t.entry(r).left {
                r = l;
            }
            Some(r)
        } else {
            let mut n = i;
            loop {
                let Some(p) = t.entry(n).order_parent else {
                    break None;
                };
                if t.entry(p).left == Some(n) {
                    break Some(p);
                }
                n = p;
            }
        };
        Some(t.identity(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::SourceId;
    use serde_json::json;

    fn id(key: u64) -> MountedNode {
        MountedNode {
            mount: SourceId::new("test", "mount"),
            key,
        }
    }

    fn instance(doc: &Arc<UiDocument>, key: u64) -> Instance {
        Instance {
            identity: id(key),
            document: doc.clone(),
            document_node: doc.root,
            attributes: BTreeMap::new(),
            direct_text: String::new(),
            states: BTreeSet::new(),
            materialization: Materialization::Deferred,
        }
    }

    fn document() -> Arc<UiDocument> {
        Arc::new(
            UiDocument::parse(
                "order.svelte",
                "<meta name=\"component\" content=\"order\"><column id=\"Root\"><text id=\"Text\">x</text></column>",
            )
            .unwrap(),
        )
    }

    #[test]
    fn keyed_order_and_ranges_follow_edits_without_changing_identity() {
        let doc = document();
        let mut tree = Topology::default();
        tree.insert(None, 0, instance(&doc, 0)).unwrap();
        let mut order = Vec::new();
        for key in 1..=73 {
            let at = (key as usize * 17) % (order.len() + 1);
            tree.insert(Some(&id(0)), at, instance(&doc, key)).unwrap();
            order.insert(at, key);
        }
        tree.take_changes();
        for step in 0..200 {
            let from = (step * 13 + 7) % order.len();
            let to = (step * 29 + 3) % order.len();
            let key = order.remove(from);
            order.insert(to, key);
            tree.move_to(&id(key), to).unwrap();
            assert_eq!(
                tree.children(Some(&id(0)), 0..usize::MAX)
                    .map(|i| i.key)
                    .collect::<Vec<_>>(),
                order
            );
            assert_eq!(
                tree.children(Some(&id(0)), 4..11)
                    .map(|i| i.key)
                    .collect::<Vec<_>>(),
                order[4..11]
            );
            for (rank, &key) in order.iter().enumerate() {
                assert_eq!(tree.position(&id(key)), Some(rank));
                assert_eq!(tree.child_at(Some(&id(0)), rank), Some(&id(key)));
                assert_eq!(tree.parent(&id(key)), Some(&id(0)));
            }
            let edits = tree.take_changes();
            if from == to {
                assert!(edits.is_empty());
            } else {
                assert_eq!(edits.len(), 1);
                assert_eq!(edits[0].moved, Some(id(key)));
                assert_eq!(edits[0].old_range, from..from + 1);
                assert_eq!(edits[0].new_range, to..to + 1);
            }
        }
        let at = 31;
        let key = order.remove(at);
        tree.remove(&id(key)).unwrap();
        tree.insert(Some(&id(0)), 2, instance(&doc, 200)).unwrap();
        order.insert(2, 200);
        assert_eq!(tree.child_count(Some(&id(0))), order.len());
        assert_eq!(
            tree.children(Some(&id(0)), 0..usize::MAX)
                .map(|i| i.key)
                .collect::<Vec<_>>(),
            order
        );
        assert!(tree.move_to(&id(200), order.len()).is_err());
        assert!(tree.insert(Some(&id(0)), 0, instance(&doc, 200)).is_err());
        assert_eq!(tree.child_at(Some(&id(0)), 2), Some(&id(200)));
    }

    #[test]
    fn deferred_style_intervals_follow_identities_through_reorder() {
        let doc = document();
        let mut tree = Topology::default();
        tree.insert(None, 0, instance(&doc, 0)).unwrap();
        for key in 1..=1000 {
            tree.insert(Some(&id(0)), key as usize - 1, instance(&doc, key))
                .unwrap();
            tree.style_resolved(&id(key)).unwrap();
        }
        tree.take_work();
        tree.invalidate_styles(Some(&id(0)), 20..990).unwrap();
        let work = tree.take_work();
        assert_eq!(work.style_intervals, 1);
        assert!(work.order_nodes < 128, "interval visited {work:?}");
        tree.move_to(&id(500), 0).unwrap();
        tree.move_to(&id(1), 500).unwrap();
        tree.remove(&id(700)).unwrap();
        for key in 1..=1000 {
            if key == 700 {
                continue;
            }
            assert_eq!(
                tree.style_is_stale(&id(key)),
                Some((21..=990).contains(&key)),
                "key {key}"
            );
        }
        tree.set_materialization(&id(989), Materialization::Visible)
            .unwrap();
        assert_eq!(tree.style_is_stale(&id(989)), Some(true));
        tree.style_resolved(&id(989)).unwrap();
        tree.move_to(&id(989), 3).unwrap();
        assert_eq!(tree.style_is_stale(&id(989)), Some(false));
        assert_eq!(tree.style_is_stale(&id(990)), Some(true));
        tree.invalidate_styles(Some(&id(0)), 0..5).unwrap();
        assert_eq!(tree.style_is_stale(&id(989)), Some(true));
        assert_eq!(tree.style_is_stale(&id(1000)), Some(false));
        assert_eq!(tree.style_is_stale(&id(0)), Some(true));
    }

    #[test]
    fn removed_relations_and_selector_facts_survive_slot_reuse() {
        let doc = document();
        let mut tree = Topology::default();
        tree.insert(None, 0, instance(&doc, 1)).unwrap();
        tree.insert(Some(&id(1)), 0, instance(&doc, 2)).unwrap();
        tree.insert(Some(&id(1)), 1, instance(&doc, 3)).unwrap();
        tree.insert(Some(&id(2)), 0, instance(&doc, 4)).unwrap();
        tree.set_attribute(&id(2), "class", Some(json!("selected")))
            .unwrap();
        tree.set_text(&id(2), " ".into()).unwrap();
        tree.set_state(&id(2), InteractionState::Hover, true)
            .unwrap();
        tree.set_materialization(&id(2), Materialization::Visible)
            .unwrap();
        assert_eq!(tree.node(&id(2)).unwrap().direct_text, " ");
        assert!(
            tree.node(&id(2))
                .unwrap()
                .states
                .contains(&InteractionState::Hover)
        );
        assert_eq!(tree.take_fact_changes().len(), 3);
        tree.set_text(&id(2), " ".into()).unwrap();
        tree.set_state(&id(2), InteractionState::Hover, true)
            .unwrap();
        assert!(tree.take_fact_changes().is_empty());
        tree.take_changes();
        tree.remove(&id(2)).unwrap();
        tree.insert(Some(&id(1)), 0, instance(&doc, 5)).unwrap();
        let changes = tree.take_changes();
        let removed = &changes[0];
        assert_eq!(removed.old_neighbors, [None, Some(id(3))]);
        assert_eq!(removed.new_neighbors, [None, Some(id(3))]);
        assert_eq!(removed.removed.len(), 2);
        let old = removed
            .removed
            .iter()
            .find(|r| r.instance.identity == id(2))
            .unwrap();
        assert_eq!(old.position, 0);
        assert_eq!(old.parent, Some(id(1)));
        assert_eq!(old.instance.attributes["class"], json!("selected"));
        assert_eq!(old.instance.direct_text, " ");
        assert!(tree.node(&id(2)).is_none());
        assert!(tree.node(&id(4)).is_none());
        assert_eq!(
            tree.children(Some(&id(1)), 0..2)
                .cloned()
                .collect::<Vec<_>>(),
            [id(5), id(3)]
        );
    }
}
