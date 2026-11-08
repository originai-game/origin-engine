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
    /// Keys of the nodes that entered or left the trees since they were
    /// last taken (`Ui::note_semantics`).
    pub(crate) moved_keys: Vec<u64>,
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
                self.moved_keys.push(nodes[j].key);
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

/// Frames a screen reader's bridge may fall behind before it reads the
/// whole tree again.
const SEMANTIC_FRAMES: u64 = 600;

/// The keys whose accessibility may have changed, by frame, for screen
/// reader bridges to read only what changed since they last did
/// (`Ui::semantic_changes`). Kept for [`SEMANTIC_FRAMES`] frames.
#[derive(Clone, Debug, Default)]
pub(crate) struct SemanticLog {
    /// The frame being finished (one past the last finished one).
    pub frame: u64,
    /// Entries from this frame on are kept; a bridge synchronised before
    /// it, or before `full` (a frame that changed what every node says),
    /// reads the whole tree.
    floor: u64,
    full: u64,
    entries: std::collections::VecDeque<(u64, u64)>,
    /// The focus and scope last frame ended with, and the scrollers'
    /// offsets as they were then.
    focus: [u64; 4],
    scope: u64,
    offsets: crate::id::KeyMap<(f32, f32)>,
}

impl SemanticLog {
    pub(crate) fn note(&mut self, key: u64) {
        if key != 0 && self.full != self.frame + 1 {
            self.entries.push_back((self.frame, key));
        }
    }

    /// Every node may say something else this frame.
    pub(crate) fn note_all(&mut self) {
        self.full = self.frame + 1;
        while self.entries.back().is_some_and(|&(f, _)| f == self.frame) {
            self.entries.pop_back();
        }
    }

    /// The keys changed after frame `since` (a bridge's last), or `None`
    /// when the bridge reads the whole tree (never synchronised, fallen
    /// behind, or every node changed since).
    pub(crate) fn since(&self, since: Option<u64>) -> Option<Vec<u64>> {
        let since = since?;
        if since < self.floor || self.full > since {
            return None;
        }
        let mut keys: Vec<u64> = self
            .entries
            .iter()
            .rev()
            .take_while(|&&(f, _)| f >= since)
            .map(|&(_, k)| k)
            .collect();
        keys.sort_unstable();
        keys.dedup();
        Some(keys)
    }
}

impl Ui {
    /// Notes what this frame may have changed for screen readers: nodes
    /// that entered or left the trees (`TreeIndex::moved_keys`), nodes
    /// written and their named ancestors and next sibling (a control is
    /// named by its text, a field by the label before it), nodes drawn
    /// elsewhere (`repainted`), the focus moved, the scope changed, and
    /// scrollers moved.
    pub(crate) fn note_semantics(&mut self, repainted: Option<&[usize]>) {
        let log = &mut self.semantics;
        let s = &self.state;
        let focus = [s.focus, s.pad_focus[0], s.pad_focus[1], s.pad_focus[2]];
        if repainted.is_none() || s.scope != log.scope {
            log.note_all();
        }
        log.scope = s.scope;
        for (a, b) in log.focus.into_iter().zip(focus) {
            if a != b {
                log.note(a);
                log.note(b);
            }
        }
        log.focus = focus;
        // A node entered or left: the tree's shape changed, and a bridge
        // reads it whole (its parents' children in the tree's order).
        if !self.tree.moved_keys.is_empty() {
            self.tree.moved_keys.clear();
            log.note_all();
        }
        for &i in self.emitted.iter().chain(repainted.unwrap_or_default()) {
            // A rich text's links are nodes of their own: the shape again.
            if !self.nodes[i].links.is_empty() {
                log.note_all();
            }
        }
        // Node 0 is the frame's root, emitted every frame and never an
        // accessibility record.
        for &i in self.emitted.iter().filter(|&&i| i != 0) {
            let mut j = i;
            while j != NONE && j != 0 && j < self.nodes.len() {
                log.note(self.nodes[j].key);
                j = self.nodes[j].parent;
            }
            let parent = self.nodes[i].parent;
            if parent != NONE {
                let kids = &self.nodes[parent].children;
                if let Some(at) = kids.iter().position(|&c| c == i)
                    && let Some(&next) = kids.get(at + 1)
                {
                    log.note(self.nodes[next].key);
                }
            }
        }
        for &i in repainted.unwrap_or_default() {
            log.note(self.nodes[i].key);
        }
        let mut offsets = crate::id::KeyMap::default();
        for &i in &self.kept.scrollers {
            let key = self.nodes[i].key;
            let at = (
                self.state.scroll_x.get(&key).copied().unwrap_or(0.0),
                self.state.scroll.get(&key).copied().unwrap_or(0.0),
            );
            if log.offsets.get(&key) != Some(&at) {
                log.note(key);
            }
            offsets.insert(key, at);
        }
        log.offsets = offsets;
        log.frame += 1;
        let floor = log.frame.saturating_sub(SEMANTIC_FRAMES);
        while log.entries.front().is_some_and(|&(f, _)| f < floor) {
            log.entries.pop_front();
        }
        log.floor = floor;
    }

    /// The keys whose accessibility may have changed since a bridge last
    /// read it at `since` (`None`: it reads the whole tree), and the frame
    /// it has now read up to.
    pub fn semantic_changes(&self, since: Option<u64>) -> (Option<Vec<u64>>, u64) {
        (self.semantics.since(since), self.semantics.frame)
    }
}
