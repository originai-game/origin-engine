use super::*;

/// What a list item's marker shows.
enum Mark {
    None,
    /// Its text, and the list style that made it (`disc`, `circle` and
    /// `square` draw as shapes).
    Text(String, String),
    /// A `list-style-image`.
    Picture(String),
}

impl Ui {
    /// A list item's marker, its ink and whether it is inside: `::marker`'s
    /// content, else its list style's text for its number (an `<li>`'s
    /// from its list, another's from the run's `list-item` counter)
    /// without the suffix's trailing space, which the marker's gap stands
    /// for, else its `list-style-image`.
    fn item_mark(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
    ) -> (Mark, Option<Srgba>, bool) {
        let kind = r.str("markerType").unwrap_or("disc");
        let ordinal = if n.selector_tag.as_deref() == Some("li") {
            r.ordinal.unwrap_or(1)
        } else {
            self.docs.state_mut().run.read("list-item").unwrap_or(1)
        };
        let (content, ink) = self.marker_override(b, n, r);
        let inside = r.str("markerPosition") == Some("inside");
        let picture = r
            .str("markerImage")
            .and_then(|v| crate::doc::lists::image(v).ok().flatten());
        let mark = match (content, picture) {
            (Some(Some(text)), _) => Mark::Text("decimal".into(), text),
            (Some(None), _) => Mark::None,
            (None, Some(name)) => Mark::Picture(name),
            (None, None) if kind == "none" => Mark::None,
            (None, None) => Mark::Text(
                kind.into(),
                crate::doc::generated::marker(ordinal, kind, &b.doc.sheet)
                    .trim_end()
                    .to_string(),
            ),
        };
        (mark, ink, inside)
    }

    /// A list item's marker, element `id`, hung from `item` (outside) or
    /// first in its row (inside).
    pub(crate) fn item_marker(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        id: &str,
        item: &str,
    ) {
        match self.item_mark(b, n, r) {
            (Mark::None, ..) => {}
            (Mark::Text(kind, text), ink, inside) => {
                self.list_marker(id, item, &kind, &text, inside, ink);
            }
            (Mark::Picture(name), _, inside) => {
                let text = if inside { "\u{FFFC}\u{a0}" } else { "\u{FFFC}" };
                let span = crate::TextSpan {
                    name: "marker".into(),
                    position: 0,
                    length: 1,
                    properties: Default::default(),
                };
                let style = crate::TextSpanStyle {
                    atom: Some(crate::rich::InlineAtom::Image(name)),
                    ..Default::default()
                };
                self.rich_text_spans(
                    id,
                    text,
                    std::slice::from_ref(&span),
                    crate::RichTextDesc::default(),
                    |_| style.clone(),
                );
            }
        }
    }

    /// A text list item's inside marker, as its text's first run: the
    /// marker's text (a bullet as its character) and a no-break space, in
    /// the marker's ink, or its picture.
    pub(crate) fn inside_text_marker(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
    ) -> Option<(String, crate::TextSpanStyle)> {
        match self.item_mark(b, n, r) {
            (Mark::Text(_, text), ink, true) => Some((
                format!("{text}\u{a0}"),
                crate::TextSpanStyle {
                    color: ink.map(crate::paint::ColorRef::Literal),
                    ..Default::default()
                },
            )),
            (Mark::Picture(name), _, true) => Some((
                "\u{FFFC}\u{a0}".into(),
                crate::TextSpanStyle {
                    atom: Some(crate::rich::InlineAtom::Image(name)),
                    ..Default::default()
                },
            )),
            _ => None,
        }
    }

    /// An `<li>`'s marker: hung before its start edge (`outside`, CSS's
    /// default: anchored to the item, so it costs the item no room), or
    /// first in its row (`inside`). A disc, circle or square is a shape on
    /// the first line, as browsers paint it, so no font has to carry the
    /// bullet; a number is text.
    pub(crate) fn list_marker(
        &mut self,
        id: &str,
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
        if let Some(open) = match kind {
            "disclosure-open" => Some(true),
            "disclosure-closed" => Some(false),
            _ => None,
        } {
            // HTML's disclosure triangle, as the theme's disclosure draws it.
            let side = self.by_text(t.controls.details_icon);
            self.open(
                id,
                Element {
                    width: Size::px(if inside { side + gap } else { side }),
                    height: Size::px(body.line_height.max(side)),
                    cross: crate::layout::Cross::Center,
                    ..Element::default()
                },
            );
            self.disclosure_marker(kind, ink, open);
            self.close();
        } else if crate::doc::lists::numbered(kind) {
            let text = if inside {
                format!("{text}\u{a0}")
            } else {
                text.to_string()
            };
            self.label(
                id,
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
                id,
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
