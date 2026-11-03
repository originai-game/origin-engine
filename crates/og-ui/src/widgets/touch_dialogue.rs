use super::*;

impl Ui {
    /// A dialogue box: a portrait slot, the speaker in primary ink, and the
    /// line typed out by the tick at `chars_per_second`. A press (or
    /// Activate) while typing shows the whole line; once shown, it answers
    /// `advanced`. A new `text` starts typing again.
    pub fn dialogue(&mut self, id: impl Into<Id>, desc: &DialogueDesc) -> DialogueResponse {
        let t = self.theme;
        let p = t.palette;
        let i = self.push(
            "dialogue",
            id.into(),
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                padding: Insets::all(t.space.lg),
                gap: t.space.lg,
                cross: Cross::Start,
                background: Some(p.panel_raised),
                radius: t.radius.panel,
                interactive: true,
                ..Element::default()
            },
        );
        self.stack.push(i);
        let key = self.nodes[i].key;
        // A dialogue that appears takes the focus (the topmost owns it: the
        // og-ui skill §4, README §Focus navigation), so Enter, Space and the
        // pad's A reach it at once; under an open dialog it waits.
        if self.state.scope == 0 && !self.state.hits.iter().any(|h| h.key == key) {
            self.request_focus = key;
        }
        let r = self.nav_target(i, false);
        let h = hash_str(&desc.text);
        let mut a = self.anim(key);
        let now = self.state.tick;
        if a.h != h || a.t == 0 {
            a = Anim {
                a: 0.0,
                b: 0.0,
                t: now,
                h,
            };
        }
        let cps = if desc.chars_per_second > 0.0 {
            desc.chars_per_second
        } else {
            40.0
        };
        let total = desc.text.chars().count() as u32;
        let typed = if a.a >= 1.0 {
            total
        } else {
            ((((now - a.t) as f32) * cps / self.ticks_per_second as f32).floor() as u32).min(total)
        };
        let mut out = DialogueResponse {
            shown: typed,
            done: typed >= total,
            advanced: false,
        };
        if r.clicked {
            if out.done {
                out.advanced = true;
            } else {
                a.a = 1.0;
                out.shown = total;
                out.done = true;
            }
        }
        self.set_anim(key, a);
        let portrait = 72.0;
        let portrait_slot = |ui: &mut Ui| {
            let s = ui.leaf(
                "portrait",
                Id::from("Portrait"),
                Element {
                    width: Size::Fixed(portrait),
                    height: Size::Fixed(portrait),
                    background: Some(p.control),
                    radius: t.radius.panel,
                    ..Element::default()
                },
            );
            let icon = if desc.portrait_icon.is_empty() {
                None
            } else {
                ui.icon_shape(
                    &desc.portrait_icon,
                    Frame::centered(0.0, 0.0, portrait * 0.7, portrait * 0.7),
                    p.ink,
                )
            };
            match (desc.portrait, icon) {
                (Some((tex, uv)), _) => {
                    ui.nodes[s].content = Content::Image {
                        texture: Some(tex),
                        uv,
                        tint: Srgba::WHITE,
                    };
                }
                (None, Some(shape)) => ui.nodes[s].shapes.push(shape),
                (None, None) => {
                    let initial: String = desc.speaker.chars().take(1).collect();
                    ui.nodes[s].shapes.push(Shape::Text {
                        frame: Frame::FULL,
                        text: initial,
                        step: t.type_ramp.display_sm,
                        color: p.ink_muted,
                        font: FontRole::Display,
                        clip: Frame::FULL,
                    });
                }
            }
        };
        if !desc.portrait_right {
            portrait_slot(self);
        }
        self.open(
            "Body",
            Element {
                direction: Direction::Column,
                width: Size::grow(),
                height: Size::Fit {
                    min: portrait,
                    max: f32::INFINITY,
                },
                gap: t.space.xs,
                ..Element::default()
            },
        );
        if !desc.speaker.is_empty() {
            self.label(
                "Speaker",
                &desc.speaker,
                crate::LabelDesc {
                    step: Some(t.type_ramp.meta),
                    color: Some(p.highlight),
                    nowrap: true,
                    ..Default::default()
                },
            );
        }
        let shown: String = desc.text.chars().take(out.shown as usize).collect();
        self.label(
            "Line",
            &shown,
            crate::LabelDesc {
                step: Some(t.type_ramp.reading),
                width: Size::grow(),
                ..Default::default()
            },
        );
        self.close();
        if desc.portrait_right {
            portrait_slot(self);
        }
        if out.done {
            // The continue mark: a small chevron that bobs.
            let bob = if t.reduced_motion {
                0.0
            } else {
                ((now / self.ticks_for(t.durations_ms.slow) as u64) % 2) as f32 * 2.0
            };
            for side in [-1.0f32, 1.0] {
                self.nodes[i].shapes.push(Shape::Bar {
                    frame: Frame {
                        fx: 1.0,
                        fy: 1.0,
                        x: -t.space.lg - 6.0 + side * 2.5 - 4.0,
                        y: -t.space.md - 2.0 + bob,
                        w: 8.0,
                        h: 2.0,
                        ..Frame::default()
                    },
                    angle: side * FRAC_PI_4,
                    color: p.ink_muted,
                });
            }
        }
        self.nodes[i].probe_text = shown;
        self.close();
        out
    }

    // ── touch controls ─────────────────────────────────────────────────────

    /// A virtual joystick: a base disc and a knob that follows a press that
    /// started on it, clamped to the base. Answers the stick's -1..1 vector.
    /// It keeps the finger that grabbed it (any of og.Pointers), so another
    /// finger can hold a button at the same time.
    pub fn joystick(&mut self, id: impl Into<Id>, desc: JoystickDesc) -> JoystickResponse {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 128.0 }.max(3.0 * self.min_target());
        let i = self.leaf(
            "joystick",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: true,
                radius: size / 2.0,
                background: Some(p.hud),
                ..Element::default()
            },
        );
        self.nodes[i].props.customize = desc.customize;
        let key = self.nodes[i].key;
        let r = self.respond(key);
        let mut out = JoystickResponse::default();
        let radius = size / 2.0;
        let (mut kx, mut ky) = (0.0, 0.0);
        // The primary pointer holding it, else a further touch that grabbed it.
        let finger = if self.state.active == key && (self.input.down || self.input.pressed) {
            Some(self.input.pointer)
        } else {
            self.touch(key, true).filter(|t| t.1).map(|t| t.0)
        };
        if let Some(finger) = finger
            && let Some(h) = self.hit(key)
        {
            let (cx, cy) = h.rect.center();
            let (mut dx, mut dy) = ((finger.x - cx) / radius, (finger.y - cy) / radius);
            let len = (dx * dx + dy * dy).sqrt();
            if len > 1.0 {
                dx /= len;
                dy /= len;
            }
            kx = dx;
            ky = dy;
            let dz = if desc.dead_zone > 0.0 {
                desc.dead_zone
            } else {
                0.15
            };
            if len.min(1.0) > dz {
                out.x = dx;
                out.y = dy;
            }
            out.active = true;
        }
        let knob = size * 0.42;
        let travel = radius - knob / 2.0;
        let n = &mut self.nodes[i];
        n.shapes.push(Shape::Rect {
            frame: Frame::centered(0.0, 0.0, size - 2.0 * t.space.lg, size - 2.0 * t.space.lg),
            radius: size,
            color: p.control,
        });
        n.shapes.push(Shape::Rect {
            frame: Frame::centered(kx * travel, ky * travel, knob, knob),
            radius: knob,
            color: if r.pressed {
                p.ink
            } else {
                p.knob.mix(p.ink, 0.35)
            },
        });
        out
    }

    /// A d-pad: four arms (`Up`, `Down`, `Left`, `Right`), each at least the
    /// touch target, answering which are held.
    pub fn dpad(&mut self, id: impl Into<Id>, desc: DpadDesc) -> DpadResponse {
        let t = self.theme;
        let p = t.palette;
        let arm = (if desc.size > 0.0 { desc.size } else { 132.0 } / 3.0).max(self.min_target());
        // Nine cells in a wrapping row three wide: the arms are the pad's
        // children. The probe names the row `dpad` (a touch control).
        let pad = self.push(
            "dpad",
            id.into(),
            Element {
                direction: Direction::Row,
                wrap: true,
                width: Size::Fixed(arm * 3.0),
                height: Size::Fixed(arm * 3.0),
                ..Element::default()
            },
        );
        self.nodes[pad].props.customize = desc.customize;
        self.stack.push(pad);
        let mut out = DpadResponse::default();
        let cells: [Option<(&str, f32)>; 9] = [
            None,
            Some(("Up", -FRAC_PI_2)),
            None,
            Some(("Left", PI)),
            None,
            Some(("Right", 0.0)),
            None,
            Some(("Down", FRAC_PI_2)),
            None,
        ];
        for (k, cell) in cells.iter().enumerate() {
            match cell {
                None => {
                    let centre = k == 4;
                    let i = self.leaf(
                        "spacer",
                        Id::default(),
                        Element {
                            width: Size::Fixed(arm),
                            height: Size::Fixed(arm),
                            background: centre.then_some(p.hud),
                            ..Element::default()
                        },
                    );
                    self.nodes[i].widget = Widget::Spacer;
                }
                Some((name, angle)) => {
                    let i = self.leaf(
                        "button",
                        Id::from(*name),
                        Element {
                            width: Size::Fixed(arm),
                            height: Size::Fixed(arm),
                            interactive: true,
                            radius: t.radius.small,
                            background: Some(p.hud),
                            ..Element::default()
                        },
                    );
                    let key = self.nodes[i].key;
                    let r = self.respond(key);
                    let held = r.pressed || self.touch(key, false).is_some_and(|t| t.1);
                    match *name {
                        "Up" => out.up = held,
                        "Down" => out.down = held,
                        "Left" => out.left = held,
                        _ => out.right = held,
                    }
                    let ink = if held { p.primary } else { p.ink };
                    let n = &mut self.nodes[i];
                    if held {
                        n.el.background = Some(p.control_strong.over(p.hud.over(p.backdrop)));
                    }
                    n.probe_text = (*name).to_owned();
                    // A chevron pointing `angle`: two strokes meeting at its tip.
                    let len = arm * 0.24;
                    let (ca, sa) = (cos(*angle), sin(*angle));
                    let tip = (ca * len * 0.35, sa * len * 0.35);
                    for side in [-1.0f32, 1.0] {
                        let back = *angle + side * 3.0 * FRAC_PI_4;
                        let (ox, oy) =
                            (tip.0 + cos(back) * len * 0.5, tip.1 + sin(back) * len * 0.5);
                        n.shapes.push(Shape::Bar {
                            frame: Frame::centered(ox, oy, len, 3.0),
                            angle: back,
                            color: ink,
                        });
                    }
                }
            }
        }
        self.close();
        out
    }

    /// A square action button (jump, fire), cornered by the style's
    /// `radius.control` like every other button, or a disc (`round`, or a
    /// `charge` ring, which follows the outline): at least the touch
    /// target; `pressed` while held, `clicked` on a tap.
    pub fn action_button(
        &mut self,
        id: impl Into<Id>,
        text: &str,
        desc: &ActionButtonDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 64.0 }.max(self.min_target());
        // An icon the atlas has draws in place of the label; else the label
        // is drawn, and one wider than the button (a longer locale, 200 %
        // text) widens it rather than being cut: the button stays its size
        // tall.
        let s = size * 0.45;
        let has_icon = !desc.icon.is_empty()
            && self
                .icon_shape(&desc.icon, Frame::centered(0.0, 0.0, s, s), t.palette.ink)
                .is_some();
        let charge = desc.charge.map(|c| {
            if c.is_finite() {
                c.clamp(0.0, 1.0)
            } else {
                0.0
            }
        });
        let round = desc.round || charge.is_some();
        // The ring's band inside the edge, and the face inside it.
        let band = if charge.is_some() {
            t.space.xs.max(size / 16.0).round()
        } else {
            0.0
        };
        let text_w = if !has_icon && !text.is_empty() {
            let step = t.type_ramp.sm;
            let m = &self.fonts.for_text(FontRole::Body, text).metrics;
            let w = m.measure_tracked(text, step.size, crate::layout::step_tracking(&step, text));
            (w + 2.0 * t.space.sm + 2.0 * band).ceil()
        } else {
            0.0
        };
        // A round pad whose label is wider grows as a disc; a square one
        // widens and stays its size tall.
        let size = if round { size.max(text_w) } else { size };
        let width = size.max(text_w);
        let i = self.leaf(
            "action",
            id.into(),
            Element {
                width: Size::Fixed(width),
                height: Size::Fixed(size),
                interactive: true,
                radius: if round { size / 2.0 } else { t.radius.control },
                ..Element::default()
            },
        );
        self.nodes[i].props.customize = desc.customize.clone();
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        // A further touch on the button: held while over it, a click when it
        // lifts over it (with the primary pointer busy on a stick).
        if let Some((_, held)) = self.touch(key, false) {
            r.pressed |= held;
            r.clicked |= !held;
        }
        let (bg, ink) = if desc.primary {
            (
                if r.pressed {
                    p.primary_active
                } else {
                    p.primary
                },
                p.on_primary,
            )
        } else {
            (
                if r.pressed {
                    p.control_strong.over(p.hud.over(p.backdrop))
                } else {
                    p.hud
                },
                p.ink,
            )
        };
        let icon = if has_icon {
            self.icon_shape(&desc.icon, Frame::centered(0.0, 0.0, s, s), ink)
        } else {
            None
        };
        let ring = charge.map(|done| {
            // The track a step darker than the pad, the lit part from the top
            // clockwise as halves of the disc (as a cooldown's), then the
            // face over the middle.
            let lit = match desc.charge_tone {
                Tone::Neutral => p.primary,
                tone => self.tone_dot(tone),
            };
            let behind = self.behind(i);
            let track = p.well.over(behind);
            let full = Frame::centered(0.0, 0.0, size, size);
            let mut shapes = vec![Shape::Rect {
                frame: full,
                radius: size,
                color: track,
            }];
            if done >= 1.0 {
                shapes.push(Shape::Rect {
                    frame: full,
                    radius: size,
                    color: lit,
                });
            } else if done > 0.0 {
                let turn = 2.0 * std::f32::consts::PI * done;
                shapes.push(Shape::Half {
                    frame: full,
                    angle: 0.0,
                    color: lit,
                });
                if done <= 0.5 {
                    shapes.push(Shape::Half {
                        frame: Frame::centered(0.0, 0.0, size + 1.0, size + 1.0),
                        angle: turn,
                        color: track,
                    });
                } else {
                    shapes.push(Shape::Half {
                        frame: full,
                        angle: turn - std::f32::consts::PI,
                        color: lit,
                    });
                }
            }
            let face = size - 2.0 * band;
            shapes.push(Shape::Rect {
                frame: Frame::centered(0.0, 0.0, face, face),
                radius: face,
                // Opaque, so the lit ring never shows through a glassy pad.
                color: bg.over(behind),
            });
            shapes
        });
        let n = &mut self.nodes[i];
        // With a ring the face is drawn over it: the element itself is the ring.
        n.el.background = if ring.is_some() { None } else { Some(bg) };
        n.widget = Widget::Button {
            pressed: r.pressed,
            press: if r.pressed { 1.0 } else { 0.0 },
        };
        n.probe_text = text.to_owned();
        n.shapes.extend(ring.into_iter().flatten());
        match icon {
            Some(s) => n.shapes.push(s),
            None => n.shapes.push(Shape::Text {
                frame: Frame::FULL,
                text: text.to_owned(),
                step: t.type_ramp.sm,
                color: ink,
                font: FontRole::Body,
                clip: Frame::FULL,
            }),
        }
        r
    }
}
