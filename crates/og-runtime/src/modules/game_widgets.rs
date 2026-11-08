use super::*;

pub fn og_ui_open_anchor(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.openAnchor", |ui| {
        let t = *ui.theme();
        let a = AnchorDesc {
            anchor: anchor_of(&d["anchor"]).unwrap_or_default(),
            margin: opt_f(&d, "margin"),
            offset: (f(&d, "offsetX", 0.0), f(&d, "offsetY", 0.0)),
            direction: match d["direction"].as_str() {
                Some("column") => Direction::Column,
                _ => Direction::Row,
            },
            gap: f(&d, "gap", 0.0),
            padding: insets(&d["padding"]),
            align: align(&d["align"]),
            cross: cross(&d["cross"]),
            background: paint(&t, &d["background"]),
            radius: f(&d, "radius", 0.0),
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            over: d["over"].as_bool().unwrap_or(false),
            tooltip: d["tooltip"].as_bool().unwrap_or(false),
            avoid: d["avoid"].as_bool().unwrap_or(false),
            frame: frame_part(&d["frame"]),
            instant: d["instant"].as_bool().unwrap_or(false),
        };
        ui.open_anchor(id.as_str(), a)
    });
}

pub fn og_ui_open_modal(id: String, open: bool, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.openModal", |ui| {
        ui.open_modal(
            id.as_str(),
            open,
            &ModalDesc {
                title: d["title"].as_str().unwrap_or_default().to_string(),
                width: size(&d["width"]),
                locked: d["locked"].as_bool().unwrap_or(false),
                ignore_backdrop: d["ignoreBackdrop"].as_bool().unwrap_or(false),
                transition: transition_kind(&d["transition"]),
                backdrop: paint(ui.theme(), &d["backdrop"]),
                bare: d["bare"].as_bool().unwrap_or(false),
            },
        )
    })
    .unwrap_or_default();
    json!({ "visible": r.visible, "dismissed": r.dismissed }).to_string()
}

pub fn og_ui_close_modal() {
    with_ui("og.ui.closeModal", |ui| ui.close_modal());
}

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

pub fn og_ui_badge(id: String, text: String, t: String) {
    let tn = tone(&Value::String(t));
    with_ui("og.ui.badge", |ui| ui.badge(id.as_str(), &text, tn));
}

pub fn og_ui_divider(id: String, vertical: bool) {
    with_ui("og.ui.divider", |ui| ui.divider(id.as_str(), vertical));
}

pub fn og_ui_spacer(id: String, s: String) {
    let v = parse(Some(&s)).unwrap_or(Value::Null);
    with_ui("og.ui.spacer", |ui| ui.spacer(id.as_str(), size(&v)));
}

pub fn og_ui_spinner(id: String, s: f64) {
    with_ui("og.ui.spinner", |ui| ui.spinner(id.as_str(), s as f32));
}

pub fn og_ui_icon(id: String, name: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.icon", |ui| {
        let t = *ui.theme();
        ui.icon(
            id.as_str(),
            &name,
            IconDesc {
                size: f(&d, "size", 0.0),
                tint: paint(&t, &d["tint"]),
                secondary: d["secondary"].as_str().map(str::to_owned),
                secondary_tint: paint(&t, &d["secondaryTint"]),
                label: d["label"].as_str().map(str::to_owned),
            },
        )
    });
}

/// The UI of an engine, for calls that work between ticks too.
pub(super) fn ui_of(world: u32) -> Option<SharedUi> {
    engine(world).ok()?.state.borrow().ui.clone()
}

pub fn og_ui_toast(world: u32, text: String, desc: Option<String>) -> u32 {
    let d = desc_of(desc);
    let Some(ui) = ui_of(world) else { return 0 };
    let mut ui = ui.lock();
    ui.toast(
        &text,
        ToastDesc {
            tone: tone(&d["tone"]),
            duration_ms: f(&d, "durationMs", 0.0),
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
    let d: Value = serde_json::from_str(&desc).unwrap_or(Value::Null);
    let desc = d.is_object().then(|| og_ui::VirtualCursorDesc {
        speed: f(&d, "speed", 0.0),
        accel: f(&d, "accel", 0.0),
        magnet: f(&d, "magnet", 0.0),
        scroll: f(&d, "scroll", 0.0),
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

pub fn og_ui_memory(id: String, initial: f64) -> f64 {
    with_ui("og.ui.memory", |ui| ui.memory(&id, initial)).unwrap_or(initial)
}

pub fn og_ui_remember(id: String, value: f64) {
    with_ui("og.ui.remember", |ui| ui.remember(&id, value));
}

pub fn og_ui_tick() -> u32 {
    with_ui("og.ui.tick", |ui| ui.tick() as u32).unwrap_or(0)
}

pub fn og_ui_preview_viewport(world: u32, viewport: String) {
    let v = parse(Some(&viewport)).unwrap_or(Value::Null);
    let set = PREVIEW.with(|p| {
        let mut p = p.borrow_mut();
        if v.is_object() {
            let num = |k: &str, d: f64| v[k].as_f64().unwrap_or(d) as f32;
            p.insert(
                world,
                Viewport {
                    width: num("width", 1280.0).max(1.0),
                    height: num("height", 720.0).max(1.0),
                    dpr: num("dpr", 1.0),
                    safe_area: insets(&v["safeArea"]),
                    small: [num("smallWidth", 0.0), num("smallHeight", 0.0)],
                    large: [num("largeWidth", 0.0), num("largeHeight", 0.0)],
                    coarse_pointer: v["coarsePointer"].as_bool().unwrap_or(false),
                    reduced_motion: v["reducedMotion"].as_bool().unwrap_or(false),
                    high_contrast: v["highContrast"].as_bool().unwrap_or(false),
                    reduced_transparency: v["reducedTransparency"].as_bool().unwrap_or(false),
                    mac_keys: v["macKeys"].as_bool().unwrap_or(false),
                    covered: ui_rect(&v["covered"]),
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
