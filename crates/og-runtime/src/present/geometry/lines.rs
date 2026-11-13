//! The immediate og.render.lines and og.render.points.

use super::*;

pub(super) fn line_mode(m: wire::LineMode) -> LineMode {
    match m {
        wire::LineMode::Strip => LineMode::Strip,
        wire::LineMode::Segments => LineMode::Segments,
        wire::LineMode::Loop => LineMode::Loop,
    }
}

/// Starts the immediate queues of a new tick.
pub(super) fn queue(rs: &mut RenderState, tick: u64) -> &mut GeoState {
    let g = &mut rs.geo;
    if g.tick != tick {
        g.lines.clear();
        g.points.clear();
        g.tick = tick;
    }
    g
}

pub fn og_render_lines(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::LinesDesc = wire::arg(desc.as_deref(), "desc")?;
        let points = finite_points(d.points, "points")?;
        if points.len() < 2 {
            return Err(format!(
                "lines need at least two points, got {}",
                points.len()
            ));
        }
        let alpha = d.opacity.clamp(0.0, 1.0);
        let colors = colors_of(&d.colors, points.len(), alpha)?;
        let screen = d.screen;
        let lines = Lines {
            colors,
            color: color4(&d.color, alpha),
            mode: match d.mode {
                wire::LineMode::Strip => LineMode::Strip,
                wire::LineMode::Segments => LineMode::Segments,
                wire::LineMode::Loop => LineMode::Loop,
            },
            width: size(d.width, "width")?,
            world_width: d.world_units && !screen,
            depth_test: d.depth_test && !screen,
            screen,
            dashed: d
                .dashed
                .then(|| (d.dash_size.max(0.0), d.gap_size.max(0.0))),
            layer: if screen {
                LineLayer::Overlay
            } else {
                LineLayer::Scene
            },
            points,
            transform: Mat4::IDENTITY,
        };
        let t = tick(&eng);
        queue(&mut eng.render.borrow_mut(), t).lines.push(lines);
        Ok(())
    })()
}

pub fn og_render_points(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::PointsDesc = wire::arg(desc.as_deref(), "desc")?;
        let positions = finite_points(d.points, "points")?;
        let alpha = d.opacity.clamp(0.0, 1.0);
        let colors = colors_of(&d.colors, positions.len(), alpha)?;
        let screen = d.screen;
        let points = Points {
            positions,
            colors,
            color: color4(&d.color, alpha),
            size: size(d.size, "size")?,
            size_attenuation: d.size_attenuation && !screen,
            depth_test: d.depth_test && !screen,
            screen,
            layer: if screen {
                LineLayer::Overlay
            } else {
                LineLayer::Scene
            },
            transform: Mat4::IDENTITY,
        };
        let t = tick(&eng);
        queue(&mut eng.render.borrow_mut(), t).points.push(points);
        Ok(())
    })()
}
