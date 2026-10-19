//! The IDL's `terrain` module (phase 2): cooked terrains loaded into the
//! renderer (`og_render::TerrainData` and the layers' textures) and into
//! physics (a height field under a Geometry handle of its own), drawn from
//! og.Terrain components at extraction, sampled and edited from game code.
//! The registry is the render state's: terrains are assets, kept across a
//! hot reload, and their edits are asset changes, not world state.

use super::engine::{Engine, engine};
use super::extract::{Hierarchy, RenderState};
use super::fx::{color, f, on, rows, u};
use super::{ok, parse, refuse};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use og_ecs::World;
use og_render::{TerrainData, TerrainLayer, TerrainLod};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The Geometry handles of terrains' height fields: a range of their own,
/// beside glTF geometries' (`assets::handle`).
const GEOMETRY_BASE: u64 = 0x7e << 32;

/// One loaded terrain.
pub(crate) struct LoadedTerrain {
    pub render: og_render::TerrainHandle,
    /// The heights and control map as they are now (edits applied).
    pub data: TerrainData,
    pub layers: Vec<TerrainLayer>,
    pub height_blend: f32,
    pub lod: TerrainLod,
    pub cast_shadows: bool,
    pub collision: bool,
}

/// Every terrain an engine loaded, by load index, and the og.Terrain
/// layout extraction reads.
#[derive(Default)]
pub(crate) struct TerrainState {
    pub loaded: Vec<LoadedTerrain>,
    pub by_id: BTreeMap<String, usize>,
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn handle(index: usize) -> u64 {
    super::assets::handle(index)
}

fn index_of(eng: &Engine, data: f64) -> Result<usize, String> {
    super::assets::index_of(data)
        .filter(|&i| i < eng.render.borrow().terrain.loaded.len())
        .ok_or_else(|| format!("{data} is not a terrain loaded with og.terrain.load"))
}

fn v3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

fn j3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

/// A terrain's placement from its entity's world position, rotation and
/// scale as game code passes them (a missing or zero quaternion is the
/// identity, a missing scale is 1).
pub(crate) fn placement(origin: &Value, quaternion: &Value, scale: &Value) -> Mat4 {
    let q = Quat::from_xyzw(
        f(quaternion, "x", 0.0),
        f(quaternion, "y", 0.0),
        f(quaternion, "z", 0.0),
        f(quaternion, "w", 0.0),
    );
    let q = if q.length_squared() > 1e-12 {
        q.normalize()
    } else {
        Quat::IDENTITY
    };
    let s = if scale.is_object() {
        Vec3::new(f(scale, "x", 1.0), f(scale, "y", 1.0), f(scale, "z", 1.0))
    } else {
        Vec3::ONE
    };
    Mat4::from_scale_rotation_translation(s, q, v3(origin))
}

/// Metres across the terrain's surface per metre of its own space (the mean
/// of its x and z scales) and up its y.
fn scales(m: Mat4) -> (f32, f32) {
    let along = (m.x_axis.truncate().length() + m.z_axis.truncate().length()) * 0.5;
    (along.max(1e-6), m.y_axis.truncate().length().max(1e-6))
}

fn heightfield(d: &TerrainData) -> og_physics::HeightfieldDesc {
    og_physics::HeightfieldDesc {
        columns: d.columns,
        rows: d.rows,
        size: d.size,
        heights: d.heights.clone(),
    }
}

/// Registers a terrain's height field with physics and remembers it, so
/// a hot reload's new physics gets it too.
fn set_heightfield(eng: &Engine, geometry: u64, d: &TerrainData) -> Result<(), String> {
    let field = heightfield(d);
    let physics = eng.state.borrow().physics.clone();
    if let Some(p) = physics {
        p.set_heightfield(geometry, field.clone())
            .map_err(|e| e.reason().to_string())?;
    }
    eng.state
        .borrow_mut()
        .physics_heightfields
        .insert(geometry, field);
    Ok(())
}

fn info(index: usize, t: &LoadedTerrain) -> Value {
    let (lo, hi) = t.data.range();
    json!({
        "data": handle(index),
        "geometry": GEOMETRY_BASE + index as u64,
        "size": { "x": t.data.size.x, "y": t.data.size.y },
        "columns": t.data.columns,
        "rows": t.data.rows,
        "minHeight": lo,
        "maxHeight": hi,
        "layers": t.layers.len(),
        "collision": t.collision,
        "castShadows": t.cast_shadows,
    })
}

/// `og.terrain.load`.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().terrain.by_id.get(&id) {
        return Ok(info(i, &eng.render.borrow().terrain.loaded[i]));
    }
    let cooked = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .terrain(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // The layers' textures, through og.load.texture's cache.
    let mut layers = Vec::with_capacity(cooked.layers.len());
    for l in &cooked.layers {
        let mut layer = l.layer;
        for (name, slot) in [
            (&l.albedo, &mut layer.albedo),
            (&l.normal, &mut layer.normal),
        ] {
            if let Some(n) = name {
                let i = super::assets::load_texture(eng, n)
                    .map_err(|e| format!("{id}: layer texture {e}"))?;
                *slot = eng.render.borrow().textures.get(i).copied();
            }
        }
        layers.push(layer);
    }
    let index = eng.render.borrow().terrain.loaded.len();
    set_heightfield(eng, GEOMETRY_BASE + index as u64, &cooked.data)?;
    let mut rs = eng.render.borrow_mut();
    let render = rs.scene.add_terrain(cooked.data.clone());
    let t = LoadedTerrain {
        render,
        data: cooked.data,
        layers,
        height_blend: cooked.height_blend,
        lod: cooked.lod,
        cast_shadows: cooked.cast_shadows,
        collision: cooked.collision,
    };
    let value = info(index, &t);
    rs.terrain.loaded.push(t);
    rs.terrain.by_id.insert(id, index);
    Ok(value)
}

pub fn og_terrain_load(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load(&eng, &path)))
}

/// The point in the terrain's space (relative to its centre) of world `x`,
/// `z` for a terrain placed by `m` (taken at the terrain's centre height, so
/// along its up axis), and whether it is over the terrain.
fn local(d: &TerrainData, m: Mat4, x: f32, z: f32) -> (Vec2, bool) {
    let l = m.inverse().transform_point3(Vec3::new(x, m.w_axis.y, z));
    let p = Vec2::new(l.x, l.z);
    let half = d.size * 0.5;
    (p, p.x.abs() <= half.x && p.y.abs() <= half.y)
}

pub fn og_terrain_sample(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let i = index_of(&eng, d["data"].as_f64().unwrap_or(0.0))?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i].data;
        let m = placement(&d["origin"], &d["quaternion"], &d["scale"]);
        let (p, inside) = local(t, m, f(&d, "x", 0.0), f(&d, "z", 0.0));
        let n = (Mat3::from_mat4(m).inverse().transpose() * t.normal(p)).normalize_or(Vec3::Y);
        let surface = m.transform_point3(Vec3::new(p.x, t.height(p), p.y));
        Ok(json!({
            "height": surface.y,
            "normal": j3(n),
            "slope": n.y.clamp(-1.0, 1.0).acos().to_degrees(),
            "inside": inside,
        }))
    })())
}

pub fn og_terrain_bounds(
    world: u32,
    data: f64,
    origin: Option<String>,
    quaternion: Option<String>,
    scale: Option<String>,
) -> String {
    wire((|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let m = placement(
            &parse(origin.as_deref())?,
            &parse(quaternion.as_deref())?,
            &parse(scale.as_deref())?,
        );
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i].data;
        let (lo, hi) = t.range();
        let half = t.size * 0.5;
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for k in 0..8 {
            let corner = m.transform_point3(Vec3::new(
                if k & 1 == 0 { -half.x } else { half.x },
                if k & 2 == 0 { lo } else { hi },
                if k & 4 == 0 { -half.y } else { half.y },
            ));
            min = min.min(corner);
            max = max.max(corner);
        }
        Ok(json!({ "min": j3(min), "max": j3(max) }))
    })())
}

const MODES: [&str; 5] = ["raise", "lower", "flatten", "smooth", "paint"];

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

/// Applies a brush to a terrain's data; answers the samples it touched.
pub(crate) fn apply(t: &mut LoadedTerrain, brush: &Value) -> Result<u32, String> {
    let mode = match &brush["mode"] {
        Value::String(s) => MODES
            .iter()
            .position(|m| m == s)
            .ok_or_else(|| format!("TerrainBrush.mode '{s}' is not one of {}", MODES.join(", ")))?,
        Value::Null => 0,
        other => return Err(format!("TerrainBrush.mode {other} is not a mode")),
    };
    let m = placement(&brush["origin"], &brush["quaternion"], &brush["scale"]);
    let (along, up) = scales(m);
    let world_center = v3(&brush["center"]);
    let center = m.inverse().transform_point3(world_center);
    let world_radius = f(brush, "radius", 8.0);
    let world_strength = f(brush, "strength", 1.0);
    // In the terrain's space: the radius across it, raise and lower up it.
    let radius = world_radius / along;
    let strength = if mode <= 1 {
        world_strength / up
    } else {
        world_strength
    };
    let falloff = f(brush, "falloff", 0.5);
    if !(world_radius > 0.0 && radius.is_finite() && world_strength >= 0.0 && strength.is_finite())
    {
        return Err("TerrainBrush.radius must be positive and strength at least 0".into());
    }
    let d = &mut t.data;
    let c = Vec2::new(center.x, center.z);
    let half = d.size * 0.5;
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
        let layer = u(brush, "layer", 0);
        if layer as usize >= t.layers.len().max(1) || layer > 3 {
            return Err(format!(
                "TerrainBrush.layer {layer}: the terrain has {} layer(s)",
                t.layers.len()
            ));
        }
        return Ok(paint(
            d,
            c,
            radius,
            falloff,
            strength.min(1.0),
            layer as usize,
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
    let old: Vec<f32> = d.heights.to_vec();
    let mut heights = old.clone();
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
    d.heights = Arc::from(heights);
    Ok(touched)
}

/// Paints `layer` into the control map (made all first-layer when the
/// terrain has none): the layer's channel raised towards 1 and the others
/// lowered, keeping their sum.
fn paint(
    d: &mut TerrainData,
    c: Vec2,
    radius: f32,
    falloff: f32,
    strength: f32,
    layer: usize,
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
    let px = d.size / Vec2::new(control.width as f32, control.height as f32);
    let half = d.size * 0.5;
    let mut touched = 0;
    for y in 0..control.height {
        for x in 0..control.width {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) * px - half;
            let w = weight((p - c).length(), radius, falloff) * strength;
            if w <= 0.0 {
                continue;
            }
            let k = ((y * control.width + x) * 4) as usize;
            for ch in 0..4 {
                let v = rgba[k + ch] as f32 / 255.0;
                let to = if ch == layer { 1.0 } else { 0.0 };
                rgba[k + ch] = ((v + (to - v) * w) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            touched += 1;
        }
    }
    control.rgba = rgba.into();
    touched
}

pub fn og_terrain_edit(world: u32, data: f64, brush: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let b = parse(brush.as_deref())?;
        let (touched, snapshot) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            let n = apply(t, &b)?;
            let snapshot = (t.render, t.data.clone());
            (n, snapshot)
        };
        let (render, data) = snapshot;
        eng.render
            .borrow_mut()
            .scene
            .set_terrain(render, data.clone());
        set_heightfield(&eng, GEOMETRY_BASE + i as u64, &data)?;
        let (lo, hi) = data.range();
        Ok(json!({ "samples": touched, "minHeight": lo, "maxHeight": hi }))
    })())
}

fn layer_json(l: &TerrainLayer) -> Value {
    json!({
        "color": { "r": l.color.x, "g": l.color.y, "b": l.color.z },
        "roughness": l.roughness,
        "tile": l.tile,
        "triplanar": l.triplanar,
        "normalStrength": l.normal_strength,
        "slopeMin": l.slope[0],
        "slopeMax": l.slope[1],
        "slopeFade": l.slope_fade,
        "altitudeMin": l.altitude[0],
        "altitudeMax": l.altitude[1],
        "altitudeFade": l.altitude_fade,
    })
}

pub fn og_terrain_layer(world: u32, data: f64, index: u32) -> String {
    wire((|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        t.layers
            .get(index as usize)
            .map(layer_json)
            .ok_or_else(|| format!("layer {index}: the terrain has {} layer(s)", t.layers.len()))
    })())
}

pub fn og_terrain_set_layer(world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    match (|| -> Result<(), String> {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let v = parse(layer.as_deref())?;
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
            color: color(&v["color"], Vec3::ONE),
            roughness: f(&v, "roughness", 0.9).clamp(0.04, 1.0),
            tile: f(&v, "tile", 4.0).max(0.01),
            triplanar: on(&v, "triplanar", false),
            normal_strength: f(&v, "normalStrength", 1.0),
            slope: [f(&v, "slopeMin", 0.0), f(&v, "slopeMax", 90.0)],
            slope_fade: f(&v, "slopeFade", 4.0).max(0.0),
            altitude: [f(&v, "altitudeMin", -1.0e6), f(&v, "altitudeMax", 1.0e6)],
            altitude_fade: f(&v, "altitudeFade", 4.0).max(0.0),
        };
        Ok(())
    })() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

/// og.Terrain components into `rs.scene.terrains`: each terrain at its
/// entity's world position with its layers; then og.TerrainScatter
/// components into instances around the camera, answering their keys
/// (entity, terrain cell) in the order they were added.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Vec<(u64, u32)> {
    rs.scene.terrains.clear();
    let layout = rs.draw.terrain.clone();
    for (e, v) in rows(world, layout.as_ref()) {
        let Some(i) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0)) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes
                .push(format!("og.Terrain on entity {e} names no loaded terrain"));
            continue;
        };
        let world = h.world_of(e);
        let spacing = f(&v, "lodSpacing", 0.0);
        let ring = u(&v, "lodRing", 0);
        rs.scene.terrains.push(og_render::Terrain {
            terrain: t.render,
            transform: world,
            layers: t.layers.clone(),
            height_blend: t.height_blend,
            cast_shadows: on(&v, "castShadows", true) && t.cast_shadows,
            lod: TerrainLod {
                spacing: if spacing > 0.0 {
                    spacing
                } else {
                    t.lod.spacing
                },
                ring: if ring > 0 { ring } else { t.lod.ring },
                budget: t.lod.budget,
            },
        });
    }
    let mut keys = Vec::new();
    let layout = rs.draw.scatter.clone();
    let eye = rs.scene.camera.position;
    for (e, v) in rows(world, layout.as_ref()) {
        let Some(i) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0)) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes.push(format!(
                "og.TerrainScatter on entity {e} names no loaded terrain"
            ));
            continue;
        };
        let index = |key: &str| (v[key].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
        let (Some(&mesh), Some(&material)) = (
            rs.geometries.get(index("geometry")),
            rs.materials.get(index("material")),
        ) else {
            continue;
        };
        let terrain = og_render::Terrain {
            transform: h.world_of(e),
            layers: t.layers.clone(),
            ..og_render::Terrain::new(t.render)
        };
        let scatter = og_render::TerrainScatter {
            layer: u(&v, "layer", 0),
            density: f(&v, "density", 2.0),
            distance: f(&v, "distance", 40.0),
            fade: f(&v, "fade", 0.3),
            scale: [f(&v, "minScale", 0.8), f(&v, "maxScale", 1.2)],
            align: f(&v, "align", 0.0),
            seed: u(&v, "seed", 1),
        };
        for (cell, m) in terrain.scatter(&t.data, &scatter, eye) {
            rs.scene.add_instance(mesh, material, m);
            keys.push((e, cell));
        }
    }
    keys
}
