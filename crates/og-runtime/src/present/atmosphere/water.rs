//! Water drawn: og.Water (its wave heights and wakes from the physics,
//! its seabed and its river) and og.Water2D over an og.WaterLine.

use super::rgb;
use crate::host::wire::{FromJson, record};
use crate::present::extract::{Hierarchy, RenderState};
use crate::present::fx::kept_rows;
use glam::{Mat4, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, World};
use og_render::{Water, Water2d, WaterCut, WaterHeights, WaterWakes};
use serde_json::Value;
use std::sync::Arc;

/// A river's path handle and its width's and speed's bits.
type RiverKey = (u64, u32, u32);

/// The water on one engine: its rivers by [`RiverKey`], the newest last.
#[derive(Default)]
pub(crate) struct ModuleState {
    rivers: std::cell::RefCell<Vec<(RiverKey, Arc<og_env::river::River>)>>,
}

/// An og.Water's river (og.Water.river: its path sampled every metre or so,
/// riverWidth and riverSpeed), the one record the tick drifts floaters with
/// and the renderer draws: made once per path, width and speed (the same
/// `Arc` frame after frame, so the renderer keeps its ribbon). None without
/// a live path.
pub(crate) fn river_of(engine: u32, v: &Value) -> Option<Arc<og_env::river::River>> {
    let eng = crate::engine::engine(engine).ok()?;
    let rivers = &eng.modules.atmosphere.rivers;
    let w = record::Water::from_json(v).ok()?;
    let handle = Some(w.river).filter(|h| *h != 0)?;
    let (width, speed) = (w.river_width.max(0.0), w.river_speed.max(0.0));
    let key = (handle, width.to_bits(), speed.to_bits());
    if let Some(r) = rivers
        .borrow()
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, r)| r.clone())
    {
        return Some(r);
    }
    let path = crate::bind::timeline::path_of(engine, handle)?;
    // A point every metre, at least the path's own segments, at most 2048.
    let n = (path.length().ceil() as usize)
        .max(path.segments() + 1)
        .clamp(2, 2048);
    let points: Vec<Vec3> = (0..n)
        .map(|i| path.sample(i as f32 / (n - 1) as f32).position)
        .collect();
    let river = Arc::new(og_env::river::River::new(&points, width, speed)?);
    let mut c = rivers.borrow_mut();
    if c.len() >= 64 {
        c.remove(0);
    }
    c.push((key, river.clone()));
    Some(river)
}

/// og.Water and og.Water2D into `rs.scene`.
pub(super) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let engine = rs.engine;
    let RenderState {
        scene,
        draw,
        geometries,
        textures,
        terrain,
        ..
    } = rs;
    scene.world.water.clear();
    scene.sprite.water_2d.clear();
    let layouts = &draw.atmo;
    let physics = layouts.physics.as_ref();
    // The terrains a sea's seabed is read from (none in the 2D profile).
    #[cfg(not(feature = "profile2d"))]
    let terrain_layout = draw.terrain.as_ref();
    #[cfg(feature = "profile2d")]
    let terrain_layout = None;
    let texture =
        |h: u64| crate::world::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    // 3D water, with its wave heights when the entity has them.
    for (e, v) in kept_rows(world, layouts.water.as_ref()) {
        let Ok(row) = record::Water::from_json(&v) else {
            continue;
        };
        let entity = Entity::from_bits(e);
        let field = physics.and_then(|p| p.water_heights(entity));
        let wakes = physics.and_then(|p| p.water_wakes(entity)).map(|w| {
            Arc::new(WaterWakes {
                epoch: w.epoch,
                wave_speed: w.wave_speed,
                damping: w.damping,
                splash: w.splash,
                dt: w.dt,
                ticks: w
                    .ticks
                    .into_iter()
                    .map(|(tick, cuts)| {
                        let cuts = cuts
                            .into_iter()
                            .map(|c| WaterCut {
                                entity: c.entity,
                                min: c.min,
                                max: c.max,
                                centre: c.centre,
                                half: c.half,
                                axis: c.axis,
                                depth: c.depth,
                                speed: c.speed,
                            })
                            .collect();
                        (tick, cuts)
                    })
                    .collect(),
            })
        });
        let m = h.world_of(e);
        // og.Water.geometry: drawn over that mesh, in the entity's frame (scale included).
        let raw = row.geometry;
        let mesh = (raw != 0)
            .then(|| geometries.get((raw & 0xffff_ffff) as usize).copied())
            .flatten();
        let transform = if mesh.is_some() {
            m
        } else if field.is_some() {
            // The wave field is centred on the entity and unrotated.
            Mat4::from_translation(m.transform_point3(Vec3::ZERO))
        } else {
            let (_, r, t) = m.to_scale_rotation_translation();
            Mat4::from_rotation_translation(r, t)
        };
        scene.world.water.push(Water {
            transform,
            size: row.size,
            segments: row.segments,
            shallow: rgb(&row.shallow),
            deep: rgb(&row.deep),
            depth_fade: row.depth_fade,
            absorption: rgb(&row.absorption),
            scattering: rgb(&row.scattering),
            opacity: row.opacity,
            foam: rgb(&row.foam),
            foam_width: row.foam_width,
            wave_scale: row.wave_scale,
            wave_speed: row.wave_speed,
            wave_strength: row.wave_strength,
            refraction: row.refraction,
            reflectivity: row.reflectivity,
            heights: field.map(|w| WaterHeights {
                resolution: w.resolution,
                size: w.size,
                heights: Arc::from(w.heights),
            }),
            wakes,
            planar_reflection: row.planar_reflection,
            shadows: row.shadows,
            f0: row.f0,
            reflection_bias: row.reflection_bias,
            underwater_color: rgb(&row.underwater_color),
            underwater_density: row.underwater_density,
            caustics: row.caustics,
            caustics_scale: row.caustics_scale,
            mesh,
            normal_map: texture(row.normal_map),
            normal_scale: row.normal_scale.max(0.0),
            flow: row.flow,
            glint: row.glint.max(0.0),
            flow_map: texture(row.flow_map),
            wind: row.wind,
            swell: row.swell,
            choppiness: row.choppiness,
            foam_streaks: row.foam_streaks.max(0.0),
            spray: row.spray.max(0.0),
            dynamic_waves: row.dynamic_waves.max(0.0),
            meniscus: row.meniscus,
            endless: row.endless,
            shore: seabed(
                world,
                terrain,
                terrain_layout,
                entity,
                &v,
                &row,
                mesh.is_some(),
            ),
            river: river_of(engine, &v),
        });
    }
    // 2D water over its spring line.
    let lines: IdMap<u64, record::WaterLine> = IdMap::from_iter(
        kept_rows(world, layouts.water_line.as_ref())
            .into_iter()
            .filter_map(|(e, v)| Some((e, record::WaterLine::from_json(&v).ok()?))),
    );
    for (e, v) in kept_rows(world, layouts.water_2d.as_ref()) {
        let (Some(line), Some(p)) = (lines.get(&e), physics) else {
            continue;
        };
        let Ok(v) = record::Water2D::from_json(&v) else {
            continue;
        };
        let at = h.world_of(e).transform_point3(Vec3::ZERO);
        let width = line.width;
        let surface = p
            .water_line(Entity::from_bits(e))
            .unwrap_or_else(|| vec![0.0; line.columns.max(2) as usize]);
        scene.sprite.water_2d.push(Water2d {
            origin: Vec2::new(at.x - width * 0.5, at.y),
            width,
            depth: line.depth,
            surface,
            top: rgb(&v.top).extend(v.top_opacity),
            bottom: rgb(&v.bottom).extend(v.bottom_opacity),
            lip: rgb(&v.lip).extend(0.9),
            lip_width: v.lip_width,
            layer: v.layer,
            z: at.z,
        });
    }
}

/// og.Water.seabed's coast for the frame (W1): what the tick floats bodies
/// on (`crate::bind::seabed::coast`, read the same way from the same components,
/// at the entity's world position as og-physics places the water), in the
/// frame's coordinates before the render origin moves them.
fn seabed(
    world: &World,
    terrains: &crate::bind::terrain::TerrainState,
    terrain_layout: Option<&crate::host::codec::Layout>,
    entity: Entity,
    v: &Value,
    row: &record::Water,
    over_mesh: bool,
) -> Option<og_render::WaterShore> {
    if over_mesh {
        return None;
    }
    let r = crate::bind::physics_ext::Reader::World(world);
    let (position, _, _) = r.pose(entity)?;
    let sea = og_env::ocean::Ocean::new(row.wind, row.swell, row.choppiness);
    let (field, trains) =
        crate::bind::seabed::coast(terrains, terrain_layout, &r, v, position, sea.as_ref())?;
    Some(og_render::WaterShore {
        origin: field.origin,
        field,
        trains,
        foam: row.shore_foam,
    })
}
