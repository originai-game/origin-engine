//! Terrain parser line tables, also emitted in the file-type reference.
use crate::grammar::Line;
use og_core::ogt::Block;

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal;)+) => {
        &[$(Line::new($key, $args, $default, $doc, "")),+]
    };
}

pub const BODY: &[Line] = lines! {
    "heightmap", "\"<png|exr>\"", "", "Image input, relative to the terrain then assets/. May be followed by generate processing. Exclusive with generate.script.";
    "generate", "{ ... }", "", "Noise or script input, followed by hydraulic, thermal, terraces, shore, rivers, biome stages in that order.";
    "height", "<low> <high>", "0 100 (EXR: 0 1)", "Metres corresponding to image 0 and 1, or noise range. Not allowed with script; authored fields are not normalised or clipped by processing.";
    "size", "<x> <z>", "one metre per sample interval", "Positive finite world extents in metres; rows run -z to +z, columns -x to +x.";
    "control", "\"<image>\" ...", "", "RGBA weights, one image per four layers, all equal size. Exclusive with biome rules.";
    "holes", "\"<image>\"", "", "Black or transparent cells have no surface or collision.";
    "tile", "<integer 16..1024>", "128", "Quads per streaming tile side.";
    "heightBlend", "<number >= 0>", "0", "Layer height-blend amount.";
    "castShadows", "true | false", "true", "Terrain casts shadows.";
    "collision", "true | false", "true", "Build terrain collision.";
    "lod", "{ ... }", "", "Drawn clipmap spacing and ring.";
    "layer", "{ ... }", "", "Repeated material layer; at most sixteen. Past four use texture arrays.";
    "layers", "\"<texture-array.ogt>\"", "", "Albedo array asset.";
    "layerNormals", "\"<texture-array.ogt>\"", "", "Normal array asset.";
};
pub const LOD: &[Line] = lines! {
    "spacing", "<number >= 0>", "0", "Finest spacing in metres; zero uses sample spacing.";
    "ring", "<integer >= 0>", "0", "Clipmap quads per side; zero selects the renderer's ring.";
};
pub const LAYER: &[Line] = lines! {
    "albedo", "\"<texture>\"", "", "Albedo asset; alpha holds material height. With arrays use albedoLayer.";
    "normal", "\"<texture>\"", "", "Normal asset; with arrays use normalLayer.";
    "albedoLayer", "<integer >= 0>", "layer index", "Albedo-array slice.";
    "normalLayer", "<integer >= 0>", "layer index", "Normal-array slice.";
    "color", "<r> <g> <b>", "1 1 1", "Linear colour multiplier.";
    "roughness", "<number 0..1>", "0.9", "PBR roughness.";
    "tile", "<metres > 0>", "4", "World metres per texture repeat.";
    "triplanar", "true | false", "false", "Project texture on three axes.";
    "normalStrength", "<number 0..4>", "1", "Normal strength.";
    "slope", "<low> <high>", "0 90", "Slope band in degrees.";
    "slopeFade", "<number >= 0>", "4", "Slope-band fade in degrees.";
    "altitude", "<low> <high>", "-1000000 1000000", "Height band in metres above origin.";
    "altitudeFade", "<number >= 0>", "4", "Height-band fade in metres.";
    "projectSlope", "<degrees 0..90>", "0", "Use the two facing projections above this slope; zero follows triplanar.";
    "antiTiling", "<number 0..1>", "0", "Hex anti-tiling amount.";
    "heightContrast", "<number 0..1>", "0", "Height-blend contrast.";
    "macroVariation", "<number 0..1>", "0", "Coarse brightness and hue variation.";
    "displacement", "<metres 0..4>", "0", "Near-camera displacement from albedo alpha.";
    "cellVariation", "<number 0..1>", "0", "Per-cell colour variation from normal alpha.";
};
pub const GENERATE: &[Line] = lines! {
    "samples", "<columns 2..4097> <rows 2..4097>", "257 257", "Noise grid only. An authored input supplies its dimensions; no implicit resampling.";
    "seed", "<integer 0..4294967295>", "1", "Noise, erosion and script seed.";
    "noise", "{ ... }", "", "Noise input parameters; exclusive with authored input.";
    "mask", "island [<falloff > 0 and <= 1>] | ridge [<strength 0..1>]", "none", "Noise shaping only; island defaults to 0.4, ridge to 1.";
    "erosion", "{ ... }", "", "Hydraulic droplets; absent means no hydraulic processing.";
    "thermal", "{ ... }", "", "Thermal talus processing; absent means none.";
    "terraces", "{ ... }", "", "Terraces; authored fields anchor at zero, noise at its low height.";
    "sea", "<finite metres>", "", "Sea level for shore flattening and automatic drainage.";
    "shore", "<metres 0..1000000>", "2", "Shore width; requires sea if explicitly written.";
    "rivers", "{ ... }", "", "Automatic drainage and river carving; absent means none.";
    "moisture", "{ ... }", "", "Flow/noise mixture read by biome rules.";
    "biome", "{ ... }", "", "Repeated rule, writes an existing material layer's control channel.";
    "script", "\"<file.ts>\"", "", "Default synchronous export returns {columns, rows, heights}; heights are metres. Relative imports and explicit inputs are tracked. Exclusive with heightmap/noise/mask/samples.";
    "params", "{ <name> <value> ... }", "{}", "Script parameters: scalars, arrays or nested blocks. Requires script.";
    "input", "\"<name>\" \"<path>\"", "", "Repeated explicit script input, delivered as ctx.inputs[name] byte array; relative to the terrain. Requires script.";
};
pub const NOISE: &[Line] = lines! {
    "octaves", "<integer 1..12>", "6", "FBM octave count.";
    "frequency", "<number 0.001..10000>", "3", "Features across the longer world side.";
    "lacunarity", "<number 1..8>", "2", "Frequency multiplier per octave.";
    "gain", "<number 0..1>", "0.5", "Amplitude multiplier per octave.";
    "warp", "<number 0..100>", "0", "Domain-warp strength in features.";
};
pub const EROSION: &[Line] = lines! {
    "drops", "<integer 0..10000000>", "40000", "Seeded droplet count.";
    "lifetime", "<integer 1..1000>", "30", "Steps per droplet.";
    "radius", "<integer 1..16>", "3", "Erosion brush radius in samples.";
    "inertia", "<number 0..1>", "0.05", "Share of previous direction.";
    "capacity", "<number 0..1000>", "4", "Sediment capacity.";
    "deposition", "<number 0..1>", "0.3", "Excess sediment deposition fraction.";
    "erosion", "<number 0..1>", "0.3", "Unused sediment capacity eroded per step.";
    "evaporation", "<number 0..1>", "0.01", "Water evaporated per step.";
    "gravity", "<number 0..1000>", "4", "Droplet acceleration.";
};
pub const THERMAL: &[Line] = lines! {
    "talus", "<degrees 0..89>", "35", "Angle above which material slumps.";
    "iterations", "<integer 0..10000>", "20", "Thermal iterations.";
    "rate", "<number 0..1>", "0.5", "Fraction of excess moved each iteration.";
};
pub const TERRACES: &[Line] = lines! {
    "step", "<metres 0.001..1000000>", "10", "Terrace step height.";
    "blend", "<number 0..1>", "0.4", "Smooth fraction of each step.";
};
pub const RIVERS: &[Line] = lines! {
    "threshold", "<integer 1..4294967295>", "400", "Contributing samples at which a river starts.";
    "depth", "<metres 0..1000000>", "2", "Full river depth.";
    "width", "<metres 0..1000000>", "6", "Full river width.";
};
pub const MOISTURE: &[Line] = lines! {
    "flow", "<number 0..1000000>", "1", "Gathered-flow weight.";
    "scale", "<number 0.001..1000000000>", "64", "Contributing samples for half the flow contribution.";
    "noise", "<number 0..1000000>", "0.5", "Noise weight.";
    "frequency", "<number 0.001..10000>", "6", "Moisture-noise features across terrain.";
};
pub const BIOME: &[Line] = lines! {
    "layer", "<integer 0..15>", "required", "Existing material layer to paint.";
    "weight", "<number 0..1>", "1", "Rule weight.";
    "altitude", "<low> <high>", "-1000000 1000000", "Ascending finite height band in metres.";
    "altitudeFade", "<metres 0..1000000>", "2", "Height-band fade.";
    "slope", "<low> <high>", "0 90", "Ascending finite slope band in degrees.";
    "slopeFade", "<degrees 0..90>", "4", "Slope-band fade.";
    "moisture", "<low> <high>", "0 1", "Ascending finite moisture band.";
    "moistureFade", "<number 0..1>", "0.1", "Moisture-band fade.";
};

pub const BLOCKS: &[(&str, &[Line])] = &[
    ("", BODY),
    ("lod", LOD),
    ("layer", LAYER),
    ("generate", GENERATE),
    ("generate.noise", NOISE),
    ("generate.erosion", EROSION),
    ("generate.thermal", THERMAL),
    ("generate.terraces", TERRACES),
    ("generate.rivers", RIVERS),
    ("generate.moisture", MOISTURE),
    ("generate.biome", BIOME),
];

pub fn check(b: &Block, lines: &[Line], what: &str) -> Result<(), String> {
    for e in &b.entries {
        if !lines.iter().any(|l| l.key == e.key) {
            return Err(format!(
                "line {}: '{}' is not a {what} entry: {}",
                e.line,
                e.key,
                crate::grammar::keys(lines, &[])
            ));
        }
        if !matches!(e.key.as_str(), "layer" | "biome" | "input") && b.get_all(&e.key).count() > 1 {
            return Err(format!(
                "line {}: duplicate {what} entry '{}'",
                e.line, e.key
            ));
        }
    }
    Ok(())
}

/// Like the effect grammar, check documented defaults through the actual
/// parser. A changed parser default cannot silently leave stale reference data.
pub fn check_defaults() -> Result<(), String> {
    for (path, fields) in BLOCKS.iter().filter(|(p, _)| p.starts_with("generate")) {
        let wrap = |body: &str| match path.strip_prefix("generate.") {
            Some("biome") => format!("generate {{\nbiome {{\nlayer 0\n{body}\n}}\n}}"),
            Some(name) => format!("generate {{\n{name} {{\n{body}\n}}\n}}"),
            None => format!("generate {{\nsea 0\n{body}\n}}"),
        };
        let parse = |body: &str| {
            let doc = og_core::ogt::Document::parse(&format!(
                "type terrain\nversion 1\n---\n{}",
                wrap(body)
            ))
            .map_err(|e| e.to_string())?;
            let mut g = super::terrain_generate::parse(
                doc.body.get("generate").expect("wrapped generate"),
            )?;
            for b in &mut g.biomes {
                b.line = 0;
            }
            Ok::<_, String>(g)
        };
        let baseline = parse("")?;
        for f in *fields {
            if f.default.is_empty() || matches!(f.default, "none" | "required" | "{}") {
                continue;
            }
            let made = parse(&format!("{} {}", f.key, f.default))?;
            if made != baseline {
                return Err(format!(
                    "file:terrain.{path}.{}: documented default {} differs from the parser",
                    f.key, f.default
                ));
            }
        }
    }
    Ok(())
}
