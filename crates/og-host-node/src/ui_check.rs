//! `og ui check`'s member (packages/engine/src/node/ui-check.ts, docs/ui.md
//! §16.5), development only: `og_node_ui_frame` is an engine's last laid-out
//! UI frame read between ticks, as JSON, with what the probe's `dump_ui`
//! does not carry: the undeclared overlaps, the layout counts and draw
//! commands, every drawn line of text (its string as drawn, size, line box
//! and clip), the fills (for the primary action), the accessibility tree
//! with its focus order, the elements in visual states, the frame's
//! warnings, and the viewport and theme facts the rules read (safe area,
//! covered region, pointer, contrast mode, the style's sounds).

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_ui::{DrawCmd, Rect};
use serde_json::{Value, json};

fn rect(r: &Rect) -> Value {
    json!([r.x, r.y, r.width, r.height])
}

fn frame(engine: u32) -> Value {
    let Some(shared) = og_runtime::shared_ui(engine) else {
        return json!({ "error": "this engine has no UI" });
    };
    let ui = shared.lock();
    let f = ui.frame();
    let vp = f.viewport;
    let theme = ui.theme();
    let sounds: serde_json::Map<String, Value> = og_ui::UiSound::ALL
        .iter()
        .map(|s| {
            (
                s.name().to_string(),
                json!(theme.art.sounds[*s as usize] != 0),
            )
        })
        .collect();
    let p = &theme.palette;
    let mut texts = Vec::new();
    let mut fills = Vec::new();
    for item in &f.items {
        // A rotated or skewed command's rectangle is not where it shows.
        if !item.transform.is_identity() {
            continue;
        }
        match &item.cmd {
            DrawCmd::Text {
                text,
                size,
                bounds,
                clip,
                color,
                ..
            } => texts.push(json!({
                "text": text, "size": size, "bounds": rect(bounds), "clip": rect(clip),
                "color": color,
            })),
            DrawCmd::Quad { rect: r, color, .. } | DrawCmd::Image { rect: r, color, .. }
                if r.width >= 8.0 && r.height >= 8.0 =>
            {
                fills.push(json!({ "rect": rect(r), "color": color }));
            }
            _ => {}
        }
    }
    let elements: Vec<Value> = f.elements.iter().map(|e| e.to_json()).collect();
    let a11y = &f.a11y;
    json!({
        "viewport": {
            "width": vp.width, "height": vp.height, "dpr": vp.dpr,
            "safeArea": [vp.safe_area.left, vp.safe_area.top, vp.safe_area.right, vp.safe_area.bottom],
            "coarsePointer": vp.coarse_pointer, "highContrast": vp.high_contrast,
            "reducedMotion": vp.reduced_motion, "covered": rect(&vp.covered),
            "breakpoint": vp.breakpoint().name(),
        },
        "style": ui.style_name(),
        "contrast": if theme.contrast == og_ui::Contrast::High { "high" } else { "standard" },
        "sounds": sounds,
        "primary": [p.primary.linear(), p.primary_hover.linear(), p.primary_active.linear()],
        "elements": elements,
        "overlaps": f.overlaps.iter().map(|o| json!({
            "path": o.path, "other": o.other, "rect": rect(&o.rect), "inFlow": o.in_flow,
        })).collect::<Vec<_>>(),
        "dense": f.dense,
        "stats": {
            "elements": f.stats.elements, "rebuilt": f.stats.rebuilt,
            "laidOut": f.stats.laid_out, "keptCommands": f.stats.kept_commands,
        },
        "draws": f.items.len(),
        "texts": texts,
        "fills": fills,
        "a11y": {
            "focused": a11y.focused,
            "focusOrder": a11y.focus_order,
            "nodes": a11y.nodes.iter().map(|n| json!({
                "path": n.path, "parent": n.parent, "role": n.role, "name": n.name,
                "states": n.states, "rect": [n.x, n.y, n.width, n.height],
            })).collect::<Vec<_>>(),
        },
        "motion": f.motion.iter().map(|m| json!({
            "path": m.path,
            "states": m.states.iter().map(|(g, s)| json!([g, s])).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "warnings": f.warnings,
    })
}

/// The engine's last laid-out UI frame for `og ui check`, or `{ error }`.
#[napi(js_name = "og_node_ui_frame")]
pub fn og_node_ui_frame(engine: u32) -> String {
    frame(engine).to_string()
}
