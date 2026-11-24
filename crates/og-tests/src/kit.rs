//! The scenes' shared set dressing (`assets/kit`, its origins and licences
//! in `assets/kit/README.md`): ruins, a town and a yard's wares
//! ([`MODELS`]) and tiling surfaces (grass, cobbles, planks, a rubble stone
//! wall and a rock face: [`SURFACES`]), so a scene stands in a
//! place instead of on a checker. A scene still builds its subject itself; this is what
//! stands around it.
//!
//! Every file is decoded once per process and shared by the scenes that
//! place it.

use crate::fx_scenes::{noise2, smooth};
use crate::scenes::asset;
use glam::{Mat4, Vec2, Vec3};
use og_render::gltf::{GltfModel, ModelParts, load_glb};
use og_render::{
    ColorSpace, Material, MaterialHandle, MeshData, Mips, RenderScene, TextureData, TextureHandle,
    Vertex,
};
use std::sync::{Arc, Mutex};

/// The kit's models (`assets/kit/<name>.glb`).
pub const MODELS: &[&str] = &[
    "column",
    "column_broken",
    "arch",
    "watchtower",
    "boulder",
    "pine",
    "ladder",
    "townhouse",
    "streetlamp",
    "car",
    "adobe",
    "counter",
    "crate",
    "barrel",
    "palm",
    "winebarrel",
    "torii",
    "stone_bench",
    "toro",
];
/// The kit's surfaces (`assets/kit/ground/<name>.{color,normal,orm}.jpg`).
pub const SURFACES: &[&str] = &["grass", "cobbles", "planks", "stone"];

/// A committed asset is part of the harness: a missing or broken one is a
/// defect of the checkout, not a condition a scene handles.
fn bytes(rel: &str) -> std::borrow::Cow<'static, [u8]> {
    asset(rel).unwrap_or_else(|e| panic!("kit asset {e}"))
}

fn cached<T: Clone>(cache: &Mutex<Vec<(String, T)>>, name: &str, make: impl FnOnce() -> T) -> T {
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, v)) = cache.iter().find(|(n, _)| n == name) {
        return v.clone();
    }
    let v = make();
    cache.push((name.to_string(), v.clone()));
    v
}

fn model(name: &str) -> Arc<GltfModel> {
    static MODELS: Mutex<Vec<(String, Arc<GltfModel>)>> = Mutex::new(Vec::new());
    cached(&MODELS, name, || {
        let rel = format!("kit/{name}.glb");
        Arc::new(load_glb(&bytes(&rel)).unwrap_or_else(|e| panic!("kit asset {rel}: {e}")))
    })
}

fn image(rel: &str, color_space: ColorSpace) -> Arc<TextureData> {
    static IMAGES: Mutex<Vec<(String, Arc<TextureData>)>> = Mutex::new(Vec::new());
    cached(&IMAGES, rel, || {
        let img = image::load_from_memory(&bytes(rel))
            .unwrap_or_else(|e| panic!("kit asset {rel}: {e}"))
            .to_rgba8();
        Arc::new(TextureData {
            width: img.width(),
            height: img.height(),
            rgba: img.into_raw(),
            color_space,
            mips: Mips::Generate,
        })
    })
}

/// A painted 2D picture from the assets (`rel`, such as
/// `sprites2d/cistern.jpg`, sRGB) as a scene texture; `nearest` for pixel
/// art, whose texels stay square when scaled up.
pub fn picture(scene: &mut RenderScene, rel: &str, nearest: bool) -> TextureHandle {
    let t = scene.add_texture((*image(rel, ColorSpace::Srgb)).clone());
    if nearest {
        scene.set_texture_filter(t, og_render::TextureFilter::Nearest);
    }
    t
}

/// A kit model in a scene: its parts and its bounds.
#[derive(Clone, Debug)]
pub struct Prop {
    parts: ModelParts,
    pub min: Vec3,
    pub max: Vec3,
}

impl Prop {
    /// The transform that stands the model on `foot` (the middle of its
    /// base there), turned `yaw` about +y and scaled to `height` metres.
    pub fn transform(&self, foot: Vec3, yaw: f32, height: f32) -> Mat4 {
        let scale = height / (self.max.y - self.min.y).max(1e-3);
        let mid = (self.min + self.max) * 0.5;
        Mat4::from_translation(foot)
            * Mat4::from_rotation_y(yaw)
            * Mat4::from_scale(Vec3::splat(scale))
            * Mat4::from_translation(Vec3::new(-mid.x, -self.min.y, -mid.z))
    }

    /// Places one copy ([`Self::transform`]).
    pub fn place(&self, scene: &mut RenderScene, foot: Vec3, yaw: f32, height: f32) {
        self.parts
            .instance(scene, self.transform(foot, yaw, height));
    }

    /// Places one copy by `transform` (a copy that leans or is stretched:
    /// a frame of the scene's times [`Self::transform`] at the origin).
    pub fn place_by(&self, scene: &mut RenderScene, transform: Mat4) {
        self.parts.instance(scene, transform);
    }

    /// The model's first part remade `height` metres tall, the middle of
    /// its base at its origin, with its material: for what places a mesh
    /// by its origin and in its own units (scattered copies, the wind
    /// hook, which leans a model by its height above its origin).
    pub fn rooted(
        &self,
        scene: &mut RenderScene,
        height: f32,
    ) -> (og_render::MeshHandle, MaterialHandle) {
        let (mesh, material) = self.parts.parts[0];
        let scale = height / (self.max.y - self.min.y).max(1e-3);
        let foot = Vec3::new(
            (self.min.x + self.max.x) * 0.5,
            self.min.y,
            (self.min.z + self.max.z) * 0.5,
        );
        let mut made = scene.mesh(mesh).expect("a kit model's mesh").clone();
        for v in &mut made.vertices {
            v.position = ((Vec3::from(v.position) - foot) * scale).into();
        }
        (scene.add_mesh(made), material)
    }

    /// The model's meshes with their materials, for what draws a mesh
    /// itself (a terrain stamp, scattered copies).
    pub fn parts(&self) -> &[(og_render::MeshHandle, MaterialHandle)] {
        &self.parts.parts
    }
}

/// Adds the kit model `name` to `scene` (its meshes, materials and
/// textures once; place copies with [`Prop::place`]).
pub fn prop(scene: &mut RenderScene, name: &str) -> Prop {
    let m = model(name);
    Prop {
        parts: m.add_to(scene),
        min: m.min,
        max: m.max,
    }
}

/// The kit surface `name` as a material: its colour, its normal map and
/// its roughness (the green of its occlusion-roughness-metal map).
pub fn surface(scene: &mut RenderScene, name: &str) -> MaterialHandle {
    surface_tinted(scene, name, [1.0; 3])
}

/// [`surface`] with its colour multiplied by `tint`.
pub fn surface_tinted(scene: &mut RenderScene, name: &str, tint: [f32; 3]) -> MaterialHandle {
    surface_finish(scene, name, tint, 1.0)
}

/// [`surface_tinted`] with its roughness map scaled by `roughness`: below
/// 1 the surface is polished or wet.
pub fn surface_finish(
    scene: &mut RenderScene,
    name: &str,
    tint: [f32; 3],
    roughness: f32,
) -> MaterialHandle {
    let [color, normal, orm] = surface_textures(scene, name);
    scene.add_material(Material {
        base_color: [tint[0], tint[1], tint[2], 1.0],
        base_color_texture: Some(color),
        normal_texture: Some(normal),
        metallic_roughness_texture: Some(orm),
        metallic: 0.0,
        roughness,
        ..Default::default()
    })
}

/// The kit surface `name`'s maps added to `scene`: its colour (sRGB), its
/// normal map and its occlusion-roughness-metal map.
pub fn surface_textures(scene: &mut RenderScene, name: &str) -> [TextureHandle; 3] {
    let mut add = |part: &str, space| {
        let rel = format!("kit/ground/{name}.{part}.jpg");
        scene.add_texture((*image(&rel, space)).clone())
    };
    [
        add("color", ColorSpace::Srgb),
        add("normal", ColorSpace::Linear),
        add("orm", ColorSpace::Linear),
    ]
}

/// Lime plaster painted `color` (linear) over the kit's rubble stone: the
/// stone's normal and roughness maps under a flat coat of paint.
pub fn plaster(scene: &mut RenderScene, color: [f32; 3]) -> MaterialHandle {
    let [_, normal, orm] = surface_textures(scene, "stone");
    scene.add_material(Material {
        base_color: [color[0], color[1], color[2], 1.0],
        normal_texture: Some(normal),
        metallic_roughness_texture: Some(orm),
        metallic: 0.0,
        roughness: 1.0,
        ..Default::default()
    })
}

/// The kit surface `name` as a terrain layer's maps: its colour, with its
/// brightness in the alpha as the layer's height (the tufts and stones
/// stand out where layers blend by height), and its normal map.
pub fn layer_maps(scene: &mut RenderScene, name: &str) -> (TextureHandle, TextureHandle) {
    let mut color = (*image(&format!("kit/ground/{name}.color.jpg"), ColorSpace::Srgb)).clone();
    for px in color.rgba.as_chunks_mut::<4>().0 {
        let luma = (u32::from(px[0]) * 54 + u32::from(px[1]) * 183 + u32::from(px[2]) * 19) >> 8;
        px[3] = (60 + luma.min(195)) as u8;
    }
    let normal = (*image(&format!("kit/ground/{name}.normal.jpg"), ColorSpace::Linear)).clone();
    (scene.add_texture(color), scene.add_texture(normal))
}

/// A flat rectangle on the plane y = `y` from `min` to `max` (xz), its
/// surface repeating every `tile` metres from the world's origin, so
/// rectangles laid side by side continue each other's pattern.
pub fn slab(min: Vec2, max: Vec2, y: f32, tile: f32) -> MeshData {
    let v = |x: f32, z: f32| Vertex {
        position: [x, y, z],
        normal: [0.0, 1.0, 0.0],
        uv: [x / tile, z / tile],
        color: [1.0; 4],
        tangent: [0.0; 4],
    };
    MeshData {
        vertices: vec![
            v(min.x, min.y),
            v(max.x, min.y),
            v(max.x, max.y),
            v(min.x, max.y),
        ],
        indices: vec![0, 2, 1, 0, 3, 2],
    }
}

/// A box from `min` to `max`, its surface repeating every `tile` metres
/// from the world's origin on every face (walls, kerbs and plinths of a
/// kit surface).
pub fn block(min: Vec3, max: Vec3, tile: f32) -> MeshData {
    let mut m = MeshData {
        vertices: Vec::with_capacity(24),
        indices: Vec::with_capacity(36),
    };
    // Each face: its axis, and the two axes across it in the order that
    // winds it outwards on its positive side.
    for (axis, u, v) in [(0, 2, 1), (1, 0, 2), (2, 1, 0)] {
        for (side, at) in [(-1.0f32, min), (1.0, max)] {
            let first = m.vertices.len() as u32;
            for (a, b) in [
                (min[u], min[v]),
                (max[u], min[v]),
                (max[u], max[v]),
                (min[u], max[v]),
            ] {
                let mut position = [0.0; 3];
                position[axis] = at[axis];
                position[u] = a;
                position[v] = b;
                let mut normal = [0.0; 3];
                normal[axis] = side;
                m.vertices.push(Vertex {
                    position,
                    normal,
                    uv: [a / tile, b / tile],
                    color: [1.0; 4],
                    tangent: [0.0; 4],
                });
            }
            let quad: [u32; 6] = if side > 0.0 {
                [0, 2, 1, 0, 3, 2]
            } else {
                [0, 1, 2, 0, 2, 3]
            };
            m.indices.extend(quad.map(|i| first + i));
        }
    }
    m
}

/// The ground: `size` metres square about the origin at y = 0, of the kit
/// surface `name` repeating every `tile` metres.
pub fn ground(scene: &mut RenderScene, name: &str, size: f32, tile: f32) -> MaterialHandle {
    let material = surface(scene, name);
    let mesh = scene.add_mesh(slab(
        Vec2::splat(-size / 2.0),
        Vec2::splat(size / 2.0),
        0.0,
        tile,
    ));
    scene.add_instance(mesh, material, Mat4::IDENTITY);
    material
}

/// The hills' height at world xz: nothing inside `HILLS_FOOT` metres of the
/// origin (the scene's own ground lies there), then rolling up to about
/// 100 m by `HILLS_REACH`, except towards `open` (a unit bearing in xz),
/// where the land stays low over 40° either side: the horizon a low sun
/// stands on.
pub fn hills_height(x: f32, z: f32, open: Option<Vec2>) -> f32 {
    let p = Vec2::new(x, z);
    let d = p.length();
    let rolling = {
        let (mut v, mut amp, mut f) = (0.0, 0.5, 1.0 / 260.0);
        for o in 0..5 {
            v += noise2(x * f + 31.0, z * f + 17.0, 71 + o) * amp;
            f *= 2.07;
            amp *= 0.5;
        }
        v
    };
    let gap = open.map_or(1.0, |o| {
        let along = p.normalize_or_zero().dot(o.normalize_or_zero());
        // cos 40° to cos 65°: low inside, the hills' own height outside.
        1.0 - smooth(0.42, 0.77, along)
    });
    -2.0 + smooth(HILLS_FOOT - 40.0, HILLS_FOOT + 40.0, d) * 5.0
        + smooth(HILLS_FOOT, HILLS_REACH, d) * (18.0 + rolling * 150.0) * (0.06 + 0.94 * gap)
}

/// Where the hills leave the ground and where they are at their height.
pub const HILLS_FOOT: f32 = 190.0;
pub const HILLS_REACH: f32 = 620.0;
/// The hills' extent: a square this many metres across.
pub const HILLS_SIZE: f32 = 2400.0;

/// The hills around a scene's ground ([`hills_height`]): a ring of land
/// from the ground's edge to the horizon, of the kit's grass, so the view
/// ends in country instead of at the ground's edge.
/// Its surface repeats every 37 m (far enough apart that the repeat does not
/// read from across a valley), darker and lighter in patches a few hundred
/// metres across.
pub fn hills(scene: &mut RenderScene, open: Option<Vec2>) {
    const N: u32 = 121;
    let cell = HILLS_SIZE / (N - 1) as f32;
    let at = |c: u32, r: u32| {
        Vec2::new(
            -HILLS_SIZE * 0.5 + c as f32 * cell,
            -HILLS_SIZE * 0.5 + r as f32 * cell,
        )
    };
    let h = |p: Vec2| hills_height(p.x, p.y, open);
    let mut vertices = Vec::with_capacity((N * N) as usize);
    for r in 0..N {
        for c in 0..N {
            let p = at(c, r);
            let e = cell * 0.5;
            let normal = Vec3::new(
                h(p - Vec2::X * e) - h(p + Vec2::X * e),
                2.0 * e,
                h(p - Vec2::Y * e) - h(p + Vec2::Y * e),
            )
            .normalize();
            // Woods and open slopes: patches of shade in the land's colour.
            let wood = smooth(
                0.42,
                0.62,
                noise2(p.x / 170.0 + 9.0, p.y / 170.0 + 4.0, 0x57),
            );
            let fine = noise2(p.x / 45.0, p.y / 45.0, 0x58);
            let shade = (0.98 - 0.36 * wood) * (0.86 + 0.28 * fine);
            vertices.push(Vertex {
                position: [p.x, h(p), p.y],
                normal: normal.to_array(),
                uv: [p.x / 37.0, p.y / 37.0],
                color: [shade * 0.96, shade, shade * 0.9, 1.0],
                tangent: [0.0; 4],
            });
        }
    }
    let mut indices = Vec::new();
    for r in 0..N - 1 {
        for c in 0..N - 1 {
            // Cells wholly under the scene's own ground are left out.
            let inside = [at(c, r), at(c + 1, r), at(c, r + 1), at(c + 1, r + 1)]
                .iter()
                .all(|p| p.length() < HILLS_FOOT - 45.0);
            if inside {
                continue;
            }
            let i = r * N + c;
            indices.extend_from_slice(&[i, i + N, i + 1, i + 1, i + N, i + N + 1]);
        }
    }
    let grass = surface_tinted(scene, "grass", [0.78, 1.0, 0.62]);
    let mesh = scene.add_mesh(MeshData { vertices, indices });
    scene.add_instance(mesh, grass, Mat4::IDENTITY);
    // Country beyond the shadows' reach: as a caster it would only stretch
    // the cascades' bounds to the horizon.
    if let Some(land) = scene.objects.instances.last_mut() {
        land.cast_shadows = false;
    }
}
