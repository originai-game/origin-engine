//! `og.procgen`: og-core's generators over the wire. Each is a pure
//! function of its desc (the seed in it included), so it needs no world.

use super::{ok, parse, refuse};
use og_core::procgen::{self, BspDesc, Fbm, NoiseKind, Wfc, WfcTile};
use serde_json::{Value, json};

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn u32_of(v: &Value, key: &str, default: u32) -> u32 {
    v[key]
        .as_u64()
        .map_or(default, |n| n.min(u64::from(u32::MAX)) as u32)
}

fn f32_of(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |n| n as f32)
}

/// An FbmDesc on the wire.
fn fbm(v: &Value) -> Result<(Fbm, bool), String> {
    let kind = match v["kind"].as_str() {
        None | Some("perlin") => NoiseKind::Perlin,
        Some("simplex") => NoiseKind::Simplex,
        Some(k) => return Err(format!("desc.kind '{k}' is not perlin or simplex")),
    };
    let f = Fbm {
        seed: u32_of(v, "seed", 0),
        kind,
        octaves: u32_of(v, "octaves", 1).clamp(1, 16),
        frequency: f32_of(v, "frequency", 1.0),
        lacunarity: f32_of(v, "lacunarity", 2.0),
        gain: f32_of(v, "gain", 0.5),
    };
    Ok((f, v["threeD"].as_bool().unwrap_or(false)))
}

pub fn og_procgen_noise(desc: String, x: f64, y: f64, z: f64) -> f64 {
    match parse(Some(&desc)).and_then(|v| fbm(&v)) {
        Ok((f, three)) => f64::from(f.at(x as f32, y as f32, z as f32, !three)),
        Err(_) => f64::NAN,
    }
}

pub fn og_procgen_noise_grid(desc: Option<String>) -> String {
    wire((|| {
        let v = parse(desc.as_deref())?;
        let (f, three) = fbm(&v["noise"])?;
        let (w, h) = (u32_of(&v, "width", 64), u32_of(&v, "height", 64));
        if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1 << 24 {
            return Err(format!(
                "desc.width x desc.height is {w} x {h}: 1 to 16,777,216 samples"
            ));
        }
        let grid = f.grid(
            w,
            h,
            f32_of(&v, "x", 0.0),
            f32_of(&v, "y", 0.0),
            f32_of(&v, "step", 1.0),
            f32_of(&v, "z", 0.0),
            !three,
        );
        Ok(json!(grid))
    })())
}

pub fn og_procgen_poisson_disk(desc: Option<String>) -> String {
    wire((|| {
        let v = parse(desc.as_deref())?;
        let pts = procgen::poisson_disk(
            u32_of(&v, "seed", 0),
            f32_of(&v, "width", 0.0),
            f32_of(&v, "height", 0.0),
            f32_of(&v, "radius", 1.0),
            u32_of(&v, "attempts", 30).clamp(1, 256),
            u32_of(&v, "maxPoints", 4096).min(1 << 20) as usize,
        )
        .map_err(|e| e.reason().to_string())?;
        Ok(json!(
            pts.iter()
                .map(|p| json!({ "x": p[0], "y": p[1] }))
                .collect::<Vec<_>>()
        ))
    })())
}

fn rect(r: &procgen::CellRect) -> Value {
    json!({ "x": r.x, "y": r.y, "width": r.width, "height": r.height })
}

pub fn og_procgen_bsp(desc: Option<String>) -> String {
    wire((|| {
        let v = parse(desc.as_deref())?;
        let d = BspDesc::default();
        let out = procgen::bsp_rooms(&BspDesc {
            seed: u32_of(&v, "seed", d.seed),
            width: u32_of(&v, "width", d.width),
            height: u32_of(&v, "height", d.height),
            min_room: u32_of(&v, "minRoom", d.min_room),
            max_room: u32_of(&v, "maxRoom", d.max_room),
            depth: u32_of(&v, "depth", d.depth),
            padding: u32_of(&v, "padding", d.padding),
            corridor: u32_of(&v, "corridor", d.corridor),
        })
        .map_err(|e| e.reason().to_string())?;
        Ok(json!({
            "rooms": out.rooms.iter().map(rect).collect::<Vec<_>>(),
            "corridors": out.corridors.iter().map(rect).collect::<Vec<_>>(),
            "cells": out.cells,
        }))
    })())
}

pub fn og_procgen_wfc(desc: Option<String>) -> String {
    wire((|| {
        let v = parse(desc.as_deref())?;
        let list = |key: &str| v[key].as_array().cloned().unwrap_or_default();
        let tiles = list("tiles")
            .iter()
            .map(|t| WfcTile {
                weight: f32_of(t, "weight", 1.0),
                sockets: [
                    u32_of(t, "up", 0),
                    u32_of(t, "right", 0),
                    u32_of(t, "down", 0),
                    u32_of(t, "left", 0),
                ],
                rotations: t["rotations"].as_bool().unwrap_or(false),
            })
            .collect();
        let fixed = list("fixed")
            .iter()
            .map(|c| {
                (
                    u32_of(c, "x", 0),
                    u32_of(c, "y", 0),
                    u32_of(c, "tile", 0),
                    u32_of(c, "rotation", 0),
                )
            })
            .collect();
        let out = procgen::wfc(&Wfc {
            seed: u32_of(&v, "seed", 0),
            width: u32_of(&v, "width", 16),
            height: u32_of(&v, "height", 16),
            tiles,
            fixed,
            wrap: v["wrap"].as_bool().unwrap_or(false),
            attempts: u32_of(&v, "attempts", 10).clamp(1, 1000),
        })
        .map_err(|e| e.reason().to_string())?;
        Ok(json!({ "tiles": out.tiles, "rotations": out.rotations, "attempt": out.attempt }))
    })())
}
