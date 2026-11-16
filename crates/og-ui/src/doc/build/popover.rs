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
use super::host::PopoverLink;
use super::nodes::Inner;
use super::*;

/// A popover's key in its mount: ids are unique in a document.
pub(super) fn popover_key(mount: &str, id: &str) -> u64 {
    crate::id::key_of_path(&format!("{mount}#{id}~popover"))
}

/// A popover's widget memory, kept in snapshots: `h` 1 while open, `t`
/// the tick it stopped showing plus one, `a` 1 while it shows (the style
/// `:popover-open` matched), `b` 1 while its layer is up.
impl Ui {
    /// Whether popover `pk` is open (widget memory: kept in snapshots).
    pub(crate) fn popover_open(&mut self, pk: u64) -> bool {
        self.has_anim(pk) && self.anim(pk).h != 0
    }

    pub(crate) fn set_popover_open(&mut self, pk: u64, open: bool) {
        if self.popover_open(pk) == open {
            return;
        }
        self.docs.state_mut().touched_popovers.insert(pk);
        // HTML: showing an auto popover closes the other auto ones, and a
        // hint the other hints.
        if open && let Some(mode) = self.docs.state().popovers.get(&pk).map(|l| l.mode) {
            let others: Vec<u64> = self
                .docs
                .state()
                .popovers
                .iter()
                .filter(|(k, l)| **k != pk && l.mode == mode && mode != Mode::Manual)
                .map(|(k, _)| *k)
                .collect();
            for k in others {
                self.docs.state_mut().touched_popovers.insert(k);
                if self.has_anim(k) {
                    let a = self.anim(k);
                    self.set_anim(k, crate::ui::Anim { h: 0, t: 0, ..a });
                }
            }
        }
        let a = self.anim(pk);
        self.set_anim(
            pk,
            crate::ui::Anim {
                h: u64::from(open),
                t: 0,
                ..a
            },
        );
    }

    /// After element `key` is built: its `title` tooltip, and what it does
    /// to the popovers it names.
    pub(crate) fn popover_invoker(
        &mut self,
        b: &Build<'_>,
        r: &Resolved,
        key: u64,
        resp: crate::ui::Response,
    ) {
        let at = self.tree.last_closed();
        if at == NONE || at >= self.tree.nodes().len() || self.tree.nodes()[at].key != key {
            return;
        }
        // The popovers it names: one whose link it changes is built again.
        let named: Vec<(u64, Option<PopoverLink>)> =
            ["popovertarget", "contextmenu", "interestfor"]
                .into_iter()
                .filter_map(|a| r.str(a))
                .map(|target| {
                    let pk = popover_key(&b.mount, target);
                    (pk, self.docs.state().popovers.get(&pk).copied())
                })
                .collect();
        self.invoke_popovers(b, r, key, at, resp);
        let state = self.docs.state_mut();
        for (pk, before) in named {
            if state.popovers.get(&pk).copied() != before {
                state.touched_popovers.insert(pk);
            }
        }
    }

    /// What element `key`, built as node `at`, does to the popovers it
    /// names, and its `title` tooltip.
    fn invoke_popovers(
        &mut self,
        b: &Build<'_>,
        r: &Resolved,
        key: u64,
        at: usize,
        resp: crate::ui::Response,
    ) {
        if let Some(title) = r.str("title").filter(|t| !t.is_empty()) {
            if self.tree.nodes()[at].a11y.description.is_none() {
                self.tree.semantics_mut(at).description = Some(title.to_string());
            }
            self.tooltip(title);
        }
        if let Some(target) = r.str("popovertarget") {
            let pk = popover_key(&b.mount, target);
            let open = self.popover_open(pk);
            // Its node this frame (and, open with none, as after a
            // restore, it is the anchor).
            let link = self.docs.state_mut().popovers.entry(pk).or_default();
            if link.invoker.1 == key || (open && link.invoker.1 == 0) {
                link.invoker = (at, key);
            }
            if resp.clicked {
                let next = match r.str("popovertargetaction") {
                    Some("show") => true,
                    Some("hide") => false,
                    _ => !open,
                };
                let link = self.docs.state_mut().popovers.entry(pk).or_default();
                link.invoker = (at, key);
                link.point = None;
                self.set_popover_open(pk, next);
            }
            self.tree.semantics_mut(at).expanded = Some(self.popover_open(pk));
        }
        // HTML's former `contextmenu` attribute: a secondary press or a long
        // press on it opens the popover at the pointer.
        if let Some(target) = r.str("contextmenu") {
            let context = resp.long_pressed
                || self.gesture_of(key).long_pressed
                || (self.interaction.input().secondary_pressed
                    && self.interaction.state().hot == key);
            if context {
                let pk = popover_key(&b.mount, target);
                let p = self.interaction.input().pointer;
                let link = self.docs.state_mut().popovers.entry(pk).or_default();
                link.invoker = (at, key);
                link.point = Some([p.x, p.y]);
                self.set_popover_open(pk, true);
            }
        }
        if let Some(target) = r.str("interestfor") {
            let pk = popover_key(&b.mount, target);
            // CSS's `interest-delay-start` and `-end` on the control, in
            // ticks; `normal` is the style's slow and fast durations.
            let theme = *self.env.theme();
            let tps = self.env.ticks_per_second();
            let delay = |prop: &str, normal: f32| {
                let ms = r
                    .str(prop)
                    .and_then(|v| crate::doc::interact_css::interest_delay(v).ok())
                    .flatten()
                    .unwrap_or(normal);
                (ms * tps as f32 / 1000.0).ceil() as u32
            };
            let start = delay("interestDelayStart", theme.durations_ms.slow);
            let end = delay("interestDelayEnd", theme.durations_ms.fast);
            let s = &self.interaction.state();
            let wants =
                (s.hot == key && s.hover_ticks >= start) || (s.focus == key && s.focus_visible);
            // Counting the hover towards interest: it builds again next
            // frame.
            let counting = s.hot == key && s.hover_ticks < start && !wants;
            let now = s.tick;
            if counting && let Some(owned) = self.tree.owning_mut().last_mut() {
                owned.moved = true;
            }
            // The tick interest was lost, plus one: widget memory, so a
            // restore lands mid-delay too.
            let mk = crate::id::child_key(pk, "~interest");
            let lost = self.anim(mk);
            let link = self.docs.state_mut().popovers.entry(pk).or_default();
            if wants {
                link.invoker = (at, key);
                link.interest = true;
                if lost.t != 0 {
                    self.set_anim(mk, crate::ui::Anim::default());
                }
            } else if link.invoker.1 == key && link.interest {
                let since = if lost.t == 0 { now + 1 } else { lost.t };
                if now + 1 >= since + u64::from(end) {
                    link.interest = false;
                    self.set_anim(mk, crate::ui::Anim::default());
                } else {
                    if lost.t == 0 {
                        self.set_anim(mk, crate::ui::Anim { t: since, ..lost });
                    }
                    // Counting down: it builds again next frame.
                    if let Some(owned) = self.tree.owning_mut().last_mut() {
                        owned.moved = true;
                    }
                }
            }
        }
    }

    /// Builds a `popover` element: nothing in its place but an empty box,
    /// and, open, a layer against the control that opened it. It is built
    /// every frame (its controls are other elements).
    ///
    /// Closed, it is HTML's `display: none` popover: it keeps no style to
    /// transition from, so it opens from its `@starting-style` (its layer
    /// shows from the frame `:popover-open` matches), and its layer carries
    /// its opacity, transform and their transitions and animations. Closing
    /// with an `allow-discrete` transition of `overlay` (or `display`), its
    /// layer stays over the page, inert, until that transition ends, while
    /// its closed style transitions in (CSS Position 4 §4.2). A styled
    /// `::backdrop` covers the page under it while its layer is up.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_popover(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        mut build_inner: impl FnMut(&mut Ui, &mut Build<'_>, Inner<'_>),
    ) -> Option<String> {
        let mode = match r.str("popover") {
            Some("manual") => Mode::Manual,
            Some("hint") => Mode::Hint,
            _ => Mode::Auto,
        };
        let pk = popover_key(&b.mount, &n.id);
        let link = {
            let l = self.docs.state_mut().popovers.entry(pk).or_default();
            l.mode = mode;
            *l
        };
        let (anchor, invoker) = link.invoker;
        let anchor = if anchor < self.tree.nodes().len() && self.tree.nodes()[anchor].key == invoker
        {
            anchor
        } else {
            NONE
        };
        let mut open = self.popover_open(pk);
        // A hint shows while its control holds interest, or while the
        // pointer rests on the hint itself.
        let pointer = self.interaction.input().pointer;
        let over = link.layer != 0
            && self
                .last_bounds(link.layer)
                .is_some_and(|rect| rect.contains(pointer.x, pointer.y));
        if link.interest || (open && mode == Mode::Hint && over) {
            open = true;
        } else if mode == Mode::Hint {
            open = false;
        }
        // Closing: the tick it stopped showing (plus one), in its widget
        // memory, so a restore lands mid-hold too.
        let now = self.interaction.state().tick;
        let memory = self.anim(pk);
        let (was_shown, was_up) = (memory.a != 0.0, memory.b != 0.0);
        let closed_at = match (open, was_shown) {
            (true, _) => 0,
            (false, true) => now + 1,
            (false, false) => memory.t,
        };
        let tps = self.env.ticks_per_second();
        let tokens = self.env.theme().motion;
        let hold_ticks = r
            .transitions
            .iter()
            .filter(|t| t.discrete && (t.covers("overlay") || t.covers("display")))
            .map(|t| {
                crate::style::Theme::ticks(
                    t.delay_ms.max(0.0) + t.transition.duration_ms(&tokens),
                    tps,
                ) as u64
            })
            .max()
            .unwrap_or(0);
        let held = closed_at != 0 && now + 1 < closed_at + hold_ticks;
        // Opening, its layer waits a frame for `:popover-open`'s style
        // (which follows `shown`) unless it is still up.
        let up = (open && (was_shown || was_up)) || held;
        if open != was_shown {
            self.docs.state_mut().happened.push(When::Toggle);
        }
        if !up {
            // Not rendered: no before-change style, as a `display: none`
            // box keeps none, so the next opening starts afresh.
            let state = self.docs.state_mut();
            state.memory.seen_now.remove(&key);
            self.interaction.state.motion.styles.remove(&key);
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
        let at = self.tree.last_closed();
        self.tree.props_mut(at).position = crate::element::Position::Absolute;
        self.docs.note_built(built);
        let t = *self.env.theme();
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
        // `::backdrop`: a layer over the page, under the popover's.
        if up && let Some(backdrop) = self.style.pseudo_node(b, r, &path, "backdrop") {
            let at = self.open_layer(
                "backdrop",
                crate::Id::from(format!("{local}~backdrop").as_str()),
                Element::default(),
                crate::layout::Float::Modal,
                NONE,
            );
            self.close();
            self.style_pseudo_box(b, at, &backdrop);
            self.tree.nodes[at].backdrop = true;
        }
        let mut layer = 0;
        let mut layer_at = NONE;
        if mode == Mode::Auto {
            let out = self.floating(&local, up, anchor, content, |ui| {
                layer_at = ui.tree.parent();
                layer = ui.tree.nodes()[layer_at].key;
                build_inner(ui, b, Inner::Children(None));
            });
            if out.dismissed && open {
                self.set_popover_open(pk, false);
            }
        } else if up {
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
            layer = self.tree.nodes()[i].key;
            layer_at = i;
            build_inner(self, b, Inner::Children(None));
            self.close();
        }
        if layer_at != NONE {
            self.place_popover(r, layer_at, link.point);
            // Its own motion on its layer: the engine's drop-in only when
            // it brings none.
            let own = !props.transitions.is_empty() || !props.animations.is_empty();
            let p = self.tree.props_mut(layer_at);
            if own {
                p.transform = props.transform;
                p.opacity = props.opacity;
            } else {
                p.opacity *= props.opacity;
                if props.transform != crate::element::Transform::IDENTITY {
                    p.transform = props.transform;
                }
            }
            p.space3d = props.space3d;
            p.transitions = props.transitions;
            p.animations = props.animations;
            p.keyframe_bursts = props.keyframe_bursts;
            p.starting = props.starting;
            p.motion = props.motion;
            p.reduced = props.reduced;
            p.inert |= !open;
        }
        let l = self.docs.state_mut().popovers.entry(pk).or_default();
        if layer != 0 {
            l.layer = layer;
        }
        l.shown = open;
        let up = layer_at != NONE;
        // A dismiss this frame closed it (`h`).
        let now_memory = self.anim(pk);
        let next = crate::ui::Anim {
            a: f32::from(u8::from(open)),
            b: f32::from(u8::from(up)),
            t: closed_at,
            ..now_memory
        };
        if next != now_memory {
            self.set_anim(pk, next);
        }
        // Built again while it changes: opening (its layer waits a frame
        // for `:popover-open`), held up while it closes, or an open hint
        // under the pointer; else when what names it changes it
        // (`touched_popovers`).
        let hint_over = mode == Mode::Hint && open;
        if (next != memory || held || hint_over)
            && let Some(owned) = self.tree.owning_mut().last_mut()
        {
            owned.moved = true;
        }
        self.docs.state_mut().touched_popovers.remove(&pk);
        let resp = crate::ui::Response::default();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// Popover layer `at`'s anchor positioning against its implicit anchor
    /// (the layout pass reads its `anchor` node, or `point`), and its arrow.
    pub(crate) fn place_popover(&mut self, r: &Resolved, at: usize, point: Option<[f32; 2]>) {
        let (spec, authored) = Self::anchor_spec(r, &mut Props::default());
        if authored {
            self.tree.props_mut(at).anchor = Some(Box::new(spec));
        }
        let background = self.tree.nodes()[at].el.background;
        let arrow = r
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
                    .or(background)
                    .unwrap_or(self.env.theme().palette.panel_raised);
                (size, color)
            });
        self.tree.anchor_at(at, point, arrow);
    }
}
