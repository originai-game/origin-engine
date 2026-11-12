//! Container queries through the settlement hook. Layout publishes each query
//! container's content box; elements whose `@container` conditions read a
//! changed box resolve again from their evaluated inputs (no viewmodel, no
//! second build) and their render fields are replaced before paint. On the
//! first frame every condition is unknown until its container is measured,
//! so the first settlement runs this too.
pub(crate) use super::host::Queried;
use super::*;
use crate::ui::settlement::{ContainerMeasurement, SettlementError};

impl Ui {
    /// Watches a query container for settlement as it is built: its first
    /// layout publishes even a zero box.
    pub(super) fn build_container(&mut self, key: u64) {
        self.docs.host.containers_now.insert(key);
        if self.docs.host.containers.insert(key) {
            self.watch_layout_container(key);
        }
    }

    /// Unwatches and forgets containers no longer built or mounted.
    pub(super) fn watch_document_containers(&mut self, live: BTreeSet<u64>) {
        let gone: Vec<u64> = self
            .docs
            .host
            .containers
            .difference(&live)
            .copied()
            .collect();
        for key in gone {
            self.unwatch_layout_container(key);
            self.docs.host.container_sizes.remove(&key);
        }
        let new: Vec<u64> = live
            .difference(&self.docs.host.containers)
            .copied()
            .collect();
        for key in new {
            self.watch_layout_container(key);
        }
        self.docs.host.containers = live;
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
                .host
                .container_sizes
                .insert(change.key, (change.current.width, change.current.height));
            changed.insert(change.key);
        }
        let mut dirty = self.requery_immediate(&changed);
        let mut host = std::mem::take(&mut self.docs.host.retained);
        let retained = match documents() {
            Some(documents) => documents.requery(&mut host, self, &changed),
            None => Ok(Vec::new()),
        };
        self.docs.host.retained = host;
        dirty.extend(retained.map_err(SettlementError::Resolver)?);
        Ok(dirty)
    }

    fn requery_immediate(&mut self, changed: &BTreeSet<u64>) -> Vec<usize> {
        let mut queried = std::mem::take(&mut self.docs.host.queried);
        // Elements restyled in this pass whose computed values changed, by
        // path: their descendants resolve again under the new values.
        let mut restyled: BTreeMap<String, Arc<crate::doc::variables::Computed>> = BTreeMap::new();
        let mut dirty = Vec::new();
        for q in &mut queried {
            let parent = q
                .path
                .rsplit_once('/')
                .and_then(|(parent, _)| restyled.get(parent).cloned());
            if parent.is_none() && q.reads.is_disjoint(changed) {
                continue;
            }
            let inherited = parent.unwrap_or_else(|| q.inherited.clone());
            let b = Build {
                doc: q.doc.clone(),
                mount: String::new(),
                vm: &Value::Null,
                env: &crate::bind::NoEnv,
                locals: Vec::new(),
                row_key: String::new(),
                inherited: inherited.clone(),
                events: Vec::new(),
                writes: Vec::new(),
                cached: Some(&q.replay.cached),
                recording: Default::default(),
            };
            let decisions = &q.replay.decisions;
            let r = self.resolve(
                &b,
                &q.doc.nodes[q.node],
                &q.path,
                &mut Built::default(),
                &inherited,
                &mut |s| {
                    decisions
                        .iter()
                        .any(|&(at, matched)| matched && at == s as *const _ as usize)
                },
            );
            q.inherited = inherited;
            q.reads = r.cascade.container_reads.clone();
            if r.cascade.computed != q.computed {
                restyled.insert(q.path.clone(), r.cascade.computed.clone());
                q.computed = r.cascade.computed.clone();
            }
            let fields = self.docs.host.render_fields.get(&q.key).cloned();
            if let Some(index) =
                self.replace_fields(fields, &q.path, &q.doc, &q.doc.nodes[q.node], &r)
            {
                dirty.push(index);
            }
        }
        self.docs.host.queried = queried;
        dirty
    }

    /// Replaces one built element's document fields from its unstyled
    /// baseline; answers its layout node.
    pub(super) fn replace_fields(
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
    /// Remembers an immediate-built element that settlement may restyle.
    pub(super) fn remember_queried(
        &mut self,
        b: &Build<'_>,
        node: usize,
        path: &str,
        key: u64,
        r: &Resolved,
    ) {
        let Some(replay) = r.replay.clone() else {
            return;
        };
        self.host.queried.push(Queried {
            doc: b.doc.clone(),
            node,
            path: path.to_owned(),
            key,
            inherited: b.inherited.clone(),
            computed: r.cascade.computed.clone(),
            reads: r.cascade.container_reads.clone(),
            replay,
        });
    }

    /// Samples the `scroll-state()` containers read last tick: a container
    /// whose flags changed restyles its readers (mounted ones through
    /// `scroll_changed`; immediate builds read the new flags as they build).
    /// `scrolled` keeps each axis's most recent direction.
    pub(super) fn sync_scroll_states(
        &mut self,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) {
        use crate::doc::media::scroll::*;
        let mut keys = std::mem::take(&mut self.host.scroll_now);
        keys.extend(self.host.retained.scroll_reads());
        let mut states = BTreeMap::new();
        for key in keys {
            let offset = interaction.offset(key);
            let (old_offset, old) = self
                .host
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
                self.host.scroll_changed.insert(key);
            }
            states.insert(key, (offset, flags));
        }
        self.host.scroll_states = states;
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
