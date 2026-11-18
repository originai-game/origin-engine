//! `og ui check`'s member (packages/engine/src/node/ui-check.ts, docs/ui.md
//! §16.5), development only: `og_node_ui_frame` is an engine's last laid-out
//! UI frame read between ticks, as JSON, with what the probe's `dump_ui`
//! does not carry: the overlaps, the layout counts and draw
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
    let mut fills = Vec::new();
    // What text can sit on: every filled rectangle, UI-pass shape and glass
    // of at least 8×8, each with how much it covers (its alpha). A stack of
    // translucent ones composites into a ground (a plate's gradient under
    // its scanlines); the check composites them, and text whose stack
    // covers less than half is over the scene (§16.5).
    let mut grounds = Vec::new();
    for item in &f.items {
        // A rotated or skewed command's rectangle is not where it shows; a
        // shape keeps its translation and scale (a scroll's offset), mapped here.
        if !item.transform.is_axis_aligned() {
            continue;
        }
        let ground = match &item.cmd {
            DrawCmd::Quad { rect: r, color, .. } | DrawCmd::Image { rect: r, color, .. } => {
                Some((*r, color[3]))
            }
            DrawCmd::Glass { rect: r, .. } => Some((*r, 1.0)),
            DrawCmd::Shape { shape, .. }
                if shape.mode == og_render::UiMode::Fill
                    && shape.transform == [1.0, 0.0, 0.0, 1.0] =>
            {
                let mean = |g: &og_render::UiGradient| {
                    g.stops.iter().map(|s| s.color[3]).sum::<f32>() / g.stops.len().max(1) as f32
                };
                let alpha = match &shape.paint {
                    og_render::UiPaint::Solid(c) => c[3],
                    og_render::UiPaint::Gradient(g) => mean(g),
                    og_render::UiPaint::GradientImage(g) => mean(&g.gradient),
                    og_render::UiPaint::Image(i) => i.tint[3],
                    og_render::UiPaint::Layer(_) => 1.0,
                };
                let (c, h) = (shape.center, shape.half);
                Some((
                    Rect::new(c.x - h.x, c.y - h.y, h.x * 2.0, h.y * 2.0),
                    alpha * shape.opacity,
                ))
            }
            _ => None,
        };
        if let Some((r, alpha)) = ground {
            let m = &item.transform;
            let (x0, y0) = m.apply(r.x, r.y);
            let (x1, y1) = m.apply(r.x + r.width, r.y + r.height);
            let r = Rect::new(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
            if r.width >= 8.0 && r.height >= 8.0 && alpha > 0.0 {
                grounds.push(json!([r.x, r.y, r.width, r.height, alpha]));
            }
        }
        if !item.transform.is_identity() {
            continue;
        }
        match &item.cmd {
            DrawCmd::Quad { rect: r, color, .. } | DrawCmd::Image { rect: r, color, .. }
                if r.width >= 8.0 && r.height >= 8.0 =>
            {
                fills.push(json!({ "rect": rect(r), "color": color }));
            }
            _ => {}
        }
    }
    // The lines as drawn, a styled one's included (an outlined HUD label
    // had no run and read as hidden, never measured), without their drops.
    let texts: Vec<Value> = og_ui::drawn_lines(f.items.iter().map(|i| &i.cmd))
        .into_iter()
        .filter_map(|c| match c {
            DrawCmd::Text {
                source,
                text,
                size,
                bounds,
                clip,
                color,
                ..
            } => {
                let mut line = json!({
                    "text": text, "size": size, "bounds": rect(bounds), "clip": rect(clip),
                    "color": color,
                });
                if *source != 0 {
                    let node = ui.draw_source(*source);
                    line["source"] = json!({
                        "path": node.as_ref().map(|(p, _)| p),
                        "clipped": node.is_some_and(|(_, c)| c),
                    });
                }
                Some(line)
            }
            _ => None,
        })
        .collect();
    let elements: Vec<Value> = ui
        .elements()
        .iter()
        .map(|e| {
            let mut e = e.clone();
            (e.x, e.y, e.width, e.height) = (e.x * u, e.y * u, e.width * u, e.height * u);
            e.to_json()
        })
        .collect();
    let a11y = ui.a11y();
    json!({
        "viewport": {
            "width": vp.width * u, "height": vp.height * u, "dpr": vp.dpr / u,
            "safeArea": [vp.safe_area.left * u, vp.safe_area.top * u, vp.safe_area.right * u, vp.safe_area.bottom * u],
            "coarsePointer": vp.coarse_pointer, "highContrast": vp.high_contrast,
            "reducedMotion": theme.reduced_motion, "covered": ur(&vp.covered),
            "breakpoint": vp.breakpoint().name(),
        },
        "style": ui.style_name(),
        "contrast": if theme.contrast == og_ui::Contrast::High { "high" } else { "standard" },
        "sounds": sounds,
        "primary": [p.primary.linear(), p.primary_hover.linear(), p.primary_active.linear()],
        // The dimmed inks: text drawn in one is dimmed on purpose (§16.5).
        "dimInks": [p.ink_muted.linear(), p.ink_faint.linear()],
        "elements": elements,
        "overlaps": ui.overlaps().iter().map(|o| json!({
            "path": o.path, "other": o.other, "rect": ur(&o.rect), "inFlow": o.in_flow,
        })).collect::<Vec<_>>(),
        "dense": ui.dense(),
        "sideways": ui.sideways(),
        "scrolls": ui.elements().iter().filter_map(|e| {
            let s = ui.scroll_state(&e.path)?;
            Some(json!({ "path": e.path, "rect": [e.x * u, e.y * u, e.width * u, e.height * u],
                "offsetY": s.offset_y * u, "viewHeight": s.view_height * u,
                "contentHeight": s.content_height * u }))
        }).collect::<Vec<_>>(),
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
        "motion": ui.motion_dump().iter().map(|m| json!({
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
