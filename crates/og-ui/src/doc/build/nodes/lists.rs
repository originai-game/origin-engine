use super::*;

impl Ui {
    /// An `<li>`'s marker: hung before its start edge (`outside`, CSS's
    /// default: anchored to the item, so it costs the item no room), or
    /// first in its row (`inside`). A disc, circle or square is a shape on
    /// the first line, as browsers paint it, so no font has to carry the
    /// bullet; a number is text.
    pub(crate) fn list_marker(
        &mut self,
        item: &str,
        kind: &str,
        text: &str,
        inside: bool,
        ink: Option<Srgba>,
    ) {
        let t = *self.env.theme();
        let body = t.type_ramp.body;
        // CSS's gap between marker and content: half an em.
        let gap = (body.size * 0.5).round();
        if crate::doc::lists::numbered(kind) {
            let text = if inside {
                format!("{text}\u{a0}")
            } else {
                text.to_string()
            };
            self.label(
                "Marker",
                &text,
                crate::LabelDesc {
                    color: ink,
                    ..crate::LabelDesc::default()
                },
            );
        } else {
            // Blink's bullet: a third of an em across, at least 3 px.
            let d = (body.size / 3.0).round().max(3.0);
            let ink = ink.unwrap_or(t.palette.ink);
            self.open(
                "Marker",
                Element {
                    width: Size::px(if inside { d + gap } else { d }),
                    height: Size::px(body.line_height),
                    cross: crate::layout::Cross::Center,
                    ..Element::default()
                },
            );
            self.open(
                kind,
                Element {
                    width: Size::px(d),
                    height: Size::px(d),
                    background: (kind == "square").then_some(ink),
                    ..Element::default()
                },
            );
            if kind != "square" {
                use crate::paint::{
                    Cap, ColorRef, ElementDraw, Paint, ShapeKind, Stroke, StrokeAlign,
                };
                let paint = Paint::Solid(ColorRef::Literal(ink));
                self.set_draw(ElementDraw {
                    shape: ShapeKind::Ellipse,
                    fills: if kind == "disc" {
                        vec![paint.clone()]
                    } else {
                        Vec::new()
                    },
                    strokes: if kind == "circle" {
                        vec![Stroke {
                            width: 1.0,
                            align: StrokeAlign::Inside,
                            dash: [0.0; 2],
                            dash_offset: 0.0,
                            dash_cap: Cap::Butt,
                            paint,
                        }]
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                });
            }
            self.close();
            self.close();
        }
        if inside {
            return;
        }
        let at = self.tree.last_closed();
        let props = self.tree.props_mut(at);
        props.position = crate::element::Position::Anchored;
        props.anchor_to = item.to_string();
        props.anchor_at = crate::layout::Anchor::TopLeft;
        props.anchor_self = crate::layout::Anchor::TopRight;
        props.insets.start = Some(-gap);
        props.overlaps.push(item.to_string());
    }
}

impl crate::ui::Docs {
    pub(crate) fn art_spec(&mut self, path: &str, r: &Resolved) {
        let s = crate::slot::SlotSpec::from_props(
            path,
            |p| r.str(p).map(str::to_string),
            |p| r.nums(p).map(<[f32]>::to_vec),
        );
        self.note_art(path.to_string(), s);
    }
}
