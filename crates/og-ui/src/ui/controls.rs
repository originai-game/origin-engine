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
        let parent = self.tree.nodes[i].parent;
        let in_column = self
            .tree
            .nodes
            .get(parent)
            .is_some_and(|p| p.el.direction == Direction::Column);
        let width = if desc.align != TextAlign::Start && desc.width == Size::fit() && in_column {
            Size::grow()
        } else {
            desc.width
        };
        self.tree.nodes[i].el.width = width;
        self.tree.nodes[i].el.padding = Insets::default();
        self.tree.draws.remove(&i);
        let ink = self.content_ink();
        // Borrowed, not copied: a label is the most frequent element.
        let t = &self.env.theme;
        let mut step = desc.step.unwrap_or(t.type_ramp.body);
        step.uppercase |= desc.uppercase;
        let fit = desc.fit_min.map_or_else(Vec::new, |min| {
            crate::style_file::fit_steps(&t.type_ramp, step, min)
        });
        let n = &mut self.tree.nodes[i];
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
            self.tree.draws.insert(
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
        let grid = match &self.tree.nodes[i].content {
            Content::Text(tc) => tc.metrics(&self.env.fonts).pixel_grid(),
            _ => 0,
        };
        let mut ancestor = parent;
        let mut scrolled = false;
        while let Some(node) = self.tree.nodes.get(ancestor) {
            scrolled |= node.el.scroll || node.el.scroll_x;
            ancestor = node.parent;
        }
        let in_control = self
            .tree
            .nodes
            .get(parent)
            .is_some_and(|p| p.el.interactive);
        if let Some((dx, _, blur, _)) = self
            .paint
            .text_drop(desc.font.effect_index(), step.size, grid, &self.env)
            .filter(|_| scrolled && !in_control)
        {
            let reach = (dx.abs() + blur / 2.0).ceil();
            let pad = &mut self.tree.nodes[i].el.padding;
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
        self.tree.stack.push(i);
        self.front.button_ink.push((i, self.env.theme.palette.ink));
        self.button_at(i, "", desc)
    }

    /// Closes a button opened with [`Ui::open_button`].
    pub(crate) fn close_button(&mut self) {
        self.front.button_ink.pop();
        self.close();
    }

    /// The ink a text or icon without its own colour takes: its button's
    /// while one holds it, else the palette's.
    pub(crate) fn content_ink(&self) -> Srgba {
        self.front
            .button_ink
            .last()
            .map_or(self.env.theme.palette.ink, |&(_, ink)| ink)
    }

    /// A texture region, or a solid rectangle without a texture; with
    /// `aspect`, a fitting side follows the other.
    pub fn image(&mut self, id: impl Into<Id>, desc: ImageDesc) {
        let i = self.leaf("image", id.into(), Element::default());
        self.apply_image(i, desc);
    }

    pub(crate) fn apply_image(&mut self, i: usize, desc: ImageDesc) {
        self.tree.nodes[i].el = Element {
            width: desc.width,
            height: desc.height,
            aspect: desc.aspect.max(0.0),
            ..Element::default()
        };
        self.tree.nodes[i].content = Content::Image {
            texture: desc.texture,
            uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
            tint: desc.tint.unwrap_or(Srgba::WHITE),
            object: Default::default(),
        };
        self.tree.nodes[i].probe_text = desc.label;
    }

    /// A capsule track filled to `value` (0–1).
    pub fn progress(&mut self, id: impl Into<Id>, value: f32, desc: ProgressDesc) {
        let t = self.env.theme;
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
            let key = child_key(self.tree.nodes[i].key, "ease");
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
        self.tree.nodes[i].widget = Widget::Progress { value: v };
        // The fill stays the track's drawing (`node_draw`): it is a fraction
        // of the track's device-pixel box, never narrower than it is tall.
        self.bar_track(
            i,
            crate::dress::PartRef::new("progress", "track", crate::tokens::ElementClass::Bar),
            t.palette.control_strong,
            0.0,
            Vec::new(),
        );
    }

    /// A bar widget's track: the persistent CSS box filling host `i`
    /// (which keeps the semantics) in `color` at `radius` unless its CSS
    /// paints it, drawing `shapes` (the bar's fractional parts) over it.
    /// Answers the box.
    pub(crate) fn bar_track(
        &mut self,
        i: usize,
        part: crate::dress::PartRef,
        color: Srgba,
        radius: f32,
        shapes: Vec<crate::layout::Shape>,
    ) -> usize {
        let slot = self.tree.write_part(
            i,
            part,
            (part.part, 0),
            Element {
                width: Size::grow(),
                height: Size::grow(),
                background: Some(color),
                radius,
                ..Element::default()
            },
            Content::None,
            &mut self.layout,
        );
        self.tree.nodes[i].content_slot = slot;
        self.tree.nodes[slot].shapes = shapes;
        slot
    }
}
