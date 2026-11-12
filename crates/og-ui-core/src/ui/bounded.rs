//! The bounded frame's check (debug builds): with `OG_UI_CHECK_BOUNDED=1`
//! every frame is also finished whole on a copy of the UI, and the two
//! must agree on everything the frame publishes and every node's place.

use super::*;

/// Whether `OG_UI_CHECK_BOUNDED` asks for the check.
pub(crate) fn checking() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("OG_UI_CHECK_BOUNDED").is_some_and(|v| v != "0"))
}

/// Panics where the bounded frame `ui` differs from the whole frame `twin`.
pub(super) fn compare(ui: &Ui, twin: &Ui) {
    let at = |what: &str| format!("bounded frame differs from the whole frame: {what}");
    let (a, b) = (&ui.out.frame, &twin.out.frame);
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
        format!("{:?}", ui.interaction.state.hits),
        format!("{:?}", twin.interaction.state.hits),
        "{}",
        at("hits")
    );
    // Motion state: the same channels with the same values. The whole
    // frame steps every animated node, the bounded one leaves a settled
    // one unvisited, so when a channel was last stepped is not compared.
    let (mu, mt) = (&ui.interaction.state.motion, &twin.interaction.state.motion);
    for (k, c) in &mu.channels {
        let mut c = *c;
        let mut w = *mt
            .channels
            .get(k)
            .unwrap_or_else(|| panic!("{}: channel {k} kept: {c:?}", at("motion state")));
        (c.stepped, w.stepped) = (0, 0);
        assert_eq!(c, w, "{}: channel {k}", at("motion state"));
    }
    assert_eq!(
        mu.channels.len(),
        mt.channels.len(),
        "{}: a channel forgotten",
        at("motion state")
    );
    let sorted = |m: &crate::motion::state::MotionState| {
        let mut groups: Vec<_> = m
            .groups
            .iter()
            .map(|(k, g)| (*k, format!("{g:?}")))
            .collect();
        groups.sort_unstable();
        let mut set: Vec<_> = m.set_states.iter().map(|(k, s)| (*k, s.clone())).collect();
        set.sort_unstable();
        let mut seen: Vec<_> = m.seen.iter().map(|(k, t)| (*k, *t)).collect();
        seen.sort_unstable();
        let mut flip: Vec<_> = m.flip.iter().map(|(k, r)| (*k, format!("{r:?}"))).collect();
        flip.sort_unstable();
        format!(
            "{groups:?}\n{set:?}\n{seen:?}\n{flip:?}\n{:?}\n{:?}",
            m.css, m.styles
        )
    };
    assert_eq!(sorted(mu), sorted(mt), "{}", at("motion state"));
    let mut order = preorder(&twin.tree.nodes, 0);
    for &f in &twin.tree.floating {
        order.extend(preorder(&twin.tree.nodes, f));
    }
    for i in order {
        let (n, m) = (&ui.tree.nodes[i], &twin.tree.nodes[i]);
        let place = |n: &Node| {
            format!(
                "{:?}",
                (
                    (
                        n.placed.rect,
                        n.placed.clip,
                        n.placed.world,
                        n.placed.screen,
                        n.placed.alpha
                    ),
                    (
                        n.layout.x,
                        n.layout.y,
                        n.layout.w,
                        n.layout.h,
                        &n.layout.lines,
                        n.layout.content_h,
                        n.layout.content_w
                    ),
                    (&n.children, &n.out),
                )
            )
        };
        assert_eq!(place(n), place(m), "{}", at(&ui.tree.path_of(i)));
    }
}

/// Panics where the trees' index (`tree`) differs from the frame's order.
pub(super) fn compare_tree(ui: &Ui, order: &[usize]) {
    let mut keys: Vec<(u64, usize)> = Vec::new();
    for &i in order {
        keys.push((ui.tree.nodes[i].key, i));
        for l in &ui.tree.nodes[i].layout.links {
            keys.push((child_key(ui.tree.nodes[i].key, l), i));
        }
    }
    keys.sort_unstable();
    keys.dedup();
    let mut held: Vec<(u64, usize)> = ui
        .tree
        .index
        .by_key
        .iter()
        .chain(&ui.tree.index.links)
        .map(|(&k, &i)| (k, i))
        .collect();
    held.sort_unstable();
    held.dedup();
    assert_eq!(
        keys, held,
        "the trees' index differs from the frame's order"
    );
    for i in 0..ui.tree.nodes.len() {
        assert_eq!(
            ui.tree.index.has(i),
            order.contains(&i),
            "the trees' index differs at '{}'",
            ui.tree.path_of(i)
        );
    }
}
