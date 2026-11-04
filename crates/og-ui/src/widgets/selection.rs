use super::*;

impl Ui {
    // ── selection ──────────────────────────────────────────────────────────

    /// A segmented control: a neutral track, the selected tab on `knob`.
    /// Tabs are `Tab#i` (the probe reads their labels); a click or Activate
    /// selects. `changed` when the selection moved.
    pub fn tabs(
        &mut self,
        id: impl Into<Id>,
        items: &[&str],
        selected: &mut usize,
        desc: TabsDesc,
    ) -> Response {
        let t = self.theme;
        // Under a finger each tab is a whole target: the track grows by its
        // inset, so the tab inside it is the control height (44 px).
        let h = self.sizes().control_md
            + if self.viewport.coarse_pointer {
                2.0 * t.space.xxs
            } else {
                0.0
            };
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: desc
                    .width
                    .unwrap_or(if desc.fill { Size::grow() } else { Size::fit() }),
                height: Size::Fixed(h),
                padding: Insets::all(t.space.xxs),
                gap: t.space.xxs,
                background: Some(t.palette.control),
                radius: t.radius.control,
                part: Some(crate::dress::PartRef::new(
                    "Tabs",
                    "track",
                    crate::tokens::ElementClass::Tab,
                )),
                ..Element::default()
            },
        );
        let list = self.parent();
        self.nodes[list].a11y.role = Some("tablist");
        if !desc.label.is_empty() {
            self.nodes[list].a11y.name = Some(desc.label.clone());
        }
        // What the unselected labels sit on: the track's translucent fill over
        // the ground behind it, blended in linear light as the renderer does
        // (the dark track reads lighter than an sRGB mix says: inkMuted was
        // 3.95:1 on it). The muted ink moves as little as it takes to read
        // 4.5:1 there (a hair over, for the bytes), else is the main ink.
        let track = t.palette.control.over_linear(self.behind(list));
        let muted = t
            .palette
            .ink_muted
            .readable_on(track, 4.55)
            .unwrap_or(t.palette.ink);
        let mut out = Response::default();
        for (k, label) in items.iter().enumerate() {
            let i = self.leaf(
                "tab",
                crate::ui_idi("Tab", k as u32),
                Element {
                    width: if desc.fill { Size::grow() } else { Size::fit() },
                    height: Size::grow(),
                    padding: Insets::xy(t.space.md, 0.0),
                    interactive: true,
                    radius: t.radius.control,
                    ..Element::default()
                },
            );
            let r = self.nav_target(i, false);
            let on = *selected == k;
            if r.clicked && !on {
                *selected = k;
                out.changed = true;
                self.feedback(crate::tokens::SoundEvent::Tab);
            }
            let amounts = self.state_amounts(i, &r);
            out.hovered |= r.hovered;
            out.focused |= r.focused;
            out.clicked |= r.clicked;
            let on = *selected == k;
            let p = &t.palette;
            let mut part =
                crate::dress::PartRef::new("Tabs", "tab", crate::tokens::ElementClass::Tab)
                    .flag_if(on, "selected")
                    .flag_if(r.hovered && !r.pressed, "hover")
                    .flag_if(r.pressed, "pressed")
                    .flag_if(r.focused, "focused");
            part.hover = amounts.0;
            part.press = amounts.1;
            // The selected tab's label sits on `controlStrong` (a version 1
            // style's on `knob`), or on what the style paints the selected
            // tab with, in the accent ink that reads 4.5:1 there.
            let resolved = self.dress.resolve(&part);
            let grammar = crate::dress::pack_grammar(&t);
            let ink = if on {
                let ground = if resolved.draw.fills.is_empty() {
                    if grammar {
                        p.control_strong.over_linear(track)
                    } else {
                        p.knob.fade(1.0)
                    }
                } else {
                    let d = resolved.element_draw(&part, &t, None);
                    crate::dress::ground_of(&d, &t, track)
                };
                if grammar {
                    crate::dress::accent_ink(&t, ground)
                } else {
                    [p.ink, p.ink_inverse, p.on_primary]
                        .into_iter()
                        .find(|c| c.contrast(ground) >= 4.5)
                        .or_else(|| p.ink.readable_on(ground, 4.5))
                        .unwrap_or(p.ink_inverse)
                }
            } else {
                muted
            };
            let mut c = self.role_text(label, crate::tokens::TypeRole::Label, ink);
            c.align = TextAlign::Center;
            // The selected label reads semibold beside the muted others: the
            // tab is told by its ink and weight, not by its chip alone.
            if on {
                c.emphasis = 0.018;
            }
            // The selected tab's ground slides in on the `ui` spring.
            let sel = self.spring_to(
                crate::id::child_key(self.nodes[i].key, "on"),
                if on { 1.0 } else { 0.0 },
            );
            let clear = p.control.fade(0.0);
            // One selection grammar: the thumb is a step up from the track.
            let thumb = if grammar { p.control_strong } else { p.knob };
            let bg = mix_srgba(
                Ui::state_color(clear, p.control, p.control, amounts),
                thumb,
                sel,
            );
            let n = &mut self.nodes[i];
            n.el.background = (bg.alpha > 0.004).then_some(bg);
            n.el.part = Some(part);
            n.content = Content::Text(c);
            n.probe_text = (*label).to_owned();
            n.a11y.selected = Some(on);
        }
        self.close();
        out
    }

    /// A select: a field-like trigger with the chosen label and a chevron;
    /// a click or Activate opens its menu (`<id>.menu`, options `Option#i`)
    /// under it, a focus scope; choosing closes it with `changed`. Back or a
    /// press outside closes it unchanged.
    pub fn select(
        &mut self,
        id: impl Into<Id>,
        options: &[&str],
        selected: &mut usize,
        desc: &SelectDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let id = id.into();
        let name = id.as_str().to_owned();
        let h = self.sizes().control_md;
        let i = self.leaf(
            "select",
            id,
            Element {
                width: desc.width.unwrap_or(Size::Grow {
                    min: 160.0,
                    max: f32::INFINITY,
                }),
                height: Size::Fixed(h),
                padding: Insets {
                    left: t.space.md,
                    right: t.space.md + t.space.lg,
                    ..Insets::default()
                },
                interactive: !desc.disabled,
                radius: t.radius.control,
                background: Some(p.control),
                ..Element::default()
            },
        );
        self.nodes[i].disabled = desc.disabled;
        let key = self.nodes[i].key;
        let mut r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        let mut a = self.anim(key);
        let menu_key = crate::id::child_key(self.nodes[0].key, &format!("{name}.menu"));
        let mut open = a.h == 1;
        if r.clicked {
            open = !open;
            if open {
                self.request_focus =
                    crate::id::child_key(menu_key, &format!("Option#{}", *selected));
            }
        }
        // A press outside the trigger and its menu closes it.
        if open && self.input.pressed && !r.clicked {
            let inside = self
                .hit(self.state.hot)
                .is_some_and(|h| h.scope == menu_key || h.key == menu_key)
                || self.state.hot == key;
            if !inside {
                open = false;
            }
        }
        if open && self.state.scope == menu_key && self.nav.back {
            self.nav.back = false;
            open = false;
        }
        let shown = options.get(*selected).copied().unwrap_or("");
        let text = if shown.is_empty() {
            desc.placeholder.as_str()
        } else {
            shown
        };
        let mut c = self.text_content(
            text,
            t.type_ramp.body,
            if shown.is_empty() || desc.disabled {
                p.ink_faint
            } else {
                p.ink
            },
        );
        c.wrap = false;
        {
            let n = &mut self.nodes[i];
            n.content = Content::Text(c);
            n.probe_text = text.to_owned();
            n.a11y.name = Some(if desc.label.is_empty() {
                name.clone()
            } else {
                desc.label.clone()
            });
            n.a11y.value = Some(shown.to_owned());
            n.a11y.expanded = Some(open);
            if r.hovered && !desc.disabled {
                n.el.background = Some(p.control_strong);
            }
            // The chevron: two short bars.
            let ink = if desc.disabled {
                p.ink_faint
            } else {
                p.ink_muted
            };
            let (len, w) = (t.space.sm, 1.5);
            let flip = if open { -1.0 } else { 1.0 };
            for side in [-1.0f32, 1.0] {
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fx: 1.0,
                        fy: 0.5,
                        x: -t.space.md - t.space.sm / 2.0 + side * len * 0.35 - len / 2.0,
                        y: -w / 2.0,
                        w: len,
                        h: w,
                        ..Frame::default()
                    },
                    angle: side * flip * -FRAC_PI_4,
                    color: ink,
                });
            }
        }
        if open {
            let mut chosen = None;
            self.open_layer(
                "menu",
                Id::from(format!("{name}.menu")),
                Element {
                    direction: Direction::Column,
                    padding: Insets::all(t.space.xs),
                    gap: t.space.xxs,
                    background: Some(p.panel_raised),
                    radius: t.radius.small + t.space.xs,
                    cross: Cross::Stretch,
                    ..Element::default()
                },
                Float::Popover,
                i,
            );
            let menu = self.parent();
            self.nodes[menu].scope = true;
            self.nodes[menu].a11y.role = Some("listbox");
            self.nodes[menu].el.width = Size::Fit {
                min: self.hit(key).map_or(160.0, |h| h.rect.width),
                max: f32::INFINITY,
            };
            for (k, label) in options.iter().enumerate() {
                let o = self.leaf(
                    "option",
                    crate::ui_idi("Option", k as u32),
                    Element {
                        width: Size::grow(),
                        height: Size::Fixed(self.sizes().control_md),
                        padding: Insets::xy(t.space.md, 0.0),
                        interactive: true,
                        radius: t.radius.small,
                        ..Element::default()
                    },
                );
                let or = self.nav_target(o, false);
                if or.clicked {
                    chosen = Some(k);
                }
                let mut c = self.text_content(label, t.type_ramp.body, p.ink);
                c.wrap = false;
                let n = &mut self.nodes[o];
                n.content = Content::Text(c);
                n.probe_text = (*label).to_owned();
                n.a11y.selected = Some(k == *selected);
                n.el.background = if k == *selected || or.hovered {
                    Some(p.control_strong)
                } else {
                    None
                };
            }
            self.close();
            if let Some(k) = chosen {
                if k != *selected {
                    *selected = k;
                    r.changed = true;
                }
                open = false;
                self.request_focus = key;
            }
        }
        a.h = u64::from(open);
        self.set_anim(key, a);
        r
    }

    /// A number stepper: `−`, the value, `+` (`<id>/Dec`, `<id>/Value`,
    /// `<id>/Inc`); the buttons step by `step` within the range, Left and
    /// Right step it while a button is focused, and a screen reader's
    /// increment and decrement on its spinbutton (`<id>`) step it too.
    pub fn stepper(&mut self, id: impl Into<Id>, value: &mut f32, desc: StepperDesc) -> Response {
        let t = self.theme;
        let step = if desc.step > 0.0 { desc.step } else { 1.0 };
        let bounded = desc.min != desc.max;
        let (lo, hi) = (desc.min.min(desc.max), desc.min.max(desc.max));
        let h = self.sizes().control_md;
        let id = id.into();
        let name = id.as_str().to_owned();
        // A `stepper`, not a row: one of og-ui's controls, its states its own.
        let spin = self.push(
            "stepper",
            id,
            Element {
                direction: Direction::Row,
                height: Size::Fixed(h),
                gap: t.space.xs,
                cross: Cross::Center,
                ..Element::default()
            },
        );
        self.stack.push(spin);
        // The spinbutton is one stop for keys and pads, Left and Right (or
        // Up and Down) stepping it (the ARIA spinbutton pattern, as
        // og.ui.numberStepper); − and + are the pointer's shortcuts for
        // that, never stops of their own.
        self.nodes[spin].el.interactive = true;
        self.nodes[spin].el.radius = t.radius.control;
        let spun = self.nav_target(spin, true);
        // Its hover and press states (and the focus ring), on a clear ground.
        let amounts = self.state_amounts(spin, &spun);
        let clear = t.palette.control.fade(0.0);
        let bg = Ui::state_color(clear, t.palette.control, t.palette.control_strong, amounts);
        self.nodes[spin].el.background = (bg.alpha > 0.004).then_some(bg);
        self.nodes[spin].a11y.role = Some("spinbutton");
        if self.nodes[spin].a11y.name.is_none() {
            self.nodes[spin].a11y.name = Some(name);
        }
        self.nodes[spin].a11y.value = Some(format!("{:.*}", desc.decimals as usize, *value));
        let spin_key = self.nodes[spin].key;
        let mut delta = if self.nav.spin.0 == spin_key {
            self.nav.spin.1
        } else {
            0
        };
        if spun.focused {
            delta += self.nav.adjust;
        }
        let mut out = Response {
            hovered: spun.hovered,
            focused: spun.focused,
            ..Response::default()
        };
        let digits = desc.decimals as usize;
        for (name, sign) in [("Dec", -1), ("Inc", 1)] {
            if name == "Inc" {
                let text = format!("{:.*}", digits, *value);
                self.label(
                    "Value",
                    &text,
                    crate::LabelDesc {
                        align: TextAlign::Center,
                        nowrap: true,
                        font: FontRole::Mono,
                        width: Size::Fit {
                            min: h * 1.25,
                            max: f32::INFINITY,
                        },
                        ..Default::default()
                    },
                );
            }
            let at_end = bounded && if sign < 0 { *value <= lo } else { *value >= hi };
            let i = self.leaf(
                "button",
                Id::from(name),
                Element {
                    width: Size::Fixed(h),
                    height: Size::Fixed(h),
                    interactive: !at_end,
                    radius: t.radius.control,
                    background: Some(t.palette.control),
                    ..Element::default()
                },
            );
            // Never a stop of their own, under a finger too: inside the
            // spinbutton's box, a pad that met them could never land on the
            // spinbutton round them.
            let r = if at_end {
                Response::default()
            } else {
                let key = self.nodes[i].key;
                self.respond(key)
            };
            if r.clicked {
                delta += sign;
            }
            if r.focused {
                delta += self.nav.adjust;
            }
            out.hovered |= r.hovered;
            out.focused |= r.focused;
            out.clicked |= r.clicked;
            let p = t.palette;
            let ink = if at_end { p.ink_faint } else { p.ink };
            let n = &mut self.nodes[i];
            if r.hovered || r.pressed {
                n.el.background = Some(p.control_strong);
            }
            n.probe_text = if sign < 0 { "−".into() } else { "+".into() };
            n.a11y.name = Some(if sign < 0 { "Decrease" } else { "Increase" }.into());
            let len = t.space.md;
            n.shapes.push(Shape::Rect {
                frame: Frame::centered(0.0, 0.0, len, 2.0),
                radius: 1.0,
                color: ink,
            });
            if sign > 0 {
                n.shapes.push(Shape::Rect {
                    frame: Frame::centered(0.0, 0.0, 2.0, len),
                    radius: 1.0,
                    color: ink,
                });
            }
        }
        self.close();
        if delta != 0 {
            let mut v = *value + step * delta as f32;
            if bounded {
                v = v.clamp(lo, hi);
            }
            let k = 10f32.powi(desc.decimals as i32);
            v = (v * k).round() / k;
            if v != *value {
                *value = v;
                out.changed = true;
            }
        }
        self.nodes[spin].a11y.value = Some(format!("{:.*}", desc.decimals as usize, *value));
        self.nodes[spin].a11y.range = Some(crate::a11y::A11yRange {
            min: bounded.then_some(lo),
            max: bounded.then_some(hi),
            now: *value,
        });
        out
    }

    /// A radio group: `Option#i` rows with a ring (filled primary when chosen)
    /// and a label.
    pub fn radio(
        &mut self,
        id: impl Into<Id>,
        options: &[&str],
        selected: &mut usize,
        desc: RadioDesc,
    ) -> Response {
        let t = self.theme;
        self.open(
            id,
            Element {
                direction: if desc.horizontal {
                    Direction::Row
                } else {
                    Direction::Column
                },
                gap: if desc.horizontal {
                    t.space.lg
                } else {
                    t.space.xs
                },
                // A row of options too wide for its room (a phone, large
                // text) flows onto a second line rather than squeezing a
                // label under its own width.
                wrap: desc.horizontal,
                ..Element::default()
            },
        );
        let group = self.parent();
        self.nodes[group].a11y.role = Some("radiogroup");
        if !desc.label.is_empty() {
            self.nodes[group].a11y.name = Some(desc.label.clone());
        }
        let mut out = Response::default();
        for (k, label) in options.iter().enumerate() {
            let r = self.mark_control(
                crate::ui_idi("Option", k as u32),
                label,
                *selected == k,
                true,
            );
            if r.clicked && *selected != k {
                *selected = k;
                out.changed = true;
            }
            out.clicked |= r.clicked;
            out.focused |= r.focused;
            out.hovered |= r.hovered;
        }
        self.close();
        out
    }

    /// A checkbox with a label; a click flips it.
    pub fn checkbox(&mut self, id: impl Into<Id>, on: &mut bool, label: &str) -> Response {
        let mut r = self.mark_control(id.into(), label, *on, false);
        if r.clicked {
            *on = !*on;
            r.changed = true;
        }
        r
    }

    /// Checkboxes in a column (`Option#i`); `changed` when any flipped.
    pub fn checkbox_list(
        &mut self,
        id: impl Into<Id>,
        labels: &[&str],
        values: &mut [bool],
    ) -> Response {
        let t = self.theme;
        self.open(
            id,
            Element {
                direction: Direction::Column,
                gap: t.space.xs,
                ..Element::default()
            },
        );
        let group = self.parent();
        self.nodes[group].a11y.role = Some("group");
        let mut out = Response::default();
        for (k, label) in labels.iter().enumerate() {
            let mut v = values.get(k).copied().unwrap_or(false);
            let r = self.checkbox(crate::ui_idi("Option", k as u32), &mut v, label);
            if let Some(slot) = values.get_mut(k) {
                *slot = v;
            }
            out.changed |= r.changed;
            out.clicked |= r.clicked;
            out.focused |= r.focused;
        }
        self.close();
        out
    }

    /// A checkbox or radio row: the mark, then the label.
    fn mark_control(&mut self, id: Id, label: &str, on: bool, round: bool) -> Response {
        let t = self.theme;
        let p = t.palette;
        let box_ = self.sizes().control_sm - 2.0 * t.space.xs;
        let h = self.sizes().control_sm;
        let i = self.leaf(
            if round { "radio" } else { "checkbox" },
            id,
            Element {
                height: Size::Fixed(h),
                padding: Insets {
                    left: box_ + t.space.sm,
                    right: t.space.xs,
                    ..Insets::default()
                },
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        let r = self.nav_target(i, false);
        if r.clicked {
            self.feedback(if round || !on {
                crate::tokens::SoundEvent::ToggleOn
            } else {
                crate::tokens::SoundEvent::ToggleOff
            });
        }
        let amounts = self.state_amounts(i, &r);
        // The mark fills on the `ui` spring as it turns on (and empties as
        // it turns off), so a click is a transition, not a jump.
        let fill_t = self.spring_to(
            crate::id::child_key(self.nodes[i].key, "on"),
            if on { 1.0 } else { 0.0 },
        );
        let mut c = self.role_text(label, crate::tokens::TypeRole::Body, p.ink);
        c.wrap = false;
        let n = &mut self.nodes[i];
        n.content = Content::Text(c);
        n.probe_text = label.to_owned();
        n.a11y.checked = Some(on);
        let frame = Frame {
            fy: 0.5,
            y: -box_ / 2.0,
            w: box_,
            h: box_,
            ..Frame::default()
        };
        // The box is a part of its own (`Checkbox.box[on]`, `Radio.box`):
        // a style dresses it; undressed it is the field's well, filling
        // with `primary` as it turns on.
        let rest = Ui::state_color(p.panel_raised, p.panel_hover, p.panel_hover, amounts);
        let lit = Ui::state_color(p.primary, p.primary_hover, p.primary_active, amounts);
        let mut part = if round {
            crate::dress::PartRef::new("Radio", "box", crate::tokens::ElementClass::Avatar)
                .flag("round")
        } else {
            crate::dress::PartRef::new("Checkbox", "box", crate::tokens::ElementClass::Field)
        }
        .flag_if(on, "on")
        .flag_if(r.hovered && !r.pressed, "hover")
        .flag_if(r.pressed, "pressed")
        .flag_if(r.focused, "focused");
        part.hover = amounts.0;
        part.press = amounts.1;
        n.shapes.push(Shape::Part {
            frame,
            part,
            fill: Some(mix_srgba(rest, lit, fill_t.clamp(0.0, 1.0))),
        });
        if fill_t > 0.01 {
            let ink = p.on_primary.fade(fill_t.clamp(0.0, 1.0));
            if round {
                let dot = box_ * 0.4 * fill_t.clamp(0.0, 1.2);
                n.shapes.push(Shape::Rect {
                    frame: Frame {
                        fy: 0.5,
                        x: (box_ - dot) / 2.0,
                        y: -dot / 2.0,
                        w: dot,
                        h: dot,
                        ..Frame::default()
                    },
                    radius: dot,
                    color: ink,
                });
            } else {
                // The check: a short and a long stroke.
                let s = box_ / 16.0;
                let w = 2.0f32.max(s * 2.0);
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fy: 0.5,
                        x: 3.5 * s,
                        y: 0.5 * s - w / 2.0,
                        w: 5.0 * s,
                        h: w,
                        ..Frame::default()
                    },
                    angle: FRAC_PI_4,
                    color: ink,
                });
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fy: 0.5,
                        x: 5.5 * s,
                        y: -1.5 * s - w / 2.0,
                        w: 9.0 * s,
                        h: w,
                        ..Frame::default()
                    },
                    angle: -FRAC_PI_4,
                    color: ink,
                });
            }
        }
        r
    }
}
