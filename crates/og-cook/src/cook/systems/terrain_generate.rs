//! Typed terrain inputs normalize into the runtime's checked generation spec.
use crate::cook::source_schemas::procedural::terrain::{TerrainMask, TerrainProcess};
use og_render_data::terrain::generate::{Biome, Channel, Hydraulic, Moisture, Rivers, Thermal};
pub use og_render_data::terrain::generate::{Generate, Mask, generate, process};

macro_rules! patch {
    ($out:expr, $input:expr, $($key:ident),+ $(,)?) => { $(
        if let Some(value) = $input.$key { $out.$key = value; }
    )+ };
}

pub(crate) fn mask(value: Option<TerrainMask>) -> Mask {
    match value {
        None | Some(TerrainMask::None) => Mask::None,
        Some(TerrainMask::Island { falloff }) => Mask::Island(falloff.unwrap_or(0.4)),
        Some(TerrainMask::Ridge { strength }) => Mask::Ridge(strength.unwrap_or(1.0)),
    }
}

pub(crate) fn processing(
    g: &mut Generate,
    input: TerrainProcess,
) -> Result<Option<String>, String> {
    if input.sea.is_none() && input.shore.is_some() {
        return Err("process.shore requires process.sea".into());
    }
    g.erosion = input.erosion.map(|v| {
        let mut out = Hydraulic::default();
        patch!(
            out,
            v,
            drops,
            lifetime,
            radius,
            inertia,
            capacity,
            deposition,
            erosion,
            evaporation,
            gravity
        );
        out
    });
    g.thermal = input.thermal.map(|v| {
        let mut out = Thermal::default();
        patch!(out, v, talus, iterations, rate);
        out
    });
    g.terraces = input
        .terraces
        .map(|v| [v.step.unwrap_or(10.0), v.blend.unwrap_or(0.4)]);
    g.rivers = input.rivers.map(|v| {
        let mut out = Rivers::default();
        patch!(out, v, threshold, depth, width);
        out
    });
    g.channels = input
        .channels
        .unwrap_or_default()
        .into_iter()
        .map(|v| Channel {
            points: v.points,
            width: v.width,
            depth: v.depth.unwrap_or(0.0),
            river: v.river.unwrap_or(false),
        })
        .collect();
    g.moisture = input.moisture.map_or_else(Moisture::default, |v| {
        let mut out = Moisture::default();
        patch!(out, v, flow, scale, noise, frequency);
        out
    });
    g.biomes = input
        .biomes
        .unwrap_or_default()
        .into_iter()
        .map(|v| {
            let mut out = Biome {
                layer: v.layer,
                ..Default::default()
            };
            patch!(
                out,
                v,
                weight,
                altitude,
                altitude_fade,
                slope,
                slope_fade,
                moisture,
                moisture_fade
            );
            out
        })
        .collect();
    patch!(g, input, shore, preserve_border);
    g.sea = input.sea;
    g.check()?;
    Ok(input.preserve.map(|r| r.id))
}
