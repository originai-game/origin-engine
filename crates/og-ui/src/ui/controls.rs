use super::*;

impl Ui {
    /// Text in MSDF, wrapping at its width unless `nowrap`.
    pub fn label(&mut self, id: impl Into<Id>, text: &str, desc: LabelDesc) {
        let i = self.leaf("label", id.into(), Element::default());
        self.apply_label(i, text, desc);
    }

    /// Replaces label-owned presentation without allocating a node or processing input.
    pub(crate) fn apply_label(&mut self, i: usize, text: &str, desc: LabelDesc) {
        // A centred or end-aligned label left to fit its text in a column
        // takes the column's width, as a CSS block does: aligned in a box
        // only as wide as its text, it would sit at the column's start.
        let parent = self.nodes[i].parent;
        let in_column = self
            .nodes
            .get(parent)
            .is_some_and(|p| p.el.direction == Direction::Column);
        let width = if desc.align != TextAlign::Start && desc.width == Size::fit() && in_column {
            Size::grow()
        } else {
            desc.width
        };
        self.nodes[i].el.width = width;
        self.nodes[i].el.padding = Insets::default();
        self.draws.remove(&i);
        let ink = self.content_ink();
        // Borrowed, not copied: a label is the most frequent element.
        let t = &self.theme;
        let mut step = desc.step.unwrap_or(t.type_ramp.body);
        step.uppercase |= desc.uppercase;
        let fit = desc.fit_min.map_or_else(Vec::new, |min| {
            crate::style_file::fit_steps(&t.type_ramp, step, min)
        });
        let n = &mut self.nodes[i];
        n.content = Content::Text(TextContent {
            text: text.to_owned(),
            step,
            color: desc.color.unwrap_or(if desc.on_scene {
                t.palette.scene_ink
            } else {
                ink
            }),
            align: desc.align,
            wrap: !desc.nowrap,
            font: desc.font,
            ellipsis: desc.ellipsis,
            max_lines: desc.max_lines,
            emphasis: 0.0,
            fit,
            wrap_style: Default::default(),
            cut: None,
            vertical: None,
        });
        // The probe and screen readers read the whole text, cut or not.
        n.probe_text = text.to_owned();
        n.heading = desc.heading;
        if desc.on_scene {
            use crate::paint::{ColorRef, ElementDraw, TextEffects, TextShadow};
            self.draws.insert(
                i,
                ElementDraw {
                    text: Some(TextEffects {
                        shadows: vec![TextShadow {
                            offset: [0.0, (step.size * 0.04).max(1.0)],
                            blur: (step.size * 0.08).max(1.0),
                            color: ColorRef::token("sceneShadow"),
                        }],
                        ..TextEffects::default()
                    }),
                    ..ElementDraw::default()
                },
            );
        }
        // Room across for the style's drop inside a scroll view, whose edge
        // clips it (pixel's results figures lost their last font pixel of
        // drop to the view's edge); elsewhere a drop draws past its box, and
        // reserving room there would change the layout of every large label.
        let grid = match &self.nodes[i].content {
            Content::Text(tc) => tc.metrics(&self.fonts).pixel_grid(),
            _ => 0,
        };
        let mut ancestor = parent;
        let mut scrolled = false;
        while let Some(node) = self.nodes.get(ancestor) {
            scrolled |= node.el.scroll || node.el.scroll_x;
            ancestor = node.parent;
        }
        let in_control = self.nodes.get(parent).is_some_and(|p| p.el.interactive);
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
        let i = self.leaf("button", id.into(), Element::default());
        self.button_at(i, text, desc)
    }

    /// A button holding content (a document's `<button><icon/>Buy<badge/></button>`):
    /// the children built until [`Ui::close_button`] lie in a row in its
    /// frame, its states and its press; text and icons in it without a
    /// colour of their own take its ink.
    pub(crate) fn open_button(&mut self, id: impl Into<Id>, desc: ButtonDesc) -> Response {
        let i = self.push("button", id.into(), Element::default());
        self.stack.push(i);
        self.button_ink.push((i, self.theme.palette.ink));
        self.button_at(i, "", desc)
    }

    /// Closes a button opened with [`Ui::open_button`].
    pub(crate) fn close_button(&mut self) {
        self.button_ink.pop();
        self.close();
    }

    /// The ink a text or icon without its own colour takes: its button's
    /// while one holds it, else the palette's.
    pub(crate) fn content_ink(&self) -> Srgba {
        self.button_ink
            .last()
            .map_or(self.theme.palette.ink, |&(_, ink)| ink)
    }

    fn button_at(&mut self, i: usize, text: &str, desc: ButtonDesc) -> Response {
        self.nodes[i].disabled = desc.disabled;
        let refused = desc.disabled
            && self.theme.art.motion.shake > 0.0
            && self.respond(self.nodes[i].key).clicked;
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
        let amounts = if desc.disabled {
            (0.0, 0.0)
        } else {
            self.state_amounts(i, &r)
        };
        let press = self.button_motion(i, r, refused);
        self.apply_button(i, text, desc, r, amounts, press);
        if r.clicked && desc.variant == ButtonVariant::Primary {
            self.feedback(crate::tokens::SoundEvent::Confirm);
        }
        r
    }

    fn button_box(&self, desc: ButtonDesc) -> Element {
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
        Element {
            width,
            height: Size::Fixed(h),
            padding: Insets::xy(pad, 0.0),
            // Its content, when it holds some, is a centred row.
            direction: Direction::Row,
            align: Align::Center,
            cross: Cross::Center,
            gap: t.space.xs,
            // A style that shakes a refused press hears presses on a disabled button.
            interactive: !desc.disabled || t.art.motion.shake > 0.0,
            radius: t.radius.control,
            ..Element::default()
        }
    }

    /// Applies presentation from an already processed response and sampled motion.
    pub(crate) fn apply_button(
        &mut self,
        i: usize,
        text: &str,
        desc: ButtonDesc,
        r: Response,
        amounts: (f32, f32),
        press: f32,
    ) {
        self.nodes[i].el = self.button_box(desc);
        self.nodes[i].disabled = desc.disabled;
        let width = self.nodes[i].el.width;
        let key = self.nodes[i].key;
        let variant = match desc.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Quiet => "quiet",
            ButtonVariant::Danger => "danger",
            ButtonVariant::Ghost => "ghost",
        };
        let named = desc.part.unwrap_or_else(|| {
            crate::doc::schema::parts::BUTTON_BASE
                .reference(Some(crate::tokens::ElementClass::Button))
        });
        let mut part = crate::dress::PartRef {
            class: Some(crate::tokens::ElementClass::Button),
            ..named
        }
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
        let resolved = self.part_values(&part, Some(key));
        let fg = resolved.color("color").unwrap_or(self.theme.palette.ink);
        let mut content = self.role_text(text, crate::tokens::TypeRole::Button, fg);
        content.align = desc.align.unwrap_or(TextAlign::Center);
        if let Some(top) = self.button_ink.last_mut().filter(|(at, _)| *at == i) {
            top.1 = fg;
        }
        let n = &mut self.nodes[i];
        n.el.part = Some(part);
        n.el.frame = None;
        n.content = if text.is_empty() {
            Content::None
        } else {
            Content::Text(content)
        };
        n.widget = Widget::Button {
            pressed: r.pressed,
            press,
        };
        n.probe_text = text.to_owned();
        let content = std::mem::replace(&mut n.content, Content::None);
        let element = Element {
            width: Size::grow(),
            height: Size::grow(),
            interactive: false,
            background: None,
            part: None,
            ..n.el
        };
        n.el.padding = Insets::default();
        n.el.gap = 0.0;
        n.el.background = None;
        n.el.radius = 0.0;
        let slot = self.write_part(i, part, ("base", 0), element, content);
        self.nodes[i].content_slot = slot;
    }

    /// A texture region, or a solid rectangle without a texture; with
    /// `aspect`, a fitting side follows the other.
    pub fn image(&mut self, id: impl Into<Id>, desc: ImageDesc) {
        let i = self.leaf("image", id.into(), Element::default());
        self.apply_image(i, desc);
    }

    pub(crate) fn apply_image(&mut self, i: usize, desc: ImageDesc) {
        self.nodes[i].el = Element {
            width: desc.width,
            height: desc.height,
            aspect: desc.aspect.max(0.0),
            ..Element::default()
        };
        self.nodes[i].content = Content::Image {
            texture: desc.texture,
            uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
            tint: desc.tint.unwrap_or(Srgba::WHITE),
            object: Default::default(),
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
            damping: crate::theme::TIMING.press_damping,
            response_ms: t.durations_ms.instant,
        });
        // Each spring is a motion channel (docs/ui.md §3.1): a retarget
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
