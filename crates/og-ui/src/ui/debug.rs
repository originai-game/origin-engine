//! What the visual UI debugger reads of the last finished frame (docs/ui.md
//! §6.4): the element under a point, one element's box model, its computed
//! styles with where each was written, its bindings and the layout
//! boundaries it sets. Every read is on demand, for the one element asked
//! about (by its render key, no walk of the tree), except picking, which
//! walks the drawn elements once per pick. Built on [`Ui::elements`] and
//! [`Ui::element_bounds`]; nothing here runs while no debugger asks.

use super::*;

/// One element as the debugger shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDebugElement {
    pub path: String,
    pub kind: String,
    /// Its border box on screen, og-ui units, before its clip.
    pub border_box: Rect,
    /// Room outside the border box (CSS margins as laid out).
    pub margin: Insets,
    pub border: Insets,
    pub padding: Insets,
    /// The border box less its border and padding.
    pub content: Rect,
    /// The document cascade's winning declarations, in the order they
    /// apply, each with `file:line`; empty for an element no document
    /// built.
    pub styles: Vec<UiDebugStyle>,
    /// The style pack's rules that dress its part, the last winning a
    /// property both set, each with where it was written.
    pub rules: Vec<UiDebugStyle>,
    /// `(property, expression as written, value now)`.
    pub bindings: Vec<(String, String, String)>,
    /// What bounds its subtree: `scroll` (a scroll container clips and
    /// scrolls it), `container` (its size answers container queries),
    /// `layer` (a layer's root, laid out on its own), `fixed` (its width
    /// and height are fixed, so its children's layout cannot move what is
    /// outside it).
    pub boundaries: Vec<&'static str>,
}

/// Bindings as the debugger shows them: `(property, expression as written,
/// value now)`.
pub(crate) type DebugBindings = Vec<(String, String, String)>;

/// One declaration and where it came from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDebugStyle {
    /// The property, or a style rule's selector.
    pub name: String,
    pub value: String,
    /// `file:line` (a document's source, a style file's).
    pub source: String,
}

impl Ui {
    /// The path of the topmost element drawn at `(x, y)` (og-ui units) in
    /// the last finished frame, inside its clip; `None` over nothing.
    pub fn debug_pick(&self, x: f32, y: f32) -> Option<String> {
        self.elements()
            .iter()
            .rev()
            .find(|e| {
                e.kind != "part"
                    && !e.clipped
                    && x >= e.x
                    && y >= e.y
                    && x < e.x + e.width
                    && y < e.y + e.height
            })
            .map(|e| e.path.clone())
    }

    /// The element at `path` as the debugger shows it; `None` when the
    /// last finished frame did not draw it.
    pub fn debug_element(&self, path: &str) -> Option<UiDebugElement> {
        let key = crate::id::key_of_path(path);
        let ui = self.finished();
        let i = ui.tree.index.by_key.get(&key).copied()?;
        let n = ui.tree.nodes.get(i)?;
        let r = n.placed.screen;
        // A margin's percentage reads its parent's content width.
        let basis = ui.tree.nodes.get(n.parent).map_or(r.width, |p| {
            p.placed.screen.width - p.el.padding.left - p.el.padding.right
        });
        let [mt, mr, mb, ml] = n.el.margin.map(|m| m.of(basis));
        let line = |k: usize| {
            n.decor
                .as_ref()
                .and_then(|d| d.borders[k].as_ref())
                .map_or(0.0, |l| l.width)
        };
        // CSS order: top, right, bottom, left.
        let border = Insets {
            top: line(0),
            right: line(1),
            bottom: line(2),
            left: line(3),
        };
        // Layout's padding holds the border inside it (a border takes room
        // as CSS's does): the CSS padding is what is left.
        let inner = n.el.padding;
        let p = Insets {
            left: (inner.left - border.left).max(0.0),
            top: (inner.top - border.top).max(0.0),
            right: (inner.right - border.right).max(0.0),
            bottom: (inner.bottom - border.bottom).max(0.0),
        };
        let content = Rect {
            x: r.x + inner.left,
            y: r.y + inner.top,
            width: (r.width - inner.left - inner.right).max(0.0),
            height: (r.height - inner.top - inner.bottom).max(0.0),
        };
        let (styles, bindings) = ui.docs.host.retained.debug_styles(key).unwrap_or_default();
        // The part it is drawn as, beside it in the element list.
        let at = self.elements().iter().position(|e| e.path == path);
        let rules = at
            .and_then(|at| self.parts().get(at).cloned().flatten())
            .map(|part| {
                ui.matching_part_rules(&part)
                    .into_iter()
                    .map(|rule| UiDebugStyle {
                        name: rule.structure.as_ref().unwrap().source.clone(),
                        value: String::new(),
                        source: format!("<part-css>:{}", rule.line),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut boundaries = Vec::new();
        if n.el.scroll || n.el.scroll_x {
            boundaries.push("scroll");
        }
        if n.props.container {
            boundaries.push("container");
        }
        if n.parent == crate::layout::NONE || ui.tree.floating.contains(&i) {
            boundaries.push("layer");
        }
        if matches!(n.el.width, crate::Size::Fixed(_))
            && matches!(n.el.height, crate::Size::Fixed(_))
        {
            boundaries.push("fixed");
        }
        Some(UiDebugElement {
            path: path.to_owned(),
            kind: ui.tree.kinds.get(i).map_or("", |k| k.0).to_owned(),
            border_box: r,
            margin: Insets {
                top: mt,
                right: mr,
                bottom: mb,
                left: ml,
            },
            border,
            padding: p,
            content,
            styles,
            rules,
            bindings,
            boundaries,
        })
    }
}

/// Draws a UI debugger's view into `ui` (a debugger's own UI, built over
/// the game's): `bounds` outlined (layout bounds), then `hover`'s and
/// `picked`'s margin, border, padding and content boxes in Chrome's
/// colours. The rectangles are CSS px; `unit` is `ui`'s (CSS px over it
/// are its units). Call it between `begin` and `end`, at the top level:
/// each box is placed on the screen, out of the flow.
pub fn debug_overlay(
    ui: &mut Ui,
    picked: Option<&UiDebugElement>,
    hover: Option<&UiDebugElement>,
    bounds: &[Rect],
    unit: f32,
) {
    let u = unit.max(1.0);
    let line = crate::Srgba {
        rgb: [242, 89, 140],
        alpha: 0.9,
    };
    for (k, r) in bounds.iter().enumerate() {
        outline(ui, &format!("Bound{k}"), *r, u, line);
    }
    for (id, e) in [("Hover", hover), ("Picked", picked)] {
        let Some(e) = e else { continue };
        let b = e.border_box;
        let grow = |r: Rect, i: Insets| Rect {
            x: r.x - i.left,
            y: r.y - i.top,
            width: r.width + i.left + i.right,
            height: r.height + i.top + i.bottom,
        };
        let shrink = |r: Rect, i: Insets| Rect {
            x: r.x + i.left,
            y: r.y + i.top,
            width: (r.width - i.left - i.right).max(0.0),
            height: (r.height - i.top - i.bottom).max(0.0),
        };
        // Each box is drawn as the ring between it and the next box in,
        // as Chrome does, so the content keeps one tint and reads under it.
        let padding = shrink(b, e.border);
        for (part, r, inner, c) in [
            (
                "Margin",
                grow(b, e.margin),
                Some(b),
                crate::Srgba {
                    rgb: [245, 178, 107],
                    alpha: 0.45,
                },
            ),
            (
                "Border",
                b,
                Some(padding),
                crate::Srgba {
                    rgb: [252, 217, 140],
                    alpha: 0.55,
                },
            ),
            (
                "Padding",
                padding,
                Some(e.content),
                crate::Srgba {
                    rgb: [194, 207, 138],
                    alpha: 0.6,
                },
            ),
            (
                "Content",
                e.content,
                None,
                crate::Srgba {
                    rgb: [138, 178, 201],
                    alpha: 0.65,
                },
            ),
        ] {
            match inner {
                Some(inner) => ring(ui, &format!("{id}{part}"), r, inner, u, c),
                None => fill(ui, &format!("{id}{part}"), r, u, c),
            }
        }
    }
}

fn place(r: Rect, u: f32) -> (crate::Element, crate::element::Props) {
    (
        crate::Element {
            width: crate::Size::Fixed((r.width / u).max(0.0)),
            height: crate::Size::Fixed((r.height / u).max(0.0)),
            ..crate::Element::default()
        },
        crate::element::Props {
            position: crate::element::Position::Screen,
            insets: crate::element::Edges {
                start: Some(r.x / u),
                top: Some(r.y / u),
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn fill(ui: &mut Ui, id: &str, r: Rect, u: f32, color: crate::Srgba) {
    let (mut el, props) = place(r, u);
    el.background = Some(color);
    ui.open_with(id, el, props);
    ui.close();
}

fn outline(ui: &mut Ui, id: &str, r: Rect, u: f32, color: crate::Srgba) {
    let inner = Rect {
        x: r.x + u,
        y: r.y + u,
        width: (r.width - 2.0 * u).max(0.0),
        height: (r.height - 2.0 * u).max(0.0),
    };
    ring(ui, id, r, inner, u, color);
}

/// Fills `outer` less `inner` (inside it): its four bands.
fn ring(ui: &mut Ui, id: &str, outer: Rect, inner: Rect, u: f32, color: crate::Srgba) {
    let (top, bottom) = (
        inner.y - outer.y,
        outer.y + outer.height - inner.y - inner.height,
    );
    let left = inner.x - outer.x;
    let right = outer.x + outer.width - inner.x - inner.width;
    let sides = [
        Rect {
            height: top,
            ..outer
        },
        Rect {
            y: inner.y + inner.height,
            height: bottom,
            ..outer
        },
        Rect {
            y: inner.y,
            width: left,
            height: inner.height,
            ..outer
        },
        Rect {
            x: inner.x + inner.width,
            y: inner.y,
            width: right,
            height: inner.height,
        },
    ];
    for (k, side) in sides.into_iter().enumerate() {
        if side.width > 0.0 && side.height > 0.0 {
            fill(ui, &format!("{id}.{k}"), side, u, color);
        }
    }
}
