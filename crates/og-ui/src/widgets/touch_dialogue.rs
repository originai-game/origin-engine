use super::*;

/// A touch control's disc (a stick's base, a d-pad's plate), as the shipped mobile games draw them: one circle of
/// the HUD ground, translucent so the scene shows through, a rim of the
/// strong line inside its edge and a dark hairline outside it, a soft drop
/// under it. A component's dressable draw: a style dresses its part
/// (`Joystick.base`, `Dpad.base`).
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
        let p = self.env.theme.palette;
        let size = if desc.size > 0.0 {
            desc.size
        } else {
            self.env.theme.controls.joystick_size
        }
        .max(3.0 * self.min_target());
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
        self.tree.props_mut(i).customize = desc.customize;
        self.tree.nodes[i].disabled = desc.disabled;
        let key = self.tree.nodes[i].key;
        let mut out = JoystickResponse::default();
        let radius = size / 2.0;
        let (mut kx, mut ky) = (0.0, 0.0);
        let finger = self.touch_owner(key, desc.disabled);
        if let Some(finger) = finger
            && let Some(h) = self.interaction.hit(key)
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
        // The knob (its rim disc knob + 2 across) stops a pixel inside the
        // base's 2 px rim: pushed full, it leans on the ring, never over it.
        let rim = 2.0;
        let travel = radius - rim - 1.0 - (knob + 2.0) / 2.0;
        let disc = |part, x, y, d, color| Shape::DressedRect {
            frame: Frame::centered(x, y, d, d),
            part: crate::dress::PartRef::of("Joystick", part, None)
                .flag_if(out.active, "active")
                .flag_if(desc.disabled, "disabled"),
            radius: d,
            color,
        };
        let build = self.tree.build();
        let n = &mut self.tree.nodes[i];
        let (kx, ky) = (kx * travel, ky * travel);
        // The knob: a drop, a rim, its face in the knob's colour, a sheen.
        n.shapes.write(build).push(disc(
            "shadow",
            kx,
            ky + 2.0,
            knob + 2.0,
            p.shadow.fade(0.45),
        ));
        n.shapes
            .write(build)
            .push(disc("rim", kx, ky, knob + 2.0, p.line_strong.fade(0.8)));
        n.shapes.write(build).push(disc(
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
        n.shapes.write(build).push(disc(
            "sheen",
            kx,
            ky - knob * 0.16,
            knob * 0.5,
            p.sheen.fade(0.35),
        ));
        // Held, the base's rim lit toward the push (Joystick.direction),
        // after the knob (whose drop reaches the rim), brighter as the push
        // grows: the primary along the edge the thumb leans to, a fifth of
        // the way round.
        if out.active && (kx != 0.0 || ky != 0.0) {
            let lean = (kx * kx + ky * ky).sqrt() / travel.max(1e-3);
            let sweep = 0.4 * std::f32::consts::PI;
            n.shapes.write(build).push(Shape::Arc {
                part: crate::dress::PartRef::of("Joystick", "direction", None)
                    .flag_if(out.active, "active")
                    .flag_if(desc.disabled, "disabled"),
                frame: Frame::FULL,
                start: ky.atan2(kx) - sweep / 2.0,
                sweep,
                thickness: rim + 1.0,
                color: p.primary.fade(0.4 + 0.6 * lean.min(1.0)),
            });
        }
        out
    }

    /// Where the finger that owns touch control `key` is: the primary
    /// pointer while its press that started on it is held, else the further
    /// touch (og.Pointers) it holds, kept while that touch is down whatever
    /// lands on the control after it, else a held touch that went down on
    /// it and no other control holds. A finger that slid onto it from
    /// elsewhere owns nothing.
    fn touch_owner(&mut self, key: u64, disabled: bool) -> Option<og_core::glam::Vec2> {
        let p = self.interaction.input.pointers;
        self.interaction
            .state
            .touch_owners
            .retain(|_, id| p.down(*id));
        if disabled {
            self.interaction.state.touch_owners.remove(&key);
            return None;
        }
        if self.interaction.state.active == key
            && (self.interaction.input.down || self.interaction.input.pressed)
        {
            return Some(self.interaction.input.pointer);
        }
        if let Some(&id) = self.interaction.state.touch_owners.get(&key) {
            return Some(p.position(id));
        }
        let rect = self.interaction.hit(key)?.rect;
        let held = |id: &u32| {
            self.interaction
                .state
                .touch_owners
                .values()
                .any(|v| v == id)
        };
        let id = (1..og_input::MAX_POINTERS as u32).find(|id| {
            let at = p.start(*id);
            p.down(*id) && !held(id) && rect.contains(at.x, at.y)
        })?;
        self.interaction.state.touch_owners.insert(key, id);
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
        let p = self.env.theme.palette;
        let arm = (if desc.size > 0.0 {
            desc.size
        } else {
            self.env.theme.controls.dpad_size
        } / 3.0)
            .max(self.min_target());
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
        self.tree.props_mut(pad).customize = desc.customize;
        self.tree.nodes[pad].disabled = desc.disabled;
        let key = self.tree.nodes[pad].key;
        let mut out = DpadResponse::default();
        if let Some(finger) = self.touch_owner(key, desc.disabled)
            && let Some(h) = self.interaction.hit(key)
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
        self.tree.stack.push(pad);
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
                self.tree.nodes[i].widget = Widget::Spacer;
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
            let build = self.tree.build();
            let n = &mut self.tree.nodes[i];
            if held {
                // The held direction lights under its chevron.
                n.shapes.write(build).push(Shape::DressedRect {
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
                n.shapes.write(build).push(Shape::Bar {
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
}
