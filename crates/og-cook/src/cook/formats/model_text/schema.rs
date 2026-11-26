//! The model parser's accepted fields and the file reference share these rows.
use crate::grammar::Line;

macro_rules! rows {
    ($($key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(Line::new($key, $args, $default, $doc, "")),*]
    };
}

pub const PLACEMENT: &[Line] = rows![
    "position", "<x> <y> <z>", "0 0 0", "Translation in metres.";
    "quaternion", "<x> <y> <z> <w>", "0 0 0 1", "Nonzero quaternion, normalized on import.";
    "scale", "<x> <y> <z>", "1 1 1", "Nonzero scale; negative components bake reflected triangles.";
    "mirror", "none | x | y | z", "none", "Reflect geometry, correcting winding and tangent handedness."
];

const GEOMETRY_COMMON: &[Line] = rows![
    "physics", "true | false", "true", "Accepted runtime descriptor field; model import settings own cooked collision.";
    "dynamic", "true | false", "false", "Accepted runtime descriptor field; a cooked geometry is static source data.";
    "uvRepeat", "<u> <v>", "1 1", "Multiply source UVs."
];

pub const MATERIAL: &[Line] = rows![
    "color", "<r> <g> <b>", "1 1 1", "Linear base colour.";
    "opacity", "<0..1>", "1", "Base alpha; below one enables blending unless alphaTest is present.";
    "metalness", "<0..1>", "0", "Metallic factor.";
    "roughness", "<0..1>", "1", "Roughness factor.";
    "emissive", "<r> <g> <b>", "0 0 0", "Linear emitted colour.";
    "emissiveIntensity", "<number >= 0>", "1", "Multiply emissive, preserving values above one through the emissive-strength extension.";
    "doubleSided", "true | false", "false", "Draw both faces.";
    "alphaTest", "<0..1>", "", "Alpha mask cutoff; takes precedence over opacity blending.";
    "map", "\"<image>\"", "", "Base colour image relative to the model, not an asset handle.";
    "normalMap", "\"<image>\"", "", "Normal image relative to the model.";
    "normalScale", "<number>", "1", "Scale normal-map strength.";
    "metalnessMap", "\"<image>\"", "", "Packed roughness (green) and metalness (blue).";
    "aoMap", "\"<image>\"", "", "Occlusion in red.";
    "transmission", "<0..1>", "0", "Specular transmission.";
    "ior", "<1..3>", "", "Explicit index of refraction.";
    "thickness", "<metres >= 0>", "0", "Volume thickness when transmission is nonzero.";
    "clearcoat", "<0..1>", "0", "Clearcoat weight.";
    "clearcoatRoughness", "<0..1>", "0", "Clearcoat roughness when clearcoat is nonzero.";
    "translucency", "<0..1>", "0", "Diffuse transmission; foliage draws both faces.";
    "domeNormals", "<0..1>", "0", "Foliage normals' canopy-dome blend.";
    "season", "<0..1>", "0", "Evergreen at zero, deciduous at one.";
    "autumnColor", "<r> <g> <b>", "0.42 0.12 0.015", "Nonnegative linear seasonal colour.";
    "autumnColor2", "<r> <g> <b>", "0.25 0.025 0.015", "Second seasonal colour.";
    "seasonShift", "<days>", "0", "Shift the species' seasonal cycle within LeafSeason.MAX_SHIFT.";
    "grime", "<0..1>", "0", "Ground grime weight.";
    "grimeColor", "<r> <g> <b>", "0 0 0", "Nonnegative linear grime colour.";
    "grimeHeight", "<metres >= 0>", "0", "Ground grime extent.";
    "stains", "<0..1>", "0", "Stain variation weight.";
    "stochastic", "<0..1>", "0", "Stochastic texture sampling weight."
];

pub const BLOCKS: &[(&str, &[Line])] = &[
    (
        "",
        rows![
            "material", "\"<name>\" { ... }", "", "Named material declared before its uses.";
            "pivot", "\"<name>\" { ... }", "", "Named TRS node; parent must precede child.";
            "part", "\"<name>\" { ... }", "", "Placed mesh; one geometry block, with optional instanced at placements.";
            "plant", "\"<name>\" { ... }", "", "One generated plant per model.";
            "fracture", "{ ... }", "", "Prefracture the imported model through the shared cutting kernel.";
            "conform", "off | character | effect", "project art contract", "Art conformance category."
        ],
    ),
    (
        "part",
        rows![
            "material", "\"<name>\"", "glTF default", "Previously declared material.";
            "pivot", "\"<name>\"", "model root", "Previously declared parent pivot.";
            "at", "{ ... }", "", "Repeat placement blocks, exclusive with the part's own placement."
        ],
    ),
    (
        "pivot",
        rows![
            "parent",
            "\"<name>\"",
            "model root",
            "Previously declared pivot. Scale must be positive; mirror is not accepted."
        ],
    ),
    (
        "boxGeometry",
        rows![
            "width", "<metres >= 0>", "1", "X extent.";
            "height", "<metres >= 0>", "1", "Y extent.";
            "depth", "<metres >= 0>", "1", "Z extent."
        ],
    ),
    (
        "planeGeometry",
        rows![
            "width", "<metres >= 0>", "1", "X extent.";
            "height", "<metres >= 0>", "1", "Y extent; plane faces +Z."
        ],
    ),
    (
        "sphereGeometry",
        rows![
            "radius", "<metres >= 0>", "1", "Sphere radius.";
            "widthSegments", "<3..512>", "32", "Longitudinal segments.";
            "heightSegments", "<2..512>", "16", "Latitudinal segments."
        ],
    ),
    (
        "cylinderGeometry",
        rows![
            "radiusTop", "<metres >= 0>", "1", "Top radius.";
            "radiusBottom", "<metres >= 0>", "1", "Bottom radius.";
            "height", "<metres >= 0>", "1", "Y extent.";
            "radialSegments", "<3..65536>", "32", "Segments around the axis.";
            "heightSegments", "<1..65536>", "1", "Segments along the axis.";
            "openEnded", "true | false", "false", "Omit end caps.";
            "thetaStart", "<radians>", "0", "Starting angle.";
            "thetaLength", "<radians>", "6.283185307179586", "Arc length."
        ],
    ),
    (
        "coneGeometry",
        rows![
            "radius", "<metres >= 0>", "1", "Base radius.";
            "height", "<metres >= 0>", "1", "Y extent.";
            "radialSegments", "<3..65536>", "32", "Segments around the axis.";
            "heightSegments", "<1..65536>", "1", "Segments along the axis.";
            "openEnded", "true | false", "false", "Omit base cap."
        ],
    ),
    (
        "capsuleGeometry",
        rows![
            "radius", "<metres >= 0>", "1", "Cap radius.";
            "height", "<metres >= 0>", "1", "Distance between cap centres.";
            "capSegments", "<1..65536>", "4", "Segments of each cap.";
            "radialSegments", "<3..65536>", "8", "Segments around the axis.";
            "heightSegments", "<1..65536>", "1", "Segments along the cylinder."
        ],
    ),
    (
        "torusGeometry",
        rows![
            "radius", "<metres >= 0>", "1", "Major radius.";
            "tube", "<metres >= 0>", "0.4", "Tube radius.";
            "radialSegments", "<2..65536>", "12", "Segments around the tube.";
            "tubularSegments", "<3..65536>", "48", "Segments around the ring.";
            "arc", "<radians>", "6.283185307179586", "Ring angle."
        ],
    ),
    (
        "ringGeometry",
        rows![
            "innerRadius", "<metres >= 0>", "0.5", "Inner radius.";
            "outerRadius", "<metres >= 0>", "1", "Outer radius.";
            "thetaSegments", "<3..65536>", "32", "Angular segments.";
            "phiSegments", "<1..65536>", "1", "Radial segments.";
            "thetaStart", "<radians>", "0", "Starting angle.";
            "thetaLength", "<radians>", "6.283185307179586", "Arc length."
        ],
    ),
    (
        "circleGeometry",
        rows![
            "radius", "<metres >= 0>", "1", "Disc radius.";
            "segments", "<3..65536>", "32", "Angular segments.";
            "thetaStart", "<radians>", "0", "Starting angle.";
            "thetaLength", "<radians>", "6.283185307179586", "Arc length."
        ],
    ),
    (
        "tubeGeometry",
        rows![
            "points", "<x y z> ...", "", "At least two polyline points; exclusive with path.";
            "path", "{ ... }", "", "3D path block, exclusive with points and closed.";
            "tubularSegments", "<1..65536>", "64", "Longitudinal segments.";
            "radius", "<metres >= 0>", "1", "Tube radius.";
            "radialSegments", "<3..65536>", "8", "Cross-section segments.";
            "closed", "true | false", "false", "Close the polyline; a path owns its own closed flag."
        ],
    ),
    (
        "shapeGeometry",
        rows![
            "shape", "{ ... }", "triangle", "Repeat Shape path blocks, each with optional holes.";
            "curveSegments", "<1..4096>", "12", "Curve subdivisions."
        ],
    ),
    (
        "extrudeGeometry",
        rows![
            "shape", "{ ... }", "unit square", "Repeat Shape path blocks, each with optional holes.";
            "depth", "<metres >= 0>", "1", "Straight extrusion depth when extrudePath is absent.";
            "steps", "<1..65536>", "1", "Steps along the extrusion.";
            "bevelEnabled", "true | false", "true", "Bevel the contour.";
            "bevelThickness", "<metres >= 0>", "0.2", "Bevel depth.";
            "bevelSize", "<metres>", "bevelThickness - 0.1", "Bevel contour inset.";
            "bevelOffset", "<metres>", "0", "Bevel contour offset.";
            "bevelSegments", "<1..4096>", "3", "Bevel subdivisions.";
            "curveSegments", "<1..4096>", "12", "Shape curve subdivisions.";
            "extrudePath", "{ ... }", "", "3D path block for curved extrusion."
        ],
    ),
    (
        "latheGeometry",
        rows![
            "points", "<x y> ...", "generator profile", "At least two radial profile points.";
            "segments", "<1..65536>", "12", "Angular segments.";
            "phiStart", "<radians>", "0", "Starting angle.";
            "phiLength", "<radians>", "6.283185307179586", "Arc length."
        ],
    ),
    (
        "polyhedronGeometry",
        rows![
            "vertices", "<x y z> ...", "required", "Input polyhedron vertices.";
            "indices", "<u32> ...", "required", "Triangle indices; at least one face.";
            "radius", "<metres >= 0>", "1", "Projected radius.";
            "detail", "<0..64>", "0", "Subdivision detail."
        ],
    ),
    (
        "solid",
        rows![
            "radius", "<metres >= 0>", "1", "Projected radius.";
            "detail", "<0..64>", "0", "Subdivision detail."
        ],
    ),
    (
        "sweepGeometry",
        rows![
            "section", "{ ... }", "required", "Path2 cross-section without holes.";
            "path", "{ ... }", "required", "3D path block.";
            "steps", "<1..65536>", "64", "Path subdivisions.";
            "scale", "<t> <value>", "", "Repeat interpolation keys along the path.";
            "twist", "<t> <radians>", "", "Repeat twist keys along the path.";
            "caps", "true | false", "true", "Close the ends.";
            "curveSegments", "<1..4096>", "12", "Section curve subdivisions."
        ],
    ),
    (
        "csgGeometry",
        rows![
            "op", "union | subtract | intersect", "union", "Boolean operation.";
            "a", "{ ... }", "required", "Placed geometry operand.";
            "b", "{ ... }", "required", "Placed geometry operand."
        ],
    ),
    (
        "mergeGeometries",
        rows![
            "part",
            "{ ... }",
            "required",
            "Repeat placed geometry blocks; retain attributes and correct reflected winding."
        ],
    ),
    (
        "path",
        rows![
            "kind", "catmullRom | bezier | linear", "catmullRom", "3D curve interpolation.";
            "closed", "true | false", "false", "Close the path.";
            "tension", "<number>", "0.5", "Catmull-Rom tension.";
            "point", "<x> <y> <z>", "required", "Repeat control points in metres."
        ],
    ),
    (
        "shape",
        rows![
            "moveTo", "<x> <y>", "", "Move the contour cursor.";
            "lineTo", "<x> <y>", "", "Append a line.";
            "quadraticCurveTo", "<cx> <cy> <x> <y>", "", "Append a quadratic curve.";
            "bezierCurveTo", "<c1x> <c1y> <c2x> <c2y> <x> <y>", "", "Append a cubic curve.";
            "splineThru", "<x y> ...", "", "Append a spline through point pairs.";
            "points", "<x y> ...", "", "Set the contour from point pairs.";
            "arc", "<x> <y> <radius> <start> <end> [clockwise]", "", "Relative-centre circular arc; angles in radians, clockwise defaults false.";
            "absarc", "<x> <y> <radius> <start> <end> [clockwise]", "", "Absolute-centre circular arc.";
            "ellipse", "<x> <y> <rx> <ry> <start> <end> [clockwise] [rotation]", "", "Relative-centre ellipse; radians, false and zero for optional values.";
            "absellipse", "<x> <y> <rx> <ry> <start> <end> [clockwise] [rotation]", "", "Absolute-centre ellipse.";
            "closePath", "", "", "Close the contour.";
            "hole", "{ ... }", "", "Hole contour; only within a Shape, never nested or in a sweep section."
        ],
    ),
    ("material", MATERIAL),
    ("placement", PLACEMENT),
    (
        "plant",
        rows![
            "kind", "<kind>", "", "Optional existing plant defaults; explicit numbers override them. No implicit species selection.";
            "seed", "<u32>", "Plant default or kind", "Deterministic generator seed.";
            "growth", "deciduous | evergreen", "Plant default or kind", "Branch growth mechanism.";
            "levels", "<0..3>", "Plant default or kind", "Branch levels after the trunk.";
            "force", "<x> <y> <z> <strength>", "Plant default or kind", "Tropism direction and strength.";
            "trunk", "{ ... }", "Plant default or kind", "Trunk level parameters.";
            "branch1", "{ ... }", "Plant default or kind", "First branch level.";
            "branch2", "{ ... }", "Plant default or kind", "Second branch level.";
            "branch3", "{ ... }", "Plant default or kind", "Third branch level.";
            "leaves", "{ ... }", "none without kind", "Enable leaves or override kind leaves.";
            "flowers", "{ ... }", "none without kind", "Enable flowers or override kind flowers.";
            "material", "\"<name>\"", "generated bark material", "Previously declared wood material.";
            "barkColor", "<r> <g> <b>", "0.13 0.085 0.05 without kind", "Nonnegative linear bark colour."
        ],
    ),
    (
        "plant.level",
        rows![
            "length", "<metres >= 0>", "inherited", "Level length.";
            "radius", "<number >= 0>", "inherited", "Trunk metres, branch share of parent radius.";
            "sections", "<1..64>", "inherited", "Longitudinal rings.";
            "segments", "<3..32>", "inherited", "Ring sides.";
            "taper", "<0..1>", "inherited", "Share of radius lost by the end.";
            "gnarliness", "<radians>", "inherited", "Section wander.";
            "twist", "<radians>", "inherited", "Turn around the axis per section.";
            "children", "<0..400>", "inherited", "Child branches.";
            "angle", "<degrees>", "inherited", "Branch angle from parent; not a trunk field.";
            "start", "<0..1>", "inherited", "First child position; not a trunk field."
        ],
    ),
    (
        "plant.leaves",
        rows![
            "shape", "blade | card | cross", "blade", "Leaf mesh construction.";
            "count", "<0..2000>", "inherited", "Leaves per last-level branch.";
            "start", "<0..1>", "inherited", "First leaf position.";
            "end", "<0..1>", "inherited", "Last leaf position, at least start.";
            "angle", "<degrees>", "inherited", "Leaf angle.";
            "facing", "<0..1>", "inherited", "Sky-facing blend.";
            "rows", "<0..8>", "inherited", "Leaf rows.";
            "size", "<metres >= 0>", "inherited", "Leaf length.";
            "width", "<number >= 0>", "inherited", "Width relative to length.";
            "sizeVariance", "<0..1>", "inherited", "Length variation.";
            "color", "<r> <g> <b>", "inherited", "Nonnegative linear colour.";
            "color2", "<r> <g> <b>", "color when specified, else inherited", "Second linear colour.";
            "colorVariation", "<0..1>", "inherited", "Per-leaf variation.";
            "material", "\"<name>\"", "generated leaf material", "Previously declared material; owns translucency, domeNormals and season fields when named.";
            "translucency", "<0..1>", "0.5 without kind", "Diffuse transmission, exclusive with a named material.";
            "domeNormals", "<0..1>", "0.6 without kind", "Canopy normal blend, exclusive with a named material.";
            "season", "<0..1>", "0 without kind", "Seasonal blend, exclusive with a named material.";
            "autumnColor", "<r> <g> <b>", "inherited", "Seasonal colour, exclusive with a named material.";
            "autumnColor2", "<r> <g> <b>", "inherited", "Second seasonal colour, exclusive with a named material.";
            "seasonShift", "<days>", "0 without kind", "Season shift, exclusive with a named material."
        ],
    ),
    (
        "plant.flowers",
        rows![
            "share", "<0..1>", "inherited", "Share of last-level ends with flowers.";
            "count", "<0..200>", "inherited", "Flowers per branch.";
            "petals", "<1..32>", "inherited", "Petal count.";
            "size", "<metres >= 0>", "inherited", "Petal length.";
            "width", "<number >= 0>", "inherited", "Relative petal width.";
            "cup", "<degrees>", "inherited", "Petal cup angle.";
            "color", "<r> <g> <b>", "inherited", "Nonnegative linear petal colour.";
            "color2", "<r> <g> <b>", "color when specified, else inherited", "Second petal colour.";
            "centerColor", "<r> <g> <b>", "inherited", "Nonnegative linear flower centre colour."
        ],
    ),
];

pub fn fields(name: &str) -> Vec<Line> {
    let solid = matches!(
        name,
        "icosahedronGeometry"
            | "dodecahedronGeometry"
            | "octahedronGeometry"
            | "tetrahedronGeometry"
    );
    let level = matches!(
        name,
        "plant.trunk" | "plant.branch1" | "plant.branch2" | "plant.branch3"
    );
    let key = if solid {
        "solid"
    } else if level {
        "plant.level"
    } else {
        name
    };
    let mut fields = BLOCKS
        .iter()
        .find(|(k, _)| *k == key)
        .expect("model parser schema")
        .1
        .to_vec();
    if name == "plant.trunk" {
        fields.retain(|f| !matches!(f.key, "angle" | "start"));
    }
    if matches!(name, "part" | "pivot") {
        fields.extend_from_slice(PLACEMENT);
        if name == "pivot" {
            fields.retain(|f| f.key != "mirror");
        } else {
            fields.extend(super::GEOMETRIES.iter().map(|name| {
                Line::new(
                    name,
                    "{ ... }",
                    "",
                    "Exactly one geometry block per part.",
                    "",
                )
            }));
        }
    }
    if super::GEOMETRIES.contains(&name) {
        fields.extend_from_slice(GEOMETRY_COMMON);
    }
    fields
}
