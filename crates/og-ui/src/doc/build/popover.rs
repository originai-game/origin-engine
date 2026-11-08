//! HTML's popovers and `title` tooltips in documents (docs/ui.md §2):
//! `popover` (auto, manual, hint) on an element makes it a layer shown over
//! the page against the control that opened it; `popovertarget` opens,
//! closes or toggles one on a press, and `interestfor` shows one while the
//! player hovers or keyboard-focuses the control (a rich tooltip). `title`
//! is the plain tooltip after its element.

use super::*;

/// What the controls naming a popover last did to it, by its key
/// ([`popover_key`]): presentation, rebuilt from the controls as they build.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PopoverLink {
    /// The control it is placed against (its node index and key).
    pub invoker: (usize, u64),
    /// An `interestfor` control holds the player's interest.
    pub interest: bool,
    /// Its layer's key, as last built (hovering it keeps a hint shown).
    pub layer: u64,
    /// It showed when last built (`ontoggle` fires as that changes).
    pub shown: bool,
    /// `auto`, `manual` or `hint`, as last built.
    pub mode: Mode,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Auto,
    Manual,
    Hint,
}

/// A popover's key in its mount: ids are unique in a document.
pub(super) fn popover_key(mount: &str, id: &str) -> u64 {
    crate::id::key_of_path(&format!("{mount}#{id}~popover"))
}

impl Ui {
    /// Whether popover `pk` is open (widget memory: kept in snapshots).
    fn popover_open(&mut self, pk: u64) -> bool {
        self.has_anim(pk) && self.anim(pk).h != 0
    }

    fn set_popover_open(&mut self, pk: u64, open: bool) {
        if self.popover_open(pk) == open {
            return;
        }
        // HTML: showing an auto popover closes the other auto ones, and a
        // hint the other hints.
        if open && let Some(mode) = self.docs.popovers.get(&pk).map(|l| l.mode) {
            let others: Vec<u64> = self
                .docs
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

    /// After element `key` is built: its `title` tooltip, and what it does
    /// to the popovers it names.
    pub(super) fn popover_invoker(
        &mut self,
        b: &Build<'_>,
        r: &Resolved,
        key: u64,
        resp: crate::ui::Response,
    ) {
        let at = self.last_closed;
        if at == NONE || at >= self.nodes.len() || self.nodes[at].key != key {
            return;
        }
        if let Some(title) = r.str("title").filter(|t| !t.is_empty()) {
            if self.nodes[at].a11y.description.is_none() {
                self.nodes[at].a11y.description = Some(title.to_string());
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
                self.docs.popovers.entry(pk).or_default().invoker = (at, key);
                self.set_popover_open(pk, next);
            }
            self.nodes[at].a11y.expanded = Some(self.popover_open(pk));
        }
        if let Some(target) = r.str("interestfor") {
            let pk = popover_key(&b.mount, target);
            let delay = (self.theme.durations_ms.slow * self.ticks_per_second as f32 / 1000.0)
                .ceil() as u32;
            let s = &self.state;
            let interest =
                (s.hot == key && s.hover_ticks >= delay) || (s.focus == key && s.focus_visible);
            let link = self.docs.popovers.entry(pk).or_default();
            if interest {
                link.invoker = (at, key);
                link.interest = true;
            } else if link.invoker.1 == key {
                link.interest = false;
            }
        }
    }

    /// Builds a `popover` element: nothing in its place but an empty box,
    /// and, open, a layer against the control that opened it. It is built
    /// every frame (its controls are other elements).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_popover(
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
        if let Some(owned) = self.owning.last_mut() {
            owned.moved = true;
        }
        let mode = match r.str("popover") {
            Some("manual") => Mode::Manual,
            Some("hint") => Mode::Hint,
            _ => Mode::Auto,
        };
        let pk = popover_key(&b.mount, &n.id);
        let link = {
            let l = self.docs.popovers.entry(pk).or_default();
            l.mode = mode;
            *l
        };
        let (anchor, invoker) = link.invoker;
        let anchor = if anchor < self.nodes.len() && self.nodes[anchor].key == invoker {
            anchor
        } else {
            NONE
        };
        let mut open = self.popover_open(pk);
        // A hint shows while its control holds interest, or while the
        // pointer rests on the hint itself.
        let pointer = self.input.pointer;
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
            self.docs.happened.push(When::Toggle);
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
        let at = self.last_closed;
        self.nodes[at].props.position = crate::element::Position::Absolute;
        self.note_built(built);
        let t = self.theme;
        let content = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                padding: Insets::xy(t.space.sm, t.space.xs),
                frame: Some(if mode == Mode::Hint {
                    crate::style_file::StylePart::Tooltip
                } else {
                    crate::style_file::StylePart::Panel
                }),
                ..Element::default()
            },
            self.em_of(r),
        );
        let local = format!("{}~popover", path.replace('/', "_"));
        let mut layer = 0;
        if mode == Mode::Auto {
            let out = self.floating(&local, open, anchor, content, |ui| {
                layer = ui.nodes[ui.parent()].key;
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
            layer = self.nodes[i].key;
            build_children(self, b, None);
            self.close();
        }
        let l = self.docs.popovers.entry(pk).or_default();
        if layer != 0 {
            l.layer = layer;
        }
        l.shown = open && layer != 0;
        let resp = crate::ui::Response::default();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}
