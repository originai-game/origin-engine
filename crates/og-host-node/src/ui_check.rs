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
    // Everything in CSS px, as the draw items are and the rules measure
    // (18 px at 1080p): the frame lays out in og-ui units (docs/ui.md
    // §15.2), so its viewport, elements, overlaps and accessibility
    // rectangles are scaled by the unit here, as the probe's are.
    let u = f.unit.max(1.0);
    let ur = |r: &Rect| json!([r.x * u, r.y * u, r.width * u, r.height * u]);
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
    // What text can sit on: every filled rectangle, UI-pass shape and glass
    // of at least 8×8 (a solid under half opacity is a tint, not a ground).
    // Text over none of them is over the scene (§16.5).
    let mut grounds = Vec::new();
    for item in &f.items {
        // A rotated or skewed command's rectangle is not where it shows.
        if !item.transform.is_identity() {
            continue;
        }
        let ground = match &item.cmd {
            DrawCmd::Quad { rect: r, color, .. } | DrawCmd::Image { rect: r, color, .. }
                if color[3] >= 0.5 =>
            {
                Some(*r)
            }
            DrawCmd::Glass { rect: r, .. } => Some(*r),
            DrawCmd::Shape { shape, .. }
                if shape.mode == og_render::UiMode::Fill
                    && !matches!(shape.paint, og_render::UiPaint::Solid(c) if c[3] < 0.5) =>
            {
                Some(Rect::new(
                    shape.center.x - shape.half.x,
                    shape.center.y - shape.half.y,
                    shape.half.x * 2.0,
                    shape.half.y * 2.0,
                ))
            }
            _ => None,
        };
        if let Some(r) = ground.filter(|r| r.width >= 8.0 && r.height >= 8.0) {
            grounds.push(rect(&r));
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
    let elements: Vec<Value> = f
        .elements
        .iter()
        .map(|e| {
            let mut e = e.clone();
            (e.x, e.y, e.width, e.height) = (e.x * u, e.y * u, e.width * u, e.height * u);
            e.to_json()
        })
        .collect();
    let a11y = &f.a11y;
    json!({
        "viewport": {
            "width": vp.width * u, "height": vp.height * u, "dpr": vp.dpr / u,
            "safeArea": [vp.safe_area.left * u, vp.safe_area.top * u, vp.safe_area.right * u, vp.safe_area.bottom * u],
            "coarsePointer": vp.coarse_pointer, "highContrast": vp.high_contrast,
            "reducedMotion": vp.reduced_motion, "covered": ur(&vp.covered),
            "breakpoint": vp.breakpoint().name(),
        },
        "style": ui.style_name(),
        "contrast": if theme.contrast == og_ui::Contrast::High { "high" } else { "standard" },
        "sounds": sounds,
        "primary": [p.primary.linear(), p.primary_hover.linear(), p.primary_active.linear()],
        // The dimmed inks: text drawn in one is dimmed on purpose (§16.5).
        "dimInks": [p.ink_muted.linear(), p.ink_faint.linear()],
        "elements": elements,
        "overlaps": f.overlaps.iter().map(|o| json!({
            "path": o.path, "other": o.other, "rect": ur(&o.rect), "inFlow": o.in_flow,
        })).collect::<Vec<_>>(),
        "dense": f.dense,
        "sideways": f.sideways,
        "stats": {
            "elements": f.stats.elements, "rebuilt": f.stats.rebuilt,
            "laidOut": f.stats.laid_out, "keptCommands": f.stats.kept_commands,
        },
        "draws": f.items.len(),
        "texts": texts,
        "fills": fills,
        "grounds": grounds,
        "a11y": {
            // Off: the screen took the pad itself (og.ui.setNavigation(false)).
            "navigation": ui.navigation(),
            "focused": a11y.focused,
            "focusOrder": a11y.focus_order,
            "nodes": a11y.nodes.iter().map(|n| json!({
                "path": n.path, "parent": n.parent, "role": n.role, "name": n.name,
                "states": n.states, "rect": [n.x * u, n.y * u, n.width * u, n.height * u],
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
