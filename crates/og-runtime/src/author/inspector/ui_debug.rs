//! The inspector's UI view (docs/ui.md §6.4): the visual debugger of the
//! game's UI in a development build. Pick picks the element under the
//! pointer, the topmost the TSX tree laid out there; the panel shows its
//! path, kind and box, and the frame draws its box over the game. Layout
//! bounds outlines every element drawn. It reads the game's UI only while
//! this view is open.

use super::Inspector;
use og_probe::UiElement;
use serde_json::{Value, json};

/// What the view read of the game's UI this frame, in CSS px.
#[derive(Clone, Debug, Default)]
pub(super) struct View {
    pub element: Option<UiElement>,
    pub hover: Option<UiElement>,
    /// Every element drawn, while Layout bounds is on.
    pub bounds: Vec<[f32; 4]>,
}

/// The tab the view is.
pub(super) const TAB: usize = 2;

/// Reads the game's UI elements (the tree's, in draw order) for this
/// frame: the element a pick asked for, the one under the pointer while
/// picking, and the bounds when asked.
pub(super) fn read(ins: &mut Inspector, elements: &[UiElement]) -> View {
    // The topmost element drawn at the point: a later one is on top.
    let at = |x: f32, y: f32| {
        elements
            .iter()
            .rev()
            .find(|e| !e.clipped && x >= e.x && y >= e.y && x < e.x + e.width && y < e.y + e.height)
            .map(|e| e.path.clone())
    };
    if let Some((x, y)) = ins.ui_pick.take() {
        ins.ui_selected = at(x, y);
        ins.status = match &ins.ui_selected {
            Some(p) => format!("Picked {p}"),
            None => "No UI element under the pointer".into(),
        };
    }
    let find = |path: &str| elements.iter().find(|e| e.path == path).cloned();
    let element = ins.ui_selected.as_deref().and_then(find);
    let hover = if ins.picking {
        let p = ins.pointer;
        at(p.x, p.y).and_then(|p| find(&p))
    } else {
        None
    };
    let bounds = if ins.ui_bounds {
        elements
            .iter()
            .filter(|e| !e.clipped)
            .map(|e| [e.x, e.y, e.width, e.height])
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

/// The panel's lines about the picked element (none when none is).
pub(super) fn lines(view: &View) -> Option<Vec<Value>> {
    let e = view.element.as_ref()?;
    let out = [
        ("line", e.path.clone()),
        ("muted", format!("<{}>", e.kind)),
        ("head", "Box".into()),
        (
            "line",
            format!(
                "{} × {} at {}, {}",
                px(e.width),
                px(e.height),
                px(e.x),
                px(e.y)
            ),
        ),
    ];
    Some(
        out.into_iter()
            .enumerate()
            .map(|(k, (kind, text))| json!({ "id": k.to_string(), "kind": kind, "text": text }))
            .collect(),
    )
}

/// The boxes drawn over the game, in CSS px: every element's bounds while
/// Layout bounds is on, then the element under the pointer and the one
/// picked.
pub(super) fn boxes(view: &View) -> Vec<Value> {
    let mut out = Vec::new();
    let mut push = |kind: &str, [x, y, w, h]: [f32; 4], ring: f32| {
        out.push(json!({
            "id": out.len().to_string(),
            "kind": kind,
            "x": x, "y": y, "w": w, "h": h,
            "t": ring, "r": ring, "b": ring, "l": ring,
        }));
    };
    for r in &view.bounds {
        push("bound", *r, 1.0);
    }
    for e in [&view.hover, &view.element].into_iter().flatten() {
        push("content", [e.x, e.y, e.width, e.height], 0.0);
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
