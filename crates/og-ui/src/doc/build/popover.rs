//! HTML's popovers and `title` tooltips in documents (docs/ui.md §2):
//! `popover` (auto, manual, hint) on an element makes it a layer shown over
//! the page against the control that opened it; `popovertarget` opens,
//! closes or toggles one on a press, and `interestfor` shows one while the
//! player hovers or keyboard-focuses the control (a rich tooltip), and
//! `contextmenu` opens one at the point of a secondary press or long press
//! on it (a context menu). `title` is the plain tooltip after its element.
//!
//! An open popover sits where CSS anchor positioning puts it against that
//! control, its implicit anchor (a context menu's is the zero-size point):
//! `position-area` picks the side and alignment, its insets offset it in
//! that area, `position-try-fallbacks` flips it where it would overflow the
//! safe area; with none of them it hangs under the control, above it where
//! there is more room. Its `::-og-arrow` draws on the side facing the
//! anchor.

pub(crate) use super::host::Mode;
use super::*;

/// A popover's key in its mount: ids are unique in a document.
pub(super) fn popover_key(mount: &str, id: &str) -> u64 {
    crate::id::key_of_path(&format!("{mount}#{id}~popover"))
}

pub(crate) trait PopoverUi {
    /// Whether popover `pk` is open (widget memory: kept in snapshots).
    fn popover_open(&mut self, pk: u64) -> bool;

    fn set_popover_open(&mut self, pk: u64, open: bool);

    /// After element `key` is built: its `title` tooltip, and what it does
    /// to the popovers it names.
    fn popover_invoker(&mut self, b: &Build<'_>, r: &Resolved, key: u64, resp: crate::ui::Response);

    /// Builds a `popover` element: nothing in its place but an empty box,
    /// and, open, a layer against the control that opened it. It is built
    /// every frame (its controls are other elements).
    #[allow(clippy::too_many_arguments)]
    fn build_popover(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String>;

    /// Popover layer `at`'s anchor positioning against its implicit anchor
    /// (the layout pass reads its `anchor` node, or `point`), and its arrow.
    fn place_popover(&mut self, r: &Resolved, at: usize, point: Option<[f32; 2]>);
}

impl PopoverUi for Ui {
    fn popover_open(&mut self, pk: u64) -> bool {
        self.has_anim(pk) && self.anim(pk).h != 0
    }

    fn set_popover_open(&mut self, pk: u64, open: bool) {
        if self.popover_open(pk) == open {
            return;
        }
        // HTML: showing an auto popover closes the other auto ones, and a
        // hint the other hints.
        if open && let Some(mode) = self.docs.host.popovers.get(&pk).map(|l| l.mode) {
            let others: Vec<u64> = self
                .docs
                .host
                .popovers
                .iter()
                .filter(|(k, l)| **k != pk && l.mode == mode && mode != Mode::Manual)
                .map(|(k, _)| *k)
                .collect();
            for k in others {
                if self.has_anim(k) {
                    self.set_anim(k, crate::ui::Anim::default());
                }
            }
        }
        self.set_anim(
            pk,
            crate::ui::Anim {
                h: u64::from(open),
                ..Default::default()
            },
        );
    }

    fn popover_invoker(
        &mut self,
        b: &Build<'_>,
        r: &Resolved,
        key: u64,
        resp: crate::ui::Response,
    ) {
        let at = self.tree.last_closed;
        if at == NONE || at >= self.tree.nodes.len() || self.tree.nodes[at].key != key {
            return;
        }
        if let Some(title) = r.str("title").filter(|t| !t.is_empty()) {
            if self.tree.nodes[at].a11y.description.is_none() {
                self.tree.nodes[at].a11y.description = Some(title.to_string());
            }
            self.tooltip(title);
        }
        if let Some(target) = r.str("popovertarget") {
            let pk = popover_key(&b.mount, target);
            let open = self.popover_open(pk);
            if resp.clicked {
                let next = match r.str("popovertargetaction") {
                    Some("show") => true,
                    Some("hide") => false,
                    _ => !open,
                };
                let link = self.docs.host.popovers.entry(pk).or_default();
                link.invoker = (at, key);
                link.point = None;
                self.set_popover_open(pk, next);
            }
            self.tree.nodes[at].a11y.expanded = Some(self.popover_open(pk));
        }
        // HTML's former `contextmenu` attribute: a secondary press or a long
        // press on it opens the popover at the pointer.
        if let Some(target) = r.str("contextmenu") {
            let context = resp.long_pressed
                || self.gesture_of(key).long_pressed
                || (self.interaction.input.secondary_pressed && self.interaction.state.hot == key);
            if context {
                let pk = popover_key(&b.mount, target);
                let p = self.interaction.input.pointer;
                let link = self.docs.host.popovers.entry(pk).or_default();
                link.invoker = (at, key);
                link.point = Some([p.x, p.y]);
                self.set_popover_open(pk, true);
            }
        }
        if let Some(target) = r.str("interestfor") {
            let pk = popover_key(&b.mount, target);
            let delay = (self.env.theme.durations_ms.slow * self.env.ticks_per_second as f32
                / 1000.0)
                .ceil() as u32;
            let s = &self.interaction.state;
            let interest =
                (s.hot == key && s.hover_ticks >= delay) || (s.focus == key && s.focus_visible);
            let link = self.docs.host.popovers.entry(pk).or_default();
            if interest {
                link.invoker = (at, key);
                link.interest = true;
            } else if link.invoker.1 == key {
                link.interest = false;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_popover(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        if let Some(owned) = self.tree.owning.last_mut() {
            owned.moved = true;
        }
        let mode = match r.str("popover") {
            Some("manual") => Mode::Manual,
            Some("hint") => Mode::Hint,
            _ => Mode::Auto,
        };
        let pk = popover_key(&b.mount, &n.id);
        let link = {
            let l = self.docs.host.popovers.entry(pk).or_default();
            l.mode = mode;
            *l
        };
        let (anchor, invoker) = link.invoker;
        let anchor = if anchor < self.tree.nodes.len() && self.tree.nodes[anchor].key == invoker {
            anchor
        } else {
            NONE
        };
        let mut open = self.popover_open(pk);
        // A hint shows while its control holds interest, or while the
        // pointer rests on the hint itself.
        let pointer = self.interaction.input.pointer;
        let over = link.layer != 0
            && self
                .last_bounds(link.layer)
                .is_some_and(|rect| rect.contains(pointer.x, pointer.y));
        if link.interest || (open && mode == Mode::Hint && over) {
            open = true;
        } else if mode == Mode::Hint {
            open = false;
        }
        if open != link.shown {
            self.docs.host.happened.push(When::Toggle);
        }
        // The element's place in the tree, as a modal dialog's, out of the
        // flow: HTML's popover is in the top layer, never in the page.
        self.leaf(
            "popover",
            crate::Id::from(id),
            Element {
                width: Size::Fixed(0.0),
                height: Size::Fixed(0.0),
                ..Element::default()
            },
        );
        let at = self.tree.last_closed;
        self.tree.props_mut(at).position = crate::element::Position::Absolute;
        self.docs.note_built(built);
        let t = self.env.theme;
        let content = Self::box_of(
            r,
            // A raised panel, as a browser's popover has its own canvas
            // and border (and the plain tooltip its bubble): its CSS
            // background, radius and padding override it.
            Element {
                direction: Direction::Column,
                padding: Insets::xy(t.space.sm, t.space.xs),
                background: Some(t.palette.panel_raised),
                radius: if mode == Mode::Hint {
                    t.radius.control
                } else {
                    t.radius.panel
                },
                frame: Some(if mode == Mode::Hint {
                    crate::style_file::StylePart::Tooltip
                } else {
                    crate::style_file::StylePart::Panel
                }),
                ..Element::default()
            },
            self.style.em_of(r, &self.env),
        );
        let local = format!("{}~popover", path.replace('/', "_"));
        let mut layer = 0;
        let mut layer_at = NONE;
        if mode == Mode::Auto {
            let out = self.floating(&local, open, anchor, content, |ui| {
                layer_at = ui.tree.parent();
                layer = ui.tree.nodes[layer_at].key;
                build_children(ui, b, None);
            });
            if out.dismissed {
                self.set_popover_open(pk, false);
            }
        } else if open {
            let float = if mode == Mode::Hint {
                crate::layout::Float::Tooltip
            } else {
                crate::layout::Float::Popover
            };
            let i = self.open_layer(
                "popover",
                crate::Id::from(local.as_str()),
                content,
                float,
                anchor,
            );
            layer = self.tree.nodes[i].key;
            layer_at = i;
            build_children(self, b, None);
            self.close();
        }
        if layer_at != NONE {
            self.place_popover(r, layer_at, link.point);
        }
        let l = self.docs.host.popovers.entry(pk).or_default();
        if layer != 0 {
            l.layer = layer;
        }
        l.shown = open && layer != 0;
        let resp = crate::ui::Response::default();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    fn place_popover(&mut self, r: &Resolved, at: usize, point: Option<[f32; 2]>) {
        let (spec, authored) = Self::anchor_spec(r, &mut Props::default());
        let build = self.tree.build();
        let n = &mut self.tree.nodes[at];
        if authored {
            n.props.write(build).anchor = Some(Box::new(spec));
        }
        n.anchor_point = point;
        n.arrow = r
            .pseudo
            .iter()
            .find(|(w, ..)| *w == "arrow")
            .map(|(_, _, p)| {
                let size = match p.size("width") {
                    Some(Size::Fixed(v)) => v,
                    _ => 10.0,
                };
                // Its own background, else the popover's, as a CSS arrow
                // usually is.
                let color = p
                    .color("background")
                    .or(n.el.background)
                    .unwrap_or(self.env.theme.palette.panel_raised);
                (size, color)
            });
    }
}
