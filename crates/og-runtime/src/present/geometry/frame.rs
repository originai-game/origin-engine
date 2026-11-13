//! Extraction: the frame's lines, points and debug draw handed to the renderer.

use super::*;

/// The vertices of a geometry handle field as points and colours.
pub(super) fn vertices_of(
    rs: &RenderState,
    geometry: u64,
    alpha: f32,
) -> Option<(Vec<Vec3>, Vec<[f32; 4]>)> {
    let index = index_of(geometry as f64)?;
    let mesh = rs.scene.mesh(*rs.geometries.get(index)?)?;
    let points = mesh
        .vertices
        .iter()
        .map(|v| Vec3::from(v.position))
        .collect();
    let colors = mesh
        .vertices
        .iter()
        .map(|v| [v.color[0], v.color[1], v.color[2], v.color[3] * alpha])
        .collect();
    Some((points, colors))
}

/// Lines, points and debug draw into the frame (`crate::present::extract`).
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy, vp: (f32, f32)) {
    rs.scene.frame.css_height = vp.1;
    let now = world.time().tick;
    // Immediate sets of the tick drawn (or the one before it, whose frame
    // this is).
    if rs.geo.tick == now || rs.geo.tick + 1 == now {
        rs.scene.effects.lines.extend(rs.geo.lines.iter().cloned());
        rs.scene
            .effects
            .points
            .extend(rs.geo.points.iter().cloned());
    }
    // og.Lines and og.Points.
    if let Some(layout) = rs.draw.lines.clone() {
        for (e, v) in crate::present::fx::rows(world, Some(&layout)) {
            let Ok(v) = wire::record::Lines::from_json(&v) else {
                continue;
            };
            // Drawn by the cameras its layers admit (the renderer tests them).
            let layers = v.layers;
            if !v.visible {
                continue;
            }
            let alpha = v.opacity.clamp(0.0, 1.0);
            let Some((points, colors)) = vertices_of(rs, v.geometry, alpha) else {
                continue;
            };
            let dash = v.dash_size;
            let at = rs.scene.effects.lines.len();
            if layers != 1 {
                rs.scene.effects.line_layers.resize(at, 1);
                rs.scene.effects.line_layers.push(layers);
            }
            rs.scene.effects.lines.push(Lines {
                points,
                colors,
                color: color4(&v.color, 1.0),
                mode: line_mode(v.mode),
                width: v.width.max(0.0),
                world_width: v.world_units,
                depth_test: v.depth_test,
                screen: false,
                dashed: (dash > 0.0).then(|| (dash, v.gap_size.max(0.0))),
                layer: LineLayer::Scene,
                transform: h.world_of(e),
            });
        }
    }
    if let Some(layout) = rs.draw.points.clone() {
        for (e, v) in crate::present::fx::rows(world, Some(&layout)) {
            let Ok(v) = wire::record::Points::from_json(&v) else {
                continue;
            };
            // Drawn by the cameras its layers admit (the renderer tests them).
            let layers = v.layers;
            if !v.visible {
                continue;
            }
            let alpha = v.opacity.clamp(0.0, 1.0);
            let Some((positions, colors)) = vertices_of(rs, v.geometry, alpha) else {
                continue;
            };
            let at = rs.scene.effects.points.len();
            if layers != 1 {
                rs.scene.effects.point_layers.resize(at, 1);
                rs.scene.effects.point_layers.push(layers);
            }
            rs.scene.effects.points.push(Points {
                positions,
                colors,
                color: color4(&v.color, 1.0),
                size: v.size.max(0.0),
                size_attenuation: v.size_attenuation,
                depth_test: v.depth_test,
                screen: false,
                layer: LineLayer::Scene,
                transform: h.world_of(e),
            });
        }
    }
    // Debug draw.
    rs.geo.draws.retain(|(until, _)| *until >= now);
    rs.geo.texts.retain(|t| t.until >= now);
    // The lit image's views; a publish build draws none of them.
    rs.scene.frame.debug = if cfg!(feature = "publish") {
        og_render::DebugView::None
    } else {
        render_view(current_view(rs))
    };
    if rs.geo.disabled {
        return;
    }
    let draws: Vec<Lines> = rs.geo.draws.iter().map(|(_, l)| l.clone()).collect();
    rs.scene.effects.lines.extend(draws);
    match VIEWS[rs.geo.view_index(&rs.options)] {
        "physics" => physics_view(rs),
        #[cfg(feature = "nav")]
        "nav" => nav_view(rs),
        "bounds" => bounds_view(rs),
        _ => {}
    }
    texts(rs, vp);
}
