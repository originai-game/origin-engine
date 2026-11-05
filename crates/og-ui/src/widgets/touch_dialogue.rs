use super::*;

/// How opaque a disabled touch control draws (its look stays, dimmed).
const DISABLED_OPACITY: f32 = 0.45;

/// A touch control's disc (a stick's base, a d-pad's plate, a skill
/// button's face), as the shipped mobile games draw them: one circle of
/// the HUD ground, translucent so the scene shows through, a rim of the
/// strong line inside its edge and a dark hairline outside it, a soft drop
/// under it. A component's dressable draw: a style dresses its part
/// (`Joystick.base`, `Dpad.base`, `ActionButton.face`).
fn touch_disc(
    fill: crate::paint::ColorRef,
    rim: crate::paint::ColorRef,
    rim_width: f32,
) -> crate::paint::ElementDraw {
    use crate::paint::{Cap, ColorRef, Effect, ElementDraw, Paint, ShapeKind, Stroke, StrokeAlign};
    let line = |width: f32, align: StrokeAlign, color: ColorRef| Stroke {
        width,
        align,
        dash: [0.0; 2],
        dash_offset: 0.0,
        dash_cap: Cap::Butt,
        paint: Paint::Solid(color),
    };
    ElementDraw {
        shape: ShapeKind::Ellipse,
        fills: vec![Paint::Solid(fill)],
        strokes: vec![
            line(rim_width, StrokeAlign::Inside, rim),
            line(1.0, StrokeAlign::Outside, tok("shadow", 0.5)),
        ],
        effects: vec![Effect::DropShadow {
            offset: [0.0, 2.0],
            blur: 8.0,
            spread: 0.0,
            color: tok("shadow", 0.35),
        }],
        ..ElementDraw::default()
    }
}

fn tok(name: &str, alpha: f32) -> crate::paint::ColorRef {
    crate::paint::ColorRef::Token {
        name: name.to_string(),
        alpha,
    }
}

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
                    crate::dress::PartRef::of("Dialogue", "portraitIcon", None),
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
                        part: crate::dress::PartRef::of("Dialogue", "initial", None),
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
                    part: crate::dress::PartRef::of("Dialogue", "continue", None)
                        .attr("side", if side < 0.0 { "left" } else { "right" }),
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

    /// A virtual joystick: one translucent disc of the HUD ground with a
    /// rim (part `Joystick.base`, which a style dresses) and a knob that
    /// follows the finger that grabbed it, clamped to the base, the rim lit
    /// toward the push while held. Answers the stick's -1..1 vector, zero
    /// inside the dead zone and rescaled past it, so it rises from 0 at the
    /// dead zone's edge to 1 at the base's. It keeps the finger that started
    /// on it (the primary pointer, or any of og.Pointers), so another finger
    /// can hold a button at once; one that slides onto it from elsewhere
    /// does not take it. Disabled, it answers nothing and shows faded.
    pub fn joystick(&mut self, id: impl Into<Id>, desc: JoystickDesc) -> JoystickResponse {
        let p = self.theme.palette;
        let size = if desc.size > 0.0 { desc.size } else { 128.0 }.max(3.0 * self.min_target());
        let i = self.leaf(
            "joystick",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: !desc.disabled,
                radius: size / 2.0,
                part: Some(
                    crate::dress::PartRef::new(
                        "Joystick",
                        "base",
                        crate::tokens::ElementClass::Hud,
                    )
                    .flag_if(desc.disabled, "disabled"),
                ),
                ..Element::default()
            },
        );
        // One translucent ring the thumb rests in, not a plate.
        self.set_draw_dressable(touch_disc(tok("hud", 0.42), tok("lineStrong", 0.55), 2.0));
        self.nodes[i].props.customize = desc.customize;
        self.nodes[i].disabled = desc.disabled;
        let key = self.nodes[i].key;
        let mut out = JoystickResponse::default();
        let radius = size / 2.0;
        let (mut kx, mut ky) = (0.0, 0.0);
        let finger = self.touch_owner(key, desc.disabled);
        if let Some(finger) = finger
            && let Some(h) = self.hit(key)
        {
            let (cx, cy) = h.rect.center();
            let (dx, dy) = ((finger.x - cx) / radius, (finger.y - cy) / radius);
            let len = (dx * dx + dy * dy).sqrt();
            let mag = len.min(1.0);
            (kx, ky) = if len > 1.0 {
                (dx / len, dy / len)
            } else {
                (dx, dy)
            };
            let dz = if desc.dead_zone > 0.0 {
                desc.dead_zone.min(0.95)
            } else {
                0.15
            };
            if mag > dz {
                let k = (mag - dz) / (1.0 - dz) / mag;
                out.x = kx * k;
                out.y = ky * k;
            }
            out.active = true;
        }
        let knob = size * 0.4;
        let travel = radius - knob / 2.0 - 2.0;
        let disc = |part, x, y, d, color| Shape::DressedRect {
            frame: Frame::centered(x, y, d, d),
            part: crate::dress::PartRef::of("Joystick", part, None)
                .flag_if(out.active, "active")
                .flag_if(desc.disabled, "disabled"),
            radius: d,
            color,
        };
        let n = &mut self.nodes[i];
        if desc.disabled {
            n.props.opacity = DISABLED_OPACITY;
        }
        // Held, a lit arc of the rim toward the push: the primary along the
        // edge the thumb leans to.
        let (kx, ky) = (kx * travel, ky * travel);
        if out.active && (kx != 0.0 || ky != 0.0) {
            let len = (kx * kx + ky * ky).sqrt().max(1e-3);
            let (ux, uy) = (kx / len, ky / len);
            let at = radius - knob * 0.35;
            n.shapes.push(disc(
                "direction",
                ux * at,
                uy * at,
                knob * 0.7,
                p.primary.fade(0.35),
            ));
        }
        // The knob: a drop, a rim, its face in the knob's colour, a sheen.
        n.shapes.push(disc(
            "shadow",
            kx,
            ky + 2.0,
            knob + 2.0,
            p.shadow.fade(0.45),
        ));
        n.shapes
            .push(disc("rim", kx, ky, knob + 2.0, p.line_strong.fade(0.8)));
        n.shapes.push(disc(
            "knob",
            kx,
            ky,
            knob - 2.0,
            if out.active {
                p.knob
            } else {
                p.knob.mix(p.hud, 0.25).fade(0.92)
            },
        ));
        n.shapes.push(disc(
            "sheen",
            kx,
            ky - knob * 0.16,
            knob * 0.5,
            p.sheen.fade(0.35),
        ));
        out
    }

    /// Where the finger that owns touch control `key` is: the primary
    /// pointer while its press that started on it is held, else the further
    /// touch (og.Pointers) it holds, kept while that touch is down whatever
    /// lands on the control after it, else a held touch that went down on
    /// it and no other control holds. A finger that slid onto it from
    /// elsewhere owns nothing.
    fn touch_owner(&mut self, key: u64, disabled: bool) -> Option<og_core::glam::Vec2> {
        let p = self.input.pointers;
        self.state.touch_owners.retain(|_, id| p.down(*id));
        if disabled {
            self.state.touch_owners.remove(&key);
            return None;
        }
        if self.state.active == key && (self.input.down || self.input.pressed) {
            return Some(self.input.pointer);
        }
        if let Some(&id) = self.state.touch_owners.get(&key) {
            return Some(p.position(id));
        }
        let rect = self.hit(key)?.rect;
        let held = |id: &u32| self.state.touch_owners.values().any(|v| v == id);
        let id = (1..og_input::MAX_POINTERS as u32).find(|id| {
            let at = p.start(*id);
            p.down(*id) && !held(id) && rect.contains(at.x, at.y)
        })?;
        self.state.touch_owners.insert(key, id);
        Some(p.position(id))
    }

    /// A d-pad: one round pad (part `Dpad.base`, which a style dresses)
    /// with four chevrons on it (`Up`, `Down`, `Left`, `Right`, each at
    /// least the touch target), the held direction lit under its chevron.
    /// The whole pad is one target, read by the angle of the finger from
    /// its centre: eight ways (a diagonal holds both of its arms), nothing
    /// in its middle, and the thumb rolling around it moves between them
    /// without lifting. It keeps the finger that started on it, as the
    /// joystick does; disabled, it answers nothing and shows faded.
    pub fn dpad(&mut self, id: impl Into<Id>, desc: DpadDesc) -> DpadResponse {
        let p = self.theme.palette;
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
                interactive: !desc.disabled,
                radius: arm * 1.5,
                part: Some(
                    crate::dress::PartRef::new("Dpad", "base", crate::tokens::ElementClass::Hud)
                        .flag_if(desc.disabled, "disabled"),
                ),
                ..Element::default()
            },
        );
        // One round pad with four directions on it (a controller's cross
        // under the thumb), not four keys.
        self.set_draw_dressable(touch_disc(tok("hud", 0.42), tok("lineStrong", 0.55), 2.0));
        self.nodes[pad].props.customize = desc.customize;
        self.nodes[pad].disabled = desc.disabled;
        if desc.disabled {
            self.nodes[pad].props.opacity = DISABLED_OPACITY;
        }
        let key = self.nodes[pad].key;
        let mut out = DpadResponse::default();
        if let Some(finger) = self.touch_owner(key, desc.disabled)
            && let Some(h) = self.hit(key)
        {
            let (cx, cy) = h.rect.center();
            let (dx, dy) = (finger.x - cx, finger.y - cy);
            // The middle is no direction; outside it, the nearest of eight.
            if dx * dx + dy * dy > (arm * 0.35) * (arm * 0.35) {
                let way = (dy.atan2(dx) / FRAC_PI_4).round().rem_euclid(8.0) as u8;
                out.right = matches!(way, 7 | 0 | 1);
                out.down = matches!(way, 1..=3);
                out.left = matches!(way, 3..=5);
                out.up = matches!(way, 5..=7);
            }
        }
        self.stack.push(pad);
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
        for cell in cells.iter() {
            let el = Element {
                width: Size::Fixed(arm),
                height: Size::Fixed(arm),
                ..Element::default()
            };
            let Some((name, angle)) = cell else {
                let i = self.leaf("spacer", Id::default(), el);
                self.nodes[i].widget = Widget::Spacer;
                continue;
            };
            let i = self.leaf("button", Id::from(*name), el);
            let held = match *name {
                "Up" => out.up,
                "Down" => out.down,
                "Left" => out.left,
                _ => out.right,
            };
            let ink = if held { p.on_primary } else { p.ink.fade(0.85) };
            let n = &mut self.nodes[i];
            if held {
                // The held direction lights under its chevron.
                n.shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Dpad", "indicator", None)
                        .flag("pressed")
                        .attr("direction", name),
                    frame: Frame::centered(0.0, 0.0, arm * 0.78, arm * 0.78),
                    radius: arm * 0.78,
                    color: p.primary.fade(0.85),
                });
            }
            n.probe_text = (*name).to_owned();
            // A chevron pointing `angle`: two strokes meeting at its tip.
            let len = arm * 0.24;
            let (ca, sa) = (cos(*angle), sin(*angle));
            let tip = (ca * len * 0.35, sa * len * 0.35);
            for side in [-1.0f32, 1.0] {
                let back = *angle + side * 3.0 * FRAC_PI_4;
                let (ox, oy) = (tip.0 + cos(back) * len * 0.5, tip.1 + sin(back) * len * 0.5);
                n.shapes.push(Shape::Bar {
                    part: crate::dress::PartRef::of("Dpad", "chevron", None)
                        .flag_if(held, "pressed")
                        .attr("direction", name)
                        .attr("side", if side < 0.0 { "left" } else { "right" }),
                    frame: Frame::centered(ox, oy, len, 3.0),
                    angle: back,
                    color: ink,
                });
            }
        }
        self.close();
        out
    }

    /// A touch action button (jump, fire, a skill): a disc with a rim
    /// (part `ActionButton.face`, flagged `primary`, `pressed` and
    /// `disabled`, which a style dresses), its icon on it; a label wider
    /// than the button with no icon widens it to a capsule instead (a
    /// longer locale, 200 % text), so it is never cut. At least the touch
    /// target; `pressed` while held, `clicked` on a tap. A further touch
    /// (og.Pointers) holds it when it went down on it, so a thumb steering
    /// a stick never fires it by sliding over, and clicks when it lifts
    /// over it. Disabled, it answers nothing and shows faded.
    pub fn action_button(
        &mut self,
        id: impl Into<Id>,
        text: &str,
        desc: &ActionButtonDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 64.0 }.max(self.min_target());
        // An icon the atlas has draws in place of the label; else the label.
        let s = size * 0.45;
        let has_icon = !desc.icon.is_empty() && self.has_icon(&desc.icon);
        let charge = desc.charge.map(|c| {
            if c.is_finite() {
                c.clamp(0.0, 1.0)
            } else {
                0.0
            }
        });
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
        // A disc unless a label is wider than it: then a capsule its size
        // tall, or (`round`, a charge ring) a disc grown to the label.
        let round = desc.round || charge.is_some() || text_w <= size;
        let size = if round { size.max(text_w) } else { size };
        let width = size.max(text_w);
        let i = self.leaf(
            "action",
            id.into(),
            Element {
                width: Size::Fixed(width),
                height: Size::Fixed(size),
                interactive: !desc.disabled,
                radius: size / 2.0,
                ..Element::default()
            },
        );
        self.nodes[i].props.customize = desc.customize.clone();
        self.nodes[i].disabled = desc.disabled;
        let key = self.nodes[i].key;
        let mut r = if desc.disabled {
            Response::default()
        } else {
            self.respond(key)
        };
        // A further touch that went down on the button: held while down, a
        // click when it lifts over it (the primary pointer busy on a stick).
        if !desc.disabled
            && let Some((at, held)) = self.touch(key, true)
        {
            r.pressed |= held;
            r.clicked |= !held && self.hit(key).is_some_and(|h| h.rect.contains(at.x, at.y));
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
                    p.hud.fade(0.62)
                },
                p.ink,
            )
        };
        let icon = if has_icon {
            self.icon_shape(
                crate::dress::PartRef::of("ActionButton", "icon", None)
                    .flag_if(desc.disabled, "disabled")
                    .flag_if(r.pressed, "pressed"),
                &desc.icon,
                Frame::centered(0.0, 0.0, s, s),
                ink,
            )
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
            let mut shapes = vec![Shape::DressedRect {
                part: crate::dress::PartRef::of("ActionButton", "track", None),
                frame: full,
                radius: size,
                color: track,
            }];
            if done >= 1.0 {
                shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("ActionButton", "sweep", None).flag("ready"),
                    frame: full,
                    radius: size,
                    color: lit,
                });
            } else if done > 0.0 {
                let turn = 2.0 * std::f32::consts::PI * done;
                shapes.push(Shape::Half {
                    part: crate::dress::PartRef::of("ActionButton", "sweep", None),
                    frame: full,
                    angle: 0.0,
                    color: lit,
                });
                if done <= 0.5 {
                    shapes.push(Shape::Half {
                        part: crate::dress::PartRef::of("ActionButton", "mask", None),
                        frame: Frame::centered(0.0, 0.0, size + 1.0, size + 1.0),
                        angle: turn,
                        color: track,
                    });
                } else {
                    shapes.push(Shape::Half {
                        part: crate::dress::PartRef::of("ActionButton", "sweep", None),
                        frame: full,
                        angle: turn - std::f32::consts::PI,
                        color: lit,
                    });
                }
            }
            let face = size - 2.0 * band;
            shapes.push(Shape::DressedRect {
                part: crate::dress::PartRef::of("ActionButton", "core", None)
                    .flag_if(r.pressed, "pressed")
                    .flag_if(desc.disabled, "disabled"),
                frame: Frame::centered(0.0, 0.0, face, face),
                radius: face,
                // Opaque, so the lit ring never shows through a glassy pad.
                color: bg.over(behind),
            });
            shapes
        });
        // The face: the button's ground with a rim (a bright one on the
        // primary), a dressable part; with a charge ring the ring and the
        // face over it are its shapes, and the draw keeps only the outer
        // hairline and the drop.
        let mut face = touch_disc(
            crate::paint::ColorRef::Literal(bg),
            if desc.primary {
                tok("onPrimary", 0.35)
            } else {
                tok("lineStrong", 0.6)
            },
            2.0,
        );
        if !round {
            face.shape = crate::paint::ShapeKind::default();
        }
        if ring.is_some() {
            face.fills.clear();
            face.strokes.remove(0);
        }
        self.nodes[i].el.part = Some(
            crate::dress::PartRef::new("ActionButton", "face", crate::tokens::ElementClass::Hud)
                .flag_if(desc.primary, "primary")
                .flag_if(r.pressed, "pressed")
                .flag_if(desc.disabled, "disabled"),
        );
        self.draws.insert(i, face);
        self.dressable.insert(i);
        let n = &mut self.nodes[i];
        if desc.disabled {
            n.props.opacity = DISABLED_OPACITY;
        }
        n.widget = Widget::Button {
            pressed: r.pressed,
            press: if r.pressed { 1.0 } else { 0.0 },
        };
        n.probe_text = text.to_owned();
        n.shapes.extend(ring.into_iter().flatten());
        match icon {
            Some(s) => n.shapes.push(s),
            None => n.shapes.push(Shape::Text {
                part: crate::dress::PartRef::of("ActionButton", "label", None)
                    .flag_if(desc.disabled, "disabled")
                    .flag_if(r.pressed, "pressed"),
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
