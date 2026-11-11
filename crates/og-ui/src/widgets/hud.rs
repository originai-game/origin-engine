use super::*;

impl Ui {
    // ── HUD ────────────────────────────────────────────────────────────────

    /// A meter (health, mana, xp): a capsule track and fill. A drop shows at
    /// once and leaves a pale trail that follows after a beat; a rise grows
    /// into place with a faint ghost of the target, by the tick.
    pub fn meter(&mut self, id: impl Into<Id>, value: f32, max: f32, desc: &MeterDesc) {
        let t = self.env.theme;
        let p = t.palette;
        let frac = if max > 0.0 && value.is_finite() {
            (value / max).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let h = if desc.height > 0.0 {
            desc.height
        } else if desc.label.is_empty() {
            t.space.md
        } else {
            t.type_ramp.caption.line_height + t.space.xxs
        };
        let i = self.leaf(
            "meter",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(h),
                background: Some(p.control_strong),
                radius: t.radius.bar,
                part: Some(crate::dress::PartRef::new(
                    "Meter",
                    "track",
                    crate::tokens::ElementClass::Bar,
                )),
                ..Element::default()
            },
        );
        let key = self.tree.nodes[i].key;
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        let now = self.interaction.state.tick;
        if fresh {
            a = Anim {
                a: frac,
                b: frac,
                t: now,
                h: 0,
            };
        }
        let hold = self.ticks_for(t.durations_ms.fast) as f32;
        let dur = self.ticks_for(t.durations_ms.slow) as f32;
        let shown = |a: &Anim| -> (f32, f32) {
            let el = (now - a.t) as f32;
            if a.b < a.a {
                // Dropped: the fill at the target, the trail from a towards b.
                let k = t
                    .tokens
                    .ease(EaseRole::WidgetEnter)
                    .apply(((el - hold) / dur).clamp(0.0, 1.0));
                (a.b, a.a + (a.b - a.a) * k)
            } else {
                // Rose: the fill grows from a to b, a ghost at b.
                let k = t
                    .tokens
                    .ease(EaseRole::WidgetEnter)
                    .apply((el / dur).clamp(0.0, 1.0));
                (a.a + (a.b - a.a) * k, a.b)
            }
        };
        if frac != a.b {
            // Start from what is on screen.
            let (main, ghost) = shown(&a);
            a.a = if frac < a.b { ghost.max(main) } else { main };
            a.b = frac;
            a.t = now;
        }
        let (main, ghost) = shown(&a);
        self.set_anim(key, a);
        let color = match desc.kind {
            MeterKind::Health if frac < 0.15 => p.danger,
            MeterKind::Health if frac < 0.3 => p.warn,
            MeterKind::Health => p.health,
            MeterKind::Mana => self.env.theme.palette.mana,
            MeterKind::Xp => self.env.theme.palette.xp,
            MeterKind::Primary => p.primary,
            MeterKind::Custom(c) => c,
        };
        // The label in two inks, each the one of fg and inkInverse that reads
        // on what it is drawn over: the fill left of its edge, the track
        // right of it (a light pack's deep status colours take the inverse).
        // Where neither reaches the bar (a mid blue or purple hue, a mid
        // accent), the better one moves on towards its end until it does,
        // else the other (white cannot read on cartoon's purple; black can).
        let behind = self.behind(i);
        let ink = |under: Srgba| {
            let under = under.over(behind);
            let (ink, other) = if p.ink_inverse.contrast(under) > p.ink.contrast(under) {
                (p.ink_inverse, p.ink)
            } else {
                (p.ink, p.ink_inverse)
            };
            // A little over the bar: the ground is the part's estimate (its
            // paints' mean, its art's tone), not the pixels.
            let aim = READABLE + 0.05;
            ink.readable_on(under, aim)
                .or_else(|| other.readable_on(under, aim))
                .unwrap_or(ink)
        };
        let kind = match desc.kind {
            MeterKind::Health => "health",
            MeterKind::Mana => "mana",
            MeterKind::Xp => "xp",
            MeterKind::Primary => "primary",
            MeterKind::Custom(_) => "custom",
        };
        let fill_part =
            crate::dress::PartRef::new("Meter", "fill", crate::tokens::ElementClass::Bar)
                .attr("kind", kind)
                .flag_if(matches!(desc.kind, MeterKind::Health) && frac < 0.3, "low");
        let track_part =
            crate::dress::PartRef::new("Meter", "track", crate::tokens::ElementClass::Bar);
        let (fill_ink, track_ink) = (
            ink(self.part_ground(&fill_part, color)),
            ink(self.part_ground(&track_part, p.control_strong)),
        );
        let n = &mut self.tree.nodes[i];
        if ghost > main {
            let trail = if a.b < a.a || frac < ghost {
                p.ink.fade(0.55)
            } else {
                color.fade(0.4)
            };
            n.shapes.push(Shape::DressedRect {
                frame: Frame::left(0.0, ghost),
                part: crate::dress::PartRef::of("Meter", "trail", None)
                    .attr("kind", kind)
                    .flag_if(a.b < a.a || frac < ghost, "loss"),
                radius: t.radius.bar,
                color: trail,
            });
        }
        if main > 0.0 {
            n.shapes.push(Shape::Part {
                frame: Frame::left(0.0, main),
                part: fill_part,
                fill: Some(color),
            });
        }
        if desc.segments > 1 {
            for k in 1..desc.segments {
                let x = k as f32 / desc.segments as f32;
                n.shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Meter", "separator", None).attr("kind", kind),
                    frame: Frame {
                        fx: x,
                        x: -1.0,
                        w: 2.0,
                        fh: 1.0,
                        ..Frame::default()
                    },
                    radius: 0.0,
                    color: p.backdrop.fade(0.55),
                });
            }
        }
        if !desc.label.is_empty() {
            for (clip, color) in [
                (Frame::left(0.0, main), fill_ink),
                (Frame::left(main, 1.0), track_ink),
            ] {
                if clip.fw > 0.0 {
                    n.shapes.push(Shape::Text {
                        part: crate::dress::PartRef::of("Meter", "label", None)
                            .attr("surface", if clip.fx == 0.0 { "fill" } else { "track" }),
                        frame: Frame::FULL,
                        text: desc.label.clone(),
                        step: t.type_ramp.caption,
                        color,
                        font: FontRole::Mono,
                        clip,
                    });
                }
            }
        }
        n.probe_text = desc.label.clone();
        n.a11y.value = Some(format!("{} of {}", value.round(), max.round()));
        n.a11y.range = Some(crate::a11y::A11yRange {
            min: Some(0.0),
            max: Some(max),
            now: value,
        });
    }

    /// An HTML `<meter>`: a bar of `value` between `min` and `max` whose
    /// fill is dressed by the region [`MeterRegion::of`] puts the value in
    /// (`Meter.fill[region=optimum|suboptimum|poor]`, undressed success,
    /// warn and danger) over `Meter.track`. Role meter with its range.
    pub fn gauge(&mut self, id: impl Into<Id>, desc: &GaugeDesc) {
        let t = self.env.theme;
        let p = t.palette;
        let (min, max) = (desc.min, desc.max.max(desc.min));
        let value = if desc.value.is_finite() {
            desc.value.clamp(min, max)
        } else {
            min
        };
        let region = MeterRegion::of(desc);
        let frac = if max > min {
            (value - min) / (max - min)
        } else {
            0.0
        };
        let i = self.leaf(
            "meter",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::Fixed(160.0)),
                height: Size::Fixed(t.space.md),
                background: Some(p.control_strong),
                radius: t.radius.bar,
                part: Some(crate::dress::PartRef::new(
                    "Meter",
                    "track",
                    crate::tokens::ElementClass::Bar,
                )),
                ..Element::default()
            },
        );
        let (name, color) = match region {
            MeterRegion::Optimum => ("optimum", p.success),
            MeterRegion::Suboptimum => ("suboptimum", p.warn),
            MeterRegion::Poor => ("poor", p.danger),
        };
        let n = &mut self.tree.nodes[i];
        if frac > 0.0 {
            n.shapes.push(Shape::Part {
                frame: Frame::left(0.0, frac),
                part: crate::dress::PartRef::new("Meter", "fill", crate::tokens::ElementClass::Bar)
                    .attr("region", name),
                fill: Some(color),
            });
        }
        n.a11y.value = Some(format!("{value}"));
        n.a11y.range = Some(crate::a11y::A11yRange {
            min: Some(min),
            max: Some(max),
            now: value,
        });
    }

    /// A cooldown: a disc whose primary ring fills clockwise from the top as
    /// the cooldown runs out (`remaining` 1 → 0), with a label or icon in
    /// the middle; full and bright when ready. A press reports `clicked`
    /// (ready or not: the game decides).
    pub fn cooldown(&mut self, id: impl Into<Id>, remaining: f32, desc: &CooldownDesc) -> Response {
        let t = self.env.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 56.0 }.max(self.min_target());
        let i = self.leaf(
            "cooldown",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: true,
                radius: size / 2.0,
                ..Element::default()
            },
        );
        let r = self.nav_target(i, false);
        let rem = if remaining.is_finite() {
            remaining.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let done = 1.0 - rem;
        let behind = self.behind(i);
        // The empty track is the panel's well, a step darker than what it
        // sits on, so the muted sweep has a ground to read against.
        let track = p.well.over(behind);
        // Only a ready ability takes the primary; the cooling sweep is muted
        // ink, so ready, cooling and partial never all shout at once.
        let fill = if rem <= 0.0 { p.primary } else { p.ink_muted };
        let inner = p.panel_raised.over(behind);
        let full = Frame::centered(0.0, 0.0, size, size);
        let mut shapes = vec![Shape::DressedRect {
            part: crate::dress::PartRef::of("Cooldown", "track", None),
            frame: full,
            radius: size,
            color: track,
        }];
        // The pie of `done` from the top, clockwise, as halves of the disc:
        // the right half, then another half that covers back (≤ 1/2) or extends (> 1/2).
        if done >= 1.0 {
            shapes.push(Shape::DressedRect {
                part: crate::dress::PartRef::of("Cooldown", "sweep", None).flag("ready"),
                frame: full,
                radius: size,
                color: fill,
            });
        } else if done > 0.0 {
            let turn = 2.0 * PI * done;
            shapes.push(Shape::Half {
                part: crate::dress::PartRef::of("Cooldown", "sweep", None),
                frame: full,
                angle: 0.0,
                color: fill,
            });
            if done <= 0.5 {
                // Half a pixel larger, so no primary fringe shows at its edge.
                shapes.push(Shape::Half {
                    part: crate::dress::PartRef::of("Cooldown", "mask", None),
                    frame: Frame::centered(0.0, 0.0, size + 1.0, size + 1.0),
                    angle: turn,
                    color: track,
                });
            } else {
                shapes.push(Shape::Half {
                    part: crate::dress::PartRef::of("Cooldown", "sweep", None),
                    frame: full,
                    angle: turn - PI,
                    color: fill,
                });
            }
        }
        let ring = t.space.xs.max(size / 14.0);
        let core = size - 2.0 * ring;
        shapes.push(Shape::DressedRect {
            part: crate::dress::PartRef::of("Cooldown", "core", None)
                .flag_if(r.pressed, "pressed")
                .flag_if(rem <= 0.0, "ready"),
            frame: Frame::centered(0.0, 0.0, core, core),
            radius: core,
            color: if r.pressed {
                p.panel_hover.over(behind)
            } else {
                inner
            },
        });
        let ink = if rem <= 0.0 { p.ink } else { p.ink_muted };
        if !desc.icon.is_empty() {
            let s = core * 0.55;
            shapes.extend(self.icon_shape(
                crate::dress::PartRef::of("Cooldown", "icon", None).flag_if(rem <= 0.0, "ready"),
                &desc.icon,
                Frame::centered(0.0, 0.0, s, s),
                ink,
            ));
        } else if !desc.label.is_empty() {
            shapes.push(Shape::Text {
                part: crate::dress::PartRef::of("Cooldown", "label", None)
                    .flag_if(rem <= 0.0, "ready"),
                frame: Frame::centered(0.0, 0.0, core, core),
                text: desc.label.clone(),
                step: t.type_ramp.lg,
                color: ink,
                font: FontRole::Mono,
                clip: Frame::FULL,
            });
        }
        let n = &mut self.tree.nodes[i];
        n.shapes = shapes;
        n.probe_text = desc.label.clone();
        n.a11y.value = Some(if remaining <= 0.0 {
            "ready".into()
        } else {
            format!("{:.0}%", remaining.clamp(0.0, 1.0) * 100.0)
        });
        r
    }

    /// A zoomable picture (a map, an artwork): two fingers pinch it to zoom
    /// about their centre and pan it as they move (og.Pointers), the wheel
    /// zooms about the pointer, a drag pans; focused by keys or a pad, `+`
    /// (`=`) and `-` zoom it about its centre, `0` fits it again and the
    /// arrows pan it a tenth of its size. It stays on its picture and
    /// within its zoom limits; the zoom and the centre are UI state.
    /// Answers where it is.
    pub fn zoom_view(&mut self, id: impl Into<Id>, desc: &ZoomViewDesc) -> ZoomResponse {
        let i = self.leaf(
            "zoom",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                aspect: desc.aspect.max(0.0),
                // A scroller to the pointer, the wheel and two touches.
                scroll: true,
                ..Element::default()
            },
        );
        let key = self.tree.nodes[i].key;
        let (min, max) = match (desc.min_zoom, desc.max_zoom) {
            (a, b) if a > 0.0 && b >= a => (a, b),
            _ => (1.0, 4.0),
        };
        let s = self.zoom_state(key).unwrap_or(crate::scroll::ZoomState {
            zoom: min,
            center: og_core::glam::Vec2::splat(0.5),
            min,
            max,
        });
        let s = crate::scroll::ZoomState { min, max, ..s };
        self.set_zoom_state(key, s);
        // Keys and pads reach it; while focused its arrows (↑ ↓ too) are
        // its own, with + - and 0.
        let r = self.nav_target(i, true);
        self.tree.nodes[i].multiline = true;
        if r.focused {
            let adjust = self.interaction.nav.adjust;
            self.zoom_keys(key, adjust);
        }
        let s = self.zoom_state(key).unwrap_or(s);
        let [u0, v0, u1, v1] = desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let half = 0.5 / s.zoom;
        let (fx0, fy0) = (s.center.x - half, s.center.y - half);
        let (fx1, fy1) = (s.center.x + half, s.center.y + half);
        let n = &mut self.tree.nodes[i];
        n.content = Content::Image {
            texture: desc.texture,
            uv: [
                u0 + (u1 - u0) * fx0,
                v0 + (v1 - v0) * fy0,
                u0 + (u1 - u0) * fx1,
                v0 + (v1 - v0) * fy1,
            ],
            tint: desc.tint.unwrap_or(Srgba::WHITE),
            object: Default::default(),
        };
        // A picture named by its id (a map), its zoom as its value.
        n.a11y.role = Some("image");
        n.a11y.name = Some(n.id.clone());
        n.a11y.value = Some(format!("{:.0}%", s.zoom * 100.0));
        ZoomResponse {
            zoom: s.zoom,
            center: s.center,
        }
    }

    /// A mini-map frame: the map texture (or the stage colour) in a
    /// `lineStrong` frame, markers as dots (the player primary and ringed),
    /// and an optional zone label under it.
    pub fn minimap(&mut self, id: impl Into<Id>, desc: &MinimapDesc) {
        let t = self.env.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 160.0 };
        self.open(
            id,
            Element {
                direction: Direction::Column,
                gap: t.space.xs,
                cross: Cross::Center,
                ..Element::default()
            },
        );
        let i = self.leaf(
            "map",
            Id::from("Map"),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                padding: Insets::all(2.0),
                // Floating chrome over the game, a HUD plate with a lift in
                // the default style (not glass: a minimap also sits in
                // panels, and glass never nests), not a control's outline
                // round a black well: the map's own texture fills it.
                background: Some(p.hud),
                radius: t.radius.control,
                part: Some(crate::dress::PartRef::new(
                    "Minimap",
                    "frame",
                    crate::tokens::ElementClass::Hud,
                )),
                ..Element::default()
            },
        );
        let mut shapes = Vec::new();
        if let Some(tex) = desc.texture {
            shapes.push(Shape::Image {
                part: crate::dress::PartRef::of("Minimap", "image", None),
                frame: Frame {
                    x: 4.0,
                    y: 4.0,
                    fw: 1.0,
                    fh: 1.0,
                    w: -8.0,
                    h: -8.0,
                    ..Frame::default()
                },
                texture: tex,
                uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
                color: Srgba::WHITE,
            });
        }
        let span = size - 8.0;
        for m in &desc.markers {
            let (x, y) = (
                (m.x.clamp(0.0, 1.0) - 0.5) * span,
                (m.y.clamp(0.0, 1.0) - 0.5) * span,
            );
            if m.player {
                let d = t.space.md;
                shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Minimap", "markerOutline", None)
                        .flag("player"),
                    frame: Frame::centered(x, y, d + 4.0, d + 4.0),
                    radius: d,
                    color: p.backdrop,
                });
                shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Minimap", "marker", None).flag("player"),
                    frame: Frame::centered(x, y, d, d),
                    radius: d,
                    color: p.primary,
                });
            } else {
                let d = t.space.sm;
                let c = if m.tone == Tone::Neutral {
                    p.ink
                } else {
                    self.tone_dot(m.tone)
                };
                shapes.push(Shape::DressedRect {
                    part: crate::dress::PartRef::of("Minimap", "marker", None)
                        .attr("tone", m.tone.name()),
                    frame: Frame::centered(x, y, d, d),
                    radius: d,
                    color: c,
                });
            }
        }
        shapes.push(Shape::Text {
            part: crate::dress::PartRef::of("Minimap", "north", None),
            frame: Frame {
                fx: 0.5,
                x: -10.0,
                y: 6.0,
                w: 20.0,
                h: 14.0,
                ..Frame::default()
            },
            text: "N".into(),
            step: t.type_ramp.xs,
            color: p.ink_muted,
            font: FontRole::Body,
            clip: Frame::FULL,
        });
        self.tree.nodes[i].shapes = shapes;
        if !desc.label.is_empty() {
            self.label(
                "Zone",
                &desc.label,
                crate::LabelDesc {
                    step: Some(t.type_ramp.meta),
                    color: Some(p.ink_muted),
                    nowrap: true,
                    ..Default::default()
                },
            );
        }
        self.close();
    }
}
