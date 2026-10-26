//! `og migrate three <file.ts> [--out <file.ts> | --write] [--json]`: a
//! three.js file rewritten towards Origin Engine with the mappings of the
//! knowledge package's `three-to-og` guide (`packages/knowledge/guides/`).
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

/// StandardMaterialDesc's fields (crates/og-api/api.ogt): a material
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
    "envMapIntensity",
    "flash",
    "flashColor",
    "flatShading",
    "gradientMap",
    "ior",
    "iridescence",
    "iridescenceIOR",
    "iridescenceMap",
    "iridescenceThicknessMap",
    "iridescenceThicknessRange",
    "map",
    "mapArray",
    "mapLayer",
    "metalness",
    "metalnessMap",
    "normalMap",
    "normalMapArray",
    "normalMapLayer",
    "normalScale",
    "opacity",
    "outlineColor",
    "outlineWidth",
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
    "thickness",
    "thicknessMap",
    "toon",
    "transmission",
    "transmissionMap",
    "transparent",
    "unlit",
    "wireframe",
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
            "the cook compresses: give it the uncompressed glTF and delete the decoder",
            S_LOAD,
        ),
        "RGBELoader" | "EXRLoader" | "CubeTextureLoader" | "PMREMGenerator" => (
            "an .ogt of type environment, loaded with og.load.environment and set as og.Environment's map",
            S_LOAD,
        ),
        "AudioLoader" | "Audio" | "PositionalAudio" | "AudioListener" => (
            "og.load.sound and og.AudioSource (the og-audio skill)",
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

impl Ctx {
    /// The three.js class or constant named at `i` (`THREE.Mesh`, or `Mesh` imported by name) and where it ends.
    fn three_name(&self, s: &str, i: usize) -> Option<(String, usize)> {
        let head = ident_at(s, i)?;
        let end = i + head.len();
        if Some(head) == self.alias.as_deref() && s.as_bytes().get(end) == Some(&b'.') {
            let name = ident_at(&s[end + 1..], 0)?;
            return Some((name.to_string(), end + 1 + name.len()));
        }
        self.named.contains(head).then(|| (head.to_string(), end))
    }

    /// `new <three class>(args)` as the whole of `s`: the class and its raw arguments.
    fn new_call(&self, s: &str) -> Option<(String, Vec<String>)> {
        let s = s.trim();
        let rest = s.strip_prefix("new")?;
        if !rest.starts_with(char::is_whitespace) {
            return None;
        }
        let at = s.len() - rest.trim_start().len();
        let (name, end) = self.three_name(s, at)?;
        let tail = s[end..].trim_start();
        if tail.is_empty() {
            return Some((name, Vec::new()));
        }
        let open = s.len() - tail.len();
        (close(s, open) == Some(s.len() - 1)).then(|| (name, split_args(&s[open + 1..s.len() - 1])))
    }

    /// Every three.js value inside `s` rewritten: constructors of values,
    /// geometries and materials, constants, MathUtils, texture loads.
    fn expr(&mut self, s: &str, notes: &mut Vec<Note>) -> String {
        let b = s.as_bytes();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut copied = 0;
        while i < b.len() {
            if let Some(j) = skip_literal(b, i) {
                i = j;
                continue;
            }
            // `new X(...)`
            if s[i..].starts_with("new") && ident_at(s, i) == Some("new") {
                let at = i + 3 + s[i + 3..].len() - s[i + 3..].trim_start().len();
                if let Some((name, end)) = self.three_name(s, at) {
                    let tail = &s[end..];
                    let open = end + tail.len() - tail.trim_start().len();
                    let shut = if b.get(open) == Some(&b'(') {
                        close(s, open)
                    } else {
                        None
                    };
                    let args = shut.map_or_else(Vec::new, |q| split_args(&s[open + 1..q]));
                    let after = shut.map_or(end, |q| q + 1);
                    // new THREE.TextureLoader().load("…")
                    if name == "TextureLoader" && s[after..].starts_with(".load(") {
                        let lo = after + ".load".len();
                        if let Some(lq) = close(s, lo) {
                            if let Some(r) = self.texture_load(&s[lo + 1..lq], notes) {
                                out.push_str(&s[copied..i]);
                                out.push_str(&r);
                                i = lq + 1;
                                copied = i;
                            } else {
                                i = end;
                            }
                            continue;
                        }
                    }
                    if let Some(r) = self.value(&name, &args, notes) {
                        out.push_str(&s[copied..i]);
                        out.push_str(&r);
                        i = after;
                        copied = i;
                        continue;
                    }
                    if statement_class(&name) {
                        notes.push(Note { kind: Kind::Manual, text: format!("a {name} that is not a declaration of its own: declare it `const x = new {name}(…)` on its own line, or spawn it by hand"), section: S_GRAPH });
                    } else if let Some((what, section)) = manual_class(&name) {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!("{name}: {what}"),
                            section,
                        });
                    } else if !matches!(name.as_str(), "TubeGeometry" | "ExtrudeGeometry" | "Color")
                    {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!("no og mapping for {name}"),
                            section: S_MODEL,
                        });
                    }
                    i = end;
                    continue;
                }
            }
            if let Some(word) = ident_at(s, i) {
                // loader.load("…") on a TextureLoader variable.
                if matches!(self.vars.get(word), Some(Var::TextureLoader))
                    && s[i + word.len()..].starts_with(".load(")
                {
                    let lo = i + word.len() + ".load".len();
                    if let Some(lq) = close(s, lo)
                        && let Some(r) = self.texture_load(&s[lo + 1..lq], notes)
                    {
                        out.push_str(&s[copied..i]);
                        out.push_str(&r);
                        i = lq + 1;
                        copied = i;
                        continue;
                    }
                }
                if word == "Math" && s[i..].starts_with("Math.random(") {
                    notes.push(Note { kind: Kind::Manual, text: "Math.random() is og.Rng (world.resource(og.Rng)), seeded so a replay repeats".into(), section: S_TICK });
                }
                if let Some((name, end)) = self.three_name(s, i) {
                    let replaced = match name.as_str() {
                        "FrontSide" => Some((end, "\"front\"".to_string())),
                        "BackSide" => Some((end, "\"back\"".to_string())),
                        "DoubleSide" => Some((end, "\"double\"".to_string())),
                        "MathUtils" => [
                            (".degToRad(", " * Math.PI / 180)"),
                            (".radToDeg(", " * 180 / Math.PI)"),
                        ]
                        .iter()
                        .find_map(|(m, tail)| {
                            let open = end + m.len() - 1;
                            s[end..].starts_with(m).then_some(())?;
                            let q = close(s, open)?;
                            let inner = self.expr(&s[open + 1..q], notes);
                            Some((q + 1, format!("(({inner}){tail}")))
                        }),
                        _ => None,
                    };
                    if let Some((to, text)) = replaced {
                        out.push_str(&s[copied..i]);
                        out.push_str(&text);
                        i = to;
                        copied = i;
                        continue;
                    }
                    if name != "MathUtils" {
                        let what = manual_class(&name).map_or_else(
                            || (format!("no og mapping for {name}"), S_MODEL),
                            |(w, sec)| (format!("{name}: {w}"), sec),
                        );
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: what.0,
                            section: what.1,
                        });
                    }
                    i = end;
                    continue;
                }
                i += word.len();
                continue;
            }
            i += 1;
        }
        out.push_str(&s[copied..]);
        out
    }

    fn texture_load(&mut self, args: &str, notes: &mut Vec<Note>) -> Option<String> {
        let a = split_args(args);
        match (a.len(), a.first().and_then(|x| asset_id(x))) {
            (1, Some(id)) => Some(format!("must(og.load.texture(world, {id}))")),
            _ => {
                notes.push(Note { kind: Kind::Manual, text: "a texture load with a callback or a computed URL: og.load.texture(world, \"<asset id>\") answers the handle at once".into(), section: S_LOAD });
                None
            }
        }
    }

    /// A colour argument: a literal through color.fromHex, else as written with a note.
    fn color_arg(&mut self, v: &str, notes: &mut Vec<Note>) -> String {
        let v = self.expr(v, notes);
        color_literal(&v).unwrap_or_else(|| {
            notes.push(Note { kind: Kind::Manual, text: format!("the colour `{}` is not a hex literal: give og a linear {{ r, g, b }}, or color.fromHex of its sRGB hex", short(&v)), section: S_MATERIAL });
            v
        })
    }

    /// A value constructor rewritten, or None when `name` is not one.
    fn value(&mut self, name: &str, raw: &[String], notes: &mut Vec<Note>) -> Option<String> {
        let mut args = |c: &mut Self| raw.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        Some(match name {
            "Vector2" => {
                let a = args(self);
                format!(
                    "{{ x: {}, y: {} }}",
                    a.first().map_or("0", String::as_str),
                    a.get(1).map_or("0", String::as_str)
                )
            }
            "Vector3" => vec3(&args(self)),
            "Quaternion" => {
                let a = args(self);
                let at = |i: usize, d: &'static str| a.get(i).map_or(d, String::as_str).to_string();
                format!(
                    "{{ x: {}, y: {}, z: {}, w: {} }}",
                    at(0, "0"),
                    at(1, "0"),
                    at(2, "0"),
                    at(3, "1")
                )
            }
            "Color" => match raw.len() {
                0 => "{ r: 1, g: 1, b: 1 }".into(),
                3 => {
                    let a = args(self);
                    format!("{{ r: {}, g: {}, b: {} }}", a[0], a[1], a[2])
                }
                _ => {
                    let mut n = Vec::new();
                    let v = self.color_arg(&raw[0], &mut n);
                    if !n.is_empty() {
                        notes.extend(n);
                        return None;
                    }
                    v
                }
            },
            "Shape" => format!("new og.Shape({})", args(self).join(", ")),
            "Path" => format!("new og.Path2({})", args(self).join(", ")),
            "ExtrudeGeometry" => {
                let a = args(self);
                let shapes = a.first().cloned().unwrap_or_else(|| "[]".into());
                let desc = match a.get(1).map(|o| (o, object(o))) {
                    None => format!("{{ shapes: {shapes} }}"),
                    Some((_, Some(mut pairs))) => {
                        if let Some(p) = pairs.iter().position(|(k, _)| k == "UVGenerator") {
                            pairs.remove(p);
                            notes.push(Note { kind: Kind::Manual, text: "ExtrudeGeometry's UVGenerator is not carried over; it was dropped".into(), section: S_GEOMETRY });
                        }
                        pairs.insert(0, ("shapes".into(), shapes));
                        emit_object(&pairs)
                    }
                    Some((o, None)) => format!("{{ shapes: {shapes}, ...{o} }}"),
                };
                format!("must(og.render.extrudeGeometry(world, {desc}))")
            }
            "TubeGeometry" => {
                let curve = raw.first().and_then(|c| self.new_call(c));
                let Some((cname, cargs)) =
                    curve.filter(|(n, a)| n == "CatmullRomCurve3" && !a.is_empty())
                else {
                    notes.push(Note { kind: Kind::Manual, text: "a tube along a curve other than new CatmullRomCurve3(points): write the curve as an og.path and pass it as `path`".into(), section: S_GEOMETRY });
                    return None;
                };
                debug_assert_eq!(cname, "CatmullRomCurve3");
                let mut pairs = vec![("points".to_string(), self.expr(&cargs[0], notes))];
                if let Some(closed) = cargs.get(1) {
                    pairs.push(("closed".into(), self.expr(closed, notes)));
                }
                for (k, v) in TUBE_PARAMS.iter().zip(raw.iter().skip(1)) {
                    if v != "undefined"
                        && !(*k == "closed" && pairs.iter().any(|(p, _)| p == "closed"))
                    {
                        pairs.push((k.to_string(), self.expr(v, notes)));
                    }
                }
                format!(
                    "must(og.render.tubeGeometry(world, {}))",
                    emit_object(&pairs)
                )
            }
            _ => {
                if let Some(g) = GEOMETRIES.iter().find(|g| g.three == name) {
                    let a = args(self);
                    let pairs: Vec<(String, String)> = g
                        .params
                        .iter()
                        .zip(&a)
                        .filter(|(_, v)| v.as_str() != "undefined")
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect();
                    if a.len() > g.params.len() {
                        let gone = &g.dropped[..(a.len() - g.params.len()).min(g.dropped.len())];
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!(
                                "{name}'s {} not in {}; the argument was dropped",
                                gone.join(", ") + if gone.len() == 1 { " is" } else { " are" },
                                g.desc
                            ),
                            section: S_GEOMETRY,
                        });
                    }
                    return Some(format!(
                        "must(og.render.{}(world, {}))",
                        g.og,
                        emit_object(&pairs)
                    ));
                }
                let (_, extra) = MATERIALS.iter().find(|(m, _)| *m == name)?;
                return Some(self.material(name, extra, raw.first().map(String::as_str), notes));
            }
        })
    }

    fn material(
        &mut self,
        name: &str,
        extra: &str,
        params: Option<&str>,
        notes: &mut Vec<Note>,
    ) -> String {
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Some((k, v)) = extra.split_once(": ") {
            pairs.push((k.into(), v.into()));
        }
        let Some(params) = params else {
            return format!(
                "must(og.render.standardMaterial(world, {}))",
                emit_object(&pairs)
            );
        };
        let Some(entries) = object(params) else {
            notes.push(Note { kind: Kind::Manual, text: format!("{name}'s parameters are passed as a value: check its keys against StandardMaterialDesc and its colours (linear)"), section: S_MATERIAL });
            let p = self.expr(params, notes);
            pairs.insert(0, (format!("...{p}"), format!("...{p}")));
            return format!(
                "must(og.render.standardMaterial(world, {}))",
                emit_object(&pairs)
            );
        };
        for (k, v) in entries {
            if k == "vertexColors" {
                notes.push(Note { kind: Kind::Removed, text: "vertexColors: the geometry's colors multiply the colour whenever it has them".into(), section: S_MATERIAL });
                continue;
            }
            if !STANDARD_FIELDS.contains(&k.as_str()) {
                notes.push(Note {
                    kind: Kind::Manual,
                    text: format!(
                        "{name}'s `{k}` has no field in StandardMaterialDesc; it was dropped"
                    ),
                    section: S_MATERIAL,
                });
                continue;
            }
            let v = if COLOR_FIELDS.contains(&k.as_str()) {
                self.color_arg(&v, notes)
            } else {
                self.expr(&v, notes)
            };
            pairs.retain(|(p, _)| *p != k);
            pairs.push((k, v));
        }
        format!(
            "must(og.render.standardMaterial(world, {}))",
            emit_object(&pairs)
        )
    }
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

impl Ctx {
    /// The statement handlers: a three.js import, a declaration, a statement on a known variable.
    fn statement(&mut self, t: &str, notes: &mut Vec<Note>) -> Option<Done> {
        let body = t.trim_end_matches(';').trim_end();
        if three_import(t) {
            let clause = t["import".len()..t.rfind("from").unwrap_or(t.len())].trim();
            if let Some(alias) = clause.strip_prefix("* as ") {
                self.alias = Some(alias.trim().to_string());
            } else if let (Some(a), Some(z)) = (clause.find('{'), clause.rfind('}')) {
                for n in clause[a + 1..z].split(',') {
                    let n = n.trim().rsplit(" as ").next().unwrap_or("").trim();
                    if is_ident(n) {
                        self.named.insert(n.to_string());
                    }
                }
            }
            return Some(Done::Removed(
                "three.js's import: og comes from @origingame/engine".into(),
                S_MODEL,
            ));
        }
        if let Some(d) = self.declaration(body, notes) {
            return Some(d);
        }
        if body.contains("appendChild(")
            && self.vars.iter().any(|(n, v)| {
                matches!(v, Var::Renderer) && body.contains(&format!("{n}.domElement"))
            })
        {
            return Some(Done::Removed("the engine owns the canvas".into(), S_LIGHT));
        }
        if body.ends_with(".dispose()") && is_ident(body.split('.').next().unwrap_or("")) {
            return Some(Done::Removed(
                "no dispose: the world owns geometries, materials and textures".into(),
                S_GAPS,
            ));
        }
        let head = ident_at(body, 0)?;
        let rest = body[head.len()..].strip_prefix('.')?;
        let var = self.vars.get(head)?.clone();
        Some(self.member(head, &var, rest, notes))
    }

    fn declaration(&mut self, body: &str, notes: &mut Vec<Note>) -> Option<Done> {
        let kw = ["const ", "let ", "var "]
            .into_iter()
            .find(|k| body.starts_with(k))?;
        let after = &body[kw.len()..];
        let name = ident_at(after, 0)?;
        let eq = after.find('=')?;
        let lhs = after[..eq].trim();
        if !(lhs == name || lhs.starts_with(&format!("{name}:"))) {
            return None;
        }
        let (class, raw) = self.new_call(&after[eq + 1..])?;
        let kw = kw.trim_end();
        let spawn = |n: &str| format!("{kw} {n} = must(world.spawn({{ name: \"{n}\" }}));");
        let mut args = |c: &mut Self| raw.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        let done = match class.as_str() {
            "Scene" => {
                self.vars.insert(name.into(), Var::Scene);
                Done::Removed(
                    "the world is the scene: every entity spawned is in it".into(),
                    S_GRAPH,
                )
            }
            "WebGLRenderer" | "WebGPURenderer" => {
                self.vars.insert(name.into(), Var::Renderer);
                Done::Removed(
                    "the engine draws every frame; antialiasing is GameDesc.render".into(),
                    S_LIGHT,
                )
            }
            "TextureLoader" => {
                self.vars.insert(name.into(), Var::TextureLoader);
                Done::Removed(
                    "no loader object: og.load.texture loads a cooked texture by its asset id"
                        .into(),
                    S_LOAD,
                )
            }
            "Group" | "Object3D" => {
                self.vars.insert(name.into(), Var::Object);
                Done::Lines(vec![spawn(name)], S_GRAPH)
            }
            "Mesh" => {
                let a = args(self);
                self.vars.insert(name.into(), Var::Mesh);
                let mut fields = Vec::new();
                for (k, v) in ["geometry", "material"].iter().zip(&a) {
                    fields.push((k.to_string(), v.clone()));
                }
                Done::Lines(
                    vec![
                        spawn(name),
                        format!("world.insert({name}, og.Mesh, {});", emit_object(&fields)),
                    ],
                    S_GRAPH,
                )
            }
            "PerspectiveCamera" | "OrthographicCamera" => {
                let a = args(self);
                self.vars.insert(name.into(), Var::Camera);
                self.camera = Some(name.into());
                let mut lines = vec![format!("{kw} {name} = must(world.resource(og.Camera));")];
                let set = |k: &str, v: Option<&String>, lines: &mut Vec<String>| {
                    if let Some(v) = v.filter(|v| v.as_str() != "undefined") {
                        lines.push(format!("{name}.{k} = {v};"));
                    }
                };
                if class == "PerspectiveCamera" {
                    set("fov", a.first(), &mut lines);
                    set("near", a.get(2), &mut lines);
                    set("far", a.get(3), &mut lines);
                } else {
                    lines.push(format!("{name}.orthographic = true;"));
                    if let (Some(top), Some(bottom)) = (a.get(2), a.get(3)) {
                        lines.push(format!("{name}.height = ({top}) - ({bottom});"));
                    }
                    set("near", a.get(4), &mut lines);
                    set("far", a.get(5), &mut lines);
                }
                Done::Lines(lines, S_LIGHT)
            }
            _ => {
                if let Some(light) = LIGHTS.iter().find(|l| l.three == class) {
                    self.vars.insert(name.into(), Var::Light(light));
                    let mut fields = Vec::new();
                    for (k, v) in light.params.iter().zip(&raw) {
                        if v == "undefined" {
                            continue;
                        }
                        match k {
                            Some("color") => fields.push(("color".to_string(), self.color_arg(v, notes))),
                            Some(k) => fields.push((k.to_string(), self.expr(v, notes))),
                            None => notes.push(Note { kind: Kind::Manual, text: format!("{class}'s decay is not carried over (og's falloff is fixed); it was dropped"), section: S_LIGHT }),
                        }
                    }
                    Done::Lines(
                        vec![
                            spawn(name),
                            format!(
                                "world.insert({name}, og.{}, {});",
                                light.component,
                                emit_object(&fields)
                            ),
                        ],
                        S_LIGHT,
                    )
                } else if let Some((what, section)) = manual_class(&class) {
                    self.vars.insert(name.into(), Var::Manual(what, section));
                    Done::Manual(format!("{class}: {what}"), section)
                } else {
                    return None; // a value constructor: the expression pass rewrites it
                }
            }
        };
        Some(done)
    }

    /// A statement `name.rest` on a variable the file declared.
    fn member(&mut self, name: &str, var: &Var, rest: &str, notes: &mut Vec<Note>) -> Done {
        let mut ex =
            |c: &mut Self, v: &[String]| v.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        let transform = |field: &str| format!("world.set({name}, og.Transform, {{ {field} }});");
        match var {
            Var::Manual(what, section) => Done::Manual(what.to_string(), *section),
            Var::TextureLoader => Done::Manual(
                "no loader object: og.load.texture(world, \"<asset id>\")".into(),
                S_LOAD,
            ),
            Var::Scene => {
                if let Some(a) = whole_call(rest, "add") {
                    if a.iter().all(|c| {
                        matches!(
                            self.vars.get(c.as_str()),
                            Some(Var::Object | Var::Mesh | Var::Light(_) | Var::Camera)
                        )
                    }) {
                        return Done::Removed(
                            "the world holds every entity: nothing is added to a scene".into(),
                            S_GRAPH,
                        );
                    }
                    let before = notes.len();
                    for c in &a {
                        self.expr(c, notes);
                    }
                    if notes.len() == before {
                        notes.push(Note { kind: Kind::Manual, text: format!("`{}` is not an entity this file spawns: spawn it, or instantiate the loaded model", short(&a.join(", "))), section: S_GRAPH });
                    }
                    return Done::Lines(Vec::new(), 0);
                }
                if let Some(a) = whole_call(rest, "remove") {
                    let a = ex(self, &a);
                    return Done::Lines(
                        a.iter().map(|c| format!("world.despawn({c});")).collect(),
                        S_GRAPH,
                    );
                }
                if let Some(("background", v)) = assignment(rest) {
                    let c = self.color_arg(v, notes);
                    let cam = self
                        .camera
                        .clone()
                        .unwrap_or_else(|| "must(world.resource(og.Camera))".into());
                    return Done::Lines(vec![format!("{cam}.background = {c};")], S_LIGHT);
                }
                match assignment(rest) {
                    Some(("fog", _)) => Done::Manual("og.Fog's color, start and end (density for FogExp2), with enabled true".into(), S_LIGHT),
                    Some(("environment", _)) => Done::Manual("og.Environment's map, from og.load.environment of a type environment asset".into(), S_MATERIAL),
                    _ => Done::Manual(format!("no og mapping for the scene's `{}`", short(rest)), S_GRAPH),
                }
            }
            Var::Renderer => {
                let head = rest.split(['(', '.', ' ', '=']).next().unwrap_or("");
                match head {
                    "setAnimationLoop" => Done::Manual("the render loop: its body goes in systems on the tick, and the engine draws every frame".into(), S_TICK),
                    "toneMapping" | "toneMappingExposure" => Done::Manual("RenderSettings.toneMapping (\"aces\", \"agx\", \"none\") and exposure".into(), S_LIGHT),
                    "setClearColor" => Done::Manual("the clear colour is og.Camera's background (linear: color.fromHex of the hex)".into(), S_LIGHT),
                    "render" | "setSize" | "setPixelRatio" | "shadowMap" | "outputColorSpace" | "outputEncoding" | "physicallyCorrectLights" | "useLegacyLights" => {
                        Done::Removed("the engine owns the renderer, its size and its shadows".into(), S_LIGHT)
                    }
                    _ => Done::Manual(format!("no og mapping for the renderer's `{}`", short(rest)), S_LIGHT),
                }
            }
            Var::Camera => {
                if let Some(a) = whole_call(rest, "position.set") {
                    return Done::Lines(
                        vec![format!("{name}.position = {};", vec3(&ex(self, &a)))],
                        S_LIGHT,
                    );
                }
                if let Some(a) = whole_call(rest, "lookAt") {
                    let a = ex(self, &a);
                    let target = if a.len() == 3 { vec3(&a) } else { a.join(", ") };
                    return Done::Lines(vec![format!("{name}.lookAt({target});")], S_LIGHT);
                }
                if let Some(a) = whole_call(rest, "rotation.set") {
                    return Done::Lines(
                        vec![format!(
                            "{name}.quaternion = quat.fromEuler({});",
                            euler(&ex(self, &a))
                        )],
                        S_OBJECT,
                    );
                }
                if whole_call(rest, "updateProjectionMatrix").is_some()
                    || matches!(assignment(rest), Some(("aspect", _)))
                {
                    return Done::Removed(
                        "the engine sizes the projection to the play surface".into(),
                        S_LIGHT,
                    );
                }
                if matches!(assignment(rest), Some(("fov" | "near" | "far" | "zoom", _))) {
                    return Done::Lines(Vec::new(), 0); // same name on og.Camera: left as written
                }
                Done::Manual(
                    format!("no og mapping for the camera's `{}`", short(rest)),
                    S_LIGHT,
                )
            }
            Var::Object | Var::Mesh | Var::Light(_) => {
                let aimed = matches!(var, Var::Light(l) if l.aimed);
                let position = |p: String| {
                    if aimed {
                        format!(
                            "position: {p}, quaternion: og.transform.lookAt({p}, {{ x: 0, y: 0, z: 0 }})"
                        )
                    } else {
                        format!("position: {p}")
                    }
                };
                if let Some(a) = whole_call(rest, "position.set") {
                    return Done::Lines(vec![transform(&position(vec3(&ex(self, &a))))], S_OBJECT);
                }
                if let Some(a) = whole_call(rest, "position.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&position(ex(self, &a).remove(0)))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "quaternion.set").filter(|a| a.len() == 4) {
                    let a = ex(self, &a);
                    return Done::Lines(
                        vec![transform(&format!(
                            "quaternion: {{ x: {}, y: {}, z: {}, w: {} }}",
                            a[0], a[1], a[2], a[3]
                        ))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "quaternion.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&format!("quaternion: {}", ex(self, &a)[0]))],
                        S_OBJECT,
                    );
                }
                if let Some(a) =
                    whole_call(rest, "rotation.set").filter(|a| (3..=4).contains(&a.len()))
                {
                    return Done::Lines(
                        vec![transform(&format!(
                            "quaternion: quat.fromEuler({})",
                            euler(&ex(self, &a))
                        ))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.set").filter(|a| a.len() == 3) {
                    return Done::Lines(
                        vec![transform(&format!("scale: {}", vec3(&ex(self, &a))))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.setScalar")
                    .filter(|a| a.len() == 1 && (is_ident(&a[0]) || a[0].parse::<f64>().is_ok()))
                {
                    let s = &a[0];
                    return Done::Lines(
                        vec![transform(&format!("scale: {{ x: {s}, y: {s}, z: {s} }}"))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&format!("scale: {}", ex(self, &a)[0]))],
                        S_OBJECT,
                    );
                }
                if let Some(children) = whole_call(rest, "add") {
                    let mut lines = Vec::new();
                    for c in &children {
                        if !matches!(
                            self.vars.get(c.as_str()),
                            Some(Var::Object | Var::Mesh | Var::Light(_))
                        ) {
                            return Done::Manual(
                                format!(
                                    "`{c}` is not an entity this file spawns: give it og.Parent {{ entity: {name} }} once it is one"
                                ),
                                S_GRAPH,
                            );
                        }
                        lines.push(format!(
                            "world.insert({c}, og.Parent, {{ entity: {name} }});"
                        ));
                    }
                    return Done::Lines(lines, S_GRAPH);
                }
                if let Some(children) = whole_call(rest, "remove") {
                    return Done::Lines(
                        children
                            .iter()
                            .map(|c| format!("world.remove({c}, og.Parent);"))
                            .collect(),
                        S_GRAPH,
                    );
                }
                if let Var::Light(light) = var
                    && let Some(a) = whole_call(rest, "color.set")
                        .or_else(|| whole_call(rest, "color.setHex"))
                        .filter(|a| a.len() == 1)
                {
                    let c = self.color_arg(&a[0], notes);
                    return Done::Lines(
                        vec![format!(
                            "world.set({name}, og.{}, {{ color: {c} }});",
                            light.component
                        )],
                        S_LIGHT,
                    );
                }
                if let Some((field, value)) = assignment(rest) {
                    let component = match var {
                        Var::Mesh if MESH_FIELDS.contains(&field) => Some("Mesh"),
                        Var::Light(l)
                            if LIGHT_FIELDS.contains(&field)
                                && (field == "castShadow" || l.params.contains(&Some(field))) =>
                        {
                            Some(l.component)
                        }
                        _ => None,
                    };
                    if let Some(component) = component {
                        let v = self.expr(value, notes);
                        return Done::Lines(
                            vec![format!(
                                "world.set({name}, og.{component}, {{ {field}: {v} }});"
                            )],
                            S_OBJECT,
                        );
                    }
                    if field == "name" {
                        return Done::Manual(
                            "an entity is named at its spawn (SpawnDesc.name), once".into(),
                            S_OBJECT,
                        );
                    }
                }
                let what = if rest.starts_with("position.")
                    || rest.starts_with("rotation.")
                    || rest.starts_with("scale.")
                    || rest.starts_with("quaternion.")
                {
                    "og.Transform is read and written whole: world.get, change the value, world.set (or a system over the column)"
                } else if rest.starts_with("lookAt(") {
                    "og.transform.lookAt(eye, target) points -Z at the target (a camera or a light); an object's +Z: og.transform.lookAt(target, eye)"
                } else if rest.starts_with("shadow.") || rest.starts_with("target.") {
                    "a light's shadow is its castShadow and RenderOptions.shadowMapSize; aim it with og.transform.lookAt"
                } else if rest.starts_with("traverse(") {
                    "a system's query over the components of what it visits"
                } else {
                    "no og mapping for this member of an Object3D"
                };
                Done::Manual(what.into(), S_OBJECT)
            }
        }
    }
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
                        "events reach the tick as og.Input (input.pressed(\"Space\")), read in a system; the engine handles resizing",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Migration {
        migrate(src).expect("a three.js file")
    }

    fn manual(m: &Migration) -> Vec<(usize, u8)> {
        m.items
            .iter()
            .filter(|i| i.kind == Kind::Manual)
            .map(|i| (i.line, i.section))
            .collect()
    }

    const SCENE: &str = r##"import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x87ceeb);
const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.5, 300);
camera.position.set(0, 2, 8);
camera.lookAt(0, 1, 0);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const steel = new THREE.MeshStandardMaterial({
  color: "#c0c8d0",
  metalness: 0.9,
  roughness: 0.35,
  side: THREE.DoubleSide,
});
const ship = new THREE.Group();
const hull = new THREE.Mesh(new THREE.CylinderGeometry(0.5, 1.25, 4, 24), steel);
hull.rotation.set(Math.PI / 2, 0, 0);
hull.castShadow = true;
ship.add(hull);
scene.add(ship);

const sun = new THREE.DirectionalLight(0xfff4e0, 2.5);
sun.position.set(5, 10, 2);
scene.add(sun);
scene.add(new THREE.AmbientLight(0x404040));

const controls = new OrbitControls(camera, renderer.domElement);
const clock = new THREE.Clock();
function animate() {
  requestAnimationFrame(animate);
  ship.rotation.y += clock.getDelta();
  controls.update();
  renderer.render(scene, camera);
}
animate();
"##;

    #[test]
    fn a_scene_file_is_rewritten_where_mechanical_and_listed_where_not() {
        let m = run(SCENE);
        let o = &m.output;
        for want in [
            "import { og, color, quat } from \"@origingame/engine\";",
            "function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {",
            "must(world.resource(og.Camera)).background = color.fromHex(0x87ceeb);",
            "const camera = must(world.resource(og.Camera));\ncamera.fov = 60;\ncamera.near = 0.5;\ncamera.far = 300;",
            "camera.position = { x: 0, y: 2, z: 8 };",
            "camera.lookAt({ x: 0, y: 1, z: 0 });",
            "const steel = must(og.render.standardMaterial(world, { color: color.fromHex(0xc0c8d0), metalness: 0.9, roughness: 0.35, side: \"double\" }));",
            "const ship = must(world.spawn({ name: \"ship\" }));",
            "const hull = must(world.spawn({ name: \"hull\" }));\nworld.insert(hull, og.Mesh, { geometry: must(og.render.cylinderGeometry(world, { radiusTop: 0.5, radiusBottom: 1.25, height: 4, radialSegments: 24 })), material: steel });",
            "world.set(hull, og.Transform, { quaternion: quat.fromEuler({ x: Math.PI / 2, y: 0, z: 0 }) });",
            "world.set(hull, og.Mesh, { castShadow: true });",
            "world.insert(hull, og.Parent, { entity: ship });",
            "world.insert(sun, og.DirectionalLight, { color: color.fromHex(0xfff4e0), intensity: 2.5 });",
            "world.set(sun, og.Transform, { position: { x: 5, y: 10, z: 2 }, quaternion: og.transform.lookAt({ x: 5, y: 10, z: 2 }, { x: 0, y: 0, z: 0 }) });",
        ] {
            assert!(o.contains(want), "missing\n{want}\nin\n{o}");
        }
        for gone in [
            "from \"three\"",
            "THREE.Scene",
            "WebGLRenderer",
            "setSize",
            "appendChild",
            "scene.add(ship)",
            "renderer.render",
            "camera.aspect",
        ] {
            assert!(!o.contains(gone), "`{gone}` left in\n{o}");
        }
        // What is not mechanical: the ambient light (§6), the controls
        // (§10, and their update), the clock (§8, and its read), the loop
        // (§8, and the in-place rotation, §3).
        assert_eq!(
            manual(&m),
            vec![
                (29, S_LIGHT),
                (31, S_GAPS),
                (32, S_TICK),
                (34, S_TICK),
                (35, S_OBJECT),
                (36, S_GAPS)
            ],
            "{:#?}",
            m.items
        );
        assert!(
            o.contains("// og migrate: AmbientLight: the sky lights every surface"),
            "{o}"
        );
        assert!(
            o.contains("  // og migrate: og.Transform is read and written whole"),
            "the comment keeps the line's indent:\n{o}"
        );
        // The OrbitControls import line is gone with three's others.
        assert!(!o.contains("three/addons"), "{o}");
    }

    #[test]
    fn named_imports_and_dropped_parameters() {
        let m = run(
            "import { Mesh, BoxGeometry, MeshBasicMaterial, PointLight, MathUtils } from 'three';\nconst box = new Mesh(new BoxGeometry(2, 3, 4, 8), new MeshBasicMaterial({ color: 0xff0000, shininess: 30 }));\nconst lamp = new PointLight(0xffffff, 5, 12, 2);\nconst a = MathUtils.degToRad(45);\n",
        );
        let o = &m.output;
        assert!(o.contains("world.insert(box, og.Mesh, { geometry: must(og.render.boxGeometry(world, { width: 2, height: 3, depth: 4 })), material: must(og.render.standardMaterial(world, { unlit: true, color: color.fromHex(0xff0000) })) });"), "{o}");
        assert!(o.contains("world.insert(lamp, og.PointLight, { color: color.fromHex(0xffffff), intensity: 5, distance: 12 });"), "{o}");
        assert!(o.contains("const a = ((45) * Math.PI / 180);"), "{o}");
        let notes: Vec<&str> = m
            .items
            .iter()
            .filter(|i| i.kind == Kind::Manual)
            .map(|i| i.og.as_str())
            .collect();
        assert_eq!(
            notes,
            [
                "BoxGeometry's widthSegments is not in BoxGeometryDesc; the argument was dropped",
                "MeshBasicMaterial's `shininess` has no field in StandardMaterialDesc; it was dropped",
                "PointLight's decay is not carried over (og's falloff is fixed); it was dropped"
            ]
        );
    }

    #[test]
    fn values_colours_and_loaders() {
        let m = run(r##"import * as THREE from "three";
const p = new THREE.Vector3(1, 2);
const q = new THREE.Quaternion();
const c = new THREE.Color(0.1, 0.2, 0.3);
const w = new THREE.Color();
const short = new THREE.Color("#f80");
const named = new THREE.Color("red");
const loader = new THREE.TextureLoader();
const crate = loader.load("./assets/textures/crate.png");
const grid = new THREE.TextureLoader().load("/textures/grid.png", (t) => t);
const tube = new THREE.TubeGeometry(new THREE.CatmullRomCurve3(pts), 64, 0.1);
const knot = new THREE.TubeGeometry(curve, 64);
const gear = new THREE.ExtrudeGeometry(shape, { depth: 0.2, bevelEnabled: false });
const hole = new THREE.Path();
"##);
        let o = &m.output;
        for want in [
            "const p = { x: 1, y: 2, z: 0 };",
            "const q = { x: 0, y: 0, z: 0, w: 1 };",
            "const c = { r: 0.1, g: 0.2, b: 0.3 };",
            "const w = { r: 1, g: 1, b: 1 };",
            "const short = color.fromHex(0xff8800);",
            "const named = new THREE.Color(\"red\");",
            "const crate = must(og.load.texture(world, \"textures/crate.png\"));",
            "const tube = must(og.render.tubeGeometry(world, { points: pts, tubularSegments: 64, radius: 0.1 }));",
            "const knot = new THREE.TubeGeometry(curve, 64);",
            "const gear = must(og.render.extrudeGeometry(world, { shapes: shape, depth: 0.2, bevelEnabled: false }));",
            "const hole = new og.Path2();",
        ] {
            assert!(o.contains(want), "missing\n{want}\nin\n{o}");
        }
        assert!(
            !o.contains("TextureLoader();"),
            "the loader declaration goes:\n{o}"
        );
        assert!(
            o.contains("new THREE.TextureLoader().load(\"/textures/grid.png\", (t) => t)"),
            "a load with a callback stays:\n{o}"
        );
        assert_eq!(
            manual(&m).iter().map(|(l, _)| *l).collect::<Vec<_>>(),
            vec![7, 10, 12],
            "{:#?}",
            m.items
        );
    }

    #[test]
    fn a_file_without_three_is_not_migrated() {
        assert!(migrate("import { og } from \"@origingame/engine\";\nconst a = 1;\n").is_none());
    }

    /// Every desc field, light field and og.render function the rewrite
    /// writes is one the generated API has, and every StandardMaterialDesc
    /// field passes through: the tables cannot drift from api.ogt.
    #[test]
    fn the_tables_match_the_generated_api() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine/schema/api.schema.json");
        let schema: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let fields = |t: &str| -> HashSet<String> {
            schema["$defs"][t]["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("no {t}"))
                .keys()
                .cloned()
                .collect()
        };
        let render: HashSet<String> = schema["x-og-modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == "render")
            .unwrap()["functions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["name"].as_str().unwrap().to_string())
            .collect();
        for g in GEOMETRIES {
            assert!(render.contains(g.og), "og.render.{}", g.og);
            let have = fields(g.desc);
            for p in g.params {
                assert!(have.contains(*p), "{}.{p}", g.desc);
            }
        }
        let tube = fields("TubeGeometryDesc");
        assert!(
            TUBE_PARAMS
                .iter()
                .chain(&["points"])
                .all(|p| tube.contains(*p))
        );
        assert!(fields("ExtrudeGeometryDesc").contains("shapes"));
        let standard: HashSet<String> = STANDARD_FIELDS.iter().map(|s| s.to_string()).collect();
        assert_eq!(standard, fields("StandardMaterialDesc"));
        for (_, extra) in MATERIALS {
            if let Some((k, _)) = extra.split_once(": ") {
                assert!(standard.contains(k), "{k}");
            }
        }
        assert!(COLOR_FIELDS.iter().all(|c| standard.contains(*c)));
        for l in LIGHTS {
            let have = fields(l.component);
            assert!(
                l.params.iter().flatten().all(|p| have.contains(*p)),
                "{}",
                l.component
            );
            assert!(
                LIGHT_FIELDS
                    .iter()
                    .filter(|f| l.params.contains(&Some(**f)) || **f == "castShadow")
                    .all(|f| have.contains(*f)),
                "{}",
                l.component
            );
        }
        let mesh = fields("Mesh");
        assert!(MESH_FIELDS.iter().all(|f| mesh.contains(*f)));
        assert!(
            fields("Camera").is_superset(
                &[
                    "fov",
                    "near",
                    "far",
                    "orthographic",
                    "height",
                    "background",
                    "position",
                    "quaternion"
                ]
                .map(String::from)
                .into()
            )
        );
    }
}
