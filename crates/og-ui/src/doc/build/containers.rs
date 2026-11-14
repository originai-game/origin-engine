//! Container queries through the settlement hook. Layout publishes each query
//! container's content box; elements whose `@container` conditions read a
//! changed box resolve again from their evaluated inputs (no viewmodel, no
//! second build) and their render fields are replaced before paint. On the
//! first frame every condition is unknown until its container is measured,
//! so the first settlement runs this too.
use super::*;
use crate::ui::settlement::{ContainerMeasurement, SettlementError};

/// The query containers documents read: `@container` boxes layout settles,
/// and the `scroll-state()` flags sampled as a tick starts.
#[derive(Clone, Debug, Default)]
pub(crate) struct Containers {
    /// Query containers' settled content boxes, by layout key.
    pub(crate) sizes: crate::doc::container::Sizes,
    /// Query containers watched for settlement, and those built this tick.
    pub(crate) watched: BTreeSet<u64>,
    pub(crate) built: BTreeSet<u64>,
    /// `scroll-state()` containers read: their offset and flags
    /// (`media::scroll`) at this tick's start, those the elements built
    /// this tick read, and those whose flags changed at its start.
    pub(crate) scroll_states: BTreeMap<u64, (og_core::glam::Vec2, u16)>,
    pub(crate) scroll_read: BTreeSet<u64>,
    pub(crate) scroll_changed: BTreeSet<u64>,
}

impl Ui {
    /// Watches a query container for settlement as it is built: its first
    /// layout publishes even a zero box.
    pub fn build_container(&mut self, key: u64) {
        self.docs.containers.built.insert(key);
        if self.docs.containers.watched.insert(key) {
            self.watch_layout_container(key);
        }
    }

    /// Unwatches and forgets containers no longer built or mounted.
    pub(super) fn watch_document_containers(&mut self, live: BTreeSet<u64>) {
        let gone: Vec<u64> = self
            .docs
            .containers
            .watched
            .difference(&live)
            .copied()
            .collect();
        for key in gone {
            self.unwatch_layout_container(key);
            self.docs.containers.sizes.remove(&key);
        }
        let new: Vec<u64> = live
            .difference(&self.docs.containers.watched)
            .copied()
            .collect();
        for key in new {
            self.watch_layout_container(key);
        }
        self.docs.containers.watched = live;
    }

    /// The settlement resolver for documents: records the measured boxes and
    /// restyles the elements that read them, answering their layout nodes.
    pub(crate) fn settle_document_containers(
        &mut self,
        changes: &[ContainerMeasurement],
    ) -> Result<Vec<usize>, SettlementError> {
        let mut changed = BTreeSet::new();
        for change in changes {
            self.docs
                .containers
                .sizes
                .insert(change.key, (change.current.width, change.current.height));
            changed.insert(change.key);
        }
        self.requery_mounted(&changed)
            .map_err(SettlementError::Resolver)
    }

    /// Restyles the mounted elements that read the query containers in
    /// `changed`, answering their layout nodes.
    fn requery_mounted(&mut self, changed: &BTreeSet<u64>) -> Result<Vec<usize>, String> {
        let mut mounts = std::mem::take(&mut self.docs.state_mut().memory.retained);
        let restyled = mounts.requery(self, changed);
        self.docs.state_mut().memory.retained = mounts;
        restyled
    }

    /// Replaces one built element's document fields from its unstyled
    /// baseline; answers its layout node.
    pub fn replace_fields(
        &mut self,
        fields: Option<fields::RenderFields>,
        path: &str,
        doc: &crate::doc::UiDocument,
        n: &DocNode,
        r: &Resolved,
    ) -> Option<usize> {
        let fields = fields?;
        let i = fields.index;
        let hidden = self.tree.nodes[i].hidden;
        self.apply_document_fields(doc, n, r, &fields);
        // A Rive frame already advanced this tick when its board is unchanged.
        let layout_key = self.tree.nodes[i].key;
        let playing = self
            .interaction
            .state
            .rive
            .get(&layout_key)
            .map(|s| s.board.clone());
        match r.str("rive") {
            Some(board) if playing.as_deref() != Some(board) => self.docs.play_rive(
                i,
                board,
                path,
                &self.env,
                &mut self.out,
                &mut self.interaction,
                &mut self.tree,
            ),
            None if playing.is_some() => {
                self.interaction.state.rive.remove(&layout_key);
                if self.tree.draws.get(&i).is_some_and(|d| d.rive.is_some()) {
                    self.tree.draws.remove(&i);
                }
            }
            _ => {}
        }
        if self.tree.nodes[i].hidden != hidden {
            // `end` derived the child lists before settlement: refilter the
            // parent, and a revealed subtree whose lists were never derived.
            let parent = self.tree.nodes[i].parent;
            if parent != crate::layout::NONE {
                self.tree.refilter_children(parent);
            }
            let mut stack = vec![i];
            while let Some(at) = stack.pop() {
                if self.tree.nodes[at].is_hidden() {
                    continue;
                }
                self.tree.refilter_children(at);
                stack.extend(self.tree.nodes[at].ordered_children.iter().copied());
            }
        }
        Some(i)
    }
}

impl crate::ui::Docs {
    /// Samples the `scroll-state()` containers read last tick: a container
    /// whose flags changed restyles its readers (through `scroll_changed`).
    /// `scrolled` keeps each axis's most recent direction.
    pub(super) fn sync_scroll_states(
        &mut self,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) {
        use crate::doc::media::scroll::*;
        let mut keys = std::mem::take(&mut self.containers.scroll_read);
        keys.extend(self.state.memory.retained.scroll_reads());
        let mut states = BTreeMap::new();
        for key in keys {
            let offset = interaction.offset(key);
            let (old_offset, old) = self
                .containers
                .scroll_states
                .get(&key)
                .copied()
                .unwrap_or((offset, 0));
            let d = offset - old_offset;
            let mut scrolled = old & SCROLLED;
            if d.y != 0.0 {
                scrolled &= !((TOP | BOTTOM) << 10);
                scrolled |= if d.y < 0.0 { TOP } else { BOTTOM } << 10;
            }
            if d.x != 0.0 {
                scrolled &= !((LEFT | RIGHT) << 10);
                scrolled |= if d.x < 0.0 { LEFT } else { RIGHT } << 10;
            }
            let flags = interaction.scroll_flags_now(key, tree) | scrolled;
            if flags != old {
                self.containers.scroll_changed.insert(key);
            }
            states.insert(key, (offset, flags));
        }
        self.containers.scroll_states = states;
    }
}

impl crate::ui::Tree {
    /// A node's shown children in built order, split by flow, as `end` does.
    fn refilter_children(&mut self, i: usize) {
        let kept: Vec<usize> = self.nodes[i]
            .built_children
            .iter()
            .copied()
            .filter(|&c| !self.nodes[c].is_hidden() && self.nodes[c].parent == i)
            .collect();
        let (out, flow) = kept
            .iter()
            .partition(|&&c| self.nodes[c].props.out_of_flow());
        self.nodes[i].ordered_children = kept;
        self.nodes[i].children = flow;
        self.nodes[i].out = out;
        self.index.pending.push(i);
    }
}
