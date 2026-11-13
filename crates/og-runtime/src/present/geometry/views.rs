//! The debug views: their names, what each covers, and the physics, nav and bounds views drawn as lines.

use super::*;

/// The debug views (`render.DebugView`), by variant index.
pub(super) const VIEWS: [&str; 18] = [
    "none",
    "physics",
    "nav",
    "bounds",
    "lod",
    "cascades",
    "ao",
    "albedo",
    "iblDiffuse",
    "iblSpecular",
    "direct",
    "aoDepth",
    "aoRaw",
    "terrain",
    "terrainLayers",
    "terrainMip",
    "terrainProjection",
    "terrainDensity",
];

/// The renderer's own view for a debug view drawn in place of the lit
/// image (`RenderScene::debug`); the overlay views draw over it.
pub(super) fn render_view(view: &str) -> og_render::DebugView {
    use og_render::DebugView as V;
    match view {
        "cascades" => V::Cascades,
        "ao" => V::AmbientOcclusion,
        "albedo" => V::Albedo,
        "iblDiffuse" => V::IblDiffuse,
        "iblSpecular" => V::IblSpecular,
        "direct" => V::DirectLight,
        "aoDepth" => V::AoDepth,
        "aoRaw" => V::AoRaw,
        "terrain" => V::Terrain,
        "terrainLayers" => V::TerrainLayers,
        "terrainMip" => V::TerrainMip,
        "terrainProjection" => V::TerrainProjection,
        "terrainDensity" => V::TerrainDensity,
        _ => V::None,
    }
}

/// Explicit coverage for paths with no shading diagnostic or overlay.
pub(crate) fn debug_coverage(view: &str) -> serde_json::Value {
    let supported: &[&str] = match view {
        "none" => return serde_json::Value::Null,
        "physics" => &["colliders"],
        "nav" => &["navigation"],
        "bounds" => &["mesh instances"],
        "lod" => &["og.Model"],
        "terrain" | "terrainLayers" | "terrainMip" | "terrainProjection" | "terrainDensity" => {
            &["terrain"]
        }
        "albedo" => &[
            "standard",
            "physical",
            "custom",
            "terrain",
            "grass",
            "scatter",
            "impostors",
            "decals on standard/physical",
        ],
        "ao" | "aoDepth" | "aoRaw" => &[
            "standard",
            "physical",
            "grass",
            "scatter",
            "water",
            "decals on standard/physical",
        ],
        "cascades" => &[
            "standard lit",
            "physical",
            "terrain",
            "grass",
            "decals on standard/physical",
        ],
        "iblDiffuse" | "iblSpecular" | "direct" => {
            &["standard lit", "terrain", "grass", "decals on standard"]
        }
        _ => &[],
    };
    serde_json::json!({
        "supported": supported,
        "otherPaths": "unsupported",
        "unsupportedShading": "magenta/black checker for 3D surfaces; sprites, 2D fluids, lines, points and UI have no shading diagnostics",
        "overlays": "physics/nav/bounds/lod retain beauty shading; only the listed subjects receive the overlay",
        "decals": "part of the receiving standard/physical surface, not a separate shading pass",
    })
}

/// The debug view drawn this frame (`none` while the overlay is off):
/// `lod` is drawn by extraction itself, tinting each model by its level.
pub(crate) fn current_view(rs: &RenderState) -> &'static str {
    if rs.geo.disabled {
        return "none";
    }
    VIEWS[rs.geo.view_index(&rs.options)]
}

/// A RenderOptions.debugView value's index in [`VIEWS`] (the IDL's
/// DebugView order); none or an unknown name is `none`.
pub(super) fn view_of(v: &Value) -> usize {
    wire::DebugView::from_json(v).map_or(0, |d| d as usize)
}

pub(super) fn physics_view(rs: &mut RenderState) {
    let Some(lines) = rs.draw.physics.as_ref().and_then(|p| p.debug_lines()) else {
        return;
    };
    if lines.is_empty() {
        return;
    }
    let mut set = debug::segments(Vec::with_capacity(lines.len() * 2), [1.0; 4], 1.5);
    set.colors.reserve(lines.len() * 2);
    for (a, b, c) in lines {
        set.points.extend([a, b]);
        set.colors.extend([c, c]);
    }
    rs.scene.effects.lines.push(set);
}

#[cfg(feature = "nav")]
pub(super) fn nav_view(rs: &mut RenderState) {
    let engine_id = rs.draw.engine;
    let Some((navs, nav, profile)) =
        crate::bind::agents::with(engine_id, |m| (m.navs.clone(), m.nav.clone(), m.profile))
    else {
        return;
    };
    let plane = crate::bind::agents::plane_of(profile);
    let lift = |p: Vec3| match plane {
        og_ai::nav::Plane::Xz => p + Vec3::Y * 0.05,
        og_ai::nav::Plane::Xy => p,
    };
    let mut edges = Vec::new();
    let mut blocked = Vec::new();
    {
        let navs = navs.lock().unwrap_or_else(|e| e.into_inner());
        for n in navs.entries.values() {
            match n {
                og_ai::nav::Nav::Grid(g) => {
                    let s = g.cell_size;
                    let corner = |x: f32, y: f32| {
                        lift(plane.to_world(g.origin + glam::Vec2::new(x, y) * s, 0.0))
                    };
                    let (w, ht) = (g.width as f32, g.height as f32);
                    edges.extend([
                        corner(0.0, 0.0),
                        corner(w, 0.0),
                        corner(w, 0.0),
                        corner(w, ht),
                        corner(w, ht),
                        corner(0.0, ht),
                        corner(0.0, ht),
                        corner(0.0, 0.0),
                    ]);
                    for y in 0..g.height {
                        for x in 0..g.width {
                            if g.cost(x as i32, y as i32) != og_ai::nav::BLOCKED {
                                continue;
                            }
                            let (fx, fy) = (x as f32, y as f32);
                            blocked.extend([
                                corner(fx + 0.15, fy + 0.15),
                                corner(fx + 0.85, fy + 0.85),
                                corner(fx + 0.85, fy + 0.15),
                                corner(fx + 0.15, fy + 0.85),
                            ]);
                        }
                    }
                }
                og_ai::nav::Nav::Mesh(m) => {
                    for tile in &m.tiles {
                        for poly in &tile.polys {
                            let n = poly.verts.len();
                            for i in 0..n {
                                let a = tile.verts[poly.verts[i] as usize];
                                let b = tile.verts[poly.verts[(i + 1) % n] as usize];
                                edges.extend([lift(a), lift(b)]);
                            }
                        }
                    }
                }
                og_ai::nav::Nav::Flow { .. } => {}
            }
        }
    }
    if !edges.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(edges, [0.2, 0.75, 1.0, 1.0], 1.0));
    }
    if !blocked.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(blocked, [1.0, 0.3, 0.25, 1.0], 1.0));
    }
    if let Some(nav) = nav {
        for (_, path) in nav.paths() {
            if path.corners.len() < 2 {
                continue;
            }
            let mut l = debug::segments(
                path.corners.iter().map(|p| lift(*p)).collect(),
                [1.0, 0.85, 0.2, 1.0],
                2.0,
            );
            l.mode = LineMode::Strip;
            rs.scene.effects.lines.push(l);
        }
    }
}

pub(super) fn bounds_view(rs: &mut RenderState) {
    let mut out = Vec::new();
    let instances: Vec<(MeshHandle, Mat4)> = rs
        .scene
        .objects
        .instances
        .iter()
        .map(|i| (i.mesh, i.transform))
        .collect();
    for (mesh, m) in instances {
        let Some((min, max)) = mesh_bounds(rs, mesh) else {
            continue;
        };
        let centre = (min + max) * 0.5;
        out.extend(debug::box_edges(
            m * Mat4::from_translation(centre),
            (max - min) * 0.5,
        ));
    }
    if !out.is_empty() {
        rs.scene
            .effects
            .lines
            .push(debug::segments(out, [0.3, 1.0, 0.45, 1.0], 1.0));
    }
}
