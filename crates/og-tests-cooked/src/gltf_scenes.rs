//! Scenes of glTF extensions through the cook (roadmap G1-3), each glTF
//! written here as a `.glb`, cooked into a project of its own (once per
//! process) and drawn from its blobs as og-runtime draws a model.
//!
//! - `gltf-uv-transform`: `KHR_texture_transform` carried into the
//!   material (not baked into the uvs: each material's emissive texture
//!   has no transform, so its slots differ). Two quads of one 2 × 2 atlas
//!   texture (red, green, blue and yellow cells, each with a white marker
//!   in its top-left corner and a dark border): on the left tiled
//!   (repeat 3 × 2, rotated 0.35 rad, offset 0.1 along u), on the right
//!   its yellow cell alone (offset 0.5 0.5, repeat 0.5 0.5). A wrong
//!   offset shows another cell on the right, a wrong scale several, a
//!   wrong rotation or order tilts the tiles on the left the other way or
//!   shears them.
//! - `gltf-punctual-lights`: `KHR_lights_punctual` on a dark floor with a
//!   box at its middle: a warm point light of infinite range on the left
//!   (its pool reaches as far as og-runtime's reach for it), a cool spot
//!   pointing down on the right (a disc with a soft rim), and a dim
//!   directional light from the front lighting the box's top and front.
//! - `gltf-instancing`: `EXT_mesh_gpu_instancing`, one box mesh drawn at
//!   seven instances along a row (each turned 0.2 rad more and scaled
//!   0.6 to 1.2): seven boxes, one mesh and one material.
//! - `gltf-variants-0`, `gltf-variants-1`: `KHR_materials_variants`, a
//!   grey body with a grey trim box on it. Variant 0 (`red`) maps the
//!   body red and leaves the trim grey; variant 1 (`blue`) maps the body
//!   blue and the trim gold.

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::{Bundle, CookedModel, ModelParts};
use og_render::{
    Background, Camera, DirectionalLight, Exposure, Light, LightKind, RenderScene, RgbaImage,
    StandardMaterial, shapes,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;

/// `bufferView.target` of vertex data.
const VERTICES: u32 = 34962;

/// A `.glb` being written: its JSON and its binary chunk.
struct Glb {
    json: Value,
    bin: Vec<u8>,
}

impl Glb {
    fn new() -> Glb {
        Glb {
            json: json!({
                "asset": { "version": "2.0", "generator": "og-tests-cooked gltf_scenes" },
                "scene": 0,
                "scenes": [{ "nodes": [] }],
                "nodes": [], "meshes": [], "materials": [], "textures": [], "images": [],
                "accessors": [], "bufferViews": [], "buffers": [],
                "extensionsUsed": [],
            }),
            bin: Vec::new(),
        }
    }

    fn push(&mut self, list: &str, item: Value) -> usize {
        let a = self.json[list].as_array_mut().expect("a list");
        a.push(item);
        a.len() - 1
    }

    fn uses(&mut self, ext: &str, required: bool) {
        self.push("extensionsUsed", json!(ext));
        if required {
            if self.json.get("extensionsRequired").is_none() {
                self.json["extensionsRequired"] = json!([]);
            }
            self.push("extensionsRequired", json!(ext));
        }
    }

    /// A buffer view of `bytes`, for vertex data (`target` 34962), indices
    /// (34963) or neither (0: an image, instance transforms).
    fn view(&mut self, bytes: &[u8], target: u32) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        let mut view = json!({ "buffer": 0, "byteOffset": offset, "byteLength": bytes.len() });
        if target != 0 {
            view["target"] = json!(target);
        }
        self.push("bufferViews", view)
    }

    /// A float accessor of `ty` (`VEC3`, `VEC2`, `VEC4`) over `values`,
    /// its view's `target` as [`Glb::view`]'s.
    fn floats(&mut self, values: &[f32], ty: &str, width: usize, target: u32) -> usize {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let view = self.view(&bytes, target);
        let mut acc = json!({
            "bufferView": view, "componentType": 5126,
            "count": values.len() / width, "type": ty,
        });
        if ty == "VEC3" {
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for p in values.chunks(3) {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            acc["min"] = json!(lo);
            acc["max"] = json!(hi);
        }
        self.push("accessors", acc)
    }

    /// A primitive of `g` drawn with `material`.
    fn primitive(&mut self, g: &Geometry, material: usize) -> Value {
        let position = self.floats(&g.positions, "VEC3", 3, VERTICES);
        let normal = self.floats(&g.normals, "VEC3", 3, VERTICES);
        let uv = self.floats(&g.uvs, "VEC2", 2, VERTICES);
        let bytes: Vec<u8> = g.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let view = self.view(&bytes, 34963);
        let indices = self.push(
            "accessors",
            json!({ "bufferView": view, "componentType": 5123, "count": g.indices.len(), "type": "SCALAR" }),
        );
        json!({
            "attributes": { "POSITION": position, "NORMAL": normal, "TEXCOORD_0": uv },
            "indices": indices, "material": material,
        })
    }

    /// A mesh of `primitives` on a new root node with `node` (its
    /// transform, extensions) merged in; the node's index.
    fn mesh_node(&mut self, name: &str, primitives: Vec<Value>, node: Value) -> usize {
        let mesh = self.push("meshes", json!({ "name": name, "primitives": primitives }));
        let mut n = json!({ "name": name, "mesh": mesh });
        n.as_object_mut()
            .expect("an object")
            .extend(node.as_object().cloned().unwrap_or_default());
        self.node(n)
    }

    /// A root node of the default scene.
    fn node(&mut self, node: Value) -> usize {
        let n = self.push("nodes", node);
        self.json["scenes"][0]["nodes"]
            .as_array_mut()
            .expect("a list")
            .push(json!(n));
        n
    }

    /// A PNG image and a texture of it.
    fn texture(&mut self, image: &RgbaImage) -> usize {
        let view = self.view(&og_render::png(image), 0);
        let source = self.push(
            "images",
            json!({ "bufferView": view, "mimeType": "image/png" }),
        );
        self.push("textures", json!({ "source": source }))
    }

    fn finish(mut self) -> Vec<u8> {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        self.json["buffers"] = json!([{ "byteLength": self.bin.len() }]);
        // glTF allows no empty top-level list.
        self.json
            .as_object_mut()
            .expect("an object")
            .retain(|_, v| v.as_array().is_none_or(|a| !a.is_empty()));
        let mut text = serde_json::to_vec(&self.json).expect("json");
        while !text.len().is_multiple_of(4) {
            text.push(b' ');
        }
        let total = 12 + 8 + text.len() + 8 + self.bin.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&(total as u32).to_le_bytes());
        out.extend_from_slice(&(text.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&text);
        out.extend_from_slice(&(self.bin.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&self.bin);
        out
    }
}

/// Triangles with normals and uvs (glTF's: v down the image).
#[derive(Default)]
struct Geometry {
    positions: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u16>,
}

impl Geometry {
    /// A face: its corners counter-clockwise seen from outside, starting
    /// at the one uv (0, 1) maps to.
    fn face(&mut self, corners: [Vec3; 4], normal: Vec3) {
        let base = (self.positions.len() / 3) as u16;
        for (c, uv) in corners
            .iter()
            .zip([[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]])
        {
            self.positions.extend(c.to_array());
            self.normals.extend(normal.to_array());
            self.uvs.extend(uv);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|i: u16| base + i));
    }

    /// An axis-aligned box from `lo` to `hi`, each face's uvs 0 to 1.
    fn cuboid(lo: Vec3, hi: Vec3) -> Geometry {
        let mut g = Geometry::default();
        for axis in 0..3 {
            for sign in [1.0f32, -1.0] {
                let mut n = Vec3::ZERO;
                n[axis] = sign;
                // Two tangents with u × v = n.
                let (u, v) = if axis == 1 {
                    (Vec3::X * sign, Vec3::NEG_Z)
                } else {
                    (Vec3::Y.cross(n), Vec3::Y)
                };
                let c = (lo + hi) * 0.5 + n * (hi - lo) * 0.5;
                let (hu, hv) = (
                    u * ((hi - lo) * u.abs()).length() * 0.5,
                    v * ((hi - lo) * v.abs()).length() * 0.5,
                );
                g.face([c - hu - hv, c + hu - hv, c + hu + hv, c - hu + hv], n);
            }
        }
        g
    }

    /// A `w` × `h` quad in the XY plane facing +Z, centred on the origin.
    fn quad(w: f32, h: f32) -> Geometry {
        let (x, y) = (w * 0.5, h * 0.5);
        let mut g = Geometry::default();
        g.face(
            [
                Vec3::new(-x, -y, 0.0),
                Vec3::new(x, -y, 0.0),
                Vec3::new(x, y, 0.0),
                Vec3::new(-x, y, 0.0),
            ],
            Vec3::Z,
        );
        g
    }
}

/// The 128 × 128 atlas of `gltf-uv-transform` (module docs).
fn atlas() -> RgbaImage {
    const N: u32 = 128;
    let cells = [[220, 40, 40], [40, 180, 60], [40, 80, 220], [230, 200, 40]];
    let mut pixels = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let (cx, cy, lx, ly) = (x / 64, y / 64, x % 64, y % 64);
            let rgb = if lx < 3 || ly < 3 || lx > 60 || ly > 60 {
                [20, 20, 24]
            } else if lx < 22 && ly < 22 {
                [255, 255, 255]
            } else {
                cells[(cy * 2 + cx) as usize]
            };
            pixels.extend([rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    RgbaImage {
        width: N,
        height: N,
        pixels,
    }
}

/// A glTF material of `color` (linear rgb) and no texture.
fn plain(name: &str, color: [f32; 3]) -> Value {
    json!({
        "name": name,
        "pbrMetallicRoughness": {
            "baseColorFactor": [color[0], color[1], color[2], 1.0],
            "metallicFactor": 0.0, "roughnessFactor": 0.8,
        },
    })
}

fn uv_transform_glb() -> Vec<u8> {
    let mut g = Glb::new();
    g.uses("KHR_texture_transform", false);
    let tex = g.texture(&atlas());
    let quad = Geometry::quad(2.0, 2.0);
    let transforms = [
        (
            "tiled",
            json!({ "offset": [0.1, 0.0], "scale": [3.0, 2.0], "rotation": 0.35 }),
            -1.2,
        ),
        (
            "atlas",
            json!({ "offset": [0.5, 0.5], "scale": [0.5, 0.5] }),
            1.2,
        ),
    ];
    for (name, transform, x) in transforms {
        let material = g.push(
            "materials",
            json!({
                "name": name,
                "pbrMetallicRoughness": {
                    "baseColorTexture": {
                        "index": tex,
                        "extensions": { "KHR_texture_transform": transform },
                    },
                    "metallicFactor": 0.0, "roughnessFactor": 0.9,
                },
                // An untransformed slot beside the transformed one: the
                // cook carries the transform instead of baking it.
                "emissiveTexture": { "index": tex },
                "emissiveFactor": [0.0, 0.0, 0.0],
            }),
        );
        let prim = g.primitive(&quad, material);
        g.mesh_node(name, vec![prim], json!({ "translation": [x, 1.0, 0.0] }));
    }
    g.finish()
}

fn lights_glb() -> Vec<u8> {
    let mut g = Glb::new();
    g.uses("KHR_lights_punctual", false);
    let floor = g.push("materials", plain("floor", [0.5, 0.5, 0.5]));
    let block = g.push("materials", plain("block", [0.7, 0.7, 0.7]));
    let prims = vec![
        g.primitive(
            &Geometry::cuboid(Vec3::new(-6.0, -0.2, -4.0), Vec3::new(6.0, 0.0, 4.0)),
            floor,
        ),
        g.primitive(
            &Geometry::cuboid(Vec3::new(-0.4, 0.0, -0.4), Vec3::new(0.4, 0.8, 0.4)),
            block,
        ),
    ];
    g.mesh_node("floor", prims, json!({}));
    g.json["extensions"] = json!({ "KHR_lights_punctual": { "lights": [
        { "name": "warm", "type": "point", "color": [1.0, 0.55, 0.2], "intensity": 6.0 },
        { "name": "cool", "type": "spot", "color": [0.3, 0.6, 1.0], "intensity": 40.0, "range": 8.0,
          "spot": { "innerConeAngle": 0.25, "outerConeAngle": 0.45 } },
        { "name": "sun", "type": "directional", "color": [1.0, 1.0, 0.95], "intensity": 0.5 },
    ] } });
    let down = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let from_front = Quat::from_rotation_x(-1.0);
    let light = |i: usize| json!({ "KHR_lights_punctual": { "light": i } });
    g.node(json!({ "name": "warm", "translation": [-3.0, 1.2, 0.0], "extensions": light(0) }));
    g.node(json!({ "name": "cool", "translation": [3.0, 3.0, 0.0],
        "rotation": down.to_array(), "extensions": light(1) }));
    g.node(json!({ "name": "sun", "rotation": from_front.to_array(), "extensions": light(2) }));
    g.finish()
}

/// The instances of `gltf-instancing`.
pub const INSTANCES: usize = 7;

fn instancing_glb() -> Vec<u8> {
    let mut g = Glb::new();
    g.uses("EXT_mesh_gpu_instancing", true);
    let mat = g.push("materials", plain("crate", [0.65, 0.42, 0.22]));
    let prim = g.primitive(
        &Geometry::cuboid(Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 0.6, 0.3)),
        mat,
    );
    let (mut t, mut r, mut s) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..INSTANCES {
        let f = i as f32;
        t.extend([(f - 3.0) * 1.1, 0.0, 0.0]);
        r.extend(Quat::from_rotation_y(0.2 * f).to_array());
        s.extend([0.6 + 0.1 * f; 3]);
    }
    let attributes = json!({
        "TRANSLATION": g.floats(&t, "VEC3", 3, 0),
        "ROTATION": g.floats(&r, "VEC4", 4, 0),
        "SCALE": g.floats(&s, "VEC3", 3, 0),
    });
    g.mesh_node(
        "crates",
        vec![prim],
        json!({ "extensions": { "EXT_mesh_gpu_instancing": { "attributes": attributes } } }),
    );
    g.finish()
}

fn variants_glb() -> Vec<u8> {
    let mut g = Glb::new();
    g.uses("KHR_materials_variants", false);
    let body = g.push("materials", plain("body", [0.45, 0.45, 0.45]));
    let trim = g.push("materials", plain("trim", [0.3, 0.3, 0.3]));
    let red = g.push("materials", plain("red", [0.8, 0.05, 0.04]));
    let blue = g.push("materials", plain("blue", [0.05, 0.15, 0.8]));
    let gold = g.push("materials", plain("gold", [0.9, 0.65, 0.1]));
    g.json["extensions"] = json!({ "KHR_materials_variants": {
        "variants": [{ "name": "red" }, { "name": "blue" }],
    } });
    let mut body_prim = g.primitive(
        &Geometry::cuboid(Vec3::new(-0.8, 0.0, -0.5), Vec3::new(0.8, 0.6, 0.5)),
        body,
    );
    body_prim["extensions"] = json!({ "KHR_materials_variants": { "mappings": [
        { "material": red, "variants": [0] },
        { "material": blue, "variants": [1] },
    ] } });
    let mut trim_prim = g.primitive(
        &Geometry::cuboid(Vec3::new(-0.5, 0.6, -0.3), Vec3::new(0.5, 0.9, 0.3)),
        trim,
    );
    trim_prim["extensions"] = json!({ "KHR_materials_variants": { "mappings": [
        { "material": gold, "variants": [1] },
    ] } });
    g.mesh_node("sofa", vec![body_prim, trim_prim], json!({}));
    g.finish()
}

/// The project holding every glTF here under `assets/models/`, cooked
/// once per process.
fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-gltf-ext-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        for (name, bytes) in [
            ("uv-transform.glb", uv_transform_glb()),
            ("lights.glb", lights_glb()),
            ("instancing.glb", instancing_glb()),
            ("variants.glb", variants_glb()),
        ] {
            std::fs::write(models.join(name), bytes).map_err(|e| format!("{name}: {e}"))?;
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        Ok(root)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// The cooked model `models/<name>.glb`.
pub fn model(name: &str) -> Result<CookedModel, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    bundle
        .load_model(&format!("models/{name}.glb"))
        .map_err(|e| e.to_string())
}

/// A scene lit by a sun from the front-left over a sky-blue background,
/// with `model` drawn at the origin (under `variant`, `None` its own
/// materials); the parts it drew.
fn with_model(
    model: &CookedModel,
    variant: Option<usize>,
    camera: Camera,
) -> (RenderScene, ModelParts) {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.5, 0.8, 0.6);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.intensity = 0.7;
    scene.lighting.environment.background = Background::Color(Vec3::new(0.55, 0.68, 0.82));
    scene.frame.camera = camera;
    let bundle = Bundle::open(&cooked().expect("cooked").join("cooked")).expect("the bundle opens");
    let textures: Vec<_> = model
        .textures
        .iter()
        .map(|t| bundle.load_texture(&t.id).and_then(|t| t.to_texture_data()))
        .collect::<Result<_, _>>()
        .expect("the model's textures decode");
    let parts = model.add_to(&mut scene, &textures);
    for (k, &(mesh, material, at)) in parts.parts.iter().enumerate() {
        let material = variant.map_or(material, |v| parts.variants[v][k]);
        scene.add_instance(mesh, material, at);
    }
    (scene, parts)
}

fn floor(scene: &mut RenderScene) {
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.42, 0.5, 0.38),
        roughness: 1.0,
        ..Default::default()
    });
    let plane = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(plane, ground, Mat4::IDENTITY);
}

/// `gltf-uv-transform` (module docs), and its model.
pub fn uv_transform_scene() -> Result<(RenderScene, CookedModel), String> {
    let model = model("uv-transform")?;
    let (scene, _) = with_model(
        &model,
        None,
        Camera {
            position: Vec3::new(0.0, 1.0, 4.2),
            target: Vec3::new(0.0, 1.0, 0.0),
            exposure: Exposure::linear(0.8),
            ..Default::default()
        },
    );
    Ok((scene, model))
}

/// The model's lights as og-runtime spawns them (`og_runtime` assets'
/// `model_lights`, then the frame's extraction): glTF's point and spot
/// lights as local lights (range 0, infinite, ends where the radiance
/// falls to 0.01; the spot's outer cone its angle, `1 - inner / outer`
/// its penumbra), its directional light as the scene's sun. Each shines
/// along its node's -Z.
pub fn model_lights(model: &CookedModel, scene: &mut RenderScene) {
    use og_cook::format::light;
    let world = model.node_world();
    for l in &model.lights {
        let (_, rotation, position) = world[l.node].to_scale_rotation_translation();
        let direction = rotation * Vec3::NEG_Z;
        let color = Vec3::from(l.color);
        let range = if l.range > 0.0 {
            l.range
        } else {
            (l.intensity.max(0.0) / 0.01).sqrt().max(0.01)
        };
        let kind = match l.kind {
            light::DIRECTIONAL => {
                scene.lighting.light = DirectionalLight {
                    direction_to_light: -direction,
                    color,
                    intensity: l.intensity,
                    ..Default::default()
                };
                continue;
            }
            light::SPOT => {
                let [inner, outer] = l.cone;
                LightKind::Spot {
                    direction,
                    angle_degrees: outer.to_degrees(),
                    penumbra: if outer > 0.0 {
                        (1.0 - inner / outer).clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                }
            }
            _ => LightKind::Point,
        };
        scene.lighting.lights.push(Light {
            kind,
            position,
            color,
            intensity: l.intensity,
            range,
            cast_shadows: false,
        });
    }
}

/// `gltf-punctual-lights` (module docs), and its model.
pub fn lights_scene() -> Result<(RenderScene, CookedModel), String> {
    let model = model("lights")?;
    let (mut scene, _) = with_model(
        &model,
        None,
        Camera {
            position: Vec3::new(0.0, 5.5, 8.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        },
    );
    scene.lighting.environment.intensity = 0.05;
    scene.lighting.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    model_lights(&model, &mut scene);
    Ok((scene, model))
}

/// `gltf-instancing` (module docs), its model and the parts it drew.
pub fn instancing_scene() -> Result<(RenderScene, CookedModel, ModelParts), String> {
    let model = model("instancing")?;
    let (mut scene, parts) = with_model(
        &model,
        None,
        Camera {
            position: Vec3::new(0.0, 2.6, 6.5),
            target: Vec3::new(0.0, 0.3, 0.0),
            exposure: Exposure::linear(0.8),
            ..Default::default()
        },
    );
    floor(&mut scene);
    Ok((scene, model, parts))
}

/// `gltf-variants-<variant>` (module docs), and its model.
pub fn variants_scene(variant: usize) -> Result<(RenderScene, CookedModel), String> {
    let model = model("variants")?;
    let (mut scene, _) = with_model(
        &model,
        Some(variant),
        Camera {
            position: Vec3::new(1.6, 1.8, 3.0),
            target: Vec3::new(0.0, 0.45, 0.0),
            exposure: Exposure::linear(0.8),
            ..Default::default()
        },
    );
    floor(&mut scene);
    Ok((scene, model))
}
