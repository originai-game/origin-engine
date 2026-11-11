//! The button: a capsule with a label, or with content in a row.

use super::{Control, ControlOutput};
use crate::id::{Id, child_key};
use crate::layout::{Align, Content, Cross, Direction, Element, Insets, Size, TextAlign, Widget};
use crate::motion::Fx;
use crate::ui::*;

/// What a button is built from.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ButtonModel {
    /// Its label (empty: it holds content).
    pub(crate) text: String,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ControlSize,
    pub(crate) disabled: bool,
    /// `None`: as wide as its label.
    pub(crate) width: Option<Size>,
    /// Takes the focus when it appears.
    pub(crate) autofocus: bool,
    /// As wide as it is tall, without padding.
    pub(crate) square: bool,
    /// Its label's alignment (`None`: centred).
    pub(crate) align: Option<TextAlign>,
    /// The CSS part it is (`None`: the button's base).
    pub(crate) part: Option<crate::dress::PartRef>,
}

/// What a button's behaviour reads of its model.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ButtonBehaviour {
    disabled: bool,
    autofocus: bool,
    /// A click confirms (the primary variant).
    confirms: bool,
}

impl ButtonModel {
    pub(crate) fn behaviour(&self) -> ButtonBehaviour {
        ButtonBehaviour {
            disabled: self.disabled,
            autofocus: self.autofocus,
            confirms: self.variant == ButtonVariant::Primary,
        }
    }
}

impl Ui {
    /// A button with a label as a leaf under the open element.
    pub(crate) fn button_element(
        &mut self,
        id: impl Into<Id>,
        model: ButtonModel,
    ) -> ControlOutput {
        let i = self.leaf("button", id.into(), Element::default());
        self.control_at(i, Control::Button(model))
    }

    /// A button holding content (a document's `<button><icon/>Buy<badge/></button>`):
    /// the children built until [`Ui::close_button_element`] lie in a row in its
    /// frame, its states and its press; text and icons in it without a
    /// colour of their own take its ink.
    pub(crate) fn open_button_element(
        &mut self,
        id: impl Into<Id>,
        model: ButtonModel,
    ) -> ControlOutput {
        let i = self.push("button", id.into(), Element::default());
        self.tree.stack.push(i);
        self.front.button_ink.push((i, self.env.theme.palette.ink));
        self.control_at(i, Control::Button(model))
    }

    /// Closes a button opened with [`Ui::open_button_element`].
    pub(crate) fn close_button_element(&mut self) {
        self.front.button_ink.pop();
        self.close();
    }

    /// The button's behaviour: its response, its focus when it appears,
    /// its state amounts and style motion, and the confirm of a primary
    /// click.
    pub(super) fn dispatch_button(&mut self, i: usize, model: ButtonBehaviour) -> ControlOutput {
        self.tree.nodes[i].disabled = model.disabled;
        // Its motion sets it again where it moves.
        self.tree.nodes[i].fx = None;
        let refused = model.disabled
            && self.env.theme.art.motion.shake > 0.0
            && self.respond(self.tree.nodes[i].key).clicked;
        let r = if model.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        // What opens takes the focus (as a dialog's first control and a
        // dialogue box do): appearing, outside a dialog.
        let key = self.tree.nodes[i].key;
        if model.autofocus
            && !model.disabled
            && self.interaction.state.scope == 0
            && !self.interaction.state.hits.iter().any(|h| h.key == key)
        {
            self.interaction.request_focus = key;
        }
        let amounts = if model.disabled {
            (0.0, 0.0)
        } else {
            self.state_amounts(i, &r)
        };
        let press = self.button_motion(i, r, refused);
        if r.clicked && model.confirms {
            self.feedback(crate::tokens::SoundEvent::Confirm);
        }
        ControlOutput {
            response: r,
            amounts,
            press,
        }
    }

    fn button_box(&self, model: &ButtonModel) -> Element {
        let t = self.env.theme;
        let sizes = self.sizes();
        // Never shorter than its words with an inset above and below: a
        // style whose button step outgrows its control size (pixel's 24
        // on a 32 control) grows the plate rather than filling it edge to edge.
        let words = self.type_step(crate::tokens::TypeRole::Button).line_height + 2.0 * t.space.xs;
        let h = match model.size {
            ControlSize::Small => sizes.control_sm,
            ControlSize::Medium => sizes.control_md,
            ControlSize::Large => sizes.control_lg,
        }
        .max(words);
        let pad = if model.size == ControlSize::Small {
            t.space.md
        } else {
            t.space.lg
        };
        let (width, pad) = if model.square {
            (Size::Fixed(h), 0.0)
        } else {
            (model.width.unwrap_or_default(), pad)
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
            interactive: !model.disabled || t.art.motion.shake > 0.0,
            radius: t.radius.control,
            ..Element::default()
        }
    }

    /// The button's appearance: its box, its part in its state, its label
    /// in the part's ink and its base box.
    pub(super) fn present_button(&mut self, i: usize, model: &ButtonModel, output: &ControlOutput) {
        let r = output.response;
        self.tree.nodes[i].el = self.button_box(model);
        self.tree.nodes[i].disabled = model.disabled;
        let width = self.tree.nodes[i].el.width;
        let key = self.tree.nodes[i].key;
        let variant = match model.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Quiet => "quiet",
            ButtonVariant::Danger => "danger",
            ButtonVariant::Ghost => "ghost",
        };
        let named = model.part.unwrap_or_else(|| {
            crate::doc::schema::parts::BUTTON_BASE
                .reference(Some(crate::tokens::ElementClass::Button))
        });
        let mut part = crate::dress::PartRef {
            class: Some(crate::tokens::ElementClass::Button),
            ..named
        }
        .flag(variant)
        .flag_if(model.disabled, "disabled")
        .flag_if(!model.disabled && r.pressed, "pressed")
        .flag_if(!model.disabled && r.hovered && !r.pressed, "hover")
        .flag_if(r.focused, "focused")
        .flag_if(model.size == ControlSize::Small, "small")
        .flag_if(model.size == ControlSize::Large, "large")
        .flag_if(model.square, "square")
        .flag_if(self.interaction.narrow_button(key, width), "narrow");
        part.hover = output.amounts.0;
        part.press = output.amounts.1;
        let resolved = self.part_values(&part, Some(key));
        let fg = resolved
            .color("color")
            .unwrap_or(self.env.theme.palette.ink);
        let mut content = self.role_text(&model.text, crate::tokens::TypeRole::Button, fg);
        content.align = model.align.unwrap_or(TextAlign::Center);
        if let Some(top) = self.front.button_ink.last_mut().filter(|(at, _)| *at == i) {
            top.1 = fg;
        }
        let n = &mut self.tree.nodes[i];
        n.el.part = Some(part);
        n.el.frame = None;
        n.content = if model.text.is_empty() {
            Content::None
        } else {
            Content::Text(content)
        };
        n.widget = Widget::Button {
            pressed: r.pressed,
            press: output.press,
        };
        n.probe_text = model.text.clone();
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
        let slot = self
            .tree
            .write_part(i, part, ("base", 0), element, content, &mut self.layout);
        self.tree.nodes[i].content_slot = slot;
    }

    /// A button's style motion: the hover lift and the press squash on the
    /// style's springs (`motion.hoverSpring`, `motion.pressSpring`), and the shake of a refused press (`motion.duration.slow`). Answers
    /// how far in the press is (0 at rest, 1 held; past either while the
    /// spring overshoots).
    fn button_motion(&mut self, i: usize, r: Response, refused: bool) -> f32 {
        let t = self.env.theme;
        let art = t.art;
        if art.hover.is_none() && art.press.is_none() && art.motion.shake <= 0.0 {
            return if r.pressed { 1.0 } else { 0.0 };
        }
        let key = self.tree.nodes[i].key;
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
            let fresh = !ui.interaction.state.motion.channels.contains_key(&k);
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
        let now = self.interaction.state.tick;
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
            self.tree.nodes[i].fx = Some(f);
        }
        press
    }
}
