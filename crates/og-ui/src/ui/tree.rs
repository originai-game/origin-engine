//! Which nodes are in the frame's trees, each by its key. Kept across
//! frames and changed only where the tree changed (a node written, a child
//! added or dropped, a layer opened or closed), so finding a node on
//! screen never walks the tree.

use super::*;

/// The trees' membership (`Ui::tree`).
#[derive(Clone, Debug, Default)]
pub(crate) struct TreeIndex {
    /// Per slot: in one of this frame's trees.
    pub in_tree: Vec<bool>,
    /// Each node in a tree by its key, and each rich text link's key to
    /// its text's node.
    pub by_key: crate::id::KeyMap<usize>,
    pub links: crate::id::KeyMap<usize>,
    /// Per slot: its children as last synchronised, and its link keys.
    kids: Vec<Vec<usize>>,
    link_keys: Vec<Vec<u64>>,
    /// The layers last synchronised.
    floats: Vec<usize>,
    /// The nodes whose children changed at the last synchronisation, and
    /// whether every node did (a fresh frame).
    pub restructured: Vec<usize>,
    /// Slots vacated since the last synchronisation: one written again
    /// this frame takes its place in the tree again.
    vacated: Vec<usize>,
    /// Parents whose children changed after the frame's synchronisation
    /// (a container query's `display`): synchronised once layout settles.
    pub(crate) pending: Vec<usize>,
    pub all_restructured: bool,
}

impl TreeIndex {
    /// The node `key` names this frame, a rich text link's included.
    pub(crate) fn find(&self, key: u64) -> Option<usize> {
        self.links
            .get(&key)
            .or_else(|| self.by_key.get(&key))
            .copied()
    }

    /// Whether slot `i` is in a tree.
    pub(crate) fn has(&self, i: usize) -> bool {
        self.in_tree.get(i).copied().unwrap_or(false)
    }

    fn grow(&mut self, n: usize) {
        if self.in_tree.len() < n {
            self.in_tree.resize(n, false);
            self.kids.resize(n, Vec::new());
            self.link_keys.resize(n, Vec::new());
        }
    }

    fn enter(&mut self, nodes: &[Node], i: usize) {
        self.by_key.insert(nodes[i].key, i);
        let keys: Vec<u64> = nodes[i]
            .links
            .iter()
            .map(|l| child_key(nodes[i].key, l))
            .collect();
        for &k in &keys {
            self.links.insert(k, i);
        }
        self.link_keys[i] = keys;
    }

    fn leave(&mut self, nodes: &[Node], i: usize) {
        if self.by_key.get(&nodes[i].key) == Some(&i) {
            self.by_key.remove(&nodes[i].key);
        }
        for k in std::mem::take(&mut self.link_keys[i]) {
            if self.links.get(&k) == Some(&i) {
                self.links.remove(&k);
            }
        }
    }

    /// Sets slot `i`'s subtree (as last synchronised) in or out of the trees.
    fn mark(&mut self, nodes: &[Node], i: usize, inside: bool) {
        let mut stack = vec![i];
        while let Some(j) = stack.pop() {
            if self.in_tree[j] != inside {
                self.in_tree[j] = inside;
                if inside {
                    self.enter(nodes, j);
                } else {
                    self.leave(nodes, j);
                }
            }
            stack.extend_from_slice(&self.kids[j]);
        }
    }
}

impl Ui {
    /// Brings the trees' membership up to date with this
    /// frame's structure: `whole` reads every node again (a fresh frame);
    /// otherwise only the nodes written and the parents whose children
    /// were derived again (`parents`), and the layers.
    pub(crate) fn sync_tree(&mut self, parents: &[usize], written: &[usize], whole: bool) {
        let n = self.nodes.len();
        let t = &mut self.tree;
        t.grow(n);
        let nodes = &self.nodes;
        t.all_restructured |= whole;
        if whole {
            *t = TreeIndex::default();
            t.all_restructured = true;
            t.grow(n);
            for i in 0..n {
                t.kids[i].clone_from(&nodes[i].ordered_children);
            }
            t.floats.clone_from(&self.floating);
            for r in std::iter::once(0).chain(self.floating.iter().copied()) {
                if !nodes[r].hidden {
                    t.mark(nodes, r, true);
                }
            }
            return;
        }
        // Nodes whose place in a tree may have changed.
        let mut queue: Vec<usize> = std::mem::take(&mut t.vacated);
        let pending = std::mem::take(&mut t.pending);
        for &p in parents.iter().chain(&pending) {
            if p >= n {
                continue;
            }
            let new = &nodes[p].ordered_children;
            if t.kids[p] == *new {
                continue;
            }
            let old = std::mem::replace(&mut t.kids[p], new.clone());
            t.restructured.push(p);
            queue.extend(old.iter().filter(|c| !new.contains(c)));
            queue.extend(new.iter().filter(|c| !old.contains(c)));
        }
        if t.floats != self.floating {
            let old = std::mem::replace(&mut t.floats, self.floating.clone());
            queue.extend(old.iter().filter(|c| !self.floating.contains(c)));
            queue.extend(self.floating.iter().filter(|c| !old.contains(c)));
        }
        // The screen's root is always in its tree.
        if n > 0 && !t.in_tree[0] {
            t.mark(nodes, 0, true);
        }
        // Parents first, so a child reads its parent's final membership.
        let depth = |mut i: usize| {
            let mut d = 0u32;
            while i != NONE && i < n {
                d += 1;
                i = nodes[i].parent;
            }
            d
        };
        let mut queue: Vec<(u32, usize)> = queue.into_iter().map(|i| (depth(i), i)).collect();
        queue.sort_unstable();
        queue.dedup();
        for (_, c) in queue {
            let node = &nodes[c];
            let inside = c == 0
                || !node.hidden
                    && node.key != 0
                    && if node.parent == NONE {
                        t.floats.contains(&c)
                    } else {
                        t.has(node.parent) && t.kids[node.parent].contains(&c)
                    };
            t.mark(nodes, c, inside);
        }
        // Written nodes: their links as written.
        for &i in written {
            if i >= n {
                continue;
            }
            if t.in_tree[i] {
                t.leave(nodes, i);
                t.enter(nodes, i);
            }
        }
    }

    /// Takes a vacated slot out of the index.
    pub(crate) fn untree(&mut self, i: usize) {
        let t = &mut self.tree;
        if i >= t.in_tree.len() {
            return;
        }
        if t.in_tree[i] {
            let nodes = &self.nodes;
            t.mark(nodes, i, false);
        }
        t.kids[i].clear();
        t.vacated.push(i);
    }
}
