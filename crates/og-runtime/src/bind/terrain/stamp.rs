//! Terrain stamps (T3): a mesh standing in for the ground where a height
//! field cannot reach (a cliff's face and lip, an arch, an overhang).
//! og.TerrainStamp draws it with its terrain (`super::extract`);
//! `og.terrain.stamp` makes the ground give way to it.
//!
//! The carve is gameplay state, so it is deterministic and replayable:
//! the mesh's triangles are taken into the terrain's own space and
//! rasterised from above over the height samples (carve) or the cells'
//! centres (hole), each point keeping the lowest triangle over it (its
//! underside). What the stamp changes is logged as that patch (a rectangle
//! of the grid and a value per point), not as the mesh: a restore replays
//! it exactly with no mesh loaded, and a stamp's later removal replays the
//! log without it.

use super::{Engine, edit_loaded, index_of, log_of, placement, replay_if_restored, terrain_id};
use crate::host::wire::{TerrainStampDesc, TerrainStampMode};
use glam::{Mat4, Vec2, Vec3};
use og_render::TerrainData;
use serde_json::{Value, json};
use std::sync::Arc;

/// TerrainStampMode's variants, in the IDL's order.
const MODES: [&str; 2] = ["carve", "hole"];

pub(crate) use og_render::terrain_stamp::Grid;

/// What a stamp changes: the points of the rectangle `[column, row,
/// width, height]` of the grid's samples (carve) or cells (hole), row by
/// row. Carve: each point's height is lowered to its value (NaN: the mesh
/// does not cover it). Hole: a point is cut where its value is 1.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Patch {
    pub hole: bool,
    pub rect: [u32; 4],
    pub values: Vec<f32>,
}

impl Patch {
    /// The points it changes.
    fn covered(&self) -> u32 {
        if self.hole {
            self.values.iter().filter(|v| **v == 1.0).count() as u32
        } else {
            self.values.iter().filter(|v| !v.is_nan()).count() as u32
        }
    }

    fn to_json(&self, stamp: u32) -> Value {
        let bytes: Vec<u8> = if self.hole {
            self.values.iter().map(|v| u8::from(*v == 1.0)).collect()
        } else {
            self.values.iter().flat_map(|v| v.to_le_bytes()).collect()
        };
        json!({
            "stamp": stamp,
            "mode": MODES[usize::from(self.hole)],
            "rect": self.rect,
            "patch": og_core::base64::encode(&bytes),
        })
    }

    fn from_json(v: &Value) -> Result<Patch, String> {
        let bad = || "a damaged terrain stamp in the edit log".to_string();
        let hole = match v["mode"].as_str() {
            Some("carve") => false,
            Some("hole") => true,
            _ => return Err(bad()),
        };
        let r: Vec<u32> = v["rect"]
            .as_array()
            .ok_or_else(bad)?
            .iter()
            .map(|n| n.as_u64().map(|n| n as u32).ok_or_else(bad))
            .collect::<Result<_, _>>()?;
        let rect: [u32; 4] = r.try_into().map_err(|_| bad())?;
        let bytes =
            og_core::base64::decode(v["patch"].as_str().ok_or_else(bad)?).ok_or_else(bad)?;
        let n = (rect[2] as usize) * (rect[3] as usize);
        let values: Vec<f32> = if hole {
            bytes.iter().map(|b| f32::from(*b)).collect()
        } else {
            let (words, rest) = bytes.as_chunks::<4>();
            if !rest.is_empty() {
                return Err(bad());
            }
            words.iter().map(|b| f32::from_le_bytes(*b)).collect()
        };
        if values.len() != n {
            return Err(bad());
        }
        Ok(Patch { hole, rect, values })
    }

    /// Applies it to a terrain's field; answers the points it changed.
    pub(crate) fn apply(&self, d: &mut TerrainData) -> Result<u32, String> {
        let [c0, r0, w, h] = self.rect;
        let (cols, rows) = if self.hole {
            d.cells()
        } else {
            (d.columns, d.rows)
        };
        if c0 + w > cols || r0 + h > rows {
            return Err(format!(
                "a terrain stamp over {w} × {h} points at ({c0}, {r0}) does not fit the terrain's {cols} × {rows}"
            ));
        }
        let at = |i: usize| ((r0 + i as u32 / w) * cols + c0 + i as u32 % w) as usize;
        if self.hole {
            let mut holes = d
                .holes
                .as_ref()
                .map_or_else(|| vec![0u8; (cols * rows) as usize], |h| h.to_vec());
            for (i, v) in self.values.iter().enumerate() {
                if *v == 1.0 {
                    holes[at(i)] = 1;
                }
            }
            d.holes = holes.contains(&1).then(|| holes.into());
        } else {
            let heights = Arc::make_mut(&mut d.heights);
            for (i, v) in self.values.iter().enumerate() {
                if !v.is_nan() {
                    let k = at(i);
                    heights[k] = heights[k].min(*v);
                }
            }
        }
        Ok(self.covered())
    }

    /// The rectangle of a terrain's space (`d`'s) it changes.
    fn region(&self, d: &TerrainData) -> (Vec2, Vec2) {
        let [c0, r0, w, h] = self.rect;
        let at = |c: u32, r: u32| Vec2::new(c as f32, r as f32) * d.cell() - d.size * 0.5;
        (at(c0, r0), at(c0 + w, r0 + h))
    }
}

/// The patch a stamp makes: the carve's heights `depth` under the
/// underside, or the cells it covers.
///
/// A carve goes `depth` under the mesh only at the samples whose four
/// neighbours the mesh covers too; a sample on the footprint's edge goes
/// down to the underside itself, no further, so the ground beside the
/// mesh meets it at the mesh's own foot (lowered by `depth` too, the edge
/// tilted the first cell outside the mesh into a trench that showed as a
/// dark line along its foot), and the step down to the carve lies under
/// the mesh.
pub(crate) fn patch(grid: Grid, triangles: &[[Vec3; 3]], hole: bool, depth: f32) -> Option<Patch> {
    let (rect, values) = og_render::terrain_stamp::patch(grid, triangles, hole, depth)?;
    Some(Patch { hole, rect, values })
}

/// A logged edit's stamp id, if it is a stamp's (or a removed one's).
pub(crate) fn id_of(entry: &str) -> Option<u32> {
    if !entry.contains("\"stamp\"") {
        return None;
    }
    let v: Value = serde_json::from_str(entry).ok()?;
    v["stamp"].as_u64().map(|n| n as u32)
}

/// Whether a logged stamp was taken back (og.terrain.unstamp).
pub(crate) fn removed(v: &Value) -> bool {
    v["removed"].as_bool() == Some(true)
}

/// Applies a logged stamp to a terrain's field (a removed one: nothing);
/// answers the points it changed and the rectangle of the terrain's space
/// they are in.
pub(crate) fn apply(
    d: &mut TerrainData,
    entry: &Value,
) -> Result<(u32, Option<(Vec2, Vec2)>), String> {
    if removed(entry) {
        return Ok((0, None));
    }
    let p = Patch::from_json(entry)?;
    Ok((p.apply(d)?, Some(p.region(d))))
}

/// `og.terrain.stamp`'s mesh in the terrain's own space: its triangles,
/// placed by the stamp's transform, then into the terrain's.
fn triangles(
    eng: &Engine,
    desc: &TerrainStampDesc,
    given: &Value,
) -> Result<Vec<[Vec3; 3]>, String> {
    let geometry = desc.geometry;
    let rs = eng.render.borrow();
    let mesh = crate::world::assets::index_of(geometry as f64)
        .and_then(|i| rs.geometries.get(i).copied())
        .and_then(|m| rs.scene.mesh(m))
        .ok_or_else(|| {
            format!("TerrainStampDesc.geometry {geometry} is not a loaded geometry with its mesh in memory")
        })?;
    let mesh = crate::bind::modules::mesh_of(mesh);
    let terrain = placement(&given["origin"], &given["quaternion"], &given["scale"]);
    let stamp = placement(
        &given["meshPosition"],
        &given["meshQuaternion"],
        &given["meshScale"],
    );
    let m: Mat4 = terrain.inverse() * stamp;
    let p: Vec<Vec3> = mesh
        .positions
        .iter()
        .map(|v| m.transform_point3(*v))
        .collect();
    Ok(mesh
        .indices
        .iter()
        .filter_map(|t| {
            Some([
                *p.get(t[0] as usize)?,
                *p.get(t[1] as usize)?,
                *p.get(t[2] as usize)?,
            ])
        })
        .collect())
}

/// `og.terrain.stamp` of `desc`, read from `given` (the desc as passed).
pub(crate) fn stamp(eng: &Engine, desc: &TerrainStampDesc, given: &Value) -> Result<Value, String> {
    replay_if_restored(eng)?;
    let i = index_of(eng, desc.data as f64)?;
    let hole = desc.mode == TerrainStampMode::Hole;
    // The depth is the terrain's own metres, as its heights are.
    let up = placement(&given["origin"], &given["quaternion"], &given["scale"])
        .y_axis
        .truncate()
        .length()
        .max(1e-6);
    // A null depth (a NaN the game passed) is the default, as it was.
    let depth = if desc.depth.is_nan() {
        0.25
    } else {
        desc.depth
    };
    if !(depth >= 0.0 && depth.is_finite()) {
        return Err("TerrainStampDesc.depth must be at least 0".into());
    }
    let triangles = triangles(eng, desc, given)?;
    let grid = {
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        t.field.as_ref().map_or(
            Grid {
                size: t.layout.size,
                columns: t.layout.columns,
                rows: t.layout.rows,
            },
            Grid::of,
        )
    };
    let patch = patch(grid, &triangles, hole, depth / up).ok_or_else(|| {
        format!(
            "the stamp's mesh ({} triangles) covers none of the terrain's {}: seen from above, it misses the terrain",
            triangles.len(),
            if hole { "cells" } else { "samples" }
        )
    })?;
    let next = {
        let log = log_of(eng);
        let log = log.lock().expect("the edit log");
        log.edits
            .iter()
            .filter_map(|(_, e)| id_of(e))
            .max()
            .unwrap_or(0)
            + 1
    };
    edit_loaded(eng, i, &patch.to_json(next), true)
}

pub(crate) fn unstamp(eng: &Engine, data: f64, stamp: u32) -> Result<Value, String> {
    replay_if_restored(eng)?;
    let i = index_of(eng, data)?;
    let id = terrain_id(&eng.render.borrow().terrain, i);
    let samples = {
        let shared = log_of(eng);
        let mut log = shared.lock().expect("the edit log");
        let k = log
            .edits
            .iter()
            .position(|(t, e)| {
                *t == id
                    && id_of(e) == Some(stamp)
                    && serde_json::from_str::<Value>(e).is_ok_and(|v| !removed(&v))
            })
            .ok_or_else(|| format!("the terrain holds no stamp {stamp}"))?;
        let entry: Value = serde_json::from_str(&log.edits[k].1).map_err(|e| e.to_string())?;
        let samples = Patch::from_json(&entry)?.covered();
        log.edits[k].1 = json!({ "stamp": stamp, "removed": true }).to_string();
        log.replay = true;
        samples
    };
    replay_if_restored(eng)?;
    let (lo, hi) = eng.render.borrow().terrain.loaded[i].range();
    Ok(json!({ "samples": samples, "minHeight": lo, "maxHeight": hi, "stamp": stamp }))
}
