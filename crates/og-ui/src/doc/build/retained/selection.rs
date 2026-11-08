//! Sibling and structural selector invalidation. Topology edits and selector
//! fact changes become sibling intervals whose styles are marked stale in
//! O(log n) each; materialized rows restyle when emitted and deferred rows
//! when they materialize, so a front insert into a virtual list restyles only
//! the visible and overscan rows.
use super::*;
use crate::doc::matching::Profile;
use crate::doc::topology::{FactChange, SiblingChange};
use std::ops::Range;

impl RetainedHost {
    fn profile(&mut self, ui: &Ui, document: &str) -> &Profile {
        self.profiles.entry(document.to_owned()).or_insert_with(|| {
            ui.docs
                .docs
                .get(document)
                .map(|doc| Profile::of_document(doc))
                .unwrap_or_default()
        })
    }

    /// The structural reads of the documents that can style `nodes`.
    fn profile_of<'a>(
        &mut self,
        ui: &Ui,
        nodes: impl IntoIterator<Item = &'a Arc<UiDocument>>,
    ) -> Profile {
        let mut out = Profile::default();
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

    /// Runs after bindings and selector facts settle and before dirty styles
    /// drain.
    pub(super) fn consume_selector_changes(&mut self, ui: &mut Ui) -> Result<(), String> {
        for filter in &ui.docs.structure.filters {
            self.topology.register_filter(filter);
        }
        let changes = self.topology.take_changes();
        let facts = self.topology.take_fact_changes();
        if !ui.docs.structure.any {
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
            self.stale(Some(node), 0..usize::MAX)?;
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
        let host = &ui.docs.retained;
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
        ui.reactive
            .execute(Command::Collection {
                source: source.clone(),
                key: "id".into(),
                items: rows(keys),
            })
            .unwrap();
        ui.reactive
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
            ui.reactive
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
        ui.reactive
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
            ui.reactive
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
        assert_eq!(ui.docs.retained.resolved, 0);
        ui.reactive
            .execute(Command::Move {
                source,
                key: "r500".into(),
                before: Some("r502".into()),
            })
            .unwrap();
        order.swap(500, 501);
        tick(&mut ui, 0.0);
        let work = (ui.docs.retained.resolved, ui.docs.retained.css);
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
        ui.reactive
            .execute(Command::Insert {
                source: source.clone(),
                row,
                before: Some("r0".into()),
            })
            .unwrap();
        order.insert(0, "front".into());
        tick(&mut ui, 0.0);
        // Every rank changed, but only materialized rows restyle.
        let resolved = ui.docs.retained.resolved;
        let now = materialized(&ui).len();
        assert!(now < shown + 4, "{now} rows materialized");
        assert_eq!(
            resolved, now,
            "restyled {resolved} of {now} materialized rows"
        );
        let css = ui.docs.retained.css;
        // The insert plus the inserted row's facts; each costs a few intervals.
        assert!(css.changes <= 8 && css.intervals <= 2 * 8, "{css:?}");
        assert!(css.selectors <= now * 5, "{css:?}");
        let far = ui
            .mounted_topology()
            .child_at(Some(&list), 1000)
            .unwrap()
            .clone();
        assert_eq!(ui.docs.retained.topology.style_is_stale(&far), Some(true));
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
}
