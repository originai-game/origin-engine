//! Terrain edits: the brush, stamps and layers.

use super::*;

/// Applies a brush to a terrain's data; answers the samples it touched
/// and the rectangle of the terrain's space it can have changed.
pub(crate) fn apply(t: &mut LoadedTerrain, brush: &Value) -> Result<(u32, Vec2, Vec2), String> {
    let b: TerrainBrush = wire::value(brush, "brush")?;
    // raise, lower, flatten, smooth, paint, hole, fill.
    let mode = b.mode as usize;
    let m = placed(b.origin, b.quaternion, b.scale);
    let (along, up) = scales(m);
    let world_center = b.center;
    let center = m.inverse().transform_point3(world_center);
    let world_radius = b.radius;
    let world_strength = b.strength;
    // In the terrain's space: the radius across it, raise and lower up it.
    let radius = world_radius / along;
    let strength = if mode <= 1 {
        world_strength / up
    } else {
        world_strength
    };
    let falloff = b.falloff;
    if !(world_radius > 0.0 && radius.is_finite() && world_strength >= 0.0 && strength.is_finite())
    {
        return Err("TerrainBrush.radius must be positive and strength at least 0".into());
    }
    let size = t.size();
    let c = Vec2::new(center.x, center.z);
    let half = size * 0.5;
    let (lo, hi) = (c - Vec2::splat(radius), c + Vec2::splat(radius));
    if c.x + radius < -half.x
        || c.x - radius > half.x
        || c.y + radius < -half.y
        || c.y - radius > half.y
    {
        return Err(format!(
            "the brush at ({}, {}) with radius {world_radius} misses the terrain ({} × {} m of its own space around ({}, {}))",
            world_center.x, world_center.z, size.x, size.y, m.w_axis.x, m.w_axis.z
        ));
    }
    if mode == 4 {
        let layer = b.layer;
        if layer as usize >= t.layers.len().max(1)
            || layer as usize >= og_render::MAX_TERRAIN_LAYERS
        {
            return Err(format!(
                "TerrainBrush.layer {layer}: the terrain has {} layer(s)",
                t.layers.len()
            ));
        }
    }
    if t.field.is_none() {
        let cooked = t
            .cooked
            .as_ref()
            .ok_or("runtime terrain is missing its heightfield")?;
        t.field = Some(cooked.field().map_err(|e| e.to_string())?);
    }
    t.edited = true;
    t.generation = next_generation();
    let d = t.field.as_mut().expect("decoded above");
    let touched = og_render::terrain_edit::apply(
        d,
        &og_render::terrain_edit::Brush {
            mode: mode as u32,
            center,
            radius,
            falloff,
            strength,
            layer: b.layer as usize,
            pages: t.layers.len().div_ceil(4).max(1),
        },
    );
    Ok((touched, lo, hi))
}

/// Loaded terrain `i`'s asset id, as the edit log names it.
pub(super) fn terrain_id(t: &TerrainState, i: usize) -> String {
    t.by_id
        .iter()
        .find(|(_, k)| **k == i)
        .map(|(id, _)| id.clone())
        .unwrap_or_default()
}

/// Applies a brush or a stamp's patch to loaded terrain `i` (and logs it
/// and hands its heights to physics, unless replaying).
pub(super) fn edit_loaded(eng: &Engine, i: usize, b: &Value, log: bool) -> Result<Value, String> {
    edit(eng, i, b, log).map(|e| e.to_json())
}

/// [`edit_loaded`]'s answer, typed.
fn edit(eng: &Engine, i: usize, b: &Value, log: bool) -> Result<TerrainEdit, String> {
    {
        let (touched, snapshot, collision) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            let (n, region) = if b.get("stamp").is_some() {
                if t.field.is_none() {
                    let cooked = t
                        .cooked
                        .as_ref()
                        .ok_or("runtime terrain is missing its heightfield")?;
                    t.field = Some(cooked.field().map_err(|e| e.to_string())?);
                }
                let res = stamp::apply(t.field.as_mut().expect("decoded above"), b)?;
                t.edited = true;
                t.generation = next_generation();
                res
            } else {
                let (n, lo, hi) = apply(t, b)?;
                (n, Some((lo, hi)))
            };
            let snapshot = (t.render, t.field.clone().expect("edited"), region);
            (n, snapshot, t.collision)
        };
        if log {
            let id = terrain_id(&eng.render.borrow().terrain, i);
            log_of(eng)
                .lock()
                .expect("the edit log")
                .edits
                .push((id, b.to_string()));
        }
        let (render, data, region) = snapshot;
        {
            let mut rs = eng.render.borrow_mut();
            // In place: the renderer builds again only the tiles the edit
            // reaches.
            match region {
                Some((lo, hi)) => rs.scene.edit_terrain(render, data.clone(), lo, hi),
                None => rs.scene.set_terrain(render, data.clone()),
            };
            rs.terrain.streamer.forget(i);
        }
        // A replay registers the final heights once (`replay_if_restored`).
        let paint =
            b.get("stamp").is_none() && wire::value::<TerrainBrush>(b, "brush")?.mode as usize == 4;
        if collision && log && !paint {
            match region {
                // Physics rebuilds only the chunks of its colliders under it.
                Some((lo, hi)) => edit_heightfield(eng, GEOMETRY_BASE + i as u64, &data, lo, hi)?,
                None => set_heightfield(eng, GEOMETRY_BASE + i as u64, &data)?,
            }
        }
        let (lo, hi) = data.range();
        let stamp = b["stamp"].as_u64().unwrap_or(0) as u32;
        Ok(TerrainEdit {
            samples: touched,
            min_height: lo,
            max_height: hi,
            stamp,
        })
    }
}

pub fn og_terrain_edit(
    world: u32,
    data: f64,
    brush: Option<String>,
) -> Result<crate::host::wire::TerrainEdit, String> {
    (|| {
        let eng = engine(world)?;
        replay_if_restored(&eng)?;
        let i = index_of(&eng, data)?;
        let b = parse(brush.as_deref())?;
        edit(&eng, i, &b, true)
    })()
}

pub fn og_terrain_stamp(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let given = parse(desc.as_deref())?;
        let d: TerrainStampDesc = wire::value(&given, "desc")?;
        // The transforms are read from the desc as given: a quaternion
        // without `w` (or a null member) reads as the stamp always read it.
        stamp::stamp(&eng, &d, &given)
    })()
}

pub fn og_terrain_unstamp(world: u32, data: f64, stamp: u32) -> Result<serde_json::Value, String> {
    (|| stamp::unstamp(&*engine(world)?, data, stamp))()
}

fn layer_desc(l: &TerrainLayer) -> TerrainLayerDesc {
    TerrainLayerDesc {
        color: Color {
            r: l.color.x,
            g: l.color.y,
            b: l.color.z,
        },
        roughness: l.roughness,
        tile: l.tile,
        triplanar: l.triplanar,
        normal_strength: l.normal_strength,
        slope_min: l.slope[0],
        slope_max: l.slope[1],
        slope_fade: l.slope_fade,
        altitude_min: l.altitude[0],
        altitude_max: l.altitude[1],
        altitude_fade: l.altitude_fade,
        project_slope: l.project_slope,
        anti_tiling: l.anti_tiling,
        height_contrast: l.height_contrast,
        macro_variation: l.macro_variation,
        displacement: l.displacement,
        cell_variation: l.cell_variation,
    }
}

pub fn og_terrain_layer(
    world: u32,
    data: f64,
    index: u32,
) -> Result<crate::host::wire::TerrainLayerDesc, String> {
    (|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        t.layers
            .get(index as usize)
            .map(layer_desc)
            .ok_or_else(|| format!("layer {index}: the terrain has {} layer(s)", t.layers.len()))
    })()
}

pub fn og_terrain_set_layer(
    world: u32,
    data: f64,
    index: u32,
    layer: Option<String>,
) -> Result<(), String> {
    match (|| -> Result<(), String> {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let v: TerrainLayerDesc = wire::arg(layer.as_deref(), "layer")?;
        let mut rs = eng.render.borrow_mut();
        let t = &mut rs.terrain.loaded[i];
        let count = t.layers.len();
        let l = t
            .layers
            .get_mut(index as usize)
            .ok_or_else(|| format!("layer {index}: the terrain has {count} layer(s)"))?;
        *l = TerrainLayer {
            albedo: l.albedo,
            normal: l.normal,
            albedo_layer: l.albedo_layer,
            normal_layer: l.normal_layer,
            color: Vec3::new(v.color.r, v.color.g, v.color.b),
            roughness: v.roughness.clamp(0.04, 1.0),
            tile: v.tile.max(0.01),
            triplanar: v.triplanar,
            normal_strength: v.normal_strength,
            slope: [v.slope_min, v.slope_max],
            slope_fade: v.slope_fade.max(0.0),
            altitude: [v.altitude_min, v.altitude_max],
            altitude_fade: v.altitude_fade.max(0.0),
            project_slope: v.project_slope.clamp(0.0, 90.0),
            anti_tiling: v.anti_tiling.clamp(0.0, 1.0),
            height_contrast: v.height_contrast.clamp(0.0, 1.0),
            macro_variation: v.macro_variation.clamp(0.0, 1.0),
            displacement: v.displacement.clamp(0.0, 4.0),
            cell_variation: v.cell_variation.clamp(0.0, 1.0),
        };
        Ok(())
    })() {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
