//! Recognised recipe fields. Notes remain ignored as in the Forge format.
use super::json::Val;
use crate::grammar::Line;
pub const LINES: &[Line] = &[
    Line::new(
        "duration",
        "<seconds > 0>",
        "frames / fps",
        "Override clip duration.",
        "",
    ),
    Line::new("loop", "true | false", "recipe", "Clip playback mode.", ""),
    Line::new(
        "impact",
        "<seconds >= 0>",
        "recipe or coverage jump",
        "One impact marker; must lie within duration.",
        "",
    ),
    Line::new(
        "event",
        "<seconds >= 0> \"<name>\"",
        "",
        "Repeated markers, at most 16 including impact; names 1..63 bytes.",
        "",
    ),
    Line::new(
        "recipe",
        "\"<path.json>\"",
        "inline JSON",
        "Relative file or asset id, exclusive with inline JSON.",
        "",
    ),
];
pub const RECIPE: &[Line] = &[
    Line::new("name", "string", "Untitled effect", "Display name.", ""),
    Line::new("w", "integer, 8..512", "64", "Canvas pixels.", ""),
    Line::new("h", "integer, 8..512", "64", "Canvas pixels.", ""),
    Line::new("frames", "integer, 1..120", "16", "Frame count.", ""),
    Line::new("fps", "integer, 1..60", "15", "Frames per second.", ""),
    Line::new(
        "seed",
        "integer, 0..2147483647",
        "1",
        "Deterministic seed.",
        "",
    ),
    Line::new("loop", "boolean", "false", "Periodic animation.", ""),
    Line::new(
        "ramps",
        "{name: ramp}",
        "Forge ramps",
        "Ramp is a named ramp, hue:<degrees>, or hex colour array.",
        "",
    ),
    Line::new("hue", "number, degrees", "0", "Ramp hue rotation.", ""),
    Line::new("sat", "number", "1", "Saturation multiplier.", ""),
    Line::new(
        "outline",
        "none | outside | outside8 | inside",
        "none",
        "Pixel outline mode.",
        "",
    ),
    Line::new(
        "outlineColor",
        "hex colour or null",
        "null",
        "Fixed outline colour.",
        "",
    ),
    Line::new("dither", "number, 0..1", "0", "Ordered dither.", ""),
    Line::new(
        "palette",
        "named palette | hex colour array | null",
        "null",
        "Quantise to an explicit palette.",
        "",
    ),
    Line::new(
        "cleanup",
        "boolean",
        "true",
        "Clean isolated pixels and holes.",
        "",
    ),
    Line::new(
        "pivot",
        "[x,y], 0..1",
        "[0.5,0.5]",
        "Top-left canvas fractions.",
        "",
    ),
    Line::new(
        "impact",
        "number, 0..1",
        "coverage jump",
        "Normalised impact time.",
        "",
    ),
    Line::new(
        "layers",
        "[layer, ...]",
        "required",
        "Layers paint in authored order.",
        "",
    ),
];
pub const COMMON: &[&str] = &[
    "type",
    "ramp",
    "t0",
    "t1",
    "x",
    "y",
    "angle",
    "squash",
    "gain",
    "cut",
    "gamma",
    "dither",
    "glow",
    "glowFalloff",
    "blend",
    "hidden",
    "seed",
    "hue",
    "ease",
];
pub const LAYERS: &[(&str, &[&str])] = &[
    (
        "burst",
        &[
            "cool",
            "drift",
            "erode",
            "erodeStart",
            "evolve",
            "freq",
            "grow",
            "hollow",
            "r0",
            "r1",
            "rough",
            "wobble",
        ],
    ),
    (
        "ring",
        &[
            "breakup", "fade", "freq", "inner", "r0", "r1", "th", "thEnd",
        ],
    ),
    (
        "flame",
        &[
            "flick", "freq", "hgt", "hollow", "rough", "speed", "taper", "w",
        ],
    ),
    (
        "beam",
        &["cap", "freq", "grow", "len", "speed", "th", "wob"],
    ),
    ("slash", &["a0", "a1", "erode", "r", "sweep", "th", "trail"]),
    (
        "orb",
        &["freq", "grow", "pulse", "pulseRate", "r", "rim", "spin"],
    ),
    (
        "comet",
        &["freq", "from", "len", "r", "rough", "speed", "until"],
    ),
    (
        "vortex",
        &["arms", "core", "dir", "freq", "r", "rIn", "spin", "twist"],
    ),
    ("pillar", &["freq", "grow", "hgt", "rough", "speed", "w"]),
    (
        "rays",
        &["count", "inner", "jitter", "r", "spin", "twinkle", "width"],
    ),
    (
        "glyph",
        &[
            "draw", "fill", "r", "rings", "runes", "spin", "star", "step", "ticks",
        ],
    ),
    (
        "puffs",
        &[
            "cone", "count", "decay", "dir", "dissolve", "drift", "emit", "flat", "freq", "life",
            "r0", "r1", "rise", "speed", "spread",
        ],
    ),
    (
        "particles",
        &[
            "area",
            "areaX",
            "areaY",
            "both",
            "count",
            "dir",
            "drag",
            "driftX",
            "emit",
            "fadeIn",
            "fallScale",
            "gravity",
            "ground",
            "heat",
            "implodeR",
            "jitter",
            "life",
            "mode",
            "orbitR",
            "shape",
            "size",
            "spanX",
            "spanY",
            "speed",
            "spread",
            "swirl",
            "turns",
            "twinkle",
            "wiggle",
            "wiggleRate",
        ],
    ),
    (
        "bolt",
        &[
            "branch",
            "count",
            "depth",
            "fadeStart",
            "flicker",
            "hold",
            "jag",
            "len",
            "mode",
            "spread",
            "thick",
            "x0",
            "x1",
            "y0",
            "y1",
        ],
    ),
    ("cracks", &["count", "fadeStart", "grow", "len"]),
    (
        "spikes",
        &[
            "count", "fan", "grow", "hgt", "hold", "lean", "spread", "width",
        ],
    ),
    (
        "star",
        &["hollow", "inner", "jitter", "points", "pop", "r", "sharp"],
    ),
    (
        "streaks",
        &[
            "count", "curve", "dir", "gap", "len", "mode", "r", "spanY", "speed", "stagger", "th",
            "turns",
        ],
    ),
    ("bubble", &["cell", "fill", "hex", "r", "shimmer", "th"]),
    ("flash", &["dur", "r"]),
    ("text", &["text", "fadeStart", "pop", "rise", "scale"]),
    (
        "tornado",
        &["freq", "hgt", "rough", "spin", "sway", "twist", "w0", "w1"],
    ),
];

pub fn layer_fields(kind: &str) -> Vec<Line> {
    COMMON.iter().chain(LAYERS.iter().find(|(name, _)| *name == kind).map_or(&[][..], |(_, fields)| *fields)).map(|&key| {
        let args = match key {
            "type" => "layer type name", "ramp" => "ramp name | $name | hex colour array",
            "hidden" | "both" => "boolean", "pop" if kind == "text" => "boolean",
            "from" => "[x,y]", "rings" => "[number,...]", "text" => "string",
            "mode" | "shape" | "core" | "blend" | "ease" => "string; see pixel-vfx for enumerated values",
            "life" | "speed" | "size" | "heat" if kind == "particles" => "number | [min,max]",
            _ => "number",
        };
        Line::new(key, args, "renderer; see pixel-vfx", "Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer.", "")
    }).collect()
}

pub(super) fn recipe(mut value: Val) -> Val {
    if let Val::Obj(root) = &mut value {
        root.retain(|(key, _)| RECIPE.iter().any(|f| f.key == key));
        if let Some((_, Val::Arr(layers))) = root.iter_mut().find(|(key, _)| key == "layers") {
            for layer in layers {
                let fields = layer_fields(layer.s("type").unwrap_or(""));
                if let Val::Obj(entries) = layer {
                    entries.retain(|(key, _)| fields.iter().any(|f| f.key == key));
                }
            }
        }
    }
    value
}
