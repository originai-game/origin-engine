//! Presentation checkpoints, screen covers, virtual cursor wiring, and
//! world-space anchor projection.

use super::*;

/// Covers the frame this tick with a timeline's screen transition
/// (presentation only: the tree draws it at the tick's end, over
/// everything).
pub(crate) fn cover(engine: u32, cover: og_ui_next::cover::Cover) {
    let Ok(eng) = crate::engine::engine(engine) else {
        return;
    };
    // A game with no UI mounted has no new core yet: the cover makes it.
    eng.render
        .borrow_mut()
        .ui_next
        .get_or_insert_with(|| Box::new(Next::new()))
        .cover = Some(cover);
}

/// og.ui.setVirtualCursor this tick: the cursor the UI's tick end moves.
pub(crate) fn set_virtual_cursor(engine: u32, desc: Option<og_ui_next::tree::VirtualCursorDesc>) {
    let Ok(eng) = crate::engine::engine(engine) else {
        return;
    };
    eng.render
        .borrow_mut()
        .ui_next
        .get_or_insert_with(|| Box::new(Next::new()))
        .vcursor = desc;
}

/// The virtual cursor as the last tick left it: where (CSS px) and
/// whether it shows; `None` when none is declared.
pub(crate) fn virtual_cursor(engine: u32) -> Option<([f32; 2], bool)> {
    let eng = crate::engine::engine(engine).ok()?;
    let rs = eng.render.borrow();
    let next = rs.ui_next.as_ref()?;
    // The tree's units, in the surface's px.
    let (p, shown) = next.tree.as_ref()?.virtual_cursor_at()?;
    Some((p.map(|v| v * next.unit), shown))
}

/// The cursor the TSX element under the pointer names (`cursor-*`), as
/// the last tick left it; `None` without a tree.
pub(crate) fn cursor(engine: u32) -> Option<og_ui_next::Cursor> {
    let eng = crate::engine::engine(engine).ok()?;
    let rs = eng.render.borrow();
    Some(rs.ui_next.as_ref()?.tree.as_ref()?.cursor())
}

impl Next {
    /// Snapshot of the UI tree's deterministic interactive state, when built.
    pub(crate) fn tree_checkpoint(&self) -> Option<og_ui_next::tree::TreeCheckpoint> {
        self.tree.as_ref().map(Tree::checkpoint)
    }

    /// Current deterministic PRNG state for `og.ui.feedback` cues.
    pub(crate) fn feedback_rng(&self) -> u64 {
        self.feedback_rng
    }

    /// Restores the UI tree's interactive state and feedback PRNG from a checkpoint.
    pub(crate) fn restore_checkpoint(
        &mut self,
        tree: Option<&og_ui_next::tree::TreeCheckpoint>,
        feedback_rng: Option<u64>,
    ) {
        if let Some(rng) = feedback_rng {
            self.feedback_rng = rng;
        }
        if let (Some(cp), Some(t)) = (tree, self.tree.as_mut()) {
            t.restore(cp);
        }
    }

    /// On a frame drawn between ticks: the anchored nodes the tick showed
    /// move to where `project` (the drawn camera, the targets' drawn
    /// places) puts them, and the tree repaints when one moved
    /// (CONVENTIONS.md §World). The next tick's end places them again.
    pub(crate) fn follow_anchors(
        &mut self,
        (faces, atlas, icons): (
            &mut crate::present::faces::Faces,
            &mut og_render::glyphs::Atlas,
            &og_ui_next::icons::IconAtlas,
        ),
        project: impl FnMut(
            og_ui_next::tree::AnchorTarget,
            [f32; 3],
            bool,
        ) -> Option<og_ui_next::tree::Sighting>,
    ) {
        let Some(tree) = self.tree.as_mut() else {
            return;
        };
        if tree.anchor_targets().next().is_none() {
            return;
        }
        tree.follow_anchors(project);
        let (_, painted) = tree.present(&mut faces.collection, atlas, icons);
        if painted {
            self.generation += 1;
            std::mem::swap(&mut self.paint, &mut tree.drawn);
            self.paint.generation = self.generation;
        }
    }
}

/// What `view` sees of world point `p` on a `w`×`h` CSS px surface, for an
/// anchored TSX node (og-ui-next CONVENTIONS.md §World): its surface point
/// and depth, or behind the eye a point well off the surface the way the
/// camera would turn to it, where an edge-clamped node pins.
pub(crate) fn sighting(
    view: &og_ui_next::view::WorldView,
    p: og_core::glam::Vec3,
    w: f32,
    h: f32,
) -> og_ui_next::tree::Sighting {
    match view.project(p, w, h) {
        Some((x, y, depth)) => og_ui_next::tree::Sighting {
            at: [x, y],
            depth,
            ..Default::default()
        },
        None => {
            let d = view.direction(p, w, h) * (w + h);
            og_ui_next::tree::Sighting {
                at: [w / 2.0 + d.x, h / 2.0 + d.y],
                behind: true,
                ..Default::default()
            }
        }
    }
}
