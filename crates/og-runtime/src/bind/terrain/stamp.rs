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
use glam::{DVec2, Mat4, Vec2, Vec3};
use og_render::TerrainData;
use serde_json::{Value, json};
use std::sync::Arc;

/// TerrainStampMode's variants, in the IDL's order.
const MODES: [&str; 2] = ["carve", "hole"];

/// A terrain's grid as a stamp rasterises it: its extent and samples.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Grid {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
}

impl Grid {
    fn of(d: &TerrainData) -> Grid {
        Grid {
            size: d.size,
            columns: d.columns,
            rows: d.rows,
        }
    }

    fn cell(&self) -> Vec2 {
        TerrainData::flat(self.size, self.columns.max(2), self.rows.max(2)).cell()
    }
}

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

/// The lowest of `triangles` (in the terrain's own space) over each point
/// of the grid's samples (`cells` false) or cells' centres (`cells`
/// true), seen straight down its y: the rectangle of the points any
/// triangle covers and, row by row, each one's lowest height (infinite
/// where none does). Triangles seen edge-on cover nothing; a point on an
/// edge is covered by both triangles sharing it. Barycentrics are
/// reckoned in f64, so a point covered once is covered on every host.
pub(crate) fn underside(
    grid: Grid,
    triangles: &[[Vec3; 3]],
    cells: bool,
) -> Option<([u32; 4], Vec<f32>)> {
    let (cols, rows) = if cells {
        (grid.columns.max(1) - 1, grid.rows.max(1) - 1)
    } else {
        (grid.columns, grid.rows)
    };
    if cols == 0 || rows == 0 {
        return None;
    }
    let cell = grid.cell().as_dvec2();
    let half = grid.size.as_dvec2() * 0.5;
    let shift = if cells { 0.5 } else { 0.0 };
    // Grid coordinates of a point in the terrain's space, and back.
    let to_grid = |p: DVec2| (p + half) / cell - shift;
    let to_local = |c: f64, r: f64| (DVec2::new(c, r) + shift) * cell - half;
    let mut lowest = vec![f32::INFINITY; (cols * rows) as usize];
    let (mut lo, mut hi) = ([u32::MAX; 2], [0u32; 2]);
    for t in triangles {
        let p = t.map(|v| DVec2::new(v.x as f64, v.z as f64));
        let area = (p[1] - p[0]).perp_dot(p[2] - p[0]);
        if area.is_nan() || area.abs() <= 1e-12 {
            continue;
        }
        let g = p.map(to_grid);
        let min = g[0].min(g[1]).min(g[2]);
        let max = g[0].max(g[1]).max(g[2]);
        let c0 = min.x.ceil().max(0.0);
        let r0 = min.y.ceil().max(0.0);
        let c1 = max.x.floor().min(cols as f64 - 1.0);
        let r1 = max.y.floor().min(rows as f64 - 1.0);
        if c0 > c1 || r0 > r1 {
            continue;
        }
        let eps = -1e-9 * area.abs();
        for r in r0 as u32..=r1 as u32 {
            for c in c0 as u32..=c1 as u32 {
                let q = to_local(c as f64, r as f64);
                let w0 = (p[2] - p[1]).perp_dot(q - p[1]) * area.signum();
                let w1 = (p[0] - p[2]).perp_dot(q - p[2]) * area.signum();
                let w2 = (p[1] - p[0]).perp_dot(q - p[0]) * area.signum();
                if w0 < eps || w1 < eps || w2 < eps {
                    continue;
                }
                let y = (w0 * t[0].y as f64 + w1 * t[1].y as f64 + w2 * t[2].y as f64) / area.abs();
                let k = (r * cols + c) as usize;
                lowest[k] = lowest[k].min(y as f32);
                lo = [lo[0].min(c), lo[1].min(r)];
                hi = [hi[0].max(c), hi[1].max(r)];
            }
        }
    }
    if lo[0] > hi[0] {
        return None;
    }
    let (w, h) = (hi[0] - lo[0] + 1, hi[1] - lo[1] + 1);
    let mut out = Vec::with_capacity((w * h) as usize);
    for r in lo[1]..=hi[1] {
        let row = (r * cols) as usize;
        out.extend_from_slice(&lowest[row + lo[0] as usize..=row + hi[0] as usize]);
    }
    Some(([lo[0], lo[1], w, h], out))
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
    let (rect, lowest) = underside(grid, triangles, hole)?;
    let [_, _, w, h] = rect;
    let covered = |c: i64, r: i64| {
        c >= 0
            && r >= 0
            && c < w as i64
            && r < h as i64
            && lowest[(r * w as i64 + c) as usize].is_finite()
    };
    let values = lowest
        .iter()
        .enumerate()
        .map(|(k, y)| {
            let (c, r) = ((k as u32 % w) as i64, (k as u32 / w) as i64);
            let inner =
                covered(c - 1, r) && covered(c + 1, r) && covered(c, r - 1) && covered(c, r + 1);
            match (y.is_finite(), hole) {
                (false, true) => 0.0,
                (true, true) => 1.0,
                (true, false) if inner => y - depth,
                (true, false) => *y,
                (false, false) => f32::NAN,
            }
        })
        .collect();
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
                size: t.cooked.layout.size,
                columns: t.cooked.layout.columns,
                rows: t.cooked.layout.rows,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A box from `min` to `max`, its twelve triangles.
    fn cuboid(min: Vec3, max: Vec3) -> Vec<[Vec3; 3]> {
        let v = |x: bool, y: bool, z: bool| {
            Vec3::new(
                if x { max.x } else { min.x },
                if y { max.y } else { min.y },
                if z { max.z } else { min.z },
            )
        };
        let quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| [[a, b, c], [a, c, d]];
        let (f, t) = (false, true);
        [
            quad(v(f, f, f), v(t, f, f), v(t, f, t), v(f, f, t)),
            quad(v(f, t, f), v(f, t, t), v(t, t, t), v(t, t, f)),
            quad(v(f, f, f), v(f, t, f), v(t, t, f), v(t, f, f)),
            quad(v(f, f, t), v(t, f, t), v(t, t, t), v(f, t, t)),
            quad(v(f, f, f), v(f, f, t), v(f, t, t), v(f, t, f)),
            quad(v(t, f, f), v(t, t, f), v(t, t, t), v(t, f, t)),
        ]
        .concat()
    }

    /// A 300 m cliff on a 0.5 m grid: a plateau at 300 m for x < 0 and a
    /// valley at 0 beyond, one cell apart (a height field's sawtooth wall).
    fn cliff() -> TerrainData {
        let (n, size) = (129u32, 64.0f32);
        let mut d = TerrainData::flat(Vec2::splat(size), n, n);
        let cell = d.cell().x;
        d.heights = (0..n * n)
            .map(|k| {
                let x = (k % n) as f32 * cell - size * 0.5;
                if x < 0.0 { 300.0 } else { 0.0 }
            })
            .collect::<Vec<_>>()
            .into();
        d
    }

    /// The steepest slope, in degrees, between neighbouring samples whose
    /// four corners all lie in the rectangle `[x0, x1] × [z0, z1]`.
    fn steepest(d: &TerrainData, x0: f32, x1: f32, z0: f32, z1: f32) -> f32 {
        let cell = d.cell();
        let half = d.size * 0.5;
        let h = |c: u32, r: u32| d.heights[(r * d.columns + c) as usize];
        let mut worst = 0.0f32;
        for r in 0..d.rows - 1 {
            for c in 0..d.columns - 1 {
                let p = Vec2::new(c as f32, r as f32) * cell - half;
                let q = p + cell;
                if p.x < x0 || q.x > x1 || p.y < z0 || q.y > z1 {
                    continue;
                }
                let dx = (h(c + 1, r) - h(c, r)).abs() / cell.x;
                let dz = (h(c, r + 1) - h(c, r)).abs() / cell.y;
                worst = worst.max(og_core::math::atan(dx.max(dz)).to_degrees());
            }
        }
        worst
    }

    #[test]
    fn a_carved_cliff_leaves_no_wall_under_its_mesh() {
        // The cliff's rock: a block over the step, its foot at the valley
        // floor, its top 2 m above the plateau.
        let mut d = cliff();
        let rock = cuboid(Vec3::new(-6.0, 0.0, -10.0), Vec3::new(6.0, 302.0, 10.0));
        let before = steepest(&d, -6.0, 6.0, -10.0, 10.0);
        assert!(before > 89.0, "the wall before the carve: {before}°");
        let p = patch(Grid::of(&d), &rock, false, 0.25).expect("the rock covers the terrain");
        assert_eq!(
            p.rect,
            [52, 44, 25, 41],
            "the samples under x, z in [-6, 6] × [-10, 10]"
        );
        let touched = p.apply(&mut d).unwrap();
        assert_eq!(touched, 25 * 41);
        // Inside the edge's cells (hidden under the mesh) no wall is left.
        let after = steepest(&d, -5.5, 5.5, -9.5, 9.5);
        assert!(
            after < 45.0,
            "the ground under the rock after the carve: {after}°"
        );
        // Under the rock it is the rock's underside less the depth; on its
        // edge the underside itself, so the valley beside the face is flat
        // up to it; beside it the cliff is untouched.
        let at = |c: u32, r: u32| d.heights[(r * d.columns + c) as usize];
        assert_eq!(at(60, 64), -0.25);
        assert_eq!(at(76, 64), 0.0, "the face's edge");
        assert_eq!(at(77, 64), 0.0);
        assert_eq!(at(40, 64), 300.0);
        assert_eq!(at(64, 30), 0.0);
    }

    #[test]
    fn a_carve_follows_the_meshs_underside_and_spares_lower_ground() {
        // An overhang: a slab whose underside rises from 2 m to 10 m along
        // x, over flat ground at 5 m. Under its low end the ground goes
        // down; under its high end it is already below and stays.
        let mut d = TerrainData::flat(Vec2::splat(16.0), 17, 17);
        d.heights = vec![5.0; 17 * 17].into();
        let (a, b, c, e) = (
            Vec3::new(-4.0, 2.0, -4.0),
            Vec3::new(4.0, 10.0, -4.0),
            Vec3::new(4.0, 10.0, 4.0),
            Vec3::new(-4.0, 2.0, 4.0),
        );
        let slab = [[a, b, c], [a, c, e]];
        patch(Grid::of(&d), &slab, false, 0.0)
            .unwrap()
            .apply(&mut d)
            .unwrap();
        let at = |x: i32| d.heights[(8 * 17 + (x + 8)) as usize];
        let near = |a: f32, b: f32| (a - b).abs() < 1e-5;
        assert!(near(at(-4), 2.0), "{}", at(-4));
        assert!(near(at(-2), 4.0), "{}", at(-2));
        assert_eq!(at(0), 5.0, "the underside is at 6 m, above the ground");
        assert_eq!(at(4), 5.0);
        assert_eq!(at(-5), 5.0, "beside the slab");
    }

    #[test]
    fn a_hole_stamp_cuts_the_cells_its_mesh_covers() {
        let mut d = TerrainData::flat(Vec2::splat(16.0), 17, 17);
        let mouth = cuboid(Vec3::new(-2.0, -1.0, -1.0), Vec3::new(2.0, 3.0, 1.0));
        let p = patch(Grid::of(&d), &mouth, true, 0.0).unwrap();
        // Cell centres at -1.5, -0.5, 0.5, 1.5 along x and -0.5, 0.5 along z.
        assert_eq!(p.rect, [6, 7, 4, 2]);
        assert_eq!(p.apply(&mut d).unwrap(), 8);
        assert!(d.hole(Vec2::new(1.5, 0.5)));
        assert!(!d.hole(Vec2::new(2.5, 0.5)));
        assert!(!d.hole(Vec2::new(0.5, 1.5)));
    }

    #[test]
    fn a_logged_stamp_replays_bit_for_bit() {
        let d0 = cliff();
        let rock = cuboid(Vec3::new(-6.3, 0.1, -10.2), Vec3::new(6.1, 302.0, 9.7));
        let p = patch(Grid::of(&d0), &rock, false, 0.25).unwrap();
        let logged = p.to_json(3).to_string();
        let back: Value = serde_json::from_str(&logged).unwrap();
        assert_eq!(id_of(&logged), Some(3));
        let (mut a, mut b) = (d0.clone(), d0.clone());
        p.apply(&mut a).unwrap();
        apply(&mut b, &back).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, d0);
        // A removed stamp replays as nothing.
        let mut c = d0.clone();
        apply(&mut c, &json!({ "stamp": 3, "removed": true })).unwrap();
        assert_eq!(c, d0);
    }

    #[test]
    fn a_mesh_seen_edge_on_or_off_the_terrain_covers_nothing() {
        let d = TerrainData::flat(Vec2::splat(16.0), 17, 17);
        let wall = [[
            Vec3::new(-4.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(4.0, 9.0, 0.0),
        ]];
        assert_eq!(patch(Grid::of(&d), &wall, false, 0.0), None);
        let away = cuboid(Vec3::new(20.0, 0.0, 20.0), Vec3::new(24.0, 1.0, 24.0));
        assert_eq!(patch(Grid::of(&d), &away, false, 0.0), None);
    }
}
