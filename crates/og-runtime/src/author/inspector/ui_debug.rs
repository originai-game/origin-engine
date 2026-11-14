//! The inspector's UI view (docs/ui.md §6.4): the visual debugger of the
//! game's UI in a development build. Pick picks the element under the
//! pointer (`Ui::debug_pick`); the panel shows its box model, the layout
//! boundaries it sets, its computed styles with the line each was written
//! on, the style rules that dress it and its bindings
//! (`Ui::debug_element`), and the frame draws its margin, border, padding
//! and content boxes over the game. Layout bounds outlines every element
//! drawn. It reads the game's UI only while this view is open, and only
//! the element picked (or under the pointer while picking).

use super::Inspector;
use og_ui::{Insets, Rect, Ui, UiDebugElement};
use serde_json::{Value, json};

/// What the view read of the game's UI this frame, in CSS px.
#[derive(Clone, Debug, Default)]
pub(super) struct View {
    pub element: Option<UiDebugElement>,
    pub hover: Option<UiDebugElement>,
    /// Every element drawn, while Layout bounds is on.
    pub bounds: Vec<Rect>,
}

/// The tab the view is.
pub(super) const TAB: usize = 2;

/// Reads the game's UI for this frame: the element a pick asked for, the
/// one under the pointer while picking, and the bounds when asked.
pub(super) fn read(ins: &mut Inspector, game: &Ui) -> View {
    let u = game.frame().unit.max(1.0);
    let at = |x: f32, y: f32| game.debug_pick(x / u, y / u);
    if let Some((x, y)) = ins.ui_pick.take() {
        ins.ui_selected = at(x, y);
        ins.status = match &ins.ui_selected {
            Some(p) => format!("Picked {p}"),
            None => "No UI element under the pointer".into(),
        };
    }
    let scaled = |e: UiDebugElement| scale(e, u);
    let element = ins
        .ui_selected
        .as_deref()
        .and_then(|p| game.debug_element(p))
        .map(scaled);
    let hover = if ins.picking {
        let p = ins.input.pointer;
        at(p.x, p.y)
            .and_then(|p| game.debug_element(&p))
            .map(scaled)
    } else {
        None
    };
    let bounds = if ins.ui_bounds {
        game.elements()
            .iter()
            .filter(|e| !e.clipped)
            .map(|e| Rect {
                x: e.x * u,
                y: e.y * u,
                width: e.width * u,
                height: e.height * u,
            })
            .collect()
    } else {
        Vec::new()
    };
    View {
        element,
        hover,
        bounds,
    }
}

fn scale(mut e: UiDebugElement, u: f32) -> UiDebugElement {
    let r = |r: Rect| Rect {
        x: r.x * u,
        y: r.y * u,
        width: r.width * u,
        height: r.height * u,
    };
    let i = |i: Insets| Insets {
        left: i.left * u,
        top: i.top * u,
        right: i.right * u,
        bottom: i.bottom * u,
    };
    e.border_box = r(e.border_box);
    e.content = r(e.content);
    e.margin = i(e.margin);
    e.border = i(e.border);
    e.padding = i(e.padding);
    e
}

/// The panel's lines about the picked element (none when none is):
/// `filter` (lower case) keeps the styles, rules and bindings whose name
/// holds it.
pub(super) fn lines(view: &View, filter: &str) -> Option<Vec<Value>> {
    let e = view.element.as_ref()?;
    let kept = |name: &str| filter.is_empty() || name.to_lowercase().contains(filter);
    let mut out: Vec<(&str, String)> =
        vec![("line", e.path.clone()), ("muted", format!("<{}>", e.kind))];
    out.push(("head", "Box model".into()));
    for (name, i) in [
        ("margin", e.margin),
        ("border", e.border),
        ("padding", e.padding),
    ] {
        out.push(("line", format!("{name}  {}", insets(i))));
    }
    let b = &e.border_box;
    out.push((
        "line",
        format!(
            "content  {} × {}  (box {} × {} at {}, {})",
            px(e.content.width),
            px(e.content.height),
            px(b.width),
            px(b.height),
            px(b.x),
            px(b.y)
        ),
    ));
    if !e.boundaries.is_empty() {
        out.push((
            "line",
            format!("layout boundary: {}", e.boundaries.join(", ")),
        ));
    }
    out.push(("head", "Computed styles".into()));
    if e.styles.is_empty() {
        out.push(("muted", "No document declarations".into()));
    }
    for s in e.styles.iter().filter(|s| kept(&s.name)) {
        out.push(("line", format!("{}: {}   {}", s.name, s.value, s.source)));
    }
    out.push(("head", "Style rules".into()));
    if e.rules.is_empty() {
        out.push(("muted", "No style rule dresses it".into()));
    }
    for s in e.rules.iter().filter(|s| kept(&s.name)) {
        out.push(("line", format!("{}   {}", s.name, s.source)));
    }
    out.push(("head", "Bindings".into()));
    if e.bindings.is_empty() {
        out.push(("muted", "No bindings".into()));
    }
    for (prop, expr, value) in e.bindings.iter().filter(|b| kept(&b.0)) {
        out.push(("line", format!("{prop} = {expr} → {value}")));
    }
    Some(
        out.into_iter()
            .enumerate()
            .map(|(k, (kind, text))| json!({ "id": k.to_string(), "kind": kind, "text": text }))
            .collect(),
    )
}

/// The boxes drawn over the game, in the panel's CSS px (`unit` is its
/// UI's): every element's bounds while Layout bounds is on, then the
/// element under the pointer and the one picked, each as its margin,
/// border and padding rings around its content, as Chrome draws them.
pub(super) fn boxes(view: &View, unit: f32) -> Vec<Value> {
    let u = unit.max(1.0);
    let mut out = Vec::new();
    let mut push = |kind: &str, r: Rect, ring: Option<Insets>| {
        let ring = ring.unwrap_or_default();
        out.push(json!({
            "id": out.len().to_string(),
            "kind": kind,
            "x": r.x / u, "y": r.y / u, "w": r.width / u, "h": r.height / u,
            "t": ring.top / u, "r": ring.right / u, "b": ring.bottom / u, "l": ring.left / u,
        }));
    };
    for r in &view.bounds {
        push("bound", *r, Some(Insets::all(u)));
    }
    for e in [&view.hover, &view.element].into_iter().flatten() {
        let b = e.border_box;
        let margin = Rect {
            x: b.x - e.margin.left,
            y: b.y - e.margin.top,
            width: b.width + e.margin.left + e.margin.right,
            height: b.height + e.margin.top + e.margin.bottom,
        };
        let padding = Rect {
            x: b.x + e.border.left,
            y: b.y + e.border.top,
            width: (b.width - e.border.left - e.border.right).max(0.0),
            height: (b.height - e.border.top - e.border.bottom).max(0.0),
        };
        let between = |outer: Rect, inner: Rect| Insets {
            left: inner.x - outer.x,
            top: inner.y - outer.y,
            right: outer.x + outer.width - inner.x - inner.width,
            bottom: outer.y + outer.height - inner.y - inner.height,
        };
        push("margin", margin, Some(e.margin));
        push("border", b, Some(e.border));
        push("padding", padding, Some(between(padding, e.content)));
        push("content", e.content, None);
    }
    out
}

fn px(v: f32) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r:.1}")
    }
}

/// CSS order: top right bottom left.
fn insets(i: Insets) -> String {
    format!(
        "{} {} {} {}",
        px(i.top),
        px(i.right),
        px(i.bottom),
        px(i.left)
    )
}
