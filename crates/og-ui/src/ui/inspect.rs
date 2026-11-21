//! What the last frame shows, for inspection: the probe's element list,
//! overlaps, dense and sideways
//! regions and motion in flight. None of it is made while a frame is
//! drawn; each is made from the finished frame's nodes the first time it
//! is asked for, and kept until the next `end`.
use super::*;
use std::cell::OnceCell;

/// The last frame's inspection, made on demand ([`Ui::elements`],
/// [`Ui::overlaps`], …).
#[derive(Clone, Debug, Default)]
pub(crate) struct Inspection {
    elements: OnceCell<(Vec<UiElement>, Vec<Option<crate::dress::PartRef>>)>,
    overlaps: OnceCell<Vec<Overlap>>,
    dense: OnceCell<Vec<String>>,
    sideways: OnceCell<Vec<String>>,
    motion: OnceCell<Vec<crate::motion::run::MotionDump>>,
}

impl Inspection {
    /// Nothing to inspect: a frame whose layout was abandoned is not
    /// published, and the nodes it left are not the last frame's.
    pub(crate) fn none() -> Self {
        let out = Self::default();
        let _ = out.elements.set(Default::default());
        let _ = out.overlaps.set(Vec::new());
        let _ = out.dense.set(Vec::new());
        let _ = out.sideways.set(Vec::new());
        let _ = out.motion.set(Vec::new());
        out
    }
}

impl Ui {
    /// Where element `key` was on screen in the last completed frame (its
    /// box cut to its clip), even while this frame is being built; `None`
    /// when it was not drawn.
    pub fn last_bounds(&self, key: u64) -> Option<Rect> {
        (0..self.tree.nodes.len())
            .filter_map(|i| self.tree.previous_node(i).map(|n| (i, n)))
            .find(|(i, n)| {
                n.key == key
                    && !n.is_hidden()
                    && self
                        .tree
                        .prev_kinds
                        .get(i)
                        .unwrap_or(&self.tree.kinds[*i])
                        .0
                        != "vacant"
            })
            .and_then(|(_, n)| {
                let (r, c) = (&n.placed.screen, &n.placed.clip);
                let outside =
                    r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
                let shown = r.intersect(c);
                (!outside && !(shown.is_empty() && !r.is_empty())).then_some(shown)
            })
    }

    /// Where the element at `path` (a full path, as the probe names it) was
    /// on screen in the last completed frame, cut to its clip: one
    /// element's box, without making the frame's element list.
    pub fn element_bounds(&self, path: &str) -> Option<Rect> {
        self.last_bounds(crate::id::key_of_path(path))
    }

    /// The last completed frame as a UI to inspect: this one once it has
    /// ended, or while a frame is being built, a copy with last frame's
    /// nodes put back (made only when something is asked mid-frame).
    pub(super) fn finished(&self) -> std::borrow::Cow<'_, Ui> {
        if !self.tree.building {
            return std::borrow::Cow::Borrowed(self);
        }
        let mut ui = self.clone();
        for (&i, n) in &self.tree.replaced {
            ui.tree.nodes[i] = n.clone();
        }
        for (&i, &k) in &self.tree.prev_kinds {
            ui.tree.kinds[i] = k;
        }
        ui.tree.floating = self.tree.prev_floating.clone();
        ui.tree.draws = self.paint.prev_draws.clone();
        ui.tree.building = false;
        ui.paint.inspection = Default::default();
        std::borrow::Cow::Owned(ui)
    }

    /// Every visible element of the last frame in draw order, with its
    /// (clipped) bounds and text.
    pub fn elements(&self) -> &[UiElement] {
        &self.inspected_elements().0
    }

    /// The element that emitted a text command, including an element
    /// scrolled completely out of the visible element list: its probe
    /// path and whether an ancestor clips its box. A leaving element
    /// whose node no longer exists has no current source.
    pub fn draw_source(&self, key: u64) -> Option<(String, bool)> {
        let ui = self.finished();
        let i = ui.tree.index.find(key)?;
        let n = &ui.tree.nodes[i];
        let r = &n.placed.screen;
        let visible = r.intersect(&n.placed.clip);
        Some((
            ui.tree.probe_path(i),
            visible.width + 0.01 < r.width || visible.height + 0.01 < r.height,
        ))
    }

    /// The style part each of [`Ui::elements`] is drawn as
    /// (`Button.base[primary]`, its variant and state flags as of the last
    /// frame), index for index; `None` for one no style dresses.
    pub fn parts(&self) -> &[Option<crate::dress::PartRef>] {
        &self.inspected_elements().1
    }

    /// The last frame's element at `path` (a full path or an id).
    pub fn element(&self, path: &str) -> Option<&UiElement> {
        find_by_id(self.elements(), path)
    }

    /// The last frame's element showing a text (`[=]text[@N]`), as
    /// `move_to_text` resolves it.
    pub fn element_by_text(&self, query: &str) -> Option<&UiElement> {
        find_by_text(self.elements(), query)
    }

    /// Overlaps between elements (docs/ui.md §2), other than those the
    /// engine places on purpose, for `og ui check`.
    pub fn overlaps(&self) -> &[Overlap] {
        self.paint.inspection.overlaps.get_or_init(|| {
            let ui = self.finished();
            ui.layout
                .find_overlaps(&ui.inspected_preorder(), &ui.env, &ui.paint, &ui.tree)
        })
    }

    /// The paths of the elements whose props say dense (a dense HUD's
    /// readouts, docs/ui.md §6.1), each covering its subtree.
    pub fn dense(&self) -> &[String] {
        self.paint.inspection.dense.get_or_init(|| {
            let ui = self.finished();
            ui.inspected_preorder()
                .into_iter()
                .filter(|&i| ui.tree.nodes[i].props.dense)
                .map(|i| ui.tree.path_of(i))
                .collect()
        })
    }

    /// The paths of the scroll containers that scroll sideways and of the
    /// tracks dragged sideways (a carousel's): a line their edge cuts
    /// across scrolls into view.
    pub fn sideways(&self) -> &[String] {
        self.paint.inspection.sideways.get_or_init(|| {
            let ui = self.finished();
            ui.inspected_preorder()
                .into_iter()
                .filter(|&i| {
                    let n = &ui.tree.nodes[i];
                    n.el.scroll_x
                        || n.props
                            .drag
                            .as_ref()
                            .is_some_and(|d| d.axis == crate::DragAxis::X)
                })
                .map(|i| ui.tree.path_of(i))
                .collect()
        })
    }

    /// Elements in visual states and the animations in flight, for `dump_ui`.
    pub fn motion_dump(&self) -> &[crate::motion::run::MotionDump] {
        self.paint.inspection.motion.get_or_init(|| {
            let ui = self.finished();
            ui.motion_dump_of(&ui.inspected_draw_order())
        })
    }

    /// The finished frame's trees in preorder (the base, then its layers).
    fn inspected_preorder(&self) -> Vec<usize> {
        if self.tree.nodes.is_empty() {
            return Vec::new();
        }
        let mut order = preorder(&self.tree.nodes, 0);
        for &f in &self.tree.floating {
            order.extend(preorder(&self.tree.nodes, f));
        }
        order
    }

    /// The finished frame's trees in drawing order.
    fn inspected_draw_order(&self) -> Vec<usize> {
        if self.tree.nodes.is_empty() {
            return Vec::new();
        }
        self.tree.drawing_order(&self.world, &self.layout)
    }

    /// The controls keys and pads reach on the last frame's screen, in
    /// the order navigation walks them.
    fn reach(&self) -> Vec<u64> {
        self.interaction
            .focusables()
            .iter()
            .map(|h| h.key)
            .collect()
    }

    fn mark(&self, reach: &[u64], key: u64, states: &mut Vec<String>) {
        if reach.contains(&key) {
            states.push("focusable".to_owned());
        }
        if key != 0 && key == self.interaction.state.focus {
            states.push("focused".to_owned());
        }
    }

    /// Scrolled or clipped out of view: not on screen. A zero-size element
    /// (an empty spacer) is on screen where it sits.
    fn offscreen(&self, i: usize) -> bool {
        let n = &self.tree.nodes[i];
        let (r, c) = (&n.placed.screen, &n.placed.clip);
        let outside = r.x > c.right() || r.right() < c.x || r.y > c.bottom() || r.bottom() < c.y;
        outside || (r.intersect(c).is_empty() && !r.is_empty())
    }

    fn inspected_elements(&self) -> &(Vec<UiElement>, Vec<Option<crate::dress::PartRef>>) {
        self.paint
            .inspection
            .elements
            .get_or_init(|| self.finished().element_list())
    }

    fn element_list(&self) -> (Vec<UiElement>, Vec<Option<crate::dress::PartRef>>) {
        {
            let order = self.inspected_draw_order();
            let reach = self.reach();
            let mut elements = Vec::new();
            let mut parts = Vec::new();
            for &i in order.iter().skip(1) {
                if self.offscreen(i) {
                    continue;
                }
                let n = &self.tree.nodes[i];
                let (role, name, _, value, mut states) = self.a11y_info(i);
                if !role.is_empty() {
                    self.mark(&reach, n.key, &mut states);
                }
                let r = &n.placed.screen;
                let visible = n.placed.screen.intersect(&n.placed.clip);
                let parent = if n.parent == NONE {
                    self.tree
                        .nodes
                        .get(n.anchor)
                        .map_or(String::new(), |a| a.id.clone())
                } else if n.parent == 0 {
                    String::new()
                } else {
                    self.tree.nodes[n.parent].id.clone()
                };
                parts.push(
                    n.part_box
                        .as_ref()
                        .map(|p| p.reference)
                        .or(n.part())
                        .or_else(|| n.el.frame.map(StylePart::part_ref)),
                );
                elements.push(UiElement {
                    id: n.id.clone(),
                    parent,
                    path: self.tree.probe_path(i),
                    kind: self.tree.kinds.get(i).map_or("", |k| k.0).to_string(),
                    disabled: n.disabled,
                    clipped: visible.width + 0.01 < r.width || visible.height + 0.01 < r.height,
                    // As read: bidi marks and isolates steer the drawing only.
                    text: n
                        .probe_text
                        .chars()
                        .filter(|&c| !og_i18n::is_bidi_control(c))
                        .collect(),
                    role: role.to_owned(),
                    name,
                    value,
                    states,
                    // Its fill, solid and at full opacity, covers its box (a
                    // rounded corner's few pixels aside).
                    opaque: n
                        .style
                        .sampled
                        .background
                        .is_some_and(|bg| bg.alpha >= 0.999)
                        && n.placed.alpha >= 0.999,
                    x: visible.x,
                    y: visible.y,
                    width: visible.width,
                    height: visible.height,
                    detail: n
                        .text_overflow()
                        .or_else(|| {
                            self.tree
                                .nodes
                                .get(n.content_slot)
                                .and_then(Node::text_overflow)
                        })
                        .map(|p| {
                            [
                                (
                                    "textOverflow".into(),
                                    serde_json::json!(match p {
                                        crate::layout::TextOverflow::Clip => "clip",
                                        crate::layout::TextOverflow::Ellipsis => "ellipsis",
                                        crate::layout::TextOverflow::Scroll { .. } => "-og-scroll",
                                    }),
                                ),
                                (
                                    "textScrollActive".into(),
                                    serde_json::json!(
                                        n.scrolling_text().is_some()
                                            || self
                                                .tree
                                                .nodes
                                                .get(n.content_slot)
                                                .is_some_and(|n| n.scrolling_text().is_some())
                                    ),
                                ),
                            ]
                            .into_iter()
                            .collect()
                        })
                        .unwrap_or_default(),
                });
                // A Rive frame's drawn objects: children for dump_ui, describe
                // and pick, their source the artboard's id and the object.
                if let Some(crate::paint::RiveDraw(player)) =
                    self.tree.draws.get(&i).and_then(|d| d.rive.as_ref())
                {
                    let board = self
                        .interaction
                        .state
                        .rive
                        .get(&n.key)
                        .map_or("", |s| s.board.as_str());
                    let file = board.split_once('#').map_or(board, |(f, _)| f);
                    let at = n.placed.world.map_rect(&n.placed.rect);
                    for e in player.elements([at.x, at.y, at.width, at.height]) {
                        let b = Rect::new(e.bounds[0], e.bounds[1], e.bounds[2], e.bounds[3]);
                        let shown = b.intersect(&n.placed.clip);
                        parts.push(None);
                        let mut detail = serde_json::Map::new();
                        detail.insert(
                            "source".into(),
                            serde_json::json!(format!("{file}#{}", e.id)),
                        );
                        detail.insert(
                            "rive".into(),
                            serde_json::json!({ "node": e.node, "opacity": e.opacity }),
                        );
                        elements.push(UiElement {
                            path: format!("{}/{}", self.tree.probe_path(i), e.id),
                            id: e.id,
                            parent: n.id.clone(),
                            kind: "rive".into(),
                            disabled: false,
                            clipped: shown.width + 0.01 < b.width || shown.height + 0.01 < b.height,
                            text: e.text.clone(),
                            role: String::new(),
                            name: String::new(),
                            value: String::new(),
                            states: Vec::new(),
                            opaque: false,
                            x: shown.x,
                            y: shown.y,
                            width: shown.width,
                            height: shown.height,
                            detail,
                        });
                    }
                }
                // A rich text's links: children for the probe.
                if !n.layout.links.is_empty() {
                    let Content::Rich(rc) = &*n.content else {
                        unreachable!("links come from rich text")
                    };
                    for (l, lr) in self.rich_links(i) {
                        let lr = n.placed.world.map_rect(&lr);
                        let shown = lr.intersect(&n.placed.clip);
                        if shown.is_empty() {
                            continue;
                        }
                        let text: String = rc
                            .spans
                            .iter()
                            .filter(|s| s.link.as_deref() == Some(l.as_str()))
                            .map(|s| s.text.as_str())
                            .collect();
                        parts.push(None);
                        elements.push(UiElement {
                            path: format!("{}/{l}", self.tree.probe_path(i)),
                            id: l,
                            parent: n.id.clone(),
                            kind: "link".into(),
                            disabled: false,
                            clipped: shown.width + 0.01 < lr.width
                                || shown.height + 0.01 < lr.height,
                            name: text.clone(),
                            text,
                            role: "link".into(),
                            value: String::new(),
                            states: Vec::new(),
                            opaque: false,
                            x: shown.x,
                            y: shown.y,
                            width: shown.width,
                            height: shown.height,
                            detail: Default::default(),
                        });
                    }
                }
            }
            (elements, parts)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::ButtonDesc;
    use crate::{InputFrame, Ui, Viewport};

    /// Asked while a frame is built, the inspection answers for the last
    /// completed frame, not for the half-built one.
    #[test]
    fn inspection_mid_frame_reads_the_last_completed_frame() {
        let mut ui = Ui::default();
        let frame = |ui: &mut Ui, label: &str| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.button("Go", label, ButtonDesc::default());
        };
        frame(&mut ui, "First");
        ui.end();
        let done = ui.elements().to_vec();
        frame(&mut ui, "Second");
        assert_eq!(ui.elements().to_vec(), done);
        ui.end();
        let label = ui.element("Go").unwrap().name.clone();
        assert_eq!(label, "Second");
    }
}
