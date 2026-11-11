use super::*;
use crate::wire;

pub fn og_ui_bounds(path: String) -> String {
    let b = with_ui("og.ui.bounds", |ui| {
        ui.element(&path)
            .map(|e| og_ui::Rect::new(e.x, e.y, e.width, e.height))
    })
    .flatten();
    b.map_or(Value::Null, |r| rect_json(&r)).to_string()
}

pub fn og_ui_scroll_state(path: String) -> String {
    let s = with_ui("og.ui.scrollState", |ui| ui.scroll_state(&path)).flatten();
    s.map_or(Value::Null, |s| {
        json!({
            "offsetX": s.offset_x, "offsetY": s.offset_y,
            "viewWidth": s.view_width, "viewHeight": s.view_height,
            "contentWidth": s.content_width, "contentHeight": s.content_height,
        })
    })
    .to_string()
}

/// The UI of an engine, for calls that work between ticks too.
pub(super) fn ui_of(world: u32) -> Option<SharedUi> {
    engine(world).ok()?.state.borrow().ui.clone()
}

pub fn og_ui_toast(world: u32, text: String, desc: Option<String>) -> u32 {
    let d: wire::ToastDesc = wire::arg(desc.as_deref(), "desc").unwrap_or_default();
    let Some(ui) = ui_of(world) else { return 0 };
    let mut ui = ui.lock();
    ui.toast(
        &text,
        ToastDesc {
            tone: Tone::from_index(d.tone as u8),
            duration_ms: d.duration_ms,
        },
    )
}

pub fn og_ui_dismiss_toast(world: u32, toast_id: u32) {
    if let Some(ui) = ui_of(world) {
        ui.lock().dismiss_toast(toast_id);
    }
}

pub fn og_ui_navigate(world: u32, intent: String) {
    let nav = match intent.as_str() {
        "up" => Nav::Up,
        "down" => Nav::Down,
        "left" => Nav::Left,
        "right" => Nav::Right,
        "next" => Nav::Next,
        "prev" => Nav::Prev,
        "activate" => Nav::Activate,
        "back" => Nav::Back,
        _ => return,
    };
    if let Some(ui) = ui_of(world) {
        ui.lock().navigate(nav);
    }
}

pub fn og_ui_back() -> bool {
    with_ui("og.ui.back", |ui| ui.back()).unwrap_or(false)
}

pub fn og_ui_feedback(event: String, rise: f64) {
    let Some(e) = og_ui::tokens::SoundEvent::from_name(&event) else {
        return;
    };
    with_ui("og.ui.feedback", |ui| {
        if rise >= 0.0 {
            ui.feedback_rising(e, rise as f32)
        } else {
            ui.feedback(e)
        }
    });
}

pub fn og_ui_set_haptics(intensity: f64) {
    with_ui("og.ui.setHaptics", |ui| ui.set_haptics(intensity as f32));
}

pub fn og_ui_set_navigation(on: bool) {
    with_ui("og.ui.setNavigation", |ui| ui.set_navigation(on));
}

pub fn og_ui_set_players(count: u32) {
    with_ui("og.ui.setPlayers", |ui| ui.set_players(count));
}

pub fn og_ui_set_virtual_cursor(desc: String) {
    let d = wire::arg::<Option<wire::VirtualCursorDesc>>(Some(&desc), "desc").unwrap_or_default();
    let desc = d.map(|d| og_ui::VirtualCursorDesc {
        speed: d.speed,
        accel: d.accel,
        magnet: d.magnet,
        scroll: d.scroll,
    });
    with_ui("og.ui.setVirtualCursor", |ui| ui.set_virtual_cursor(desc));
}

pub fn og_ui_virtual_cursor() -> String {
    match with_ui("og.ui.virtualCursor", |ui| ui.virtual_cursor()).flatten() {
        Some((p, shown)) => json!({ "x": p.x, "y": p.y, "shown": shown }).to_string(),
        None => "null".into(),
    }
}

pub fn og_ui_set_hud_edit(on: bool) {
    with_ui("og.ui.setHudEdit", |ui| ui.set_hud_edit(on));
}

pub fn og_ui_set_hud_player(player: u32) {
    with_ui("og.ui.setHudPlayer", |ui| ui.set_hud_player(player));
}

pub fn og_ui_hud_edit() -> String {
    let e = with_ui("og.ui.hudEdit", |ui| ui.hud_edit()).unwrap_or_default();
    let p = e.placement;
    json!({
        "on": e.on, "player": e.player, "selected": e.selected,
        "placement": { "x": p.x, "y": p.y, "scale": p.scale, "opacity": p.opacity },
        "keys": e.keys, "changed": e.changed,
    })
    .to_string()
}

pub fn og_ui_hud_layout(player: u32) -> String {
    with_ui("og.ui.hudLayout", |ui| ui.hud_layout(player)).unwrap_or_else(|| "{}".into())
}

pub fn og_ui_set_hud_layout(player: u32, layout: String) -> bool {
    with_ui("og.ui.setHudLayout", |ui| {
        match ui.set_hud_layout(player, &layout) {
            Ok(()) => true,
            Err(e) => {
                ui.warn(e);
                false
            }
        }
    })
    .unwrap_or(false)
}

pub fn og_ui_hud_select(key: String) {
    with_ui("og.ui.hudSelect", |ui| ui.hud_select(&key));
}

pub fn og_ui_hud_step(scale: i32, opacity: i32) {
    with_ui("og.ui.hudStep", |ui| ui.hud_step(scale, opacity));
}

pub fn og_ui_hud_reset(key: String) {
    with_ui("og.ui.hudReset", |ui| ui.hud_reset(&key));
}

pub fn og_ui_focus(pad: u32) -> String {
    with_ui("og.ui.focus", |ui| ui.focus_path(pad).to_string()).unwrap_or_default()
}

/// An accessibility tree as og.ui.a11y answers it (IDL `ui.A11yTree`).
/// The accessibility tree as JSON, its rectangles times `unit` (1: the
/// game's units, as og.ui.a11y gives them; the frame's og-ui unit: CSS px,
/// as the page's mirror places its nodes).
pub(crate) fn a11y_json(t: &og_ui::A11yTree, unit: f32) -> Value {
    let nodes: Vec<Value> = t
        .nodes
        .iter()
        .map(|n| {
            json!({
                "path": n.path, "parent": n.parent, "role": n.role, "name": n.name,
                "description": n.description, "value": n.value, "states": n.states,
                "valueMin": n.value_min, "valueMax": n.value_max, "valueNow": n.value_now,
                "level": n.level, "live": n.live, "atomic": n.atomic, "notify": n.notify, "x": n.x * unit, "y": n.y * unit, "width": n.width * unit, "height": n.height * unit,
            })
        })
        .collect();
    json!({"nodes": nodes, "focusOrder": t.focus_order, "focused": t.focused})
}

pub fn og_ui_a11y() -> String {
    with_ui("og.ui.a11y", |ui| a11y_json(ui.a11y(), 1.0).to_string())
        .unwrap_or_else(|| r#"{"nodes":[],"focusOrder":[],"focused":""}"#.to_string())
}

pub fn og_ui_tick() -> u32 {
    with_ui("og.ui.tick", |ui| ui.tick() as u32).unwrap_or(0)
}

pub fn og_ui_preview_viewport(world: u32, viewport: String) {
    let raw = parse(Some(&viewport)).unwrap_or(Value::Null);
    let v = wire::value::<Option<wire::Viewport>>(&raw, "viewport").unwrap_or_default();
    // An absent width or height is a 1280 × 720 surface's.
    let or = |key: &str, x: f32, d: f32| if raw[key].is_null() { d } else { x };
    let set = PREVIEW.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(v) = v {
            let (a, c) = (v.safe_area, v.covered.unwrap_or_default());
            let n = |x: Option<f32>| x.unwrap_or(0.0);
            p.insert(
                world,
                Viewport {
                    width: or("width", v.width, 1280.0).max(1.0),
                    height: or("height", v.height, 720.0).max(1.0),
                    dpr: v.dpr,
                    safe_area: Insets {
                        left: a.left,
                        top: a.top,
                        right: a.right,
                        bottom: a.bottom,
                    },
                    small: [n(v.small_width), n(v.small_height)],
                    large: [n(v.large_width), n(v.large_height)],
                    coarse_pointer: v.coarse_pointer,
                    reduced_motion: v.reduced_motion.unwrap_or(false),
                    high_contrast: v.high_contrast.unwrap_or(false),
                    reduced_transparency: v.reduced_transparency.unwrap_or(false),
                    mac_keys: v.mac_keys.unwrap_or(false),
                    covered: og_ui::Rect::new(c.x, c.y, c.width, c.height),
                },
            );
        } else {
            p.remove(&world);
        }
        p.get(&world).copied()
    });
    // The screen overlay (screen sprites and texts, the UI) is laid on the
    // previewed CSS surface too: a capture at 2 device pixels per CSS pixel
    // drew the UI at 1 in the corner of its frame when a screen sprite (a
    // HUD page's scene) had made the overlay first, at the target's size.
    if let Ok(eng) = engine(world)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.preview = set;
    }
}
