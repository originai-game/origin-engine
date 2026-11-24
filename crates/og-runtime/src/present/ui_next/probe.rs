//! Probe inspection (`dump_ui`, op boxes), accessibility updates, and
//! host text-input target queries.

use super::*;

impl Next {
    /// The focused `Input` for the host shell's text-entry overlay
    /// (`packages/engine/src/web/text-input.ts`), in CSS px, or `Value::Null`.
    pub(crate) fn text_target(&self) -> Value {
        let Some(mut target) = self.tree.as_ref().and_then(|t| t.text_target()) else {
            return Value::Null;
        };
        let u = self.unit;
        if u != 1.0 {
            for v in target.rect.iter_mut().chain(&mut target.caret) {
                *v *= u;
            }
        }
        target.to_json()
    }

    /// Text copied or cut from a focused `Input` since the last frame.
    pub(crate) fn take_copy(&mut self) -> Option<String> {
        self.tree.as_mut().and_then(|t| t.take_copy())
    }

    /// The accessibility tree's update for the web shell's mirror
    /// ([`og_ui_next::a11y::web`]), when what it shows changed since the
    /// last one; in CSS px, as the tree lays out.
    pub(crate) fn a11y(&mut self) -> Option<Value> {
        let tree = self.tree.as_ref()?;
        let update = tree.a11y(&mut self.a11y, tree.focused(), self.unit)?;
        Some(og_ui_next::a11y::web(&update))
    }

    /// The accessibility update for a native window's AccessKit adapter
    /// (`client`: what that session was sent), at `width` physical pixels
    /// across: the tree's CSS px scaled to them by its root.
    pub(crate) fn a11y_native(
        &self,
        client: &mut og_ui_next::a11y::Tree,
        width: u32,
    ) -> Option<og_ui_next::a11y::accesskit::TreeUpdate> {
        let tree = self.tree.as_ref()?;
        let css = tree.size().0.max(1.0);
        tree.a11y(client, tree.focused(), width as f32 / css)
    }

    /// The spans as the probe's timings name them.
    pub(crate) fn timings(&self) -> Vec<(String, f64)> {
        let s = &self.spans;
        vec![
            ("og.ui.next.build".into(), s.build),
            ("og.ui.next.frame".into(), s.frame),
            ("og.ui.next.commit".into(), s.commit),
            ("og.ui.next.layout".into(), s.layout),
        ]
    }

    /// The TSX renderer's nodes, as the probe's UI dump lists them.
    pub(crate) fn elements(&self) -> Vec<og_probe::UiElement> {
        self.tree
            .as_ref()
            .map(|tree| Self::op_elements(tree, self.unit).0)
            .unwrap_or_default()
    }

    /// The TSX renderer's nodes that show, and the focus order as their
    /// paths, with the accessibility tree built whole now (only when the
    /// probe asks). Each is what a game-making agent reads: its op id, its
    /// `path` of op ids (what `touch_move_to` and the captures name), its
    /// box and text; a control's `role`; `disabled`, `checked` and
    /// `focused` among its states; its name only where its text is not it.
    fn op_elements(tree: &Tree, unit: f32) -> (Vec<og_probe::UiElement>, Vec<String>) {
        /// The roles a player operates: what the model may press or move.
        const CONTROLS: &[&str] = &[
            "button",
            "link",
            "checkbox",
            "radio",
            "switch",
            "slider",
            "scrollbar",
            "textbox",
            "combobox",
            "option",
            "tab",
            "spinbutton",
            "menuitem",
            "listbox",
            "grid",
            "tree",
            "radiogroup",
        ];
        const STATES: &[&str] = &["disabled", "checked", "focused"];
        let (facts, order) = tree.a11y_snapshot();
        let ops = tree.dump_ops();
        let parent: std::collections::HashMap<u32, u32> =
            ops.iter().map(|n| (n.id, n.parent)).collect();
        let path = |mut id: u32| {
            let mut ids = vec![id];
            while let Some(&p) = parent.get(&id).filter(|&&p| p != 0) {
                ids.push(p);
                id = p;
            }
            ids.iter()
                .rev()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join("/")
        };
        let by_node: std::collections::HashMap<_, _> = ops.iter().map(|n| (n.node, n.id)).collect();
        let order = order
            .iter()
            .filter_map(|node| by_node.get(node).map(|&id| path(id)))
            .collect();
        let elements = ops
            .iter()
            // What a scroller or a cut box hides entirely is not on screen;
            // a box that is empty of itself (a text with no face to measure
            // it, a zero-size view) still is, where it is laid out.
            .filter(|n| {
                n.shown
                    && !(n.clipped
                        && (n.visible[2] <= n.visible[0] || n.visible[3] <= n.visible[1]))
            })
            .map(|n| {
                let f = facts.get(&n.node);
                let control = f
                    .is_some_and(|f| CONTROLS.contains(&f.role) || f.states.contains(&"focusable"));
                // In the surface's CSS px, as og-ui's elements are.
                // The part that shows on screen; clipped when its
                // ancestors' cuts take some of it.
                let [x0, y0, x1, y1] = n.visible;
                let clipped = n.clipped;
                let [x, y, width, height] = [x0, y0, x1 - x0, y1 - y0].map(|v| v * unit);
                og_probe::UiElement {
                    id: n.id.to_string(),
                    parent: if n.parent == 0 {
                        String::new()
                    } else {
                        n.parent.to_string()
                    },
                    path: path(n.id),
                    text: n.text.into(),
                    role: f.filter(|_| control).map_or("", |f| f.role).into(),
                    name: f
                        .filter(|f| control && f.name != n.text)
                        .map_or(String::new(), |f| f.name.clone()),
                    states: f.map_or(Vec::new(), |f| {
                        let mut s: Vec<String> = f
                            .states
                            .iter()
                            .filter(|s| STATES.contains(s))
                            .map(|s| s.to_string())
                            .collect();
                        s.dedup();
                        s
                    }),
                    disabled: f.is_some_and(|f| f.states.contains(&"disabled")),
                    clipped,
                    x,
                    y,
                    width,
                    height,
                    detail: {
                        let mut d = serde_json::Map::new();
                        // A word its lines cut where UAX #14 has no break.
                        if !n.broken.is_empty() {
                            d.insert("brokenWords".into(), json!(n.broken));
                        }
                        // The screen rectangle its ancestors' scrollers and
                        // overflow cut it to (x, y, width, height): of its
                        // textLines, only what lies inside is drawn.
                        if let Some([cx0, cy0, cx1, cy1]) = n.clip {
                            d.insert(
                                "clip".into(),
                                json!([cx0, cy0, cx1 - cx0, cy1 - cy0].map(|v| v * unit)),
                            );
                        }
                        // Its text as drawn, for reading its ground in a
                        // frame: the ink as sRGB 0-255 with alpha 0-1, the
                        // font size and each line's box, in CSS px, before
                        // its transform.
                        if !n.lines.is_empty() {
                            let [r, g, b, a] = n.ink;
                            let srgb = og_core::color::linear_to_srgb8;
                            d.insert("ink".into(), json!([srgb(r), srgb(g), srgb(b), a]));
                            d.insert("fontSize".into(), json!(n.size * unit));
                            let lines: Vec<[f32; 4]> =
                                n.lines.iter().map(|l| l.map(|v| v * unit)).collect();
                            d.insert("textLines".into(), json!(lines));
                            // A transformed text's lines are boxes through
                            // this matrix (the quad it draws in).
                            if let Some([a, b, c, d2, tx, ty]) = n.transform {
                                d.insert(
                                    "textTransform".into(),
                                    json!([a, b, c, d2, tx * unit, ty * unit]),
                                );
                            }
                        }
                        // Its motion facts (og-ui-next `OpNode`): a press
                        // squashes it, and it enters from a `starting:`
                        // record and leaves to an `exit:` one.
                        if n.press {
                            d.insert("press".into(), json!(true));
                        }
                        let presence: Vec<&str> = ["enter", "exit"]
                            .into_iter()
                            .zip(n.presence)
                            .filter_map(|(k, on)| on.then_some(k))
                            .collect();
                        if !presence.is_empty() {
                            d.insert("presence".into(), json!(presence));
                        }
                        if n.modal {
                            d.insert("modal".into(), json!(true));
                        }
                        // The fill its nine-sliced frame is drawn in, sRGB
                        // 0-255 with alpha 0-1: a nested frame on the same
                        // ground as its parent's does not show.
                        if let Some([r, g, b, a]) = n.frame {
                            let srgb = og_core::color::linear_to_srgb8;
                            d.insert("frame".into(), json!([srgb(r), srgb(g), srgb(b), a]));
                        }
                        d.into_iter().collect()
                    },
                    ..Default::default()
                }
            })
            .collect();
        (elements, order)
    }

    /// A TSX node's box on the UI's surface, in CSS px, as last laid out
    /// (the inspector's panel): the tree's og-ui units times the unit.
    #[cfg(not(feature = "publish"))]
    pub(crate) fn op_box(&self, id: u32) -> Option<[f32; 4]> {
        let b = self.tree.as_ref()?.op_box(id)?;
        Some(b.map(|v| v * self.unit))
    }

    /// The new core's op nodes as the probe lists them, and the focus
    /// order as their paths: for `og ui check`.
    pub(crate) fn op_dump(&self) -> (Vec<og_probe::UiElement>, Vec<String>) {
        self.tree
            .as_ref()
            .map(|tree| Self::op_elements(tree, self.unit))
            .unwrap_or_default()
    }

    /// Whether the TSX tree's motion is still on its way to rest (finite
    /// animations, transitions and exits; never a loop): `dump_ui`'s
    /// `moving`.
    pub(crate) fn moving(&self) -> bool {
        self.tree.as_ref().is_some_and(|t| t.settling())
    }

    /// What `__runtime_probe__` reads beside the timings.
    pub(crate) fn probe(&self) -> Value {
        let s = &self.spans;
        let p = &self.paint;
        let (mut solid, mut gradient, mut glyph, mut icon) = (0usize, 0usize, 0usize, 0usize);
        let (mut border, mut shadow, mut image, mut line, mut group) =
            (0usize, 0usize, 0usize, 0usize, 0usize);
        for q in &p.quads {
            match q.kind {
                k if k == og_render::paint::PaintKind::Solid as u32 => solid += 1,
                k if k == og_render::paint::PaintKind::Gradient as u32 => gradient += 1,
                k if k == og_render::paint::PaintKind::Raster as u32 => glyph += 1,
                k if k == og_render::paint::PaintKind::Icon as u32 => icon += 1,
                k if k == og_render::paint::PaintKind::Border as u32 => border += 1,
                k if k == og_render::paint::PaintKind::Shadow as u32
                    || k == og_render::paint::PaintKind::InsetShadow as u32 =>
                {
                    shadow += 1;
                }
                k if k == og_render::paint::PaintKind::Image as u32 => image += 1,
                k if k == og_render::paint::PaintKind::Line as u32 => line += 1,
                k if k == og_render::paint::PaintKind::Group as u32 => group += 1,
                _ => {}
            }
        }
        // The first glyphs' places, as the paint pass reads them.
        let glyphs: Vec<Value> = p
            .quads
            .iter()
            .filter(|q| q.kind == og_render::paint::PaintKind::Raster as u32)
            .take(4)
            .map(|q| {
                json!({
                    "x": q.rect[0], "y": q.rect[1], "width": q.rect[2], "height": q.rect[3],
                    "color": q.color, "clip": q.clip, "tier": q.a, "page": q.b,
                })
            })
            .collect();
        let kinds = json!({
            "solid": solid,
            "gradient": gradient,
            "glyph": glyph,
            "icon": icon,
            "border": border,
            "shadow": shadow,
            "image": image,
            "line": line,
            "group": group,
        });
        json!({
            "build_ms": s.build, "build_allocations": s.build_allocations,
            "frame_ms": s.frame, "commit_ms": s.commit, "layout_ms": s.layout,
            "allocations": s.allocations,
            "elements": s.elements, "quads": p.quads.len(), "gradients": p.gradients.len(),
            "kinds": kinds, "font": self.font, "glyphs": glyphs,
            "generation": p.generation,
        })
    }
}
