//! OGT `.ogt` parser for a terrain's `generate { ... }` block, delegating
//! heightfield, erosion, river and biome generation to
//! [`og_render_data::terrain::generate`].

use super::terrain_schema::{
    BIOME as BIOME_KEYS, EROSION as EROSION_KEYS, GENERATE as KEYS, MOISTURE as MOISTURE_KEYS,
    NOISE as NOISE_KEYS, RIVERS as RIVER_KEYS, TERRACES as TERRACE_KEYS, THERMAL as THERMAL_KEYS,
    check as check_keys,
};
use og_core::ogt::{Block, Entry};
pub use og_render_data::terrain::generate::{
    Biome, Channel, Generate, Hydraulic, Mask, Moisture, Rivers, Thermal, generate, process,
};

/// The block of `key` inside `b`, its keys checked against `keys`.
fn sub<'a>(
    b: &'a Block,
    key: &str,
    keys: &[crate::grammar::Line],
) -> Result<Option<&'a Block>, String> {
    let Some(e) = b.get(key) else {
        return Ok(None);
    };
    let Some(inner) = &e.block else {
        return Err(format!(
            "line {}: '{key}' is a block: {key} {{ ... }}",
            e.line
        ));
    };
    check_keys(inner, keys, key)?;
    Ok(Some(inner))
}

/// The numbers of `key` (its line in errors), or `None`.
fn values(b: &Block, key: &str) -> Result<Option<(Vec<f32>, usize)>, String> {
    let Some(e) = b.get(key) else {
        return Ok(None);
    };
    let v: Option<Vec<f32>> = e
        .values
        .iter()
        .map(|v| v.as_number().map(|n| n.as_f32()))
        .collect();
    match v {
        Some(v) if v.iter().all(|x| x.is_finite()) => Ok(Some((v, e.line))),
        _ => Err(format!("line {}: '{key}' takes numbers", e.line)),
    }
}

/// One number in `lo..=hi` (default `default`).
fn num(b: &Block, key: &str, default: f32, lo: f32, hi: f32) -> Result<f32, String> {
    match values(b, key)? {
        None => Ok(default),
        Some((v, line)) => {
            if v.len() != 1 {
                return Err(format!("line {line}: '{key}' takes one number"));
            }
            if !(lo..=hi).contains(&v[0]) {
                return Err(format!(
                    "line {line}: '{key}' is from {lo} to {hi}, got {}",
                    v[0]
                ));
            }
            Ok(v[0])
        }
    }
}

/// One whole number in `lo..=hi`, parsed in `f64` so full 32-bit seeds
/// (`> 2^24`) never lose precision.
fn whole(b: &Block, key: &str, default: u32, lo: u32, hi: u32) -> Result<u32, String> {
    let Some(e) = b.get(key) else {
        return Ok(default);
    };
    if e.values.len() != 1 {
        return Err(format!("line {}: '{key}' takes one number", e.line));
    }
    let Some(v) = e.values[0].as_number().map(|n| n.as_f64()) else {
        return Err(format!("line {}: '{key}' takes numbers", e.line));
    };
    if !v.is_finite() || v.fract() != 0.0 {
        return Err(format!(
            "line {}: '{key}' is a whole number, got {v}",
            e.line
        ));
    }
    if v < f64::from(lo) || v > f64::from(hi) {
        return Err(format!(
            "line {}: '{key}' is from {lo} to {hi}, got {v}",
            e.line
        ));
    }
    Ok(v as u32)
}

/// Two numbers, the first no larger than the second.
fn range(b: &Block, key: &str, default: [f32; 2]) -> Result<[f32; 2], String> {
    match values(b, key)? {
        None => Ok(default),
        Some((v, _)) if v.len() == 2 && v[0] <= v[1] => Ok([v[0], v[1]]),
        Some((_, line)) => Err(format!(
            "line {line}: '{key}' takes two numbers, the low then the high"
        )),
    }
}

/// Reads a terrain's `generate` entry.
pub fn parse(entry: &Entry) -> Result<Generate, String> {
    let Some(b) = &entry.block else {
        return Err(format!(
            "line {}: generate is a block: generate {{ ... }}",
            entry.line
        ));
    };
    check_keys(b, KEYS, "generate")?;
    let (columns, rows) = match values(b, "samples")? {
        None => (257, 257),
        Some((v, line)) => {
            let ok = v.len() == 2
                && v.iter()
                    .all(|x| x.fract() == 0.0 && (2.0..=4097.0).contains(x));
            if !ok {
                return Err(format!(
                    "line {line}: 'samples' takes two whole numbers from 2 to 4097 (columns and rows)"
                ));
            }
            (v[0] as u32, v[1] as u32)
        }
    };
    let seed = whole(b, "seed", 1, 0, u32::MAX)?;
    let empty = Block::default();
    let noise = sub(b, "noise", NOISE_KEYS)?.unwrap_or(&empty);
    let mask = match b.get("mask") {
        None => Mask::None,
        Some(e) => {
            let word = e.values.first().and_then(|v| v.as_word());
            let amount = e
                .values
                .get(1)
                .and_then(|v| v.as_number())
                .map(|n| n.as_f32());
            match (word, amount, e.values.len()) {
                (Some("island"), a, n) if n <= 2 => {
                    let f = a.unwrap_or(0.4);
                    if !(f > 0.0 && f <= 1.0) {
                        return Err(format!(
                            "line {}: an island's falloff is a share of the half size, above 0 and at most 1, got {f}",
                            e.line
                        ));
                    }
                    Mask::Island(f)
                }
                (Some("ridge"), a, n) if n <= 2 => {
                    let s = a.unwrap_or(1.0);
                    if !(0.0..=1.0).contains(&s) {
                        return Err(format!(
                            "line {}: a ridge's strength is from 0 to 1, got {s}",
                            e.line
                        ));
                    }
                    Mask::Ridge(s)
                }
                _ => {
                    return Err(format!(
                        "line {}: mask is `mask island <falloff>` or `mask ridge <strength>`",
                        e.line
                    ));
                }
            }
        }
    };
    let erosion = match sub(b, "erosion", EROSION_KEYS)? {
        None => None,
        Some(e) => Some(Hydraulic {
            drops: whole(e, "drops", 40000, 0, 10_000_000)?,
            lifetime: whole(e, "lifetime", 30, 1, 1000)?,
            radius: whole(e, "radius", 3, 1, 16)?,
            inertia: num(e, "inertia", 0.05, 0.0, 1.0)?,
            capacity: num(e, "capacity", 4.0, 0.0, 1000.0)?,
            deposition: num(e, "deposition", 0.3, 0.0, 1.0)?,
            erosion: num(e, "erosion", 0.3, 0.0, 1.0)?,
            evaporation: num(e, "evaporation", 0.01, 0.0, 1.0)?,
            gravity: num(e, "gravity", 4.0, 0.0, 1000.0)?,
        }),
    };
    let thermal = match sub(b, "thermal", THERMAL_KEYS)? {
        None => None,
        Some(t) => Some(Thermal {
            talus: num(t, "talus", 35.0, 0.0, 89.0)?,
            iterations: whole(t, "iterations", 20, 0, 10000)?,
            rate: num(t, "rate", 0.5, 0.0, 1.0)?,
        }),
    };
    let terraces = match sub(b, "terraces", TERRACE_KEYS)? {
        None => None,
        Some(t) => Some([
            num(t, "step", 10.0, 1.0e-3, 1.0e6)?,
            num(t, "blend", 0.4, 0.0, 1.0)?,
        ]),
    };
    let sea = match values(b, "sea")? {
        None => None,
        Some((v, _)) if v.len() == 1 => Some(v[0]),
        Some((_, line)) => {
            return Err(format!(
                "line {line}: 'sea' takes one number: the metres above the origin"
            ));
        }
    };
    if let (None, Some(e)) = (sea, b.get("shore")) {
        return Err(format!(
            "line {}: 'shore' flattens the land beside the sea: give the sea first (sea <metres>)",
            e.line
        ));
    }
    let rivers = match sub(b, "rivers", RIVER_KEYS)? {
        None => None,
        Some(r) => Some(Rivers {
            threshold: whole(r, "threshold", 400, 1, u32::MAX)?,
            depth: num(r, "depth", 2.0, 0.0, 1.0e6)?,
            width: num(r, "width", 6.0, 0.0, 1.0e6)?,
        }),
    };
    let m = sub(b, "moisture", MOISTURE_KEYS)?.unwrap_or(&empty);
    let moisture = Moisture {
        flow: num(m, "flow", 1.0, 0.0, 1.0e6)?,
        scale: num(m, "scale", 64.0, 1.0e-3, 1.0e9)?,
        noise: num(m, "noise", 0.5, 0.0, 1.0e6)?,
        frequency: num(m, "frequency", 6.0, 1.0e-3, 1.0e4)?,
    };
    let mut biomes = Vec::new();
    for e in b.get_all("biome") {
        let Some(r) = &e.block else {
            return Err(format!(
                "line {}: a biome is a block: biome {{ layer 0 ... }}",
                e.line
            ));
        };
        check_keys(r, BIOME_KEYS, "biome")?;
        if r.get("layer").is_none() {
            return Err(format!(
                "line {}: a biome names the layer it writes: layer <0 to 15>",
                e.line
            ));
        }
        biomes.push(Biome {
            layer: whole(
                r,
                "layer",
                0,
                0,
                og_render_data::MAX_TERRAIN_LAYERS as u32 - 1,
            )?,
            weight: num(r, "weight", 1.0, 0.0, 1.0)?,
            altitude: range(r, "altitude", [-1.0e6, 1.0e6])?,
            altitude_fade: num(r, "altitudeFade", 2.0, 0.0, 1.0e6)?,
            slope: range(r, "slope", [0.0, 90.0])?,
            slope_fade: num(r, "slopeFade", 4.0, 0.0, 90.0)?,
            moisture: range(r, "moisture", [0.0, 1.0])?,
            moisture_fade: num(r, "moistureFade", 0.1, 0.0, 1.0)?,
            line: e.line,
        });
    }
    let mut channels = Vec::new();
    for e in b.get_all("channel") {
        if !e.values.is_empty() {
            return Err("channel takes a block, not values".into());
        }
        let c = e.block.as_ref().ok_or("channel needs a block")?;
        check_keys(c, super::terrain_schema::CHANNEL, "channel")?;
        let mut points = Vec::new();
        for p in c.get_all("point") {
            let values: Option<Vec<_>> = p
                .values
                .iter()
                .map(|v| v.as_number().map(|v| v.as_f32()))
                .collect();
            let values = values
                .filter(|v| v.len() == 3 && p.block.is_none())
                .ok_or("channel point takes x y z")?;
            points.push([values[0], values[1], values[2]]);
        }
        if c.get("width").is_none() {
            return Err("channel needs an explicit width".into());
        }
        let river = c
            .get("river")
            .map(|e| {
                if e.values.len() != 1 || e.block.is_some() {
                    return Err("channel.river takes true or false".to_string());
                }
                e.values[0]
                    .as_bool()
                    .ok_or_else(|| "channel.river takes true or false".to_string())
            })
            .transpose()?
            .unwrap_or(false);
        channels.push(Channel {
            points,
            width: num(c, "width", 0.0, 0.001, 1e6)?,
            depth: num(c, "depth", 0.0, 0.0, 1e6)?,
            river,
        });
    }
    let generate = Generate {
        columns,
        rows,
        seed,
        octaves: whole(noise, "octaves", 6, 1, 12)?,
        frequency: num(noise, "frequency", 3.0, 1.0e-3, 1.0e4)?,
        lacunarity: num(noise, "lacunarity", 2.0, 1.0, 8.0)?,
        gain: num(noise, "gain", 0.5, 0.0, 1.0)?,
        warp: num(noise, "warp", 0.0, 0.0, 100.0)?,
        mask,
        erosion,
        thermal,
        terraces,
        sea,
        shore: num(b, "shore", 2.0, 0.0, 1.0e6)?,
        rivers,
        channels,
        preserve: Vec::new(),
        preserve_border: whole(b, "preserveBorder", 0, 0, 4097)?,
        moisture,
        biomes,
    };
    generate.check()?;
    Ok(generate)
}
