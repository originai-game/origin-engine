use super::*;
use crate::host::wire;

/// Runs `f` on the TSX UI of the engine whose frame is open, made when it
/// has none yet; outside a frame, warns as any og.ui frame call does.
fn on_next(call: &str, f: impl FnOnce(&mut crate::present::ui_next::Next)) {
    let Some(eng) = super::ui_desc::building(call) else {
        return;
    };
    let mut rs = eng.render.borrow_mut();
    f(rs.ui_next
        .get_or_insert_with(|| Box::new(crate::present::ui_next::Next::new())));
}

pub fn og_ui_feedback(event: String, rise: f64) {
    on_next("og.ui.feedback", |n| {
        n.feedback(&event, (rise >= 0.0).then_some(rise as f32))
    });
}

pub fn og_ui_set_haptics(intensity: f64) {
    on_next("og.ui.setHaptics", |n| {
        n.haptic_strength = if intensity.is_finite() {
            intensity.clamp(0.0, 1.0) as f32
        } else {
            1.0
        }
    });
}

pub fn og_ui_set_virtual_cursor(desc: String) {
    let d = wire::arg::<Option<wire::VirtualCursorDesc>>(Some(&desc), "desc").unwrap_or_default();
    let desc = d.map(|d| og_ui_next::tree::VirtualCursorDesc {
        speed: d.speed,
        accel: d.accel_ms,
        magnet: d.magnet,
        scroll: d.scroll,
    });
    if let Some(engine) = super::ui_desc::ui_engine() {
        crate::present::ui_next::set_virtual_cursor(engine, desc);
    }
}

pub fn og_ui_virtual_cursor() -> Option<crate::host::wire::VirtualCursor> {
    let (p, shown) = crate::present::ui_next::virtual_cursor(super::ui_desc::ui_engine()?)?;
    Some(wire::VirtualCursor {
        x: p[0],
        y: p[1],
        shown,
    })
}

pub fn og_ui_tick() -> u32 {
    with_frame("og.ui.tick", |f| f.tick as u32).unwrap_or(0)
}

pub fn og_ui_preview_viewport(world: u32, viewport: String) {
    let v = wire::arg::<Option<wire::Viewport>>(Some(&viewport), "viewport").unwrap_or_default();
    let set = v.map(|v| {
        let (a, c) = (v.safe_area, v.covered.unwrap_or_default());
        let n = |x: Option<f32>| x.unwrap_or(0.0);
        Viewport {
            width: v.width.max(1.0),
            height: v.height.max(1.0),
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
            reduced_transparency: false,
            mac_keys: false,
            covered: og_ui::Rect::new(c.x, c.y, c.width, c.height),
            // The renderer's surface says (viewport_of).
            wide_gamut: false,
            high_range: false,
        }
    });
    if let Ok(eng) = engine(world) {
        eng.modules.ui.preview.set(set);
    }
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
