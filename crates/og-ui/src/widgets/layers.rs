use super::*;

impl Ui {
    // ── layers ─────────────────────────────────────────────────────────────

    /// Opens an anchored group; children follow until [`Ui::close`].
    pub fn open_anchor(&mut self, id: impl Into<Id>, desc: AnchorDesc) {
        let margin = desc.margin.unwrap_or(self.sizes().gutter);
        let el = Element {
            direction: desc.direction,
            width: desc.width,
            height: desc.height,
            padding: desc.padding,
            gap: desc.gap,
            align: desc.align,
            cross: desc.cross,
            background: desc.background,
            radius: desc.radius,
            frame: desc.frame.or(Some(crate::style_file::StylePart::Panel)),
            ..Element::default()
        };
        let i = self.open_layer(
            "anchor",
            id.into(),
            el,
            Float::Anchored {
                anchor: desc.anchor,
                margin,
                dx: desc.offset.0,
                dy: desc.offset.1,
                over: desc.over,
                tooltip: desc.tooltip,
                avoid: desc.avoid,
            },
            NONE,
        );
        // A style's entrance and stagger for a screen panel, from the tick it
        // first appears; an instant panel is settled from there (`h` 1).
        let m = self.theme.art.motion;
        if desc.background.is_some() && (m.enter.is_some() || m.stagger_ms > 0.0) {
            let key = crate::id::child_key(self.nodes[i].key, "enter");
            let fresh = !self.has_anim(key);
            let mut a = self.anim(key);
            if fresh {
                a.t = self.state.tick;
                a.h = u64::from(desc.instant);
            }
            self.set_anim(key, a);
            if a.h == 0 {
                self.enter(i, a.t, m.enter);
            }
        }
    }

    /// Plays a style entrance on layer `i` from tick `since`: its own
    /// transition (`kind`) and its controls' stagger.
    fn enter(&mut self, i: usize, since: u64, kind: Option<TransitionKind>) {
        let t = self.theme;
        let ms = kind.map_or(t.durations_ms.base, |k| k.duration_ms(t.durations_ms.base));
        let ticks = self.ticks_for(ms) as f32;
        let p = ((self.state.tick - since) as f32 + 1.0) / ticks;
        if let Some(kind) = kind
            && p < 1.0
        {
            let mut f = fx(
                kind,
                t.tokens.ease(EaseRole::WidgetEnter).apply(p.min(1.0)),
                t.space.xl,
                t.reduced_motion,
            );
            f.inert = true;
            self.nodes[i].fx = Some(f);
        }
        let stagger = t.art.motion.stagger_ms;
        if stagger > 0.0 && !t.reduced_motion {
            self.nodes[i].stagger = Some(crate::layout::Stagger {
                since,
                step: stagger * self.ticks_per_second as f32 / 1000.0,
                ticks,
                kind: TransitionKind::SlideUp,
            });
        }
    }

    /// An anchored group built by `f`.
    pub fn anchor(&mut self, id: impl Into<Id>, desc: AnchorDesc, f: impl FnOnce(&mut Ui)) {
        self.open_anchor(id, desc);
        f(self);
        self.close();
    }

    /// Advances a presence memory and answers it with its eased value.
    fn presence(&mut self, key: u64, show: bool, ms: f32, instant: bool) -> (Presence, f32) {
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        if fresh {
            a.a = if show && instant { 1.0 } else { 0.0 };
        }
        let step = 1.0 / self.ticks_for(ms) as f32;
        let before = a.a;
        a.a = if show {
            (a.a + step).min(1.0)
        } else {
            (a.a - step).max(0.0)
        };
        if fresh && show && !instant {
            a.a = step.min(1.0);
        }
        self.set_anim(key, a);
        let p = a.a;
        let visible = p > 0.0 || show;
        let e = if show {
            self.theme.tokens.ease(EaseRole::WidgetEnter).apply(p)
        } else {
            1.0 - self.theme.tokens.ease(EaseRole::WidgetExit).apply(1.0 - p)
        };
        (
            Presence {
                visible,
                progress: p,
                entering: show && p < 1.0,
                leaving: !show && p > 0.0 && before > 0.0,
            },
            e,
        )
    }

    /// Opens a transition container (a column); children follow until
    /// [`Ui::close`]. Build them only while `visible`: it stays visible while
    /// it leaves. Interaction inside waits until it settles.
    pub fn open_transition(
        &mut self,
        id: impl Into<Id>,
        show: bool,
        desc: TransitionDesc,
    ) -> Presence {
        let id = id.into();
        let i = self.push(
            "transition",
            id,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
        );
        self.stack.push(i);
        let key = self.nodes[i].key;
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else {
            desc.kind.duration_ms(self.theme.durations_ms.base)
        };
        let (pr, e) = self.presence(key, show, ms, desc.instant);
        let distance = if desc.distance > 0.0 {
            desc.distance
        } else {
            self.theme.space.xl
        };
        let mut f = fx(desc.kind, e, distance, self.theme.reduced_motion);
        f.inert = pr.progress < 1.0 || !show;
        self.nodes[i].fx = Some(f);
        self.nodes[i].hidden = !pr.visible;
        pr
    }

    /// A transition built by `f` while visible.
    pub fn transition(
        &mut self,
        id: impl Into<Id>,
        show: bool,
        desc: TransitionDesc,
        f: impl FnOnce(&mut Ui),
    ) -> Presence {
        let p = self.open_transition(id, show, desc);
        if p.visible {
            f(self);
        }
        self.close();
        p
    }

    /// Opens a dialog: a scrim backdrop over everything with a centred
    /// panel (`panelRaised`, `radius.panel`). Its content follows until
    /// [`Ui::close`]; build it only while `visible`. Focus moves into it and
    /// stays there while it is the topmost; Back and the backdrop dismiss it
    /// unless `locked`. The probe sees the backdrop as `<id>` and the panel
    /// as `<id>/Panel`.
    pub fn open_modal(&mut self, id: impl Into<Id>, open: bool, desc: &ModalDesc) -> ModalResponse {
        let t = self.theme;
        let backdrop = self.open_layer(
            "modal",
            id.into(),
            Element {
                direction: Direction::Column,
                align: Align::Center,
                cross: Cross::Center,
                padding: Insets::all(self.sizes().gutter),
                background: Some(desc.backdrop.unwrap_or(t.palette.scrim)),
                ..Element::default()
            },
            Float::Modal,
            NONE,
        );
        let key = self.nodes[backdrop].key;
        let kind = desc
            .transition
            .or(t.art.motion.enter)
            .unwrap_or(TransitionKind::Scale);
        let ms = kind.duration_ms(t.durations_ms.base);
        let (pr, e) = self.presence(key, open, ms, false);
        if pr.entering && pr.progress <= 1.0 / self.ticks_for(ms) as f32 + 1e-6 {
            self.sound(crate::style_file::UiSound::Open);
        }
        if pr.leaving && pr.progress >= 1.0 - 1.0 / self.ticks_for(ms) as f32 - 1e-6 {
            self.sound(crate::style_file::UiSound::Close);
        }
        let mut r = ModalResponse {
            visible: pr.visible,
            dismissed: false,
        };
        {
            let n = &mut self.nodes[backdrop];
            n.hidden = !pr.visible;
            n.backdrop = true;
            n.scope = true;
            n.locked = desc.locked;
            // Its content takes input from the first frame it shows: a menu
            // or a popover is pressed while it still eases in. Leaving, it
            // takes none.
            n.fx = Some(crate::motion::Fx {
                alpha: e,
                inert: !open,
                ..Default::default()
            });
        }
        if open && !desc.locked {
            let resp = self.respond(key);
            // A press on the backdrop dismisses once it has entered: the
            // second press of a double click on what opened it does not
            // close it again.
            let entered = pr.progress >= 1.0;
            if (resp.clicked && entered && !desc.ignore_backdrop)
                || (self.state.scope == key && self.nav.back)
            {
                if self.state.scope == key {
                    self.nav.back = false;
                }
                r.dismissed = true;
            }
        }
        let vw = self.viewport.width - self.viewport.safe_area.horizontal();
        let width = desc
            .width
            .unwrap_or(Size::Fixed(420.0f32.min(vw - 2.0 * self.sizes().gutter)));
        let panel = self.push(
            "panel",
            Id::from("Panel"),
            Element {
                direction: Direction::Column,
                width,
                padding: if desc.bare {
                    Insets::default()
                } else {
                    Insets::all(t.space.xl)
                },
                gap: t.space.md,
                background: (!desc.bare).then_some(t.palette.panel_raised),
                radius: t.radius.panel,
                frame: Some(crate::style_file::StylePart::Panel),
                interactive: !desc.bare,
                cross: if desc.bare {
                    Cross::Center
                } else {
                    Cross::Start
                },
                ..Element::default()
            },
        );
        self.stack.push(panel);
        self.nodes[panel].a11y.role = Some("dialog");
        self.nodes[panel].a11y.name = Some(desc.title.clone());
        if open && t.art.motion.stagger_ms > 0.0 {
            let since = self.since(crate::id::child_key(key, "enter"));
            self.enter(panel, since, None);
        }
        let moving = fx(kind, e, t.space.xl, t.reduced_motion);
        self.nodes[panel].fx = Some(crate::motion::Fx {
            alpha: 1.0,
            inert: false,
            ..moving
        });
        if !desc.title.is_empty() && pr.visible {
            self.label(
                "Title",
                &desc.title,
                crate::LabelDesc {
                    step: Some(self.type_step(crate::tokens::TypeRole::Heading)),
                    font: self.type_font(crate::tokens::TypeRole::Heading),
                    heading: true,
                    ..Default::default()
                },
            );
        }
        r
    }

    /// Closes a dialog opened with [`Ui::open_modal`] (its panel and backdrop).
    pub fn close_modal(&mut self) {
        self.close();
        self.close();
    }

    /// Opens an HTML `<dialog>`. Modal, it is [`Ui::open_modal`] (a scrim
    /// that takes every press, the focus moved in, trapped, and returned
    /// on close; Back or the backdrop dismisses it); its panel is
    /// `<id>/Panel`. Non-modal, it is a panel in place, `<id>` (`Dialog.
    /// panel[open]`), shown only while open: Back (Escape, gamepad B)
    /// dismisses it while the focus is inside, and on closing with the
    /// focus inside, the focus returns to what held it when it opened. Its
    /// content follows until [`Ui::close_dialog`]. Role dialog, named
    /// `desc.name`.
    pub fn open_dialog(
        &mut self,
        id: impl Into<Id>,
        open: bool,
        desc: &DialogDesc,
    ) -> ModalResponse {
        let id = id.into();
        if desc.modal {
            // Its place in the tree: a retained document finds the dialog's
            // node by its id under its parent, and the backdrop is a layer
            // outside the tree (keyed as `open_layer` keys it).
            let layer = crate::id::child_key(self.nodes[0].key, &id.as_str().replace('/', "_"));
            self.layer_place(&id, layer);
            let r = self.open_modal(
                id,
                open,
                &ModalDesc {
                    width: desc.width,
                    backdrop: desc.backdrop,
                    locked: desc.closedby == ClosedBy::None,
                    ignore_backdrop: desc.closedby != ClosedBy::Any,
                    bare: desc.bare,
                    ..ModalDesc::default()
                },
            );
            let panel = self.parent();
            // The scrim is `Dialog.backdrop` to a style (CSS `::backdrop`).
            let scrim = self.nodes[panel].parent;
            self.nodes[scrim].el.part = crate::dress::PartRef::parse(
                "dialog::part(backdrop)",
                None::<crate::tokens::ElementClass>,
            )
            .ok();
            self.nodes[panel].a11y.name = Some(desc.name.clone());
            return r;
        }
        let t = self.theme;
        let path = self.child_path(id.as_str());
        let opener_key = format!("{}~opener", id.as_str());
        let was_open = self.memory(&format!("{}~open", id.as_str()), 0.0) > 0.0;
        let focus = self.state.focus;
        let inside = self.focus_path(0).starts_with(&format!("{path}/"));
        let mut r = ModalResponse {
            visible: open,
            dismissed: false,
        };
        let closes = matches!(desc.closedby, ClosedBy::Any | ClosedBy::CloseRequest);
        if open && inside && closes && self.nav.back {
            self.nav.back = false;
            r.dismissed = true;
        }
        if open && !was_open {
            self.remember(&opener_key, f64::from_bits(focus));
            self.sound(crate::style_file::UiSound::Open);
        }
        let opener = self.memory(&opener_key, 0.0).to_bits();
        if (!open || r.dismissed) && was_open {
            if inside && opener != 0 {
                self.request_focus = opener;
            }
            self.sound(crate::style_file::UiSound::Close);
        }
        self.remember(
            &format!("{}~open", id.as_str()),
            if open && !r.dismissed { 1.0 } else { 0.0 },
        );
        let i = self.push(
            "panel",
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or_default(),
                padding: if desc.bare {
                    Insets::default()
                } else {
                    Insets::all(t.space.xl)
                },
                gap: t.space.md,
                background: (!desc.bare).then_some(t.palette.panel_raised),
                radius: if desc.bare { 0.0 } else { t.radius.panel },
                frame: (!desc.bare).then_some(crate::style_file::StylePart::Panel),
                part: (!desc.bare)
                    .then(|| {
                        crate::dress::PartRef::parse(
                            if open {
                                "dialog::part(panel):open"
                            } else {
                                "dialog::part(panel)"
                            },
                            crate::tokens::ElementClass::Panel,
                        )
                        .ok()
                    })
                    .flatten(),
                interactive: true,
                ..Element::default()
            },
        );
        self.stack.push(i);
        self.nodes[i].hidden = !open;
        self.nodes[i].a11y.role = Some("dialog");
        self.nodes[i].a11y.name = Some(desc.name.clone());
        r
    }

    /// Closes a dialog opened with [`Ui::open_dialog`].
    pub fn close_dialog(&mut self, desc: &DialogDesc) {
        if desc.modal {
            self.close_modal();
        } else {
            self.close();
        }
    }

    /// A dialog built by `f` while visible.
    pub fn modal(
        &mut self,
        id: impl Into<Id>,
        open: bool,
        desc: &ModalDesc,
        f: impl FnOnce(&mut Ui),
    ) -> ModalResponse {
        let r = self.open_modal(id, open, desc);
        if r.visible {
            f(self);
        }
        self.close_modal();
        r
    }

    /// Queues a toast: it stacks at the toast anchor, enters, stays for its
    /// duration and leaves; at most three are up (older ones leave first).
    /// A press on one dismisses it. Callable between ticks. Returns its id.
    pub fn toast(&mut self, text: &str, desc: ToastDesc) -> u32 {
        let (m, table) = (self.theme.art.motion, crate::theme::TIMING);
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else if desc.tone == Tone::Danger {
            if m.toast_danger_ms > 0.0 {
                m.toast_danger_ms
            } else {
                table.toast_danger_ms
            }
        } else if m.toast_hold_ms > 0.0 {
            m.toast_hold_ms
        } else {
            table.toast_hold_ms
        };
        self.state.next_toast += 1;
        let id = self.state.next_toast;
        let born = self.state.tick + u64::from(!self.building);
        let ticks = self.ticks_for(ms);
        match desc.tone {
            Tone::Success => self.sound(crate::style_file::UiSound::Success),
            Tone::Danger => self.sound(crate::style_file::UiSound::Error),
            _ => {}
        }
        self.state.toasts.push(Toast {
            id,
            text: text.to_owned(),
            tone: desc.tone.index(),
            born,
            ticks,
            leaving: 0,
        });
        id
    }

    /// Starts a toast's exit.
    pub fn dismiss_toast(&mut self, id: u32) {
        let now = self.state.tick;
        for t in self.state.toasts.iter_mut() {
            if t.id == id && t.leaving == 0 {
                t.leaving = now.max(1);
            }
        }
    }

    /// The toasts up now (id, text), oldest first.
    pub fn toasts(&self) -> Vec<(u32, String)> {
        self.state
            .toasts
            .iter()
            .map(|t| (t.id, t.text.clone()))
            .collect()
    }

    /// Builds the toast stack (called by `end`).
    pub(crate) fn build_toasts(&mut self) {
        let now = self.state.tick;
        let exit = u64::from(self.ticks_for(self.theme.durations_ms.fast));
        let enter = self.ticks_for(self.theme.durations_ms.base);
        // Expire, cap at three, drop the ones whose exit finished.
        let mut up: Vec<usize> = Vec::new();
        for (k, t) in self.state.toasts.iter_mut().enumerate() {
            if t.leaving == 0 && now >= t.born + u64::from(t.ticks) {
                t.leaving = now;
            }
            if t.leaving == 0 {
                up.push(k);
            }
        }
        if up.len() > 3 {
            for &k in &up[..up.len() - 3] {
                self.state.toasts[k].leaving = now;
            }
        }
        self.state
            .toasts
            .retain(|t| t.leaving == 0 || now < t.leaving + exit);
        if self.state.toasts.is_empty() {
            return;
        }
        let t = self.theme;
        let top = matches!(
            self.toast_anchor,
            Anchor::Top | Anchor::TopLeft | Anchor::TopRight
        );
        let margin = self.sizes().gutter;
        self.open_layer(
            "toasts",
            Id::from("Toasts"),
            Element {
                direction: Direction::Column,
                gap: t.space.sm,
                cross: Cross::Center,
                ..Element::default()
            },
            Float::Toasts {
                anchor: self.toast_anchor,
                margin,
            },
            NONE,
        );
        let toasts = self.state.toasts.clone();
        let mut dismiss = Vec::new();
        for toast in toasts {
            let age = now.saturating_sub(toast.born) as f32;
            let p = if toast.leaving != 0 {
                1.0 - ((now - toast.leaving) as f32 / exit as f32).min(1.0)
            } else {
                ((age + 1.0) / enter as f32).min(1.0)
            };
            let e = if toast.leaving != 0 {
                1.0 - t.tokens.ease(EaseRole::WidgetExit).apply(1.0 - p)
            } else {
                t.tokens.ease(EaseRole::WidgetEnter).apply(p)
            };
            let kind = t.art.motion.toast.unwrap_or(if top {
                TransitionKind::SlideDown
            } else {
                TransitionKind::SlideUp
            });
            let distance = match kind {
                TransitionKind::SlideLeft | TransitionKind::SlideRight => 4.0 * t.space.xxl,
                _ => t.space.lg,
            };
            let i = self.push(
                "toast",
                crate::ui_idi("Toast", toast.id),
                Element {
                    direction: Direction::Row,
                    height: Size::Fixed(self.sizes().control_lg),
                    padding: Insets::xy(t.space.lg, 0.0),
                    gap: t.space.sm,
                    cross: Cross::Center,
                    background: Some(t.palette.panel_raised),
                    radius: t.radius.control,
                    interactive: true,
                    frame: Some(crate::style_file::StylePart::Hud),
                    ..Element::default()
                },
            );
            self.stack.push(i);
            self.nodes[i].a11y.notify = true;
            let mut f = fx(kind, e, distance, t.reduced_motion);
            f.inert = toast.leaving != 0;
            self.nodes[i].fx = Some(f);
            if self.respond(self.nodes[i].key).clicked {
                dismiss.push(toast.id);
            }
            let tone = Tone::from_index(toast.tone);
            if tone != Tone::Neutral {
                let dot = t.space.sm;
                let c = self.tone_dot(tone);
                self.nodes[i].shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Toast", "indicator", None)
                        .attr("tone", tone.name()),
                    frame: Frame {
                        fy: 0.5,
                        x: t.space.lg,
                        y: -dot / 2.0,
                        w: dot,
                        h: dot,
                        ..Frame::default()
                    },
                    radius: dot,
                    color: c,
                });
                self.nodes[i].el.padding.left = t.space.lg + dot + t.space.sm;
            }
            self.label(
                "Text",
                &toast.text,
                crate::LabelDesc {
                    nowrap: true,
                    ..Default::default()
                },
            );
            self.nodes[i].probe_text = toast.text.clone();
            self.close();
        }
        self.close();
        for id in dismiss {
            self.dismiss_toast(id);
        }
    }

    pub(super) fn tone_dot(&self, tone: Tone) -> Srgba {
        let p = &self.theme.palette;
        match tone {
            Tone::Success => p.success,
            Tone::Warn => p.warn,
            Tone::Danger => p.danger,
            Tone::Info => p.info,
            Tone::Primary => p.primary,
            Tone::Inverse => p.ink,
            Tone::Neutral => p.ink_muted,
        }
    }

    /// A spotlight over everything (a tutorial step): the scrim on four
    /// sides of `hole` (CSS px; `None`: the whole surface dimmed), a primary
    /// edge around it. Presses on the dimmed parts stop there (`clicked`);
    /// presses in the hole reach what is under it. Focus stays with the
    /// controls under the hole and the layers above it. The probe sees the
    /// shades as `<id>/Top`, `Left`, `Right`, `Bottom` and the hole as `<id>/Hole`.
    pub fn spotlight(
        &mut self,
        id: impl Into<Id>,
        hole: Option<crate::Rect>,
        desc: SpotlightDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let vp = self.viewport;
        let pad = if desc.padding > 0.0 {
            desc.padding
        } else {
            t.space.sm
        };
        let h = hole.map(|r| r.outset(pad)).map(|r| {
            let x = r.x.clamp(0.0, vp.width);
            let y = r.y.clamp(0.0, vp.height);
            crate::Rect::new(
                x,
                y,
                (r.right().min(vp.width) - x).max(0.0),
                (r.bottom().min(vp.height) - y).max(0.0),
            )
        });
        let layer = self.open_layer(
            "spotlight",
            id.into(),
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            Float::Modal,
            NONE,
        );
        // The modal placement pads by the safe area; the shades must not.
        self.nodes[layer].el.padding = Insets::default();
        let mut out = Response::default();
        let mut shade = |ui: &mut Ui, name: &str, w: Size, hgt: Size| {
            let i = ui.leaf(
                "shade",
                Id::from(name),
                Element {
                    width: w,
                    height: hgt,
                    background: Some(p.scrim.deepen(1.6)),
                    interactive: true,
                    ..Element::default()
                },
            );
            let r = ui.respond(ui.nodes[i].key);
            out.clicked |= r.clicked;
            out.hovered |= r.hovered;
        };
        match h {
            None => shade(self, "Top", Size::grow(), Size::grow()),
            Some(h) => {
                shade(self, "Top", Size::grow(), Size::Fixed(h.y));
                self.open(
                    "Middle",
                    Element {
                        direction: Direction::Row,
                        width: Size::grow(),
                        height: Size::Fixed(h.height),
                        ..Element::default()
                    },
                );
                shade(self, "Left", Size::Fixed(h.x), Size::grow());
                let hole = self.leaf(
                    "hole",
                    Id::from("Hole"),
                    Element {
                        width: Size::Fixed(h.width),
                        height: Size::grow(),
                        ..Element::default()
                    },
                );
                let w = t.space.xxs;
                let edges = [
                    Frame {
                        fw: 1.0,
                        h: w,
                        ..Frame::default()
                    },
                    Frame {
                        fw: 1.0,
                        fy: 1.0,
                        y: -w,
                        h: w,
                        ..Frame::default()
                    },
                    Frame {
                        fh: 1.0,
                        w,
                        ..Frame::default()
                    },
                    Frame {
                        fx: 1.0,
                        x: -w,
                        fh: 1.0,
                        w,
                        ..Frame::default()
                    },
                ];
                for frame in edges {
                    self.nodes[hole].shapes.push(Shape::DressedRect {
                        part: crate::doc::schema::parts::SPOTLIGHT_EDGE.reference(None),
                        frame,
                        radius: 0.0,
                        color: p.focus,
                    });
                }
                shade(self, "Right", Size::grow(), Size::grow());
                self.close();
                shade(self, "Bottom", Size::grow(), Size::grow());
            }
        }
        self.close();
        out
    }
}
