//! og.debug: lines, shapes and text for one frame or a duration, and the probe's commands.

use super::*;

/// A debug label (`og.debug.text`).
pub(super) struct DebugText {
    pub(super) until: u64,
    pub(super) position: Vec3,
    pub(super) text: String,
    pub(super) color: [f32; 4],
    pub(super) size: f32,
}

pub(super) struct DrawDesc {
    pub(super) color: [f32; 4],
    pub(super) width: f32,
    pub(super) until: u64,
    pub(super) rotation: Quat,
}

pub(super) fn draw_desc(eng: &Engine, desc: Option<String>) -> DrawDesc {
    let d: wire::DebugDrawDesc = wire::arg(desc.as_deref(), "desc").unwrap_or_default();
    let w = d.width;
    DrawDesc {
        color: color4(&d.color, 1.0),
        width: if w.is_finite() { w.max(0.0) } else { 2.0 },
        // Drawn at the frame after the tick that asked (its world reads
        // the next tick), and `duration` ticks past it.
        until: tick(eng) + 1 + u64::from(d.duration),
        rotation: unit(d.rotation),
    }
}

pub(super) fn draw(world: u32, desc: Option<String>, shape: impl FnOnce(&DrawDesc) -> Vec<Lines>) {
    let Ok(eng) = engine(world) else {
        return;
    };
    let dd = draw_desc(&eng, desc);
    let sets = shape(&dd);
    let mut rs = eng.render.borrow_mut();
    rs.geo.draws.extend(sets.into_iter().map(|l| (dd.until, l)));
}

pub(super) fn arg_vec3(text: &str) -> Vec3 {
    wire::arg(Some(text), "point").unwrap_or_default()
}

pub fn og_debug_line(world: u32, from: String, to: String, desc: Option<String>) {
    let (a, b) = (arg_vec3(&from), arg_vec3(&to));
    draw(world, desc, |d| {
        vec![debug::segments(vec![a, b], d.color, d.width)]
    });
}

pub fn og_debug_box(world: u32, center: String, size: String, desc: Option<String>) {
    let (c, s) = (arg_vec3(&center), arg_vec3(&size));
    draw(world, desc, |d| {
        let m = Mat4::from_rotation_translation(d.rotation, c);
        vec![debug::segments(
            debug::box_edges(m, s * 0.5),
            d.color,
            d.width,
        )]
    });
}

pub fn og_debug_sphere(world: u32, center: String, radius: f64, desc: Option<String>) {
    let c = arg_vec3(&center);
    draw(world, desc, |d| {
        vec![debug::segments(
            debug::sphere(c, radius as f32, d.rotation),
            d.color,
            d.width,
        )]
    });
}

pub fn og_debug_arrow(world: u32, from: String, to: String, desc: Option<String>) {
    let (a, b) = (arg_vec3(&from), arg_vec3(&to));
    draw(world, desc, |d| {
        let head = (b - a).length() * 0.2;
        vec![debug::segments(debug::arrow(a, b, head), d.color, d.width)]
    });
}

pub fn og_debug_axes(world: u32, position: String, size: f64, desc: Option<String>) {
    let p = arg_vec3(&position);
    draw(world, desc, |d| {
        let m = Mat4::from_rotation_translation(d.rotation, p);
        debug::axes(m, size as f32, d.width).into()
    });
}

pub fn og_debug_text(world: u32, position: String, text: String, desc: Option<String>) {
    let Ok(eng) = engine(world) else {
        return;
    };
    let p = arg_vec3(&position);
    let d = draw_desc(&eng, desc);
    eng.render.borrow_mut().geo.texts.push(DebugText {
        until: d.until,
        position: p,
        text,
        color: d.color,
        size: if d.width <= 2.0 { 14.0 } else { d.width },
    });
}

pub fn og_debug_clear(world: u32) {
    if let Ok(eng) = engine(world) {
        let mut rs = eng.render.borrow_mut();
        rs.geo.draws.clear();
        rs.geo.texts.clear();
    }
}

pub fn og_debug_set_view(world: u32, view: String) {
    if let Ok(eng) = engine(world) {
        // A DebugView by name (an index is not part of the API).
        eng.render.borrow_mut().geo.view = Some(view_of(&Value::String(view)));
    }
}

pub fn og_debug_view(world: u32) -> &'static str {
    let i = engine(world).map_or(0, |eng| {
        let rs = eng.render.borrow();
        rs.geo.view_index(&rs.options)
    });
    VIEWS[i]
}

pub fn og_debug_set_enabled(world: u32, enabled: bool) {
    if let Ok(eng) = engine(world) {
        eng.render.borrow_mut().geo.disabled = !enabled;
    }
}

pub fn og_debug_enabled(world: u32) -> bool {
    engine(world).is_ok_and(|eng| !eng.render.borrow().geo.disabled)
}

/// The command channel's `debug:on|off|<view>`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "debug",
        og_probe::CommandDesc {
            usage: "debug:on|off|none|physics|nav|bounds|lod|cascades|ao|albedo|iblDiffuse|iblSpecular|direct|aoDepth|aoRaw|terrain|terrainLayers|terrainMip|terrainProjection|terrainDensity".into(),
            doc: "show or hide the debug overlay (og.debug's draws), or pick its view: physics (colliders, joints, contacts), nav (grids, navmeshes, agents' paths), bounds (every drawn mesh's box), lod (every model tinted by its level of detail), cascades (the shadow cascade per pixel), ao (the ambient occlusion term), albedo, iblDiffuse, iblSpecular, direct (the lit materials' lighting terms), aoDepth (the AO pass's depth), aoRaw (the AO before its blur), terrain (projection weights and clipmap cells), terrainLayers (strongest raw layer), terrainMip (normal-map mip footprint), terrainProjection (unobscured XYZ projection weights), terrainDensity (XYZ vertical albedo texel density encoded as d/(1+d)), none".into(),
        },
        move |_world, args| {
            let arg = args.first().map(String::as_str).unwrap_or("on");
            let eng = engine(engine_id)?;
            let mut rs = eng.render.borrow_mut();
            match arg {
                "on" => rs.geo.disabled = false,
                "off" => rs.geo.disabled = true,
                v => {
                    let i = VIEWS.iter().position(|n| *n == v).ok_or_else(|| {
                        format!("debug:{v}: write debug:on, debug:off or a view: {}", VIEWS.join(", "))
                    })?;
                    rs.geo.view = Some(i);
                    rs.geo.disabled = false;
                }
            }
            Ok(())
        },
    );
}

/// Debug labels: each world point projected to the play surface, the
/// label centred above it in the UI's body font.
// Render-only: the labels are drawn into the frame, never read by the tick.
#[allow(clippy::disallowed_methods)]
pub(super) fn texts(rs: &mut RenderState, vp: (f32, f32)) {
    if rs.geo.texts.is_empty() || vp.0 <= 0.0 || vp.1 <= 0.0 {
        return;
    }
    let cam = &rs.scene.frame.camera;
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let aspect = vp.0 / vp.1;
    let proj = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => {
            glam::camera::rh::proj::directx::perspective(
                fov_y_degrees.to_radians(),
                aspect,
                cam.near,
                cam.far,
            )
        }
        og_render::Projection::Orthographic { height } => {
            let h = height.abs().max(1e-6) * 0.5;
            glam::camera::rh::proj::directx::orthographic(
                -h * aspect,
                h * aspect,
                -h,
                h,
                cam.near,
                cam.far,
            )
        }
    };
    let vpm = proj * view;
    let ui = rs.ui.clone();
    let mut out = Vec::new();
    for t in &rs.geo.texts {
        let clip = vpm * t.position.extend(1.0);
        if clip.w <= 1e-6 {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < 0.0 || ndc.z > 1.0 {
            continue;
        }
        let Some(body) = ui.as_ref().and_then(|u| {
            u.lock()
                .fonts()
                .for_text(og_ui::FontRole::Body, &t.text)
                .handle
        }) else {
            continue;
        };
        let pages = rs
            .ui_pages
            .get(&body)
            .cloned()
            .unwrap_or_else(|| vec![body]);
        let Some(first) = pages.first().and_then(|p| rs.scene.font(*p)) else {
            continue;
        };
        let width = first.measure(&t.text) * t.size;
        let x = (ndc.x * 0.5 + 0.5) * vp.0 - width * 0.5;
        let y = (0.5 - ndc.y * 0.5) * vp.1 - t.size * 0.6;
        for page in pages {
            out.push(og_render::Text {
                font: page,
                text: t.text.clone(),
                position: Vec3::new(x, y, 0.0),
                size: t.size,
                rotation: 0.0,
                color: t.color,
                layer: og_render::screen_layer(og_render::UI_LAYER),
                clip: None,
                tracking: 0.0,
                weight: 0.0,
                glow: None,
                placements: Vec::new(),
            });
        }
    }
    if out.is_empty() {
        return;
    }
    let o = rs
        .scene
        .ui
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(vp.0, vp.1, 0.0));
    o.texts.extend(out);
}
