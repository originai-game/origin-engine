//! `og ui check`'s member (packages/engine/src/node/ui-check.ts, docs/ui.md
//! §16.5), development only: `og_node_ui_frame` is an engine's last laid-out
//! UI read between ticks, as JSON: the TSX tree's elements (CSS px), its
//! accessibility nodes and focus order, and the viewport and theme facts the
//! rules read (safe area, covered region, pointer, contrast mode, the pack's
//! sounds). The drawn text lines, fills, grounds and overlaps come with the
//! paint list's reading; until then they are empty and the rules that need
//! them are "not checkable yet" (ui-check.ts RULES).

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use serde_json::{Value, json};

fn frame(engine: u32) -> Value {
    let Some((vp, sounds)) = og_runtime::ui_surface(engine) else {
        return json!({ "error": "this engine has no UI" });
    };
    let theme = og_runtime::og_ui_theme(engine);
    let sounds: serde_json::Map<String, Value> =
        ["hover", "press", "open", "close", "success", "error"]
            .iter()
            .map(|e| (e.to_string(), json!(sounds.iter().any(|s| s == e))))
            .collect();
    let (elements, focus_order) = og_runtime::ui_next_dump(engine);
    let c = vp.covered;
    let s = vp.safe_area;
    json!({
        "viewport": {
            "width": vp.width, "height": vp.height, "dpr": if vp.dpr > 0.0 { vp.dpr } else { 1.0 },
            "safeArea": [s.left, s.top, s.right, s.bottom],
            "coarsePointer": vp.coarse_pointer, "highContrast": vp.high_contrast,
            "reducedMotion": theme.reduced_motion, "covered": [c.x, c.y, c.width, c.height],
            "breakpoint": vp.breakpoint().name(),
        },
        "style": theme.style,
        "contrast": theme.contrast.as_str(),
        "sounds": sounds,
        "primary": [],
        "elements": elements.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
        "overlaps": [],
        "stats": { "elements": elements.len(), "rebuilt": 0, "laidOut": 0, "keptCommands": 0 },
        "draws": 0,
        "texts": [],
        "fills": [],
        "a11y": {
            "focused": elements.iter().find(|e| e.states.iter().any(|s| s == "focused")).map_or("", |e| e.path.as_str()),
            "focusOrder": focus_order,
            "nodes": elements.iter().filter(|e| !e.role.is_empty()).map(|e| json!({
                "path": e.path, "role": e.role, "states": e.states,
                "rect": [e.x, e.y, e.width, e.height],
            })).collect::<Vec<_>>(),
        },
        "motion": [],
        "warnings": [],
    })
}

/// The engine's last laid-out UI frame for `og ui check`, or `{ error }`.
#[napi(js_name = "og_node_ui_frame")]
pub fn og_node_ui_frame(engine: u32) -> String {
    frame(engine).to_string()
}
