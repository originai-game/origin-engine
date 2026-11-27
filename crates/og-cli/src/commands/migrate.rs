//! `og migrate three <file.ts> [--out <file.ts> | --write] [--json]`: a
//! three.js file rewritten towards Origin Engine with the mappings of the
//! `three-to-og` guide (`crates/og-api/guides/`, `og api show three-to-og`).
//!
//! What is mechanical is rewritten: the three.js imports to one import of
//! `og` from `@origingame/engine` (and a `must` helper), the geometry
//! constructors to their `og.render` calls with the positional arguments
//! named, the materials to `og.render.standardMaterial`, colours through
//! `color.fromHex`, vectors and colours to plain values, `Mesh`, `Group`,
//! `Object3D` and the lights declared on their own line to spawns with
//! their components, `add` and `remove` to og.Parent and despawns, the
//! transform setters to `world.set` of og.Transform, the camera to the
//! og.Camera resource, texture loads to `og.load.texture`, and the scene,
//! the renderer and what is said to them deleted.
//!
//! What is not stays as written under a `// og migrate: <what to do>
//! (three-to-og §N)` line and is listed with its line: the render loop
//! (three-to-og §8), the glTF and environment loaders (three-to-og §7),
//! the Raycaster (three-to-og §9), OrbitControls, InstancedMesh and
//! ShaderMaterial (three-to-og §10), ambient lights (three-to-og §6),
//! Euler and in-place arithmetic (three-to-og §3). The validator of the
//! knowledge package checks that every section cited here exists.
//!
//! The file is read as text, not parsed as TypeScript: a statement is a
//! line (joined while a parenthesis or bracket is open), strings and
//! comments are skipped when brackets are matched, and a pattern that does
//! not match whole is left alone and listed, never guessed at.

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::ExitCode;

mod rewrite;

const USAGE: &str = "usage: og migrate three <file.ts> [--out <file.ts> | --write] [--json]";

// The guide's sections, by what they map.
const S_MODEL: u8 = 1; // three-to-og §1
const S_GRAPH: u8 = 2; // three-to-og §2
const S_OBJECT: u8 = 3; // three-to-og §3
const S_GEOMETRY: u8 = 4; // three-to-og §4
const S_MATERIAL: u8 = 5; // three-to-og §5
const S_LIGHT: u8 = 6; // three-to-og §6
const S_LOAD: u8 = 7; // three-to-og §7
const S_TICK: u8 = 8; // three-to-og §8
const S_PICK: u8 = 9; // three-to-og §9
const S_GAPS: u8 = 10; // three-to-og §10

/// What happened to one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rewritten,
    Removed,
    Manual,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Rewritten => "rewritten",
            Kind::Removed => "removed",
            Kind::Manual => "manual",
        }
    }
}

/// One entry of the list: the line in the input, what it said, what it
/// became (or why it went, or what to do by hand), and the guide's section.
#[derive(Clone, Debug)]
pub struct Item {
    pub line: usize,
    pub kind: Kind,
    pub three: String,
    pub og: String,
    pub section: u8,
}

/// The rewritten file and the list.
pub struct Migration {
    pub output: String,
    pub items: Vec<Item>,
}

/// A geometry constructor: its og.render function, the IDL desc it fills,
/// three.js's positional parameters it carries, and those it drops.
struct Geometry {
    three: &'static str,
    og: &'static str,
    desc: &'static str,
    params: &'static [&'static str],
    dropped: &'static [&'static str],
}

const GEOMETRIES: &[Geometry] = &[
    Geometry {
        three: "BoxGeometry",
        og: "boxGeometry",
        desc: "BoxGeometryDesc",
        params: &["width", "height", "depth"],
        dropped: &["widthSegments", "heightSegments", "depthSegments"],
    },
    Geometry {
        three: "PlaneGeometry",
        og: "planeGeometry",
        desc: "PlaneGeometryDesc",
        params: &["width", "height"],
        dropped: &["widthSegments", "heightSegments"],
    },
    Geometry {
        three: "SphereGeometry",
        og: "sphereGeometry",
        desc: "SphereGeometryDesc",
        params: &["radius", "widthSegments", "heightSegments"],
        dropped: &["phiStart", "phiLength", "thetaStart", "thetaLength"],
    },
    Geometry {
        three: "CylinderGeometry",
        og: "cylinderGeometry",
        desc: "CylinderGeometryDesc",
        params: &[
            "radiusTop",
            "radiusBottom",
            "height",
            "radialSegments",
            "heightSegments",
            "openEnded",
            "thetaStart",
            "thetaLength",
        ],
        dropped: &[],
    },
    Geometry {
        three: "ConeGeometry",
        og: "coneGeometry",
        desc: "ConeGeometryDesc",
        params: &[
            "radius",
            "height",
            "radialSegments",
            "heightSegments",
            "openEnded",
        ],
        dropped: &["thetaStart", "thetaLength"],
    },
    Geometry {
        three: "CapsuleGeometry",
        og: "capsuleGeometry",
        desc: "CapsuleGeometryDesc",
        params: &[
            "radius",
            "height",
            "capSegments",
            "radialSegments",
            "heightSegments",
        ],
        dropped: &[],
    },
    Geometry {
        three: "TorusGeometry",
        og: "torusGeometry",
        desc: "TorusGeometryDesc",
        params: &["radius", "tube", "radialSegments", "tubularSegments", "arc"],
        dropped: &[],
    },
    Geometry {
        three: "RingGeometry",
        og: "ringGeometry",
        desc: "RingGeometryDesc",
        params: &[
            "innerRadius",
            "outerRadius",
            "thetaSegments",
            "phiSegments",
            "thetaStart",
            "thetaLength",
        ],
        dropped: &[],
    },
    Geometry {
        three: "CircleGeometry",
        og: "circleGeometry",
        desc: "CircleGeometryDesc",
        params: &["radius", "segments", "thetaStart", "thetaLength"],
        dropped: &[],
    },
    Geometry {
        three: "LatheGeometry",
        og: "latheGeometry",
        desc: "LatheGeometryDesc",
        params: &["points", "segments", "phiStart", "phiLength"],
        dropped: &[],
    },
    Geometry {
        three: "ShapeGeometry",
        og: "shapeGeometry",
        desc: "ShapeGeometryDesc",
        params: &["shapes", "curveSegments"],
        dropped: &[],
    },
    Geometry {
        three: "PolyhedronGeometry",
        og: "polyhedronGeometry",
        desc: "PolyhedronGeometryDesc",
        params: &["vertices", "indices", "radius", "detail"],
        dropped: &[],
    },
    Geometry {
        three: "IcosahedronGeometry",
        og: "icosahedronGeometry",
        desc: "SolidGeometryDesc",
        params: &["radius", "detail"],
        dropped: &[],
    },
    Geometry {
        three: "DodecahedronGeometry",
        og: "dodecahedronGeometry",
        desc: "SolidGeometryDesc",
        params: &["radius", "detail"],
        dropped: &[],
    },
    Geometry {
        three: "OctahedronGeometry",
        og: "octahedronGeometry",
        desc: "SolidGeometryDesc",
        params: &["radius", "detail"],
        dropped: &[],
    },
    Geometry {
        three: "TetrahedronGeometry",
        og: "tetrahedronGeometry",
        desc: "SolidGeometryDesc",
        params: &["radius", "detail"],
        dropped: &[],
    },
];

/// TubeGeometry after its curve: `points` (a CatmullRomCurve3's) or `path`.
const TUBE_PARAMS: &[&str] = &["tubularSegments", "radius", "radialSegments", "closed"];

/// StandardMaterialDesc's fields (crates/og-api/api/render.ogt): a material
/// parameter of another name is dropped and listed. A test holds this to
/// packages/engine/schema/api.schema.json.
const STANDARD_FIELDS: &[&str] = &[
    "alphaTest",
    "alphaToCoverage",
    "anisotropy",
    "anisotropyMap",
    "anisotropyRotation",
    "aoMap",
    "aoMapIntensity",
    "aoMapOffset",
    "aoMapRepeat",
    "aoMapRotation",
    "attenuationColor",
    "attenuationDistance",
    "clearcoat",
    "clearcoatMap",
    "clearcoatNormalMap",
    "clearcoatNormalScale",
    "clearcoatRoughness",
    "clearcoatRoughnessMap",
    "color",
    "dissolve",
    "dissolveEdgeColor",
    "dissolveEdgeWidth",
    "dissolveMap",
    "dissolveScale",
    "emissive",
    "emissiveIntensity",
    "emissiveMap",
    "emissiveMapOffset",
    "emissiveMapRepeat",
    "emissiveMapRotation",
    "envMapIntensity",
    "flash",
    "flashColor",
    "flatShading",
    "foliage",
    "gradientMap",
    "hair",
    "ior",
    "iridescence",
    "iridescenceIOR",
    "iridescenceMap",
    "iridescenceThicknessMap",
    "iridescenceThicknessRange",
    "irisDepth",
    "irisRadius",
    "map",
    "mapArray",
    "mapLayer",
    "mapOffset",
    "mapRepeat",
    "mapRotation",
    "metalness",
    "metalnessMap",
    "metalnessMapOffset",
    "metalnessMapRepeat",
    "metalnessMapRotation",
    "normalMap",
    "normalMapArray",
    "normalMapLayer",
    "normalMapOffset",
    "normalMapRepeat",
    "normalMapRotation",
    "normalScale",
    "opacity",
    "outlineColor",
    "outlineWidth",
    "paintDetail",
    "rimColor",
    "rimIntensity",
    "rimPower",
    "roughness",
    "roughnessMap",
    "sheen",
    "sheenColor",
    "sheenColorMap",
    "sheenRoughness",
    "sheenRoughnessMap",
    "side",
    "styled",
    "subsurface",
    "subsurfaceRadius",
    "thickness",
    "thicknessMap",
    "toon",
    "transmission",
    "transmissionMap",
    "transparent",
    "unlit",
    "weathering",
    "wireframe",
    "wireframeAngle",
    "wireframeColor",
];

/// Material fields that hold a colour: a hex number or `#rrggbb` becomes `color.fromHex`.
const COLOR_FIELDS: &[&str] = &[
    "color",
    "emissive",
    "sheenColor",
    "attenuationColor",
    "rimColor",
    "outlineColor",
    "flashColor",
    "dissolveEdgeColor",
    "wireframeColor",
];

/// Material constructors and the fields each adds before the author's.
const MATERIALS: &[(&str, &str)] = &[
    ("MeshStandardMaterial", ""),
    ("MeshPhysicalMaterial", ""),
    ("MeshBasicMaterial", "unlit: true"),
    ("MeshToonMaterial", "toon: true"),
    ("MeshLambertMaterial", "roughness: 1"),
    ("MeshPhongMaterial", ""),
];

/// A light: its component and three.js's positional parameters, the
/// dropped ones as None. `aimed` lights shine along the entity's -Z.
struct Light {
    three: &'static str,
    component: &'static str,
    params: &'static [Option<&'static str>],
    aimed: bool,
}

const LIGHTS: &[Light] = &[
    Light {
        three: "DirectionalLight",
        component: "DirectionalLight",
        params: &[Some("color"), Some("intensity")],
        aimed: true,
    },
    Light {
        three: "PointLight",
        component: "PointLight",
        params: &[Some("color"), Some("intensity"), Some("distance"), None],
        aimed: false,
    },
    Light {
        three: "SpotLight",
        component: "SpotLight",
        params: &[
            Some("color"),
            Some("intensity"),
            Some("distance"),
            Some("angle"),
            Some("penumbra"),
            None,
        ],
        aimed: true,
    },
    Light {
        three: "RectAreaLight",
        component: "RectLight",
        params: &[
            Some("color"),
            Some("intensity"),
            Some("width"),
            Some("height"),
        ],
        aimed: false,
    },
];

/// Light fields an assignment (`light.intensity = 2`) may write.
const LIGHT_FIELDS: &[&str] = &[
    "castShadow",
    "intensity",
    "distance",
    "angle",
    "penumbra",
    "width",
    "height",
];

/// og.Mesh fields an assignment on a mesh may write.
const MESH_FIELDS: &[&str] = &[
    "castShadow",
    "receiveShadow",
    "visible",
    "geometry",
    "material",
];

/// three.js classes with no mechanical rewrite: what to do instead.
fn manual_class(name: &str) -> Option<(&'static str, u8)> {
    Some(match name {
        "Clock" => (
            "the render loop's clock: per-tick logic goes in a system, where clock.getDelta() is time.dt",
            S_TICK,
        ),
        "Raycaster" => (
            "picking is og.render.raycast(world, camera.screenToRay(x, y)), over the whole world",
            S_PICK,
        ),
        "GLTFLoader" | "OBJLoader" | "FBXLoader" => (
            "load the cooked model with og.load.gltf(world, \"<asset id>\") and world.instantiate it once ready",
            S_LOAD,
        ),
        "DRACOLoader" | "KTX2Loader" | "MeshoptDecoder" => (
            "delete the decoder: the cook reads Draco, meshopt and quantised glTF and KTX2/Basis and WebP images itself, and compresses what it writes",
            S_LOAD,
        ),
        "RGBELoader" | "EXRLoader" | "CubeTextureLoader" | "PMREMGenerator" => (
            "a content/**/*.ts file.environment declaration, loaded by canonical module/content ID with og.load.environment and set as og.Environment's map",
            S_LOAD,
        ),
        "AudioLoader" | "Audio" | "PositionalAudio" | "AudioListener" => (
            "og.load.sound and og.AudioSource (`og api show audio`)",
            S_LOAD,
        ),
        "FontLoader" => (
            "og.load.font of a TTF or OTF, for og.render.text and og.ui",
            S_LOAD,
        ),
        "AnimationMixer" => (
            "og.AnimationPlayer or og.AnimationGraph on the model's instance",
            S_LOAD,
        ),
        "OrbitControls" | "MapControls" | "TrackballControls" => (
            "put og.OrbitCamera on the orbited entity and list ...og.systems.cameras in GameDesc.systems",
            S_GAPS,
        ),
        "InstancedMesh" => (
            "spawn one entity per instance with the same geometry and material: the renderer draws them instanced",
            S_GAPS,
        ),
        "ShaderMaterial" | "RawShaderMaterial" => (
            "a type shader asset in WGSL (hooks, or mode full), made with og.render.shaderMaterial",
            S_GAPS,
        ),
        "TransformControls" | "CSS2DRenderer" | "CSS3DRenderer" | "CSS2DObject" | "CSS3DObject" => {
            (
                "not in the engine: og.ui, or camera.worldToScreen for a label's place",
                S_GAPS,
            )
        }
        "AmbientLight" | "HemisphereLight" => (
            "the sky lights every surface: RenderSettings.environmentIntensity, or an og.Environment map",
            S_LIGHT,
        ),
        "Fog" | "FogExp2" => (
            "og.Fog's color, start and end (density for FogExp2), with enabled true",
            S_LIGHT,
        ),
        "EffectComposer" | "RenderPass" | "UnrealBloomPass" | "ShaderPass" | "OutputPass" => (
            "og.PostEffects' fields, or a type post asset in PostEffects.custom",
            S_LIGHT,
        ),
        "WebGLRenderTarget" => ("og.render.renderTarget, drawn by an og.CameraView", S_LIGHT),
        "Euler" => (
            "rotations are quaternions: quat.fromEuler({ x, y, z }) where it is written",
            S_OBJECT,
        ),
        "CatmullRomCurve3"
        | "CubicBezierCurve3"
        | "QuadraticBezierCurve3"
        | "LineCurve3"
        | "CurvePath" => ("a 3D curve is an og.path (og.path.create)", S_GEOMETRY),
        "TorusKnotGeometry" => (
            "no torus knot: a tubeGeometry through the knot's points",
            S_GEOMETRY,
        ),
        "TextGeometry" => ("no text geometry: og.render.text, or og.ui", S_GEOMETRY),
        "EdgesGeometry" | "WireframeGeometry" => (
            "wireframe: true on the material, or og.render.lines",
            S_GEOMETRY,
        ),
        "BufferGeometry"
        | "BufferAttribute"
        | "Float32BufferAttribute"
        | "Uint32BufferAttribute"
        | "Uint16BufferAttribute" => (
            "og.render.bufferGeometry(world, { positions, normals, uvs, colors, indices }) from the typed arrays",
            S_GEOMETRY,
        ),
        "ExtrudeBufferGeometry" => (
            "ExtrudeGeometry's old name: og.render.extrudeGeometry",
            S_GEOMETRY,
        ),
        "LineBasicMaterial" | "LineDashedMaterial" | "PointsMaterial" | "Line" | "LineLoop"
        | "LineSegments" | "Points" | "Line2" => (
            "og.render.lines or og.render.points each tick, or the og.Lines and og.Points components",
            S_MATERIAL,
        ),
        "Sprite" | "SpriteMaterial" => ("og.Sprite, or og.render.sprite each tick", S_MATERIAL),
        "MeshNormalMaterial"
        | "MeshDepthMaterial"
        | "MeshMatcapMaterial"
        | "MeshDistanceMaterial" => (
            "no such material: og.render.standardMaterial, or a type shader asset",
            S_MATERIAL,
        ),
        "Texture" | "DataTexture" | "CanvasTexture" | "VideoTexture" => (
            "textures are cooked assets loaded with og.load.texture (og.load.video for footage)",
            S_MATERIAL,
        ),
        "Matrix4" | "Matrix3" | "Box3" | "Sphere" | "Ray" | "Plane" | "Frustum" | "Vector4"
        | "Spherical" => (
            "plain values, with the mat4, box3, sphere, ray, plane and frustum pure functions",
            S_MODEL,
        ),
        _ => return None,
    })
}

/// Constructors that only make sense as a declaration of their own (`const x = new …`).
fn statement_class(name: &str) -> bool {
    matches!(
        name,
        "Mesh"
            | "Group"
            | "Object3D"
            | "Scene"
            | "WebGLRenderer"
            | "WebGPURenderer"
            | "PerspectiveCamera"
            | "OrthographicCamera"
            | "TextureLoader"
    ) || LIGHTS.iter().any(|l| l.three == name)
}

// ── lexing ─────────────────────────────────────────────────────────────────

/// The byte just past the string or comment at `i`, or None when none starts there.
fn skip_literal(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        q @ (b'"' | b'\'' | b'`') => {
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == q {
                    return Some(j + 1);
                }
                j += 1;
            }
            Some(b.len())
        }
        b'/' if b.get(i + 1) == Some(&b'/') => Some(
            b[i..]
                .iter()
                .position(|&c| c == b'\n')
                .map_or(b.len(), |p| i + p),
        ),
        b'/' if b.get(i + 1) == Some(&b'*') => Some(
            b[i + 2..]
                .windows(2)
                .position(|w| w == b"*/")
                .map_or(b.len(), |p| i + 2 + p + 2),
        ),
        _ => None,
    }
}

/// The index of the bracket closing the one at `open`.
fn close(s: &str, open: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < b.len() {
        if let Some(j) = skip_literal(b, i) {
            i = j;
            continue;
        }
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `s` split at its top-level commas, each part trimmed.
fn split_args(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let (mut out, mut depth, mut start, mut i) = (Vec::new(), 0i32, 0, 0);
    while i < b.len() {
        if let Some(j) = skip_literal(b, i) {
            i = j;
            continue;
        }
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let last = s[start..].trim();
    if !last.is_empty() {
        out.push(last.to_string());
    }
    out
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// The identifier starting at `i`, if one does (and none ends just before it).
fn ident_at(s: &str, i: usize) -> Option<&str> {
    let b = s.as_bytes();
    if i > 0 && (is_ident_byte(b[i - 1]) || b[i - 1] == b'.') {
        return None;
    }
    if !b
        .get(i)
        .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_' || *c == b'$')
    {
        return None;
    }
    let end = b[i..]
        .iter()
        .position(|&c| !is_ident_byte(c))
        .map_or(b.len(), |p| i + p);
    Some(&s[i..end])
}

fn is_ident(s: &str) -> bool {
    !s.is_empty() && ident_at(s, 0).is_some_and(|i| i.len() == s.len())
}

/// A key: value object literal's entries, or None for anything else (a spread, a method, not an object).
fn object(s: &str) -> Option<Vec<(String, String)>> {
    let s = s.trim();
    if !s.starts_with('{') || close(s, 0) != Some(s.len() - 1) {
        return None;
    }
    let mut out = Vec::new();
    for part in split_args(&s[1..s.len() - 1]) {
        if part.starts_with("...") {
            return None;
        }
        let b = part.as_bytes();
        let colon =
            (0..b.len()).find(|&i| b[i] == b':' && !part[..i].contains(['(', '{', '[', '"', '\'']));
        match colon {
            Some(c) => {
                let key = part[..c].trim().trim_matches(['"', '\'']);
                if !is_ident(key) {
                    return None;
                }
                out.push((key.to_string(), part[c + 1..].trim().to_string()));
            }
            None if is_ident(&part) => out.push((part.clone(), part)),
            None => return None,
        }
    }
    Some(out)
}

fn emit_object(pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        return "{}".into();
    }
    format!(
        "{{ {} }}",
        pairs
            .iter()
            .map(|(k, v)| if k == v {
                k.clone()
            } else {
                format!("{k}: {v}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// `name(args)` as the whole of `s`: the arguments.
fn whole_call(s: &str, name: &str) -> Option<Vec<String>> {
    let rest = s.strip_prefix(name)?.trim_start();
    if !rest.starts_with('(') || close(rest, 0) != Some(rest.len() - 1) {
        return None;
    }
    Some(split_args(&rest[1..rest.len() - 1]))
}

/// `field = value` as the whole of `s` (not `==`, not `+=`).
fn assignment(s: &str) -> Option<(&str, &str)> {
    let eq = s.find('=')?;
    let (lhs, rhs) = (s[..eq].trim(), &s[eq + 1..]);
    if rhs.starts_with('=') || !is_ident(lhs) {
        return None;
    }
    Some((lhs, rhs.trim()))
}

fn vec3(a: &[String]) -> String {
    let at = |i: usize| a.get(i).map_or("0", String::as_str).to_string();
    format!("{{ x: {}, y: {}, z: {} }}", at(0), at(1), at(2))
}

fn short(s: &str) -> String {
    let one = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 140 {
        format!("{}…", one.chars().take(139).collect::<String>())
    } else {
        one
    }
}

/// A colour literal as og's linear Color: a hex number or `#rgb`/`#rrggbb` through color.fromHex.
fn color_literal(v: &str) -> Option<String> {
    let v = v.trim();
    if v.starts_with("color.fromHex(") || v.starts_with('{') {
        return Some(v.to_string());
    }
    let hex_number =
        v.starts_with("0x") && v.len() > 2 && v[2..].bytes().all(|c| c.is_ascii_hexdigit());
    if hex_number || (!v.is_empty() && v.bytes().all(|c| c.is_ascii_digit())) {
        return Some(format!("color.fromHex({v})"));
    }
    let hex = v
        .strip_prefix(['"', '\''])
        .and_then(|x| x.strip_suffix(['"', '\'']))?
        .strip_prefix('#')?;
    if !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let full = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => hex.to_string(),
        _ => return None,
    };
    Some(format!("color.fromHex(0x{})", full.to_ascii_lowercase()))
}

/// An asset id from a texture URL: the leading `./`, `/`, `public/` or `assets/` removed.
fn asset_id(literal: &str) -> Option<String> {
    let q = literal.chars().next()?;
    let inner = literal.strip_prefix(['"', '\''])?.strip_suffix(q)?;
    let mut id = inner;
    for p in ["./", "/", "public/", "assets/"] {
        id = id.strip_prefix(p).unwrap_or(id);
    }
    Some(format!("\"{id}\""))
}

// ── the rewrite ────────────────────────────────────────────────────────────

#[derive(Clone)]
enum Var {
    Scene,
    Renderer,
    Camera,
    Object,
    Mesh,
    Light(&'static Light),
    TextureLoader,
    Manual(&'static str, u8),
}

/// A note an expression leaves on its line.
struct Note {
    kind: Kind,
    text: String,
    section: u8,
}

#[derive(Default)]
struct Ctx {
    /// `THREE` of `import * as THREE from "three"`.
    alias: Option<String>,
    /// Names imported from three (and its addons) by name.
    named: HashSet<String>,
    vars: HashMap<String, Var>,
    camera: Option<String>,
}

/// Logical lines: physical lines joined while a parenthesis or bracket is
/// open, each with its first line's number (1-based).
fn logical_lines(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut first = 0;
    for (n, line) in src.split('\n').enumerate() {
        if cur.is_empty() {
            first = n + 1;
        } else {
            cur.push('\n');
        }
        cur.push_str(line);
        if depth(&cur) <= 0 {
            out.push((first, std::mem::take(&mut cur)));
        }
    }
    if !cur.is_empty() {
        out.push((first, cur));
    }
    out
}

fn depth(s: &str) -> i32 {
    let b = s.as_bytes();
    let (mut d, mut i) = (0, 0);
    while i < b.len() {
        if let Some(j) = skip_literal(b, i) {
            i = j;
            continue;
        }
        match b[i] {
            b'(' | b'[' => d += 1,
            b')' | b']' => d -= 1,
            _ => {}
        }
        i += 1;
    }
    d
}

/// The module an import line reads, when it is three.js or one of its addon paths.
fn three_import(t: &str) -> bool {
    if !t.starts_with("import") {
        return false;
    }
    let Some(from) = t.rfind("from") else {
        return false;
    };
    let module = t[from + 4..]
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_matches(['"', '\'']);
    module == "three" || module.starts_with("three/")
}

/// What one statement became.
enum Done {
    /// Rewritten to these lines.
    Lines(Vec<String>, u8),
    /// Deleted, and why.
    Removed(String, u8),
    /// Left as written, with what to do.
    Manual(String, u8),
}

fn euler(a: &[String]) -> String {
    let xyz = vec3(a);
    match a.get(3) {
        Some(order) => format!("{xyz}, {order}"),
        None => xyz,
    }
}

/// The whole rewrite of one file's text. None when it imports nothing from three.
pub fn migrate(src: &str) -> Option<Migration> {
    let mut ctx = Ctx::default();
    let mut items = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut engine_import_at = None;
    let mut last_import_at = None;
    let mut any_three = false;
    for (line, raw) in logical_lines(src) {
        let indent: String = raw
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        let t = raw.trim();
        let mut notes = Vec::new();
        let done = ctx.statement(t, &mut notes);
        if three_import(t) {
            any_three = true;
            engine_import_at.get_or_insert(out.len());
        } else if t.starts_with("import ") {
            last_import_at = Some(out.len());
        }
        let three = short(t);
        let mut comments = Vec::new();
        let mut body: Vec<String> = Vec::new();
        match done {
            Some(Done::Lines(lines, _)) if lines.is_empty() => body.push(raw.clone()),
            Some(Done::Lines(lines, section)) => {
                items.push(Item {
                    line,
                    kind: Kind::Rewritten,
                    three: three.clone(),
                    og: short(&lines.join(" ")),
                    section,
                });
                body.extend(lines.into_iter().map(|l| format!("{indent}{l}")));
            }
            Some(Done::Removed(why, section)) => items.push(Item {
                line,
                kind: Kind::Removed,
                three: three.clone(),
                og: why,
                section,
            }),
            Some(Done::Manual(what, section)) => {
                comments.push((what.clone(), section));
                items.push(Item {
                    line,
                    kind: Kind::Manual,
                    three: three.clone(),
                    og: what,
                    section,
                });
                body.push(raw.clone());
            }
            None => {
                let rewritten = ctx.expr(&raw, &mut notes);
                for (name, var) in &ctx.vars {
                    if let Var::Manual(what, section) = var
                        && contains_word(t, name)
                        && !notes.iter().any(|n| n.text == *what)
                    {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: what.to_string(),
                            section: *section,
                        });
                    }
                }
                for (word, what, section) in [
                    (
                        "requestAnimationFrame",
                        "the render loop: its body goes in systems on the tick, and the engine draws every frame",
                        S_TICK,
                    ),
                    (
                        "addEventListener",
                        "events reach the tick as og.Input (input.justPressed(\"Space\")), read in a system; the engine handles resizing",
                        S_TICK,
                    ),
                ] {
                    if contains_word(t, word) {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: what.into(),
                            section,
                        });
                    }
                }
                if rewritten != raw {
                    items.push(Item {
                        line,
                        kind: Kind::Rewritten,
                        three: three.clone(),
                        og: short(rewritten.trim()),
                        section: section_of(&rewritten),
                    });
                }
                body.push(rewritten);
            }
        }
        for n in notes {
            if n.kind == Kind::Manual {
                comments.push((n.text.clone(), n.section));
            }
            items.push(Item {
                line,
                kind: n.kind,
                three: three.clone(),
                og: n.text,
                section: n.section,
            });
        }
        comments.dedup();
        for (what, section) in comments {
            out.push(format!(
                "{indent}// og migrate: {what} (three-to-og §{section})"
            ));
        }
        out.extend(body);
    }
    if !any_three {
        return None;
    }
    let text = out.join("\n");
    let mut names = vec!["og"];
    if text.contains("color.fromHex(") {
        names.push("color");
    }
    if text.contains("quat.") {
        names.push("quat");
    }
    let import = format!(
        "import {{ {} }} from \"@origingame/engine\";",
        names.join(", ")
    );
    let must = text.contains("must(") && !src.contains("function must");
    let helper = "\nfunction must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {\n  if (!r.ok) throw new Error(r.reason);\n  return r.value;\n}\n";
    // The engine import where three's first was; the helper after the last import.
    let at = engine_import_at.unwrap_or(0);
    let mut lines = out;
    if must {
        let helper_at = last_import_at.filter(|l| *l >= at).map_or(at, |l| l + 1);
        lines.insert(helper_at, helper.to_string());
    }
    lines.insert(at, import);
    let mut output = lines.join("\n");
    if !output.ends_with('\n') && src.ends_with('\n') {
        output.push('\n');
    }
    Some(Migration { output, items })
}

/// The section a rewritten expression line belongs to, by what it calls.
fn section_of(s: &str) -> u8 {
    if s.contains("Material(") {
        S_MATERIAL
    } else if s.contains("og.load.") {
        S_LOAD
    } else if s.contains("Geometry(") || s.contains("og.Shape") || s.contains("og.Path2") {
        S_GEOMETRY
    } else {
        S_MODEL
    }
}

fn contains_word(s: &str, w: &str) -> bool {
    let b = s.as_bytes();
    s.match_indices(w).any(|(i, _)| {
        (i == 0 || !is_ident_byte(b[i - 1]))
            && b.get(i + w.len()).is_none_or(|c| !is_ident_byte(*c))
    })
}

// ── the command ────────────────────────────────────────────────────────────

pub fn run(args: &[&str]) -> ExitCode {
    let [kind, rest @ ..] = args else {
        eprintln!("og migrate: {USAGE}");
        return ExitCode::from(2);
    };
    if *kind != "three" {
        eprintln!("og migrate: `{kind}`: only `three` (a three.js file) is migrated. {USAGE}");
        return ExitCode::from(2);
    }
    let (mut file, mut out, mut write, mut json) = (None, None, false, false);
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--write" => write = true,
            "--json" => json = true,
            "--out" => {
                let Some(v) = rest.get(i + 1) else {
                    eprintln!("og migrate: --out needs a path. {USAGE}");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(v));
                i += 1;
            }
            f if f.starts_with('-') => {
                eprintln!("og migrate: unknown option `{f}`. {USAGE}");
                return ExitCode::from(2);
            }
            f if file.is_none() => file = Some(PathBuf::from(f)),
            f => {
                eprintln!("og migrate: one file at a time (`{f}` too). {USAGE}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(file) = file else {
        eprintln!("og migrate: no file. {USAGE}");
        return ExitCode::from(2);
    };
    if write && out.is_some() {
        eprintln!("og migrate: --out or --write, not both. {USAGE}");
        return ExitCode::from(2);
    }
    let src = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og migrate: {}: {e}", file.display());
            return ExitCode::FAILURE;
        }
    };
    let Some(m) = migrate(&src) else {
        eprintln!(
            "og migrate: {}: imports nothing from three (`import * as THREE from \"three\"` or named imports); nothing to migrate",
            file.display()
        );
        return ExitCode::FAILURE;
    };
    let target = if write { Some(file.clone()) } else { out };
    if let Some(t) = &target
        && let Err(e) = std::fs::write(t, &m.output)
    {
        eprintln!("og migrate: {}: {e}", t.display());
        return ExitCode::FAILURE;
    }
    let count = |k: Kind| m.items.iter().filter(|i| i.kind == k).count();
    if json {
        let mut v = json!({
            "event": "migrate",
            "file": file.display().to_string(),
            "out": target.as_ref().map(|t| t.display().to_string()),
            "rewritten": count(Kind::Rewritten),
            "removed": count(Kind::Removed),
            "manual": count(Kind::Manual),
            "items": m.items.iter().map(|i| json!({ "line": i.line, "kind": i.kind.as_str(), "three": i.three, "og": i.og, "section": i.section })).collect::<Vec<_>>(),
        });
        if target.is_none() {
            v["output"] = json!(m.output);
        }
        println!("{v}");
        return ExitCode::SUCCESS;
    }
    let mut report = format!(
        "og migrate: {}: {} rewritten, {} removed, {} to do by hand{}\n",
        file.display(),
        count(Kind::Rewritten),
        count(Kind::Removed),
        count(Kind::Manual),
        target
            .as_ref()
            .map_or(String::new(), |t| format!(" -> {}", t.display()))
    );
    for i in m.items.iter().filter(|i| i.kind == Kind::Manual) {
        report.push_str(&format!(
            "  {}: {} (three-to-og §{})\n      {}\n",
            i.line, i.og, i.section, i.three
        ));
    }
    report.push_str("next: move the setup into GameDesc.start(world) and the loop into systems, then `og check`\n");
    if target.is_some() {
        print!("{report}");
    } else {
        print!("{}", m.output);
        eprint!("{report}");
    }
    ExitCode::SUCCESS
}
