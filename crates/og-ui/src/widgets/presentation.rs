use super::*;
use crate::ui::IconCell;

impl Ui {
    /// A chip with a selectable `base` part.
    pub fn chip(&mut self, id: impl Into<Id>, text: &str, desc: ChipDesc) -> Response {
        let t = self.env.theme;
        let p = t.palette;
        let h = self.sizes().control_sm;
        let i = self.leaf(
            "chip",
            id.into(),
            Element {
                // A short label ("All") still makes a target a finger hits.
                width: Size::Fit {
                    min: if self.env.viewport.coarse_pointer {
                        h
                    } else {
                        0.0
                    },
                    max: f32::INFINITY,
                },
                height: Size::Fixed(h),
                padding: Insets::xy(t.space.md, 0.0),
                interactive: !desc.disabled,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.tree.nodes[i].disabled = desc.disabled;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        let amounts = if desc.disabled {
            (0.0, 0.0)
        } else {
            self.state_amounts(i, &r)
        };
        if r.clicked {
            self.feedback(crate::tokens::SoundEvent::Select);
        }
        let (bg, fg) = if desc.disabled {
            (p.control, p.ink_faint)
        } else if desc.selected {
            (p.ink, p.ink_inverse)
        } else {
            (
                Ui::state_color(p.control, p.control_strong, p.control_strong, amounts),
                p.ink,
            )
        };
        let mut c = self.role_text(text, crate::tokens::TypeRole::Caption, fg);
        c.align = TextAlign::Center;
        let mut part =
            crate::dress::PartRef::new("Chip", "base", crate::tokens::ElementClass::Chip)
                .flag_if(desc.selected, "selected")
                .flag_if(desc.disabled, "disabled")
                .flag_if(r.hovered && !r.pressed, "hover")
                .flag_if(r.pressed, "pressed")
                .flag_if(r.focused, "focused");
        part.hover = amounts.0;
        part.press = amounts.1;
        let n = &mut self.tree.nodes[i];
        n.el.background = Some(bg);
        n.el.part = Some(part);
        n.content = Content::Text(c);
        n.probe_text = text.to_owned();
        n.a11y.selected = Some(desc.selected);
        r
    }

    /// A badge: a small status capsule in its tone (tinted fill, tone ink).
    ///
    /// Its ink reads [`READABLE`] on what it is drawn on. The tones and the
    /// accent are the palette's, made for its panels (where the packs' test
    /// holds them); on a ground on the other side of the scale from the
    /// palette's `panel` (a light pack's deep status colours on its dark
    /// stage) the ink moves towards its own end of the scale until it reads
    /// there (a deep green lightens on a dark screen; in the cream panel it
    /// stays deep). The neutral badge's ground is its own flat `control`, so
    /// its `inkMuted` moves wherever it does not read on it, and where even
    /// white or black would not, the fill moves away from the ink instead
    /// (scifi's mid-blue fill darkens under its pale ink).
    pub fn badge(&mut self, id: impl Into<Id>, text: &str, tone: Tone) {
        let t = self.env.theme;
        let p = t.palette;
        let (bg, fg) = match tone {
            Tone::Neutral => (p.control, p.ink_muted),
            Tone::Primary => (p.primary_soft, p.highlight),
            Tone::Inverse => (p.ink, p.ink_inverse),
            // The status tones: the tone's own ink on its 16 % tint, mixed in
            // sRGB as a CSS wash would be (Srgba::tint_over, once the badge
            // knows what is behind it), so each pack's tones read 4.5:1.
            other => {
                let c = self.tone_dot(other);
                (c.fade(0.16), c)
            }
        };
        let i = self.leaf(
            "badge",
            id.into(),
            Element {
                height: Size::Fixed(t.type_ramp.xs.line_height + t.space.xs),
                padding: Insets::xy(t.space.sm, 0.0),
                background: Some(bg),
                radius: t.radius.control,
                part: Some(
                    crate::dress::PartRef::new("Badge", "base", crate::tokens::ElementClass::Badge)
                        .attr("tone", tone.name()),
                ),
                ..Element::default()
            },
        );
        let under = self.behind(i);
        let (mut bg, ground) = match tone {
            // og-render composites the tint in linear light: tint_over
            // picks the alpha whose blend has the browser's luminance.
            Tone::Success | Tone::Warn | Tone::Danger | Tone::Info => {
                (fg.tint_over(under, 0.16), fg.fade(0.16).over(under))
            }
            _ => (bg, bg.over_linear(under)),
        };
        let dark = |c: Srgba| Srgba::WHITE.contrast(c) > Srgba::BLACK.contrast(c);
        let moves = match tone {
            Tone::Neutral => true,
            _ => dark(ground) != dark(p.panel),
        };
        // The ground is an estimate in bytes (the tint's sRGB mix), so a
        // moved ink aims a little over the bar.
        let aim = READABLE + 0.1;
        let fg = match fg.readable_on(ground, aim) {
            Some(ink) if moves => ink,
            None if moves => {
                if let Some(g) = ground.readable_on(fg, aim) {
                    bg = g;
                }
                fg
            }
            _ => fg,
        };
        let mut c = self.text_content(text, t.type_ramp.xs, fg);
        c.wrap = false;
        c.align = TextAlign::Center;
        let n = &mut self.tree.nodes[i];
        n.el.background = Some(bg);
        n.content = Content::Text(c);
        n.probe_text = text.to_owned();
    }

    /// A hairline (`line`) across its parent; `vertical` stands it up.
    pub fn divider(&mut self, id: impl Into<Id>, vertical: bool) {
        let c = self.env.theme.palette.line;
        let px = 1.0 / self.env.viewport.dpr().max(1.0);
        let px = px.max(0.5);
        self.leaf(
            "divider",
            id.into(),
            Element {
                width: if vertical {
                    Size::Fixed(px)
                } else {
                    Size::grow()
                },
                height: if vertical {
                    Size::grow()
                } else {
                    Size::Fixed(px)
                },
                background: Some(c),
                ..Element::default()
            },
        );
    }

    /// Empty space: `size` along both axes (the zero value grows).
    pub fn spacer(&mut self, id: impl Into<Id>, size: Option<Size>) {
        let s = size.unwrap_or(Size::grow());
        let i = self.leaf(
            "spacer",
            id.into(),
            Element {
                width: s,
                height: s,
                ..Element::default()
            },
        );
        self.tree.nodes[i].widget = Widget::Spacer;
    }

    /// A key and its value on one row (`inkMuted` key, `ink` value at the end).
    pub fn key_value(&mut self, id: impl Into<Id>, key: &str, value: &str, mono: bool) {
        let t = self.env.theme;
        let h = self.sizes().row;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                height: Size::Fit {
                    min: h * 0.75,
                    max: f32::INFINITY,
                },
                align: Align::SpaceBetween,
                cross: Cross::Center,
                gap: t.space.md,
                ..Element::default()
            },
        );
        self.label(
            "Key",
            key,
            crate::LabelDesc {
                color: Some(t.palette.ink_muted),
                nowrap: true,
                ..Default::default()
            },
        );
        self.label(
            "Value",
            value,
            crate::LabelDesc {
                nowrap: true,
                align: TextAlign::End,
                font: if mono { FontRole::Mono } else { FontRole::Body },
                ..Default::default()
            },
        );
        self.close();
    }

    /// A skeleton: the neutral fill standing in for content, with a sheen
    /// that passes every 1.6 s (still under reduced motion).
    pub fn skeleton(&mut self, id: impl Into<Id>, desc: SkeletonDesc) {
        let t = self.env.theme;
        let lines = desc.lines.max(1);
        let frame = crate::dress::PartRef::of("Skeleton", "frame", None);
        let sheen = crate::dress::PartRef::of("Skeleton", "sheen", None);
        let h = if desc.height > 0.0 {
            desc.height
        } else {
            t.type_ramp.body.size
        };
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or(Size::grow()),
                gap: t.space.sm,
                part: Some(frame),
                ..Element::default()
            },
        );
        let phase = self.part_cycle(&sheen, 1600.0);
        for k in 0..lines {
            let last = lines > 1 && k == lines - 1;
            let part = crate::dress::PartRef::of("Skeleton", "line", None).flag_if(last, "last");
            let i = self.leaf(
                "skeleton",
                Id::default(),
                Element {
                    width: if last {
                        Size::Percent(0.6)
                    } else {
                        Size::grow()
                    },
                    height: Size::Fixed(h),
                    background: Some(t.palette.control),
                    radius: t.radius.control,
                    part: Some(part),
                    ..Element::default()
                },
            );
            if !t.reduced_motion {
                let band = 0.3;
                let x = -band + phase * (1.0 + band);
                let (a, b) = (x.max(0.0), (x + band).min(1.0));
                if b > a {
                    self.tree.nodes[i].shapes.push(Shape::DressedRect {
                        part: sheen,
                        frame: Frame::left(a, b),
                        radius: 0.0,
                        color: t.palette.sheen.fade(0.35),
                    });
                }
            }
        }
        self.close();
    }

    /// A loading spinner: eight dots around a circle, the bright one turning.
    pub fn spinner(&mut self, id: impl Into<Id>, size: f32) {
        let t = self.env.theme;
        let base = crate::dress::PartRef::of("Spinner", "base", None);
        let dot_part = crate::dress::PartRef::of("Spinner", "dot", None);
        let s = if size > 0.0 {
            size
        } else {
            self.sizes().control_sm - t.space.xs
        };
        let i = self.leaf(
            "spinner",
            id.into(),
            Element {
                width: Size::Fixed(s),
                height: Size::Fixed(s),
                part: Some(base),
                ..Element::default()
            },
        );
        self.tree.nodes[i].a11y.busy = true;
        self.tree.nodes[i].a11y.name = Some("Loading".into());
        let head = (self.part_cycle(&dot_part, t.durations_ms.slow * 2.0) * 8.0) as i32;
        let dot = s / 5.0;
        for k in 0..8 {
            let ang = k as f32 * PI / 4.0 - FRAC_PI_2;
            let (dx, dy) = (
                (s / 2.0 - dot / 2.0) * cos(ang),
                (s / 2.0 - dot / 2.0) * sin(ang),
            );
            let age = (head - k).rem_euclid(8) as f32;
            self.tree.nodes[i].shapes.push(Shape::DressedRect {
                part: dot_part.attr(
                    "age",
                    ["0", "1", "2", "3", "4", "5", "6", "7"][age as usize],
                ),
                frame: Frame::centered(dx, dy, dot, dot),
                radius: dot,
                color: t.palette.ink.fade(1.0 - age / 9.0),
            });
        }
    }

    /// An icon of the atlas (`set_icons`) by name; a missing name draws an
    /// empty slot and warns.
    pub fn icon(&mut self, id: impl Into<Id>, name: &str, desc: IconDesc) {
        let i = self.leaf("icon", id.into(), Element::default());
        self.apply_icon(i, name, desc);
    }

    pub(crate) fn apply_icon(&mut self, i: usize, name: &str, desc: IconDesc) {
        let size = if desc.size > 0.0 { desc.size } else { 24.0 };
        self.tree.nodes[i].el = Element {
            width: Size::Fixed(size),
            height: Size::Fixed(size),
            ..Element::default()
        };
        self.tree.nodes[i].shapes.clear();
        for (part, name, tint) in std::iter::once((
            crate::doc::schema::parts::ICON_IMAGE,
            name,
            desc.tint.unwrap_or(self.content_ink()),
        ))
        .chain(desc.secondary.as_deref().map(|name| {
            (
                crate::doc::schema::parts::ICON_SECONDARY,
                name,
                desc.secondary_tint.unwrap_or(self.env.theme.palette.accent),
            )
        })) {
            match self.icon_shape(part.reference(None), name, Frame::FULL, tint) {
                Some(shape) => self.tree.nodes[i].shapes.push(shape),
                None => self.out.warnings.push(format!(
                    "icon '{name}' is in neither the icon atlas nor UiDesc.fallbackIcons; add it to one of them"
                )),
            }
        }
        self.tree.nodes[i].probe_text = desc.label.unwrap_or_else(|| name.to_owned());
    }

    /// An art slot (docs/ui.md §3.3): its texture fitted as its spec says
    /// (cover around the focal point, contain, a nine-slice…), cut to its
    /// mask, with the style's treatments (a vignette, a frame, a rarity
    /// glow); until the art is there, a quiet surface and a small centred
    /// mark from the kind's icon set, with the slot's name in the default
    /// style only. The mark never stretches with the slot. With a
    /// height left to fit it takes the slot's aspect.
    pub fn art(&mut self, id: impl Into<Id>, desc: &ArtDesc) {
        let i = self.leaf("art", id.into(), Element::default());
        self.apply_art(i, desc);
    }

    pub(crate) fn apply_art(&mut self, i: usize, desc: &ArtDesc) {
        use crate::paint::{ColorRef, Effect, ElementDraw, Paint, ShapeKind};
        use crate::slot::{Mask, SlotAspect, Treatment};
        let t = self.env.theme;
        let spec = &desc.spec;
        let aspect = match &spec.aspect {
            SlotAspect::One(a) => a.ratio(),
            SlotAspect::AnyOf(v) => v.first().map_or(0.0, |a| a.ratio()),
            SlotAspect::Range(lo, _) => lo.ratio(),
            SlotAspect::Free => 0.0,
        };
        self.tree.nodes[i].el = Element {
            width: desc.width,
            height: desc.height,
            aspect,
            ..Element::default()
        };
        self.tree.nodes[i].shapes.clear();
        let token = |n: &str, a: f32| ColorRef::Token {
            name: n.to_string(),
            alpha: a,
        };
        let shape = match spec.mask {
            Mask::Circle => ShapeKind::Ellipse,
            Mask::Hexagon => ShapeKind::Polygon {
                sides: 6,
                inner: 1.0,
                rounding: 0.0,
                rotation: 0.0,
            },
            Mask::Round(r) => ShapeKind::Rect {
                radii: Some([r; 4]),
                corner: crate::paint::Corner::Round,
            },
            Mask::Squircle => ShapeKind::Rect {
                radii: None,
                corner: crate::paint::Corner::Squircle(4.0),
            },
            Mask::Shape(c) => {
                let (corner, r) = crate::dress::corner_of(t.tokens.shape(c));
                ShapeKind::Rect {
                    radii: Some([r; 4]),
                    corner,
                }
            }
            Mask::None | Mask::Alpha => ShapeKind::Rect {
                radii: Some(
                    [t.tokens
                        .shape(crate::tokens::ElementClass::Slot)
                        .radius
                        .min(8.0); 4],
                ),
                corner: crate::paint::Corner::Round,
            },
        };
        let mut d = ElementDraw {
            shape,
            clip: crate::paint::Clip::Shape,
            ..Default::default()
        };
        let ph = spec.kind.placeholder();
        match desc.texture {
            Some(texture) => {
                // At its own size (none, scale-down) the texture's own
                // pixels place it: the renderer reads them for a zero size.
                let own = matches!(
                    spec.fit,
                    crate::slot::Fit::None | crate::slot::Fit::ScaleDown
                );
                let size = desc.size.filter(|_| !own).unwrap_or(match &spec.aspect {
                    _ if own => [0.0; 2],
                    SlotAspect::One(a) => [f32::from(a.w), f32::from(a.h)],
                    _ => [1.0, 1.0],
                });
                d.fills.push(Paint::Image(crate::paint::ImagePaint {
                    texture,
                    uv: [0.0, 0.0, 1.0, 1.0],
                    size,
                    density: 1.0,
                    tint: ColorRef::Literal(Srgba::WHITE),
                    fit: match spec.fit {
                        crate::slot::Fit::Cover => crate::paint::Fit::Cover,
                        crate::slot::Fit::Contain => crate::paint::Fit::Contain,
                        crate::slot::Fit::None => crate::paint::Fit::None,
                        crate::slot::Fit::ScaleDown => crate::paint::Fit::ScaleDown,
                        _ => crate::paint::Fit::Fill,
                    },
                    focal: spec.focal,
                    offset: desc.offset,
                    nearest: desc.nearest,
                }));
            }
            None => {
                d.fills.push(Paint::Solid(token("well", 1.0)));
            }
        }
        for tr in &spec.treatments {
            match tr {
                Treatment::Vignette => d.effects.push(Effect::InnerShadow {
                    offset: [0.0, 0.0],
                    blur: 24.0,
                    spread: 0.0,
                    color: token("shadow", 1.0),
                }),
                Treatment::Frame => d.strokes.push(crate::paint::Stroke {
                    width: t.tokens.stroke(crate::tokens::StrokeWidth::Regular),
                    align: crate::paint::StrokeAlign::Inside,
                    dash: [0.0; 2],
                    dash_offset: 0.0,
                    dash_cap: crate::paint::Cap::Butt,
                    paint: Paint::Solid(token("lineStrong", 1.0)),
                }),
                Treatment::RarityGlow => d.effects.push(Effect::OuterGlow {
                    blur: 10.0,
                    spread: 0.0,
                    color: token("primary", 0.6),
                }),
                Treatment::Duotone => {}
            }
        }
        self.tree.draws.insert(i, d);
        if desc.texture.is_none() {
            // The kind's silhouette, a third of the shorter side, centred
            // (a little high when the slot's name sits under it).
            let named = self.env.style_name.is_empty();
            let k = 0.36;
            let frame = Frame {
                fx: 0.5 - k / 2.0,
                fy: if named { 0.4 - k / 2.0 } else { 0.5 - k / 2.0 },
                fw: k,
                fh: k,
                square_max: Some(32.0),
                ..Frame::default()
            };
            let ink = t.palette.ink_muted;
            match self.icon_shape(
                crate::dress::PartRef::of("ArtSlot", "icon", None),
                ph.icon,
                frame,
                ink,
            ) {
                Some(shape) => self.tree.nodes[i].shapes.push(shape),
                // No icon set: a disc, the shape of a subject.
                None => self.tree.nodes[i].shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("ArtSlot", "placeholder", None),
                    frame,
                    radius: 9999.0,
                    color: ink.fade(0.5),
                }),
            }
            if named {
                let step = self.type_step(crate::tokens::TypeRole::Caption);
                self.tree.nodes[i].shapes.push(Shape::Text {
                    part: crate::dress::PartRef::of("ArtSlot", "label", None),
                    frame: Frame {
                        fy: 0.72,
                        fh: 0.2,
                        ..Frame::FULL
                    },
                    text: spec.name.clone(),
                    step,
                    color: ink,
                    font: FontRole::Mono,
                    clip: Frame::FULL,
                });
            }
        }
        let n = &mut self.tree.nodes[i];
        n.probe_text = if desc.label.is_empty() {
            spec.name.clone()
        } else {
            desc.label.clone()
        };
        n.a11y.role = Some("image");
        n.a11y.name = Some(n.probe_text.clone());
    }

    /// Whether the icon atlas has an icon of this name.
    pub fn has_icon(&self, name: &str) -> bool {
        self.env.icons.find(name).is_some()
    }

    pub(super) fn icon_shape(
        &self,
        part: crate::dress::PartRef,
        name: &str,
        frame: Frame,
        color: Srgba,
    ) -> Option<Shape> {
        Some(match self.env.icons.find(name)? {
            IconCell::Glyph(font, ch) => Shape::Glyph {
                part,
                frame,
                font,
                ch,
                color,
            },
            IconCell::Image(texture, uv) => Shape::Image {
                part,
                frame,
                texture,
                uv,
                color,
            },
        })
    }
}
