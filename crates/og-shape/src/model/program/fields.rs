//! Value vocabulary for the exact operation/profile key registry.
use og_core::ogt::line::Line;

/// The registry supplies names; this vocabulary supplies their value forms.
/// Missing forms fail API generation instead of publishing an untyped field.
pub fn field(kind: &str, key: &'static str, profile: bool) -> Result<Line, String> {
    let args = match (profile, kind, key) {
        (false, "clip", "keys") => {
            "[{time: seconds, pose: {jointName: number}}]; strictly increasing times"
        }
        (false, "joint", "axis") => "[x,y,z]; local direction",
        (true, "mirror", "axis") => "\"x\" | \"y\"",
        (_, _, "axis") => "\"+Y\" | \"+Z\"",
        (_, "fillet", "profile") => "number; bevel profile factor",
        (_, _, "profile") => "one-key profile object; see file:model-program.profile.*",
        (_, _, "plan" | "top" | "bottom") | (false, "body", "n") => {
            "number | [[y,value], ...] | {points: [[y,value],...], smooth: boolean}; model units"
        }
        (_, _, "on" | "frame" | "plane" | "local") => {
            "{origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame"
        }
        (_, "dent", "at") => "[[x,y,z], ...]; model units",
        (_, _, "at") => "frame | [frame, ...]; model units",
        (_, "along", "path") => "{points:[[x,y,z], ...], closed:boolean}",
        (_, _, "path") => {
            "point/piece array; arc {through,to}, tangentArc {to}, spline {points}; 3D also {helix:{radius,pitch,turns}}"
        }
        (_, _, "sections") => {
            "[{profile, z:number}, ...] | [{profile, at:frame}, ...]; no mixed placement forms"
        }
        (_, _, "until") => "{part: id, face: string}; extrude until the selected face",
        (_, _, "mimic") => "{joint:string, multiplier:number}",
        (_, _, "anchors") => "{name: frame, ...}",
        (_, _, "bones") => {
            "[{name:string, parent?:string, pivot?:[x,y,z] or frame, regions?:[faceName,...]}, ...]; parent first"
        }
        (_, "skin", "joints") => {
            "[{name:string, parent?:string, at:[x,y,z]}, ...]; bone-heat skeleton, exclusive with bones"
        }
        (_, "morph", "target") => "part id; exclusive with deltas/regions",
        (_, _, "id" | "target" | "tool" | "host" | "part" | "child" | "parent" | "to") => {
            "u32 part id | {id:u32}"
        }
        (_, _, "ids" | "children" | "parts") => "part id | [part id, ...]",
        (_, _, "color" | "emissive" | "sheenColor") => "\"#rrggbb\" | [r,g,b]; sRGB",
        (_, _, "uvs" | "bothSides" | "closed" | "straight" | "stagger" | "check" | "bind") => {
            "boolean"
        }
        (true, "polygon", "sides") => "u32; polygon side count",
        (_, _, "open" | "edges" | "sharp" | "sides") => {
            "string | [string, ...]; named faces/edges/sides"
        }
        (_, "measure", "posed") => "{jointName:number, ...}; optional pose for measurement",
        (_, _, "points") if profile => {
            "[[x,y], ...]; poly also accepts {x,y,radius?,bulge?} (bulge in half-chords)"
        }
        (_, _, "points" | "deltas") => "[[x,y,z], ...]; model units",
        (_, _, "regions") => "[string, ...]; named faces",
        (_, "morph", "offset" | "scale") => "[x,y,z]; displacement in model units / scale factors",
        (_, "displace", "scale") => "number; noise frequency scale",
        (_, _, "origin" | "normal" | "up" | "direction") => {
            "[x,y,z]; origin in model units, others directions"
        }
        (_, _, "range" | "counterbore" | "countersink") => {
            "[number,number]; range in joint units, counterbore [diameter,depth], countersink [diameter,radians]"
        }
        (_, _, "fillets") => "[number, ...]; one model-unit radius per path point",
        (_, _, "unit") => "\"mm\" | \"cm\" | \"m\"",
        (_, _, "kind") => {
            "\"revolute\" | \"continuous\" | \"prismatic\" | \"fixed\" | \"floating\""
        }
        (_, _, "keep") => "\"above\" | \"below\"",
        (_, _, "curve") => {
            "\"auto\" | \"straight\" | \"catmullRom\" | \"spline\"; placed sections only straight/catmullRom"
        }
        (_, _, "noise") => "\"fbm\" | \"ridged\" | \"crater\"",
        (_, _, "join") => "\"round\" | \"miter\" | \"bevel\"",
        (_, _, "roundCaps") => "boolean",
        (_, _, "surface") => {
            "{density?, seed?, normalScale?, layers?:[layer,...]}; see file:model-program.surface"
        }
        (_, _, "name" | "text" | "font" | "map" | "joint" | "clip") => {
            "string; font/map names are tracked source assets"
        }
        (
            _,
            _,
            "nx" | "ny" | "n" | "count" | "segments" | "tubeSegments" | "samples" | "steps"
            | "teeth" | "levels" | "seed" | "octaves",
        ) => "u32; zero requests the mechanism's automatic choice where supported",
        (_, _, "angle" | "start" | "twist" | "pressureAngle" | "rx" | "ry" | "rz")
            if !profile || !matches!(key, "rx" | "ry") =>
        {
            "number; radians"
        }
        (_, _, "x" | "y" | "z") if kind == "scale" => "number; scale factor (zero keeps one)",
        (_, _, "time") => "number; seconds",
        (_, _, "value") => "number; radians for rotary joints, model units for prismatic joints",
        (
            _,
            _,
            "metalness" | "roughness" | "opacity" | "clearcoat" | "clearcoatRoughness"
            | "sheenRoughness" | "transmission" | "iridescence" | "taper" | "scaleEnd" | "exponent"
            | "blend" | "weight" | "density",
        ) => "number; dimensionless",
        (
            _,
            _,
            "width" | "depth" | "height" | "radius" | "radiusTop" | "outer" | "inner" | "tube"
            | "length" | "pitch" | "diameter" | "module" | "bore" | "hub" | "hubHeight"
            | "distance" | "distance2" | "thickness" | "fillet" | "flats" | "rx" | "ry" | "size"
            | "spacing" | "mapSize" | "maxEdge" | "amount" | "px" | "py" | "x" | "y" | "z",
        ) => "number; model units",
        _ => return Err(format!("model program {kind}.{key} has no value contract")),
    };
    Ok(Line::new(
        key,
        args,
        "see model mechanism",
        "Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up.",
        "",
    ))
}

/// Procedural surface keys shared by JSON validation and the file catalogue.
pub const SURFACE: &[Line] = &[
    Line::new(
        "density",
        "number",
        "0 (kernel default)",
        "Texels per model unit.",
        "",
    ),
    Line::new("seed", "u32", "0", "Surface random seed.", ""),
    Line::new(
        "normalScale",
        "number",
        "0 (kernel default)",
        "Relief-to-normal strength.",
        "",
    ),
    Line::new(
        "layers",
        "[layer, ...]",
        "[]",
        "Ordered procedural paint layers.",
        "",
    ),
];
/// One procedural paint layer's keys, values and defaults.
pub const SURFACE_LAYER: &[Line] = &[
    Line::new(
        "pattern",
        "noise | ridged | craters | strata | stripes | panels | rivets | veins | edges | cavities | facing | gradient",
        "required",
        "Shared paint mechanism.",
        "",
    ),
    Line::new(
        "scale",
        "number",
        "0 (kernel default)",
        "Pattern scale in model units.",
        "",
    ),
    Line::new(
        "width",
        "number",
        "0 (kernel default)",
        "Pattern width.",
        "",
    ),
    Line::new(
        "cell",
        "number",
        "0 (kernel default)",
        "Pattern cell size.",
        "",
    ),
    Line::new("inset", "number", "0 (kernel default)", "Panel inset.", ""),
    Line::new(
        "axis",
        "signed axis string | [x,y,z]",
        "+Z",
        "Projection direction.",
        "",
    ),
    Line::new(
        "octaves",
        "u32",
        "0 (kernel default)",
        "Noise octave count.",
        "",
    ),
    Line::new(
        "threshold",
        "number",
        "pattern default",
        "Pattern threshold.",
        "",
    ),
    Line::new(
        "softness",
        "number",
        "0 (kernel default)",
        "Threshold softness.",
        "",
    ),
    Line::new("seed", "u32", "0", "Layer random seed.", ""),
    Line::new("amount", "number", "1", "Layer blend weight.", ""),
    Line::new("breakup", "number", "0", "Pattern breakup.", ""),
    Line::new(
        "color",
        "hex string | [r,g,b]",
        "unchanged",
        "Layer colour.",
        "",
    ),
    Line::new("jitter", "number", "0", "Colour variation.", ""),
    Line::new("roughness", "number", "unchanged", "Layer roughness.", ""),
    Line::new(
        "metalness",
        "number",
        "unchanged",
        "Layer metallic factor.",
        "",
    ),
    Line::new(
        "emissive",
        "hex string | [r,g,b]",
        "unchanged",
        "Emitted colour.",
        "",
    ),
    Line::new("glow", "number", "1", "Emission strength.", ""),
    Line::new("relief", "number", "0", "Surface displacement.", ""),
];
