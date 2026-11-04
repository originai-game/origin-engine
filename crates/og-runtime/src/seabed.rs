//! og.Water.seabed (W1): the sea's floor from an og.Terrain, as
//! `og_core::shore::Shore` (its depth under the water's rest level and the
//! distance to the waterline), the trains rolling in on it, for both the
//! tick (og-physics' `WaterBody::shore`, which og.Buoyancy and
//! og.physics.waterHeight read) and the frame (og-render's `Water::shore`).
//! Both read it through [`coast`], from the same components the same way,
//! so a floater rides the sea that is drawn. Derived state: it is made from
//! the terrain (cooked, and its edits, which are world state) and the
//! water's components, so the world hash already covers what it is; it is
//! cached by what made it and rebuilt only when that changes.

use super::codec::Layout;
use super::physics_ext::Reader;
use super::terrain::{SeabedGrid, TerrainState};
use glam::{Mat4, Vec2, Vec3};
use og_core::ocean::{Ocean, Shore, ShoreTrains};
use og_ecs::Entity;
use serde_json::Value;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// The last fields made, by their key and rest level, newest last.
    static CACHE: RefCell<Vec<(u64, f32, Arc<Shore>)>> = const { RefCell::new(Vec::new()) };
}

/// How many fields are kept (a world rarely has more than one sea).
const KEEP: usize = 4;

/// The coast of the og.Water `v` at `position` (its rest level's world
/// point) over its sea `sea`: its seabed (og.Water.seabed's og.Terrain, of
/// the `terrains` loaded, its rows read with `r` by `terrain_layout`, placed
/// by that entity's world pose and its own scale) and the trains
/// (og.Water.shore, shoreWaveLength). `None` without a seabed, a sea, or a
/// terrain under the water.
pub(crate) fn coast(
    terrains: &TerrainState,
    terrain_layout: Option<&Layout>,
    r: &Reader<'_>,
    v: &Value,
    position: Vec3,
    sea: Option<&Ocean>,
) -> Option<(Arc<Shore>, ShoreTrains)> {
    let sea = sea?;
    let raw = v["seabed"].as_f64().unwrap_or(0.0);
    if raw <= 0.0 {
        return None;
    }
    let e = Entity::from_bits(raw as u64);
    let rows = r.rows(terrain_layout).ok()?;
    let (_, tv) = rows.into_iter().find(|(id, _)| *id == e.to_bits())?;
    let index = super::assets::index_of(tv["data"].as_f64().unwrap_or(0.0))?;
    let (at, q, scale) = r.pose(e)?;
    let m = Mat4::from_scale_rotation_translation(scale, q, at);
    let size = super::fx::vec2(&v["size"], Vec2::splat(16.0));
    let centre = Vec2::new(position.x, position.z);
    let grid =
        super::terrain::seabed_grid(terrains, index, m, centre - size * 0.5, centre + size * 0.5)?;
    let shore = shore(terrains, &grid, position.y)?;
    let trains = ShoreTrains::new(
        sea.significant_height(),
        super::fx::f(v, "shore", 1.0),
        super::fx::f(v, "shoreWaveLength", 0.0),
    );
    Some((shore, trains))
}

/// The seabed's field under a rest level of `level`: cached, or made from
/// the terrain's heights.
fn shore(terrains: &TerrainState, grid: &SeabedGrid, level: f32) -> Option<Arc<Shore>> {
    let found = CACHE.with(|c| {
        c.borrow()
            .iter()
            .find(|(k, l, _)| *k == grid.key && l.to_bits() == level.to_bits())
            .map(|(_, _, s)| s.clone())
    });
    if let Some(s) = found {
        return Some(s);
    }
    let depth: Vec<f32> = grid.heights(terrains).iter().map(|h| level - h).collect();
    let made = Arc::new(Shore::new(
        grid.origin,
        grid.cell,
        grid.columns,
        grid.rows,
        &depth,
    )?);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        c.push((grid.key, level, made.clone()));
        if c.len() > KEEP {
            c.remove(0);
        }
    });
    Some(made)
}
