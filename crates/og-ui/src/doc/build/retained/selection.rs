//! Sibling and structural selector invalidation. Topology edits and selector
//! fact changes become sibling intervals whose styles are marked stale in
//! O(log n) each; materialized rows restyle when emitted and deferred rows
//! when they materialize, so a front insert into a virtual list restyles only
//! the visible and overscan rows.
use super::*;
use crate::doc::matching::Profile;
use crate::doc::topology::{FactChange, SiblingChange};
use std::ops::Range;

pub(crate) trait RetainedSelectionRetainedHost {
    fn profile(&mut self, ui: &Ui, document: &str) -> &Profile;

    /// The structural reads of the documents that can style `nodes`.
    fn profile_of<'a>(
        &mut self,
        ui: &Ui,
        nodes: impl IntoIterator<Item = &'a Arc<UiDocument>>,
    ) -> Profile;

    fn document_of(&self, node: &MountedNode) -> Option<Arc<UiDocument>>;

    fn stale(&mut self, parent: Option<&MountedNode>, range: Range<usize>) -> Result<(), String>;

    /// `:has()`: the elements whose descendants or later siblings include
    /// `node` (its ancestors, and each one's earlier siblings a sibling
    /// `:has()` reaches) restyle when a subject compound has it; when a
    /// `:has()` is before the subject, only the descendants of a possible
    /// anchor that can be the subject do. Nothing else can change a match.
    fn stale_has(&mut self, profile: &Profile, node: Option<&MountedNode>) -> Result<(), String>;

    /// The descendants of `anchor` down to the context's depth whose own
    /// facts can be its subject.
    fn stale_subjects(
        &mut self,
        anchor: &MountedNode,
        context: &crate::doc::matching::HasContext,
    ) -> Result<(), String>;

    /// Runs after bindings and selector facts settle and before dirty styles
    /// drain.
    fn consume_selector_changes(&mut self, ui: &mut Ui) -> Result<(), String>;

    fn sibling_change(&mut self, ui: &Ui, change: &SiblingChange) -> Result<(), String>;

    fn fact_change(&mut self, ui: &Ui, fact: &FactChange) -> Result<(), String>;
}

impl RetainedSelectionRetainedHost for RetainedHost {
    fn profile(&mut self, ui: &Ui, document: &str) -> &Profile {
        self.profiles.entry(document.to_owned()).or_insert_with(|| {
            ui.docs
                .host
                .docs
                .get(document)
                .map(|doc| Profile::of_document(doc))
                .unwrap_or_default()
        })
    }

    fn profile_of<'a>(
        &mut self,
        ui: &Ui,
        nodes: impl IntoIterator<Item = &'a Arc<UiDocument>>,
    ) -> Profile {
        let mut out = ui.docs.host.global_structure.clone();
        for doc in nodes {
            let profile = self.profile(ui, &doc.name).clone();
            out.merge(&profile);
        }
        out
    }

    fn document_of(&self, node: &MountedNode) -> Option<Arc<UiDocument>> {
        self.elements.get(node).map(|e| e.document.clone())
    }

    fn stale(&mut self, parent: Option<&MountedNode>, range: Range<usize>) -> Result<(), String> {
        if range.start < range.end {
            self.css.intervals += 1;
            self.topology.invalidate_styles(parent, range)?;
        }
        Ok(())
    }

    fn stale_has(&mut self, profile: &Profile, node: Option<&MountedNode>) -> Result<(), String> {
        if !profile.has {
            return Ok(());
        }
        let mut at = node.cloned();
        while let Some(n) = at {
            let parent = self.topology.parent(&n).cloned();
            let facts = crate::doc::matching::Tree::facts(&self.topology, &n);
            let anchor = crate::doc::matching::filter_matches(&facts, &profile.has_anchors);
            let contexts: Vec<_> = profile
                .has_context
                .iter()
                .filter(|c| {
                    crate::doc::matching::filter_matches(&facts, std::slice::from_ref(&c.anchor))
                })
                .collect();
            if let Some(k) = self.topology.position(&n) {
                if anchor {
                    self.stale(parent.as_ref(), k..k + 1)?;
                }
                if profile.has_siblings > 0 {
                    self.stale(parent.as_ref(), k.saturating_sub(profile.has_siblings)..k)?;
                }
            }
            for context in contexts {
                self.stale_subjects(&n, context)?;
            }
            at = parent;
        }
        Ok(())
    }

    fn stale_subjects(
        &mut self,
        anchor: &MountedNode,
        context: &crate::doc::matching::HasContext,
    ) -> Result<(), String> {
        let mut level = vec![anchor.clone()];
        let mut depth = 0;
        let mut hits = Vec::new();
        while !level.is_empty() && depth < context.depth {
            depth += 1;
            let mut next = Vec::new();
            for parent in &level {
                for (k, child) in self
                    .topology
                    .children(Some(parent), 0..usize::MAX)
                    .enumerate()
                {
                    let facts = crate::doc::matching::Tree::facts(&self.topology, child);
                    if crate::doc::matching::filter_matches(
                        &facts,
                        std::slice::from_ref(&context.subject),
                    ) {
                        hits.push((parent.clone(), k));
                    }
                    next.push(child.clone());
                }
            }
            level = next;
        }
        for (parent, k) in hits {
            self.stale(Some(&parent), k..k + 1)?;
        }
        Ok(())
    }

    fn consume_selector_changes(&mut self, ui: &mut Ui) -> Result<(), String> {
        for filter in &ui.docs.host.structure.filters {
            self.topology.register_filter(filter);
        }
        let changes = self.topology.take_changes();
        let facts = self.topology.take_fact_changes();
        if !ui.docs.host.structure.any {
            return Ok(());
        }
        for change in changes {
            self.css.changes += 1;
            self.sibling_change(ui, &change)?;
        }
        for fact in facts {
            self.css.changes += 1;
            self.fact_change(ui, &fact)?;
        }
        Ok(())
    }

    fn sibling_change(&mut self, ui: &Ui, change: &SiblingChange) -> Result<(), String> {
        let parent = change.parent.as_ref();
        // Its parent left later in the same refresh (an `{#if}` removed the
        // row whose children changed): nothing under it is left to restyle.
        if parent.is_some_and(|p| self.topology.node(p).is_none()) {
            return Ok(());
        }
        let mut docs: Vec<_> = parent
            .and_then(|p| self.document_of(p))
            .into_iter()
            .collect();
        for node in change
            .added
            .iter()
            .chain(&change.moved)
            .chain(change.old_neighbors.iter().flatten())
            .chain(change.new_neighbors.iter().flatten())
        {
            docs.extend(self.document_of(node));
        }
        docs.extend(change.removed.iter().map(|r| r.instance.document.clone()));
        let profile = self.profile_of(ui, &docs);
        if !profile.any {
            return Ok(());
        }
        let count = change.new_count;
        let low = change.old_range.start.min(change.new_range.start);
        let high = change.old_range.end.max(change.new_range.end).min(count);
        let moved = change.moved.is_some();
        // The moved or inserted elements themselves.
        self.stale(parent, change.new_range.clone())?;
        if profile.next {
            for neighbor in change.old_neighbors[1]
                .iter()
                .chain(&change.new_neighbors[1])
            {
                if let Some(at) = self.topology.position(neighbor) {
                    self.stale(parent, at..at + 1)?;
                }
            }
        }
        let forward = profile.forward || !profile.filters.is_empty();
        let backward = profile.backward || !profile.filters.is_empty();
        if profile.subsequent || (forward && !moved) {
            // Every later sibling gains or loses a preceding sibling.
            self.stale(parent, low..count)?;
        } else if forward {
            self.stale(parent, low..high)?;
        }
        if backward && !moved {
            self.stale(
                parent,
                0..change.new_range.end.max(change.old_range.start).min(count),
            )?;
        } else if backward {
            self.stale(parent, low..high)?;
        }
        if profile.ends {
            for neighbor in change
                .old_neighbors
                .iter()
                .chain(&change.new_neighbors)
                .flatten()
            {
                if let Some(at) = self.topology.position(neighbor) {
                    self.stale(parent, at..at + 1)?;
                }
            }
            if count > 0 {
                self.stale(parent, 0..1)?;
                self.stale(parent, count - 1..count)?;
            }
        }
        // The parent's and ancestors' `:has()`, and earlier siblings'
        // `:has(+ …)` over the changed range.
        self.stale_has(&profile, parent)?;
        if profile.has_siblings > 0 {
            let low = change.old_range.start.min(change.new_range.start);
            self.stale(parent, low.saturating_sub(profile.has_siblings)..low)?;
        }
        if profile.empty
            && (change.old_count == 0) != (change.new_count == 0)
            && let Some(parent) = parent
            && let Some(at) = self.topology.position(parent)
        {
            let grand = self.topology.parent(parent).cloned();
            self.stale(grand.as_ref(), at..at + 1)?;
        }
        Ok(())
    }

    fn fact_change(&mut self, ui: &Ui, fact: &FactChange) -> Result<(), String> {
        let node = match fact {
            FactChange::Attribute { node, .. } | FactChange::State { node, .. } => node,
            FactChange::Text(node) => node,
        };
        let relevant = |p: &Profile| match fact {
            FactChange::Attribute { name, .. } => {
                p.attributes.contains(if name.starts_with("class:") {
                    "class"
                } else {
                    name
                })
            }
            FactChange::State { state, .. } => p.states.contains(state),
            FactChange::Text(_) => p.empty,
        };
        let Some(at) = self.topology.position(node) else {
            return Ok(());
        };
        let parent = self.topology.parent(node).cloned();
        let mut docs: Vec<_> = self.document_of(node).into_iter().collect();
        docs.extend(parent.as_ref().and_then(|p| self.document_of(p)));
        let profile = self.profile_of(ui, &docs);
        if !profile.any || !relevant(&profile) {
            return Ok(());
        }
        let parent = parent.as_ref();
        let count = self.topology.child_count(parent);
        self.stale(parent, at..at + 1)?;
        let node = node.clone();
        self.stale_has(&profile, Some(&node))?;
        if profile.next {
            self.stale(parent, at + 1..(at + 2).min(count))?;
        }
        if profile.subsequent || !profile.filters.is_empty() {
            self.stale(parent, at + 1..count)?;
        }
        if !profile.filters.is_empty() {
            self.stale(parent, 0..at)?;
        }
        if profile.ancestors {
            self.stale(Some(&node), 0..usize::MAX)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Size;
    use crate::reactive::session::Command;
    use crate::{InputFrame, UiDesc, Viewport};

    const STYLE: &str = r#"
        <style>
            .item:nth-child(odd) { width: 101px; }
            .item:nth-child(even) { width: 202px; }
            .item:first-child { height: 11px; }
            .item:last-child { height: 33px; }
            .item + .item { opacity: 0.5; }
        </style>"#;

    /// What each instantiated child should resolve to, derived from the order
    /// alone: (text, width, height, opacity).
    type Expected = (String, Option<Size>, Option<Size>, Option<f32>);

    fn expected(children: &[(String, bool)]) -> Vec<Expected> {
        let n = children.len();
        children
            .iter()
            .enumerate()
            .map(|(i, (text, item))| {
                if !item {
                    return (text.clone(), None, None, None);
                }
                let width = Size::px(if i % 2 == 0 { 101.0 } else { 202.0 });
                let height = match (i == 0, i == n - 1) {
                    (_, true) => Some(Size::px(33.0)),
                    (true, false) => Some(Size::px(11.0)),
                    _ => None,
                };
                let separated = i > 0 && children[i - 1].1;
                (text.clone(), Some(width), height, separated.then_some(0.5))
            })
            .collect()
    }

    fn styles(ui: &Ui, parent: &MountedNode, range: Range<usize>) -> Vec<Expected> {
        let host = &ui.docs.host.retained;
        host.topology
            .children(Some(parent), range)
            .map(|id| {
                let values = &host.elements[id].resolved.as_ref().unwrap().values;
                (
                    host.topology.node(id).unwrap().direct_text.to_owned(),
                    values.size("width"),
                    values.size("height"),
                    values.num("opacity"),
                )
            })
            .collect()
    }

    fn rows(keys: &[&str]) -> Vec<serde_json::Map<String, Value>> {
        keys.iter()
            .map(|k| json!({"id": k, "label": k}).as_object().unwrap().clone())
            .collect()
    }

    fn tick(ui: &mut Ui, wheel: f32) {
        ui.begin(
            InputFrame {
                pointer_inside: true,
                pointer: og_core::glam::Vec2::new(100.0, 100.0),
                wheel: og_core::glam::Vec2::new(0.0, wheel),
                ..Default::default()
            },
            Viewport::new(1280.0, 720.0),
        );
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
    }

    fn mount(ui: &mut Ui, body: &str, keys: &[&str]) -> (SourceId, SourceId) {
        ui.define_document(
            UiDocument::parse(
                "zebra.svelte",
                &format!("<meta name=\"component\" content=\"zebra\">{body}{STYLE}"),
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "rows");
        let extra = SourceId::new("test", "extra");
        ui.docs
            .reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: rows(keys),
            })
            .unwrap();
        ui.docs
            .reactive
            .execute(Command::Signal {
                source: extra.clone(),
                initial: json!(""),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "zebra"),
            "zebra",
            BTreeMap::from([
                ("rows".into(), source.clone()),
                ("extra".into(), extra.clone()),
            ]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        tick(ui, 0.0);
        (source, extra)
    }

    fn root(ui: &Ui) -> MountedNode {
        ui.mounted_topology().child_at(None, 0).unwrap().clone()
    }

    fn items(keys: &[&str], extra_after: Option<&str>) -> Vec<(String, bool)> {
        let mut out = Vec::new();
        for k in keys {
            out.push((k.to_string(), true));
            if extra_after == Some(k) {
                out.push(("+".into(), false));
            }
        }
        out
    }

    #[test]
    fn zebra_and_ends_follow_keyed_reorder_conditional_insert_and_restore() {
        let mut ui = Ui::new(UiDesc::default());
        let (source, extra) = mount(
            &mut ui,
            r#"<column id="Root">{#each $rows as row (row.id)}<text id="Row" class="item">{row.label}</text>{#if $extra == row.id}<text id="Extra">+</text>{/if}{/each}</column>"#,
            &["a", "b", "c", "d", "e", "f"],
        );
        let root = root(&ui);
        let all = 0..usize::MAX;
        let mut order = vec!["a", "b", "c", "d", "e", "f"];
        assert_eq!(
            styles(&ui, &root, all.clone()),
            expected(&items(&order, None))
        );
        let saved = ui.save_state();
        for (key, before) in [("f", Some("a")), ("c", Some("b")), ("a", None)] {
            ui.docs
                .reactive
                .execute(Command::Move {
                    source: source.clone(),
                    key: key.into(),
                    before: before.map(Into::into),
                })
                .unwrap();
            let k = order.iter().position(|x| *x == key).unwrap();
            order.remove(k);
            let at = before.map_or(order.len(), |b| order.iter().position(|x| *x == b).unwrap());
            order.insert(at, key);
            tick(&mut ui, 0.0);
            assert_eq!(
                styles(&ui, &root, all.clone()),
                expected(&items(&order, None)),
                "after moving {key}"
            );
        }
        // An element inserted between rows shifts every later parity and
        // breaks the `.item + .item` adjacency of the row after it.
        assert_eq!(order, ["f", "c", "b", "d", "e", "a"]);
        ui.docs
            .reactive
            .execute(Command::Set {
                source: extra.clone(),
                value: json!("b"),
                force: false,
            })
            .unwrap();
        tick(&mut ui, 0.0);
        assert_eq!(
            styles(&ui, &root, all.clone()),
            expected(&items(&order, Some("b")))
        );
        ui.restore_state(&saved).unwrap();
        tick(&mut ui, 0.0);
        assert_eq!(
            styles(&ui, &root, all.clone()),
            expected(&items(&["a", "b", "c", "d", "e", "f"], None))
        );
    }

    #[test]
    fn separators_and_ends_follow_removals() {
        let mut ui = Ui::new(UiDesc::default());
        let (source, _) = mount(
            &mut ui,
            r#"<column id="Root"><text id="Head">head</text>{#each $rows as row (row.id)}<text id="Row" class="item">{row.label}</text>{/each}</column>"#,
            &["a", "b", "c", "d"],
        );
        let root = root(&ui);
        let mut order = vec!["a", "b", "c", "d"];
        let with_head = |order: &[&str]| {
            let mut out = vec![("head".to_string(), false)];
            out.extend(order.iter().map(|k| (k.to_string(), true)));
            out
        };
        assert_eq!(
            styles(&ui, &root, 0..usize::MAX),
            expected(&with_head(&order))
        );
        for key in ["b", "d", "a"] {
            ui.docs
                .reactive
                .execute(Command::Remove {
                    source: source.clone(),
                    key: key.into(),
                })
                .unwrap();
            order.retain(|k| *k != key);
            tick(&mut ui, 0.0);
            assert_eq!(
                styles(&ui, &root, 0..usize::MAX),
                expected(&with_head(&order)),
                "after removing {key}"
            );
        }
    }

    #[test]
    fn adjacent_move_restyles_only_its_interval() {
        let mut ui = Ui::new(UiDesc::default());
        let mut order: Vec<String> = (0..1000).map(|i| format!("r{i}")).collect();
        let keys: Vec<&str> = order.iter().map(String::as_str).collect();
        let (source, _) = mount(
            &mut ui,
            r#"<column id="Root">{#each $rows as row (row.id)}<text id="Row" class="item">{row.label}</text>{/each}</column>"#,
            &keys,
        );
        let root = root(&ui);
        tick(&mut ui, 0.0);
        assert_eq!(ui.docs.host.retained.resolved, 0);
        ui.docs
            .reactive
            .execute(Command::Move {
                source,
                key: "r500".into(),
                before: Some("r502".into()),
            })
            .unwrap();
        order.swap(500, 501);
        tick(&mut ui, 0.0);
        let work = (ui.docs.host.retained.resolved, ui.docs.host.retained.css);
        // The swapped rows, the rows after the old and new positions (`+`)
        // and the two ends: a constant, not a function of the 1,000 rows.
        assert!(work.0 <= 6, "restyled {work:?}");
        assert!(
            work.1.intervals <= 12 && work.1.selectors <= 6 * 5,
            "{work:?}"
        );
        let pairs: Vec<_> = order.iter().map(|k| (k.clone(), true)).collect();
        assert_eq!(styles(&ui, &root, 0..usize::MAX), expected(&pairs));
    }

    #[test]
    fn virtual_front_insert_restyles_visible_rows_and_zebra_holds_at_the_end() {
        let mut ui = Ui::new(UiDesc::default());
        let mut order: Vec<String> = (0..1000).map(|i| format!("r{i}")).collect();
        let keys: Vec<&str> = order.iter().map(String::as_str).collect();
        let (source, _) = mount(
            &mut ui,
            r#"<list id="Rows" row-height="40" height="400">{#each $rows as row (row.id)}<button id="Row" class="item">{row.label}</button>{/each}</list>"#,
            &keys,
        );
        let list = root(&ui);
        tick(&mut ui, 0.0);
        let materialized = |ui: &Ui| -> Vec<usize> {
            let t = ui.mounted_topology();
            (0..t.child_count(Some(&list)))
                .filter(|&i| {
                    let id = t.child_at(Some(&list), i).unwrap();
                    t.node(id).unwrap().materialization != Materialization::Deferred
                })
                .collect()
        };
        let shown = materialized(&ui).len();
        assert!((10..40).contains(&shown), "{shown} rows materialized");
        let mut row = serde_json::Map::new();
        row.insert("id".into(), json!("front"));
        row.insert("label".into(), json!("front"));
        ui.docs
            .reactive
            .execute(Command::Insert {
                source: source.clone(),
                row,
                before: Some("r0".into()),
            })
            .unwrap();
        order.insert(0, "front".into());
        tick(&mut ui, 0.0);
        // Every rank changed, but only materialized rows restyle.
        let resolved = ui.docs.host.retained.resolved;
        let now = materialized(&ui).len();
        assert!(now < shown + 4, "{now} rows materialized");
        assert_eq!(
            resolved, now,
            "restyled {resolved} of {now} materialized rows"
        );
        let css = ui.docs.host.retained.css;
        // The insert plus the inserted row's facts; each costs a few intervals.
        assert!(css.changes <= 8 && css.intervals <= 2 * 8, "{css:?}");
        assert!(css.selectors <= now * 5, "{css:?}");
        let far = ui
            .mounted_topology()
            .child_at(Some(&list), 1000)
            .unwrap()
            .clone();
        assert_eq!(
            ui.docs.host.retained.topology.style_is_stale(&far),
            Some(true)
        );
        let pairs: Vec<_> = order.iter().map(|k| (k.clone(), true)).collect();
        let all = expected(&pairs);
        for i in materialized(&ui) {
            assert_eq!(styles(&ui, &list, i..i + 1)[0], all[i], "row {i}");
        }
        tick(&mut ui, 100000.0);
        tick(&mut ui, 0.0);
        let end = materialized(&ui);
        assert!(end.contains(&1000) && end.len() < 40, "{end:?}");
        for i in end {
            assert_eq!(styles(&ui, &list, i..i + 1)[0], all[i], "row {i}");
        }
    }

    /// `:has()` follows a descendant's class and `:first-of-type` the
    /// typed order; a flip in one of 200 rows restyles that row's own
    /// chain, not the list.
    #[test]
    fn has_and_of_type_follow_facts_with_bounded_restyle() {
        let mut ui = Ui::new(UiDesc::default());
        let body = r#"<column id="Root">{#each $rows as row (row.id)}<div id="Card" class="card"><text id="Label" class:hot={$extra == row.id}>{row.label}</text></div>{/each}<button id="Tail">t</button><text id="After">a</text></column>
<style>
  .card:has(> .hot) { width: 50px; }
  #Root:has(.hot) > text:first-of-type { height: 7px; }
  div:last-of-type { opacity: 0.5; }
</style>"#;
        ui.define_document(
            UiDocument::parse(
                "has.svelte",
                &format!("<meta name=\"component\" content=\"has\">{body}"),
            )
            .unwrap(),
        );
        let source = SourceId::new("test", "rows");
        let extra = SourceId::new("test", "extra");
        let keys: Vec<String> = (0..200).map(|i| format!("r{i}")).collect();
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        ui.docs
            .reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: rows(&keys),
            })
            .unwrap();
        ui.docs
            .reactive
            .execute(Command::Signal {
                source: extra.clone(),
                initial: json!(""),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "has"),
            "has",
            BTreeMap::from([("rows".into(), source), ("extra".into(), extra.clone())]),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        // The mount's ancestor-selector cascade settles over two ticks.
        tick(&mut ui, 0.0);
        tick(&mut ui, 0.0);
        tick(&mut ui, 0.0);
        assert_eq!(
            ui.docs.host.retained.resolved, 0,
            "an idle tick restyles nothing"
        );
        let root = root(&ui);
        let value = |ui: &Ui, i: usize, prop: &str| {
            let host = &ui.docs.host.retained;
            let id = host.topology.child_at(Some(&root), i).unwrap();
            let v = &host.elements[id].resolved.as_ref().unwrap().values;
            match prop {
                "width" => v.size("width").map(|s| format!("{s:?}")),
                "height" => v.size("height").map(|s| format!("{s:?}")),
                _ => v.num(prop).map(|n| n.to_string()),
            }
        };
        let px = |n: f32| Some(format!("{:?}", Size::px(n)));
        assert_eq!(value(&ui, 7, "width"), None);
        assert_eq!(value(&ui, 201, "height"), None);
        assert_eq!(value(&ui, 199, "opacity").as_deref(), Some("0.5"));
        assert_eq!(value(&ui, 198, "opacity"), None);
        ui.docs
            .reactive
            .execute(Command::Set {
                source: extra.clone(),
                value: json!("r7"),
                force: false,
            })
            .unwrap();
        tick(&mut ui, 0.0);
        let resolved = ui.docs.host.retained.resolved;
        assert_eq!(value(&ui, 7, "width"), px(50.0));
        assert_eq!(value(&ui, 8, "width"), None);
        // The first <text> child of Root is "After", past the <button>.
        assert_eq!(value(&ui, 201, "height"), px(7.0));
        // The label, its card and the root's chain, and what the root's
        // restyle reaches: not the 200 cards.
        assert!(resolved < 20, "restyled {resolved}");
    }

    /// Layers (with `!important` reversed), nesting, `@supports`, `@scope`,
    /// `revert-layer`, `inherit` and a non-inheriting `@property`, read off
    /// the resolved elements.
    #[test]
    fn layers_nesting_scope_supports_and_wide_keywords_cascade() {
        let mut ui = Ui::new(UiDesc::default());
        let text = r#"<meta name="component" content="cascade">
<column id="Root" class="theme">
  <div id="Card" class="card"><text id="Title" class="title">t</text></div>
  <div id="Other" class="card other"><text id="Note">n</text></div>
</column>
<style>
  @import "theme/base.css" layer(base) supports(display: grid) (width > 10px);
  @layer base, top;
  @layer top {
    .card { width: 30px; opacity: 0.8 !important; }
    .other { width: 50px; }
  }
  @layer base { .card { width: 10px; height: 4px; opacity: 0.9 !important; } }
  .card {
    &.other { height: 9px; }
    .title { opacity: 0.5; }
  }
  #Other { width: 70px; width: revert-layer; }
  #Note { width: inherit; }
  @scope (.other) { text { opacity: 0.25; } }
  @supports (display: grid) and (not (frobnicate: 1)) { #Root { opacity: 0.75; } }
  @supports (frobnicate: 1) { #Root { opacity: 0.1; } }
  @supports selector(:has(> .x)) { #Root { height: 3px; } }
  @property --gap { syntax: "<length>"; inherits: false; initial-value: 7px; }
  .theme { --gap: 3px; }
  #Title { height: var(--gap); }
</style>"#;
        let imports = vec![(
            "theme/base.css".to_string(),
            "#Root { width: 40px; } .theme { height: 99px; }".to_string(),
        )];
        assert_eq!(UiDocument::imports(text), ["theme/base.css"]);
        ui.define_document(
            UiDocument::parse_with_imports("cascade.svelte", text, imports).unwrap(),
        );
        let missing = UiDocument::parse("cascade.svelte", text)
            .unwrap_err()
            .join("\n");
        assert!(missing.contains("@import \"theme/base.css\""), "{missing}");
        ui.mount_document(
            SourceId::new("test", "cascade"),
            "cascade",
            BTreeMap::new(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        tick(&mut ui, 0.0);
        let host = &ui.docs.host.retained;
        let get = |id: &str| {
            let e = host
                .elements
                .values()
                .find(|e| e.document.nodes[e.index].id == id)
                .unwrap_or_else(|| {
                    panic!(
                        "{id}: {:?}",
                        host.elements.values().map(|e| &e.id).collect::<Vec<_>>()
                    )
                });
            e.resolved.as_ref().unwrap().values.clone()
        };
        let px = |n: f32| Some(Size::px(n));
        let (root, card, other) = (get("Root"), get("Card"), get("Other"));
        let (title, note) = (get("Title"), get("Note"));
        assert_eq!(card.size("width"), px(30.0), "the later layer wins");
        assert_eq!(
            card.num("opacity"),
            Some(0.9),
            "an important earlier layer wins"
        );
        assert_eq!(card.size("height"), px(4.0));
        assert_eq!(other.size("height"), px(9.0), "&.other nests");
        assert_eq!(
            other.size("width"),
            px(50.0),
            "revert-layer falls to the layers"
        );
        assert_eq!(note.size("width"), px(50.0), "inherit takes the parent's");
        assert_eq!(title.num("opacity"), Some(0.5), "a relative nested rule");
        assert_eq!(
            note.num("opacity"),
            Some(0.25),
            "@scope reaches its subtree"
        );
        assert_eq!(title.size("height"), px(7.0), "--gap does not inherit");
        assert_eq!(root.num("opacity"), Some(0.75));
        assert_eq!(
            root.size("height"),
            px(3.0),
            "unlayered beats the imported layer"
        );
        assert_eq!(root.size("width"), px(40.0), "the import's rule applies");
    }
}
