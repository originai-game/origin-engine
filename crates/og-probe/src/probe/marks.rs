//! `marks`: the frame of the current tick with a numbered box over each UI
//! element and scene entity it shows (Set-of-Mark), and the map from each
//! number to its path, kind, box, a point to pick it at, and the document or
//! scene line that made it (IDL `probe.MarkedCapture`).

use super::Probe;
use crate::base64;
use crate::frame::{PixelRect, decode_png, draw_mark};
use crate::hooks::UiElement;
use og_ecs::World;
use serde_json::{Value, json};

/// What `marks` was asked (IDL `EngineProbe.marks`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarksDesc {
    /// The render size; 0×0 is the render target's.
    pub width: u32,
    pub height: u32,
    /// The longest side of the PNG; 0 keeps the rendered size.
    pub fit: u32,
    pub ui: bool,
    pub entities: bool,
    /// The most marks drawn.
    pub max: usize,
}

/// A UI element's box colour (amber) and an entity's (cyan): two hues a
/// reader tells apart at a glance and that stand off most frames.
const UI_COLOUR: [u8; 3] = [255, 196, 0];
const ENTITY_COLOUR: [u8; 3] = [0, 214, 255];

/// One mark before it is numbered.
struct Candidate {
    /// Its box in CSS px.
    css: [f32; 4],
    value: Value,
}

impl Probe {
    /// `marks` (see the module doc); `None` without a renderer.
    pub fn marks(&mut self, world: &mut World, desc: MarksDesc) -> Option<Value> {
        let described = desc.entities.then(|| self.describe(world));
        let frame = self
            .host
            .capture
            .as_mut()?
            .capture(world, desc.width, desc.height)?;
        let image = decode_png(&frame.png).ok()?;
        let css_scale = self.surface_scale(world, image.width);
        let mut image = image.fit(desc.fit);
        let scale = css_scale * image.width as f32 / frame.width.max(1) as f32;

        let mut candidates = Vec::new();
        if desc.ui
            && let Some(ui) = self.host.ui.as_ref()
        {
            let surface = ui.surface(world);
            candidates.extend(ui_marks(&ui.elements(world), surface));
        }
        if let Some(d) = &described {
            for e in d["entities"].as_array().into_iter().flatten() {
                let b = &e["box"];
                let css = [
                    b["x"].as_f64().unwrap_or(0.0) as f32,
                    b["y"].as_f64().unwrap_or(0.0) as f32,
                    b["width"].as_f64().unwrap_or(0.0) as f32,
                    b["height"].as_f64().unwrap_or(0.0) as f32,
                ];
                let id = e["id"].as_str().unwrap_or_default().to_owned();
                let source = self.names.resolve(world, &id).and_then(|entity| {
                    self.host
                        .scene_writer
                        .as_ref()
                        .and_then(|w| w.source(world, entity))
                });
                let what: Vec<&str> = e["labels"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                candidates.push(Candidate {
                    css,
                    value: json!({
                        "kind": "entity",
                        "id": id,
                        "path": id,
                        "what": what.join(", "),
                        "text": e["name"].as_str().unwrap_or_default(),
                        "point": e["point"],
                        "source": source,
                        "part": Value::Null,
                        "rule": Value::Null,
                        "depth": e["depth"],
                    }),
                });
            }
        }

        let total = candidates.len();
        let mut marks = Vec::new();
        // Entities are drawn first and UI over them, as the frame layers
        // them; the numbers keep the listed order.
        let mut drawn: Vec<(PixelRect, u32, [u8; 3])> = Vec::new();
        for c in candidates {
            if marks.len() >= desc.max {
                break;
            }
            let [x, y, w, h] = c.css;
            let Some(rect) = PixelRect::clipped(x, y, w, h, scale, image.width, image.height)
            else {
                continue;
            };
            let n = marks.len() as u32 + 1;
            let mut v = c.value;
            let colour = if v["kind"] == "ui" {
                UI_COLOUR
            } else {
                ENTITY_COLOUR
            };
            v["n"] = json!(n);
            v["rect"] =
                json!({ "x": rect.x, "y": rect.y, "width": rect.width, "height": rect.height });
            drawn.push((rect, n, colour));
            marks.push(v);
        }
        let omitted = total - marks.len().min(total);
        for (rect, n, colour) in drawn
            .iter()
            .filter(|d| d.2 == ENTITY_COLOUR)
            .chain(drawn.iter().filter(|d| d.2 == UI_COLOUR))
        {
            draw_mark(&mut image, *rect, *n, *colour);
        }
        Some(json!({
            "tick": world.time().tick,
            "width": image.width,
            "height": image.height,
            "scale": scale,
            "png": base64::encode(&image.png()),
            "marks": marks,
            "omitted": omitted,
        }))
    }
}

/// The UI elements a mark goes on, in reading order: those `pickUi` answers
/// (they show something or take input), inside the surface; an element and
/// an ancestor of the same box (within a CSS pixel) once, as the one that
/// takes input or else the topmost; no backdrop that covers the whole
/// surface and takes no input.
fn ui_marks(elements: &[UiElement], surface: Option<(f32, f32, f32)>) -> Vec<Candidate> {
    const LAYOUT: [&str; 6] = ["layer", "root", "row", "column", "scroll", "stack"];
    let interactive = |e: &UiElement| e.states.iter().any(|s| s == "focusable");
    let shows = |e: &UiElement| {
        !LAYOUT.contains(&e.kind.as_str())
            || !e.text.is_empty()
            || !e.role.is_empty()
            || !e.states.is_empty()
            || e.opaque
            || e.detail.contains_key("part")
    };
    let (sw, sh) = surface.map_or((f32::INFINITY, f32::INFINITY), |(w, h, _)| (w, h));
    let same = |a: &UiElement, b: &UiElement| {
        (a.x - b.x).abs() <= 1.0
            && (a.y - b.y).abs() <= 1.0
            && (a.width - b.width).abs() <= 1.0
            && (a.height - b.height).abs() <= 1.0
    };
    let mut kept: Vec<&UiElement> = Vec::new();
    for e in elements {
        if !shows(e) || e.width < 1.0 || e.height < 1.0 {
            continue;
        }
        if e.x >= sw || e.y >= sh || e.x + e.width <= 0.0 || e.y + e.height <= 0.0 {
            continue;
        }
        let backdrop = e.x <= 0.5 && e.y <= 0.5 && e.width >= sw - 1.0 && e.height >= sh - 1.0;
        if backdrop && !interactive(e) {
            continue;
        }
        // Drawn order: an ancestor comes before its descendants.
        let related = |k: &&UiElement| {
            same(k, e) && (e.path.starts_with(&format!("{}/", k.path)) || e.parent == k.id)
        };
        if let Some(i) = kept.iter().position(related) {
            if interactive(kept[i]) && !interactive(e) {
                continue;
            }
            kept.remove(i);
        }
        kept.push(e);
    }
    // Reading order: rows of 8 CSS px, then left to right.
    let row = |e: &UiElement| (e.y / 8.0).floor() as i64;
    kept.sort_by(|a, b| {
        row(a)
            .cmp(&row(b))
            .then(a.x.total_cmp(&b.x))
            .then(a.path.cmp(&b.path))
    });
    kept.into_iter()
        .map(|e| {
            let field = |k: &str| e.detail.get(k).cloned().unwrap_or(Value::Null);
            let rule = e
                .detail
                .get("rules")
                .and_then(Value::as_array)
                .and_then(|r| r.last())
                .cloned()
                .unwrap_or(Value::Null);
            Candidate {
                css: [e.x, e.y, e.width, e.height],
                value: json!({
                    "kind": "ui",
                    "id": e.id,
                    "path": if e.path.is_empty() { &e.id } else { &e.path },
                    "what": e.kind,
                    "text": e.text,
                    "point": { "x": e.x + e.width / 2.0, "y": e.y + e.height / 2.0 },
                    "source": field("source"),
                    "part": field("part"),
                    "rule": rule,
                    "depth": Value::Null,
                }),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(id: &str, path: &str, kind: &str, r: [f32; 4]) -> UiElement {
        UiElement {
            id: id.into(),
            path: path.into(),
            kind: kind.into(),
            x: r[0],
            y: r[1],
            width: r[2],
            height: r[3],
            ..UiElement::default()
        }
    }

    #[test]
    fn one_mark_per_box_reading_order_and_no_backdrop() {
        let mut button = el("buy", "Shop/buy", "button", [200.0, 100.0, 80.0, 30.0]);
        button.states = vec!["focusable".into()];
        let mut label = el("t", "Shop/buy/t", "label", [200.0, 100.0, 80.0, 30.0]);
        label.text = "Buy".into();
        let mut panel = el("Shop", "Shop", "panel", [0.0, 0.0, 640.0, 360.0]);
        panel.opaque = true;
        let mut title = el("title", "Shop/title", "label", [20.0, 103.0, 100.0, 20.0]);
        title.text = "Shop".into();
        let mut row = el("row", "Shop/row", "row", [10.0, 10.0, 600.0, 40.0]);
        row.kind = "row".into();
        let mut off = el("off", "Shop/off", "label", [700.0, 10.0, 50.0, 20.0]);
        off.text = "gone".into();
        let marks = ui_marks(
            &[panel, row, button, label, title, off],
            Some((640.0, 360.0, 1.0)),
        );
        let paths: Vec<&str> = marks
            .iter()
            .map(|c| c.value["path"].as_str().unwrap())
            .collect();
        // The full-surface panel and the bare row get no mark, the label of
        // the button's own box merges into the button, the element off the
        // surface is left out; the title (y 103) reads before the button
        // (y 100) in one row only by x.
        assert_eq!(paths, ["Shop/title", "Shop/buy"]);
        assert_eq!(marks[1].value["point"], json!({ "x": 240.0, "y": 115.0 }));
    }
}
