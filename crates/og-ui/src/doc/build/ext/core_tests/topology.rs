use crate::UiDocument;
use crate::bind::InteractionState;
use crate::doc::build::ext::*;
use crate::doc::topology::*;
use crate::reactive::SourceId;
use crate::reactive::graph::MountedNode;
use serde_json::json;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

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
