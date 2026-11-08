//! The bounded frame's check (debug builds): with `OG_UI_CHECK_BOUNDED=1`
//! every frame is also finished whole on a copy of the UI, and the two
//! must agree on everything the frame publishes and every node's place.

use super::*;

/// Whether `OG_UI_CHECK_BOUNDED` asks for the check.
pub(super) fn checking() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("OG_UI_CHECK_BOUNDED").is_some_and(|v| v != "0"))
}

/// Panics where the bounded frame `ui` differs from the whole frame `twin`.
pub(super) fn compare(ui: &Ui, twin: &Ui) {
    let at = |what: &str| format!("bounded frame differs from the whole frame: {what}");
    let (a, b) = (&ui.frame, &twin.frame);
    if a.items != b.items {
        let k = a.items.iter().zip(&b.items).position(|(x, y)| x != y);
        panic!(
            "{}: {} vs {} items, first difference at {k:?}:\n bounded {:?}\n whole   {:?}",
            at("draw items"),
            a.items.len(),
            b.items.len(),
            k.and_then(|k| a.items.get(k)),
            k.and_then(|k| b.items.get(k))
                .or(b.items.get(a.items.len())),
        );
    }
    assert_eq!(a.warnings, b.warnings, "{}", at("warnings"));
    assert_eq!(
        a.missing_glyphs,
        b.missing_glyphs,
        "{}",
        at("missing glyphs")
    );
    assert_eq!(a.focus_paths, b.focus_paths, "{}", at("focus"));
    assert_eq!(
        format!("{:?}", ui.state.hits),
        format!("{:?}", twin.state.hits),
        "{}",
        at("hits")
    );
    let mut order = preorder(&twin.nodes, 0);
    for &f in &twin.floating {
        order.extend(preorder(&twin.nodes, f));
    }
    for i in order {
        let (n, m) = (&ui.nodes[i], &twin.nodes[i]);
        let place = |n: &Node| {
            format!(
                "{:?}",
                (
                    (n.rect, n.clip, n.world, n.screen, n.alpha),
                    (n.x, n.y, n.w, n.h, &n.lines, n.content_h, n.content_w),
                    (&n.children, &n.out),
                )
            )
        };
        assert_eq!(place(n), place(m), "{}", at(&ui.path_of(i)));
    }
}

/// Panics where the trees' index (`tree`) differs from the frame's order.
pub(super) fn compare_tree(ui: &Ui, order: &[usize]) {
    let mut keys: Vec<(u64, usize)> = Vec::new();
    for &i in order {
        keys.push((ui.nodes[i].key, i));
        for l in &ui.nodes[i].links {
            keys.push((child_key(ui.nodes[i].key, l), i));
        }
    }
    keys.sort_unstable();
    keys.dedup();
    let mut held: Vec<(u64, usize)> = ui
        .tree
        .by_key
        .iter()
        .chain(&ui.tree.links)
        .map(|(&k, &i)| (k, i))
        .collect();
    held.sort_unstable();
    held.dedup();
    assert_eq!(
        keys, held,
        "the trees' index differs from the frame's order"
    );
    for i in 0..ui.nodes.len() {
        assert_eq!(
            ui.tree.has(i),
            order.contains(&i),
            "the trees' index differs at '{}'",
            ui.path_of(i)
        );
    }
}
