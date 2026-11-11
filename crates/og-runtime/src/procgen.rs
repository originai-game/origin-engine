//! `og.procgen`: og-core's generators over the wire. Each is a pure
//! function of its desc (the seed in it included), so it needs no world.

use super::refuse;
use super::wire::{self, ToJson};
use og_core::glam::Vec2;
use og_core::procgen::{self, BspDesc, Fbm, NoiseKind, Wfc, WfcTile};
use serde_json::json;

fn wire<T: ToJson>(r: Result<T, String>) -> String {
    match r {
        Ok(v) => wire::ok(&v),
        Err(e) => refuse(e),
    }
}

/// An FbmDesc read off the wire as og-core's generator.
fn fbm(d: &wire::FbmDesc) -> Fbm {
    Fbm {
        seed: d.seed,
        kind: match d.kind {
            wire::NoiseKind::Perlin => NoiseKind::Perlin,
            wire::NoiseKind::Simplex => NoiseKind::Simplex,
        },
        octaves: d.octaves.clamp(1, 16),
        frequency: d.frequency,
        lacunarity: d.lacunarity,
        gain: d.gain,
    }
}

pub fn og_procgen_noise(desc: String, x: f64, y: f64, z: f64) -> f64 {
    match wire::arg::<wire::FbmDesc>(Some(&desc), "desc") {
        Ok(d) => f64::from(fbm(&d).at(x as f32, y as f32, z as f32, !d.three_d)),
        Err(_) => f64::NAN,
    }
}

pub fn og_procgen_noise_grid(desc: Option<String>) -> String {
    wire((|| {
        let d: wire::NoiseGridDesc = wire::arg(desc.as_deref(), "desc")?;
        let (w, h) = (d.width, d.height);
        if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1 << 24 {
            return Err(format!(
                "desc.width x desc.height is {w} x {h}: 1 to 16,777,216 samples"
            ));
        }
        let grid = fbm(&d.noise).grid(w, h, d.x, d.y, d.step, d.z, !d.noise.three_d);
        Ok(json!(grid))
    })())
}

pub fn og_procgen_poisson_disk(desc: Option<String>) -> String {
    wire((|| {
        let d: wire::PoissonDesc = wire::arg(desc.as_deref(), "desc")?;
        let pts = procgen::poisson_disk(
            d.seed,
            d.width,
            d.height,
            d.radius,
            d.attempts.clamp(1, 256),
            d.max_points.min(1 << 20) as usize,
        )
        .map_err(|e| e.reason().to_string())?;
        Ok(pts.into_iter().map(Vec2::from_array).collect::<Vec<_>>())
    })())
}

fn rect(r: &procgen::CellRect) -> wire::CellRect {
    wire::CellRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

pub fn og_procgen_bsp(desc: Option<String>) -> String {
    wire((|| {
        let d: wire::BspDesc = wire::arg(desc.as_deref(), "desc")?;
        let out = procgen::bsp_rooms(&BspDesc {
            seed: d.seed,
            width: d.width,
            height: d.height,
            min_room: d.min_room,
            max_room: d.max_room,
            depth: d.depth,
            padding: d.padding,
            corridor: d.corridor,
        })
        .map_err(|e| e.reason().to_string())?;
        Ok(wire::BspResult {
            rooms: out.rooms.iter().map(rect).collect(),
            corridors: out.corridors.iter().map(rect).collect(),
            cells: out.cells.into_iter().map(u32::from).collect(),
        })
    })())
}

pub fn og_procgen_wfc(desc: Option<String>) -> String {
    wire((|| {
        let d: wire::WfcDesc = wire::arg(desc.as_deref(), "desc")?;
        let tiles = d
            .tiles
            .iter()
            .map(|t| WfcTile {
                weight: t.weight,
                sockets: [t.up, t.right, t.down, t.left],
                rotations: t.rotations,
            })
            .collect();
        let fixed = d
            .fixed
            .iter()
            .map(|c| (c.x, c.y, c.tile, c.rotation))
            .collect();
        let out = procgen::wfc(&Wfc {
            seed: d.seed,
            width: d.width,
            height: d.height,
            tiles,
            fixed,
            wrap: d.wrap,
            attempts: d.attempts.clamp(1, 1000),
        })
        .map_err(|e| e.reason().to_string())?;
        Ok(wire::WfcResult {
            tiles: out.tiles,
            rotations: out.rotations,
            attempt: out.attempt,
        })
    })())
}
