use super::*;

impl Ui {
    /// Text in MSDF, wrapping at its width unless `nowrap`.
    pub fn label(&mut self, id: impl Into<Id>, text: &str, desc: LabelDesc) {
        let t = self.theme;
        // A centred or end-aligned label left to fit its text in a column
        // takes the column's width, as a CSS block does: aligned in a box
        // only as wide as its text, it would sit at the column's start.
        let in_column = self
            .stack
            .last()
            .is_some_and(|&p| self.nodes[p].el.direction == Direction::Column);
        let width = if desc.align != TextAlign::Start && desc.width == Size::fit() && in_column {
            Size::grow()
        } else {
            desc.width
        };
        let i = self.leaf(
            "label",
            id.into(),
            Element {
                width,
                ..Element::default()
            },
        );
        let step = desc.step.unwrap_or(t.type_ramp.body);
        let fit = desc.fit_min.map_or_else(Vec::new, |min| {
            crate::style_file::fit_steps(&t.type_ramp, step, min)
        });
        let n = &mut self.nodes[i];
        n.content = Content::Text(TextContent {
            text: text.to_owned(),
            step,
            color: desc.color.unwrap_or(t.palette.ink),
            align: desc.align,
            wrap: !desc.nowrap,
            font: desc.font,
            ellipsis: desc.ellipsis,
            max_lines: desc.max_lines,
            emphasis: 0.0,
            fit,
        });
        // The probe and screen readers read the whole text, cut or not.
        n.probe_text = text.to_owned();
        n.heading = desc.heading;
        // Room across for the style's drop inside a scroll view, whose edge
        // clips it (pixel's results figures lost their last font pixel of
        // drop to the view's edge); elsewhere a drop draws past its box, and
        // reserving room there would change the layout of every large label.
        let grid = match &self.nodes[i].content {
            Content::Text(tc) => tc.metrics(&self.fonts).pixel_grid(),
            _ => 0,
        };
        let scrolled = self
            .stack
            .iter()
            .any(|&p| self.nodes[p].el.scroll || self.nodes[p].el.scroll_x);
        let in_control = self
            .stack
            .last()
            .is_some_and(|&p| self.nodes[p].el.interactive);
        if let Some((dx, _, blur, _)) = self
            .text_drop(desc.font.effect_index(), step.size, grid)
            .filter(|_| scrolled && !in_control)
        {
            let reach = (dx.abs() + blur / 2.0).ceil();
            let pad = &mut self.nodes[i].el.padding;
            if dx > 0.0 {
                pad.right = pad.right.max(reach);
            } else if dx < 0.0 {
                pad.left = pad.left.max(reach);
            }
        }
    }

    /// A capsule button with a label. `clicked` is a press and a release inside it.
    pub fn button(&mut self, id: impl Into<Id>, text: &str, desc: ButtonDesc) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        // Never shorter than its words with an inset above and below: a
        // style whose button step outgrows its control size (pixel's 24
        // on a 32 control) grows the plate rather than filling it edge to edge.
        let words = self.type_step(crate::tokens::TypeRole::Button).line_height + 2.0 * t.space.xs;
        let h = match desc.size {
            ControlSize::Small => sizes.control_sm,
            ControlSize::Medium => sizes.control_md,
            ControlSize::Large => sizes.control_lg,
        }
        .max(words);
        let pad = if desc.size == ControlSize::Small {
            t.space.md
        } else {
            t.space.lg
        };
        let (width, pad) = if desc.square {
            (Size::Fixed(h), 0.0)
        } else {
            (desc.width.unwrap_or_default(), pad)
        };
        let i = self.leaf(
            "button",
            id.into(),
            Element {
                width,
                height: Size::Fixed(h),
                padding: Insets::xy(pad, 0.0),
                // A style that shakes a refused press hears presses on a disabled button.
                interactive: !desc.disabled || t.art.motion.shake > 0.0,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.nodes[i].disabled = desc.disabled;
        let refused =
            desc.disabled && t.art.motion.shake > 0.0 && self.respond(self.nodes[i].key).clicked;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        // What opens takes the focus (as a dialog's first control and a
        // dialogue box do): appearing, outside a dialog.
        let key = self.nodes[i].key;
        if desc.autofocus
            && !desc.disabled
            && self.state.scope == 0
            && !self.state.hits.iter().any(|h| h.key == key)
        {
            self.request_focus = key;
        }
        let p = &t.palette;
        let amounts = if desc.disabled {
            (0.0, 0.0)
        } else {
            self.state_amounts(i, &r)
        };
        // Each variant's colour at rest, hovered and pressed, mixed by how
        // far into each the button is (the style's `ui` spring).
        let mix = |rest, hover, pressed| Ui::state_color(rest, hover, pressed, amounts);
        let (bg, fg) = if desc.disabled {
            (Some(p.control), p.ink_faint)
        } else {
            match desc.variant {
                ButtonVariant::Primary => (
                    Some(mix(p.primary, p.primary_hover, p.primary_active)),
                    p.on_primary,
                ),
                ButtonVariant::Secondary => (
                    Some(mix(p.control, p.control_strong, p.control_strong)),
                    p.ink,
                ),
                // Ghost is quiet's fill with a strong outline (the token
                // stroke of a `ghost` part), so it reads as a button at rest.
                ButtonVariant::Quiet | ButtonVariant::Ghost => {
                    let clear = p.control.fade(0.0);
                    let c = mix(clear, p.control, p.control);
                    ((c.alpha > 0.0).then_some(c), p.ink)
                }
                // Danger ink on the danger tint (10 % at rest, 15 %
                // hovered), composited in sRGB over what is behind it, so its label
                // reads at 4.5:1 on every ground.
                ButtonVariant::Danger => {
                    let behind = self.behind(i);
                    let rest = p.danger.fade(0.10).over(behind);
                    let lit = p.danger.fade(0.15).over(behind);
                    (Some(mix(rest, lit, lit)), p.danger)
                }
            }
        };
        let variant = match desc.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Quiet => "quiet",
            ButtonVariant::Danger => "danger",
            ButtonVariant::Ghost => "ghost",
        };
        let mut part =
            crate::dress::PartRef::new("Button", "base", crate::tokens::ElementClass::Button)
                .flag(variant)
                .flag_if(desc.disabled, "disabled")
                .flag_if(!desc.disabled && r.pressed, "pressed")
                .flag_if(!desc.disabled && r.hovered && !r.pressed, "hover")
                .flag_if(r.focused, "focused")
                .flag_if(desc.size == ControlSize::Small, "small")
                .flag_if(desc.size == ControlSize::Large, "large")
                .flag_if(desc.square, "square")
                .flag_if(self.narrow_button(key, width), "narrow");
        part.hover = amounts.0;
        part.press = amounts.1;
        let resolved = self.dress.resolve(&part);
        let slice = resolved.frame.slice.filter(|_| t.art.atlas.is_some());
        // A style's tinted button art darkens (or fades) the fill: the label
        // reads at 7:1 on the fill as drawn (the packs' bar, which keeps 4.5:1
        // in the frame once MSDF strokes thin it), so a game's own primary colour over a
        // dark neon body keeps a light label instead of the flat token's ink.
        // A fill in the middle of the scale (a game's crimson) that neither
        // end of the scale reads 7:1 on moves away from the better ink, as
        // little as it takes, so the colour stays the game's. A dressing
        // that paints its own fills (a gradient) is read the same way, by
        // the mean of its paints.
        let painted = slice.is_none() && !resolved.draw.fills.is_empty();
        let (bg, fg) = match (bg, slice) {
            // Only art whose centre the style measured (`tone`): untoned
            // art is the style's to judge.
            (Some(fill), Some(s))
                if s.tone != Slice::WHITE
                    && !desc.disabled
                    && desc.variant != ButtonVariant::Danger =>
            {
                let behind = self.behind(i);
                let drawn = |c: Srgba| s.drawn(c).unwrap_or(c).over_linear(behind);
                let bar = crate::style::BUTTON_READABLE;
                match fg.readable_on(drawn(fill), bar) {
                    Some(ink) => (Some(fill), ink),
                    None => {
                        let ground = drawn(fill);
                        let ink = if p.ink.contrast(ground) >= p.ink_inverse.contrast(ground) {
                            p.ink
                        } else {
                            p.ink_inverse
                        };
                        let away = if ink.luminance() > ground.luminance() {
                            Srgba::BLACK
                        } else {
                            Srgba::WHITE
                        };
                        let moved = (1..=20)
                            .map(|n| fill.mix(away, n as f32 * 0.05))
                            .find(|c| ink.contrast(drawn(*c)) >= bar)
                            .unwrap_or(away);
                        (Some(moved), ink)
                    }
                }
            }
            (fill, None) if painted && !desc.disabled => {
                let behind = self.behind(i);
                let d = resolved.element_draw(&part, &t, fill);
                let ground = crate::dress::ground_of(&d, &t, behind);
                let bar = crate::style::BUTTON_READABLE;
                let ink = fg
                    .readable_on(ground, bar)
                    .or_else(|| {
                        let other = if fg == p.ink { p.ink_inverse } else { p.ink };
                        other.readable_on(ground, bar)
                    })
                    .unwrap_or(fg);
                (fill, ink)
            }
            _ => (bg, fg),
        };
        let mut content = self.role_text(text, crate::tokens::TypeRole::Button, fg);
        content.align = TextAlign::Center;
        let press = self.button_motion(i, r, refused);
        if r.clicked && desc.variant == ButtonVariant::Primary {
            self.feedback(crate::tokens::SoundEvent::Confirm);
        }
        let radius = crate::dress::corner_of(resolved.shape(&part, &t)).1;
        let n = &mut self.nodes[i];
        n.el.background = bg;
        n.el.radius = radius.min(h / 2.0);
        n.el.part = Some(part);
        n.el.frame = None;
        n.content = Content::Text(content);
        n.widget = Widget::Button {
            pressed: r.pressed,
            press,
        };
        n.probe_text = text.to_owned();
        r
    }

    /// A texture region, or a solid rectangle without a texture; with
    /// `aspect`, a fitting side follows the other.
    pub fn image(&mut self, id: impl Into<Id>, desc: ImageDesc) {
        let i = self.leaf(
            "image",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                aspect: desc.aspect.max(0.0),
                ..Element::default()
            },
        );
        self.nodes[i].content = Content::Image {
            texture: desc.texture,
            uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
            tint: desc.tint.unwrap_or(Srgba::WHITE),
        };
        self.nodes[i].probe_text = desc.label;
    }

    /// A capsule track filled to `value` (0–1).
    pub fn progress(&mut self, id: impl Into<Id>, value: f32, desc: ProgressDesc) {
        let t = self.theme;
        let i = self.leaf(
            "progress",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(t.space.sm),
                ..Element::default()
            },
        );
        let mut v = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let ms = t.art.motion.progress_ms;
        if ms > 0.0 && !t.reduced_motion {
            // Eases toward the value: a constant share of the gap per tick.
            let key = child_key(self.nodes[i].key, "ease");
            let fresh = !self.has_anim(key);
            let mut a = self.anim(key);
            if fresh {
                a.a = v;
            } else {
                let k = (4.0 / self.ticks_for(ms) as f32).min(1.0);
                a.a += (v - a.a) * k;
                if (v - a.a).abs() < 0.002 {
                    a.a = v;
                }
            }
            self.set_anim(key, a);
            v = a.a;
        }
        self.nodes[i].widget = Widget::Progress { value: v };
    }

    /// A button's style motion: the hover lift and the press squash on the
    /// style's springs (`motion.hoverSpring`, `motion.pressSpring`), and the shake of a refused press (`motion.duration.slow`). Answers
    /// how far in the press is (0 at rest, 1 held; past either while the
    /// spring overshoots).
    fn button_motion(&mut self, i: usize, r: Response, refused: bool) -> f32 {
        let t = self.theme;
        let art = t.art;
        if art.hover.is_none() && art.press.is_none() && art.motion.shake <= 0.0 {
            return if r.pressed { 1.0 } else { 0.0 };
        }
        let key = self.nodes[i].key;
        let hover_spring = art
            .motion
            .hover_spring
            .unwrap_or(crate::theme::SPRING_DEFAULT);
        let press_spring = art.motion.press_spring.unwrap_or(crate::theme::Spring {
            damping: 0.55,
            response_ms: t.durations_ms.instant,
        });
        // Each spring is a motion channel (docs/ui.md §7.2): a retarget
        // keeps its value and velocity, so an interrupted hover never jumps.
        let step = |ui: &mut Ui, name: &str, spring: crate::theme::Spring, target: f32| -> f32 {
            let k = child_key(key, name);
            let spec = crate::motion::SpringSpec::from_theme(spring);
            let fresh = !ui.state.motion.channels.contains_key(&k);
            let from = if fresh && target != 0.0 {
                Some(0.0)
            } else {
                None
            };
            ui.animate_key(k, target, crate::motion::Transition::Spring(spec), from)
        };
        let hover = step(
            self,
            "hover",
            hover_spring,
            if r.hovered { 1.0 } else { 0.0 },
        );
        let press = step(
            self,
            "press",
            press_spring,
            if r.pressed { 1.0 } else { 0.0 },
        );
        let key = child_key(key, "motion");
        let mut a = self.anim(key);
        let now = self.state.tick;
        if refused {
            a.t = now;
            a.b = 1.0;
            self.sound(crate::style_file::UiSound::Error);
        }
        self.set_anim(key, a);
        if t.reduced_motion {
            return if r.pressed { 1.0 } else { 0.0 };
        }
        let mut f = Fx::default();
        if let Some(h) = art.hover {
            f.dy += h.dy * hover;
            f.scale = 1.0 + (h.scale - 1.0) * hover;
        }
        if let Some(p) = art.press {
            f.dy += p.dy * press;
        }
        if a.b > 0.0 {
            let k = (now - a.t) as f32 / self.ticks_for(t.durations_ms.slow) as f32;
            if k < 1.0 {
                // Three decaying swings, arithmetic only (a triangle wave).
                let x = k * 3.0;
                let tri = 4.0 * (x - x.floor() - 0.5).abs() - 1.0;
                f.dx = art.motion.shake * (1.0 - k) * tri;
            }
        }
        if f != Fx::default() {
            self.nodes[i].fx = Some(f);
        }
        press
    }
}

impl Ui {
    /// A horizontal slider over `value`; dragging anywhere on it sets the value.
    pub fn slider(&mut self, id: impl Into<Id>, value: &mut f32, desc: SliderDesc) -> Response {
        let sizes = self.sizes();
        let i = self.leaf(
            "slider",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(sizes.control_sm),
                interactive: true,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let mut r = self.nav_target(i, true);
        let (lo, hi) = if desc.min == desc.max {
            (0.0, 1.0)
        } else {
            (desc.min, desc.max)
        };
        if r.focused && self.nav.adjust != 0 {
            // Left and right step by `step`, or a twentieth of the range.
            let unit = if desc.step > 0.0 {
                desc.step
            } else {
                (hi - lo) / 20.0
            };
            let v = (*value + unit * self.nav.adjust as f32).clamp(lo.min(hi), lo.max(hi));
            if v != *value {
                *value = v;
                r.changed = true;
            }
        }
        let thumb = self.thumb();
        if self.state.active == key
            && (self.input.down || self.input.pressed)
            && let Some(h) = self.hit(key)
        {
            let travel = (h.rect.width - thumb).max(1.0);
            let t = ((self.input.pointer.x - h.rect.x - thumb / 2.0) / travel).clamp(0.0, 1.0);
            let mut v = lo + t * (hi - lo);
            if desc.step > 0.0 {
                v = lo + ((v - lo) / desc.step).round() * desc.step;
                v = v.clamp(lo.min(hi), lo.max(hi));
            }
            if v != *value {
                *value = v;
                r.changed = true;
            }
        }
        let t = if hi != lo {
            ((*value - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.nodes[i].widget = Widget::Slider {
            t: if t.is_finite() { t } else { 0.0 },
            active: r.pressed,
        };
        // Its value as people read it: the step's decimals, at most two.
        let decimals = if desc.step >= 1.0 || (hi - lo).abs() > 20.0 {
            0
        } else {
            2
        };
        self.nodes[i].a11y.value = Some(format!("{:.*}", decimals, *value));
        self.nodes[i].a11y.range = Some(crate::a11y::A11yRange {
            min: Some(lo.min(hi)),
            max: Some(lo.max(hi)),
            now: *value,
        });
        r
    }

    pub(crate) fn thumb(&self) -> f32 {
        self.sizes().control_sm - 2.0 * self.theme.space.xs
    }

    pub(crate) fn switch_size(&self) -> (f32, f32) {
        let h = self.sizes().control_sm - self.theme.space.xs;
        // The switch's 51:31 proportion (geometry, not a design value).
        (h * 51.0 / 31.0, h)
    }

    /// A switch with a label; a click flips `on`.
    pub fn toggle(&mut self, id: impl Into<Id>, on: &mut bool, label: &str) -> Response {
        let t = self.theme;
        let sizes = self.sizes();
        let (sw, _) = self.switch_size();
        let i = self.leaf(
            "toggle",
            id.into(),
            Element {
                height: Size::Fixed(sizes.control_sm),
                padding: Insets {
                    left: sw + t.space.sm,
                    ..Insets::default()
                },
                interactive: true,
                ..Element::default()
            },
        );
        let mut r = self.nav_target(i, false);
        if r.clicked {
            *on = !*on;
            r.changed = true;
            self.feedback(if *on {
                crate::tokens::SoundEvent::ToggleOn
            } else {
                crate::tokens::SoundEvent::ToggleOff
            });
        }
        let key = self.nodes[i].key;
        let slide = self.spring_to(child_key(key, "slide"), if *on { 1.0 } else { 0.0 });
        let content = self.role_text(label, crate::tokens::TypeRole::Body, t.palette.ink);
        let n = &mut self.nodes[i];
        n.content = Content::Text(content);
        n.widget = Widget::Toggle { on: *on, slide };
        n.probe_text = label.to_owned();
        r
    }
}
