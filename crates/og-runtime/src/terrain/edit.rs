//! Terrain edits: the brush, stamps and layers.

use super::*;

/// The brush's weight at distance `d` from its centre: 1 inside the
/// unfaded part, smoothly to 0 at the radius.
fn weight(d: f32, radius: f32, falloff: f32) -> f32 {
    if d >= radius {
        return 0.0;
    }
    let inner = radius * (1.0 - falloff.clamp(0.0, 1.0));
    if d <= inner {
        return 1.0;
    }
    let t = (d - inner) / (radius - inner).max(1e-6);
    let s = 1.0 - t;
    s * s * (3.0 - 2.0 * s)
}

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
    if t.field.is_none() {
        t.field = Some(t.cooked.field().map_err(|e| e.to_string())?);
    }
    t.edited = true;
    t.generation = next_generation();
    let d = t.field.as_mut().expect("decoded above");
    let c = Vec2::new(center.x, center.z);
    let half = d.size * 0.5;
    let (lo, hi) = (c - Vec2::splat(radius), c + Vec2::splat(radius));
    if c.x + radius < -half.x
        || c.x - radius > half.x
        || c.y + radius < -half.y
        || c.y - radius > half.y
    {
        return Err(format!(
            "the brush at ({}, {}) with radius {world_radius} misses the terrain ({} × {} m of its own space around ({}, {}))",
            world_center.x, world_center.z, d.size.x, d.size.y, m.w_axis.x, m.w_axis.z
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
        let n = paint(
            d,
            c,
            radius,
            falloff,
            strength.min(1.0),
            layer as usize,
            t.layers.len().div_ceil(4).max(1),
        );
        return Ok((n, lo, hi));
    }
    if mode >= 5 {
        return Ok((
            cut(d, c, radius, falloff, strength.min(1.0), mode == 5),
            lo,
            hi,
        ));
    }
    let cell = d.cell();
    let (cols, rows) = (d.columns as i64, d.rows as i64);
    let to_grid = |p: Vec2| ((p + half) / cell).to_array();
    let [gx0, gz0] = to_grid(c - Vec2::splat(radius));
    let [gx1, gz1] = to_grid(c + Vec2::splat(radius));
    let (c0, c1) = (
        (gx0.floor() as i64).max(0),
        (gx1.ceil() as i64).min(cols - 1),
    );
    let (r0, r1) = (
        (gz0.floor() as i64).max(0),
        (gz1.ceil() as i64).min(rows - 1),
    );
    // One copy of the field (the scene holds the other), read from the
    // unedited one.
    let old = d.heights.clone();
    let heights = Arc::make_mut(&mut d.heights);
    let mut touched = 0;
    let target = center.y;
    for r in r0..=r1 {
        for col in c0..=c1 {
            let p = Vec2::new(col as f32, r as f32) * cell - half;
            let w = weight((p - c).length(), radius, falloff);
            if w <= 0.0 {
                continue;
            }
            let k = (r * cols + col) as usize;
            let h = old[k];
            heights[k] = match mode {
                0 => h + strength * w,
                1 => h - strength * w,
                2 => h + (target - h) * (strength.min(1.0) * w),
                _ => {
                    let at = |c: i64, r: i64| {
                        old[(r.clamp(0, rows - 1) * cols + c.clamp(0, cols - 1)) as usize]
                    };
                    let mean =
                        (at(col - 1, r) + at(col + 1, r) + at(col, r - 1) + at(col, r + 1)) * 0.25;
                    h + (mean - h) * (strength.min(1.0) * w)
                }
            };
            touched += 1;
        }
    }
    Ok((touched, lo, hi))
}

/// Cuts holes (`hole`) or fills them in under the brush: every cell whose
/// centre the brush weighs at least half (its falloff times `strength`).
/// A terrain left with no hole has no hole map.
fn cut(d: &mut TerrainData, c: Vec2, radius: f32, falloff: f32, strength: f32, hole: bool) -> u32 {
    let (cx, cz) = d.cells();
    let mut holes = d
        .holes
        .as_ref()
        .map_or_else(|| vec![0u8; (cx * cz) as usize], |h| h.to_vec());
    let cell = d.cell();
    let half = d.size * 0.5;
    let mut touched = 0;
    let (c0, c1) = under(c.x - radius, c.x + radius, half.x, cell.x, cx);
    let (r0, r1) = under(c.y - radius, c.y + radius, half.y, cell.y, cz);
    for r in r0..r1 {
        for col in c0..c1 {
            let p = (Vec2::new(col as f32, r as f32) + 0.5) * cell - half;
            if weight((p - c).length(), radius, falloff) * strength < 0.5 {
                continue;
            }
            holes[(r * cx + col) as usize] = u8::from(hole);
            touched += 1;
        }
    }
    d.holes = holes.contains(&1).then(|| holes.into());
    touched
}

/// Paints `layer` into the control map (made all first-layer when the
/// terrain has none): the layer's channel raised towards 1 and the others
/// lowered, keeping their sum. The map first grows to `pages` pages of
/// four layers, a new page all 1 (what a layer past the pages read).
fn paint(
    d: &mut TerrainData,
    c: Vec2,
    radius: f32,
    falloff: f32,
    strength: f32,
    layer: usize,
    pages: usize,
) -> u32 {
    let control = d.control.get_or_insert_with(|| {
        let (w, h) = (d.columns.clamp(2, 1024), d.rows.clamp(2, 1024));
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            rgba.extend_from_slice(&[255, 0, 0, 0]);
        }
        og_render::TerrainControl {
            width: w,
            height: h,
            rgba: rgba.into(),
        }
    });
    let mut rgba = control.rgba.to_vec();
    let page = (control.width * control.height * 4) as usize;
    let pages = pages.max(control.pages() as usize);
    rgba.resize(page * pages, 255);
    let px = d.size / Vec2::new(control.width as f32, control.height as f32);
    let half = d.size * 0.5;
    let mut touched = 0;
    let (x0, x1) = under(c.x - radius, c.x + radius, half.x, px.x, control.width);
    let (y0, y1) = under(c.y - radius, c.y + radius, half.y, px.y, control.height);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) * px - half;
            let w = weight((p - c).length(), radius, falloff) * strength;
            if w <= 0.0 {
                continue;
            }
            let k = ((y * control.width + x) * 4) as usize;
            for ch in 0..4 * pages {
                let at = (ch / 4) * page + k + ch % 4;
                let v = rgba[at] as f32 / 255.0;
                let to = if ch == layer { 1.0 } else { 0.0 };
                rgba[at] = ((v + (to - v) * w) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            touched += 1;
        }
    }
    control.rgba = rgba.into();
    touched
}

/// The cells (of `size` from `-half`, `n` of them) whose centres can lie
/// in `a`..`b`: a range to visit instead of every cell.
fn under(a: f32, b: f32, half: f32, size: f32, n: u32) -> (u32, u32) {
    let at = |v: f32| (((v + half) / size).max(0.0) as u32).min(n);
    (at(a).saturating_sub(1), (at(b) + 2).min(n))
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
                    t.field = Some(t.cooked.field().map_err(|e| e.to_string())?);
                }
                t.edited = true;
                stamp::apply(t.field.as_mut().expect("decoded above"), b)?
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
        if collision && log {
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
) -> Result<crate::wire::TerrainEdit, String> {
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
) -> Result<crate::wire::TerrainLayerDesc, String> {
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
