use super::*;

pub(crate) trait NodesListsUi {
    /// An `<li>`'s marker: hung before its start edge (`outside`, CSS's
    /// default: anchored to the item, so it costs the item no room), or
    /// first in its row (`inside`). A disc, circle or square is a shape on
    /// the first line, as browsers paint it, so no font has to carry the
    /// bullet; a number is text.
    fn list_marker(&mut self, item: &str, kind: &str, text: &str, inside: bool, ink: Option<Srgba>);
}

impl NodesListsUi for Ui {
    fn list_marker(
        &mut self,
        item: &str,
        kind: &str,
        text: &str,
        inside: bool,
        ink: Option<Srgba>,
    ) {
        let t = self.env.theme;
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
        let at = self.tree.last_closed;
        let props = self.tree.props_mut(at);
        props.position = crate::element::Position::Anchored;
        props.anchor_to = item.to_string();
        props.anchor_at = crate::layout::Anchor::TopLeft;
        props.anchor_self = crate::layout::Anchor::TopRight;
        props.insets.start = Some(-gap);
        props.overlaps.push(item.to_string());
    }
}

pub(crate) trait NodesListsUi2 {
    /// One item of a `for` outside a virtualised container.
    fn build_item(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        var: &str,
        item: Value,
        k: usize,
        origin: Option<&str>,
    );

    /// A `list` or `cells` and its `for`: only the rows in view are built.
    #[allow(clippy::too_many_arguments)]
    fn build_virtual(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
    );
}

impl NodesListsUi2 for Ui {
    fn build_item(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        var: &str,
        item: Value,
        k: usize,
        origin: Option<&str>,
    ) {
        b.locals
            .push((var.to_string(), item, origin.map(|o| format!("{o}.{k}"))));
        b.locals.push(("$index".into(), json!(k), None));
        let NodeKind::For { key, .. } = &n.kind else {
            unreachable!()
        };
        let row_key = b.eval(key);
        let Some(row_key) = row_key.as_str().filter(|key| !key.is_empty()) else {
            self.out.warnings.push(format!(
                "{}:{}: each key must be a nonempty string",
                b.doc.source, n.line
            ));
            b.locals.pop();
            b.locals.pop();
            return;
        };
        let outer_key = std::mem::replace(&mut b.row_key, row_key.into());
        let doc = b.doc.clone();
        for &c in &n.children {
            let id = match &doc.nodes[c].kind {
                NodeKind::Element(kind) if doc.nodes[c].id.is_empty() => {
                    Some(format!("{kind}#{row_key}"))
                }
                NodeKind::Element(_) | NodeKind::Use(_) => {
                    Some(format!("{}#{row_key}", doc.nodes[c].id))
                }
                _ => None,
            };
            self.build_node(b, c, id.as_deref());
        }
        b.row_key = outer_key;
        b.locals.pop();
        b.locals.pop();
    }

    #[allow(clippy::too_many_arguments)]
    fn build_virtual(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
    ) {
        let doc = b.doc.clone();
        let Some(&f) = n.children.first() else { return };
        let NodeKind::For { var, list, .. } = &doc.nodes[f].kind else {
            return;
        };
        let items = b.eval(list).as_array().cloned().unwrap_or_default();
        let origin = match list {
            crate::bind::Expr::Path(p) => b.write_path(p),
            _ => None,
        };
        let count = items.len() as u32;
        let padding = r.nums("padding").map(insets_of).unwrap_or_default();
        let range = if kind == "list" {
            self.open_list(
                id,
                crate::widgets::ListDesc {
                    count,
                    row_height: r.num("rowHeight").unwrap_or(0.0),
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        } else {
            self.open_grid(
                id,
                crate::widgets::GridDesc {
                    count,
                    cell_width: r.num("cellWidth").unwrap_or(0.0),
                    cell_height: r.num("cellHeight").unwrap_or(0.0),
                    columns: r.num("cellColumns").unwrap_or(0.0) as u32,
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
            )
        };
        let at = self.tree.parent();
        if props != Props::default() {
            *self.tree.props_mut(at) = props;
        }
        for k in range.first..range.last.min(count) {
            self.docs.structure_row(k as usize);
            let resp = if kind == "list" {
                self.list_item(k, false)
            } else {
                self.grid_item(k, false)
            };
            let row = format!(
                "{path}/{}",
                if kind == "list" {
                    crate::ui_idi("Row", k)
                } else {
                    crate::ui_idi("Cell", k)
                }
                .as_str()
            );
            if resp.clicked {
                b.event(&row, "select", id, json!(k));
            }
            self.build_item(
                b,
                &doc.nodes[f],
                var,
                items[k as usize].clone(),
                k as usize,
                origin.as_deref(),
            );
            self.close();
        }
        self.close_list();
    }
}

pub(crate) trait NodesListsDocs {
    fn art_spec(&mut self, path: &str, r: &Resolved);
}

impl NodesListsDocs for crate::ui::Docs {
    fn art_spec(&mut self, path: &str, r: &Resolved) {
        let s = crate::slot::SlotSpec::from_props(
            path,
            |p| r.str(p).map(str::to_string),
            |p| r.nums(p).map(<[f32]>::to_vec),
        );
        self.note_art(path.to_string(), s);
    }
}
